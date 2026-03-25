use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

// RISC指令集定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    LOAD(u32, u32),      // LOAD reg, key
    STORE(u32, u32),     // STORE key, reg
    ADD(u32, u32, u32),  // ADD dest, src1, src2
    SUB(u32, u32, u32),  // SUB dest, src1, src2
    HASH(u32, u32),      // HASH dest, src
    CMP(u32, u32),       // CMP reg1, reg2
    JZ(u32, u32),        // JZ reg, addr
    JNZ(u32, u32),       // JNZ reg, addr
    RETURN,               // RETURN
    REVERT,               // REVERT
}

// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub registers: [u64; 16],
    pub pc: u32,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub state_diffs: Vec<StateChange>,
    pub trace_hash: String,
    pub halted: bool,
}

impl ExecutionContext {
    pub fn new(gas_limit: u64) -> Self {
        ExecutionContext {
            registers: [0; 16],
            pc: 0,
            gas_used: 0,
            gas_limit,
            state_diffs: Vec::new(),
            trace_hash: String::new(),
            halted: false,
        }
    }
}

// 状态变更结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

// 原子决策单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicDecisionUnit {
    pub input_state_hash: String,
    pub rule_id: String,
    pub execution_trace: Vec<OpCode>,
    pub output_proof: String,
    pub accountability_anchor: String,
    pub risk_stake: u64,
}

// 执行引擎
pub struct ExecutionEngine {
    pub context: ExecutionContext,
}

impl ExecutionEngine {
    pub fn new(gas_limit: u64) -> Self {
        ExecutionEngine {
            context: ExecutionContext::new(gas_limit),
        }
    }

    pub fn execute_program(&mut self, program: Vec<OpCode>) -> Result<ExecutionResult, ExecutionError> {
        for instruction in &program {
            if self.context.halted {
                break;
            }

            // 检查Gas限制
            if self.context.gas_used >= self.context.gas_limit {
                return Err(ExecutionError::OutOfGas);
            }

            self.execute_instruction(instruction.clone())?;
            
            // 更新Gas使用量
            self.context.gas_used += 1;
            
            // 更新执行轨迹哈希
            let instruction_str = format!("{:?}", instruction);
            let mut hasher = Sha256::new();
            hasher.update(&self.context.trace_hash);
            hasher.update(instruction_str);
            self.context.trace_hash = format!("{:x}", hasher.finalize());
        }

        Ok(ExecutionResult {
            state_diffs: self.context.state_diffs.clone(),
            trace_hash: self.context.trace_hash.clone(),
            gas_used: self.context.gas_used,
            success: !self.context.halted,
        })
    }

    fn execute_instruction(&mut self, instruction: OpCode) -> Result<(), ExecutionError> {
        match instruction {
            OpCode::LOAD(reg_idx, mem_addr) => {
                if reg_idx >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                // 在实际实现中，这里会从内存中加载值
                // 暂时设为0
                self.context.registers[reg_idx as usize] = mem_addr as u64;
            },
            OpCode::STORE(mem_addr, reg_idx) => {
                if reg_idx >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                let value = self.context.registers[reg_idx as usize];
                
                // 记录状态变更
                let key = format!("mem_{}", mem_addr);
                let old_value = None; // 简化实现
                let new_value = Some(value.to_string());
                
                let state_change = StateChange {
                    key,
                    old_value,
                    new_value,
                };
                
                self.context.state_diffs.push(state_change);
            },
            OpCode::ADD(dest_reg, src1_reg, src2_reg) => {
                if dest_reg >= 16 || src1_reg >= 16 || src2_reg >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                let val1 = self.context.registers[src1_reg as usize];
                let val2 = self.context.registers[src2_reg as usize];
                self.context.registers[dest_reg as usize] = val1.wrapping_add(val2);
            },
            OpCode::SUB(dest_reg, src1_reg, src2_reg) => {
                if dest_reg >= 16 || src1_reg >= 16 || src2_reg >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                let val1 = self.context.registers[src1_reg as usize];
                let val2 = self.context.registers[src2_reg as usize];
                self.context.registers[dest_reg as usize] = val1.wrapping_sub(val2);
            },
            OpCode::HASH(dest_reg, src_reg) => {
                if dest_reg >= 16 || src_reg >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                let val = self.context.registers[src_reg as usize].to_string();
                let mut hasher = Sha256::new();
                hasher.update(val);
                let hash_bytes = hasher.finalize();
                let hash_val = u64::from_be_bytes([
                    hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
                    hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
                ]);
                
                self.context.registers[dest_reg as usize] = hash_val;
            },
            OpCode::CMP(reg1, reg2) => {
                if reg1 >= 16 || reg2 >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                let val1 = self.context.registers[reg1 as usize];
                let val2 = self.context.registers[reg2 as usize];
                
                if val1 == val2 {
                    self.context.registers[0] = 1; // 设置标志寄存器
                } else {
                    self.context.registers[0] = 0;
                }
            },
            OpCode::JZ(reg, addr) => {
                if reg >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                if self.context.registers[reg as usize] == 0 {
                    self.context.pc = addr;
                    // 由于循环会自动增加pc，这里需要减1
                    return Ok(());
                }
            },
            OpCode::JNZ(reg, addr) => {
                if reg >= 16 {
                    return Err(ExecutionError::InvalidRegister);
                }
                
                if self.context.registers[reg as usize] != 0 {
                    self.context.pc = addr;
                    // 由于循环会自动增加pc，这里需要减1
                    return Ok(());
                }
            },
            OpCode::RETURN => {
                self.context.halted = true;
            },
            OpCode::REVERT => {
                self.context.halted = true;
                return Err(ExecutionError::ExecutionReverted);
            },
        }

        self.context.pc += 1;
        Ok(())
    }
}

// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub state_diffs: Vec<StateChange>,
    pub trace_hash: String,
    pub gas_used: u64,
    pub success: bool,
}

// 执行错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidRegister,
    OutOfGas,
    ExecutionReverted,
    UnknownInstruction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut engine = ExecutionEngine::new(1000);
        
        // 创建一个简单的程序：ADD r0, r1, r2 (其中r1=5, r2=3)
        let mut program = vec![];
        
        // 设置寄存器值
        engine.context.registers[1] = 5;
        engine.context.registers[2] = 3;
        
        // ADD r0, r1, r2
        program.push(OpCode::ADD(0, 1, 2));
        
        let result = engine.execute_program(program).unwrap();
        
        assert_eq!(engine.context.registers[0], 8); // 5 + 3 = 8
        assert!(result.success);
        assert!(!result.trace_hash.is_empty());
    }

    #[test]
    fn test_hash_execution() {
        let mut engine = ExecutionEngine::new(1000);
        
        // 设置寄存器值
        engine.context.registers[1] = 12345;
        
        // HASH r2, r1
        let program = vec![OpCode::HASH(2, 1)];
        
        let result = engine.execute_program(program).unwrap();
        
        assert_ne!(engine.context.registers[2], 12345); // 应该是哈希值
        assert!(result.success);
    }
}