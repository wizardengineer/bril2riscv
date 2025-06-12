use crate::cfg::IrFunction;
use crate::cfg::IrModule;
use anyhow::{Error, Result};
use std::collections::{BTreeMap, HashMap};

/// Help with having more readable code
type BlockID = usize;

/// Set up the Dominator Trees and Dominance Frontier
/// Using the Cytron algo for creating a SSA
///
/// Cytron et al.’s SSA-construction recipe bundles all of this into a single,
/// reasonably efficient flow:
///
///1.Compute dominators (often via a faster “Lengauer–Tarjan” or equivalent algorithm,
///   not the naive Braun iteration).
///2.Build the immediate-dominator (idom) tree.
///3.Compute each node’s DF (in a single pass over the CFG + dom-tree).
///4.Place ϕ-nodes for each variable at all blocks in the union of DF(definition blocks).
#[derive(Debug)]
pub struct SSAFormation {
    idom: HashMap<BlockID, usize>,
    dom_tree: HashMap<BlockID, usize>,
    dom_frontier: BTreeMap<BlockID, Vec<usize>>,
}

/// Convert our IrModule into a true SSA form
impl TryFrom<&IrModule> for SSAFormation {
    type Error = anyhow::Error;

    fn try_from(ir_module: &IrModule) -> Result<SSAFormation> {
        let out = SSAFormation::new(&ir_module.functions)?;
        Ok(out)
    }
}

impl SSAFormation {
    pub fn new(funcs: &[IrFunction]) -> Result<Self> {
        let mut out = SSAFormation {
            idom: HashMap::new(),
            dom_tree: HashMap::new(),
            dom_frontier: BTreeMap::new(),
        };

        for func in funcs {
            out.compute_idom(func)?;
            out.compute_df(func)?;
            //out.build_dom_tree(func)?;
        }

        Ok(out)
    }

    // TODO: Later in the future implement lengauer_tarjan_idom
    pub fn compute_idom(&mut self, funcs: &IrFunction) -> Result<()> {
        let n = funcs.blocks.len();
        // usize::MAX means the idom is an unknown for now
        let mut idom_vec = vec![usize::MAX; n];

        // entry point to entry
        idom_vec[0] = 0;

        // find the fix-point of the loop
        loop {
            // b_idx = block index
            // starting from block 1 because idom[0] is 0
            for b_idx in 1..n {
                let preds = &funcs.blocks[b_idx].preds;

                // Skip for if preds empty, we care for the preds because of the idom
                if preds.is_empty() {
                    continue;
                }

                let mut new_idom = None;
                for &p in preds {
                    if idom_vec[p] != usize::MAX {
                        new_idom = Some(p);
                        break;
                    }
                }

                let mut _new_idom = match new_idom {
                    Some(x) => x,
                    None => continue,
                };
            }
        }

        Ok(())
    }

    pub fn compute_df(&mut self, funcs: &IrFunction) -> Result<()> {
        for block in &funcs.blocks {
            let current_idx = funcs.block_index(&block.label);

            if !block.preds.len() >= 2 {
                continue;
            }

            for pred in block.preds {
                let mut runner = pred;
                while runner != self.idom.get(current_idx).copied() {
                    self.dom_frontier
                        .entry(runner)
                        .or_insert_with(Vec::new())
                        .push(current_idx);
                    runner = self.idom.get(pred).copied();
                }
            }
        }
        Ok(())
    }
}
