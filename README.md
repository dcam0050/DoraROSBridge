# ROS1 to ROS2 Bridge Examples

This project demonstrates how to bridge data between ROS1 and ROS2 using Dora dataflow. It includes multiple examples:

## Examples

### 1. Image Bridge
The `ros1-image-source` subscribes to a ROS1 topic publishing `sensor_msgs/Image`, and the `ros2-image-sink` publishes the received images to a ROS2 topic.

### 2. TTS (Text-to-Speech) Bridge
The `ros2-tts-source` subscribes to ROS2 text topics and forwards text to ROS1 TTS topics via the `ros1-tts-sink`.

### 3. Audio Streaming Bridge
The `gstreamer-audio-receiver` receives audio from a robot via UDP RTP and forwards it to ROS2 via the `ros2-audio-publisher`.

## Quick Start

```bash
# 1. Setup the project
npm run setup

# 2. Build ROS1 components (requires Docker)
npm run build:ros1

# 3. Configure your ROS1 environment in dataflow.yml
# 4. Test your setup
npm run test:setup

# 5. Run with image viewer
npm run start:with-viewer
```

## Prerequisites

- Docker (required for building ROS1 components)
- Rust toolchain
- ROS1 master running (e.g., on a remote machine)
- ROS2 Rolling installed locally
- Network access to ROS1 master

## Quick Start

### 1. Test Your Setup
First, verify that everything is properly configured:
```bash
npm run test:setup
```

This will check:
- ROS2 environment and tools
- ROS2 image viewer availability
- Dora CLI availability
- Node binaries are built

### 2. Build the Nodes

**Important**: ROS1 components must be built in Docker due to ROS1 (Noetic) dependencies.

Build the ROS1 source node in Docker:
```bash
npm run build:ros1
```

Build the ROS2 sink node locally:
```bash
npm run build:ros2
```

Build both components:
```bash
npm run build
```

Build only ROS2 components (for development):
```bash
npm run build:local
```

### 3. Configure ROS Environment

Update the ROS1 environment variables in `dataflow.yml` to match your setup:

```yaml
env:
  ROS_MASTER_URI: "http://192.168.30.120:11311"  # Your ROS1 master
  ROS_HOSTNAME: "katana"                         # Your hostname
  ROS_IMAGE_TOPIC: "/xtion/rgb/image_raw"        # Your ROS1 image topic
```

The ROS2 sink will publish to `/camera/image_raw` by default.

### 4. Run the Dataflow

#### Option A: Run with Image Viewer (Recommended)
```bash
# Start the complete pipeline with ROS2 image viewer
npm run start:with-viewer
```

#### Option B: Run with Topic Echo (Quick Test)
```bash
# Start the pipeline and echo one image message
npm run start:with-echo
```

#### Option C: Run Dataflow Only
```bash
# Start just the dataflow
npm run start

# In another terminal, view logs
npm run logs:source  # ROS1 source logs
npm run logs:sink    # ROS2 sink logs
```

## Running with Image Viewers

### ROS2 Image Viewer (rqt_image_view)
```bash
npm run start:with-viewer
```
This command:
1. Starts the Dora dataflow
2. Waits for initialization
3. Launches `rqt_image_view` to display images from `/camera/image_raw`
4. Provides a GUI window showing the live image stream
5. Stops everything when you press Ctrl+C

### Alternative Image Viewers
If you prefer other ROS2 image viewers:

```bash
# Start the dataflow first
npm run start &

# Then use any of these viewers in another terminal:
ros2 run rqt_image_view rqt_image_view /camera/image_raw
# or
ros2 run image_view image_view /camera/image_raw
# or
ros2 run rviz2 rviz2  # Then add Image display and set topic to /camera/image_raw
```

## Dataflow Architecture

```
┌─────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Timer     │───▶│ ros1-image-source│───▶│ ros2-image-sink │
│ (10ms tick) │    │ (ROS1→Dora)      │    │ (Dora→ROS2)     │
└─────────────┘    └──────────────────┘    └─────────────────┘
```

## Node Descriptions

- **ros1-image-source**: Subscribes to ROS1 `/xtion/rgb/image_raw`, converts `sensor_msgs/Image` to Dora format with metadata
- **ros2-image-sink**: Receives Dora image data and publishes it to ROS2 `/camera/image_raw` as `sensor_msgs/Image`

## Testing

### Test Your Setup
```bash
npm run test:setup
```

### Test ROS1 Connection
```bash
npm run test:ros1        # List ROS1 topics
npm run test:ros1:topic  # Echo ROS1 image topic
```

### Test ROS2 Output
```bash
npm run test:ros2        # List ROS2 topics
npm run test:ros2:topic  # Echo ROS2 image topic
```

## Troubleshooting

### Build Issues
- **ROS1 build fails**: Ensure Docker is running and Noetic container can access the workspace
- **ROS2 build fails**: Ensure ROS2 Rolling is sourced (`source /opt/ros/rolling/setup.bash`)

### Runtime Issues
- **ROS1 connection fails**: Verify ROS master is reachable and environment variables are correct
- **ROS2 publishing fails**: Check that ROS2 is properly sourced and network is accessible
- **Image viewer not showing images**: Check that the ROS2 topic `/camera/image_raw` is publishing

### Logs
- Source logs show "sending image: N bytes, WxH, encoding" when frames arrive from ROS1
- Sink logs show "publishing ROS2 image: N bytes, WxH, encoding" when publishing to ROS2

## Development

To modify the nodes:
1. Edit `node/src/main.rs` (ROS1 source) or `ros2-sink/src/main.rs` (ROS2 sink)
2. Rebuild using the appropriate command:
   - `npm run build:ros1` for ROS1 changes
   - `npm run build:ros2` for ROS2 changes
3. Restart the dataflow

## Audio Streaming System

The audio streaming system allows you to stream microphone audio from a robot to ROS2 using GStreamer and Dora nodes.

### Quick Start

```bash
# 1. Build the audio nodes
npm run build:audio

# 2. Start the audio streaming system
npm run start:audio

# 3. Deploy and run the audio sender on the robot
npm run audio:deploy

# 4. Test the system
npm run test:audio
```

### Architecture

```
┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│   Robot         │───▶│ gstreamer-audio-     │───▶│ ros2-audio-         │
│ (GStreamer      │    │ receiver             │    │ publisher           │
│  Sender)        │    │ (UDP RTP → Dora)     │    │ (Dora → ROS2)       │
└─────────────────┘    └──────────────────────┘    └─────────────────────┘
```

### Configuration

Edit `python_helpers/deploy_and_run_audio_sender.sh` to configure:
- `REMOTE_USER`: Username on the robot
- `REMOTE_HOST`: Hostname/IP of the robot  
- `LOCAL_IP`: IP address of the local machine receiving audio
- `UDP_PORT`: UDP port for audio streaming (default: 5004)

### Audio Format

- Sample rate: 48kHz
- Channels: 1 (mono)
- Format: S16LE (16-bit signed little-endian)
- Protocol: RTP L16 payload over UDP

### ROS2 Topics

The system publishes audio data to `/robot/audio` as `std_msgs/UInt8MultiArray` messages.

For more details, see [AUDIO_STREAMING.md](AUDIO_STREAMING.md).

## Files

- `dataflow.yml`: Main dataflow configuration
- `dataflow.audio.yml`: Audio streaming dataflow configuration
- `node/src/main.rs`: ROS1 image subscriber and Dora bridge
- `ros2-sink/src/main.rs`: ROS2 image publisher
- `gstreamer-audio-receiver/src/main.rs`: GStreamer audio receiver
- `ros2-audio-publisher/src/main.rs`: ROS2 audio publisher
- `run-with-viewer.sh`: Script to run pipeline with image viewer
- `test-setup.sh`: Script to test your setup
- `test-audio.sh`: Script to test audio streaming system
- `python_helpers/deploy_and_run_audio_sender.sh`: Audio sender deployment script
- `Dockerfile.noetic`: Docker build environment for ROS1 (optional)
