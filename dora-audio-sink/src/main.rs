use dora_node_api::{self, DoraNode, Event, Parameter};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use common_audio_playback::run_audio_playback_thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AudioDebugInfo {
    timestamp: std::time::SystemTime,
    data_length: usize,
    sample_rate: String,
    channels: String,
    format: String,
    first_bytes: Vec<u8>,
    last_bytes: Vec<u8>,
    data_stats: AudioDataStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AudioDataStats {
    min_value: i16,
    max_value: i16,
    avg_value: f64,
    zero_crossings: usize,
    rms: f64,
}

fn main() -> eyre::Result<()> {
    println!("Starting Dora audio sink debug node with audio playback");

    // Configuration
    let enable_debug = std::env::var("ENABLE_DEBUG")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    let max_debug_entries = if enable_debug {
        std::env::var("DEBUG_MAX_ENTRIES")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100)
    } else {
        0
    };

    let debug_file = std::env::var("DEBUG_FILE").unwrap_or_else(|_| "audio_debug.json".to_string());
    let enable_playback = std::env::var("ENABLE_PLAYBACK")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    // Store recent debug info for analysis (only if debug is enabled)
    let mut debug_history: VecDeque<AudioDebugInfo> = VecDeque::with_capacity(max_debug_entries);
    let mut total_packets = 0;
    let mut total_bytes = 0;

    // Create shutdown signal for audio playback thread
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = shutdown_signal.clone();

    // Initialize audio playback (deferred start until first packet's metadata)
    let audio_queue = Arc::new(Mutex::new(VecDeque::new()));
    let mut audio_thread_handle: Option<thread::JoinHandle<()>> = None;
    let mut playback_started = false;

    // Set up signal handler for graceful shutdown
    let shutdown_signal_for_handler = shutdown_signal.clone();
    ctrlc::set_handler(move || {
        println!("Received interrupt signal, shutting down gracefully...");
        shutdown_signal_for_handler.store(true, Ordering::Relaxed);
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // --- Dora: init and process events ------------------------------------------------------
    let (_node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    println!("Dora audio sink debug node ready. Listening for audio data...");

    while let Some(event) = events.next() {
        // Check if shutdown signal is set
        if shutdown_signal.load(Ordering::Relaxed) {
            println!("Shutdown signal received, stopping...");
            break;
        }

        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "audio" => {
                    total_packets += 1;
                    
                    // Extract audio data from Arrow format
                    let audio_data: Vec<u8> = (&data).try_into()
                        .map_err(|e| eyre::eyre!("Failed to convert data to bytes: {e}"))?;
                    
                    total_bytes += audio_data.len();
                    
                    // Extract metadata
                    let sample_rate = metadata.parameters.get("sample_rate")
                        .and_then(|p| match p {
                            Parameter::String(sr) => Some(sr.clone()),
                            Parameter::Integer(sr) => Some(sr.to_string()),
                            _ => None
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let channels = metadata.parameters.get("channels")
                        .and_then(|p| match p {
                            Parameter::String(ch) => Some(ch.clone()),
                            Parameter::Integer(ch) => Some(ch.to_string()),
                            _ => None
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let format = metadata.parameters.get("format")
                        .and_then(|p| match p {
                            Parameter::String(fmt) => Some(fmt.clone()),
                            _ => None
                        })
                        .unwrap_or_else(|| "unknown".to_string());

                    // Start playback thread on first packet if enabled
                    if enable_playback && !playback_started {
                        let sr_hz: u32 = sample_rate.parse().unwrap_or(48000);
                        let queue_clone = audio_queue.clone();
                        let shutdown_signal_clone2 = shutdown_signal_clone.clone();
                        println!("Starting audio playback thread at {} Hz", sr_hz);
                        audio_thread_handle = Some(thread::spawn(move || {
                            if let Err(e) = run_audio_playback_thread(queue_clone, shutdown_signal_clone2, sr_hz) {
                                eprintln!("Audio playback error: {}", e);
                            }
                        }));
                        playback_started = true;
                    }
                    
                    // Analyze audio data
                    let data_stats = analyze_audio_data(&audio_data, &format);
                    
                    // Create debug info
                    let debug_info = AudioDebugInfo {
                        timestamp: std::time::SystemTime::now(),
                        data_length: audio_data.len(),
                        sample_rate: sample_rate.clone(),
                        channels: channels.clone(),
                        format: format.clone(),
                        first_bytes: audio_data.iter().take(16).cloned().collect(),
                        last_bytes: audio_data.iter().rev().take(16).cloned().collect(),
                        data_stats,
                    };
                    
                    // Convert to S16LE mono using metadata and send to playback queue
                    if enable_playback {
                        let channels_num: usize = channels.parse().unwrap_or(1);
                        let converted = to_s16le_mono(&audio_data, &format, channels_num);
                        if let Ok(mut guard) = audio_queue.lock() {
                            guard.push_back(converted);
                            while guard.len() > 100 { guard.pop_front(); }
                        }
                    }
                    
                    // Debug output (only if enabled)
                    if enable_debug {
                        // Add to history
                        if debug_history.len() >= max_debug_entries {
                            debug_history.pop_front();
                        }
                        debug_history.push_back(debug_info.clone());
                        
                        // Print detailed debug information
                        println!("=== Audio Packet #{} ===", total_packets);
                        println!("Timestamp: {:?}", debug_info.timestamp);
                        println!("Data length: {} bytes", debug_info.data_length);
                        println!("Sample rate: {}Hz", debug_info.sample_rate);
                        println!("Channels: {}", debug_info.channels);
                        println!("Format: {}", debug_info.format);
                        println!("First 16 bytes: {:02x?}", debug_info.first_bytes);
                        println!("Last 16 bytes: {:02x?}", debug_info.last_bytes);
                        println!("Data stats: {:?}", debug_info.data_stats);
                        println!("Total packets: {}, Total bytes: {}", total_packets, total_bytes);
                        println!("Average packet size: {:.2} bytes", total_bytes as f64 / total_packets as f64);
                        
                        // Calculate expected vs actual sample rate
                        let bytes_per_sample = match debug_info.format.as_str() {
                            "S16LE" => 2,
                            "S32LE" => 4,
                            "F32LE" => 4,
                            "S8" => 1,
                            "U8" => 1,
                            _ => 2,
                        };
                        let samples_per_packet = debug_info.data_length / bytes_per_sample;
                        let expected_sample_rate = debug_info.sample_rate.parse::<f64>().unwrap_or(48000.0);
                        let actual_sample_rate = if total_packets > 1 {
                            let time_diff = debug_info.timestamp.duration_since(debug_history.front().unwrap().timestamp).unwrap_or_default();
                            if time_diff.as_millis() > 0 {
                                (samples_per_packet as f64 * 1000.0) / time_diff.as_millis() as f64
                            } else {
                                expected_sample_rate
                            }
                        } else {
                            expected_sample_rate
                        };
                        
                        println!("Samples per packet: {}", samples_per_packet);
                        println!("Expected sample rate: {:.0}Hz", expected_sample_rate);
                        println!("Calculated sample rate: {:.0}Hz", actual_sample_rate);
                        println!("Sample rate difference: {:.0}Hz", (actual_sample_rate - expected_sample_rate).abs());
                        println!("");
                        
                        // Save debug info to file periodically
                        if total_packets % 10 == 0 {
                            if let Ok(json) = serde_json::to_string_pretty(&debug_history) {
                                if let Err(e) = std::fs::write(&debug_file, json) {
                                    eprintln!("Failed to write debug file: {}", e);
                                } else {
                                    println!("Debug info saved to {}", debug_file);
                                }
                            }
                        }
                    } else {
                        // Simple status output for production
                        if total_packets % 100 == 0 {
                            println!("Audio sink: Received {} packets, {} total bytes", total_packets, total_bytes);
                        }
                    }
                }
                other => eprintln!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => {
                println!("Received stop signal");
                break;
            }
            other => eprintln!("Received unexpected input: {other:?}"),
        }
    }

    // Cleanup
    println!("Cleaning up Dora audio sink...");
    
    // Signal audio playback thread to stop
    shutdown_signal.store(true, Ordering::Relaxed);
    
    // Wait for audio playback thread to finish
    if let Some(handle) = audio_thread_handle {
        println!("Waiting for audio playback thread to finish...");
        if let Err(e) = handle.join() {
            eprintln!("Error joining audio playback thread: {:?}", e);
        }
    }

    // Save final debug info (only if debug is enabled)
    if enable_debug && !debug_history.is_empty() {
        if let Ok(json) = serde_json::to_string_pretty(&debug_history) {
            if let Err(e) = std::fs::write(&debug_file, json) {
                eprintln!("Failed to write final debug file: {}", e);
            } else {
                println!("Final debug info saved to {}", debug_file);
            }
        }
    }

    println!("Dora audio sink debug node stopped");
    Ok(())
}

fn analyze_audio_data(audio_data: &[u8], format: &str) -> AudioDataStats {
    match format {
        "S16LE" => analyze_s16le_data(audio_data),
        "S32LE" => analyze_s32le_data(audio_data),
        "F32LE" => analyze_f32le_data(audio_data),
        "S8" => analyze_s8_data(audio_data),
        "U8" => analyze_u8_data(audio_data),
        _ => analyze_s16le_data(audio_data), // Default to S16LE
    }
}

// Audio playback is now handled by the common-audio-playback crate

fn to_s16le_mono(input: &[u8], format: &str, channels: usize) -> Vec<u8> {
    match format {
        "S16LE" => {
            if channels <= 1 { return input.to_vec(); }
            // Downmix simple average across channels
            let mut out: Vec<i16> = Vec::with_capacity(input.len() / 2 / channels);
            let mut idx = 0;
            while idx + channels * 2 <= input.len() {
                let mut acc: i32 = 0;
                for ch in 0..channels {
                    let base = idx + ch * 2;
                    let s = i16::from_le_bytes([input[base], input[base+1]]) as i32;
                    acc += s;
                }
                let avg = (acc / channels as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                out.push(avg);
                idx += channels * 2;
            }
            out.iter().flat_map(|s| s.to_le_bytes()).collect()
        }
        "F32LE" => {
            let mut out: Vec<i16> = Vec::with_capacity(input.len() / 4);
            if channels <= 1 {
                for chunk in input.chunks_exact(4) {
                    let f = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let s = (f.clamp(-1.0, 1.0) * 32767.0) as i16;
                    out.push(s);
                }
            } else {
                let mut idx = 0;
                while idx + channels * 4 <= input.len() {
                    let mut acc: f32 = 0.0;
                    for ch in 0..channels {
                        let base = idx + ch * 4;
                        let f = f32::from_le_bytes([input[base], input[base+1], input[base+2], input[base+3]]);
                        acc += f;
                    }
                    let avg = (acc / channels as f32).clamp(-1.0, 1.0);
                    out.push((avg * 32767.0) as i16);
                    idx += channels * 4;
                }
            }
            out.iter().flat_map(|s| s.to_le_bytes()).collect()
        }
        "S32LE" => {
            let mut out: Vec<i16> = Vec::with_capacity(input.len() / 4);
            if channels <= 1 {
                for chunk in input.chunks_exact(4) {
                    let s32 = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    // Scale down from 32-bit to 16-bit
                    out.push((s32 >> 16) as i16);
                }
            } else {
                let mut idx = 0;
                while idx + channels * 4 <= input.len() {
                    let mut acc: i64 = 0;
                    for ch in 0..channels {
                        let base = idx + ch * 4;
                        let s32 = i32::from_le_bytes([input[base], input[base+1], input[base+2], input[base+3]]) as i64;
                        acc += s32;
                    }
                    let avg32 = acc / channels as i64;
                    out.push(((avg32 >> 16) as i32) as i16);
                    idx += channels * 4;
                }
            }
            out.iter().flat_map(|s| s.to_le_bytes()).collect()
        }
        _ => input.to_vec(),
    }
}

fn analyze_s16le_data(audio_data: &[u8]) -> AudioDataStats {
    if audio_data.len() < 2 {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let samples: Vec<i16> = audio_data.chunks(2)
        .map(|chunk| {
            if chunk.len() == 2 {
                i16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                0
            }
        })
        .collect();

    if samples.is_empty() {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let min_value = *samples.iter().min().unwrap_or(&0);
    let max_value = *samples.iter().max().unwrap_or(&0);
    let avg_value = samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64;
    
    let zero_crossings = samples.windows(2)
        .filter(|window| {
            let prev = window[0];
            let curr = window[1];
            (prev < 0 && curr >= 0) || (prev > 0 && curr <= 0)
        })
        .count();
    
    let rms = (samples.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt();

    AudioDataStats {
        min_value,
        max_value,
        avg_value,
        zero_crossings,
        rms,
    }
}

fn analyze_s32le_data(audio_data: &[u8]) -> AudioDataStats {
    if audio_data.len() < 4 {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let samples: Vec<i32> = audio_data.chunks(4)
        .map(|chunk| {
            if chunk.len() == 4 {
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            } else {
                0
            }
        })
        .collect();

    if samples.is_empty() {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let min_value = *samples.iter().min().unwrap_or(&0) as i16;
    let max_value = *samples.iter().max().unwrap_or(&0) as i16;
    let avg_value = samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64;
    
    let zero_crossings = samples.windows(2)
        .filter(|window| {
            let prev = window[0];
            let curr = window[1];
            (prev < 0 && curr >= 0) || (prev > 0 && curr <= 0)
        })
        .count();
    
    let rms = (samples.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt();

    AudioDataStats {
        min_value,
        max_value,
        avg_value,
        zero_crossings,
        rms,
    }
}

fn analyze_f32le_data(audio_data: &[u8]) -> AudioDataStats {
    if audio_data.len() < 4 {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let samples: Vec<f32> = audio_data.chunks(4)
        .map(|chunk| {
            if chunk.len() == 4 {
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            } else {
                0.0
            }
        })
        .collect();

    if samples.is_empty() {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let min_value = (samples.iter().fold(f32::INFINITY, |a, &b| a.min(b)) * i16::MAX as f32) as i16;
    let max_value = (samples.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) * i16::MAX as f32) as i16;
    let avg_value = samples.iter().sum::<f32>() as f64 / samples.len() as f64;
    
    let zero_crossings = samples.windows(2)
        .filter(|window| {
            let prev = window[0];
            let curr = window[1];
            (prev < 0.0 && curr >= 0.0) || (prev > 0.0 && curr <= 0.0)
        })
        .count();
    
    let rms = (samples.iter().map(|&x| x.powi(2)).sum::<f32>() / samples.len() as f32).sqrt() as f64;

    AudioDataStats {
        min_value,
        max_value,
        avg_value,
        zero_crossings,
        rms,
    }
}

fn analyze_s8_data(audio_data: &[u8]) -> AudioDataStats {
    let samples: Vec<i8> = audio_data.iter().map(|&b| b as i8).collect();

    if samples.is_empty() {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let min_value = *samples.iter().min().unwrap_or(&0) as i16;
    let max_value = *samples.iter().max().unwrap_or(&0) as i16;
    let avg_value = samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64;
    
    let zero_crossings = samples.windows(2)
        .filter(|window| {
            let prev = window[0];
            let curr = window[1];
            (prev < 0 && curr >= 0) || (prev > 0 && curr <= 0)
        })
        .count();
    
    let rms = (samples.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt();

    AudioDataStats {
        min_value,
        max_value,
        avg_value,
        zero_crossings,
        rms,
    }
}

fn analyze_u8_data(audio_data: &[u8]) -> AudioDataStats {
    let samples: Vec<u8> = audio_data.to_vec();

    if samples.is_empty() {
        return AudioDataStats {
            min_value: 0,
            max_value: 0,
            avg_value: 0.0,
            zero_crossings: 0,
            rms: 0.0,
        };
    }

    let min_value = (*samples.iter().min().unwrap_or(&0) as i16) - 128;
    let max_value = (*samples.iter().max().unwrap_or(&0) as i16) - 128;
    let avg_value = (samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64) - 128.0;
    
    let zero_crossings = samples.windows(2)
        .filter(|window| {
            let prev = window[0] as i16 - 128;
            let curr = window[1] as i16 - 128;
            (prev < 0 && curr >= 0) || (prev > 0 && curr <= 0)
        })
        .count();
    
    let rms = (samples.iter().map(|&x| ((x as f64) - 128.0).powi(2)).sum::<f64>() / samples.len() as f64).sqrt();

    AudioDataStats {
        min_value,
        max_value,
        avg_value,
        zero_crossings,
        rms,
    }
}
