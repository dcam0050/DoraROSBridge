#!/bin/bash

# Container management script for ROS bridge
# This script creates and manages permanent containers for building

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

CONTAINER_ROS1="dora-ros1-builder"
CONTAINER_ROS2="dora-ros2-builder"

# Function to create ROS1 container
create_ros1_container() {
    log "Creating permanent ROS1 container..."
    
    # Remove existing container if it exists
    docker rm -f $CONTAINER_ROS1 2>/dev/null || true
    
    # Build the image
    docker build -f build/docker/Dockerfile.ros1 -t dora-ros1-builder .
    
    # Create a permanent container with host user permissions and separate target directory
    docker create --name $CONTAINER_ROS1 \
        -v $(pwd):/workspace \
        -v $(pwd)/target-ros1:/workspace/target \
        -v ~/.cargo:/home/ubuntu/.cargo \
        -w /workspace \
        -u $(id -u):$(id -g) \
        dora-ros1-builder
    
    log "✅ ROS1 container created: $CONTAINER_ROS1"
}

# Function to create ROS2 container
create_ros2_container() {
    log "Creating permanent ROS2 container..."
    
    # Remove existing container if it exists
    docker rm -f $CONTAINER_ROS2 2>/dev/null || true
    
    # Build the image
    docker build -f build/docker/Dockerfile.ros2 -t dora-ros2-builder .
    
    # Create a permanent container with X11 and audio support, using host user permissions and separate target directory
    docker create --name $CONTAINER_ROS2 \
        -v $(pwd):/workspace \
        -v $(pwd)/target-ros2:/workspace/target \
        -v ~/.cargo:/home/ubuntu/.cargo \
        -w /workspace \
        -u $(id -u):$(id -g) \
        -e DISPLAY=$DISPLAY \
        -e AMENT_PREFIX_PATH=/opt/ros/rolling:/workspace/custom_msgs/install \
        -e ROS_DISTRO=rolling \
        -e ROS_VERSION=2 \
        -e RMW_IMPLEMENTATION=rmw_cyclonedds_cpp \
        -e ROS_DOMAIN_ID=42 \
        -e ROS_LOCALHOST_ONLY=0 \
        -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
        -v /dev/snd:/dev/snd \
        --device /dev/snd \
        --group-add audio \
        dora-ros2-builder
    
    log "✅ ROS2 container created: $CONTAINER_ROS2"
}

# Function to build using permanent containers
build_with_container() {
    local ros_type=$1
    local packages=$2
    
    case "$ros_type" in
        "ros1")
            if ! docker ps -a --format "table {{.Names}}" | grep -q "^$CONTAINER_ROS1$"; then
                create_ros1_container
            fi
            log "Building ROS1 components: $packages"
            PACKAGES_FLAGS=$(echo "$packages" | sed 's/,/ -p /g' | sed 's/^/-p /')
            
            # Stop ROS2 container to avoid conflicts
            docker stop $CONTAINER_ROS2 2>/dev/null || true
            
            # Create target directory if it doesn't exist
            mkdir -p target-ros1
            
            docker start $CONTAINER_ROS1
            docker exec -u $(id -u):$(id -g) $CONTAINER_ROS1 cargo build $PACKAGES_FLAGS
            ;;
        "ros2")
            if ! docker ps -a --format "table {{.Names}}" | grep -q "^$CONTAINER_ROS2$"; then
                create_ros2_container
            fi
            log "Building ROS2 components: $packages"
            PACKAGES_FLAGS=$(echo "$packages" | sed 's/,/ -p /g' | sed 's/^/-p /')
            
            # Stop ROS1 container to avoid conflicts
            docker stop $CONTAINER_ROS1 2>/dev/null || true
            
            # Create target directory if it doesn't exist
            mkdir -p target-ros2
            
            docker start $CONTAINER_ROS2
            # Allow X11 connections from container
            xhost +local:docker 2>/dev/null || true
            docker exec -u $(id -u):$(id -g) $CONTAINER_ROS2 bash -c "source /opt/ros/rolling/setup.bash && source /opt/ros2_ws/install/setup.bash && cargo build $PACKAGES_FLAGS"
            ;;
        *)
            error "Invalid ROS type: $ros_type"
            exit 1
            ;;
    esac
}

# Function to clean up containers
cleanup_containers() {
    log "Cleaning up containers..."
    docker rm -f $CONTAINER_ROS1 2>/dev/null || true
    docker rm -f $CONTAINER_ROS2 2>/dev/null || true
    docker rmi dora-ros1-builder 2>/dev/null || true
    docker rmi dora-ros2-builder 2>/dev/null || true
    log "✅ Containers cleaned up"
}

# Function to show container status
show_status() {
    log "Container status:"
    echo "ROS1 container:"
    docker ps -a --filter "name=$CONTAINER_ROS1" --format "table {{.Names}}\t{{.Status}}\t{{.Image}}"
    echo ""
    echo "ROS2 container:"
    docker ps -a --filter "name=$CONTAINER_ROS2" --format "table {{.Names}}\t{{.Status}}\t{{.Image}}"
}

# Function to enable X11 access
enable_x11() {
    log "Enabling X11 access for containers..."
    xhost +local:docker 2>/dev/null || true
    xhost +local:root 2>/dev/null || true
    log "✅ X11 access enabled"
}

# Main script logic
case "${1:-}" in
    "create")
        create_ros1_container
        create_ros2_container
        ;;
    "build")
        if [ $# -lt 3 ]; then
            error "Usage: $0 build <ros1|ros2> <packages>"
            exit 1
        fi
        build_with_container "$2" "$3"
        ;;
    "cleanup")
        cleanup_containers
        ;;
    "status")
        show_status
        ;;
    "x11")
        enable_x11
        ;;
    *)
        echo "Usage: $0 {create|build|cleanup|status|x11}"
        echo "  create   - Create permanent containers"
        echo "  build    - Build using permanent containers"
        echo "  cleanup  - Remove containers and images"
        echo "  status   - Show container status"
        echo "  x11      - Enable X11 access for containers"
        exit 1
        ;;
esac
