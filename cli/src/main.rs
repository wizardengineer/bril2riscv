use anyhow::Result;
use bril_frontend::Program;
use bril_ir::{IrModule, SSAFormation};
//use bril_passes::constant_propagate::ConstantPropagationPass;
use bril_passes::{
    compute_liveness, ConstantFoldPass, ConstantPropagationPass, DeadCodeRemovalPass, PassManager,
};

fn main() -> Result<()> {
    let json_text = include_str!("../../tests/add.json");
    let bril_prog: Program = serde_json::from_str(&json_text)?;
    let mut ir_mod: IrModule = IrModule::try_from(&bril_prog)?;
    let _ = SSAFormation::try_from(&mut ir_mod)?;
    let mut pm = PassManager::new();
    pm.add_pass(ConstantPropagationPass {});
    pm.add_pass(ConstantFoldPass {});
    pm.add_pass(DeadCodeRemovalPass {});
    pm.run(&mut ir_mod);

    let (live_out, live_in) = compute_liveness(&ir_mod.functions[0]);
    println!("{:#?}", live_out);
    println!("{:#?}", live_in);

    //println!("{:#?}", ssa);
    println!("{:#?}", ir_mod);
    Ok(())
}
