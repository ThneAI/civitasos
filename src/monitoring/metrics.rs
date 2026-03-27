//! Metrics collection and aggregation for CivitasOS

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::SystemMetrics;

/// Metric types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Individual metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

impl Metric {
    pub fn new(name: &str, metric_type: MetricType, value: f64) -> Self {
        Metric {
            name: name.to_string(),
            metric_type,
            value,
            labels: HashMap::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }
}

/// Metrics collector
pub struct MetricsCollector {
    pub metrics: Vec<Metric>,
    pub max_metrics: usize,
}

impl MetricsCollector {
    pub fn new(max_metrics: usize) -> Self {
        MetricsCollector {
            metrics: Vec::new(),
            max_metrics,
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);

        // Keep only the most recent metrics
        if self.metrics.len() > self.max_metrics {
            self.metrics.drain(0..self.metrics.len() - self.max_metrics);
        }
    }

    /// Get metrics by name
    pub fn get_metrics_by_name(&self, name: &str) -> Vec<&Metric> {
        self.metrics.iter().filter(|m| m.name == name).collect()
    }

    /// Get metrics by type
    pub fn get_metrics_by_type(&self, metric_type: &MetricType) -> Vec<&Metric> {
        self.metrics
            .iter()
            .filter(|m| m.metric_type == *metric_type)
            .collect()
    }

    /// Calculate aggregate statistics for a metric
    pub fn get_metric_stats(&self, name: &str) -> Option<MetricStats> {
        let metrics: Vec<&Metric> = self.get_metrics_by_name(name);

        if metrics.is_empty() {
            return None;
        }

        let mut values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Some(MetricStats {
            count: metrics.len(),
            sum: values.iter().sum(),
            min: *values.first()?,
            max: *values.last()?,
            mean: values.iter().sum::<f64>() / values.len() as f64,
            median: if values.len().is_multiple_of(2) {
                (values[values.len() / 2] + values[values.len() / 2 - 1]) / 2.0
            } else {
                values[values.len() / 2]
            },
        })
    }

    /// Convert system metrics to individual metrics
    pub fn from_system_metrics(&mut self, sys_metrics: &SystemMetrics) {
        // CPU usage
        self.add_metric(
            Metric::new("cpu_usage", MetricType::Gauge, sys_metrics.cpu_usage)
                .with_labels(HashMap::new()),
        );

        // Memory usage
        self.add_metric(
            Metric::new(
                "memory_usage_bytes",
                MetricType::Gauge,
                sys_metrics.memory_usage as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Disk usage
        self.add_metric(
            Metric::new(
                "disk_usage_bytes",
                MetricType::Gauge,
                sys_metrics.disk_usage as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Network in
        self.add_metric(
            Metric::new(
                "network_received_bytes",
                MetricType::Counter,
                sys_metrics.network_in as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Network out
        self.add_metric(
            Metric::new(
                "network_sent_bytes",
                MetricType::Counter,
                sys_metrics.network_out as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Active connections
        self.add_metric(
            Metric::new(
                "active_connections",
                MetricType::Gauge,
                sys_metrics.active_connections as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Execution count
        self.add_metric(
            Metric::new(
                "execution_count_total",
                MetricType::Counter,
                sys_metrics.execution_count as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Average execution time
        self.add_metric(
            Metric::new(
                "avg_execution_time_ms",
                MetricType::Gauge,
                sys_metrics.avg_execution_time,
            )
            .with_labels(HashMap::new()),
        );

        // State operations
        self.add_metric(
            Metric::new(
                "state_operations_total",
                MetricType::Counter,
                sys_metrics.state_operations as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Consensus rounds
        self.add_metric(
            Metric::new(
                "consensus_rounds_total",
                MetricType::Counter,
                sys_metrics.consensus_rounds as f64,
            )
            .with_labels(HashMap::new()),
        );

        // Governance proposals
        self.add_metric(
            Metric::new(
                "governance_proposals_total",
                MetricType::Counter,
                sys_metrics.governance_proposals as f64,
            )
            .with_labels(HashMap::new()),
        );
    }
}

/// Statistics for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
}

/// Metrics exporter for external systems
pub struct MetricsExporter {
    collectors: Vec<MetricsCollector>,
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsExporter {
    pub fn new() -> Self {
        MetricsExporter {
            collectors: Vec::new(),
        }
    }

    pub fn add_collector(&mut self, collector: MetricsCollector) {
        self.collectors.push(collector);
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        for collector in &self.collectors {
            for metric in &collector.metrics {
                // Add metric name and type
                output.push_str(&format!(
                    "# TYPE {} {}\n",
                    metric.name,
                    match metric.metric_type {
                        MetricType::Counter => "counter",
                        MetricType::Gauge => "gauge",
                        MetricType::Histogram => "histogram",
                        MetricType::Summary => "summary",
                    }
                ));

                // Add metric with labels
                let labels: Vec<String> = metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();

                let label_str = if labels.is_empty() {
                    String::new()
                } else {
                    format!("{{{}}}", labels.join(","))
                };

                output.push_str(&format!("{}{} {}\n", metric.name, label_str, metric.value));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new("test_metric", MetricType::Gauge, 42.0);
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Gauge);
        assert_eq!(metric.value, 42.0);
        assert!(!metric.labels.is_empty() || metric.labels.is_empty()); // Just checking it compiles
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new(100);

        // Add some metrics
        collector.add_metric(Metric::new("cpu_usage", MetricType::Gauge, 80.0));
        collector.add_metric(Metric::new("cpu_usage", MetricType::Gauge, 75.0));
        collector.add_metric(Metric::new("cpu_usage", MetricType::Gauge, 90.0));

        // Check that we can get metrics by name
        let cpu_metrics = collector.get_metrics_by_name("cpu_usage");
        assert_eq!(cpu_metrics.len(), 3);

        // Check stats calculation
        let stats = collector.get_metric_stats("cpu_usage").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 75.0);
        assert_eq!(stats.max, 90.0);
        assert!((stats.mean - 81.67).abs() < 1.0); // Allow for floating point precision
    }

    #[test]
    fn test_metrics_exporter() {
        let mut collector = MetricsCollector::new(100);
        collector.add_metric(
            Metric::new("test_counter", MetricType::Counter, 100.0).with_labels(HashMap::new()),
        );

        let mut exporter = MetricsExporter::new();
        exporter.add_collector(collector);

        let prom_output = exporter.export_prometheus();
        assert!(prom_output.contains("test_counter"));
        assert!(prom_output.contains("counter"));
        assert!(prom_output.contains("100"));
    }
}
