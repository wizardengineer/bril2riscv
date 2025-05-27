use anyhow::Result;
use bril_frontend::Program;
use bril_ir::IrModule;
use serde_json;

fn main() -> Result<()> {
    let json_text = std::fs::read_to_string("../tests/add.json")?;
    let bril_prog: Program = serde_json::from_str(&json_text)?;
    let ir_mod = IrModule::try_from(&bril_prog)?;
    println!("{:#?}", ir_mod);
    Ok(())
}
