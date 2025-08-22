#!/bin/bash

# Stop metrics script
# This script stops all metrics services

set -e

# Source common utilities
source "$(dirname "$0")/common.sh"

log "Stopping metrics services..."

# Stop services
docker compose -f docker-compose.metrics.yml down

log "✅ Metrics services stopped successfully!"
