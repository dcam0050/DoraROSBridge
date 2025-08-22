# Audio Streaming System

This system allows you to stream microphone audio from a robot to ROS2 using GStreamer and Dora nodes, with robust audio playback capabilities.

## Architecture

The audio streaming system consists of several components:

1. **GStreamer Audio Sender** (on the robot): Captures audio from the robot's microphone and streams it via UDP RTP
2. **GStreamer Audio Receiver** (Dora node): Receives audio data from the robot via UDP and forwards it to Dora with metadata
3. **Dora Audio Sink** (Dora node): Receives audio data and provides local playback using the common audio playback component
4. **ROS2 Audio Publisher** (Dora node): Publishes audio data to ROS2 topics
5. **Common Audio Playback** (shared library): Handles robust audio playback with resampling, prebuffering, and format conversion

## Prerequisites

### On the Robot (Remote System)
- GStreamer 1.0 with plugins: `gstreamer1.0-tools`, `gstreamer1.0-plugins-base`, `gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, `gstreamer1.0-plugins-ugly`
- PulseAudio: `pulseaudio`
- SSH access with key-based authentication

### On the Local System
- Dora Dataflow
- ROS2 environment
- GStreamer development libraries (for building the receiver node)
- ALSA or PulseAudio for audio playback

## Configuration

### Network Configuration
Edit `python_helpers/deploy_and_run_audio_sender.sh` and update:
- `REMOTE_USER`: Username on the robot
- `REMOTE_HOST`: Hostname/IP of the robot
- `LOCAL_IP`: IP address of the local machine receiving audio
- `UDP_PORT`: UDP port for audio streaming (default: 5004)

### Audio Configuration
The system is highly configurable via environment variables in `dataflow.audio.yml`:

#### Basic Audio Settings
```yaml
env:
  AUDIO_UDP_PORT: "5004"
  AUDIO_SAMPLE_RATE: "48000"
  AUDIO_CHANNELS: "1"
  ENABLE_PLAYBACK: "true"
```

#### Advanced Audio Settings (Optional)
```yaml
env:
  # RTP/Encoding configuration
  AUDIO_ENCODING_NAME: "L16"           # e.g. OPUS, L16, PCMA, PCMU
  AUDIO_PAYLOAD: "96"
  AUDIO_CLOCK_RATE: "48000"           # defaults to AUDIO_SAMPLE_RATE
  
  # GStreamer pipeline settings
  AUDIO_JITTERBUFFER_LATENCY: "10"     # milliseconds
  AUDIO_VOLUME: "0.5"
  AUDIO_APPSINK_SYNC: "false"
  AUDIO_APPSINK_DROP: "true"
  AUDIO_APPSINK_MAX_BUFFERS: "100"
  AUDIO_FORCE_FORMAT: "S16LE"          # target raw caps format
  
  # Full pipeline override (optional)
  AUDIO_PIPELINE_OVERRIDE: "..."       # complete custom pipeline string
```

## Usage

### 1. Build the Audio Nodes

```bash
# Build all audio nodes
npm run build:audio

# Or build individually
cargo build -p gstreamer-audio-receiver
cargo build -p dora-audio-sink
cargo build -p ros2-audio-publisher
cargo build -p common-audio-playback
```

### 2. Start the Audio Streaming System

#### Option A: With Local Playback (Recommended)
```bash
# Start with gstreamer-audio-receiver handling playback
npm run start:audio
```

#### Option B: With Dora Audio Sink Playback
```bash
# Edit dataflow.audio.yml to disable receiver playback and enable sink playback
# Then start the dataflow
dora run dataflow.audio.yml
```

### 3. Deploy and Start Audio Sender on Robot

```bash
# Deploy and run the audio sender on the robot
npm run audio:deploy
```

### 4. Monitor and Control

```bash
# View logs
dora logs gstreamer-audio-receiver
dora logs dora-audio-sink
dora logs ros2-audio-publisher

# Stop the system
dora stop
```

## Audio Playback Features

### Automatic Format Conversion
The system automatically converts between various audio formats:
- **S16LE** (16-bit signed little-endian) - Default
- **F32LE** (32-bit float little-endian)
- **S32LE** (32-bit signed little-endian)
- **S8** (8-bit signed)
- **U8** (8-bit unsigned)

### Resampling
- Automatically resamples from input rate to device output rate
- Uses linear interpolation for smooth playback
- Supports common rates: 44.1kHz, 48kHz, 96kHz, etc.

### Channel Handling
- Converts multi-channel audio to mono for playback
- Duplicates mono audio to all output channels
- Supports stereo, 5.1, 7.1, and custom channel configurations

### Buffer Management
- Prebuffers ~100ms of audio to prevent glitches
- Queues multiple audio packets for smooth streaming
- Implements intelligent buffer size management
- Prevents audio dropouts during network jitter

## GStreamer Pipelines

### Sender Pipeline (on robot)
```bash
gst-launch-1.0 -v \
  pulsesrc do-timestamp=true buffer-time=10000 latency-time=5000 \
  ! audio/x-raw,format=S16LE,rate=48000,channels=1 \
  ! audioconvert \
  ! rtpL16pay pt=96 \
  ! udpsink host=192.168.30.110 port=5004
```

### Receiver Pipeline (configurable via env vars)
```bash
udpsrc port=5004 caps="application/x-rtp,media=audio,encoding-name=L16,clock-rate=48000,channels=1,payload=96" \
  ! rtpjitterbuffer latency=10 \
  ! rtpL16depay \
  ! audioconvert \
  ! audioresample \
  ! audio/x-raw,format=S16LE,rate=48000,channels=1 \
  ! volume volume=0.5 \
  ! appsink name=appsink sync=false drop=true max-buffers=100
```

### Supported Encoding Formats
- **L16**: Raw PCM16 over RTP (default)
- **OPUS**: Opus codec over RTP
- **PCMA**: A-law encoding over RTP
- **PCMU**: μ-law encoding over RTP

## ROS2 Topics

The system publishes audio data to the `/robot/audio` topic as `std_msgs/UInt8MultiArray` messages with metadata.

### Subscribing to Audio in ROS2
```bash
# Listen to audio data
ros2 topic echo /robot/audio

# Check topic info
ros2 topic info /robot/audio

# Check topic type
ros2 topic type /robot/audio
```

## Debugging and Monitoring

### Enable Debug Mode
```yaml
env:
  ENABLE_DEBUG: "true"
  DEBUG_MAX_ENTRIES: "100"
  DEBUG_FILE: "audio_debug.json"
```

### Audio Statistics
The system provides detailed audio statistics including:
- Sample rate detection and validation
- Audio level analysis (min, max, RMS)
- Zero-crossing detection
- Packet timing analysis
- Format conversion metrics

## Troubleshooting

### Common Issues

1. **Audio playback not working**
   - Check `ENABLE_PLAYBACK` environment variable
   - Verify audio device permissions
   - Check system volume settings

2. **Audio stuttering or dropouts**
   - Increase `AUDIO_APPSINK_MAX_BUFFERS`
   - Adjust `AUDIO_JITTERBUFFER_LATENCY`
   - Check network connectivity and firewall

3. **Wrong audio pitch**
   - Verify `AUDIO_SAMPLE_RATE` matches sender
   - Check device sample rate compatibility
   - Enable debug mode to see detected rates

4. **GStreamer pipeline errors**
   - Check `AUDIO_ENCODING_NAME` matches sender
   - Verify GStreamer plugins are installed
   - Use `AUDIO_PIPELINE_OVERRIDE` for custom pipelines

### Debugging Commands

```bash
# Test audio device
aplay -l
pactl list sources short

# Test GStreamer playback
gst-launch-1.0 audiotestsrc ! autoaudiosink

# Check network connectivity
nc -u robot_ip 5004 < /dev/null

# Monitor system audio
pavucontrol
```

## Customization

### Custom Audio Formats
```yaml
env:
  AUDIO_ENCODING_NAME: "OPUS"
  AUDIO_FORCE_FORMAT: "F32LE"
  AUDIO_SAMPLE_RATE: "96000"
  AUDIO_CHANNELS: "2"
```

### Custom GStreamer Pipeline
```yaml
env:
  AUDIO_PIPELINE_OVERRIDE: "udpsrc port=5004 ! custom_element ! appsink"
```

### Multiple Audio Streams
To handle multiple audio streams:
1. Use different UDP ports for each stream
2. Create separate Dora nodes for each stream
3. Use different ROS2 topics for each stream
4. Configure unique environment variables for each stream

## Performance Optimization

### Buffer Tuning
```yaml
env:
  AUDIO_APPSINK_MAX_BUFFERS: "200"    # More buffers for high-latency networks
  AUDIO_JITTERBUFFER_LATENCY: "50"    # Higher latency for unstable networks
```

### Quality vs Latency
- Lower latency: `AUDIO_JITTERBUFFER_LATENCY: "5"`
- Higher quality: `AUDIO_JITTERBUFFER_LATENCY: "20"`
- Stable networks: `AUDIO_APPSINK_DROP: "true"`
- Unstable networks: `AUDIO_APPSINK_DROP: "false"`

## Signal Handling

The system includes robust signal handling for graceful shutdown:
- Handles SIGINT (Ctrl+C) and SIGTERM
- Properly cleans up audio streams and threads
- Prevents audio device resource leaks
- Ensures clean shutdown of all components
