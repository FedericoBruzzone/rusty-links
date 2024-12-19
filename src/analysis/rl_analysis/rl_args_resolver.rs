use rustc_middle::mir;
use serde::Serialize;

use super::{
    rl_context::{CallKind, MutabilityKind, OperandKind, RLContext, RLTyKind},
    rl_graph::RLGraph,
};

pub struct RLArgsResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    ctx: &'a RLContext<'tcx, 'a, G>,
}

impl<'tcx, 'a, G> RLArgsResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    pub fn new(ctx: &'a RLContext<'tcx, 'a, G>) -> Self {
        Self { ctx }
    }

    pub fn resolve(
        &self,
        call_kind: &'a CallKind,
        args: &'a [mir::Operand<'tcx>],
    ) -> (CallKind, Vec<(OperandKind, MutabilityKind, RLTyKind)>) {
        (call_kind.clone(), self.resolve_arg_types(args))
    }

    fn resolve_arg_types(
        &self,
        args: &'a [mir::Operand<'tcx>],
    ) -> Vec<(OperandKind, MutabilityKind, RLTyKind)> {
        let mut arg_weights = Vec::new();
        for arg in args {
            arg_weights.push(self.resolve_arg_type(arg));
        }
        arg_weights
    }

    fn resolve_arg_type(
        &self,
        arg: &mir::Operand<'tcx>,
    ) -> (OperandKind, MutabilityKind, RLTyKind) {
        match arg {
            mir::Operand::Move(place) => {
                let (mutability, rl_ty) = self.resolve_place_type(place);
                (OperandKind::Move, mutability, rl_ty)
            }
            mir::Operand::Copy(place) => {
                let (mutability, rl_ty) = self.resolve_place_type(place);
                (OperandKind::Copy, mutability, rl_ty)
            }
            mir::Operand::Constant(const_operand) => {
                let (mutability, rl_ty) = self.resolve_const_type(const_operand);
                assert!(mutability == MutabilityKind::Not);
                (OperandKind::Constant, mutability, rl_ty)
            }
        }
    }

    fn resolve_place_type(&self, place: &mir::Place<'tcx>) -> (MutabilityKind, RLTyKind) {
        // Handle the clone
        // Static is not good
        // Consider the type of the place (if it is a reference, a value, etc.)
        
        // TODO: find the top-level place (user-defined type)

        let rl_ty = self.ctx.map_place_ty[&place.local].clone();
        (rl_ty.mutability(), RLTyKind::from(rl_ty.kind()))
    }

    fn resolve_const_type(
        &self,
        const_operand: &mir::ConstOperand<'tcx>,
    ) -> (MutabilityKind, RLTyKind) {
        // Handle the constant
        // Static is not good
        // Consider the type of the constant
        (
            MutabilityKind::Not,
            RLTyKind::from(const_operand.const_.ty().kind()),
        )
    }
}
