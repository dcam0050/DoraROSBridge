#!/bin/bash

# Clean up metrics monitoring services and data for dora
# This script removes all containers, volumes, and data

set -e

echo "🧹 Cleaning up dora metrics monitoring services and data..."

# Check if docker compose is available
if ! docker compose version &> /dev/null; then
    echo "❌ docker compose is not available."
    exit 1
fi

# Stop and remove containers, networks, and volumes
docker compose -f docker-compose.metrics.yml down -v

echo "✅ Metrics services and data cleaned up successfully!"
echo ""
echo "💡 To start fresh metrics services: ./scripts/start-metrics.sh"
