# Audio Streaming System

This system allows you to stream microphone audio from a robot to ROS2 using GStreamer and Dora nodes.

## Architecture

The audio streaming system consists of three main components:

1. **GStreamer Audio Sender** (on the robot): Captures audio from the robot's microphone and streams it via UDP RTP
2. **GStreamer Audio Receiver** (Dora node): Receives audio data from the robot via UDP and forwards it to Dora
3. **ROS2 Audio Publisher** (Dora node): Publishes audio data to ROS2 topics

## Prerequisites

### On the Robot (Remote System)
- GStreamer 1.0 with plugins: `gstreamer1.0-tools`, `gstreamer1.0-plugins-base`, `gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, `gstreamer1.0-plugins-ugly`
- PulseAudio: `pulseaudio`
- SSH access with key-based authentication

### On the Local System
- Dora Dataflow
- ROS2 environment
- GStreamer development libraries (for building the receiver node)

## Configuration

### Network Configuration
Edit `python_helpers/deploy_and_run_audio_sender.sh` and update:
- `REMOTE_USER`: Username on the robot
- `REMOTE_HOST`: Hostname/IP of the robot
- `LOCAL_IP`: IP address of the local machine receiving audio
- `UDP_PORT`: UDP port for audio streaming (default: 5004)

### Audio Configuration
The system is configured for:
- Sample rate: 48kHz
- Channels: 1 (mono)
- Format: S16LE (16-bit signed little-endian)
- Protocol: RTP L16 payload over UDP

## Usage

### 1. Build the Audio Nodes

```bash
# Build both audio nodes
npm run build:audio

# Or build individually
cargo build -p gstreamer-audio-receiver
cargo build -p ros2-audio-publisher
```

### 2. Start the Audio Streaming System

```bash
# Start the Dora dataflow with audio nodes
npm run start:audio
```

This will start:
- GStreamer audio receiver listening on UDP port 5004
- ROS2 audio publisher publishing to `/robot/audio` topic

### 3. Deploy and Start Audio Sender on Robot

```bash
# Deploy and run the audio sender on the robot
npm run audio:deploy
```

This will:
- Create an audio sender script on the robot
- Start streaming audio from the robot's microphone
- Stream audio to the local machine via UDP

### 4. Monitor and Control

```bash
# View logs
dora logs gstreamer-audio-receiver
dora logs ros2-audio-publisher

# Stop the system
dora stop
```

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

### Receiver Pipeline (in Dora node)
```bash
udpsrc port=5004 caps="application/x-rtp,media=audio,encoding-name=L16,clock-rate=48000,channels=1,payload=96" \
  ! rtpjitterbuffer latency=10 \
  ! rtpL16depay \
  ! audioconvert \
  ! appsink name=appsink sync=false
```

## ROS2 Topics

The system publishes audio data to the `/robot/audio` topic as `std_msgs/UInt8MultiArray` messages.

### Subscribing to Audio in ROS2
```bash
# Listen to audio data
ros2 topic echo /robot/audio

# Check topic info
ros2 topic info /robot/audio
```

## Troubleshooting

### Common Issues

1. **GStreamer not found on robot**
   ```bash
   sudo apt-get install gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
   ```

2. **PulseAudio not found on robot**
   ```bash
   sudo apt-get install pulseaudio
   ```

3. **SSH connection fails**
   - Ensure SSH key-based authentication is set up
   - Check that the robot is reachable on the network

4. **Audio not being received**
   - Check firewall settings on both machines
   - Verify the LOCAL_IP is correct
   - Ensure the UDP port is not blocked

5. **GStreamer pipeline errors**
   - Check that all required GStreamer plugins are installed
   - Verify audio device permissions on the robot

### Debugging

1. **Test GStreamer on robot manually**:
   ```bash
   ssh user@robot "gst-launch-1.0 -v pulsesrc ! fakesink"
   ```

2. **Test network connectivity**:
   ```bash
   # From robot to local machine
   ssh user@robot "nc -u local_ip 5004 < /dev/null"
   ```

3. **Check audio devices on robot**:
   ```bash
   ssh user@robot "pactl list sources short"
   ```

## Customization

### Changing Audio Format
To change the audio format, modify:
1. The GStreamer pipeline in `deploy_and_run_audio_sender.sh`
2. The environment variables in `dataflow.audio.yml`
3. The caps in the receiver node

### Adding Audio Processing
You can add audio processing by:
1. Adding GStreamer elements to the pipeline
2. Creating additional Dora nodes for processing
3. Modifying the ROS2 message format

### Multiple Audio Streams
To handle multiple audio streams:
1. Use different UDP ports for each stream
2. Create separate Dora nodes for each stream
3. Use different ROS2 topics for each stream
