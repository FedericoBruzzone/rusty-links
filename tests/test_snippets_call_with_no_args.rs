mod utils;

mod test_snippets_call_with_no_args {
    use crate::utils::run_with_cargo_bin_and_snippet;

    const FOLDER: &str = "tests/snippets/call_with_no_args";

    #[test]
    fn test_monomorphized_args() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/monomorphized_args.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        assert!(output.contains("0 -> 1")); // main -> call_once
        assert!(output.contains("2 -> 3")); // main -> main::{{closure}}

        Ok(())
    }

    #[test]
    fn test_statically_unknown() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/statically_unknown.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        println!("{}", output);
        assert!(output.contains("0 -> 1")); // main -> STATICALLY_UNKNOWN
        assert!(output.contains("2 -> 0")); // main -> outline

        Ok(())
    }

    #[test]
    fn test_wrapper() -> Result<(), String> {
        let snippet = &std::fs::read_to_string(format!("{FOLDER}/wrapper.rs")).unwrap();
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

        println!("{}", output);
        assert!(output.contains("1 -> 0")); // main -> outline
        assert!(output.contains("1 -> 2")); // main -> main::{{closure}}

        Ok(())
    }
}
