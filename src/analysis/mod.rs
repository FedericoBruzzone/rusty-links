use crate::CliArgs;

use std::{cell::Cell, time::Duration};

use rustc_index::Idx;
use rustc_middle::mir;
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty;

pub struct Analyzer<'tcx> {
    tcx: ty::TyCtxt<'tcx>,
    cli_args: CliArgs,
}

impl<'tcx> Analyzer<'tcx> {
    pub fn new(tcx: ty::TyCtxt<'tcx>, cli_args: CliArgs) -> Self {
        Self { tcx, cli_args }
    }

    fn pre_process_cli_args(&self) {
        log::info!("Pre-processing CLI arguments");
        if self.cli_args.print_crate {
            log::info!("Printing the crate");
            let resolver_and_krate = self.tcx.resolver_for_lowering().borrow();
            let krate = &*resolver_and_krate.1;
            println!("{:#?}", krate);
        }

        // In case of "optimized" MIR, in the `config` callback we do not set the `mir_opt_level` to 0.
        if self.cli_args.print_mir || self.cli_args.print_unoptimized_mir {
            log::info!("Printing the MIR");
            mir::write_mir_pretty(self.tcx, None, &mut std::io::stdout())
                .expect("write_mir_pretty failed");
        }
    }

    fn post_process_cli_args(&self) {
        log::info!("Post-processing CLI arguments");
    }

    fn run_analysis(&mut self, name: &str, f: impl FnOnce(&mut Self)) {
        log::info!("Running analysis: {}", name);
        f(self);
        log::info!("Finished analysis: {}", name);
    }

    pub fn run(&mut self) {
        self.pre_process_cli_args();
        self.run_analysis("FirstAnalysis", |analyzer| {
            FirstAnalysis::new(analyzer).run();
        });
        self.post_process_cli_args();
    }
}

struct FirstAnalysis<'tcx, 'a> {
    analyzer: &'a mut Analyzer<'tcx>,
    elapsed: Cell<Option<Duration>>,
}

impl<'tcx, 'a> FirstAnalysis<'tcx, 'a> {
    pub fn new(analyzer: &'a mut Analyzer<'tcx>) -> Self {
        Self {
            analyzer,
            elapsed: Cell::new(None),
        }
    }

    fn visitor(&self) {
        for local_def_id in self.analyzer.tcx.hir().body_owners() {
            let def_id = local_def_id.to_def_id();
            let body = self
                .analyzer
                .tcx
                .instance_mir(ty::InstanceKind::Item(def_id));
            let _promoted_mir = self.analyzer.tcx.promoted_mir(def_id);

            // TODO: Check if the body has some promoted MIR

            println!("{:#?}", def_id);
            let stmts = &body.basic_blocks[BasicBlock::new(0)].statements;
            if !stmts.is_empty() {
                let first = &stmts[0].kind;
                match first {
                    mir::StatementKind::Assign(bbox) => {
                        // let place = &bbox.0;
                        let rvalue = &bbox.1;
                        match rvalue {
                            mir::Rvalue::Use(_operand) => todo!(),
                            _ => println!(),
                        }
                    }
                    _ => println!(),
                }
            }

            // println!("{:#?}\n", body);
            // println!("{:#?}\n", promoted_mir);
            // println!("{:#?}\n", body.local_decls);
            // println!("{:#?}\n", body.basic_blocks);
        }
    }

    pub fn run(&self) {
        let start_time = std::time::Instant::now();
        self.visitor();
        let elapsed = start_time.elapsed();
        self.elapsed.set(Some(elapsed));
    }
}
