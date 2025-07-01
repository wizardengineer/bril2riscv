use bril_frontend::Literal;
use bril_ir::IrFunction;
use pass_manager::FunctionPass;
use std::collection::HashMap;

struct ConstantPropagationPass {}

impl FunctionPass for ConstantPropagationPass {
    fn name(&self) -> &str {
        "ConstantPropagationPass"
    }

    fn run_on_function(&mut self, function: &mut IrFunction) {
        let const_env: HashMap<String, Option<i64>> = HashMap::new();
        for blocks in function.blocks {
            for instr in blocks.instrs {
                match instr {
                    // TODO: Need to add more patterns to match for
                    IrInstruction::Const { dest, value } => {
                        const_env.insert(dest.clone(), Literal::Int(*value))
                    }

                    IrInstruction::Add { dest, lhs, rhs }
                    | IrInstruction::Mul { dest, lhs, rhs }
                    | IrInstruction::Sub { dest, lhs, rhs }
                    | IrInstruction::Div { dest, lhs, rhs } => {
                        if Some(Literal::Int(imm_var)) = const_env.get(lhs) {
                            *lhs = imm_var.to_string();
                        }
                        if Some(Literal::Int(imm_var)) = const_env.get(rhs) {
                            *rhs = imm_var.to_string();
                        }
                    }

                    IrInstruction::Add { dest, .. }
                    | IrInstruction::Mul { dest, .. }
                    | IrInstruction::Sub { dest, .. }
                    | IrInstruction::Div { dest, .. } => {
                        const_env.remove(dest);
                    }
                }
            }
        }
        true
    }
}
