use anyhow::Result;
use bril_frontend::Program;
use bril_ir::IrModule;
use bril_ir::SSAFormation;
//use bril_passes::constant_propagate::ConstantPropagationPass;
use bril_passes::ConstantFoldPass;
use bril_passes::ConstantPropagationPass;
use bril_passes::PassManager;

fn main() -> Result<()> {
    let json_text = include_str!("../../tests/add.json");
    let bril_prog: Program = serde_json::from_str(&json_text)?;
    let mut ir_mod: IrModule = IrModule::try_from(&bril_prog)?;
    let ssa: SSAFormation = SSAFormation::try_from(&mut ir_mod)?;
    let mut pm = PassManager::new();
    pm.add_pass(ConstantPropagationPass {});
    pm.add_pass(ConstantFoldPass {});
    pm.run(&mut ir_mod);

    //println!("{:#?}", ssa);
    println!("{:#?}", ir_mod);
    Ok(())
}
