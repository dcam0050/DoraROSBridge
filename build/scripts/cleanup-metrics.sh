#!/bin/bash

# Cleanup metrics script
# This script stops and cleans up all metrics services

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

log "Cleaning up metrics services..."

# Stop services
log "Stopping metrics services..."
docker compose -f docker-compose.metrics.yml down -v

# Clean up Docker system
log "Cleaning up Docker system..."
docker system prune -f

log "✅ Metrics cleanup completed successfully!"
