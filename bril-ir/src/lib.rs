pub mod cfg;
pub mod ssa;
pub use cfg::IrFunction;
pub use cfg::IrInstruction;
pub use cfg::IrModule;
pub use ssa::SSAFormation;

#[cfg(test)]
mod tests {
    use crate::cfg::IrBasicBlock;

    use super::*;

    /// Build the 5-block “diamond” CFG:
    ///
    ///      0
    ///      │
    ///      1
    ///     / \
    ///    2   3
    ///     \ /
    ///      4
    ///      │
    ///      5
    fn diamond_cfg() -> IrFunction {
        let block_labels = ["entry", "A", "B", "C", "D", "Exit"];

        let preds = vec![
            Vec::new(), // 0: entry
            vec![0],    // 1: A
            vec![1],    // 2: B
            vec![1],    // 3: C
            vec![2, 3], // 4: D (preds are 2 & 3)
            vec![4],    // 5: exit
        ];

        let mut blocks = Vec::new();
        for (i, &label) in block_labels.iter().enumerate() {
            blocks.push(IrBasicBlock {
                label: label.to_string(),
                instrs: Vec::new(),
                preds: preds[i].clone(),
                succs: Vec::new(),
            });
        }

        let mut label_to_idx = std::collections::HashMap::new();
        for (i, &label) in block_labels.iter().enumerate() {
            label_to_idx.insert(label.to_string(), i);
        }

        IrFunction {
            name: "diamond".to_string(),
            args: Vec::new(),
            blocks,
            label_to_idx,
        }
    }

    #[test]
    fn test_idom_df_and_domtree_on_diamond() {
        let func = diamond_cfg();

        let temp_funcs = vec![func];
        let mut ssa = SSAFormation::new(&temp_funcs).unwrap();

        // IDOM Compute
        ssa.compute_idom(&temp_funcs[0]).unwrap();
        dbg!("{:?}", &ssa.idom);
        assert_eq!(ssa.idom[&0], 0);
        assert_eq!(ssa.idom[&1], 0);
        assert_eq!(ssa.idom[&2], 1);
        assert_eq!(ssa.idom[&3], 1);
        assert_eq!(ssa.idom[&4], 1);
        assert_eq!(ssa.idom[&5], 4);
    }
}
