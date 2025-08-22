# Dora ROS Bridge

A comprehensive ROS1 to ROS2 bridge system built with Dora Dataflow, supporting multiple data modalities including images, audio, and text-to-speech.

## 🏗️ Project Structure

```
ros_bridge/
├── build/                    # Build system and scripts
│   ├── scripts/             # Build and deployment scripts
│   ├── docker/              # Docker files and configurations
│   └── tools/               # Build tools and utilities
├── nodes/                   # All Dora nodes organized by modality
│   ├── audio/               # Audio-related nodes
│   │   ├── gstreamer-audio-receiver/
│   │   ├── dora-audio-sink/
│   │   ├── ros2-audio-publisher/
│   │   ├── common-audio-playback/
│   │   └── dataflow.audio.yml
│   ├── image/               # Image-related nodes
│   │   ├── ros1-image-source/
│   │   ├── ros2-image-sink/
│   │   └── dataflow.image.yml
│   ├── tts/                 # Text-to-speech nodes
│   │   ├── ros2-tts-source/
│   │   ├── ros1-tts-sink/
│   │   └── dataflow.tts.yml
│   └── common/              # Shared components
├── docs/                    # Documentation
│   ├── AUDIO_STREAMING.md
│   ├── METRICS.md
│   └── README.md
├── python_helpers/          # Python utilities and deployment scripts
├── Cargo.toml              # Root workspace configuration
├── package.json            # NPM configuration and scripts
└── .gitignore
```

## 🚀 Quick Start

### Prerequisites

- **Dora Dataflow**: Install Dora CLI and runtime
- **ROS1 & ROS2**: Install ROS1 and ROS2 environments
- **Docker**: For building ROS1 nodes (optional)
- **GStreamer**: For audio streaming capabilities
- **Rust**: For building Rust-based nodes

### Setup

```bash
# Clone the repository
git clone <repository-url>
cd ros_bridge

# Install dependencies and setup environment
npm run setup

# Build all nodes
npm run build
```

## 📊 Data Modalities

### 🎵 Audio Streaming
Stream microphone audio from robot to local system with robust playback.

```bash
# Build audio nodes
npm run build:audio

# Start audio streaming
npm run start:audio

# Deploy audio sender to robot
npm run audio:deploy
```

**Features:**
- Real-time audio streaming via UDP RTP
- Automatic format conversion and resampling
- Robust buffer management preventing dropouts
- Configurable via environment variables
- Local audio playback with metadata-driven configuration

**Documentation:** [Audio Streaming Guide](docs/AUDIO_STREAMING.md)

### 🖼️ Image Bridge
Bridge ROS1 image topics to ROS2 with optional compression.

```bash
# Build image nodes
npm run build:ros2

# Start image bridge
npm run start:image

# Start with viewer
npm run start:with-viewer
```

**Features:**
- ROS1 to ROS2 image topic bridging
- Configurable compression and quality
- Real-time image streaming
- Optional image viewer integration

### 🗣️ Text-to-Speech
Bridge TTS services between ROS1 and ROS2 systems.

```bash
# Build TTS nodes
npm run build:ros2

# Start TTS bridge
npm run start:tts

# Deploy to remote robot
npm run tts:deploy
```

**Features:**
- ROS2 to ROS1 TTS service bridging
- Remote deployment capabilities
- Configurable TTS parameters

## 🛠️ Build System

### Local Development
```bash
# Build all nodes
npm run build

# Build specific modality
npm run build:audio
npm run build:ros2

# Build with Docker (for ROS1 nodes)
npm run build:docker
```

### Docker Support
```bash
# Build ROS1 nodes in Docker
npm run build:ros1

# Build ROS2 nodes in Docker
npm run build:ros2:docker
```

## 📈 Monitoring & Metrics

```bash
# Start metrics collection
npm run metrics:start

# View metrics
npm run metrics:test

# Stop metrics
npm run metrics:stop

# Cleanup metrics data
npm run metrics:cleanup
```

**Features:**
- Prometheus metrics collection
- Grafana dashboards
- InfluxDB time-series storage
- Telegraf data collection

**Documentation:** [Metrics Guide](docs/METRICS.md)

## 🔧 Configuration

### Environment Variables
Each modality can be configured via environment variables in their respective dataflow files:

- **Audio**: `nodes/audio/dataflow.audio.yml`
- **Image**: `nodes/image/dataflow.image.yml`
- **TTS**: `nodes/tts/dataflow.tts.yml`

### Network Configuration
Update deployment scripts in `python_helpers/`:
- `deploy_and_run_remote.sh` - TTS deployment
- `deploy_and_run_audio_sender.sh` - Audio sender deployment

## 🧪 Testing

```bash
# Test setup
npm run test:setup

# Test audio system
npm run test:audio

# Test release package
npm run test:release-docker
```

## 📦 Release Management

```bash
# Create release package
npm run create-release

# Test release package
npm run test:release-docker
```

## 🐛 Troubleshooting

### Common Issues

1. **Build Failures**
   - Ensure all prerequisites are installed
   - Check Rust toolchain: `rustup show`
   - Verify ROS environments are sourced

2. **Audio Issues**
   - Check GStreamer installation
   - Verify audio device permissions
   - Review [Audio Streaming Guide](docs/AUDIO_STREAMING.md)

3. **Network Issues**
   - Verify SSH connectivity to robot
   - Check firewall settings
   - Validate IP addresses in deployment scripts

4. **ROS Bridge Issues**
   - Ensure ROS1 and ROS2 environments are properly sourced
   - Check topic/service names match
   - Verify message types are compatible

### Debug Mode
Enable debug logging by setting environment variables:
```bash
export RUST_LOG=debug
export DORA_LOG_LEVEL=debug
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Dora Dataflow](https://github.com/dora-rs/dora) - The underlying dataflow framework
- [ROS](https://www.ros.org/) - Robot Operating System
- [GStreamer](https://gstreamer.freedesktop.org/) - Multimedia framework for audio streaming
