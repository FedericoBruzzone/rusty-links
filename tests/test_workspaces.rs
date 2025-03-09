mod utils;

mod test_workspaces {
    use crate::utils::run_with_cargo_bin;
    use pretty_assertions::assert_eq;

    const FOLDER: &str = "tests/workspaces/first";

    #[test]
    fn test_version_output() -> Result<(), String> {
        let (output, _) = run_with_cargo_bin(FOLDER, None, &["-V"])?;
        assert_eq!(output, "0.1.0-nightly-2025-02-20\n");
        Ok(())
    }

    #[test]
    fn test_help_output() -> Result<(), String> {
        let (output, _) = run_with_cargo_bin(FOLDER, None, &["--help"])?;
        for options in &["--print-crate"] {
            assert!(output.contains(options));
        }
        Ok(())
    }

    #[test]
    fn test_first_same_output_plug_version() -> Result<(), String> {
        let (output, expected_output) =
            run_with_cargo_bin(FOLDER, Some("expected_output.plug_version"), &["-V"])?;
        assert_eq!(output, expected_output.unwrap());
        Ok(())
    }

    #[test]
    fn test_first_contains_plug_version() -> Result<(), String> {
        let (output, _) = run_with_cargo_bin(FOLDER, None, &["-V"])?;
        assert!(output.contains("0.1.0"));
        Ok(())
    }
}
