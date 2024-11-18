mod rl_analysis;
mod utils;

use crate::CliArgs;
use rl_analysis::rl_graph::{RLEdge, RLGraph, RLIndex, RLNode};
use rl_analysis::RLAnalysis;
use serde::Serialize;
use utils::TextMod;

use rustc_middle::mir;
use rustc_middle::ty;
use std::cell::Cell;

pub struct Analyzer<'tcx, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    tcx: ty::TyCtxt<'tcx>,
    cli_args: CliArgs,
    rl_graph: Cell<G>,
}

impl<'tcx, G> Analyzer<'tcx, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex> + Default + Clone + Serialize,
{
    pub fn new(tcx: ty::TyCtxt<'tcx>, cli_args: CliArgs) -> Self {
        Self {
            tcx,
            cli_args,
            rl_graph: Cell::new(G::default()),
        }
    }

    fn pre_process_cli_args(&self) {
        log::debug!("Pre-processing CLI arguments");
        if self.cli_args.print_crate {
            log::debug!("Printing the crate");
            let resolver_and_krate = self.tcx.resolver_for_lowering().borrow();
            let krate = &*resolver_and_krate.1;
            println!("{:#?}", krate);
        }

        // In case of "optimized" MIR, in the `config` callback we do not set the `mir_opt_level` to 0.
        if self.cli_args.print_mir || self.cli_args.print_unoptimized_mir {
            log::debug!("Printing the MIR");
            mir::write_mir_pretty(self.tcx, None, &mut std::io::stdout())
                .expect("write_mir_pretty failed");
        }
    }

    fn post_process_cli_args(&self) {
        log::debug!("Post-processing CLI arguments");
        let rl_graph = self.rl_graph.take();

        if self.cli_args.print_rl_graph {
            log::debug!("Printing the RustyLinks graph");
            rl_graph.print_dot();
        }

        if self.cli_args.print_serialized_rl_graph {
            log::debug!("Printing the serialized RustyLinks graph");
            let serialized = serde_json::to_string(&rl_graph).unwrap();
            println!("{}", serialized);
            // let deserialized: G = serde_json::from_str(&serialized).unwrap();
        }
    }

    fn modify_if_needed(&self, msg: &str, text_mod: TextMod) -> String {
        if self.cli_args.color_log {
            text_mod.apply(msg)
        } else {
            msg.to_string()
        }
    }

    fn run_analysis(&mut self, name: &str, f: impl FnOnce(&Self)) {
        log::debug!("Running analysis: {}", name);
        f(self);
        log::debug!("Finished analysis: {}", name);
    }

    pub fn run(&mut self) {
        self.pre_process_cli_args();
        self.run_analysis("FirstAnalysis", |analyzer| {
            RLAnalysis::new(analyzer).run();
        });
        self.post_process_cli_args();
    }
}
