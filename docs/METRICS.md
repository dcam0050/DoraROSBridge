# Dora Metrics Monitoring

This guide explains how to set up and use metrics monitoring for your dora ROS bridge dataflow.

## Overview

Based on the [dora-rs metrics documentation](https://dora-rs.ai/docs/guides/Debugging/metrics), dora exports metrics to `http://localhost:4317` using OpenTelemetry. This setup provides:

- **CPU usage** per node
- **Memory usage** per node  
- **Disk usage** per node
- **GPU memory usage** per node (if NVIDIA GPU available)

## Quick Start

### 1. Start Metrics Services

```bash
npm run metrics:start
```

This starts:
- **InfluxDB** (port 8086) - Time-series database for storing metrics
- **Grafana** (port 3000) - Dashboard for visualizing metrics
- **Telegraf** (port 4317) - Metrics collector that receives dora metrics

### 2. Access the Dashboard

1. Open Grafana: http://localhost:3000
2. Login with: `admin` / `admin`
3. The "Dora ROS Bridge Metrics" dashboard should be automatically loaded

### 3. Run Your Dataflow

```bash
npm run start
```

Metrics will be automatically collected and displayed in Grafana.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Dora Nodes    │    │    Telegraf     │    │    InfluxDB     │
│                 │    │                 │    │                 │
│ ros1-image-     │───▶│  (port 4317)    │───▶│  (port 8086)    │
│ source          │    │                 │    │                 │
│ ros2-image-     │    │                 │    │                 │
│ sink            │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
                                              ┌─────────────────┐
                                              │     Grafana     │
                                              │   (port 3000)   │
                                              │                 │
                                              │   Dashboards    │
                                              └─────────────────┘
```

## Available Metrics

### Per Node Metrics
- **CPU Usage**: `process_cpu_seconds_total`
- **Memory Usage**: `process_resident_memory_bytes`
- **Open File Descriptors**: `process_open_fds`
- **GPU Memory** (if available): `nvidia_gpu_memory_used_bytes`

### Dashboard Panels
1. **CPU Usage by Node** - Shows CPU utilization for each dora node
2. **Memory Usage by Node** - Shows memory consumption for each dora node
3. **Open File Descriptors** - Shows file descriptor usage
4. **GPU Memory Usage** - Shows GPU memory usage (if NVIDIA GPU available)

## Management Commands

### Start Metrics Services
```bash
npm run metrics:start
```

### Stop Metrics Services
```bash
npm run metrics:stop
```

### Clean Up Everything (including data)
```bash
npm run metrics:cleanup
```

## Configuration

### InfluxDB
- **URL**: http://localhost:8086
- **Organization**: dora-org
- **Bucket**: dora-metrics
- **Token**: dora-token

### Grafana
- **URL**: http://localhost:3000
- **Username**: admin
- **Password**: admin

### Telegraf
- **OpenTelemetry Endpoint**: localhost:4317 (gRPC)
- **Collection Interval**: 10 seconds
- **Output**: InfluxDB
- **Protocol**: OpenTelemetry gRPC (not HTTP)

## Customization

### Adding Custom Metrics
To add custom metrics to your dora nodes, you can use the dora metrics API:

```rust
use dora_metrics::Metrics;

let metrics = Metrics::new();
metrics.counter("custom_metric", 1.0);
```

### Modifying Dashboard
1. Edit `scripts/grafana/dashboards/dora-metrics.json`
2. Restart metrics services: `npm run metrics:stop && npm run metrics:start`

### Changing Collection Interval
Edit `scripts/telegraf/telegraf.conf` and modify the `interval` setting.

## Troubleshooting

### No Metrics Appearing
1. Check if Telegraf is running: `docker ps | grep telegraf`
2. Verify dora is exporting metrics: `curl http://localhost:4317/metrics`
3. Check InfluxDB logs: `docker logs dora-influxdb`

### Dashboard Not Loading
1. Check Grafana logs: `docker logs dora-grafana`
2. Verify datasource connection in Grafana UI
3. Check if dashboard provisioning worked

### High Resource Usage
- Reduce collection interval in Telegraf config
- Increase aggregation windows in dashboard queries
- Consider using data retention policies in InfluxDB

## Development vs Production

### Development (Current Setup)
- All services run in Docker containers
- Data stored in Docker volumes
- Easy to start/stop with npm scripts
- Suitable for development and testing

### Production Considerations
- Use external InfluxDB/InfluxDB Cloud
- Set up proper authentication and security
- Configure data retention policies
- Use monitoring and alerting
- Consider using managed Grafana (Grafana Cloud)

## References

- [Dora Metrics Documentation](https://dora-rs.ai/docs/guides/Debugging/metrics)
- [OpenTelemetry](https://opentelemetry.io/)
- [InfluxDB Documentation](https://docs.influxdata.com/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Telegraf Documentation](https://docs.influxdata.com/telegraf/)
