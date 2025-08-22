#!/bin/bash

# Build custom messages script
# This script builds the custom_msgs package and makes it available to dora-ros2-bridge

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

log "Building custom ROS2 messages..."

# Check if we're in the right directory
if [ ! -f "custom_msgs/package.xml" ]; then
    error "custom_msgs package not found. Run this script from the project root."
    exit 1
fi

# Create build directory
mkdir -p custom_msgs/build
cd custom_msgs/build

# Configure and build
log "Configuring custom_msgs package..."
cmake .. -DCMAKE_INSTALL_PREFIX=../install

log "Building custom_msgs package..."
make -j$(nproc)

log "Installing custom_msgs package..."
make install

# Go back to project root
cd ../..

# Set AMENT_PREFIX_PATH to include our custom messages
export AMENT_PREFIX_PATH="$(pwd)/custom_msgs/install:$AMENT_PREFIX_PATH"

log "Custom messages built successfully!"
log "AMENT_PREFIX_PATH now includes: $(pwd)/custom_msgs/install"

# Test that messages are discoverable
log "Testing message discovery..."
if ros2 interface list | grep -q "custom_msgs"; then
    log "✅ Custom messages are discoverable"
else
    warning "Custom messages not found in ros2 interface list"
fi
