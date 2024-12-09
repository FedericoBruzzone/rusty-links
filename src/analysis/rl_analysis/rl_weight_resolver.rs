use std::ops::Deref;

use rustc_middle::mir;
use serde::{Deserialize, Serialize};

use super::{
    rl_context::{CallKind, RLContext},
    rl_graph::RLGraph,
};

const STATIC_MUT_CALL_MULTIPLIER: f32 = 1.0;
const STATIC_CALL_MULTIPLIER: f32 = 1.0;
const METHOD_CALL_MULTIPLIER: f32 = 1.0;
const FUNCTION_CALL_MULTIPLIER: f32 = 1.0;
const CLOSURE_CALL_MULTIPLIER: f32 = 1.0;
const CONST_CALL_MULTIPLIER: f32 = 1.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallKindMultiplier {
    StaticMut,
    Static,
    Method,
    Function,
    Closure,
    Const,
}

impl Deref for CallKindMultiplier {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        match self {
            CallKindMultiplier::StaticMut => &STATIC_MUT_CALL_MULTIPLIER,
            CallKindMultiplier::Static => &STATIC_CALL_MULTIPLIER,
            CallKindMultiplier::Method => &METHOD_CALL_MULTIPLIER,
            CallKindMultiplier::Function => &FUNCTION_CALL_MULTIPLIER,
            CallKindMultiplier::Closure => &CLOSURE_CALL_MULTIPLIER,
            CallKindMultiplier::Const => &CONST_CALL_MULTIPLIER,
        }
    }
}

const MOVE_OPERAND_MULTIPLIER: f32 = 1.0;
const COPY_OPERAND_MULTIPLIER: f32 = 1.0;
const CONSTANT_OPERAND_MULTIPLIER: f32 = 1.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperandMultiplier {
    Move,
    Copy,
    Constant,
}

impl Deref for OperandMultiplier {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        match self {
            OperandMultiplier::Move => &MOVE_OPERAND_MULTIPLIER,
            OperandMultiplier::Copy => &COPY_OPERAND_MULTIPLIER,
            OperandMultiplier::Constant => &CONSTANT_OPERAND_MULTIPLIER,
        }
    }
}

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
    ) -> (CallKindMultiplier, Vec<(OperandMultiplier, f32)>) {
        match call_kind {
            CallKind::StaticMut => (CallKindMultiplier::StaticMut, self.resolve_mut_static(args)),
            CallKind::Static => (CallKindMultiplier::Static, self.resolve_static(args)),
            CallKind::Method => (
                CallKindMultiplier::Method,
                self.resolve_method_weights(args),
            ),
            CallKind::Function => (
                CallKindMultiplier::Function,
                self.resolve_function_weights(args),
            ),
            CallKind::Closure => (
                CallKindMultiplier::Closure,
                self.resolve_closure_weights(args),
            ),
            CallKind::Const => (CallKindMultiplier::Const, self.resolve_const(args)),
            CallKind::Clone => unreachable!(),
            CallKind::Unknown => unreachable!(),
        }
    }

    #[inline(always)]
    fn resolve_const(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<(OperandMultiplier, f32)> {
        self.resolve_args(args)
    }

    #[inline(always)]
    fn resolve_static(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<(OperandMultiplier, f32)> {
        self.resolve_args(args)
    }

    #[inline(always)]
    fn resolve_mut_static(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<(OperandMultiplier, f32)> {
        self.resolve_args(args)
    }

    #[inline(always)]
    fn resolve_method_weights(
        &self,
        args: &'a [mir::Operand<'tcx>],
    ) -> Vec<(OperandMultiplier, f32)> {
        let self_weight = self.resolve_self(&args[0]);
        let mut arg_weights = if args.len() > 1 {
            self.resolve_args(&args[1..])
        } else {
            Vec::new()
        };
        arg_weights.insert(0, self_weight);
        arg_weights
    }

    #[inline(always)]
    fn resolve_function_weights(
        &self,
        args: &'a [mir::Operand<'tcx>],
    ) -> Vec<(OperandMultiplier, f32)> {
        self.resolve_args(args)
    }

    #[inline(always)]
    fn resolve_closure_weights(
        &self,
        args: &'a [mir::Operand<'tcx>],
    ) -> Vec<(OperandMultiplier, f32)> {
        self.resolve_args(args)
    }

    fn resolve_args(&self, args: &'a [mir::Operand<'tcx>]) -> Vec<(OperandMultiplier, f32)> {
        let mut arg_weights = Vec::new();
        for arg in args {
            arg_weights.push(self.resolve_arg(arg));
        }
        arg_weights
    }

    fn resolve_self(&self, zelf: &mir::Operand<'tcx>) -> (OperandMultiplier, f32) {
        // TODO: For sake of simplicity, the self is treated as an argument
        self.resolve_arg(zelf)
    }

    fn resolve_arg(&self, arg: &mir::Operand<'tcx>) -> (OperandMultiplier, f32) {
        match arg {
            mir::Operand::Move(place) => (OperandMultiplier::Move, self.resolve_place(place)),
            mir::Operand::Copy(place) => (OperandMultiplier::Copy, self.resolve_place(place)),
            mir::Operand::Constant(const_operand) => (
                OperandMultiplier::Constant,
                self.resolve_const_operand(const_operand),
            ),
        }
    }

    fn resolve_place(&self, _place: &mir::Place<'tcx>) -> f32 {
        // Handle the clone
        // Static is not good
        // Consider the type of the place (if it is a reference, a value, etc.)
        1.0
    }

    fn resolve_const_operand(&self, _const_operand: &mir::ConstOperand<'tcx>) -> f32 {
        1.0
    }
}
