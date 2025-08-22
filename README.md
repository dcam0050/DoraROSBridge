# Dora ROS Bridge

A comprehensive ROS1 to ROS2 bridge system built with Dora Dataflow, supporting multiple data modalities including images, audio, and text-to-speech.

> **Note**: This project now uses [task](https://taskfile.dev/) for repository management instead of npm scripts. See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for details.

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
- **task**: For repository management (recommended) or npm (legacy)

### Setup

```bash
# Clone the repository
git clone <repository-url>
cd ros_bridge

# Install task (if not already installed)
./install-task.sh

# Install dependencies and setup environment
task setup
# or npm run setup (legacy)

# Build all nodes
task build
# or npm run build (legacy)
```

## 📊 Data Modalities

### 🎵 Audio Streaming
Stream microphone audio from robot to local system with robust playback.

```bash
# Build audio nodes
task build:audio
# or npm run build:audio (legacy)

# Start audio streaming
task start:audio
# or npm run start:audio (legacy)

# Deploy audio sender to robot
task audio:deploy
# or npm run audio:deploy (legacy)
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
task build:vision
# or npm run build:vision (legacy)

# Start image bridge
task start:image
# or npm run start:image (legacy)

# Start with viewer
task start:with-viewer
# or npm run start:with-viewer (legacy)
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
task build:tts
# or npm run build:tts (legacy)

# Start TTS bridge
task start:tts
# or npm run start:tts (legacy)

# Deploy to remote robot
task tts:deploy
# or npm run tts:deploy (legacy)
```

**Features:**
- ROS2 to ROS1 TTS service bridging
- Remote deployment capabilities
- Configurable TTS parameters

## 🛠️ Build System

### Local Development
```bash
# Build all nodes
task build
# or npm run build (legacy)

# Build specific modality
task build:audio
task build:vision
task build:tts
task build:custom
# or npm run build:audio (legacy)

# Build with Docker (for ROS1 nodes)
# Docker builds are handled automatically by the build scripts
```

### Docker Support
```bash
# Build ROS1 nodes in Docker
task build
# Docker builds are handled automatically by the build scripts

# Build ROS2 nodes in Docker
# Docker builds are handled automatically by the build scripts
```

## 📈 Monitoring & Metrics

```bash
# Start metrics collection
task metrics:start
# or npm run metrics:start (legacy)

# View metrics
task metrics:test
# or npm run metrics:test (legacy)

# Stop metrics
task metrics:stop
# or npm run metrics:stop (legacy)

# Cleanup metrics data
task metrics:cleanup
# or npm run metrics:cleanup (legacy)
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
task test:setup
# or npm run test:setup (legacy)

# Test audio system
task test:audio
# or npm run test:audio (legacy)

# Test release package
task test:release
# or npm run test:release (legacy)
```

## 📦 Release Management

```bash
# Create release package
task create-release
# or npm run create-release (legacy)

# Test release package
task test:release
# or npm run test:release (legacy)
```

## 🚀 Task Commands & MCP Integration

This project uses [task](https://taskfile.dev/) for repository management, providing better integration with MCP servers and improved developer experience.

### Quick Task Commands

```bash
# Show all available commands
task help

# Setup and build
task setup
task build

# Start systems
task start          # Complete system
task start:image    # Image pipeline only
task start:tts      # TTS system only
task start:audio    # Audio system only
task start:custom   # Custom message test

# Testing
task test:setup     # Test environment
task test:vision    # Test vision components
task test:tts       # Test TTS components
task test:audio     # Test audio system
task test:metrics   # Test metrics setup

# Monitoring
task logs           # View all logs
task stop           # Stop all systems
task clean          # Clean build artifacts

# Metrics
task metrics:start  # Start metrics services
task metrics:stop   # Stop metrics services
task metrics:test   # Test metrics setup

# Development
task dev            # Build and start complete system
```

### MCP Server Benefits

Using task commands enables better integration with MCP servers:

- **Programmatic Execution**: Execute tasks through MCP APIs
- **Task Discovery**: Get available tasks and descriptions
- **Dependency Management**: Handle task dependencies automatically
- **Cross-platform**: Consistent behavior across different operating systems
- **Performance**: Faster execution than npm scripts

### Migration from npm

If you're migrating from npm scripts, see [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for a complete reference.

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
