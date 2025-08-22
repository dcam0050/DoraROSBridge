#!/bin/bash

# Common utilities for ROS bridge scripts
# This file contains shared functions and variables used across multiple scripts

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Common logging functions
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

# Common cleanup function
cleanup() {
    local message=${1:-"Cleaning up..."}
    log "$message"
    
    # Stop Dora dataflow
    if pgrep -f "dora run" > /dev/null; then
        log "Stopping Dora dataflow..."
        dora stop 2>/dev/null || true
    fi
    
    # Stop remote TTS bridge if running
    if pgrep -f "deploy_and_run_remote.sh" > /dev/null; then
        log "Stopping remote TTS bridge..."
        ./python_helpers/deploy_and_run_remote.sh stop 2>/dev/null || true
    fi
    
    log "Cleanup completed"
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

# Check if ROS1 tools are available
check_ros1() {
    if ! command -v rostopic &> /dev/null; then
        warning "ROS1 tools not found (this is OK if using remote ROS1 master)"
    else
        log "ROS1 tools are available"
    fi
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
        exit 1
    fi
    log "Docker is available"
}

# Check if Docker Compose is available
check_docker_compose() {
    if ! docker compose version &> /dev/null; then
        error "Docker Compose is not available. Please install Docker Compose first."
        exit 1
    fi
    log "Docker Compose is available"
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

# Check if rqt_image_view is available
check_rqt_image_view() {
    if ! ros2 pkg list | grep -q "rqt_image_view"; then
        error "rqt_image_view not found. Installing..."
        error "sudo apt install ros-rolling-rqt-image-view"
        exit 1
    fi
    log "rqt_image_view is available"
}

# Check if node binaries are built
check_node_binaries() {
    local nodes=("$@")
    for node in "${nodes[@]}"; do
        if [ ! -f "target/debug/$node" ]; then
            error "$node not built. Run: task build"
            exit 1
        fi
        log "$node is built"
    done
}

# Test port availability
test_port() {
    local port=$1
    local protocol=${2:-"tcp"}
    log "Testing $protocol port $port availability..."
    
    if netstat -tuln 2>/dev/null | grep -q ":$port "; then
        warning "$protocol port $port is already in use"
        return 1
    else
        log "$protocol port $port is available"
        return 0
    fi
}

# Test HTTP endpoint
test_http_endpoint() {
    local url=$1
    local name=${2:-"endpoint"}
    
    if curl -s "$url" > /dev/null; then
        log "$name is accessible at $url"
        return 0
    else
        error "$name is not accessible at $url"
        return 1
    fi
}

# Start Dora dataflow with cleanup
start_dora_dataflow() {
    local dataflow_file=$1
    local description=${2:-"dataflow"}
    
    log "Starting $description..."
    log "Press Ctrl+C to stop"
    
    # Start the dataflow in background
    dora run "$dataflow_file" &
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
    trap "cleanup 'Stopping $description...' && exit 0" SIGINT SIGTERM
    
    # Wait for user to stop
    wait $dora_pid
}

# Deploy and start remote TTS bridge
deploy_tts_bridge() {
    log "Deploying TTS bridge to remote robot..."
    ./python_helpers/deploy_and_run_remote.sh &
    local deploy_pid=$!
    
    # Wait a moment for deployment to complete
    sleep 5
    
    # Check if deployment was successful
    if ! kill -0 $deploy_pid 2>/dev/null; then
        error "TTS bridge deployment failed"
        exit 1
    fi
    
    log "TTS bridge deployed successfully (PID: $deploy_pid)"
    echo $deploy_pid
}

# Show script usage
show_usage() {
    local script_name=$1
    local description=$2
    local usage_text=$3
    
    if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
        echo "Usage: $script_name"
        echo ""
        echo "$description"
        echo ""
        echo "$usage_text"
        exit 0
    fi
}
