pub mod json;
pub use json::Function;
pub use json::Instruction;
pub use json::Program;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn print_add_json() {
        // Include the JSON file at compile time. Path is relative to the crate root.
        let json = include_str!("../../tests/add.json");

        // Deserialize
        let program: Program =
            serde_json::from_str(json).expect("Failed to parse add.json into Program");

        // Print when needed `cargo test -- --nocapture`
        println!("{:#?}", program);

        // Basic sanity checks
        assert_eq!(program.functions.len(), 1);
    }
}
