use rustc_middle::mir;
use serde::Serialize;

use super::{
    rl_context::{CallKind, RLContext},
    rl_graph::RLGraph,
};

const MOVE_MULTIPLIER: f32 = 1.0;
const COPY_MULTIPLIER: f32 = 1.0;
const CONSTANT_MULTIPLIER: f32 = 1.0;

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
    pub fn new(ctx: &'a RLContext<'tcx, 'a, G>) -> Self {
        Self { _ctx: ctx }
    }

    pub fn resolve_arg_weights(
        &self,
        call_kind: &'a CallKind,
        args: &'a [mir::Operand<'tcx>],
    ) -> Vec<f32> {
        match call_kind {
            CallKind::StaticMut => self.resolve_mut_static(args),
            CallKind::Static => self.resolve_static(args),
            CallKind::Const => self.resolve_const(args),
            CallKind::Method => self.resolve_method_weights(args),
            CallKind::Function => self.resolve_function_weights(args),
            CallKind::Closure => self.resolve_closure_weights(args),
            CallKind::Clone => unreachable!(),
            CallKind::Unknown => unreachable!(),
        }
    }

    fn resolve_const(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_static(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_mut_static(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_method_weights(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        let self_weight = self.resolve_self(&args[0]);
        let mut arg_weights = self.resolve_args(&args[1..]);
        arg_weights.insert(0, self_weight);
        arg_weights
    }

    fn resolve_function_weights(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_closure_weights(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        self.resolve_args(args)
    }

    fn resolve_args(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<f32> {
        let mut arg_weights = Vec::new();
        for arg in args {
            arg_weights.push(self.resolve_arg(arg));
        }
        arg_weights
    }

    fn resolve_self(&self, _zelf: &mir::Operand<'tcx>) -> f32 {
        // TODO: implement
        1.0
    }

    fn resolve_arg(&self, arg: &mir::Operand<'tcx>) -> f32 {
        match arg {
            mir::Operand::Move(place) => self.resolve_place(place) * MOVE_MULTIPLIER,
            mir::Operand::Copy(place) => self.resolve_place(place) * COPY_MULTIPLIER,
            mir::Operand::Constant(const_operand) => {
                self.resolve_const_operand(const_operand) * CONSTANT_MULTIPLIER
            } // Static is not good
        }
    }

    fn resolve_place(&self, _place: &mir::Place<'tcx>) -> f32 {
        1.0
    }

    fn resolve_const_operand(&self, _const_operand: &mir::ConstOperand<'tcx>) -> f32 {
        1.0
    }
}
