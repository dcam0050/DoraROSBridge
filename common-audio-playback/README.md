# Common Audio Playback

A shared Rust library for continuous audio playback that can be used by multiple Dora nodes. This component provides smooth, continuous audio streaming with proper buffering to prevent audio dropouts and "one sample per chunk" issues.

## Features

- **Continuous Audio Buffering**: Uses a `VecDeque<f32>` buffer to ensure smooth playback
- **Real-time Processing**: Processes all incoming audio packets and feeds them continuously to the audio device
- **Automatic Buffer Management**: Maintains optimal buffer size (max 5 seconds) to prevent memory issues
- **Cross-platform**: Uses cpal for cross-platform audio I/O
- **Thread-safe**: Uses Arc<Mutex> for safe concurrent access
- **Easy Integration**: Simple API that can be used by any Dora node

## Usage

### Basic Usage

```rust
use common_audio_playback::run_audio_playback_thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// Create audio queue
let audio_queue = Arc::new(Mutex::new(VecDeque::new()));

// Start playback in a separate thread
let queue_clone = audio_queue.clone();
std::thread::spawn(move || {
    if let Err(e) = run_audio_playback_thread(queue_clone) {
        eprintln!("Audio playback error: {}", e);
    }
});

// Add audio data to the queue
if let Ok(mut guard) = audio_queue.lock() {
    guard.push_back(audio_data);
}
```

### Advanced Usage with AudioPlayback Struct

```rust
use common_audio_playback::AudioPlayback;

// Create and start playback
let mut playback = AudioPlayback::new()?;
playback.start()?;

// Get the audio queue for adding data
let audio_queue = playback.get_audio_queue();

// Add audio data
if let Ok(mut guard) = audio_queue.lock() {
    guard.push_back(audio_data);
}

// Keep playback running
loop {
    playback.keep_alive();
}
```

## Audio Format

The component expects S16LE (16-bit signed little-endian) audio data:

- **Sample Rate**: 48kHz (hardcoded for consistency)
- **Channels**: 1 (mono)
- **Format**: S16LE
- **Normalization**: Automatically normalizes to [-1.0, 1.0] range

## Architecture

The component uses a two-stage buffering system:

1. **Input Queue**: `VecDeque<Vec<u8>>` - Stores raw audio packets from the source
2. **Audio Buffer**: `VecDeque<f32>` - Stores normalized, continuous audio samples

### Processing Flow

1. Audio packets are added to the input queue
2. The audio callback processes all available packets
3. S16LE samples are converted to f32 and normalized
4. Samples are added to the continuous audio buffer
5. The output buffer is filled from the continuous buffer
6. Used samples are removed from the buffer
7. Buffer size is maintained (max 5 seconds)

## Integration with Dora Nodes

This component is designed to be used by Dora nodes that need audio playback:

- **gstreamer-audio-receiver**: Receives UDP RTP audio and can play it back
- **dora-audio-sink**: Receives audio from Dora dataflow and can play it back

Both nodes use the same playback logic, ensuring consistent behavior.

## Dependencies

- `cpal = "0.15"`: Cross-platform audio I/O
- `eyre = "0.6"`: Error handling
- Standard library: `Arc`, `Mutex`, `VecDeque`

## Building

```bash
cargo build -p common-audio-playback
```

## Testing

The component is tested through the nodes that use it:

```bash
# Test with gstreamer-audio-receiver
cargo build -p gstreamer-audio-receiver

# Test with dora-audio-sink  
cargo build -p dora-audio-sink

# Test complete audio pipeline
npm run build:audio
```
