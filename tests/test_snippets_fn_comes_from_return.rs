mod utils;

mod test_snippets_fn_comes_from_return {
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/fn_comes_from_return";

    #[test]
    fn test_return_fn_as_impl_fn_t() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/return_fn_as_impl_fn_t.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("3 -> 2")); // main -> return_test
        assert!(output.contains("3 -> 1")); // main -> test

        Ok(())
    }

    #[test]
    fn test_return_fn_as_fn_t() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/return_fn_as_fn_t.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("3 -> 2")); // main -> return_test
        assert!(output.contains("3 -> 4")); // main -> STATICALLY_UNKOWN

        Ok(())
    }
}
