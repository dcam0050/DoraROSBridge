#!/bin/bash

echo "=== GPU Configuration Diagnostic ==="
echo

echo "1. GPU Hardware:"
lspci | grep -E "(VGA|3D)" || echo "   No GPU info found"
echo

echo "2. Graphics drivers:"
lsmod | grep -E "(nvidia|nouveau|i915|amdgpu)" || echo "   No graphics drivers loaded"
echo

echo "3. Current X11 server info:"
echo "   DISPLAY: $DISPLAY"
xrandr --listproviders 2>/dev/null || echo "   xrandr not available"
echo

echo "4. OpenGL renderer (host):"
glxinfo | grep -E "(OpenGL renderer|OpenGL vendor)" 2>/dev/null || echo "   glxinfo not available"
echo

echo "5. Container GPU access test:"
docker run --rm --network=host -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix:rw dora-ros2-builder bash -c "
echo '   Container GPU test:'
glxinfo | grep -E '(OpenGL renderer|OpenGL vendor)' 2>/dev/null || echo '   No OpenGL in container'
" 2>/dev/null || echo "   Container test failed"
echo

echo "=== Hybrid GPU Solutions ==="
echo
echo "For laptops with integrated + discrete GPU:"
echo "1. Force Intel integrated graphics: __GLX_VENDOR_LIBRARY_NAME=mesa"
echo "2. Use software rendering: LIBGL_ALWAYS_SOFTWARE=1"
echo "3. Try different X11 forwarding: ssh -X instead of docker"
echo "4. Use VNC/web-based viewer instead"
echo
