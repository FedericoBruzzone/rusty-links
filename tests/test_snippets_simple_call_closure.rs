mod utils;

mod test_snippets {
    // use pretty_assertions::assert_eq;
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/simple_call_closure";

    #[test]
    fn test_simple_call_with_file_call_closure_directly() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_closure_directly.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0} [test_own]

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_closure_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_closure_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0} [test_own]

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_closure_ref_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_closure_ref_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0} [test_own]

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_closure_directly_from_clusure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_closure_directly_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0} 
        assert!(output.contains("1 -> 2")); // {closure#0} -> {closure#1} [test_own]

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_closure_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_closure_alias_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0}
        assert!(output.contains("1 -> 2")); // {closure#0} -> {closure#1} [test_own]

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_closure_ref_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_closure_ref_alias_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> {closure#0}
        assert!(output.contains("1 -> 2")); // {closure#0} -> {closure#1} [test_own]

        Ok(())
    }
}
