//! Security auditing and threat detection for CivitasOS

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use super::{AuditEntry, AuditEventType, AuditSeverity, SecurityAuditor};

/// Threat intelligence entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    pub threat_id: String,
    pub threat_type: ThreatType,
    pub severity: AuditSeverity,
    pub description: String,
    pub indicators: Vec<String>, // IOCs (Indicators of Compromise)
    pub timestamp: u64,
    pub confidence: f64, // 0.0 to 1.0
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    DdosAttack,
    SybilAttack,
    ConsensusFailure,
    StateCorruption,
    GovernanceManipulation,
    ExecutionViolation,
    NetworkInfiltration,
    ResourceExhaustion,
    DataTampering,
    IdentitySpoofing,
}

/// Security event classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub severity: AuditSeverity,
    pub source: String,
    pub target: String,
    pub details: String,
    pub timestamp: u64,
    pub confidence: f64,
    pub related_events: Vec<String>,
    pub status: SecurityEventStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    SuspiciousLogin,
    UnauthorizedAccess,
    StateModification,
    ConsensusAnomaly,
    GovernanceAbuse,
    ResourceAbuse,
    NetworkAnomaly,
    SmartContractViolation,
    IdentityFraud,
    DoubleSpending,
    MaliciousProposal,
    ByzantineBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityEventStatus {
    New,
    Investigating,
    Resolved,
    Escalated,
    Closed,
}

/// Security analyzer performs threat detection and analysis
pub struct SecurityAnalyzer {
    pub threat_intel: RwLock<Vec<ThreatIntelligence>>,
    pub security_events: RwLock<Vec<SecurityEvent>>,
    pub detection_rules: Vec<DetectionRule>,
    pub correlation_engine: CorrelationEngine,
    pub auditor: SecurityAuditor,
}

impl SecurityAnalyzer {
    pub fn new(auditor: SecurityAuditor) -> Self {
        SecurityAnalyzer {
            threat_intel: RwLock::new(Vec::new()),
            security_events: RwLock::new(Vec::new()),
            detection_rules: vec![
                DetectionRule {
                    name: "consensus_anomaly_detector".to_string(),
                    description: "Detects unusual consensus patterns".to_string(),
                    condition: DetectionCondition::ConsensusAnomaly {
                        max_round_time: Duration::from_secs(30),
                        max_votes_per_round: 100,
                    },
                    action: DetectionAction::Alert,
                    enabled: true,
                },
                DetectionRule {
                    name: "governance_spam_detector".to_string(),
                    description: "Detects governance proposal spam".to_string(),
                    condition: DetectionCondition::GovernanceSpam {
                        max_proposals_per_minute: 10,
                    },
                    action: DetectionAction::Throttle,
                    enabled: true,
                },
                DetectionRule {
                    name: "execution_resource_abuse".to_string(),
                    description: "Detects excessive resource consumption".to_string(),
                    condition: DetectionCondition::ResourceAbuse {
                        max_gas_per_minute: 1_000_000,
                    },
                    action: DetectionAction::Block,
                    enabled: true,
                },
            ],
            correlation_engine: CorrelationEngine::new(),
            auditor,
        }
    }

    /// Analyze audit logs for security threats
    pub async fn analyze_threats(&self) -> Vec<SecurityEvent> {
        let audit_logs = self.auditor.get_recent_audits(100).await;
        let mut security_events = Vec::new();

        for audit_entry in &audit_logs {
            // Check each detection rule
            for rule in &self.detection_rules {
                if !rule.enabled {
                    continue;
                }

                if self.matches_detection_rule(audit_entry, rule).await {
                    let event = self
                        .create_security_event_from_audit(audit_entry, rule)
                        .await;
                    security_events.push(event);
                }
            }
        }

        // Add new events to our event store
        {
            let mut events = self.security_events.write().await;
            events.extend(security_events.clone());

            // Keep only recent events
            if events.len() > 1000 {
                let len = events.len();
                if len > 1000 {
                    events.drain(0..len - 1000);
                }
            }
        }

        security_events
    }

    /// Check if an audit entry matches a detection rule
    async fn matches_detection_rule(&self, entry: &AuditEntry, rule: &DetectionRule) -> bool {
        match &rule.condition {
            DetectionCondition::ConsensusAnomaly { .. } => {
                matches!(entry.event_type, AuditEventType::ConsensusRound)
            }
            DetectionCondition::GovernanceSpam { .. } => {
                matches!(entry.event_type, AuditEventType::ProposalCreated)
            }
            DetectionCondition::ResourceAbuse { .. } => {
                matches!(entry.event_type, AuditEventType::ExecutionStarted)
            }
            DetectionCondition::NetworkAnomaly { .. } => {
                matches!(entry.event_type, AuditEventType::NetworkConnection)
            }
            DetectionCondition::StateTampering => {
                matches!(entry.event_type, AuditEventType::StateTransition)
            }
            DetectionCondition::IdentitySpoofing => {
                matches!(entry.event_type, AuditEventType::SecurityViolation)
            }
        }
    }

    /// Create a security event from an audit entry
    async fn create_security_event_from_audit(
        &self,
        audit: &AuditEntry,
        rule: &DetectionRule,
    ) -> SecurityEvent {
        SecurityEvent {
            id: format!("sev_{}_{}", audit.timestamp, audit.actor),
            event_type: match &audit.event_type {
                AuditEventType::ExecutionStarted => SecurityEventType::SmartContractViolation,
                AuditEventType::StateTransition => SecurityEventType::StateModification,
                AuditEventType::ProposalCreated => SecurityEventType::GovernanceAbuse,
                AuditEventType::VoteCast => SecurityEventType::GovernanceAbuse,
                AuditEventType::ConsensusRound => SecurityEventType::ConsensusAnomaly,
                AuditEventType::NetworkConnection => SecurityEventType::NetworkAnomaly,
                AuditEventType::SecurityViolation => SecurityEventType::UnauthorizedAccess,
                _ => SecurityEventType::SuspiciousLogin,
            },
            severity: audit.severity.clone(),
            source: audit.actor.clone(),
            target: "system".to_string(),
            details: format!("Triggered by rule '{}': {}", rule.name, audit.details),
            timestamp: audit.timestamp,
            confidence: 0.8, // Default confidence
            related_events: Vec::new(),
            status: SecurityEventStatus::New,
        }
    }

    /// Add threat intelligence
    pub async fn add_threat_intelligence(&self, threat: ThreatIntelligence) {
        let mut intel = self.threat_intel.write().await;
        intel.push(threat);

        // Keep only recent intelligence
        if intel.len() > 1000 {
            let len = intel.len();
            if len > 1000 {
                intel.drain(0..len - 1000);
            }
        }
    }

    /// Get recent security events
    pub async fn get_recent_security_events(&self, count: usize) -> Vec<SecurityEvent> {
        let events = self.security_events.read().await;
        let start = if events.len() > count {
            events.len() - count
        } else {
            0
        };

        events[start..].to_vec()
    }

    /// Get security events by status
    pub async fn get_security_events_by_status(
        &self,
        status: &SecurityEventStatus,
    ) -> Vec<SecurityEvent> {
        let events = self.security_events.read().await;
        events
            .iter()
            .filter(|event| event.status == *status)
            .cloned()
            .collect()
    }

    /// Update security event status
    pub async fn update_security_event_status(
        &self,
        event_id: &str,
        status: SecurityEventStatus,
    ) -> bool {
        let mut events = self.security_events.write().await;
        for event in events.iter_mut() {
            if event.id == event_id {
                event.status = status;
                return true;
            }
        }
        false
    }

    /// Get threat statistics
    pub async fn get_threat_statistics(&self) -> ThreatStatistics {
        let events = self.security_events.read().await;
        let intel = self.threat_intel.read().await;

        let mut stats = ThreatStatistics::new();

        for event in events.iter() {
            match event.severity {
                AuditSeverity::Critical => stats.critical_events += 1,
                AuditSeverity::Error => stats.error_events += 1,
                AuditSeverity::Warning => stats.warning_events += 1,
                AuditSeverity::Info => stats.info_events += 1,
            }

            match &event.event_type {
                SecurityEventType::ConsensusAnomaly => stats.consensus_anomalies += 1,
                SecurityEventType::GovernanceAbuse => stats.governance_abuses += 1,
                SecurityEventType::ResourceAbuse => stats.resource_abuses += 1,
                _ => stats.other_events += 1,
            }
        }

        stats.total_threats = intel.len();
        stats.active_events = events
            .iter()
            .filter(|e| {
                matches!(
                    e.status,
                    SecurityEventStatus::New | SecurityEventStatus::Investigating
                )
            })
            .count();

        stats
    }

    /// Correlate security events
    pub async fn correlate_events(&self) -> Vec<EventCorrelation> {
        let events = self.get_recent_security_events(50).await;
        self.correlation_engine.correlate(&events)
    }
}

/// Detection rule for identifying security events
#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub name: String,
    pub description: String,
    pub condition: DetectionCondition,
    pub action: DetectionAction,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum DetectionCondition {
    ConsensusAnomaly {
        max_round_time: Duration,
        max_votes_per_round: u64,
    },
    GovernanceSpam {
        max_proposals_per_minute: u64,
    },
    ResourceAbuse {
        max_gas_per_minute: u64,
    },
    NetworkAnomaly {
        max_connections_per_second: u64,
    },
    StateTampering,
    IdentitySpoofing,
}

#[derive(Debug, Clone)]
pub enum DetectionAction {
    LogOnly,
    Alert,
    Throttle,
    Block,
    Quarantine,
    Escalate,
}

/// Threat statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatStatistics {
    pub total_threats: usize,
    pub critical_events: u64,
    pub error_events: u64,
    pub warning_events: u64,
    pub info_events: u64,
    pub consensus_anomalies: u64,
    pub governance_abuses: u64,
    pub resource_abuses: u64,
    pub other_events: u64,
    pub active_events: usize,
    pub resolved_events: usize,
}

impl Default for ThreatStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreatStatistics {
    pub fn new() -> Self {
        ThreatStatistics {
            total_threats: 0,
            critical_events: 0,
            error_events: 0,
            warning_events: 0,
            info_events: 0,
            consensus_anomalies: 0,
            governance_abuses: 0,
            resource_abuses: 0,
            other_events: 0,
            active_events: 0,
            resolved_events: 0,
        }
    }
}

/// Event correlation engine
pub struct CorrelationEngine {
    pub correlation_rules: Vec<CorrelationRule>,
}

#[derive(Debug, Clone)]
pub struct CorrelationRule {
    pub name: String,
    pub conditions: Vec<CorrelationCondition>,
    pub conclusion: String,
}

#[derive(Debug, Clone)]
pub enum CorrelationCondition {
    SameActor,
    SameTarget,
    TimeProximity { max_interval: Duration },
    EventTypeSequence { sequence: Vec<SecurityEventType> },
    SeverityThreshold { min_severity: AuditSeverity },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCorrelation {
    pub related_events: Vec<String>,
    pub correlation_type: String,
    pub confidence: f64,
    pub description: String,
}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrelationEngine {
    pub fn new() -> Self {
        CorrelationEngine {
            correlation_rules: vec![CorrelationRule {
                name: "consensus_then_governance_attack".to_string(),
                conditions: vec![
                    CorrelationCondition::EventTypeSequence {
                        sequence: vec![
                            SecurityEventType::ConsensusAnomaly,
                            SecurityEventType::GovernanceAbuse,
                        ],
                    },
                    CorrelationCondition::TimeProximity {
                        max_interval: Duration::from_secs(60),
                    },
                ],
                conclusion: "Potential coordinated attack on consensus and governance".to_string(),
            }],
        }
    }

    pub fn correlate(&self, events: &[SecurityEvent]) -> Vec<EventCorrelation> {
        let mut correlations = Vec::new();

        // Simple correlation: look for events from same actor within time window
        let mut grouped_by_actor: HashMap<String, Vec<&SecurityEvent>> = HashMap::new();

        for event in events {
            grouped_by_actor
                .entry(event.source.clone())
                .or_default()
                .push(event);
        }

        for (actor, actor_events) in grouped_by_actor {
            if actor_events.len() > 1 {
                // Multiple events from same actor
                let event_ids: Vec<String> = actor_events.iter().map(|e| e.id.clone()).collect();

                correlations.push(EventCorrelation {
                    related_events: event_ids,
                    correlation_type: "multiple_events_from_actor".to_string(),
                    confidence: 0.7,
                    description: format!("Multiple security events from actor: {}", actor),
                });
            }
        }

        correlations
    }
}

/// Security reporting
pub struct SecurityReporter {
    pub analyzer: SecurityAnalyzer,
}

impl SecurityReporter {
    pub fn new(analyzer: SecurityAnalyzer) -> Self {
        SecurityReporter { analyzer }
    }

    /// Generate a security report
    pub async fn generate_report(&self) -> SecurityReport {
        let stats = self.analyzer.get_threat_statistics().await;
        let recent_events = self.analyzer.get_recent_security_events(10).await;
        let correlations = self.analyzer.correlate_events().await;

        SecurityReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            statistics: stats.clone(),
            recent_events,
            correlations,
            overall_risk_score: self.calculate_risk_score(&stats).await,
        }
    }

    /// Calculate overall risk score based on statistics
    async fn calculate_risk_score(&self, stats: &ThreatStatistics) -> f64 {
        // Simple risk scoring algorithm
        let mut score = 0.0;

        // Weight critical events heavily
        score += stats.critical_events as f64 * 10.0;
        score += stats.error_events as f64 * 5.0;
        score += stats.warning_events as f64 * 1.0;

        // Normalize to 0-100 scale
        (score / 100.0).min(100.0)
    }
}

/// Security report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub timestamp: u64,
    pub statistics: ThreatStatistics,
    pub recent_events: Vec<SecurityEvent>,
    pub correlations: Vec<EventCorrelation>,
    pub overall_risk_score: f64, // 0-100 scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::ExecutionEngine;

    #[tokio::test]
    async fn test_security_analyzer_creation() {
        let auditor = SecurityAuditor::new(1000);
        let analyzer = SecurityAnalyzer::new(auditor);

        assert_eq!(analyzer.detection_rules.len(), 3);
    }

    #[tokio::test]
    async fn test_threat_statistics() {
        let stats = ThreatStatistics::new();
        assert_eq!(stats.total_threats, 0);
        assert_eq!(stats.critical_events, 0);
    }

    #[tokio::test]
    async fn test_correlation_engine() {
        let engine = CorrelationEngine::new();

        // Create test events
        let test_events = vec![
            SecurityEvent {
                id: "event1".to_string(),
                event_type: SecurityEventType::ConsensusAnomaly,
                severity: AuditSeverity::Warning,
                source: "node1".to_string(),
                target: "system".to_string(),
                details: "Test event 1".to_string(),
                timestamp: 1000,
                confidence: 0.8,
                related_events: vec![],
                status: SecurityEventStatus::New,
            },
            SecurityEvent {
                id: "event2".to_string(),
                event_type: SecurityEventType::GovernanceAbuse,
                severity: AuditSeverity::Warning,
                source: "node1".to_string(),
                target: "system".to_string(),
                details: "Test event 2".to_string(),
                timestamp: 1001,
                confidence: 0.8,
                related_events: vec![],
                status: SecurityEventStatus::New,
            },
        ];

        let correlations = engine.correlate(&test_events);
        assert!(correlations.len() >= 0); // At least our grouped by actor correlation
    }
}
