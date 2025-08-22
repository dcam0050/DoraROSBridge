#!/bin/bash

# ROS1 to ROS2 Image Bridge with Viewer
# This script starts the complete pipeline and launches an image viewer

set -e

# Default ROS2 topic, can be overridden with ROS2_TOPIC environment variable
ROS2_TOPIC="/camera/image_raw"
echo "🚀 Starting ROS1→ROS2 Image Bridge with viewer..."
echo "   ROS2 Topic: $ROS2_TOPIC"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "🛑 Stopping pipeline..."
    if [ ! -z "$VIEWER_PID" ]; then
        echo "   Killing viewer process (PID: $VIEWER_PID)"
        kill $VIEWER_PID 2>/dev/null || true
    fi
    if [ ! -z "$DORA_PID" ]; then
        echo "   Killing Dora process (PID: $DORA_PID)"
        kill $DORA_PID 2>/dev/null || true
    fi
    echo "✅ Cleanup complete"
    exit 0
}

# Set up signal handlers
trap cleanup INT TERM EXIT

echo "1️⃣  Starting ROS2 image viewer..."
echo "   Topic: $ROS2_TOPIC"
ros2 run rqt_image_view rqt_image_view $ROS2_TOPIC &
VIEWER_PID=$!
echo "   Viewer PID: $VIEWER_PID"

echo ""
echo "2️⃣  Waiting for viewer to initialize..."
sleep 3

echo ""
echo "3️⃣  Starting Dora dataflow..."
npm run start &
DORA_PID=$!
echo "   Dora PID: $DORA_PID"

echo ""
echo "4️⃣  Pipeline running! 🎉"
echo "   - ROS2 viewer PID: $VIEWER_PID"
echo "   - Dora dataflow PID: $DORA_PID"
echo "   - Viewing topic: $ROS2_TOPIC"
echo ""
echo "Press Ctrl+C to stop everything"
echo ""

# Wait for background processes
wait
