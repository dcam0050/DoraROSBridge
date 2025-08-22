#!/bin/bash

# Run with viewer script
# This script starts the image pipeline with rqt_image_view

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

show_usage "$0" \
    "This script starts the image pipeline with rqt_image_view for visualization" \
    "Prerequisites:\n- ROS2 must be installed and sourced\n- rqt_image_view must be available"

# Check prerequisites
check_dora
check_ros2
check_rqt_image_view

log "Starting image pipeline with viewer..."

# Start the dataflow in background
dora run ./nodes/image/dataflow.image.yml &
DORA_PID=$!

# Wait for dataflow to start
sleep 3

# Check if Dora is still running
if ! kill -0 $DORA_PID 2>/dev/null; then
    error "Dora dataflow failed to start"
    exit 1
fi

log "Dora dataflow started with PID: $DORA_PID"

# Start rqt_image_view
log "Starting rqt_image_view..."
rqt_image_view &
VIEWER_PID=$!

log "System started successfully!"
log "Dora PID: $DORA_PID"
log "Viewer PID: $VIEWER_PID"
log "Press Ctrl+C to stop both processes"

# Set up cleanup
trap "cleanup 'Stopping image pipeline with viewer...' && kill $VIEWER_PID 2>/dev/null || true && exit 0" SIGINT SIGTERM

# Wait for either process to exit
wait
