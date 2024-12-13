mod utils;

mod test_snippets_simple_call_switch {
    // use pretty_assertions::assert_eq;
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/simple_call_switch";

    #[test]
    fn test_call_switch_directly() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_switch_directly.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test
        assert!(output.contains("0 -> 2")); // main -> test2

        Ok(())
    }

    #[test]
    fn test_call_switch_alias() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_switch_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test
        assert!(output.contains("0 -> 2")); // main -> test2

        Ok(())
    }

    #[test]
    fn test_call_switch_ref_alias() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_switch_ref_alias.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> test
        assert!(output.contains("0 -> 2")); // main -> test2

        Ok(())
    }

    #[test]
    fn test_call_switch_directly_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_switch_directly_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test
        assert!(output.contains("1 -> 3")); // closure -> test2

        Ok(())
    }

    #[test]
    fn test_call_switch_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_switch_alias_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test
        assert!(output.contains("1 -> 3")); // closure -> test2

        Ok(())
    }

    #[test]
    fn test_call_switch_ref_alias_from_closure() -> Result<(), String> {
        let snippet =
            &std::fs::read_to_string(format!("{FOLDER}/call_switch_ref_alias_from_closure.rs"))
                .unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> closure
        assert!(output.contains("1 -> 2")); // closure -> test
        assert!(output.contains("1 -> 3")); // closure -> test2

        Ok(())
    }
}
