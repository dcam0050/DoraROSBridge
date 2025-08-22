#!/bin/bash

# Start metrics monitoring services for dora
# This script starts InfluxDB, Grafana, and Telegraf for collecting dora metrics

set -e

echo "🚀 Starting dora metrics monitoring services..."

# Check if docker compose is available
if ! docker compose version &> /dev/null; then
    echo "❌ docker compose is not available. Please install Docker Compose first."
    exit 1
fi

# Start the metrics services
docker compose -f docker-compose.metrics.yml up -d

echo "✅ Metrics services started successfully!"
echo ""
echo "📊 Services available at:"
echo "  - Grafana Dashboard: http://localhost:3000 (admin/admin)"
echo "  - InfluxDB: http://localhost:8086"
echo "  - Telegraf (metrics collector): localhost:4317"
echo ""
echo "🔍 To view metrics:"
echo "  1. Open Grafana at http://localhost:3000"
echo "  2. Login with admin/admin"
echo "  3. The 'Dora ROS Bridge Metrics' dashboard should be available"
echo ""
echo "📈 To collect metrics from your dora dataflow:"
echo "  1. Start your dora dataflow: npm run start"
echo "  2. Metrics will be automatically collected and displayed in Grafana"
echo ""
echo "🛑 To stop metrics services: ./scripts/stop-metrics.sh"
