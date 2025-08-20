#!/bin/bash

# Test the release package using Docker containers
# This script sets up ROS1 and ROS2 in separate containers and tests the dataflow

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="$SCRIPT_DIR/../release"

echo "Testing release package with Docker..."

# Check if release directory exists
if [ ! -d "$RELEASE_DIR" ]; then
    echo "❌ Release directory not found. Run './scripts/create-release.sh' first."
    exit 1
fi

# Check if binaries exist
if [ ! -f "$RELEASE_DIR/bin/dora-cli" ] || [ ! -f "$RELEASE_DIR/bin/ros1-image-source" ] || [ ! -f "$RELEASE_DIR/bin/ros2-image-sink" ]; then
    echo "❌ Release binaries not found. Run './scripts/create-release.sh' first."
    exit 1
fi

echo "✅ Release package found and validated"

# Start ROS1 container
echo "Starting ROS1 container..."
docker run -d --name ros1-core --network host ros:noetic-ros-base roscore

# Wait for ROS1 to start
sleep 3

# Start ROS2 container
echo "Starting ROS2 container..."
docker run -d --name ros2-daemon --network host ros:rolling-ros-base bash -c "ros2 daemon"

# Wait for ROS2 to start
sleep 3

# Run the dataflow
echo "Running dataflow..."
docker run --rm --network host \
  -v "$RELEASE_DIR:/workspace" \
  -w /workspace \
  ubuntu:24.04 \
  bash -c "
    export PATH=/workspace/bin:\$PATH
    export LD_LIBRARY_PATH=/workspace/lib:\$LD_LIBRARY_PATH
    echo 'Testing release package...'
    dora-cli daemon --run-dataflow ./dataflow.yml
  "

# Cleanup
echo "Cleaning up containers..."
docker stop ros1-core ros2-daemon 2>/dev/null || true
docker rm ros1-core ros2-daemon 2>/dev/null || true

echo "✅ Release package test completed"
