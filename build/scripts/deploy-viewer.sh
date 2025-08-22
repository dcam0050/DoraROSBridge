#!/bin/bash

# Smart ROS2 viewer deployment script
# Automatically detects system configuration and applies appropriate settings

set -e
source "$(dirname "$0")/common.sh"

echo "📺 Deploying ROS2 image viewer in Docker container..."
echo "DISPLAY: $DISPLAY"
echo "ROS_DOMAIN_ID: ${ROS_DOMAIN_ID:-42}"
echo "ROS_LOCALHOST_ONLY: ${ROS_LOCALHOST_ONLY:-0}"

# Enable X11 access
xhost +local:docker 2>/dev/null || true
xhost +local:root 2>/dev/null || true

# Detect GPU configuration
GPU_FLAGS=""
if lspci | grep -q "VGA.*Intel.*Graphics" && lspci | grep -q "VGA.*NVIDIA\|VGA.*AMD"; then
    echo "🔍 Detected hybrid GPU system - using software rendering"
    GPU_FLAGS="-e LIBGL_ALWAYS_SOFTWARE=1 -e __GLX_VENDOR_LIBRARY_NAME=mesa"
else
    echo "🔍 Standard GPU configuration detected"
fi

# Common environment variables for Qt/X11 compatibility
# Pass through host ROS environment variables if available
COMMON_ENV="-e DISPLAY=$DISPLAY \
-e ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-42} \
-e ROS_LOCALHOST_ONLY=${ROS_LOCALHOST_ONLY:-0} \
-e QT_X11_NO_MITSHM=1 \
-e _X11_NO_MITSHM=1 \
-e XDG_RUNTIME_DIR=/tmp/runtime-docker \
$GPU_FLAGS"

# Determine which GUI application to run
APP="${2:-rqt_image_view}"
case "$APP" in
    "rqt_image_view")
        APP_CMD="ros2 run rqt_image_view rqt_image_view /camera/image_raw"
        APP_NAME="rqt_image_view"
        ;;
    "rqt_graph")
        APP_CMD="rqt_graph"
        APP_NAME="rqt_graph"
        ;;
    "rviz2")
        APP_CMD="rviz2"
        APP_NAME="rviz2"
        ;;
    *)
        APP_CMD="ros2 run rqt_image_view rqt_image_view /camera/image_raw"
        APP_NAME="rqt_image_view"
        ;;
esac

# Choose deployment method based on arguments
case "${1:-standalone}" in
    "standalone")
        echo "X11 access enabled. Starting standalone $APP_NAME..."
        docker run --rm -it --network=host $COMMON_ENV \
            -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
            -v $(pwd):/workspace -w /workspace \
            dora-ros2-builder bash -c "
                mkdir -p /tmp/runtime-docker && chmod 0700 /tmp/runtime-docker && 
                source /opt/ros/rolling/setup.bash && 
                source /opt/ros2_ws/install/setup.bash && 
                echo 'Starting $APP_NAME...' && 
                $APP_CMD"
        ;;
    "background")
        echo "X11 access enabled. Starting $APP_NAME in background container..."
        docker run --rm -d --network=host $COMMON_ENV \
            -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
            -v $(pwd):/workspace -w /workspace \
            dora-ros2-builder bash -c "
                mkdir -p /tmp/runtime-docker && chmod 0700 /tmp/runtime-docker && 
                source /opt/ros/rolling/setup.bash && 
                source /opt/ros2_ws/install/setup.bash && 
                $APP_CMD"
        ;;
    "attached")
        echo "X11 access enabled. Starting $APP_NAME in existing container..."
        docker exec -d -u $(id -u):$(id -g) $COMMON_ENV dora-ros2-builder bash -c "
            mkdir -p /tmp/runtime-docker && chmod 0700 /tmp/runtime-docker && 
            source /opt/ros/rolling/setup.bash && 
            source /opt/ros2_ws/install/setup.bash && 
            $APP_CMD"
        ;;
    *)
        echo "Usage: $0 [standalone|background|attached] [rqt_image_view|rqt_graph|rviz2]"
        echo "  standalone - Run viewer in new container (interactive)"
        echo "  background - Run viewer in new container (background)"
        echo "  attached   - Run viewer in existing container"
        echo "  GUI apps   - rqt_image_view (default), rqt_graph, rviz2"
        exit 1
        ;;
esac
