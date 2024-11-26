mod utils;

// mod test_snippets_simple_call_method_ref_mut_self {
//     // use pretty_assertions::assert_eq;
//     use crate::utils::run_with_cargo_bin_and_snippet;

//     const FOLDER: &str = "tests/snippets/simple_call_method_ref_mut_self";

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_directly() -> Result<(), String> {
//         let snippet =
//             &std::fs::read_to_string(format!("{FOLDER}/call_method_ref_mut_self_directly.rs")).unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> T::test

//         Ok(())
//     }

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_alias() -> Result<(), String> {
//         let snippet =
//             &std::fs::read_to_string(format!("{FOLDER}/call_method_ref_mut_self_alias.rs")).unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> T::test

//         Ok(())
//     }

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_ref_alias() -> Result<(), String> {
//         let snippet =
//             &std::fs::read_to_string(format!("{FOLDER}/call_method_ref_mut_self_ref_alias.rs")).unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> T::test

//         Ok(())
//     }

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_directly_from_closure() -> Result<(), String> {
//         let snippet = &std::fs::read_to_string(format!(
//             "{FOLDER}/call_method_ref_mut_self_directly_from_closure.rs"
//         ))
//         .unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> closure
//         assert!(output.contains("2 -> 3")); // closure -> T::test

//         Ok(())
//     }

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_alias_from_closure() -> Result<(), String> {
//         let snippet =
//             &std::fs::read_to_string(format!("{FOLDER}/call_method_ref_mut_self_alias_from_closure.rs"))
//                 .unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> closure
//         assert!(output.contains("2 -> 3")); // closure -> T::test

//         Ok(())
//     }

//     #[test]
//     fn test_simple_call_with_file_call_method_ref_mut_self_ref_alias_from_closure() -> Result<(), String> {
//         let snippet = &std::fs::read_to_string(format!(
//             "{FOLDER}/call_method_ref_mut_self_ref_alias_from_closure.rs"
//         ))
//         .unwrap();
//         let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-rl-graph"])?;

//         assert!(output.contains("1 -> 2")); // main -> closure
//         assert!(output.contains("2 -> 3")); // closure -> T::test

//         Ok(())
//     }
// }
