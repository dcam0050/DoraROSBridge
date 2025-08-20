#!/bin/bash

# Stop metrics monitoring services for dora
# This script stops InfluxDB, Grafana, and Telegraf

set -e

echo "🛑 Stopping dora metrics monitoring services..."

# Check if docker compose is available
if ! docker compose version &> /dev/null; then
    echo "❌ docker compose is not available."
    exit 1
fi

# Stop the metrics services
docker compose -f docker-compose.metrics.yml down

echo "✅ Metrics services stopped successfully!"
echo ""
echo "💡 To start metrics services again: ./scripts/start-metrics.sh"
