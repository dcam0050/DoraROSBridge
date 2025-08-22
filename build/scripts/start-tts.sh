#!/bin/bash

# TTS Startup Script
# This script starts both the TTS bridge on the remote robot and the local Dora dataflow

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

# Function to cleanup on exit
cleanup() {
    log "Cleaning up..."
    
    # Stop Dora dataflow
    if pgrep -f "dora run" > /dev/null; then
        log "Stopping Dora dataflow..."
        dora stop
    fi
    
    # Stop remote TTS bridge
    log "Stopping remote TTS bridge..."
    ./python_helpers/deploy_and_run_remote.sh stop 2>/dev/null || true
    
    log "Cleanup completed"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    log "Starting TTS system..."
    
    # Start the TTS bridge on remote robot in background
    log "Deploying TTS bridge to remote robot..."
    ./python_helpers/deploy_and_run_remote.sh &
    TTS_DEPLOY_PID=$!
    
    # Wait a moment for deployment to complete
    sleep 5
    
    # Check if deployment was successful
    if ! kill -0 $TTS_DEPLOY_PID 2>/dev/null; then
        error "TTS bridge deployment failed"
        exit 1
    fi
    
    log "TTS bridge deployed successfully (PID: $TTS_DEPLOY_PID)"
    
    # Start the Dora dataflow
    log "Starting Dora TTS dataflow..."
    dora run ./nodes/tts/dataflow.tts.yml &
    DORA_PID=$!
    
    log "TTS system started successfully!"
    log "Remote TTS bridge PID: $TTS_DEPLOY_PID"
    log "Dora dataflow PID: $DORA_PID"
    log "Press Ctrl+C to stop both processes"
    
    # Wait for either process to exit
    wait
}

# Show usage if help requested
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "Usage: $0"
    echo ""
    echo "This script starts the complete TTS system:"
    echo "1. Deploys and starts the TTS bridge on the remote robot"
    echo "2. Starts the local Dora TTS dataflow"
    echo "3. Runs both processes in parallel"
    echo "4. Handles cleanup when stopped with Ctrl+C"
    echo ""
    echo "Prerequisites:"
    echo "- Remote robot must be accessible via SSH"
    echo "- TTS bridge configuration must be set in deploy_and_run_remote.sh"
    exit 0
fi

# Run main function
main "$@"
