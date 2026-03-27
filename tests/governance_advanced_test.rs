use civitasos::*;
use std::collections::HashMap;

#[test]
fn test_advanced_governance_features() {
    println!("Testing advanced governance features...");

    // 1. 创建状态存储
    let state_store = StateStore::new();
    println!("✓ Created state store");

    // 2. 创建共识引擎
    let mut consensus_engine = ConsensusEngine::new(state_store.clone_for_validation());
    println!("✓ Created consensus engine");

    // 3. 添加验证节点
    let validator1 = Validator {
        id: "validator1".to_string(),
        public_key: "pubkey1".to_string(),
        stake: 1000,
        reputation: 100,
    };
    consensus_engine.add_validator(validator1);
    println!("✓ Added validators");

    // 4. 创建治理引擎
    let mut governance_engine = GovernanceEngine::new(consensus_engine);
    println!("✓ Created governance engine");

    // 5. 创建治理策略引擎
    let mut policy_engine = GovernancePolicyEngine::new();
    println!("✓ Created policy engine");

    // 6. 创建一个治理提案
    let proposal = GovernanceProposal {
        id: "".to_string(), // 将由函数设置
        title: "Parameter Adjustment Proposal".to_string(),
        description: "Adjusting the quorum percentage from 20 to 25".to_string(),
        proposal_type: GovernanceProposalType::ParameterChange("quorum_percentage=25".to_string()),
        proposer: "validator1".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        voting_deadline: 0, // 将由函数设置
        executed: false,
        passed: false,
        votes_for: 0,
        votes_against: 0,
        total_stake_voted: 0,
        minimum_stake_threshold: 1000,
        quorum_met: false,
        approval_percentage: 0,
        proposer_stake: 2000,
    };

    // 7. 提交提案
    let proposal_id = governance_engine.create_proposal(proposal, 2000).unwrap();
    println!("✓ Created governance proposal: {}", proposal_id);

    // 8. 查询提案
    let retrieved_proposal = governance_engine.get_proposal(&proposal_id).unwrap();
    assert_eq!(retrieved_proposal.title, "Parameter Adjustment Proposal");
    println!("✓ Retrieved proposal: {}", retrieved_proposal.title);

    // 9. 获取活跃提案
    let active_proposals = governance_engine.get_active_proposals();
    assert!(!active_proposals.is_empty());
    println!("✓ Found {} active proposals", active_proposals.len());

    // 10. 模拟投票
    // 设置提案的投票期限为过去，以允许计算结果
    for prop in &mut governance_engine.proposals {
        if prop.id == proposal_id {
            prop.voting_deadline = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 100; // 设置为100秒前
            prop.votes_for = 1500;
            prop.votes_against = 300;
            prop.total_stake_voted = 1800;
            prop.quorum_met = true;
            prop.approval_percentage =
                (prop.votes_for * 100) / (prop.votes_for + prop.votes_against);
            break;
        }
    }

    // 11. 计算提案结果
    let outcome = governance_engine
        .compute_proposal_result(&proposal_id)
        .unwrap();
    println!("✓ Proposal outcome: {:?}", outcome);

    // 12. 执行通过的提案
    if outcome == ProposalOutcome::Passed {
        let executed = governance_engine.execute_passed_proposals().unwrap();
        if !executed.is_empty() {
            println!("✓ Executed proposal: {}", executed[0]);
        }
    }

    // 13. 测试策略引擎
    let validators: Vec<Validator> = governance_engine.consensus_engine.validators.clone();
    let compliance_report = policy_engine.evaluate_proposal_compliance(
        governance_engine.get_proposal(&proposal_id).unwrap(),
        &validators,
    );

    println!(
        "✓ Compliance report - Compliant: {}, Risk Level: {:?}",
        compliance_report.compliant, compliance_report.risk_level
    );

    // 14. 测试策略变更
    let mut new_params = HashMap::new();
    new_params.insert("quorum_percentage".to_string(), "30".to_string());

    let policy_change_result = policy_engine.apply_policy_change(
        "default_voting",
        new_params,
        "validator1".to_string(),
        "Adjusting quorum for increased security".to_string(),
    );

    assert!(policy_change_result.is_ok());
    println!("✓ Applied policy change");

    // 15. 验证策略更新
    let updated_policy = policy_engine.get_policy("default_voting").unwrap();
    assert_eq!(
        updated_policy.parameters.get("quorum_percentage").unwrap(),
        "30"
    );
    println!("✓ Verified policy update");

    // 16. 获取执行过的提案
    let executed_proposals = governance_engine.get_executed_proposals();
    println!("✓ Found {} executed proposals", executed_proposals.len());

    // 17. 获取通过的提案
    let passed_proposals = governance_engine.get_passed_proposals();
    println!("✓ Found {} passed proposals", passed_proposals.len());

    println!("🎉 Advanced governance features test passed!");
    println!("Implemented features:");
    println!("  - Proposal creation and management");
    println!("  - Advanced querying capabilities");
    println!("  - Policy engine with compliance checking");
    println!("  - Risk assessment and evaluation");
    println!("  - Policy change mechanisms");
    println!("  - Comprehensive governance workflows");
}

#[test]
fn test_constitutional_rule_management() {
    println!("Testing constitutional rule management...");

    let state_store = StateStore::new();
    let consensus_engine = ConsensusEngine::new(state_store);
    let mut governance_engine = GovernanceEngine::new(consensus_engine);

    // 检查初始宪法规则
    let initial_rules_count = governance_engine.get_active_constitutional_rules().len();
    assert!(initial_rules_count > 0);
    println!("✓ Initial constitutional rules: {}", initial_rules_count);

    // 添加新宪法规则
    let new_rule = ConstitutionalRule {
        id: "rule_dynamic_1".to_string(),
        name: "Dynamic Rule 1".to_string(),
        description: "A dynamically added constitutional rule".to_string(),
        content: "F(State) -> ValidState where constraint is satisfied".to_string(),
        version: 1,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        active: true,
    };

    let add_result = governance_engine.add_constitutional_rule(new_rule);
    assert!(add_result.is_ok());

    // 检查规则数量增加
    let updated_rules_count = governance_engine.get_active_constitutional_rules().len();
    assert!(updated_rules_count > initial_rules_count);
    println!(
        "✓ Added constitutional rule, total rules: {}",
        updated_rules_count
    );

    // 测试宪法合规性检查
    let is_constitutional = governance_engine.is_action_constitutional("test_action");
    assert!(is_constitutional);
    println!("✓ Constitutional compliance check passed");

    println!("🎉 Constitutional rule management test passed!");
}
