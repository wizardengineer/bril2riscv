use anyhow::{bail, Result};
use bril_frontend::Function as BrilFunction;
use bril_frontend::Instruction as BrilInstr;
use bril_frontend::Literal;
use bril_frontend::Op;
use bril_frontend::Program as BrilProgam;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
}

#[derive(Debug)]
pub struct IrFunction {
    pub name: String,
    pub args: Vec<String>,
    pub blocks: Vec<IrBasicBlock>,
}

#[derive(Debug)]
pub struct IrBasicBlock {
    pub label: String,
    pub instrs: Vec<IrInstruction>,
    preds: Vec<String>,
    succs: Vec<String>,
}

#[derive(Debug)]
pub enum IrInstruction {
    // == Arithematic ==
    Add {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Mul {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Sub {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Div {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Comparsion ==
    Eq {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Lt {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Gt {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Ge {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Le {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Logical Operator ==
    Not {
        dest: String,
        args: String,
    },

    Or {
        dest: String,
        lhs: String,
        rhs: String,
    },

    And {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Control Flow ==
    Call {
        target_func: String,
        args: Vec<String>,
        dest: String,
    },

    Br {
        cond: String,
        then_lbl: String,
        else_lbl: String,
    },

    Jmp {
        label: String,
    },

    Ret {
        args: Vec<String>,
    },

    // == Literals ==
    Const {
        dest: String,
        value: Literal,
    },

    // == Misc ==
    Print {
        value: String,
    },

    Assign {
        lhs: String,
        rhs: String,
    },
}

impl TryFrom<&BrilProgam> for IrModule {
    type Error = anyhow::Error;

    fn try_from(program: &BrilProgam) -> Result<Self> {
        let mut functions: Vec<IrFunction> = Vec::with_capacity(program.functions.len());

        for func in &program.functions {
            functions.push(convert_to_cfg(func)?);
        }

        Ok(IrModule { functions })
    }
}

/// Converting Flat Functions into CFG
/// TODO: Need to wire up edges for each block
fn convert_to_cfg(functions: &BrilFunction) -> Result<IrFunction> {
    let mut blocks = split_into_blocks(&functions.instrs)?;

    // Build label to index mapping
    let mut label_map = HashMap::new();
    for (i, block) in blocks.iter().enumerate() {
        label_map.insert(block.label.clone(), i);
    }

    // Connecting the block edges for both Successors & Predecessors
    connect_block_edges(&mut blocks, &label_map)?;

    Ok(IrFunction {
        name: functions.name.clone(),
        args: functions
            .args
            .iter()
            .map(|arg| arg.name.clone()) // pull out each ValueDef.name
            .collect(),
        blocks,
    })
}

fn connect_block_edges(
    blocks: &mut [IrBasicBlock],
    label_map: &HashMap<String, usize>,
) -> Result<()> {
    for i in 0..blocks.len() {
        // Helps with determining if we should fall through the label
        let fallthrough_lbl = if i + 1 < blocks.len() {
            Some(blocks[i + 1].label.clone())
        } else {
            None
        };

        let block = &mut blocks[i];

        if let Some(terminator) = block.instrs.last() {
            match terminator {
                IrInstruction::Br {
                    then_lbl, else_lbl, ..
                } => {
                    block.succs.push(then_lbl.clone());
                    block.succs.push(else_lbl.clone());
                }

                IrInstruction::Jmp { label } => {
                    block.succs.push(label.clone());
                }

                IrInstruction::Ret { .. } => {}

                // Fall through the next label, if needed so
                _ => {
                    if let Some(next_lbl) = fallthrough_lbl {
                        block.succs.push(next_lbl);
                    }
                }
            }
        }
    }

    // Build up the list of predecessors
    let mut list_of_preds = Vec::new();
    for block in &*blocks {
        for succs_lbl in &block.succs {
            let idx = label_map[succs_lbl];
            list_of_preds.push((idx, block.label.clone()));
        }
    }

    // Connect the list of predecessors
    for (idx, preds_lbl) in list_of_preds {
        blocks[idx].preds.push(preds_lbl);
    }

    Ok(())
}

fn split_into_blocks(instrs: &Vec<BrilInstr>) -> Result<Vec<IrBasicBlock>> {
    let mut blocks = Vec::new();

    // Pointer to the current block we're managing
    let mut current_block = IrBasicBlock {
        label: "entry".to_string(),
        instrs: Vec::new(),
        preds: Vec::new(),
        succs: Vec::new(),
    };

    for instr in instrs {
        match instr {
            BrilInstr::Label { label } => {
                // we're done with the current block
                blocks.push(current_block);

                // manage our new block
                current_block = IrBasicBlock {
                    label: label.clone(),
                    instrs: Vec::new(),
                    preds: Vec::new(),
                    succs: Vec::new(),
                };
            }

            BrilInstr::Op(op) => match op {
                Op::Const { dest, value, .. } => {
                    current_block.instrs.push(IrInstruction::Const {
                        dest: dest.clone(),
                        value: value.clone(),
                    });
                }

                // == Arithematic ==
                Op::Add { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Add {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Mul { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Mul {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Sub { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Sub {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Div { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Div {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                // == Comparsion ==
                Op::Eq { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Eq {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Lt { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Lt {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Gt { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Gt {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Ge { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Ge {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::Le { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Le {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                // == Logical Operator ==
                Op::Not { dest, args } => {
                    current_block.instrs.push(IrInstruction::Not {
                        dest: dest.clone(),
                        args: args[0].clone(),
                    });
                }

                Op::Or { dest, args } => {
                    current_block.instrs.push(IrInstruction::Or {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                Op::And { dest, args } => {
                    current_block.instrs.push(IrInstruction::And {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    });
                }

                // == Misc ==
                Op::Print { args } => {
                    current_block.instrs.push(IrInstruction::Print {
                        value: args[0].clone(),
                    });
                }

                Op::Id { dest, args, .. } => {
                    current_block.instrs.push(IrInstruction::Assign {
                        rhs: args[0].clone(),
                        lhs: dest.clone(),
                    });
                }
                // == Control flow ==
                Op::Ret { args } => {
                    current_block
                        .instrs
                        .push(IrInstruction::Ret { args: args.clone() });
                }

                Op::Jmp { labels } => {
                    current_block.instrs.push(IrInstruction::Jmp {
                        label: labels[0].clone(),
                    });
                }

                Op::Call {
                    dest, args, funcs, ..
                } => {
                    current_block.instrs.push(IrInstruction::Call {
                        target_func: funcs[0].clone(),
                        args: args.clone(),
                        dest: dest.as_ref().unwrap().to_string(),
                    });
                }

                Op::Br { args, labels } => {
                    current_block.instrs.push(IrInstruction::Br {
                        cond: args[0].clone(),
                        then_lbl: labels[0].clone(),
                        else_lbl: labels[1].clone(),
                    });
                }

                _ => bail!("Not there yet buddy...wait {:?}", instr),
            },
        }
    }

    // Push final block (current one)
    blocks.push(current_block);

    Ok(blocks)
}
