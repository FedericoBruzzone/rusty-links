mod utils;

mod test_snippets_simple_call_mut_static {
    // use pretty_assertions::assert_eq;
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/simple_call_mut_static";

    #[test]
    fn test_simple_call_with_file_call_mut_static_directly() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_mut_static_directly.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> TEST

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_static_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_static_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> TEST 

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_static_ref_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_static_ref_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> TEST 

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_mut_static_directly_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_mut_static_direcly_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> TEST

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_static_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_static_alias_from_closure.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> TEST

        Ok(())
    }

    #[test]
    fn test_simple_call_with_file_call_static_ref_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_static_ref_alias_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> TEST

        Ok(())
    }
}
