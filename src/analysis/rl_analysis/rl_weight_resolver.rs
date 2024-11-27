use rustc_middle::mir;
use serde::Serialize;

use super::{
    rl_context::{CallKind, RLContext},
    rl_graph::RLGraph,
};

pub struct RLWeightResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    _ctx: &'a RLContext<'tcx, 'a, G>,
}

impl<'tcx, 'a, G> RLWeightResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    pub fn new(context: &'a RLContext<'tcx, 'a, G>) -> Self {
        Self { _ctx: context }
    }

    pub fn resolve_arg_weights(
        &self,
        call_kind: &'a CallKind,
        args: &'a Vec<mir::Operand<'tcx>>,
    ) -> Vec<f32> {
        // return self.args.iter().map(|_| 1.0).collect();
        match call_kind {
            CallKind::Method => self.resolve_method_weights(args),
            CallKind::Function => self.resolve_function_weights(args),
            CallKind::Closure => self.resolve_closure_weights(args),
            CallKind::Clone => unreachable!(),
            CallKind::Unknown => unreachable!(),
        }
    }

    fn resolve_method_weights(&self, args: &'a Vec<mir::Operand<'tcx>>) -> Vec<f32> {
        let self_weight = self.resolve_self_weight(&args[0]);
        let mut arg_weights = self.resolve_args(&args[1..].into());
        arg_weights.insert(0, self_weight);
        arg_weights
    }

    fn resolve_function_weights(&self, args: &'a Vec<mir::Operand<'tcx>>) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_closure_weights(&self, args: &'a Vec<mir::Operand<'tcx>>) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_args(&self, args: &Vec<mir::Operand<'tcx>>) -> Vec<f32> {
        let mut arg_weights = Vec::new();
        for arg in args {
            arg_weights.push(self.resolve_arg(arg));
        }
        arg_weights
    }

    fn resolve_self_weight(&self, _zelf: &mir::Operand<'tcx>) -> f32 {
        // TODO: implement
        1.0
    }

    fn resolve_arg(&self, _arg: &mir::Operand<'tcx>) -> f32 {
        // TODO: implement
        1.0
    }
}
