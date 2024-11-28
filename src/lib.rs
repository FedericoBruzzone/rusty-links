#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod analysis;
pub mod instrument;

use analysis::{rl_analysis::RLAnalysis, Analyzer};
use clap::Parser;
use instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env};

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default, Clone)]
pub struct CliArgs {
    // Color lor
    #[clap(long)]
    color_log: bool,

    /// Use unoptimized MIR
    #[clap(long)]
    use_unoptimized_mir: bool,

    /// Print the AST of the crate
    #[clap(long)]
    print_crate: bool,

    // Print MIR
    #[clap(long)]
    print_mir: bool,

    // Print RustyLinks graph
    #[clap(long)]
    print_rl_graph: bool,

    // Print serialized RustyLinks graph
    #[clap(long)]
    print_serialized_rl_graph: bool,

    #[clap(last = true)]
    // mytool --allcaps -- some extra args here
    //                     ^^^^^^^^^^^^^^^^^^^^ these are cargo args
    cargo_args: Vec<String>,
}

// This struct is the plugin provided to the intrumentation module,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustyLinks;

impl RustyLinks {
    pub fn after_exec() {
        log::debug!("After exec");
        RLAnalysis::<rustworkx_core::petgraph::graph::DiGraph<_, _, _>>::merge_all_rl_graphs();
    }
}

impl RustcPlugin for RustyLinks {
    type Args = CliArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "rusty-links-driver".into()
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        cargo.args(&args.cargo_args);
    }

    // In the CLI, we ask Clap to parse arguments and also specify a CrateFilter.
    // If one of the CLI arguments was a specific file to analyze, then you
    // could provide a different filter.
    fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
        // We cannot use `#[cfg(test)]` here because the test suite installs the plugin.
        // In other words, in the test suite we need to compile (install) the plugin with
        // `--features test-mode` to skip the first argument that is the `cargo` command.
        //
        // # Explanation:
        //
        // ## Test
        //
        // In tests we run something like `cargo rusty-links --print-dot` because the plugin is installed as a binary in a temporary directory.
        // It is expanded to `/tmp/rusty-links/bin/cargo-rusty-links rusty-links --print-dot`, so we need to skip the first argument because it is the `cargo` command.
        //
        // ## Cli
        // In the CLI we run something like `cargo run --bin rusty-links -- --print-dot` or `./target/debug/cargo-rusty-links --print-dot`.
        // It is expanded to `.target/debug/cargo-rusty-links --print-dot`, so we don't need to skip the first argument.
        #[cfg(feature = "test-mode")]
        let args = CliArgs::parse_from(env::args().skip(1));

        #[cfg(not(feature = "test-mode"))]
        let args = CliArgs::parse_from(env::args());

        let filter = CrateFilter::AllCrates;
        RustcPluginArgs { args, filter }
    }

    // In the driver, we use the Rustc API to start a compiler session
    // for the arguments given to us by rustc_plugin.
    fn run(
        self,
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        log::debug!("Running plugin with compiler args: {:?}", compiler_args);
        log::debug!("Running plugin with args: {:?}", plugin_args);
        let mut callbacks = PluginCallbacks { args: plugin_args };
        let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
        compiler.run()
    }
}

struct PluginCallbacks {
    args: CliArgs,
}

impl rustc_driver::Callbacks for PluginCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, config: &mut rustc_interface::Config) {
        // Set the session creation callback to initialize the Fluent bundle.
        // It will make the compiler silent and use the fallback bundle.
        // Errors will not be printed in the `stderr`.
        // config.psess_created = Some(Box::new(|sess| {
        //     let fallback_bundle = rustc_errors::fallback_fluent_bundle(
        //         rustc_driver::DEFAULT_LOCALE_RESOURCES.to_vec(),
        //         false,
        //     );
        //
        //     sess.dcx().make_silent(fallback_bundle, None, false);
        // }));

        if self.args.use_unoptimized_mir {
            config.opts.unstable_opts.mir_opt_level = Some(0);
        }
    }

    /// Called after expansion. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        // Abort if errors occurred during expansion.
        compiler.sess.dcx().abort_if_errors();
        queries
            .global_ctxt()
            .expect("Error: global context not found")
            .enter(|tcx: rustc_middle::ty::TyCtxt| {
                Analyzer::<'tcx>::new(tcx, self.args.clone()).run()
            });
        compiler.sess.dcx().abort_if_errors();
        rustc_driver::Compilation::Continue
    }
}
