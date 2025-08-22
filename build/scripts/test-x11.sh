#!/bin/bash

# Basic X11 connectivity test
echo "=== X11 Connectivity Test ==="
echo "DISPLAY: $DISPLAY"
echo

if [ -z "$DISPLAY" ]; then
    echo "❌ DISPLAY variable not set"
    exit 1
fi

if [ ! -d "/tmp/.X11-unix" ]; then
    echo "❌ X11 socket directory not found"
    exit 1
fi

echo "✅ X11 environment looks good"
echo
echo "💡 Use 'task deploy:viewer' to test GUI applications"
echo "💡 Use 'task test:gpu' for GPU-specific diagnostics"
