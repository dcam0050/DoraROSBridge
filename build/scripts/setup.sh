#!/bin/bash

# Setup script for ROS bridge
# This script performs initial setup and environment checks

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

log "Setting up ROS bridge environment..."

# Check prerequisites
check_dora
check_ros2
check_ros1
check_docker

# Create permanent containers
log "Creating permanent build containers..."
"$(dirname "$0")/manage-containers.sh" create

# Enable X11 access for GUI applications
log "Enabling X11 access..."
"$(dirname "$0")/manage-containers.sh" x11

log "✅ Setup completed successfully!"
log ""
log "Next steps:"
log "  task test:setup     # Test your environment"
log "  task build:vision   # Build vision components"
log "  task build:tts      # Build TTS components"
log "  task build:audio    # Build audio components"
log "  task start:image    # Start image pipeline"
log "  task start:tts      # Start TTS system"
log "  task start          # Start complete system"
log "  task help           # Show all available commands"
log ""
log "Note: Components are not built during setup to avoid timeouts."
log "Run individual build tasks as needed."
