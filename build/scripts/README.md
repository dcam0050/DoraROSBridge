# Build Scripts

This directory contains streamlined build and utility scripts for the ROS bridge project.

## Script Organization

The scripts have been streamlined to reduce duplication and improve maintainability:

### Core Scripts

- **`common.sh`** - Common utilities and functions used across all scripts
- **`setup.sh`** - Initial environment setup and component building
- **`create-release.sh`** - Creates release packages with all components
- **`run-with-viewer.sh`** - Starts image pipeline with rqt_image_view
- **`start-metrics.sh`** - Starts metrics monitoring services
- **`stop-metrics.sh`** - Stops metrics monitoring services
- **`cleanup-metrics.sh`** - Cleans up metrics services and data

### Removed Scripts

The following scripts have been removed and their functionality moved to `package.json`:

- `test-audio.sh` → `npm run test:audio`
- `test-metrics.sh` → `npm run test:metrics`
- `test-setup.sh` → `npm run test:setup`
- `start-all.sh` → `npm run start`
- `start-tts.sh` → `npm run start:tts`
- `test-release-docker.sh` → `npm run test:release`

## Usage

**Primary Interface**: Use `package.json` scripts as the main interface:

```bash
# Setup and testing
npm run setup          # Initial setup
npm run test:setup     # Test environment
npm run test:audio     # Test audio system
npm run test:metrics   # Test metrics system

# Starting systems
npm run start          # Start complete system
npm run start:image    # Start image pipeline only
npm run start:tts      # Start TTS system only
npm run start:audio    # Start audio system only
npm run start:with-viewer # Start with image viewer

# Monitoring and management
npm run logs           # View all logs
npm run stop           # Stop all systems
npm run metrics:start  # Start metrics
npm run metrics:stop   # Stop metrics

# Help
npm run help           # Show all available commands
```

**Direct Script Usage**: For advanced use cases, scripts can be run directly:

```bash
# Setup
./build/scripts/setup.sh

# Create release
./build/scripts/create-release.sh

# Run with viewer
./build/scripts/run-with-viewer.sh

# Metrics management
./build/scripts/start-metrics.sh
./build/scripts/stop-metrics.sh
./build/scripts/cleanup-metrics.sh

# Release testing
./build/scripts/test-release-docker.sh
```

## Common Utilities

The `common.sh` script provides shared functionality:

- **Logging functions**: `log()`, `error()`, `warning()`, `info()`
- **Prerequisite checks**: `check_dora()`, `check_ros2()`, `check_docker()`, etc.
- **Utility functions**: `test_port()`, `test_http_endpoint()`, `start_dora_dataflow()`
- **Cleanup functions**: `cleanup()`, `deploy_tts_bridge()`

## Benefits of Streamlining

1. **Reduced Duplication**: Common functions are shared across scripts
2. **Single Source of Truth**: `package.json` serves as the primary interface
3. **Easier Maintenance**: Changes to common functionality only need to be made in one place
4. **Better Discoverability**: All commands are visible in `package.json`
5. **Consistent Interface**: All commands follow the same `npm run` pattern

## Migration Guide

If you were using the old scripts directly, update your commands:

| Old Command | New Command |
|-------------|-------------|
| `./build/scripts/test-audio.sh` | `npm run test:audio` |
| `./build/scripts/test-metrics.sh` | `npm run test:metrics` |
| `./build/scripts/test-setup.sh` | `npm run test:setup` |
| `./build/scripts/start-all.sh` | `npm run start` |
| `./build/scripts/start-tts.sh` | `npm run start:tts` |
| `./build/scripts/start-metrics.sh` | `npm run metrics:start` |
| `./build/scripts/stop-metrics.sh` | `npm run metrics:stop` |
| `./build/scripts/cleanup-metrics.sh` | `npm run metrics:cleanup` |
| `./build/scripts/test-release-docker.sh` | `npm run test:release` |
