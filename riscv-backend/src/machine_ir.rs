use bril_ir::{BlockID, IrFunction};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct MachineFunc {
    pub name: String,
    pub args: Vec<VReg>,
    pub blocks: Vec<MachineBlock>,
    pub label_to_idx: HashMap<String, usize>,
}

impl MachineFunc {
    pub fn new(func: &IrFunction) -> Self {
        Self {
            name: func.name.to_string(),
            args: Vec::new(),
            blocks: Vec::new(),
            label_to_idx: func.label_to_idx.clone(),
        }
    }

    pub fn block_index(&self, label: &String) -> Option<usize> {
        self.label_to_idx.get(label).copied()
    }
}

#[derive(Debug, Clone)]
pub struct MachineBlock {
    pub name: String,
    pub instrs: Vec<MachineInstr>,
    pub succs: Vec<BlockID>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VReg {
    Virtual(i32),
    // Temp registers
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,

    // Function arguments
    A0, // function argument 0 / return value 0
    A1, // function argument 1 / return value 1
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,

    // Saved registers
    S0, // frame pointer
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,

    // Return address value
    RA,

    // Stack pointer & Frame pointer
    SP,
    FP,

    // Global Register
    GP,
}

/// Machine Instructions, 1:1 to RiscV
#[derive(Debug, Clone)]
pub enum MachineInstr {
    // R1 = R2 + Imm
    Addi { rd: VReg, rs1: VReg, imm: i64 },

    Add { rd: VReg, rs1: VReg, rs2: VReg },

    Mul { rd: VReg, rs1: VReg, rs2: VReg },

    Sub { rd: VReg, rs1: VReg, rs2: VReg },

    Div { rd: VReg, rs1: VReg, rs2: VReg },

    // Load & Store
    // t1 = 1
    Li { rd: VReg, imm: i64 },

    Mv { rd: VReg, rs1: VReg },
    // Control flow Instructions
    // May not be needed? Seems we can use
    // Pseudoinstructions like Call or Ret
    Jal { rd: VReg, offset: usize },

    // Unconditional jump
    Jmp { label: String },

    Beqz { rs1: VReg, label: String },

    Beq { rs1: VReg, rs2: VReg, label: String },

    Ret { rd: VReg },

    Call { func: String },

    Print { args: Vec<VReg> },
    // TODO: Add more instructions
}

impl MachineInstr {
    pub fn defs(&self) -> Vec<VReg> {
        match self {
            MachineInstr::Add { rd, .. }
            | MachineInstr::Addi { rd, .. }
            | MachineInstr::Mul { rd, .. }
            | MachineInstr::Sub { rd, .. }
            | MachineInstr::Div { rd, .. }
            | MachineInstr::Mv { rd, .. }
            | MachineInstr::Li { rd, .. } => {
                vec![*rd]
            }
            _ => Vec::new(),
        }
    }

    pub fn uses(&self) -> Vec<VReg> {
        match self {
            MachineInstr::Add { rs1, rs2, .. }
            | MachineInstr::Mul { rs1, rs2, .. }
            | MachineInstr::Sub { rs1, rs2, .. }
            | MachineInstr::Beq { rs1, rs2, .. }
            | MachineInstr::Div { rs1, rs2, .. } => {
                vec![*rs1, *rs2]
            }

            MachineInstr::Addi { rs1, .. } | MachineInstr::Mv { rs1, .. } => {
                vec![*rs1]
            }

            _ => Vec::new(),
        }
    }
}
