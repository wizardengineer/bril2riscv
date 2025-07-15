use anyhow::Result;
use bril_frontend::Program;
use bril_ir::{IrModule, SSAFormation};
use bril_passes::{ConstantFoldPass, ConstantPropagationPass, DeadCodeRemovalPass, PassManager};

use riscv_backend::*;
//use std::collections::HashMap;

fn main() -> Result<()> {
    let json_text = include_str!("../../tests/palindrome.json");
    let bril_prog: Program = serde_json::from_str(&json_text)?;
    let mut ir_mod: IrModule = IrModule::try_from(&bril_prog)?;
    let _ = SSAFormation::try_from(&mut ir_mod)?;
    let mut pm = PassManager::new();
    pm.add_pass(ConstantPropagationPass {});
    pm.add_pass(ConstantFoldPass {});
    pm.add_pass(DeadCodeRemovalPass {});
    pm.run(&mut ir_mod);

    let mut machine_module = Vec::new();
    for func in ir_mod.functions.iter() {
        let mf = select_instructions(func);
        machine_module.push(mf);
    }

    println!("\n###### SSA IR ######");
    println!("{:#?}\n", ir_mod);

    println!("\n###### MachineIR ######");
    println!("{:#?}\n", machine_module);

    println!("\n###### Assembly ######");
    emit_riscv(&machine_module);

    Ok(())
}
