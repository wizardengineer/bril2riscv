use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub args: Vec<ValueDef>,
    pub instrs: Vec<Instruction>,
    #[serde(rename = "type", default)]
    pub ret_typ: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValueDef {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Instruction {
    /// A bare label instruction without an "op" field
    Label { label: String },

    /// All other instructions, identified by the "op" tag
    //#[serde(tag = "op", rename_all = "snake_case")]
    Op(Op),
}

/// Specicially made for const opcode
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Literal {
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    // Arithmetic operations
    Add {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Sub {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Mul {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Div {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },

    // Comparison operations
    Eq {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Lt {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Gt {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Le {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },
    Ge {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },

    // Logical operations
    Not {
        dest: String,
        args: Vec<String>,
    },
    And {
        dest: String,
        args: [String; 2],
    },
    Or {
        dest: String,
        args: [String; 2],
    },

    // Constants and identity
    Const {
        dest: String,
        #[serde(rename = "type")]
        typ: String,
        value: Literal,
    },
    Id {
        dest: String,
        args: Vec<String>,
        #[serde(rename = "type")]
        typ: String,
    },

    // Control flow
    Br {
        args: Vec<String>,
        labels: [String; 2],
    },
    Jmp {
        labels: Vec<String>,
    },

    // Function call and return
    Call {
        #[serde(default)]
        dest: Option<String>,
        funcs: Vec<String>,
        #[serde(default)]
        args: Vec<String>,
        #[serde(rename = "type")]
        typ: String,
    },
    Ret {
        #[serde(default)]
        args: Vec<String>,
    },

    // Miscellaneous
    Print {
        #[serde(default)]
        args: Vec<String>,
    },
    Nop,
}
