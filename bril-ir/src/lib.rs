pub mod cfg;
pub mod ssa;
pub use cfg::IrFunction;
pub use cfg::IrInstruction;
pub use cfg::IrModule;
pub use ssa::SSAFormation;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
