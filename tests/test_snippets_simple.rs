mod utils;

mod test_snippets_simple {
    // use pretty_assertions::assert_eq;
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/simple";

    #[test]
    fn test_first() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/first.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-crate"])?;

        assert!(output.contains("Hello, World!"));

        Ok(())
    }
}
