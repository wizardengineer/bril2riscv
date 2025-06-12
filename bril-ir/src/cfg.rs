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
    pub label_to_idx: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct IrBasicBlock {
    pub label: String,
    pub instrs: Vec<IrInstruction>,
    pub preds: Vec<usize>,
    pub succs: Vec<usize>,
}

impl IrFunction {
    // just in case i was to do some testing
    pub fn new(func_name: &str) -> Self {
        Self {
            name: func_name.to_string(),
            args: Vec::new(),
            blocks: Vec::new(),
            label_to_idx: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, label: &str) -> usize {
        // current block we're on
        let idx = self.blocks.len();

        self.blocks.push(IrBasicBlock {
            label: label.to_string(),
            instrs: Vec::new(),
            preds: Vec::new(),
            succs: Vec::new(),
        });

        // build our label to index mapping, for each
        // block we add to the Block vectors
        self.label_to_idx.insert(label.to_string(), idx);

        // return index of newly added block index
        idx
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.blocks[from].succs.push(to);
        self.blocks[to].preds.push(from);
    }

    pub fn append_instr(&mut self, idx: usize, instr: &IrInstruction) {
        self.blocks[idx].instrs.push(instr.clone());
    }

    pub fn block_index(&self, label: &String) -> Option<usize> {
        self.label_to_idx.get(label).copied()
    }
}

#[derive(Debug, Clone)]
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
fn convert_to_cfg(func: &BrilFunction) -> Result<IrFunction> {
    let mut ir_func = IrFunction::new(&func.name);
    split_into_blocks(&mut ir_func, func)?;

    wire_block_edges(&mut ir_func)?;

    Ok(ir_func)
}

/// This functions deals with converting the IR into true
/// Control-Flow Graphs by wiring up the blocks
fn wire_block_edges(func: &mut IrFunction) -> Result<()> {
    // Build up the list of Successors & Predecessors fork
    for curr_block_idx in 0..func.blocks.len() {
        if let Some(terminator) = func.blocks[curr_block_idx].instrs.last() {
            match terminator {
                IrInstruction::Br {
                    then_lbl, else_lbl, ..
                } => {
                    let then_idx = func.block_index(then_lbl).unwrap();
                    let else_idx = func.block_index(else_lbl).unwrap();

                    func.add_edge(curr_block_idx, then_idx);
                    func.add_edge(curr_block_idx, else_idx);
                }

                IrInstruction::Jmp { label } => {
                    let target_idx = func.block_index(label).unwrap();
                    func.add_edge(curr_block_idx, target_idx);
                }

                // TODO: I think I'll need to manage this later on?
                IrInstruction::Ret { .. } => {}

                // Fall through the next label, if needed so
                _ => {
                    // check to see if we're still within the range of the blocks list
                    if curr_block_idx + 1 < func.blocks.len() - 1 {
                        func.add_edge(curr_block_idx, curr_block_idx + 1);
                    }
                }
            }
        }
    }

    Ok(())
}

fn split_into_blocks(func: &mut IrFunction, bril_func: &BrilFunction) -> Result<()> {
    // Pointer to current block we'll be indexing in
    let mut current_idx = func.add_block("entry");

    // 2) Now walk each Bril instruction in order:
    let bril_instrs = &bril_func.instrs;
    for instr in bril_instrs {
        match instr {
            BrilInstr::Label { label } => {
                // Whenever we see a Bril label, start a new block with that name:
                // (subsequent instructions go into this new block)
                current_idx = func.add_block(&label);
            }

            BrilInstr::Op(op) => {
                // Translate each Bril “op” into an IrInstruction instance.
                let ir_inst = match op {
                    Op::Const { dest, value, .. } => IrInstruction::Const {
                        dest: dest.clone(),
                        value: value.clone(),
                    },

                    // == Arithmetic ==
                    Op::Add { dest, args, .. } => IrInstruction::Add {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Mul { dest, args, .. } => IrInstruction::Mul {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Sub { dest, args, .. } => IrInstruction::Sub {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Div { dest, args, .. } => IrInstruction::Div {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    // == Comparison ==
                    Op::Eq { dest, args, .. } => IrInstruction::Eq {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Lt { dest, args, .. } => IrInstruction::Lt {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Gt { dest, args, .. } => IrInstruction::Gt {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Ge { dest, args, .. } => IrInstruction::Ge {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::Le { dest, args, .. } => IrInstruction::Le {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    // == Logical ==
                    Op::Not { dest, args } => IrInstruction::Not {
                        dest: dest.clone(),
                        args: args[0].clone(),
                    },

                    Op::Or { dest, args } => IrInstruction::Or {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    Op::And { dest, args } => IrInstruction::And {
                        dest: dest.clone(),
                        lhs: args[0].clone(),
                        rhs: args[1].clone(),
                    },

                    // == Control Flow ==
                    Op::Call {
                        dest, args, funcs, ..
                    } => IrInstruction::Call {
                        target_func: funcs[0].clone(),
                        args: args.clone(),
                        dest: dest.as_ref().unwrap().clone(),
                    },

                    Op::Br { args, labels } => IrInstruction::Br {
                        cond: args[0].clone(),
                        then_lbl: labels[0].clone(),
                        else_lbl: labels[1].clone(),
                    },

                    Op::Jmp { labels } => IrInstruction::Jmp {
                        label: labels[0].clone(),
                    },

                    Op::Ret { args } => IrInstruction::Ret { args: args.clone() },

                    // == Misc ==
                    Op::Print { args } => IrInstruction::Print {
                        value: args[0].clone(),
                    },

                    Op::Id { dest, args, .. } => IrInstruction::Assign {
                        lhs: dest.clone(),
                        rhs: args[0].clone(),
                    },

                    other => {
                        panic!(
                            "Unimplemented Bril opcode in split_into_blocks: {:?}",
                            other
                        );
                    }
                };

                // 3) Append the newly created IR instruction into the “current” block
                func.append_instr(current_idx, &ir_inst);
            }
        }
    }

    Ok(())
}
