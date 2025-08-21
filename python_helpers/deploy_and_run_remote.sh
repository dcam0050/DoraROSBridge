#!/bin/bash

# Deploy and Run TTS Bridge on Remote System
# This script copies the TTS bridge files to a remote system and runs them

set -e

# Configuration - EDIT THESE VALUES
REMOTE_USER="pal"
REMOTE_HOST="tiago-119c"
REMOTE_DIR="/home/$REMOTE_USER/tts_bridge"
LOCAL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

# Check if remote connection details are configured
check_config() {
    if [[ "$REMOTE_USER" == "your_username" ]] || [[ "$REMOTE_HOST" == "your_remote_host" ]]; then
        error "Please edit this script and set REMOTE_USER and REMOTE_HOST variables"
        error "Current values:"
        error "  REMOTE_USER=$REMOTE_USER"
        error "  REMOTE_HOST=$REMOTE_HOST"
        exit 1
    fi
}

# Test SSH connection
test_connection() {
    log "Testing SSH connection to $REMOTE_USER@$REMOTE_HOST..."
    if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$REMOTE_USER@$REMOTE_HOST" "echo 'SSH connection successful'" 2>/dev/null; then
        error "Cannot connect to $REMOTE_USER@$REMOTE_HOST"
        error "Please check your SSH configuration and ensure key-based authentication is set up"
        exit 1
    fi
    log "SSH connection successful"
}

# Copy files to remote system
copy_files() {
    log "Copying TTS bridge files to remote system..."
    
    # Create remote directory
    ssh "$REMOTE_USER@$REMOTE_HOST" "mkdir -p $REMOTE_DIR"
    
    # Copy Python files
    scp "$LOCAL_DIR/tts_bridge.py" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_DIR/"
    
    # Make files executable on remote system
    ssh "$REMOTE_USER@$REMOTE_HOST" "chmod +x $REMOTE_DIR/tts_bridge.py"
    
    log "Files copied successfully to $REMOTE_DIR"
}

# Run the bridge on remote system
run_remote_bridge() {
    log "Starting TTS bridge on remote system..."
    log "Press Ctrl+C to stop the remote process"
    
    # Run the bridge on remote system with ROS environment and PAL workspace sourced
    ssh "$REMOTE_USER@$REMOTE_HOST" "cd $REMOTE_DIR && source /opt/ros/melodic/setup.bash && source /home/pal/catkin_ws/devel/setup.bash && source init_pal_env.sh && python tts_bridge.py"
}

# Cleanup function
cleanup() {
    log "Stopping remote TTS bridge..."
    ssh "$REMOTE_USER@$REMOTE_HOST" "pkill -f 'python.*tts_bridge'" 2>/dev/null || true
    log "Remote TTS bridge stopped"
    exit 0
}

# Main execution
main() {
    log "TTS Bridge Remote Deploy and Run Script"
    log "Target: $REMOTE_USER@$REMOTE_HOST:$REMOTE_DIR"
    
    # Check configuration
    check_config
    
    # Test connection
    test_connection
    
    # Copy files
    copy_files
    
    # Set up signal handler for Ctrl+C
    trap cleanup SIGINT SIGTERM
    
    # Run the bridge
    run_remote_bridge
}

# Show usage if no arguments or help requested
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "Usage: $0"
    echo ""
    echo "This script deploys and runs the TTS bridge on a remote system."
    echo ""
    echo "Before running:"
    echo "1. Edit this script and set REMOTE_USER and REMOTE_HOST variables"
    echo "2. Ensure SSH key-based authentication is set up"
    echo "3. Make sure ROS is installed on the remote system"
    echo ""
    echo "The script will:"
    echo "1. Copy TTS bridge files to the remote system"
    echo "2. Start the TTS bridge on the remote system"
    echo "3. Keep running until you press Ctrl+C"
    echo ""
    echo "Files will be copied to: \$REMOTE_DIR on the remote system"
    exit 0
fi

# Run main function
main "$@"
