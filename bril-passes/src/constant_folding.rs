use crate::pass_manager::FunctionPass;
use bril_frontend::Literal;
use bril_ir::IrFunction;
use bril_ir::IrInstruction;
use std::collections::HashMap;

/// Intraprocedural Constant Fold
pub struct ConstantFoldPass {}

impl FunctionPass for ConstantFoldPass {
    fn name(&self) -> &str {
        "ConstantFoldPass"
    }

    fn run_on_function(&mut self, function: &mut IrFunction) -> bool {
        for blocks in function.blocks.iter_mut() {
            for instr in blocks.instrs.iter_mut() {
                match instr {
                    IrInstruction::Add { dest, lhs, rhs } => {
                        let right = rhs.parse::<i64>().unwrap();
                        let left = lhs.parse::<i64>().unwrap();
                        let sum = left + right;
                        *instr = IrInstruction::Const {
                            dest: dest.to_string(),
                            value: Literal::Int(sum),
                        };
                    }

                    IrInstruction::Mul { dest, lhs, rhs } => {}
                    _ => {}
                }
            }
        }
        true
    }
}
