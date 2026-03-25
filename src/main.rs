use civitasos::{ExecutionEngine, OpCode, StateStore};

fn main() {
    println!("CivitasOS MVP - Week 1-2 Demo");
    println!("===============================");
    
    // 演示基本执行功能
    demonstrate_execution_engine();
    println!();
    
    // 演示状态管理功能
    demonstrate_state_management();
    println!();
    
    // 演示完整的执行流程
    demonstrate_complete_flow();
}

fn demonstrate_execution_engine() {
    println!("1. 执行引擎演示:");
    
    let mut engine = ExecutionEngine::new(1000);
    
    // 设置初始寄存器值
    engine.context.registers[1] = 15;
    engine.context.registers[2] = 25;
    
    // 创建一个简单的程序：ADD r0, r1, r2
    let program = vec![
        OpCode::ADD(0, 1, 2),  // r0 = r1 + r2 (15 + 25 = 40)
        OpCode::RETURN,
    ];
    
    match engine.execute_program(program) {
        Ok(result) => {
            println!("   程序执行成功!");
            println!("   结果寄存器 r0 = {}", engine.context.registers[0]);
            println!("   使用Gas: {}", result.gas_used);
            println!("   轨迹哈希: {}", &result.trace_hash[..16]); // 只显示前16个字符
            println!("   执行成功: {}", result.success);
        }
        Err(e) => {
            println!("   程序执行失败: {:?}", e);
        }
    }
}

fn demonstrate_state_management() {
    println!("2. 状态管理演示:");
    
    let mut state_store = StateStore::new();
    
    // 添加一些键值对
    state_store.put("balance:user1".to_string(), "100".to_string());
    state_store.put("balance:user2".to_string(), "200".to_string());
    
    // 获取值
    if let Some(value) = state_store.get("balance:user1") {
        println!("   user1 余额: {}", value.value);
    }
    
    // 更新值
    state_store.put("balance:user1".to_string(), "150".to_string());
    
    if let Some(value) = state_store.get("balance:user1") {
        println!("   user1 余额更新后: {}", value.value);
    }
    
    // 显示状态根哈希
    println!("   当前状态根哈希: {}", &state_store.get_root_hash()[..16]);
    println!("   历史版本数量: {}", state_store.versions.len());
}

fn demonstrate_complete_flow() {
    println!("3. 完整执行流程演示:");
    
    // 创建执行引擎
    let mut engine = ExecutionEngine::new(1000);
    
    // 设置初始值
    engine.context.registers[1] = 100; // 初始值
    engine.context.registers[2] = 10;  // 减数
    
    // 创建程序：从100减去10，然后存储结果
    let program = vec![
        OpCode::SUB(0, 1, 2),           // r0 = r1 - r2 (100 - 10 = 90)
        OpCode::STORE(1, 0),            // 存储结果到内存位置1
        OpCode::RETURN,
    ];
    
    match engine.execute_program(program) {
        Ok(result) => {
            println!("   执行成功，生成 {} 个状态变更", result.state_diffs.len());
            println!("   最终结果: {}", engine.context.registers[0]);
            println!("   轨迹哈希: {}", &result.trace_hash[..16]);
            
            // 创建状态存储并应用结果
            let mut state_store = StateStore::new();
            match state_store.apply_diff(result.state_diffs) {
                Ok(new_root) => {
                    println!("   状态更新成功!");
                    println!("   新状态根: {}", &new_root[..16]);
                }
                Err(e) => {
                    println!("   状态更新失败: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("   执行失败: {:?}", e);
        }
    }
}