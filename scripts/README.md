# Scripts Directory

This directory contains utility scripts for the ROS1-ROS2 Image Bridge project.

## Scripts

### `create-release.sh`
Creates a portable release package that can be deployed on any Ubuntu 24.04 system without requiring Cargo or ROS1/ROS2 installations.

**Usage:**
```bash
./scripts/create-release.sh
```

**Creates:**
- `release/` directory with all binaries and configuration
- `rust-ros2-ros1-release.tar.gz` compressed package

### `run-with-viewer.sh`
Runs the dataflow with an optional ROS2 image viewer to visualize the bridged images.

**Usage:**
```bash
# Run with default topic
./scripts/run-with-viewer.sh

# Run with custom topic
ROS2_TOPIC=/your/topic ./scripts/run-with-viewer.sh
```

**Features:**
- Starts the Dora dataflow
- Optionally launches ROS2 image viewer
- Handles cleanup on exit

### `test-setup.sh`
Sets up the test environment for ROS1 and ROS2 communication.

**Usage:**
```bash
./scripts/test-setup.sh
```

**Features:**
- Starts ROS1 core
- Starts ROS2 daemon
- Provides test commands for verification

### `test-release-docker.sh`
Tests the release package using Docker containers to verify it works correctly.

**Usage:**
```bash
./scripts/test-release-docker.sh
```

**Features:**
- Validates release package exists and has required binaries
- Sets up ROS1 and ROS2 in Docker containers
- Tests the dataflow with the release binaries
- Automatically cleans up containers

## NPM Scripts

These scripts can also be run via npm:

```bash
# Create release package
npm run create-release

# Run with viewer
npm run start:with-viewer

# Run with custom topic
ROS2_TOPIC=/your/topic npm run start:with-viewer:custom

# Test setup
npm run test:setup

# Test release package with Docker
./scripts/test-release-docker.sh
```

## Notes

- All scripts are executable and can be run directly
- Scripts use relative paths and should be run from the project root
- The release package is self-contained and portable
