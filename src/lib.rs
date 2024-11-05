#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

pub mod instrument;

use clap::Parser;
use instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env};

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default)]
pub struct PluginArgs {
    /// Print the AST of the crate
    #[clap(long)]
    print_crate: bool,

    #[clap(last = true)]
    // mytool --allcaps -- some extra args here
    //                     ^^^^^^^^^^^^^^^^^^^^ these are cargo args
    cargo_args: Vec<String>,
}

// This struct is the plugin provided to the intrumentation module,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustyLinks;

impl RustcPlugin for RustyLinks {
    type Args = PluginArgs;

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
        let args = PluginArgs::parse_from(env::args().skip(1));

        #[cfg(not(feature = "test-mode"))]
        let args = PluginArgs::parse_from(env::args());

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
        let mut callbacks = PluginCallbacks { args: plugin_args };
        let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
        compiler.run()
    }
}

struct PluginCallbacks {
    args: PluginArgs,
}

impl PluginCallbacks {
    fn pre_process_cli_args(&self, tcx: &rustc_middle::ty::TyCtxt) {
        if self.args.print_crate {
            let resolver_and_krate = tcx.resolver_for_lowering().borrow();
            let krate = &*resolver_and_krate.1;
            println!("{:#?}", krate);
        }
    }

    fn post_process_cli_args(&self, _visitor: &Visitor) {}
}

impl rustc_driver::Callbacks for PluginCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, _config: &mut rustc_interface::Config) {
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
    }

    /// Called after expansion. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        queries
            .global_ctxt()
            .expect("Error: global context not found")
            .enter(|tcx: rustc_middle::ty::TyCtxt| {
                self.pre_process_cli_args(&tcx);

                // visit AST
                let visitor = &mut Visitor {};

                self.post_process_cli_args(&visitor);
            });

        rustc_driver::Compilation::Stop
    }
}

/// AST visitor to collect data to build the graphs
struct Visitor;
