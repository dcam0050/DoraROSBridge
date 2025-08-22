use dora_node_api::{self, DoraNode, Event, IntoArrow, MetadataParameters, Parameter, dora_core::config::DataId};
use eyre::{Context, eyre};
use futures::{StreamExt, task::SpawnExt};
use gstreamer as gst;
use gstreamer::prelude::{ElementExt, Cast, GstBinExt};
use gstreamer_app as gst_app;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use common_audio_playback::run_audio_playback_thread;

fn main() -> eyre::Result<()> {
    println!("starting GStreamer audio receiver node");

    // Initialize GStreamer
    gst::init()?;

    // Dora output id for audio data
    let output = DataId::from("audio".to_owned());

    // Get configuration from environment variables
    let udp_port = std::env::var("AUDIO_UDP_PORT").unwrap_or_else(|_| "5004".to_string());
    let sample_rate = std::env::var("AUDIO_SAMPLE_RATE").unwrap_or_else(|_| "48000".to_string());
    let channels = std::env::var("AUDIO_CHANNELS").unwrap_or_else(|_| "1".to_string());
    let encoding_name = std::env::var("AUDIO_ENCODING_NAME").unwrap_or_else(|_| "L16".to_string());
    let payload = std::env::var("AUDIO_PAYLOAD").unwrap_or_else(|_| "96".to_string());
    let clock_rate = std::env::var("AUDIO_CLOCK_RATE").unwrap_or_else(|_| sample_rate.clone());
    let jitter_latency = std::env::var("AUDIO_JITTERBUFFER_LATENCY").unwrap_or_else(|_| "10".to_string());
    let volume = std::env::var("AUDIO_VOLUME").unwrap_or_else(|_| "0.5".to_string());
    let appsink_sync = std::env::var("AUDIO_APPSINK_SYNC").ok().and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
    let appsink_drop = std::env::var("AUDIO_APPSINK_DROP").ok().and_then(|v| v.parse::<bool>().ok()).unwrap_or(true);
    let appsink_max_buffers = std::env::var("AUDIO_APPSINK_MAX_BUFFERS").unwrap_or_else(|_| "100".to_string());
    let force_format = std::env::var("AUDIO_FORCE_FORMAT").unwrap_or_else(|_| "S16LE".to_string());
    let pipeline_override = std::env::var("AUDIO_PIPELINE_OVERRIDE").ok();
    
    println!("Listening on UDP port: {}", udp_port);
    println!("Audio format: {}Hz, {} channels, encoding {} payload {}", sample_rate, channels, encoding_name, payload);

    // Store pending audio buffers to avoid dropping between ticks
    let pending_audio: Arc<Mutex<VecDeque<Vec<u8>>>> = Arc::new(Mutex::new(VecDeque::new()));
    let pending_audio_clone = Arc::clone(&pending_audio);

    // Initialize audio playback
    let enable_playback = std::env::var("ENABLE_PLAYBACK")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    // Create shutdown signal for audio playback thread
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = shutdown_signal.clone();
    
    // Store audio playback thread handle for proper cleanup
    let audio_queue_for_playback = if enable_playback {
        Some(Arc::new(Mutex::new(VecDeque::new())))
    } else {
        None
    };
    
    let audio_thread_handle = if enable_playback {
        println!("Initializing audio playback...");
        let queue_clone = audio_queue_for_playback.as_ref().unwrap().clone();
        let input_rate_hz: u32 = sample_rate.parse().unwrap_or(48000);
        
        // Start audio playback thread using common component
        Some(thread::spawn(move || {
            if let Err(e) = run_audio_playback_thread(queue_clone, shutdown_signal_clone, input_rate_hz) {
                eprintln!("Audio playback error: {}", e);
            }
        }))
    } else {
        None
    };

    // Create GStreamer pipeline for receiving audio
    let pipeline_str = if let Some(override_str) = pipeline_override.clone() {
        println!("Using AUDIO_PIPELINE_OVERRIDE");
        override_str
    } else {
        // Choose depay/decode chain based on encoding name (support common cases)
        let depay_decode = match encoding_name.as_str() {
            // Raw PCM16 over RTP
            "L16" => "rtpL16depay".to_string(),
            // Opus over RTP
            "OPUS" | "opus" => "rtpopusdepay ! opusdec".to_string(),
            // A-law / mu-law (best-effort)
            "PCMA" => "rtppcmadepay ! alawdec".to_string(),
            "PCMU" => "rtppcmudepay ! mulawdec".to_string(),
            other => {
                eprintln!("Unknown AUDIO_ENCODING_NAME '{}', defaulting to rtpL16depay", other);
                "rtpL16depay".to_string()
            }
        };

        format!(
            "udpsrc port={} caps=\"application/x-rtp,media=audio,encoding-name={},clock-rate={},channels={},payload={}\" ! \
             rtpjitterbuffer latency={} ! \
             {} ! \
             audioconvert ! \
             audioresample ! \
             audio/x-raw,format={},rate={},channels={} ! \
             volume volume={} ! \
             appsink name=appsink sync={} drop={} max-buffers={}",
            udp_port, encoding_name, clock_rate, channels, payload,
            jitter_latency,
            depay_decode,
            force_format, sample_rate, channels,
            volume,
            appsink_sync, appsink_drop, appsink_max_buffers,
        )
    };

    println!("Creating GStreamer pipeline: {}", pipeline_str);

    let pipeline = gst::parse_launch(&pipeline_str)
        .context("Failed to create GStreamer pipeline")?;

    // Get the appsink element from the pipeline
    let pipeline_bin = pipeline
        .clone()
        .dynamic_cast::<gst::Bin>()
        .unwrap();
    
    let appsink = pipeline_bin
        .by_name("appsink")
        .expect("Failed to get appsink element");

    // Store actual sample rate from GStreamer
    let actual_sample_rate: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let actual_sample_rate_clone = Arc::clone(&actual_sample_rate);

    // Clone audio queue for callback - use the same queue as the playback thread
    let audio_queue_clone = audio_queue_for_playback.clone();

    // Set up appsink callbacks
    let appsink = appsink.dynamic_cast::<gst_app::AppSink>().unwrap();
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                
                // Get actual sample rate from caps
                if let Some(caps) = sample.caps() {
                    if let Some(structure) = caps.structure(0) {
                        if let Ok(rate) = structure.get::<i32>("rate") {
                            if let Ok(mut guard) = actual_sample_rate_clone.lock() {
                                *guard = Some(rate.to_string());
                            }
                            println!("GStreamer detected sample rate: {}Hz", rate);
                        }
                    }
                }
                
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                let audio_data = map.as_slice().to_vec();
                let audio_len = audio_data.len();
                
                // Queue the audio data
                if let Ok(mut guard) = pending_audio_clone.lock() {
                    guard.push_back(audio_data.clone());
                    // Cap queue size to avoid unbounded growth (keep ~200 buffers)
                    while guard.len() > 200 {
                        guard.pop_front();
                    }
                }
                
                // Send audio data to playback queue
                if let Some(ref queue) = audio_queue_clone {
                    if let Ok(mut guard) = queue.lock() {
                        guard.push_back(audio_data);
                        // Keep queue size reasonable (max 10 packets)
                        while guard.len() > 10 {
                            guard.pop_front();
                        }
                    }
                }
                
                println!("Received audio data: {} bytes", audio_len);
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Start the pipeline
    pipeline.set_state(gst::State::Playing)
        .context("Failed to start GStreamer pipeline")?;

    // Create ThreadPool for GStreamer message handling
    let pool = futures::executor::ThreadPool::new()?;
    
    // Background task to handle GStreamer messages
    let pipeline_clone = pipeline.clone();
    let shutdown_signal_for_gst = shutdown_signal.clone();
    pool.spawn(async move {
        let bus = pipeline_clone.bus().unwrap();
        loop {
            // Check shutdown signal before processing messages
            if shutdown_signal_for_gst.load(Ordering::Relaxed) {
                println!("GStreamer message handler shutting down");
                break;
            }
            
            // Use a timeout to avoid blocking indefinitely
            if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
                match msg.view() {
                    gst::MessageView::Error(err) => {
                        eprintln!("GStreamer error: {:?}", err);
                        break;
                    }
                    gst::MessageView::Eos(..) => {
                        println!("GStreamer pipeline reached end of stream");
                        break;
                    }
                    gst::MessageView::StateChanged(state) => {
                        println!("GStreamer state changed: {:?}", state);
                    }
                    _ => {}
                }
            }
        }
        println!("GStreamer message handler thread exiting");
    }).context("failed to spawn GStreamer message handler")?;

    // Set up signal handlers for graceful shutdown (SIGINT and SIGTERM)
    let shutdown_signal_for_handler = shutdown_signal.clone();
    
    // Handle SIGINT (Ctrl+C)
    ctrlc::set_handler(move || {
        println!("Received SIGINT (Ctrl+C), shutting down gracefully...");
        shutdown_signal_for_handler.store(true, Ordering::Relaxed);
    }).expect("Error setting SIGINT handler");
    
    // Handle SIGTERM (timeout kills, systemd, docker stop, etc.)
    let shutdown_signal_for_sigterm = shutdown_signal.clone();
    let shutdown_signal_check = shutdown_signal.clone();
    let signal_thread_handle = thread::spawn(move || {
        let mut signals = Signals::new(&[SIGTERM]).expect("Error setting up SIGTERM handler");
        
        loop {
            // Check if we should exit due to other shutdown signal (like SIGINT)
            if shutdown_signal_check.load(Ordering::Relaxed) {
                println!("Signal handler thread detected shutdown signal, exiting...");
                break;
            }
            
            // Check for SIGTERM with a timeout so we don't block forever
            if let Some(sig) = signals.pending().next() {
                match sig {
                    SIGTERM => {
                        println!("Received SIGTERM, shutting down gracefully...");
                        shutdown_signal_for_sigterm.store(true, Ordering::Relaxed);
                        break;
                    }
                    _ => unreachable!(),
                }
            }
            
            // Small delay to avoid busy waiting
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!("Signal handler thread exiting");
    });

    // --- Dora: init and process events ------------------------------------------------------
    let (mut node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    // Forward latest audio to Dora on each tick
    while let Some(event) = events.next() {
        // Check if shutdown signal is set
        if shutdown_signal.load(Ordering::Relaxed) {
            println!("Shutdown signal received, stopping...");
            break;
        }

        match event {
            Event::Input { id, metadata, data: _ } => match id.as_str() {
                "tick" => {
                    // Drain and concatenate all pending audio buffers to avoid gaps
                    let mut combined: Option<Vec<u8>> = None;
                    {
                        if let Ok(mut q) = pending_audio.lock() {
                            if !q.is_empty() {
                                let total_len: usize = q.iter().map(|b| b.len()).sum();
                                let mut buf = Vec::with_capacity(total_len);
                                while let Some(chunk) = q.pop_front() {
                                    buf.extend_from_slice(&chunk);
                                }
                                combined = Some(buf);
                            }
                        }
                    }

                    if let Some(audio_data) = combined {
                        let mut params: MetadataParameters = metadata.parameters;
                        params.insert("length".into(), Parameter::Integer(audio_data.len() as i64));
                        
                        // Use actual detected sample rate if available, otherwise fall back to configured
                        let actual_rate = {
                            let guard = actual_sample_rate.lock().ok();
                            guard.and_then(|g| g.as_ref().cloned()).unwrap_or_else(|| sample_rate.clone())
                        };
                        params.insert("sample_rate".into(), Parameter::String(actual_rate.clone()));
                        params.insert("channels".into(), Parameter::String(channels.clone()));
                        params.insert("format".into(), Parameter::String(force_format.clone()));

                        // Send audio data as Arrow BinaryArray
                        println!("sending audio data: {} bytes with sample rate: {}Hz", audio_data.len(), actual_rate);
                        node.send_output(output.clone(), params, audio_data.into_arrow())?;
                    } else {
                        // No audio received yet; ignore this tick
                    }
                }
                other => eprintln!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => {
                println!("Received stop");
                break;
            }
            other => eprintln!("Received unexpected input: {other:?}"),
        }
    }

    // Cleanup
    println!("Cleaning up GStreamer audio receiver...");
    
    // Signal audio playback thread to stop
    shutdown_signal.store(true, Ordering::Relaxed);
    
    // Wait for audio playback thread to finish
    if let Some(handle) = audio_thread_handle {
        println!("Waiting for audio playback thread to finish...");
        if let Err(e) = handle.join() {
            eprintln!("Error joining audio playback thread: {:?}", e);
        }
    }
    
    // Stop GStreamer pipeline
    println!("Stopping GStreamer pipeline...");
    pipeline.set_state(gst::State::Null)
        .context("Failed to stop GStreamer pipeline")?;
    
    // Wait for pipeline to fully stop
    let _ = pipeline.state(gst::ClockTime::from_seconds(5));
    
    // Explicitly shutdown the ThreadPool
    println!("Shutting down ThreadPool...");
    drop(pool);
    
    // Wait for signal handler thread to finish (with timeout)
    println!("Waiting for signal handler thread to finish...");
    let signal_join_result = signal_thread_handle.join();
    if let Err(e) = signal_join_result {
        eprintln!("Error joining signal handler thread: {:?}", e);
    }
    
    println!("GStreamer audio receiver cleanup complete");

    Ok(())
}

// Audio playback is now handled by the common-audio-playback crate
