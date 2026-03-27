//! Monitoring and observability for CivitasOS
//! Provides performance metrics, system health checks, and security auditing

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub mod metrics;
pub mod health;
pub mod security;

use crate::execution::ExecutionResult;
use crate::state::StateStore;

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64, // in bytes
    pub disk_usage: u64,   // in bytes
    pub network_in: u64,   // bytes received
    pub network_out: u64,  // bytes sent
    pub active_connections: usize,
    pub execution_count: u64,
    pub avg_execution_time: f64, // in milliseconds
    pub state_operations: u64,
    pub consensus_rounds: u64,
    pub governance_proposals: u64,
}

impl SystemMetrics {
    pub fn new() -> Self {
        SystemMetrics {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0,
            disk_usage: 0,
            network_in: 0,
            network_out: 0,
            active_connections: 0,
            execution_count: 0,
            avg_execution_time: 0.0,
            state_operations: 0,
            consensus_rounds: 0,
            governance_proposals: 0,
        }
    }
}

/// Performance monitor collects and aggregates system metrics
pub struct PerformanceMonitor {
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    max_metrics_history: usize,
    last_execution_time: std::time::Instant,
    execution_times: Vec<f64>,
    execution_count: AtomicU64,
}

impl PerformanceMonitor {
    pub fn new(max_metrics_history: usize) -> Self {
        PerformanceMonitor {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            max_metrics_history,
            last_execution_time: std::time::Instant::now(),
            execution_times: Vec::new(),
            execution_count: AtomicU64::new(0),
        }
    }

    /// Record execution metrics
    pub async fn record_execution(&mut self, execution_result: &ExecutionResult) {
        self.execution_times.push(execution_result.gas_used as f64); // Using gas as a proxy for execution time
        self.execution_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Calculate average execution time
    pub fn calculate_avg_execution_time(&self) -> f64 {
        if self.execution_times.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.execution_times.iter().sum();
        sum / self.execution_times.len() as f64
    }

    /// Collect current system metrics
    pub async fn collect_metrics(&self) -> SystemMetrics {
        let mut metrics = SystemMetrics::new();
        
        // Get current averages
        metrics.avg_execution_time = self.calculate_avg_execution_time();
        metrics.execution_count = self.execution_count.load(Ordering::Relaxed);
        
        // Add to metrics history
        let mut metrics_history = self.metrics_history.write().await;
        metrics_history.push(metrics.clone());
        
        // Keep only the most recent metrics
        if metrics_history.len() > self.max_metrics_history {
            let len = metrics_history.len();
            if len > self.max_metrics_history {
                metrics_history.drain(0..len - self.max_metrics_history);
            }
        }
        
        metrics
    }

    /// Get aggregated metrics for a time period
    pub async fn get_aggregated_metrics(&self, minutes: u64) -> Option<SystemMetrics> {
        let metrics_history = self.metrics_history.read().await;
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (minutes * 60);
        
        let recent_metrics: Vec<&SystemMetrics> = metrics_history
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .collect();
        
        if recent_metrics.is_empty() {
            return None;
        }
        
        // Aggregate the metrics
        let mut agg_metrics = SystemMetrics::new();
        agg_metrics.timestamp = cutoff_time;
        
        for metric in &recent_metrics {
            agg_metrics.cpu_usage += metric.cpu_usage;
            agg_metrics.memory_usage += metric.memory_usage;
            agg_metrics.disk_usage += metric.disk_usage;
            agg_metrics.network_in += metric.network_in;
            agg_metrics.network_out += metric.network_out;
            agg_metrics.active_connections += metric.active_connections;
            agg_metrics.execution_count += metric.execution_count;
            agg_metrics.avg_execution_time += metric.avg_execution_time;
            agg_metrics.state_operations += metric.state_operations;
            agg_metrics.consensus_rounds += metric.consensus_rounds;
            agg_metrics.governance_proposals += metric.governance_proposals;
        }
        
        // Average the values
        let count = recent_metrics.len() as f64;
        agg_metrics.cpu_usage /= count;
        agg_metrics.avg_execution_time /= count;
        
        Some(agg_metrics)
    }

    /// Get current metrics history
    pub async fn get_metrics_history(&self) -> Vec<SystemMetrics> {
        self.metrics_history.read().await.clone()
    }
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// System health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: u64,
    pub overall_status: HealthStatus,
    pub component_status: HashMap<String, HealthStatus>,
    pub uptime: Duration,
    pub error_count: u64,
    pub warnings: Vec<String>,
}

impl HealthReport {
    pub fn new() -> Self {
        HealthReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            overall_status: HealthStatus::Unknown,
            component_status: HashMap::new(),
            uptime: Duration::new(0, 0),
            error_count: 0,
            warnings: Vec::new(),
        }
    }
}

/// Security audit trail for tracking important events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub actor: String, // Node ID or user ID
    pub details: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    ExecutionStarted,
    ExecutionCompleted,
    StateTransition,
    ProposalCreated,
    VoteCast,
    ConsensusRound,
    NetworkConnection,
    SecurityViolation,
    ConfigurationChange,
    ErrorOccurred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security auditor tracks and analyzes system events
pub struct SecurityAuditor {
    audit_trail: Arc<RwLock<Vec<AuditEntry>>>,
    max_audit_entries: usize,
    security_rules: Vec<SecurityRule>,
}

#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub name: String,
    pub description: String,
    pub condition: SecurityCondition,
    pub action: SecurityAction,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum SecurityCondition {
    ExecutionThreshold { max_gas: u64 },
    StateAccessPattern { pattern: String },
    NetworkRateLimit { max_requests_per_minute: u64 },
    ConsensusAnomaly,
    GovernanceSpam { max_proposals_per_hour: u64 },
}

#[derive(Debug, Clone)]
pub enum SecurityAction {
    LogOnly,
    Alert,
    Throttle,
    Block,
    Quarantine,
}

impl SecurityAuditor {
    pub fn new(max_audit_entries: usize) -> Self {
        SecurityAuditor {
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            max_audit_entries,
            security_rules: vec![
                SecurityRule {
                    name: "execution_gas_limit".to_string(),
                    description: "Monitor execution gas usage".to_string(),
                    condition: SecurityCondition::ExecutionThreshold { max_gas: 100000 },
                    action: SecurityAction::Alert,
                    enabled: true,
                },
                SecurityRule {
                    name: "proposal_rate_limit".to_string(),
                    description: "Prevent governance spam".to_string(),
                    condition: SecurityCondition::GovernanceSpam { max_proposals_per_hour: 10 },
                    action: SecurityAction::Throttle,
                    enabled: true,
                },
            ],
        }
    }

    /// Log an audit entry
    pub async fn log_event(
        &self,
        event_type: AuditEventType,
        severity: AuditSeverity,
        actor: String,
        details: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut audit_trail = self.audit_trail.write().await;
        
        let audit_entry = AuditEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type,
            severity,
            actor,
            details,
            metadata: HashMap::new(),
        };
        
        audit_trail.push(audit_entry);
        
        // Keep only the most recent entries
        if audit_trail.len() > self.max_audit_entries {
            let len = audit_trail.len();
            if len > self.max_audit_entries {
                audit_trail.drain(0..len - self.max_audit_entries);
            }
        }
        
        Ok(())
    }

    /// Check if an event violates security rules
    pub fn check_security_rules(&self, event: &AuditEntry) -> Vec<&SecurityRule> {
        let mut violations = Vec::new();
        
        for rule in &self.security_rules {
            if !rule.enabled {
                continue;
            }
            
            match &rule.condition {
                SecurityCondition::ExecutionThreshold { max_gas } => {
                    if matches!(event.event_type, AuditEventType::ExecutionStarted) {
                        // Check if execution details exceed threshold
                        if event.details.contains(&max_gas.to_string()) {
                            violations.push(rule);
                        }
                    }
                },
                SecurityCondition::GovernanceSpam { max_proposals_per_hour } => {
                    if matches!(event.event_type, AuditEventType::ProposalCreated) {
                        // This would require additional tracking logic
                        // Simplified check for demonstration
                        violations.push(rule);
                    }
                },
                _ => {
                    // Other conditions would be checked here
                }
            }
        }
        
        violations
    }

    /// Get recent audit entries
    pub async fn get_recent_audits(&self, count: usize) -> Vec<AuditEntry> {
        let audit_trail = self.audit_trail.read().await;
        let start = if audit_trail.len() > count {
            audit_trail.len() - count
        } else {
            0
        };
        
        audit_trail[start..].to_vec()
    }

    /// Get security alerts
    pub async fn get_security_alerts(&self, hours: u64) -> Vec<AuditEntry> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (hours * 3600);
        
        let audit_trail = self.audit_trail.read().await;
        audit_trail
            .iter()
            .filter(|entry| {
                entry.timestamp >= cutoff_time && 
                matches!(entry.severity, AuditSeverity::Warning | AuditSeverity::Error | AuditSeverity::Critical)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics::new();
        assert!(metrics.timestamp > 0);
        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.avg_execution_time, 0.0);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(100);
        
        // Simulate some executions
        for _ in 0..10 {
            let execution_result = crate::execution::ExecutionResult {
                gas_used: 1000,
                success: true,
                state_diffs: vec![],
                trace_hash: "test_hash".to_string(),
            };
            monitor.record_execution(&execution_result).await;
        }
        
        let metrics = monitor.collect_metrics().await;
        assert_eq!(monitor.execution_count.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_health_report_creation() {
        let report = HealthReport::new();
        assert!(report.timestamp > 0);
        assert_eq!(report.overall_status, HealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_security_auditor() {
        let auditor = SecurityAuditor::new(1000);
        
        // Log a test event
        auditor.log_event(
            AuditEventType::ExecutionStarted,
            AuditSeverity::Info,
            "test_node".to_string(),
            "Execution started".to_string(),
        ).await.unwrap();
        
        let audits = auditor.get_recent_audits(10).await;
        assert_eq!(audits.len(), 1);
        assert_eq!(audits[0].actor, "test_node");
    }
}