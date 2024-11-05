use crate::CliArgs;

use std::{cell::Cell, time::Duration};

use rustc_middle::mir;
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

    pub fn run(&mut self)  {
        self.pre_process_cli_args();
        self.run_analysis("FirstAnalysis", |analyzer| {
            FirstAnalysis::new(analyzer).run();
        });
        self.post_process_cli_args();
    }
}

struct FirstAnalysis<'tcx, 'a> {
    _analyzer: &'a mut Analyzer<'tcx>,
    elapsed: Cell<Option<Duration>>,
}

impl<'tcx, 'a> FirstAnalysis<'tcx, 'a> {
    pub fn new(analyzer: &'a mut Analyzer<'tcx>) -> Self {
        Self {
            _analyzer: analyzer,
            elapsed: Cell::new(None),
        }
    }

    fn visitor(&self) {
        todo!()
    }

    pub fn run(&self) {
        let start_time = std::time::Instant::now();
        self.visitor();
        let elapsed = start_time.elapsed();
        self.elapsed.set(Some(elapsed));
    }
}
