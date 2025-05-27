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
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValueDef {
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Instruction {
    // == Arithmetic Operator ==
    Add {
        dest: String,
        args: [String; 2],
        // We don't need to specify (type) because the Arithmetic
        // Opcode all deal with Integers in Bril case...however I
        // added to make a 1:1 copy of the json output
        #[serde(rename = "type")]
        typ: String,
    },

    Mul {
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

    Div {
        dest: String,
        args: [String; 2],
        #[serde(rename = "type")]
        typ: String,
    },

    // == Comparsion Operator ==
    Eq {
        dest: String,
        args: [String; 2],
    },

    Lt {
        dest: String,
        args: [String; 2],
    },

    Gt {
        dest: String,
        args: [String; 2],
    },

    Ge {
        dest: String,
        args: [String; 2],
    },

    Le {
        dest: String,
        args: [String; 2],
    },

    // == Logical Operator ==
    Not {
        dest: String,
        args: String,
    },

    Or {
        dest: String,
        args: [String; 2],
    },

    And {
        dest: String,
        args: [String; 2],
    },

    // == Literal constant & Id ==
    Const {
        dest: String,
        #[serde(rename = "type")]
        typ: String,
        value: i64,
    },

    Id {
        dest: String,
        args: String,
    },

    // == Control flow ==
    Br {
        args: String,
        labels: [String; 2],
    },

    Label {
        label: String,
    },

    Jmp {
        target: String,
    },

    Call {
        target: String,
    },

    Ret {
        #[serde(default)]
        args: Vec<String>,
    },

    // == Print ==
    Print {
        #[serde(default)]
        args: Vec<String>,
    },
}
