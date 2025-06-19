use crate::cfg::IrFunction;
use crate::cfg::IrModule;
use anyhow::Result;
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
#[derive(Debug, Default)]
pub struct SSAFormation {
    pub idom: HashMap<BlockID, BlockID>,
    pub dom_tree: HashMap<BlockID, Vec<BlockID>>,
    pub dom_frontier: BTreeMap<BlockID, Vec<BlockID>>,
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
            out.build_dom_tree()?;
        }

        Ok(out)
    }

    // TODO: Later in the future implement lengauer_tarjan_idom
    pub fn compute_idom(&mut self, func: &IrFunction) -> Result<()> {
        let n = func.blocks.len();
        // usize::MAX means the idom is an unknown for now
        let mut idom_vec = vec![usize::MAX; n];

        // entry point to entry
        idom_vec[0] = 0;

        // find the fix-point of the loop
        loop {
            let mut changed = false;
            // b_idx = block index
            // starting from block 1 because idom[0] is 0
            for b in 1..n {
                let preds = &func.blocks[b].preds;

                // Skip for if preds empty, we care for the preds because of the idom
                if preds.is_empty() {
                    continue;
                }

                let mut new_idom = match preds.iter().find(|&&p| idom_vec[p] != usize::MAX) {
                    Some(&p) => p,
                    None => continue,
                };

                // collect into a Vec<usize>
                let others: Vec<usize> = preds
                    .iter()
                    .copied()
                    .filter(|&p| p != new_idom && idom_vec[p] != usize::MAX)
                    .collect();

                // climb the preds in order to see if the dominance chains match
                for p in others {
                    let mut finger1 = p;
                    let mut finger2 = new_idom;
                    while finger1 != finger2 {
                        while finger1 > finger2 {
                            finger1 = idom_vec[finger1];
                        }
                        while finger2 > finger1 {
                            finger2 = idom_vec[finger2];
                        }
                    }
                    new_idom = finger1;
                }

                if idom_vec[b] != new_idom {
                    idom_vec[b] = new_idom;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        self.idom.clear();
        for (block, &dom) in idom_vec.iter().enumerate() {
            if dom == usize::MAX {
                panic!("could not compute idom for Block {}", block);
            }
            self.idom.insert(block, dom);
        }

        Ok(())
    }

    // TODO: Finish this and dom tree too. Then test it out
    pub fn compute_df(&mut self, func: &IrFunction) -> Result<()> {
        self.dom_frontier.clear();

        for block in &func.blocks {
            let b = func.block_index(&block.label).unwrap();

            // making sure it's a joint point
            if block.preds.len() < 2 {
                continue;
            }

            let idom_b = *self.idom.get(&b).expect("idom wasn't computed");

            for &p in &block.preds {
                let mut runner = p;

                while runner != idom_b {
                    let entry = self.dom_frontier.entry(runner).or_default();
                    if !entry.contains(&b) {
                        entry.push(b);
                    }

                    // climbing up the pred, the one runner is equal to
                    runner = *self.idom.get(&runner).unwrap();
                }
            }
        }

        Ok(())
    }

    pub fn build_dom_tree(&mut self) -> Result<()> {
        self.dom_tree.clear();

        for (&b, &p) in &self.idom {
            if b != p {
                self.dom_tree.entry(p).or_default().push(b);
            }
        }
        Ok(())
    }
}
