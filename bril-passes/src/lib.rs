pub mod pass_manager;
pub use pass_manager::FunctionPass;
pub use pass_manager::PassManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
