#!/bin/bash

# Create release script for ROS bridge
# This script creates a release package with all necessary components

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

# Configuration
RELEASE_NAME="ros-bridge-release"
VERSION=$(node -p "require('../../package.json').version")
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RELEASE_DIR="${RELEASE_NAME}-${VERSION}-${TIMESTAMP}"

log "Creating ROS bridge release..."

# Check prerequisites
check_docker

# Create release directory
log "Creating release directory: $RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Copy essential files
log "Copying project files..."
cp -r nodes "$RELEASE_DIR/"
cp -r build "$RELEASE_DIR/"
cp -r python_helpers "$RELEASE_DIR/"
cp package.json "$RELEASE_DIR/"
cp README.md "$RELEASE_DIR/"
cp Cargo.toml "$RELEASE_DIR/"

# Copy Docker files
log "Copying Docker configuration..."
cp build/docker/Dockerfile.ros1 "$RELEASE_DIR/"
cp build/docker/Dockerfile.ros2 "$RELEASE_DIR/"

# Copy metrics configuration if it exists
if [ -f "docker-compose.metrics.yml" ]; then
    cp docker-compose.metrics.yml "$RELEASE_DIR/"
fi

# Create release README
log "Creating release README..."
cat > "$RELEASE_DIR/RELEASE_README.md" << EOF
# ROS Bridge Release v${VERSION}

This release contains the ROS1 to ROS2 bridge system with image, TTS, and audio support.

## Quick Start

1. Install dependencies:
   \`\`\`bash
   # Install Node.js and npm
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   
   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh
   \`\`\`

2. Setup the environment:
   \`\`\`bash
   npm run setup
   \`\`\`

3. Test the setup:
   \`\`\`bash
   npm run test:setup
   \`\`\`

4. Start the system:
   \`\`\`bash
   npm run start
   \`\`\`

## Available Commands

- \`npm run help\` - Show all available commands
- \`npm run start:image\` - Start image pipeline only
- \`npm run start:tts\` - Start TTS system only
- \`npm run start:audio\` - Start audio system only
- \`npm run metrics:start\` - Start metrics monitoring

## Documentation

See the main README.md for detailed documentation.

## Release Information

- Version: ${VERSION}
- Created: $(date)
- Timestamp: ${TIMESTAMP}
EOF

# Create archive
log "Creating release archive..."
tar -czf "${RELEASE_DIR}.tar.gz" "$RELEASE_DIR"

# Cleanup
rm -rf "$RELEASE_DIR"

log "✅ Release created successfully: ${RELEASE_DIR}.tar.gz"
log "Release size: $(du -h "${RELEASE_DIR}.tar.gz" | cut -f1)"
