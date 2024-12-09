#![feature(rustc_private)]

mod utils;

mod test_workspaces_simple_project_with_workspace {
    use petgraph::graph::DiGraph;
    use rusty_links::analysis::rl_analysis::{
        rl_graph::{RLEdge, RLGraph, RLIndex, RLNode},
        RLAnalysis,
    };
    use rusty_links::analysis::utils::{MERGED_FILE_NAME, RL_SERDE_FOLDER};

    use crate::utils::run_with_cargo_bin;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_workspaces_simple_project_with_workspace_dot_file() -> Result<(), String> {
        const FOLDER: &str = "tests/workspaces/simple_project_with_workspace";
        let _ = run_with_cargo_bin(FOLDER, None, &[])?;
        let folder_path = format!("{}/{}/{}", FOLDER, RL_SERDE_FOLDER, MERGED_FILE_NAME);
        let output = format!(
            "{:?}",
            RLAnalysis::<DiGraph<RLNode, RLEdge, RLIndex>>::deserialized_rl_graph_from_file(
                folder_path.as_str(),
            )
            .as_dot_str()
        );

        assert!(output.contains("1 -> 0"));
        assert!(output.contains("2 -> 1"));
        assert!(output.contains("2 -> 3"));
        assert!(output.contains("2 -> 4"));
        assert!(output.contains("2 -> 5"));

        Ok(())
    }
}
