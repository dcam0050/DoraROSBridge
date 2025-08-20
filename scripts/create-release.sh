#!/bin/bash

# Create release package for ROS1-ROS2 image bridge
# This script packages all binaries needed to run the dataflow on a system without Cargo

set -e

echo "Creating release package..."

# Create release directory
RELEASE_DIR="release"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Get the current project directory
PROJECT_DIR=$(pwd)
echo "Project directory: $PROJECT_DIR"

# Build all components first in release mode
echo "Building all components in release mode..."

# Build ROS1 node using Docker
echo "Building ROS1 node using Docker..."
docker build -f Dockerfile.ros1 -t dora-ros1-builder .
docker run --rm -v "$PROJECT_DIR:/workspace" dora-ros1-builder cargo build --release -p ros1-image-source

# Build ROS2 node using Docker
echo "Building ROS2 node using Docker..."
docker build -f Dockerfile.ros2 -t dora-ros2-builder .
docker run --rm -v "$PROJECT_DIR:/workspace" dora-ros2-builder cargo build --release -p ros2-image-sink

# Build dora-cli using Docker (using ROS2 environment)
echo "Building dora-cli using Docker..."
docker run --rm -v "$PROJECT_DIR:/workspace" dora-ros2-builder cargo install dora-cli --root /workspace/target

# Copy ROS1 node binary
echo "Copying ROS1 node..."
mkdir -p "$RELEASE_DIR/bin"
cp "target/release/ros1-image-source" "$RELEASE_DIR/bin/"

# Copy ROS2 node binary  
echo "Copying ROS2 node..."
cp "target/release/ros2-image-sink" "$RELEASE_DIR/bin/"

# Copy dora-cli binary
echo "Copying dora-cli..."
cp "target/bin/dora" "$RELEASE_DIR/bin/"

# Create release-specific dataflow configuration
echo "Creating release dataflow configuration..."
cat > "$RELEASE_DIR/dataflow.yml" << 'EOF'
nodes:
  - id: ros1-image-source
    path: ./bin/ros1-image-source
    inputs:
      tick: dora/timer/millis/10
    env:
      ROS_MASTER_URI: "http://tiago-119c:11311"
      ROS_HOSTNAME: "katana"
      ROS_IMAGE_TOPIC: "/xtion/rgb/image_raw"
      CMAKE_PREFIX_PATH: "/opt/ros/noetic"
      ROS_PACKAGE_PATH: "/opt/ros/noetic/share"
      ROSRUST_MSG_PATH: "/opt/ros/noetic/share"
    outputs:
      - image

  - id: ros2-image-sink
    path: ./bin/ros2-image-sink
    inputs:
      image: ros1-image-source/image
    env:
      ROS2_TOPIC: "/camera/image_raw"
EOF

# Copy run script
echo "Creating run script..."
cat > "$RELEASE_DIR/run.sh" << 'EOF'
#!/bin/bash

# Run the ROS1-ROS2 image bridge dataflow
# This script runs the dataflow on a system without Cargo

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$SCRIPT_DIR/bin:$PATH"

echo "Starting ROS1-ROS2 image bridge dataflow..."
echo "Make sure ROS1 and ROS2 are running in separate terminals:"
echo "  Terminal 1: roscore"
echo "  Terminal 2: ros2 daemon"

# Run the dataflow using the included dora binary
./bin/dora run ./dataflow.yml
EOF

chmod +x "$RELEASE_DIR/run.sh"

# Create README
echo "Creating README..."
cat > "$RELEASE_DIR/README.md" << 'EOF'
# ROS1-ROS2 Image Bridge Release

This release contains all binaries needed to run the ROS1-ROS2 image bridge dataflow.

## Contents

- `bin/` - Compiled binaries
  - `dora` - Dora CLI for running dataflows
  - `ros1-image-source` - ROS1 image source node
  - `ros2-image-sink` - ROS2 image sink node
- `dataflow.yml` - Dataflow configuration
- `run.sh` - Script to run the dataflow (requires ROS1/ROS2 installed)

## Prerequisites

- ROS1 Noetic installed and sourced
- ROS2 Rolling installed and sourced

## Usage

1. Start ROS1:
   ```bash
   roscore
   ```

2. Start ROS2:
   ```bash
   ros2 daemon
   ```

3. Run the dataflow:
   ```bash
   ./run.sh
   ```

## Configuration

Before running, update the ROS1 environment variables in `dataflow.yml`:

```yaml
env:
  ROS_MASTER_URI: "http://your-ros1-master:11311"
  ROS_HOSTNAME: "your-hostname"
  ROS_IMAGE_TOPIC: "/your/image/topic"
```

## Mounting on Ubuntu 24.04

To mount this release on a plain Ubuntu 24.04 system:

1. Copy the release directory to the target system
2. Ensure ROS1 and ROS2 are installed and running
3. Configure the dataflow.yml file
4. Run the dataflow script

## Scripts Included

- **`run.sh`** - Runs the dataflow with local ROS1/ROS2 installation
  - Automatically sets up PATH to find binaries
  - Provides clear instructions for starting ROS1/ROS2

## Troubleshooting

- Ensure ROS1 and ROS2 are running before starting the dataflow
- Check that all binaries are executable: `chmod +x bin/*`
- Verify network connectivity between ROS1 and ROS2
- The scripts automatically set up the correct PATH environment
EOF

# Create a tarball for easy distribution
echo "Creating release tarball..."
tar -czf "ros-bridge-release.tar.gz" "$RELEASE_DIR"

echo "Release package created successfully!"
echo "Release directory: $RELEASE_DIR"
echo "Release tarball: ros-bridge-release.tar.gz"
echo ""
echo "✅ Release package includes:"
echo "   - All compiled binaries (dora, ros1-image-source, ros2-image-sink)"
echo "   - Fixed run.sh script with proper PATH setup"
echo "   - Complete documentation and usage instructions"
echo ""
echo "To test on Ubuntu 24.04:"
echo "1. Extract the tarball"
echo "2. Ensure ROS1 and ROS2 are running"
echo "3. Configure dataflow.yml"
echo "4. Run: ./run.sh"
