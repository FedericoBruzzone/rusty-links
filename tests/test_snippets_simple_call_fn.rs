mod utils;

mod test_snippets_simple_call_fn {
    // use pretty_assertions::assert_eq;
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/simple_call_fn";

    #[test]
    fn test_file_call_fn_directly() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_fn_directly.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test_own

        Ok(())
    }

    #[test]
    fn test_file_call_fn_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_fn_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test_own

        Ok(())
    }

    #[test]
    fn test_file_call_fn_ref_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_fn_ref_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test_own

        Ok(())
    }

    #[test]
    fn test_file_call_fn_directly_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_fn_directly_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test_own

        Ok(())
    }

    #[test]
    fn test_file_call_fn_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_fn_alias_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test_own

        Ok(())
    }

    #[test]
    fn test_file_call_fn_ref_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_fn_ref_alias_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test_own

        Ok(())
    }
}
