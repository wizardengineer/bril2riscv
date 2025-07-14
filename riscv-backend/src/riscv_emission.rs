use crate::machine_ir::*;
use crate::register_alloc::LinearScan;
use crate::VReg;
use std::collections::HashMap;

pub fn emit_riscv(module: &[MachineFunc]) {
    let mut allocator = LinearScan::new();
    let func_by_intervals = allocator.run(module);

    println!(".section .text");
    println!(".p2align 2"); // align to 4-byte boundary

    for func in module.iter() {
        println!(".globl {}", func.name);
    }

    for func in module.iter() {
        let mut spill_slots = HashMap::<VReg, usize>::new();
        let mut stack_frame: usize = 0;
        let live_intervals = &func_by_intervals.get(&func.name).unwrap();
        for (&vreg, ivs) in live_intervals.iter() {
            if ivs.mark_spilled {
                spill_slots.insert(vreg, stack_frame);
                stack_frame += 8;
            }
        }

        println!("\n{}:", func.name); // function label
        if stack_frame > 0 {
            println!("  addi sp, sp, {}", stack_frame);
        }

        for block in func.blocks.iter() {
            println!("  .{}:", block.name);

            for instr in block.instrs.iter() {
                // TODO: Add more instructions
                match instr {
                    MachineInstr::Li { rd, imm } => {
                        let phy_reg = live_intervals[rd].phy_reg.unwrap();
                        println!("  li {}, {}", phy_reg.name(), imm);
                    }

                    MachineInstr::Add { rd, rs1, rs2 } => {
                        let phy_reg = live_intervals[rd].phy_reg.unwrap();
                        let prs1 = live_intervals[rs1].phy_reg.unwrap();
                        let prs2 = live_intervals[rs2].phy_reg.unwrap();

                        println!("  add {}, {}, {}", phy_reg.name(), prs1.name(), prs2.name());
                    }

                    MachineInstr::Ret { rd } => {
                        if let Some(r) = rd {
                            let phy_reg = live_intervals[r].phy_reg.unwrap();

                            println!("  ret {}", phy_reg.name());
                        }

                        println!("  ret");
                    }

                    _ => {}
                }
            }
        }
    }
}
