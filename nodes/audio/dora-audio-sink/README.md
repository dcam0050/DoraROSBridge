# Dora Audio Sink

A Dora node for receiving and analyzing audio data from the gstreamer-audio-receiver. This node can optionally play back audio and provide detailed debug information.

## Features

- **Audio Reception**: Receives audio data from gstreamer-audio-receiver via Dora dataflow
- **Audio Playback**: Optional real-time audio playback using cpal
- **Debug Analysis**: Optional detailed audio analysis and statistics
- **JSON Export**: Optional export of debug data to JSON files
- **Production Ready**: Clean, minimal output in production mode

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLE_PLAYBACK` | `true` | Enable/disable audio playback |
| `ENABLE_DEBUG` | `false` | Enable/disable detailed debug output |
| `DEBUG_MAX_ENTRIES` | `100` | Maximum number of debug entries to keep in memory |
| `DEBUG_FILE` | `audio_debug.json` | File path for debug data export |

## Usage

### Production Mode (Default)
```bash
# Run with minimal output
dora run ./dataflow.audio.yml
```

### Debug Mode
```bash
# Enable debug output and playback
ENABLE_DEBUG=true ENABLE_PLAYBACK=true dora run ./dataflow.audio.yml
```

### Audio Analysis Only
```bash
# Enable debug output without playback
ENABLE_DEBUG=true ENABLE_PLAYBACK=false dora run ./dataflow.audio.yml
```

## Debug Output

When `ENABLE_DEBUG=true`, the node provides detailed information about each audio packet:

- Timestamp
- Data length and format
- Sample rate and channels
- First and last 16 bytes (hex)
- Audio statistics (min, max, average, RMS, zero crossings)
- Sample rate calculations and comparisons

## Audio Playback

When `ENABLE_PLAYBACK=true`, the node will:

- Initialize the default audio output device
- Play received audio in real-time
- Handle S16LE audio format (most common)
- Normalize audio to [-1.0, 1.0] range
- Maintain a small buffer for smooth playback

## Dataflow Integration

This node is designed to work with the gstreamer-audio-receiver in the audio pipeline:

```
gstreamer-audio-receiver → dora-audio-sink
                      ↓
              ros2-audio-publisher
```

## Building

```bash
cargo build -p dora-audio-sink
```

## Dependencies

- `dora-node-api`: Dora framework integration
- `cpal`: Cross-platform audio I/O
- `serde`: JSON serialization for debug data
- `eyre`: Error handling
