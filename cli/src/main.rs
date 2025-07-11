use anyhow::Result;
use bril_frontend::Program;
use bril_ir::{IrModule, SSAFormation};
//use bril_passes::constant_propagate::ConstantPropagationPass;
use bril_passes::{
    compute_liveness, ConstantFoldPass, ConstantPropagationPass, DeadCodeRemovalPass, PassManager,
};

use riscv_backend::register_alloc;
use riscv_backend::select_instructions;
use riscv_backend::Interval;
use riscv_backend::LinearScan;
use riscv_backend::MachineFunc;
use riscv_backend::VReg;
use std::collections::HashMap;

fn main() -> Result<()> {
    let json_text = include_str!("../../tests/factorial.json");
    let bril_prog: Program = serde_json::from_str(&json_text)?;
    let mut ir_mod: IrModule = IrModule::try_from(&bril_prog)?;
    let _ = SSAFormation::try_from(&mut ir_mod)?;
    let mut pm = PassManager::new();
    pm.add_pass(ConstantPropagationPass {});
    pm.add_pass(ConstantFoldPass {});
    pm.add_pass(DeadCodeRemovalPass {});
    pm.run(&mut ir_mod);

    let mut machine_module = Vec::new();
    let mut live_intervals = Vec::new();
    for func in ir_mod.functions.iter() {
        machine_module.push(select_instructions(func));
        let (live_out, live_in) = compute_liveness(func);
        let mut register_alloc = LinearScan::default();

        live_intervals.push(register_alloc.build_intervals(&machine_module.iter().last().unwrap()));
    }

    //println!("{:#?}", live_out);
    //println!("{:#?}", live_in);
    println!("{:#?}", live_intervals);

    //println!("{:#?}", ssa);
    println!("{:#?}\n", ir_mod);
    println!("{:#?}\n", machine_module);
    Ok(())
}
