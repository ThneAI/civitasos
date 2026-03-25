use civitasos::{ExecutionEngine, OpCode, StateStore};

#[test]
fn test_complete_execution_flow() {
    // 创建执行引擎
    let mut engine = ExecutionEngine::new(1000);
    
    // 初始化一些寄存器值
    engine.context.registers[1] = 10;
    engine.context.registers[2] = 20;
    
    // 创建一个简单的程序：将两个数相加
    let program = vec![
        OpCode::ADD(0, 1, 2),  // r0 = r1 + r2 (10 + 20 = 30)
        OpCode::STORE(100, 0), // 将结果存储到内存位置100
    ];
    
    // 执行程序
    let result = engine.execute_program(program).unwrap();
    
    // 验证执行结果
    assert!(result.success);
    assert_eq!(engine.context.registers[0], 30); // 检查计算结果
    assert!(!result.trace_hash.is_empty()); // 检查轨迹哈希生成
    
    // 创建状态存储并应用执行结果
    let mut state_store = StateStore::new();
    let state_root = state_store.apply_diff(result.state_diffs).unwrap();
    
    // 验证状态根哈希
    assert!(!state_root.is_empty());
    
    println!("Basic execution flow test passed!");
    println!("Trace hash: {}", result.trace_hash);
    println!("State root: {}", state_root);
}

#[test]
fn test_deterministic_execution() {
    // 验证相同输入产生相同输出
    let mut engine1 = ExecutionEngine::new(1000);
    let mut engine2 = ExecutionEngine::new(1000);
    
    // 设置相同的初始状态
    engine1.context.registers[1] = 5;
    engine1.context.registers[2] = 7;
    
    engine2.context.registers[1] = 5;
    engine2.context.registers[2] = 7;
    
    let program = vec![
        OpCode::ADD(0, 1, 2),
        OpCode::RETURN,
    ];
    
    let result1 = engine1.execute_program(program.clone()).unwrap();
    let result2 = engine2.execute_program(program).unwrap();
    
    // 验证两个执行结果相同
    assert_eq!(result1.trace_hash, result2.trace_hash);
    assert_eq!(result1.gas_used, result2.gas_used);
    assert_eq!(engine1.context.registers[0], engine2.context.registers[0]);
    
    println!("Deterministic execution test passed!");
    println!("Trace hash: {}", result1.trace_hash);
}