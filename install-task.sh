#!/bin/bash

# Install task (Taskfile.dev) if not already installed
# This script helps users get started with the task-based repository management

set -e

echo "🔧 Checking for task installation..."

# Check if task is already installed
if command -v task &> /dev/null; then
    echo "✅ task is already installed: $(task --version)"
    exit 0
fi

echo "📦 task not found. Installing..."

# Detect operating system
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    if command -v apt &> /dev/null; then
        # Ubuntu/Debian
        echo "Installing task via snap (Ubuntu/Debian)..."
        sudo snap install task --classic
    elif command -v yum &> /dev/null; then
        # CentOS/RHEL
        echo "Installing task via snap (CentOS/RHEL)..."
        sudo snap install task --classic
    else
        echo "❌ Unsupported Linux distribution. Please install task manually:"
        echo "   Visit: https://taskfile.dev/installation/"
        exit 1
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    if command -v brew &> /dev/null; then
        echo "Installing task via Homebrew (macOS)..."
        brew install go-task
    else
        echo "❌ Homebrew not found. Please install Homebrew first:"
        echo "   https://brew.sh/"
        exit 1
    fi
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    # Windows
    if command -v choco &> /dev/null; then
        echo "Installing task via Chocolatey (Windows)..."
        choco install task
    else
        echo "❌ Chocolatey not found. Please install Chocolatey first:"
        echo "   https://chocolatey.org/install"
        exit 1
    fi
else
    echo "❌ Unsupported operating system. Please install task manually:"
    echo "   Visit: https://taskfile.dev/installation/"
    exit 1
fi

# Verify installation
if command -v task &> /dev/null; then
    echo "✅ task installed successfully: $(task --version)"
    echo ""
    echo "🎉 You can now use task commands:"
    echo "   task help          # Show all available commands"
    echo "   task setup         # Initial setup"
    echo "   task build         # Build all components"
    echo "   task start         # Start the system"
else
    echo "❌ Installation failed. Please install task manually:"
    echo "   Visit: https://taskfile.dev/installation/"
    exit 1
fi
