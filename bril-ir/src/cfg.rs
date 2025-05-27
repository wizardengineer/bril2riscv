use anyhow::{bail, Result};
use bril_frontend::Function as BrilFunction;
use bril_frontend::Instruction as BrilInstr;
use bril_frontend::Program as BrilProgam;

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
        value: i64,
    },

    // == Misc ==
    Print {
        value: String,
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

/// Converting Flat IrFunctions into CFG
fn convert_to_cfg(functions: &BrilFunction) -> Result<IrFunction> {
    let mut blocks = Vec::new();

    // Pointer to the current block we're managing
    let mut current_block = IrBasicBlock {
        label: "entry".to_string(),
        instrs: Vec::new(),
    };

    for instr in &functions.instrs {
        match instr {
            BrilInstr::Label { label } => {
                // we're done with the current block
                blocks.push(current_block);

                // manage our new block
                current_block = IrBasicBlock {
                    label: label.clone(),
                    instrs: Vec::new(),
                };
            }

            BrilInstr::Const { dest, value, .. } => {
                current_block.instrs.push(IrInstruction::Const {
                    dest: dest.clone(),
                    value: *value,
                });
            }

            BrilInstr::Add { dest, args, .. } => {
                current_block.instrs.push(IrInstruction::Add {
                    dest: dest.clone(),
                    lhs: args[0].clone(),
                    rhs: args[1].clone(),
                });
            }

            BrilInstr::Print { args } => {
                current_block.instrs.push(IrInstruction::Print {
                    value: args[0].clone(),
                });
            }

            BrilInstr::Ret { args } => {
                current_block
                    .instrs
                    .push(IrInstruction::Ret { args: args.clone() });
            }

            //BrilInstr::Mul { dest, args, .. } => {}
            //BrilInstr::Sub { dest, args, .. } => {}
            //BrilInstr::Div { dest, args, .. } => {}
            _ => bail!("Not there yet buddy...wait {:?}", instr),
        }
    }

    // Push final block (current one)
    blocks.push(current_block);

    Ok(IrFunction {
        name: functions.name.clone(),
        args: functions
            .args
            .iter()
            .map(|arg| arg.value.clone()) // pull out each ValueDef.value
            .collect(),
        blocks,
    })
}
