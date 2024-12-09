#![feature(rustc_private)]

mod utils;

mod test_workspaces_project_maybe_conflict_closure {
    use petgraph::graph::DiGraph;
    use rusty_links::analysis::rl_analysis::{
        rl_graph::{RLEdge, RLGraph, RLIndex, RLNode},
        RLAnalysis,
    };
    use rusty_links::analysis::utils::{MERGED_FILE_NAME, RL_SERDE_FOLDER};

    use crate::utils::run_with_cargo_bin;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_workspaces_project_maybe_conflict_closure_dot_file() -> Result<(), String> {
        const FOLDER: &str = "tests/workspaces/project_maybe_conflict_closure";
        let _ = run_with_cargo_bin(FOLDER, None, &[])?;
        let folder_path = format!("{}/{}/{}.rlg", FOLDER, RL_SERDE_FOLDER, MERGED_FILE_NAME);
        let output =
            RLAnalysis::<DiGraph<RLNode, RLEdge, RLIndex>>::deserialized_rl_graph_from_file(
                folder_path.as_str(),
            )
            .as_dot_str();

        assert!(output.contains("2 -> 3")); // crate_a::add -> TEST
        assert!(output.contains("6 -> 7")); // crate_b::add -> TEST
        assert!(output.contains("8 -> 2")); // main -> crate_a::add
        assert!(output.contains("8 -> 6")); // main -> crate_b::add

        Ok(())
    }
}
