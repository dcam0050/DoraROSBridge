#!/bin/bash

set -e

echo "Setting up ROS Bridge project..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if dora-cli is installed
if ! command -v dora &> /dev/null; then
    echo "📦 Installing dora-cli..."
    cargo install dora-cli
else
    echo "✅ dora-cli is already installed"
fi

# Check if ROS2 is available
if ! command -v ros2 &> /dev/null; then
    echo "⚠️  ROS2 is not found in PATH. Please source your ROS2 installation:"
    echo "   source /opt/ros/rolling/setup.bash"
    echo "   (or add it to your ~/.bashrc)"
fi

# Build the ROS2 part locally
echo "🔨 Building ROS2 components..."
cargo build -p ros2-image-sink

echo "📦 Note: ROS1 components will be built in Docker when needed"
echo "   Use 'npm run build:ros1' to build ROS1 components"

echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Configure your ROS1 environment in nodes/image/dataflow.image.yml"
echo "2. Run: npm run test:setup"
echo "3. Run: npm run start:with-viewer"
