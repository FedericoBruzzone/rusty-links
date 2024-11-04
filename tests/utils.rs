use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Once;

const PLUGIN_NAME: &str = "rustc-ex";
const TEST_MODE_FEATURE: &str = "test-mode";
static INSTALL_PLUGIN: Once = Once::new();

/// Run the plugin with the `cargo` command
///
/// This function will install the plugin (cargo-rustc-ex binary) in a temporary directory and run it with the `cargo` command.
/// The plugin will be installed only once.
///
/// # Arguments
/// * `cargo_project_name` - The name of the cargo project in the `tests` directory. E.g. `workspaces/simple_feature_no_weights`
/// * `expected_outout_name` - The name of the file containing the expected output in the cargo project directory. E.g. `expected_output.stdout`
/// * `plugin_args` - The arguments to pass to the plugin
pub fn run_with_cargo_bin(
    cargo_project_name: &str,
    expected_outout_name: Option<&str>,
    plugin_args: &[&str],
) -> Result<(String, Option<String>), String> {
    // Install the plugin
    let root_dir = env::temp_dir().join("rustc-ex");
    let current_dir = Path::new(".").canonicalize().unwrap();
    INSTALL_PLUGIN.call_once(|| {
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.args(["install", "--path", ".", "--debug", "--locked", "--root"]);
        cargo_cmd.arg(&root_dir);
        cargo_cmd.current_dir(&current_dir);
        // See the `args` function on `impl RustcPlugin for RustcEx` for the explanation of why we need to pass the `--features test-mode` argument.
        cargo_cmd.args(["--features", TEST_MODE_FEATURE]);
        let status = cargo_cmd.status().unwrap();
        if !status.success() {
            panic!("Failed to install the plugin");
        }
    });

    // Prepare the cargo command
    let path = format!(
        "{}:{}",
        root_dir.join("bin").display(),
        env::var("PATH").unwrap_or_default()
    );
    let workspace_path = current_dir.join("tests").join(cargo_project_name);
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg(PLUGIN_NAME);
    for arg in plugin_args {
        cargo_cmd.arg(arg);
    }
    cargo_cmd.env("PATH", path);
    cargo_cmd.current_dir(&workspace_path);

    // Clean the target directory of the workspace
    let _ = fs::remove_dir_all(workspace_path.join("target"));

    // Run the plugin
    let output = cargo_cmd.output().unwrap();
    // assert!(output.status.success());  This cannot be true because the plugin is change all `#[cfg(` to `#[my_cfg(` in order to process all the features

    if let Some(expected_outout_name) = expected_outout_name {
        let expected_output_path = workspace_path.join(expected_outout_name);
        let expected_output = fs::read_to_string(expected_output_path).unwrap();
        Ok((
            String::from_utf8(output.stdout).unwrap(),
            Some(expected_output),
        ))
    } else {
        Ok((String::from_utf8(output.stdout).unwrap(), None))
    }
}

pub fn create_cargo_project_with_snippet(snippet: &str) -> Result<(), String> {
    let current_dir = Path::new(".").canonicalize().unwrap();
    let workspace_path = current_dir.join("tests").join("workspaces").join("temp");
    fs::create_dir_all(&workspace_path).unwrap();
    let lib_rs_path = workspace_path.join("src").join("lib.rs");
    fs::create_dir_all(lib_rs_path.parent().unwrap()).unwrap();
    fs::write(lib_rs_path, snippet).unwrap();
    let manifest_path = workspace_path.join("Cargo.toml");
    fs::write(
        manifest_path,
        r#"
[package]
name = "temp"
version = "0.1.0"
edition = "2018"

[dependencies]
"#,
    )
    .unwrap();
    Ok(())
}

pub fn remove_cargo_project_with_snippet() -> Result<(), String> {
    let current_dir = Path::new(".").canonicalize().unwrap();
    let workspace_path = current_dir.join("tests").join("workspaces").join("temp");
    fs::remove_dir_all(workspace_path).unwrap();
    Ok(())
}

#[allow(dead_code)] // FIXME: https://github.com/rust-lang/rust/issues/46379
pub fn run_with_cargo_bin_and_snippet(
    snippet: &str,
    plugin_args: &[&str],
) -> Result<(String, Option<String>), String> {
    create_cargo_project_with_snippet(snippet).unwrap();
    let result = run_with_cargo_bin("workspaces/temp", None, plugin_args);
    remove_cargo_project_with_snippet().unwrap();
    result
}
