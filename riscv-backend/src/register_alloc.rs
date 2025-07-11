use crate::machine_ir::{MachineFunc, VReg};
use std::{cmp, collections::HashMap};

/// So far we're going to use Linear Scan for doing register allocation.
/// TODO: Implementing Graph coloring...somewhere in the near future

#[derive(Debug, Default, Clone)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct LiveIntervals {
    vreg: VReg,
    interval: Interval,
    mark_spill: Option<VReg>,
}

const ALL_REGS: &[VReg] = &[
    // Temp registers
    VReg::T0,
    VReg::T1,
    VReg::T2,
    VReg::T3,
    VReg::T4,
    VReg::T5,
    VReg::T6,
    // Function arguments
    VReg::A0, // function argument 0 / return value 0
    VReg::A1, // function argument 1 / return value 1
    VReg::A2,
    VReg::A3,
    VReg::A4,
    VReg::A5,
    VReg::A6,
    VReg::A7,
    // Saved registers
    //VReg::S0, // frame pointer
    VReg::S1,
    VReg::S2,
    VReg::S3,
    VReg::S4,
    VReg::S5,
    VReg::S6,
    VReg::S7,
    VReg::S8,
    VReg::S9,
    VReg::S10,
    VReg::S11,
    // Return address, Stack pointer & Frame pointer
    VReg::RA,
    VReg::SP,
    VReg::FP,
    // Global Register
    VReg::GP,
];

#[derive(Debug, Default)]
pub struct LinearScan {}

impl LinearScan {
    pub fn new(funcs: &[MachineFunc]) -> Self {
        let mut ra = Self {};
        for func in funcs.iter() {
            let interval = ra.build_intervals(func);
        }

        ra
    }

    pub fn build_intervals(&mut self, mf: &MachineFunc) -> HashMap<VReg, Interval> {
        let mut intervals: HashMap<VReg, Interval> = HashMap::new();

        let mut instrs_global_pos = HashMap::new();
        let mut instr_pos = 0;
        for (b_idx, block) in mf.blocks.iter().enumerate() {
            for i in 0..block.instrs.len() {
                instrs_global_pos.insert((b_idx, i), instr_pos);
                instr_pos += 1;
            }
        }

        for (b_idx, block) in mf.blocks.iter().enumerate() {
            for (i, instr) in block.instrs.iter().enumerate() {
                let pos = instrs_global_pos.get(&(b_idx, i)).unwrap();

                for def in instr.defs() {
                    let interval = intervals.entry(def).or_insert(Interval {
                        start: *pos,
                        end: *pos,
                    });

                    interval.start = cmp::min(interval.start, *pos);
                }

                for u in instr.uses() {
                    let interval = intervals.entry(u).or_insert(Interval {
                        start: *pos,
                        end: *pos,
                    });

                    interval.end = cmp::max(interval.end, *pos);
                }
            }
        }
        intervals
    }

    pub fn linear_scan(&mut self, intervals: &mut HashMap<VReg, Interval>) {
        // Store our intervals in our Live Intervals sort intervals
        let mut live_intervals: Vec<LiveIntervals> = intervals
            .iter()
            .map(|(vreg, interval)| LiveIntervals {
                vreg: *vreg,
                interval: interval.clone(),
                mark_spill: None,
            })
            .collect();

        live_intervals.sort_by_key(|ivl| ivl.interval.start);

        let mut active_alloc_intervals: Vec<LiveIntervals> = Vec::new();
        let mut free_regs = ALL_REGS.to_vec();

        for iv in live_intervals.iter_mut() {
            active_alloc_intervals.retain(|old_iv| {
                if old_iv.interval.end < iv.interval.start {
                    false
                } else {
                    true
                }
            });

            // incompleted
        }
    }
}
