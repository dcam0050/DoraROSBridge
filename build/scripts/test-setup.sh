#!/bin/bash

# Test script to verify ROS1→ROS2 Image Bridge setup

echo "🔍 Testing ROS1→ROS2 Image Bridge setup..."
echo ""

# Test 1: Check if ROS2 is sourced
echo "1️⃣  Testing ROS2 environment..."
if command -v ros2 &> /dev/null; then
    echo "   ✅ ROS2 is available"
    echo "   Version: $(ros2 --help | head -1)"
else
    echo "   ❌ ROS2 not found. Please source ROS2:"
    echo "      source /opt/ros/rolling/setup.bash"
    exit 1
fi

# Test 2: Check if rqt_image_view is available
echo ""
echo "2️⃣  Testing ROS2 image viewer..."
if ros2 pkg list | grep -q "rqt_image_view"; then
    echo "   ✅ rqt_image_view is available"
else
    echo "   ❌ rqt_image_view not found. Installing..."
    echo "   sudo apt install ros-rolling-rqt-image-view"
    exit 1
fi

# Test 3: Check if ROS1 tools are available
echo ""
echo "3️⃣  Testing ROS1 environment..."
if command -v rostopic &> /dev/null; then
    echo "   ✅ ROS1 tools are available"
else
    echo "   ⚠️  ROS1 tools not found (this is OK if using remote ROS1 master)"
fi

# Test 4: Check if Dora CLI is available
echo ""
echo "4️⃣  Testing Dora CLI..."
if cargo run --package dora-cli --release -- --help &> /dev/null; then
    echo "   ✅ Dora CLI is available"
else
    echo "   ❌ Dora CLI not found. Please build Dora first."
    exit 1
fi

# Test 5: Check if nodes are built
echo ""
echo "5️⃣  Testing node binaries..."
if [ -f "../../target/debug/ros1-image-source" ]; then
    echo "   ✅ ROS1 source node is built"
else
    echo "   ❌ ROS1 source node not built. Run: npm run build:ros1"
    exit 1
fi

if [ -f "../../target/debug/ros2-image-sink" ]; then
    echo "   ✅ ROS2 sink node is built"
else
    echo "   ❌ ROS2 sink node not built. Run: npm run build:ros2"
    exit 1
fi

echo ""
echo "✅ All tests passed! You're ready to run the pipeline."
echo ""
echo "Next steps:"
echo "  npm run start:with-viewer  # Run with image viewer"
echo "  npm run start:with-echo    # Run with topic echo"
echo "  npm run start              # Run dataflow only"
