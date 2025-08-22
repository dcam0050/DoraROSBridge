#!/bin/bash

# Source common utilities
source "$(dirname "$0")/common.sh"

# Check if parameter is provided
if [ $# -eq 0 ]; then
    error "Usage: $0 <ros1|ros2> [packages]"
    echo "  ros1: Build ROS1 components"
    echo "  ros2: Build ROS2 components" 
    echo "  packages: Optional comma-separated list of specific packages to build"
    echo "           If not provided, builds all packages for that ROS version"
    exit 1
fi

ROS_TYPE="$1"
PACKAGES="$2"

# Default packages for each ROS version
DEFAULT_ROS1_PACKAGES="ros1-image-source,ros1-tts-sink"
DEFAULT_ROS2_PACKAGES="ros2-image-sink,ros2-tts-source"

case "$ROS_TYPE" in
    "ros1")
        PACKAGES_TO_BUILD=${PACKAGES:-$DEFAULT_ROS1_PACKAGES}
        log "Building ROS1 components: $PACKAGES_TO_BUILD"
        "$(dirname "$0")/manage-containers.sh" build ros1 "$PACKAGES_TO_BUILD"
        ;;
    "ros2")
        PACKAGES_TO_BUILD=${PACKAGES:-$DEFAULT_ROS2_PACKAGES}
        log "Building ROS2 components: $PACKAGES_TO_BUILD"
        "$(dirname "$0")/manage-containers.sh" build ros2 "$PACKAGES_TO_BUILD"
        ;;
    *)
        error "Invalid parameter: $ROS_TYPE"
        echo "Valid options: ros1, ros2"
        exit 1
        ;;
esac
