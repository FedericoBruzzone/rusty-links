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
        let file_path = format!("{}/{}/{}.rlg", FOLDER, RL_SERDE_FOLDER, MERGED_FILE_NAME);
        let output =
            RLAnalysis::<DiGraph<RLNode, RLEdge, RLIndex>>::deserialized_rl_graph_from_file(
                file_path.as_str(),
            )
            .as_dot_str();

        // Print folder content
        println!("Folder content:");
        let output2 = std::fs::read_dir(format!("{}/{}", FOLDER, RL_SERDE_FOLDER))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .map(|path| path.display().to_string())
            .collect::<Vec<String>>()
            .join("\n");

        println!("{}", output);
        println!("{}", output2);

        assert!(output.contains("1 -> 0"));
        assert!(output.contains("3 -> 1"));
        assert!(output.contains("3 -> 2"));
        assert!(output.contains("3 -> 4"));
        assert!(output.contains("3 -> 5"));

        Ok(())
    }
}