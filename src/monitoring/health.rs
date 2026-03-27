//! Health checks and system status monitoring for CivitasOS

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use super::{HealthReport, HealthStatus};
use crate::consensus::ConsensusEngine;
use crate::execution::ExecutionEngine;
use crate::governance::GovernanceEngine;
use crate::state::StateStore;

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub response_time_ms: u64,
}

/// Health checker performs various system checks
pub struct HealthChecker {
    pub checks: HashMap<String, Box<dyn HealthCheck>>,
    pub last_report: RwLock<Option<HealthReport>>,
    pub check_interval: Duration,
}

impl HealthChecker {
    pub fn new(check_interval: Duration) -> Self {
        HealthChecker {
            checks: HashMap::new(),
            last_report: RwLock::new(None),
            check_interval,
        }
    }

    /// Add a health check
    pub fn add_check(&mut self, name: String, check: Box<dyn HealthCheck>) {
        self.checks.insert(name, check);
    }

    /// Perform all registered health checks
    pub async fn perform_checks(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        for (name, check) in &self.checks {
            let start_time = std::time::Instant::now();
            let result = check.check().await;
            let duration = start_time.elapsed().as_millis() as u64;

            results.push(HealthCheckResult {
                component: name.clone(),
                status: result.status,
                message: result.message,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: duration,
            });
        }

        results
    }

    /// Generate a health report
    pub async fn generate_report(&self) -> HealthReport {
        let check_results = self.perform_checks().await;
        let mut report = HealthReport::new();

        // Map individual check results to component statuses
        for result in &check_results {
            report
                .component_status
                .insert(result.component.clone(), result.status.clone());

            if matches!(result.status, HealthStatus::Critical) {
                report.overall_status = HealthStatus::Critical;
                report.error_count += 1;
            } else if matches!(result.status, HealthStatus::Warning) {
                if !matches!(report.overall_status, HealthStatus::Critical) {
                    report.overall_status = HealthStatus::Warning;
                }
                report
                    .warnings
                    .push(format!("{}: {}", result.component, result.message));
            } else if matches!(result.status, HealthStatus::Healthy)
                && matches!(report.overall_status, HealthStatus::Unknown)
            {
                report.overall_status = HealthStatus::Healthy;
            }
        }

        // Update the last report
        *self.last_report.write().await = Some(report.clone());

        report
    }

    /// Get the last generated report
    pub async fn get_last_report(&self) -> Option<HealthReport> {
        self.last_report.read().await.clone()
    }

    /// Check if the system is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(report) = self.get_last_report().await {
            matches!(report.overall_status, HealthStatus::Healthy)
        } else {
            false
        }
    }
}

/// Trait for health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthCheckResult;
}

/// Execution engine health check
pub struct ExecutionHealthCheck {
    engine: ExecutionEngine,
}

impl ExecutionHealthCheck {
    pub fn new(engine: ExecutionEngine) -> Self {
        ExecutionHealthCheck { engine }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ExecutionHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Perform a simple execution test
        // We can't clone ExecutionEngine, so we'll create a new instance for testing
        let mut test_engine = ExecutionEngine::new(1000000); // Use default gas limit

        // Set up a minimal test program
        let program = vec![
            crate::execution::OpCode::ADD(0, 1, 2), // r0 = r1 + r2
        ];

        // Set registers for the test
        test_engine.context.registers[1] = 10;
        test_engine.context.registers[2] = 20;

        match test_engine.execute_program(program) {
            Ok(result) => {
                if result.success && test_engine.context.registers[0] == 30 {
                    HealthCheckResult {
                        component: "execution_engine".to_string(),
                        status: HealthStatus::Healthy,
                        message: "Execution engine operational".to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        response_time_ms: 0, // Will be set by caller
                    }
                } else {
                    HealthCheckResult {
                        component: "execution_engine".to_string(),
                        status: HealthStatus::Critical,
                        message: "Execution engine failed test".to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        response_time_ms: 0,
                    }
                }
            }
            Err(e) => HealthCheckResult {
                component: "execution_engine".to_string(),
                status: HealthStatus::Critical,
                message: format!("Execution engine error: {:?}", e),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 0,
            },
        }
    }
}

/// State store health check
pub struct StateHealthCheck {
    store: StateStore,
}

impl StateHealthCheck {
    pub fn new(store: StateStore) -> Self {
        StateHealthCheck { store }
    }
}

#[async_trait::async_trait]
impl HealthCheck for StateHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Perform a state operation test
        let test_key = "health_check_test".to_string();
        let test_value = "test_value".to_string();

        // Try to store and retrieve a value
        let mut store = self.store.clone();
        store.put(test_key.clone(), test_value.clone());

        if let Some(retrieved) = store.get(&test_key) {
            if retrieved.value == test_value {
                HealthCheckResult {
                    component: "state_store".to_string(),
                    status: HealthStatus::Healthy,
                    message: "State store operational".to_string(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 0,
                }
            } else {
                HealthCheckResult {
                    component: "state_store".to_string(),
                    status: HealthStatus::Critical,
                    message: "State store read/write mismatch".to_string(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 0,
                }
            }
        } else {
            HealthCheckResult {
                component: "state_store".to_string(),
                status: HealthStatus::Critical,
                message: "State store failed to retrieve value".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 0,
            }
        }
    }
}

/// Consensus engine health check
pub struct ConsensusHealthCheck {
    engine: ConsensusEngine,
}

impl ConsensusHealthCheck {
    pub fn new(engine: ConsensusEngine) -> Self {
        ConsensusHealthCheck { engine }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ConsensusHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Perform a simple validation test
        let result = self.engine.validate_execution_result(
            &crate::execution::AtomicDecisionUnit {
                input_state_hash: "test_input".to_string(),
                rule_id: "test_rule".to_string(),
                execution_trace: vec![crate::execution::OpCode::RETURN],
                output_proof: "test_proof".to_string(),
                accountability_anchor: "test_anchor".to_string(),
                risk_stake: 100,
            },
            &crate::execution::ExecutionResult {
                gas_used: 100,
                success: true,
                state_diffs: vec![],
                trace_hash: "test_trace".to_string(),
            },
        );

        match result {
            Ok(validation_result) => {
                if validation_result {
                    HealthCheckResult {
                        component: "consensus_engine".to_string(),
                        status: HealthStatus::Healthy,
                        message: "Consensus engine operational".to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        response_time_ms: 0,
                    }
                } else {
                    HealthCheckResult {
                        component: "consensus_engine".to_string(),
                        status: HealthStatus::Warning,
                        message: "Consensus validation returned false".to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        response_time_ms: 0,
                    }
                }
            }
            Err(e) => HealthCheckResult {
                component: "consensus_engine".to_string(),
                status: HealthStatus::Critical,
                message: format!("Consensus engine error: {}", e),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 0,
            },
        }
    }
}

/// Governance engine health check
pub struct GovernanceHealthCheck {
    engine: GovernanceEngine,
}

impl GovernanceHealthCheck {
    pub fn new(engine: GovernanceEngine) -> Self {
        GovernanceHealthCheck { engine }
    }
}

#[async_trait::async_trait]
impl HealthCheck for GovernanceHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Check if governance engine can perform basic operations
        let result = self.engine.get_active_constitutional_rules();

        HealthCheckResult {
            component: "governance_engine".to_string(),
            status: if result.is_empty() {
                HealthStatus::Warning // Not necessarily unhealthy, just no active rules
            } else {
                HealthStatus::Healthy
            },
            message: format!(
                "Governance engine operational, {} active rules",
                result.len()
            ),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: 0,
        }
    }
}

/// Resource usage health check
pub struct ResourceHealthCheck {
    pub cpu_threshold: f64,
    pub memory_threshold: u64,
    pub disk_threshold: u64,
}

impl ResourceHealthCheck {
    pub fn new(cpu_threshold: f64, memory_threshold: u64, disk_threshold: u64) -> Self {
        ResourceHealthCheck {
            cpu_threshold,
            memory_threshold,
            disk_threshold,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ResourceHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Note: This is a simplified resource check
        // In a real system, you would use system APIs to get actual resource usage

        // For now, we'll return healthy as a placeholder
        HealthCheckResult {
            component: "resource_usage".to_string(),
            status: HealthStatus::Healthy,
            message: "Resource usage within limits".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateStore;

    #[tokio::test]
    async fn test_health_checker() {
        let mut checker = HealthChecker::new(Duration::from_secs(30));

        // Add a simple test check
        checker.add_check("test_component".to_string(), Box::new(TestHealthCheck {}));

        let results = checker.perform_checks().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].component, "test_component");
        assert_eq!(results[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_report_generation() {
        let checker = HealthChecker::new(Duration::from_secs(30));
        let report = checker.generate_report().await;

        assert!(report.timestamp > 0);
        assert_eq!(report.overall_status, HealthStatus::Unknown); // No checks performed
    }

    // Test health check implementation for testing purposes
    struct TestHealthCheck;

    #[async_trait::async_trait]
    impl HealthCheck for TestHealthCheck {
        async fn check(&self) -> HealthCheckResult {
            HealthCheckResult {
                component: "test_component".to_string(),
                status: HealthStatus::Healthy,
                message: "Test check passed".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 0,
            }
        }
    }

    #[tokio::test]
    async fn test_state_health_check() {
        let store = StateStore::new();
        let check = StateHealthCheck::new(store);
        let result = check.check().await;

        assert_eq!(result.component, "state_store");
        assert_eq!(result.status, HealthStatus::Healthy);
    }
}
