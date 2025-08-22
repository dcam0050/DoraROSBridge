use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Audio playback manager that handles continuous audio streaming
pub struct AudioPlayback {
    audio_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    stream: Option<cpal::Stream>,
}

impl AudioPlayback {
    /// Create a new audio playback instance
    pub fn new() -> eyre::Result<Self> {
        let audio_queue = Arc::new(Mutex::new(VecDeque::new()));
        Ok(Self {
            audio_queue,
            stream: None,
        })
    }

    /// Get a reference to the audio queue for adding audio data
    pub fn get_audio_queue(&self) -> Arc<Mutex<VecDeque<Vec<u8>>>> {
        self.audio_queue.clone()
    }

    /// Start audio playback
    pub fn start(&mut self) -> eyre::Result<()> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| eyre::eyre!("No output device available"))?;
        
        println!("Using audio device: {}", device.name()?);
        
        let config = device.default_output_config()?;
        println!("Default config: {:?}", config);
        
        let sample_rate = 48000;
        let _channels = 1; // Mono audio
        
        let config = cpal::StreamConfig {
            channels: config.channels(),
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        let output_channels: usize = config.channels as usize;
        
        // Create a continuous audio buffer
        let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));
        let audio_buffer_clone = audio_buffer.clone();
        let audio_queue = self.audio_queue.clone();
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut queue_guard = audio_queue.lock().unwrap();
                let mut buffer_guard = audio_buffer_clone.lock().unwrap();
                
                // How many frames (mono samples) are needed for this callback
                let frames_needed = data.len() / output_channels;
                let min_prefill_frames = (sample_rate as usize) / 10; // ~100ms prebuffer
                
                // Process audio packets until we have enough mono samples for the output frames
                while buffer_guard.len() < frames_needed && !queue_guard.is_empty() {
                    if let Some(audio_data) = queue_guard.pop_front() {
                        let samples: Vec<f32> = audio_data
                            .chunks_exact(2)
                            .map(|chunk| {
                                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                                sample as f32 / 32768.0 // Normalize to [-1.0, 1.0]
                            })
                            .collect();
                        buffer_guard.extend(samples);
                    }
                }

                // If we still do not have enough and the buffer is not yet prefllled, output silence to avoid glitches
                if buffer_guard.len() < frames_needed && buffer_guard.len() < min_prefill_frames {
                    for sample in data.iter_mut() { *sample = 0.0; }
                    return;
                }
                
                // Write interleaved frames, duplicating mono to all output channels
                let frames_to_write = frames_needed;
                for frame_index in 0..frames_to_write {
                    let sample_value = buffer_guard.get(frame_index).copied().unwrap_or(0.0);
                    let base = frame_index * output_channels;
                    for ch in 0..output_channels {
                        data[base + ch] = sample_value;
                    }
                }
                
                // Remove used mono frames from buffer
                let frames_removed = frames_to_write.min(buffer_guard.len());
                for _ in 0..frames_removed {
                    buffer_guard.pop_front();
                }
                
                // Keep buffer size reasonable (max 5 seconds of audio)
                let max_buffer_samples = sample_rate as usize * 5;
                while buffer_guard.len() > max_buffer_samples {
                    buffer_guard.pop_front();
                }
            },
            |err| eprintln!("Audio playback error: {}", err),
            None,
        )?;
        
        stream.play()?;
        println!("Audio playback started");
        
        self.stream = Some(stream);
        Ok(())
    }

    /// Stop audio playback
    pub fn stop(&mut self) {
        self.stream = None;
    }

    /// Keep the playback running (call this in a loop)
    pub fn keep_alive(&self) {
        if self.stream.is_some() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

impl Drop for AudioPlayback {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convenience function to run audio playback in a separate thread
pub fn run_audio_playback_thread(
    audio_queue: Arc<Mutex<VecDeque<Vec<u8>>>>, 
    shutdown_signal: Arc<AtomicBool>,
    input_sample_rate_hz: u32,
) -> eyre::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or_else(|| eyre::eyre!("No output device available"))?;
    
    println!("Using audio device: {}", device.name()?);
    
    let supported = device.default_output_config()?;
    println!("Default config: {:?}", supported);
    
    let device_rate_hz: u32 = supported.sample_rate().0;
    let config = cpal::StreamConfig {
        channels: supported.channels(),
        sample_rate: cpal::SampleRate(device_rate_hz),
        buffer_size: cpal::BufferSize::Default,
    };
    let output_channels: usize = config.channels as usize;
    
    // Create a continuous audio buffer
    let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let audio_buffer_clone = audio_buffer.clone();
    
    // Clone the audio queue for the closure
    let audio_queue_clone = audio_queue.clone();
    
    // Simple linear resampler from input rate to device rate for mono f32 samples
    let resample = move |src: &[f32], dst: &mut Vec<f32>| {
        if input_sample_rate_hz == device_rate_hz {
            dst.extend_from_slice(src);
            return;
        }
        if src.is_empty() { return; }
        let ratio = device_rate_hz as f32 / input_sample_rate_hz as f32;
        let out_len = (src.len() as f32 * ratio).round() as usize;
        let mut pos = 0.0f32;
        for _ in 0..out_len {
            let i = pos.floor() as usize;
            let frac = pos - i as f32;
            let s0 = src.get(i).copied().unwrap_or(*src.last().unwrap());
            let s1 = src.get(i+1).copied().unwrap_or(s0);
            dst.push(s0 + (s1 - s0) * frac);
            pos += 1.0/ratio;
        }
    };

    let build_f32 = || -> eyre::Result<cpal::Stream> {
        let resample_cb = resample.clone();
        let audio_queue_clone = audio_queue_clone.clone();
        let audio_buffer_clone = audio_buffer_clone.clone();
        Ok(device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut queue_guard = audio_queue_clone.lock().unwrap();
                let mut buffer_guard = audio_buffer_clone.lock().unwrap();
                
                let frames_needed = data.len() / output_channels;
                let min_prefill_frames = (device_rate_hz as usize) / 10; // ~100ms
                
                while buffer_guard.len() < frames_needed && !queue_guard.is_empty() {
                    if let Some(audio_data) = queue_guard.pop_front() {
                        let mono: Vec<f32> = audio_data
                            .chunks_exact(2)
                            .map(|chunk| {
                                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                                sample as f32 / 32768.0
                            })
                            .collect();
                        let mut resampled = Vec::with_capacity(mono.len() * 2);
                        resample_cb(&mono, &mut resampled);
                        buffer_guard.extend(resampled);
                    }
                }
                if buffer_guard.len() < frames_needed && buffer_guard.len() < min_prefill_frames {
                    for sample in data.iter_mut() { *sample = 0.0; }
                    return;
                }
                let frames_to_write = frames_needed;
                for frame_index in 0..frames_to_write {
                    let sample_value = buffer_guard.get(frame_index).copied().unwrap_or(0.0);
                    let base = frame_index * output_channels;
                    for ch in 0..output_channels {
                        data[base + ch] = sample_value;
                    }
                }
                let frames_removed = frames_to_write.min(buffer_guard.len());
                for _ in 0..frames_removed { buffer_guard.pop_front(); }
                let max_buffer_samples = device_rate_hz as usize * 5;
                while buffer_guard.len() > max_buffer_samples { buffer_guard.pop_front(); }
            },
            |err| eprintln!("Audio playback error: {}", err),
            None,
        )?)
    };

    let build_i16 = || -> eyre::Result<cpal::Stream> {
        let resample_cb = resample.clone();
        let audio_queue_clone = audio_queue_clone.clone();
        let audio_buffer_clone = audio_buffer_clone.clone();
        Ok(device.build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut queue_guard = audio_queue_clone.lock().unwrap();
                let mut buffer_guard = audio_buffer_clone.lock().unwrap();
                
                let frames_needed = data.len() / output_channels;
                let min_prefill_frames = (device_rate_hz as usize) / 10;
                
                while buffer_guard.len() < frames_needed && !queue_guard.is_empty() {
                    if let Some(audio_data) = queue_guard.pop_front() {
                        let mono: Vec<f32> = audio_data
                            .chunks_exact(2)
                            .map(|chunk| {
                                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                                sample as f32 / 32768.0
                            })
                            .collect();
                        let mut resampled = Vec::with_capacity(mono.len() * 2);
                        resample_cb(&mono, &mut resampled);
                        buffer_guard.extend(resampled);
                    }
                }
                if buffer_guard.len() < frames_needed && buffer_guard.len() < min_prefill_frames {
                    for sample in data.iter_mut() { *sample = 0; }
                    return;
                }
                let frames_to_write = frames_needed;
                for frame_index in 0..frames_to_write {
                    let sample_value_f32 = buffer_guard.get(frame_index).copied().unwrap_or(0.0);
                    let sample_i16 = (sample_value_f32.clamp(-1.0, 1.0) * 32767.0) as i16;
                    let base = frame_index * output_channels;
                    for ch in 0..output_channels { data[base + ch] = sample_i16; }
                }
                let frames_removed = frames_to_write.min(buffer_guard.len());
                for _ in 0..frames_removed { buffer_guard.pop_front(); }
                let max_buffer_samples = device_rate_hz as usize * 5;
                while buffer_guard.len() > max_buffer_samples { buffer_guard.pop_front(); }
            },
            |err| eprintln!("Audio playback error: {}", err),
            None,
        )?)
    };

    let stream = match supported.sample_format() {
        SampleFormat::F32 => build_f32()?,
        SampleFormat::I16 => build_i16()?,
        SampleFormat::U16 => build_i16()?,
        _ => build_f32()?,
    };
    
    stream.play()?;
    println!("Audio playback started");
    
    // Keep the playback running with proper shutdown handling
    while !shutdown_signal.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("Audio playback thread shutting down");
    Ok(())
}
