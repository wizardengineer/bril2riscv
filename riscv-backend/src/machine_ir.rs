use bril_ir::BlockID;

#[derive(Default, Debug, Clone)]
pub struct MachineFunc {
    pub name: String,
    pub args: Vec<VReg>,
    pub blocks: Vec<MachineBlock>,
}

impl MachineFunc {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.to_string(),
            args: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MachineBlock {
    pub name: String,
    pub instrs: Vec<MachineInstr>,
    pub succs: Vec<BlockID>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
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

    // Control flow Instructions
    Jal { rd: VReg, offset: usize },

    // Unconditional jump
    Jmp { imm: String },

    Beqz { rs1: VReg, imm: String },

    Beq { rs1: VReg, rs2: VReg, imm: String },

    Ret { rd: VReg },

    Call { func: String },

    Print { args: Vec<VReg> },
    // TODO: Add more instructions
}
