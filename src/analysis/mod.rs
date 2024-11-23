pub mod rl_analysis;
mod utils;

use crate::CliArgs;
use rl_analysis::rl_graph::{RLEdge, RLGraph, RLIndex, RLNode};
use rl_analysis::RLAnalysis;
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_middle::mir;
use rustc_middle::ty;
use serde::de::DeserializeOwned;
use serde::Serialize;
use utils::{TextMod, RL_SERDE_FOLDER};

pub struct Analyzer<'tcx> {
    tcx: ty::TyCtxt<'tcx>,
    cli_args: CliArgs,
}

impl<'tcx> Analyzer<'tcx> {
    pub fn new(tcx: ty::TyCtxt<'tcx>, cli_args: CliArgs) -> Self {
        Self { tcx, cli_args }
    }

    fn pre_process_cli_args(&self) {
        log::debug!("Pre-processing CLI arguments");
        if self.cli_args.print_crate {
            log::debug!("Printing the crate");
            let resolver_and_krate = self.tcx.resolver_for_lowering().borrow();
            let krate = &*resolver_and_krate.1;
            println!("{:#?}", krate);
        }

        if self.cli_args.print_mir {
            log::debug!("Printing the MIR");
            mir::write_mir_pretty(self.tcx, None, &mut std::io::stdout())
                .expect("write_mir_pretty failed");
        }
    }

    fn post_process_cli_args<G>(&self)
    where
        G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
            + Default
            + Clone
            + Serialize
            + DeserializeOwned,
    {
        log::debug!("Post-processing CLI arguments");
        let rl_graph: G =
            self.deserialize_rl_graph_from_file(&self.tcx.crate_name(LOCAL_CRATE).to_string());

        if self.cli_args.print_rl_graph {
            log::debug!("Printing the RustyLinks graph");
            rl_graph.print_dot();
        }

        if self.cli_args.print_serialized_rl_graph {
            log::debug!("Printing the serialized RustyLinks graph");
            let serialized = serde_json::to_string(&rl_graph).unwrap();
            println!("{}", serialized);
        }
    }

    fn modify_if_needed(&self, msg: &str, text_mod: TextMod) -> String {
        if self.cli_args.color_log {
            text_mod.apply(msg)
        } else {
            msg.to_string()
        }
    }

    fn deserialize_rl_graph_from_file<G>(&self, krate_name: &str) -> G
    where
        G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
            + Default
            + Clone
            + Serialize
            + DeserializeOwned,
    {
        let file_name = format!("{}/{}.rlg", RL_SERDE_FOLDER, krate_name);
        let file = std::fs::File::open(file_name).expect("Failed to open file");
        let rl_graph: G = serde_json::from_reader(file).expect("Failed to deserialize RLGraph");
        rl_graph
    }

    fn run_analysis(&mut self, name: &str, f: impl FnOnce(&Self)) {
        log::debug!("Running analysis: {}", name);
        f(self);
        log::debug!("Finished analysis: {}", name);
    }

    pub fn run(&mut self) {
        self.pre_process_cli_args();
        self.run_analysis("FirstAnalysis", |analyzer| {
            RLAnalysis::<rustworkx_core::petgraph::graph::DiGraph<_, _, _>>::new(analyzer).run();
        });
        self.post_process_cli_args::<rustworkx_core::petgraph::graph::DiGraph<_, _, _>>();
    }
}
