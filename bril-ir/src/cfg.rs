use crate::BlockID;
use anyhow::Result;
use bril_frontend::Function as BrilFunction;
use bril_frontend::Instruction as BrilInstr;
use bril_frontend::Literal;
use bril_frontend::Op;
use bril_frontend::Program as BrilProgam;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub args: Vec<String>,
    pub blocks: Vec<IrBasicBlock>,
    pub label_to_idx: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
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
        dest: Option<String>,
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

    Phi {
        dest: String,                 // value the be dictated by previous values
        sources: Vec<Option<String>>, // this will store the blocks id of preds for blocks
    },

    // == Literals ==
    Const {
        dest: String,
        value: Literal,
    },

    // == Misc ==
    Print {
        values: Vec<String>,
    },

    Assign {
        lhs: String,
        rhs: String,
    },
}

impl IrInstruction {
    // Returns a slice of a defined variable
    // describes what name does this instruction *write*
    pub fn defs(&self) -> &[String] {
        match self {
            IrInstruction::Add { dest, .. }
            | IrInstruction::Sub { dest, .. }
            | IrInstruction::Mul { dest, .. }
            | IrInstruction::Div { dest, .. }
            | IrInstruction::Eq { dest, .. }
            | IrInstruction::Lt { dest, .. }
            | IrInstruction::Gt { dest, .. }
            | IrInstruction::Le { dest, .. }
            | IrInstruction::Ge { dest, .. }
            | IrInstruction::Or { dest, .. }
            | IrInstruction::And { dest, .. }
            | IrInstruction::Not { dest, .. }
            | IrInstruction::Const { dest, .. }
            // TODO: Maybe we should remove the assign?
            // Find something else to use
            | IrInstruction::Assign { lhs: dest, .. }
            | IrInstruction::Phi { dest, .. } => std::slice::from_ref(dest),

            IrInstruction::Call { dest, .. } => {
                if let Some(d) = dest {
                    std::slice::from_ref(d)
                } else {
                    &[]
                }
            },

            _ => &[],
        }
    }

    // describes what name does this instruction *reads*
    pub fn uses(&self) -> Vec<String> {
        match self {
            IrInstruction::Add { lhs, rhs, .. }
            | IrInstruction::Sub { lhs, rhs, .. }
            | IrInstruction::Mul { lhs, rhs, .. }
            | IrInstruction::Div { lhs, rhs, .. }
            | IrInstruction::Eq { lhs, rhs, .. }
            | IrInstruction::Lt { lhs, rhs, .. }
            | IrInstruction::Gt { lhs, rhs, .. }
            | IrInstruction::Ge { lhs, rhs, .. }
            | IrInstruction::Le { lhs, rhs, .. }
            | IrInstruction::Or { lhs, rhs, .. }
            | IrInstruction::And { lhs, rhs, .. } => vec![lhs.to_string(), rhs.to_string()],

            IrInstruction::Not { args, .. } => vec![args.to_string()],

            IrInstruction::Br { cond, .. } => vec![cond.to_string()],
            IrInstruction::Call { args, .. } => args.to_vec(),
            IrInstruction::Ret { args, .. } => args.to_vec(),
            IrInstruction::Phi { sources, .. } => sources.iter().flatten().cloned().collect(),

            IrInstruction::Print { values, .. } => values.to_vec(),
            _ => Vec::new(),
        }
    }
}

/// For getting the mapping of each variable block(s) where variable might be defined
pub fn collect_defs(func: &IrFunction) -> HashMap<String, Vec<BlockID>> {
    let mut defs_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (block_idx, block) in func.blocks.iter().enumerate() {
        for instr in &block.instrs {
            for var in instr.defs() {
                defs_map.entry(var.clone()).or_default().push(block_idx);
            }
        }
    }

    defs_map
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
                        dest: Some(dest.as_ref().unwrap().clone()),
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
                        values: args.to_vec(),
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
