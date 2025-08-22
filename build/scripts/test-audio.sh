#!/bin/bash

# Test Audio Streaming System
# This script tests the audio streaming system by starting the Dora nodes and checking connectivity

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

# Check if Dora is installed
check_dora() {
    if ! command -v dora &> /dev/null; then
        error "Dora is not installed or not in PATH"
        error "Please install Dora: https://github.com/dora-rs/dora"
        exit 1
    fi
    log "Dora is available"
}

# Check if ROS2 is available
check_ros2() {
    if ! command -v ros2 &> /dev/null; then
        error "ROS2 is not installed or not in PATH"
        error "Please install ROS2 and source the setup.bash"
        exit 1
    fi
    log "ROS2 is available"
}

# Check if GStreamer is available
check_gstreamer() {
    if ! command -v gst-launch-1.0 &> /dev/null; then
        error "GStreamer is not installed"
        error "Please install GStreamer: sudo apt-get install gstreamer1.0-tools"
        exit 1
    fi
    log "GStreamer is available"
}

# Build the audio nodes
build_audio_nodes() {
    log "Building audio nodes..."
    npm run build:audio
    log "Audio nodes built successfully"
}

# Test UDP port availability
test_udp_port() {
    local port=5004
    log "Testing UDP port $port availability..."
    
    if netstat -tuln 2>/dev/null | grep -q ":$port "; then
        warning "UDP port $port is already in use"
        warning "This might cause issues with the audio receiver"
    else
        log "UDP port $port is available"
    fi
}

# Start the audio dataflow
start_audio_dataflow() {
    log "Starting audio dataflow..."
    log "This will start the GStreamer receiver and ROS2 publisher"
    log "Press Ctrl+C to stop"
    
    # Start the dataflow in background
    dora run ./nodes/audio/dataflow.audio.yml &
    local dora_pid=$!
    
    # Wait a moment for startup
    sleep 3
    
    # Check if Dora is still running
    if ! kill -0 $dora_pid 2>/dev/null; then
        error "Dora dataflow failed to start"
        exit 1
    fi
    
    log "Dora dataflow started with PID: $dora_pid"
    
    # Set up cleanup
    trap "cleanup $dora_pid" SIGINT SIGTERM
    
    # Wait for user to stop
    wait $dora_pid
}

# Cleanup function
cleanup() {
    local dora_pid=$1
    log "Stopping audio dataflow..."
    
    if kill -0 $dora_pid 2>/dev/null; then
        kill $dora_pid
        wait $dora_pid 2>/dev/null || true
    fi
    
    dora stop 2>/dev/null || true
    log "Audio dataflow stopped"
    exit 0
}

# Test ROS2 topic
test_ros2_topic() {
    log "Testing ROS2 topic /robot/audio..."
    
    # Check if topic exists
    if ros2 topic list | grep -q "/robot/audio"; then
        log "ROS2 topic /robot/audio is available"
        
        # Show topic info
        log "Topic info:"
        ros2 topic info /robot/audio
        
        # Show topic type
        log "Topic type:"
        ros2 topic type /robot/audio
    else
        warning "ROS2 topic /robot/audio not found"
        warning "This is normal if no audio data has been received yet"
    fi
}

# Main execution
main() {
    log "Audio Streaming System Test"
    log "=========================="
    
    # Check prerequisites
    check_dora
    check_ros2
    check_gstreamer
    
    # Build nodes
    build_audio_nodes
    
    # Test port availability
    test_udp_port
    
    # Start dataflow
    start_audio_dataflow
}

# Show usage if help requested
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "Usage: $0"
    echo ""
    echo "This script tests the audio streaming system by:"
    echo "1. Checking prerequisites (Dora, ROS2, GStreamer)"
    echo "2. Building the audio nodes"
    echo "3. Testing UDP port availability"
    echo "4. Starting the audio dataflow"
    echo "5. Testing ROS2 topic availability"
    echo ""
    echo "The system will:"
    echo "- Listen for audio on UDP port 5004"
    echo "- Publish audio to ROS2 topic /robot/audio"
    echo "- Keep running until you press Ctrl+C"
    echo ""
    echo "To test with actual audio, run the audio sender on the robot:"
    echo "  npm run audio:deploy"
    exit 0
fi

# Run main function
main "$@"
