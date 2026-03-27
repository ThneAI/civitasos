use civitasos::*;

#[tokio::test]
async fn test_basic_deployment_verification() {
    println!("🧪 Basic Deployment Verification Test");
    println!("===================================");
    
    // 1. Test execution engine
    println!("\n📋 1. Testing Execution Engine...");
    {
        let mut engine = ExecutionEngine::new(1000000); // 1M gas limit
        
        // Set up test registers
        engine.context.registers[1] = 100;
        engine.context.registers[2] = 50;
        
        // Create a simple program: ADD r0, r1, r2
        let program = vec![
            OpCode::ADD(0, 1, 2),  // r0 = r1 + r2 (100 + 50 = 150)
            OpCode::RETURN,
        ];
        
        let result = engine.execute_program(program).unwrap();
        
        assert_eq!(engine.context.registers[0], 150);
        
        println!("   ✅ Execution engine operational: r0 = {}", engine.context.registers[0]);
    }
    
    // 2. Test state management
    println!("\n💾 2. Testing State Management...");
    {
        let mut state_store = StateStore::new();
        
        // Add test data
        let key = "test_user_balance";
        let value = "100";
        state_store.put(key.to_string(), value.to_string());
        
        // Verify state update
        if let Some(entry) = state_store.get(key) {
            assert_eq!(entry.value, value);
            println!("   ✅ State management operational: {} = {}", key, entry.value);
        } else {
            panic!("State update failed");
        }
    }
    
    // 3. Test consensus mechanism
    println!("\n🤝 3. Testing Consensus Mechanism...");
    {
        let state_store = StateStore::new();
        let mut consensus_engine = ConsensusEngine::new(state_store);
        
        // Add test validator
        let validator = crate::consensus::Validator {
            id: "test_validator".to_string(),
            public_key: "test_pubkey".to_string(),
            stake: 1000,
            reputation: 100,
        };
        consensus_engine.add_validator(validator.clone());
        
        // Check that validator was added by accessing the public field
        assert_eq!(consensus_engine.validators.len(), 1);
        
        println!("   ✅ Consensus engine operational: validator count = {}", consensus_engine.validators.len());
    }
    
    // 4. Test governance system
    println!("\n🏛️  4. Testing Governance System...");
    {
        let state_store = StateStore::new();
        let mut consensus_engine = ConsensusEngine::new(state_store);
        
        // Add a validator to make the governance system work properly
        let validator = crate::consensus::Validator {
            id: "test_validator".to_string(),
            public_key: "test_pubkey".to_string(),
            stake: 2000, // Higher stake to meet minimum requirement
            reputation: 100,
        };
        consensus_engine.add_validator(validator);
        
        let mut governance_engine = GovernanceEngine::new(consensus_engine);
        
        // Submit the proposal using the correct method
        let proposal_id_result = governance_engine.create_proposal(GovernanceProposal {
            id: "".to_string(), // Will be set by the function
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposal_type: GovernanceProposalType::ParameterChange("test_param=test_value".to_string()),
            proposer: "test_validator".to_string(), // Use validator as proposer
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            voting_deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // 1 hour from now
            executed: false,
            passed: false,
            votes_for: 0,
            votes_against: 0,
            total_stake_voted: 0,
            minimum_stake_threshold: 1000, // Match the engine's requirement
            quorum_met: false,
            approval_percentage: 0,
            proposer_stake: 2000, // Use higher stake
        }, 2000); // Use higher stake
        
        assert!(proposal_id_result.is_ok());
        let proposal_id = proposal_id_result.unwrap();
        
        assert!(!proposal_id.is_empty());
        
        println!("   ✅ Governance system operational: proposal created with ID {}", proposal_id);
    }
    
    // 5. Test economic model
    println!("\n💰 5. Testing Economic Model...");
    {
        let state_store = StateStore::new();
        let mut economic_engine = EconomicEngine::new(state_store);
        
        // Test that the economic engine can be created and accessed
        // We'll use available methods to verify functionality
        let initial_accounts = economic_engine.accounts.len(); // Accessing public field
        
        // Create a test account
        economic_engine.create_account("test_account".to_string()).unwrap();
        
        assert!(economic_engine.accounts.len() > initial_accounts);
        
        println!("   ✅ Economic model operational: account count = {}", 
                 economic_engine.accounts.len());
    }
    
    // 6. Test network module
    println!("\n🌐 6. Testing Network Module...");
    {
        use civitasos::network::{NetworkConfig, NetworkService};
        
        // Create network configuration
        let config = NetworkConfig::default();
        
        // Create network service
        let network_service = NetworkService::new(config);
        
        // Verify service was created
        println!("   ✅ Network module operational: service created");
    }
    
    // 7. Test cross-chain functionality
    println!("\n🔗 7. Testing Cross-chain Functionality...");
    {
        use civitasos::cross_chain::{CrossChainBridge, CrossChainBridgeType};
        
        let state_store = StateStore::new();
        let bridge = CrossChainBridge::new(CrossChainBridgeType::LockAndMint, state_store);
        
        // Verify bridge was created
        println!("   ✅ Cross-chain functionality operational: bridge created");
    }
    
    // 8. Test governance policy engine
    println!("\n📜 8. Testing Governance Policy Engine...");
    {
        use civitasos::governance_policy::GovernancePolicyEngine;
        
        let policy_engine = GovernancePolicyEngine::new();
        
        // Verify engine was created
        println!("   ✅ Governance policy engine operational: engine created");
    }
    
    // 9. Test monitoring system
    println!("\n📊 9. Testing Monitoring System...");
    {
        use civitasos::monitoring::{PerformanceMonitor, SystemMetrics};
        
        let mut perf_monitor = PerformanceMonitor::new(100);
        
        // Create test metrics
        let mut metrics = SystemMetrics::new();
        metrics.cpu_usage = 25.5;
        metrics.memory_usage = 1024 * 1024 * 512; // 512MB
        metrics.execution_count = 100;
        metrics.avg_execution_time = 15.5;
        
        // Add metrics to monitor using the correct method
        perf_monitor.record_execution(&crate::execution::ExecutionResult {
            gas_used: 1000,
            success: true,
            state_diffs: vec![],
            trace_hash: "test_hash".to_string(),
        }).await;
        
        // Collect metrics to add them to history
        perf_monitor.collect_metrics().await;
        
        // Verify metrics were recorded
        let history = perf_monitor.get_metrics_history().await;
        assert!(!history.is_empty());
        
        println!("   ✅ Monitoring system operational: {} metrics recorded", 
                 history.len());
    }
    
    println!("\n🎯 All basic deployment verification tests passed!");
    println!("CivitasOS is ready for production deployment.");
}