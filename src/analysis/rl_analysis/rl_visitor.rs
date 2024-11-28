use crate::analysis::rl_analysis::rl_context::RLTy;
use crate::analysis::rl_analysis::rl_context::RLValue;
use crate::analysis::rl_analysis::rl_weight_resolver::RLWeightResolver;
use crate::analysis::utils::TextMod;

use rustc_middle::mir;
use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::Rvalue;
use rustc_middle::ty;
use rustc_span::def_id::DefId;
use rustc_span::def_id::LocalDefId;
use rustc_span::source_map::Spanned;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::rl_context::CallKind;
use super::rl_context::RLContext;
use super::rl_graph::RLGraph;
use super::rl_graph::RLGraphEdge;
use super::rl_graph::RLGraphNode;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use super::Analyzer;

pub struct RLVisitor<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    analyzer: &'a Analyzer<'tcx>,
    ctx: RLContext<'tcx, 'a, G>,
    rl_graph: G,
}

// Guardare le tre diverse tipologie di linear: copy move e borrow
impl<'tcx, 'a, G> RLVisitor<'tcx, 'a, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
        + Default
        + Clone
        + Serialize
        + DeserializeOwned,
{
    pub fn new(analyzer: &'a Analyzer<'tcx>) -> Self {
        Self {
            analyzer,
            ctx: RLContext::new(),
            rl_graph: G::default(),
        }
    }

    pub fn rl_graph(&self) -> G {
        self.rl_graph.clone()
    }

    /// The entry point of the visitor.
    /// It visits the local_def_id and the body of the function.
    pub fn visit_local_def_id(&mut self, local_def_id: LocalDefId, body: &'a mir::Body<'tcx>) {
        let _ = self.add_node_if_needed(local_def_id.to_def_id());

        self.ctx
            .stack_local_def_id
            .push((local_def_id.to_def_id(), &body.local_decls));

        // It ensures that the local variable is in the map.
        for (local, _) in body.local_decls.iter_enumerated() {
            self.ctx.map_place_rlvalue.insert(local, Vec::new());
        }

        // It ensures that the local variable is in the map.
        for (local, local_decl) in body.local_decls.iter_enumerated() {
            let ty = RLTy::new(
                local_decl.ty.kind(),
                local_decl.mutability,
                match local_decl.local_info.as_ref() {
                    mir::ClearCrossCrate::Set(v) => match v.as_ref() {
                        mir::LocalInfo::User(binding_form) => Some(binding_form.clone()),
                        _ => None,
                    },
                    mir::ClearCrossCrate::Clear => None,
                },
            );
            self.ctx.map_place_ty.insert(local, ty);
        }

        let message = self.analyzer.modify_if_needed(
            format!("Visiting the local_def_id: {:?}", local_def_id).as_str(),
            TextMod::Blue,
        );
        log::trace!("{}", message);
        self.visit_body(body);

        // Clear map_place_rvalue
        for (local, _) in body.local_decls.iter_enumerated() {
            self.ctx.map_place_rlvalue.remove(&local);
        }

        // Clear map_place_ty
        for (local, _) in body.local_decls.iter_enumerated() {
            self.ctx.map_place_ty.remove(&local);
        }

        self.ctx.stack_local_def_id.pop();
    }

    /// Update the arguments of the function call.
    /// It returns a vector of the arguments.
    ///
    /// In case of a function, the arguments are already in the correct format.
    ///
    /// In case of a closure, the arguments are in a tuple.
    /// The first argument is the closure itself.
    /// The second argument is a tuple of arguments which are passed to the closure.
    /// A vec of arguments is created by iterating over the tuple.
    fn update_args(
        &self,
        args: &[Spanned<mir::Operand<'tcx>>],
        call_kind: &CallKind,
    ) -> Vec<mir::Operand<'tcx>> {
        match call_kind {
            CallKind::Function => args.iter().map(|arg| arg.node.clone()).collect::<Vec<_>>(),
            CallKind::Method => args.iter().map(|arg| arg.node.clone()).collect::<Vec<_>>(),
            CallKind::Const => args.iter().map(|arg| arg.node.clone()).collect::<Vec<_>>(),
            CallKind::Static => args.iter().map(|arg| arg.node.clone()).collect::<Vec<_>>(),
            CallKind::StaticMut => args.iter().map(|arg| arg.node.clone()).collect::<Vec<_>>(),
            CallKind::Closure => {
                // It is safe to assume that the second argument is a tuple by construction.
                let args = match &args[1].node {
                    mir::Operand::Move(place) => {
                        let tuple = self.ctx.map_place_rlvalue[&place.local].last().unwrap();
                        match tuple {
                            RLValue::Rvalue(Rvalue::Aggregate(aggregate_kind, index_vec)) => {
                                match **aggregate_kind {
                                    mir::AggregateKind::Tuple => {
                                        let mut args = Vec::new();
                                        for index in index_vec {
                                            args.push(index.clone())
                                        }
                                        args
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    mir::Operand::Constant(const_operand) => {
                        // A closure with no arguments
                        if let mir::Const::Val(mir::ConstValue::ZeroSized, _) = const_operand.const_
                        {
                            return Vec::new();
                        }
                        todo!()
                    }
                    mir::Operand::Copy(_place) => {
                        // As far as I know, this case should not happen:
                        // in `std::ops::Fn<T>::call(&self, args: T)` the T is always `move` if
                        // there are arguments, or a ZST `Constant` if there are no arguments.
                        // Note: &self if the closure itself.
                        unreachable!()
                    }
                };
                args
            }
            CallKind::Clone => unreachable!(),
            CallKind::Unknown => unreachable!(),
        }
    }

    /// Add an edge between the current visited function and the function that is called.
    /// The edge is weighted by the arguments of the function call.
    /// The `to_def_id` is the def_id of the function that is called.
    /// Abstractly, the `from_def_id` is the def_id of the current visited function.
    fn add_edge(&mut self, to_def_id: DefId, arg_weights: Vec<f32>) {
        log::debug!(
            "Adding an edge between the current visited function ({:?}) and the function that is called ({:?}) with the arguments: {:?}",
            self.ctx.stack_local_def_id.last().unwrap().0,
            to_def_id,
            arg_weights
        );
        let fun_caller =
            self.ctx.rl_graph_index_map[&self.ctx.stack_local_def_id.last().unwrap().0];
        let fun_callee = self.add_node_if_needed(to_def_id);
        let edge = RLEdge::create(arg_weights);
        self.rl_graph.rl_add_edge(fun_caller, fun_callee, edge);
    }

    /// Add a node to the graph if it is not already present.
    /// This function returns the index of the node in the graph.
    ///
    /// It can be used also when an edge should be added between the current
    /// visited function and another function, calling it with the def_id of
    /// the called function.
    fn add_node_if_needed(&mut self, def_id: DefId) -> G::Index {
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.ctx.rl_graph_index_map.entry(def_id)
        {
            let node = RLNode::create(def_id);
            let index = self.rl_graph.rl_add_node(node);
            entry.insert(index);
        }
        self.ctx.rl_graph_index_map[&def_id]
    }
}

impl<'tcx, G> Visitor<'tcx> for RLVisitor<'tcx, '_, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
        + Default
        + Clone
        + Serialize
        + DeserializeOwned,
{
    // Entry point
    fn visit_body(&mut self, body: &mir::Body<'tcx>) {
        // log::trace!("Visiting the body {:?}", body);
        self.super_body(body);
    }

    // Call by the super_body
    fn visit_ty(&mut self, ty: ty::Ty<'tcx>, context: mir::visit::TyContext) {
        log::trace!("Visiting the ty: {:?}, {:?}", ty, context);
        self.super_ty(ty);
    }

    // Call by the super_body
    fn visit_basic_block_data(&mut self, block: mir::BasicBlock, data: &mir::BasicBlockData<'tcx>) {
        let message = self.analyzer.modify_if_needed(
            format!("Visiting the basic_block_data: {:?}, {:?}", block, data).as_str(),
            TextMod::Yellow,
        );
        log::trace!("{}", message);
        self.super_basic_block_data(block, data);
    }

    // TODO: implement
    // Call by the super_body
    fn visit_source_scope(&mut self, scope: mir::SourceScope) {
        self.super_source_scope(scope);
    }

    // TODO: implement
    // Call by the super_body
    fn visit_local_decl(&mut self, local: mir::Local, local_decl: &mir::LocalDecl<'tcx>) {
        self.super_local_decl(local, local_decl);
    }

    // TODO: implement
    // Call by the super_body
    fn visit_user_type_annotation(
        &mut self,
        index: ty::UserTypeAnnotationIndex,
        ty: &ty::CanonicalUserTypeAnnotation<'tcx>,
    ) {
        self.super_user_type_annotation(index, ty);
    }

    // TODO: implement
    // Call by the super_body
    fn visit_var_debug_info(&mut self, var_debug_info: &mir::VarDebugInfo<'tcx>) {
        self.super_var_debug_info(var_debug_info);
    }

    // TODO: implement
    // Call by the super_body
    fn visit_span(&mut self, span: rustc_span::Span) {
        self.super_span(span);
    }

    // Call by the super_body
    fn visit_const_operand(&mut self, constant: &mir::ConstOperand<'tcx>, location: mir::Location) {
        log::trace!("Visiting the const_operand: {:?}, {:?}", constant, location);
        self.super_const_operand(constant, location);
    }

    // Call by the super_const_operand
    fn visit_ty_const(&mut self, ct: ty::Const<'tcx>, location: mir::Location) {
        let message = self.analyzer.modify_if_needed(
            format!("Visiting the ty_const: {:?}, {:?}", ct, location).as_str(),
            TextMod::Magenta,
        );
        log::trace!("{}", message);
        self.super_ty_const(ct, location);
    }

    // Call by the super_basic_block_data
    fn visit_statement(&mut self, statement: &mir::Statement<'tcx>, location: mir::Location) {
        let message = self.analyzer.modify_if_needed(
            format!("Visiting the statement: {:?}, {:?}", statement, location).as_str(),
            TextMod::Green,
        );
        log::trace!("{}", message);
        self.super_statement(statement, location)
    }

    // Call by the super_basic_block_data
    fn visit_terminator(&mut self, terminator: &mir::Terminator<'tcx>, location: mir::Location) {
        let message = self.analyzer.modify_if_needed(
            format!("Visiting the terminator: {:?}, {:?}", terminator, location).as_str(),
            TextMod::Green,
        );
        log::trace!("{}", message);
        let mir::Terminator {
            source_info: _,
            kind,
        } = terminator;
        match kind {
            // We can analyze the `call_source` field of the `Call` variant
            // to know if it a `Normal` call.
            mir::TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                unwind,
                call_source,
                fn_span,
            } => {
                let message = self.analyzer.modify_if_needed(
                    format!(
                        "Visiting the call: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                        func, args, destination, target, unwind, call_source, fn_span
                    )
                    .as_str(),
                    TextMod::Magenta,
                );
                log::trace!("{}", message);

                let (def_id, call_kind) = self.ctx.retrieve_call_def_id(func, self.analyzer);

                // Update the map_place_rvalue with the destination of the call.
                match call_kind {
                    CallKind::Clone => {
                        self.ctx.push_or_insert_map_place_rlvalue(
                            destination.local,
                            RLValue::TermCallClone(args[0].node.clone()),
                        );
                    }
                    CallKind::Function | CallKind::Closure | CallKind::Method => {
                        self.ctx.push_or_insert_map_place_rlvalue(
                            destination.local,
                            RLValue::TermCall(def_id),
                        );
                    }
                    CallKind::Const => {
                        self.ctx.push_or_insert_map_place_rlvalue(
                            destination.local,
                            RLValue::TermCallConst(def_id),
                        );
                    }
                    CallKind::Static => {
                        self.ctx.push_or_insert_map_place_rlvalue(
                            destination.local,
                            RLValue::TermCallStatic(def_id),
                        );
                    }
                    CallKind::StaticMut => {
                        self.ctx.push_or_insert_map_place_rlvalue(
                            destination.local,
                            RLValue::TermCallStaticMut(def_id),
                        );
                    }
                    CallKind::Unknown => unreachable!(),
                }

                if call_kind != CallKind::Unknown && call_kind != CallKind::Clone {
                    let args = self.update_args(args, &call_kind);
                    let arg_weights =
                        RLWeightResolver::new(&self.ctx).resolve_arg_weights(&call_kind, &args);
                    self.add_edge(def_id, arg_weights);
                }

                self.visit_place(
                    destination,
                    mir::visit::PlaceContext::MutatingUse(mir::visit::MutatingUseContext::Call),
                    location,
                );
            }
            _ => self.super_terminator(terminator, location),
        }
    }

    // Call by the super_terminator
    fn visit_operand(&mut self, operand: &mir::Operand<'tcx>, location: mir::Location) {
        log::trace!("Visiting the operand: {:?}, {:?}", operand, location);
        self.super_operand(operand, location)
    }

    // Call by the super_statement
    // Call by the super_terminator
    fn visit_source_info(&mut self, source_info: &mir::SourceInfo) {
        log::trace!("Visiting the source info: {:?}", source_info);
        self.super_source_info(source_info)
    }

    // Call by super_statement
    fn visit_assign(
        &mut self,
        place: &mir::Place<'tcx>,
        rvalue: &mir::Rvalue<'tcx>,
        location: mir::Location,
    ) {
        let message = self.analyzer.modify_if_needed(
            format!(
                "Visiting the assign: {:?}, {:?}, {:?}",
                place, rvalue, location
            )
            .as_str(),
            TextMod::Magenta,
        );
        log::trace!("{}", message);

        self.ctx
            .push_or_insert_map_place_rlvalue(place.local, RLValue::Rvalue(rvalue.clone()));
        self.super_assign(place, rvalue, location);
    }

    // Call by the super_assign
    fn visit_place(
        &mut self,
        place: &mir::Place<'tcx>,
        context: mir::visit::PlaceContext,
        location: mir::Location,
    ) {
        log::trace!(
            "Visiting the place: {:?}, {:?}, {:?}",
            place,
            context,
            location
        );
        self.super_place(place, context, location);
    }

    // Call by the super_assign and super_place
    fn visit_local(
        &mut self,
        local: mir::Local,
        context: mir::visit::PlaceContext,
        location: mir::Location,
    ) {
        log::trace!(
            "Visiting the local: {:?}, {:?}, {:?}",
            local,
            context,
            location
        );
        match context {
            mir::visit::PlaceContext::NonUse(non_use_context) => match non_use_context {
                mir::visit::NonUseContext::StorageDead => {
                    // We can not remove the local from the map_place_rvalue
                    // because when we need to find the origian rvalue of a local,
                    // we need to look at the map_place_rvalue.
                    // let _ = self.map_place_rvalue.insert(local, None);
                }
                mir::visit::NonUseContext::StorageLive => {
                    // It is not always true that if the map contains the local,
                    // then the value is not None.
                    // For intance, the first `bb`` can have a `StorageLive` for a local
                    // that is only initialized in the local_decls, but never assigned.
                    assert!(self.ctx.map_place_rlvalue.contains_key(&local))
                }
                mir::visit::NonUseContext::AscribeUserTy(_variance) => {}
                mir::visit::NonUseContext::VarDebugInfo => {}
            },
            mir::visit::PlaceContext::NonMutatingUse(_non_mutating_use_context) => { /* TODO */ }
            mir::visit::PlaceContext::MutatingUse(_mutating_use_context) => { /* TODO */ }
        }
    }

    // Call by the super_assign
    fn visit_rvalue(&mut self, rvalue: &mir::Rvalue<'tcx>, location: mir::Location) {
        let mut message = format!("Visiting the rvalue ({:?}, {:?})", rvalue, location);
        match rvalue {
            mir::Rvalue::Use(operand) => match operand {
                mir::Operand::Copy(place) => {
                    message.push_str(format!(" Use(Copy): place: {:?}", place).as_str());
                }
                mir::Operand::Move(place) => {
                    message.push_str(format!(" Use(Move): place: {:?}", place).as_str());
                }
                mir::Operand::Constant(const_operand) => {
                    message.push_str(
                        format!(" Use(Constant): const_operand: {:?}", const_operand).as_str(),
                    );
                }
            },
            mir::Rvalue::Repeat(operand, _) => {
                message.push_str(format!(" Repeat(Operand): {:?}", operand).as_str());
            }
            mir::Rvalue::Ref(region, borrow_kind, place) => {
                message.push_str(
                    format!(
                        " Ref: region: {:?}, borrow_kind: {:?}, place: {:?}",
                        region, borrow_kind, place
                    )
                    .as_str(),
                );
            }
            mir::Rvalue::ThreadLocalRef(def_id) => {
                message.push_str(format!(" ThreadLocalRef: def_id: {:?}", def_id).as_str());
            }
            mir::Rvalue::RawPtr(mutability, place) => {
                message.push_str(
                    format!(" RawPtr: mutability: {:?}, place: {:?}", mutability, place).as_str(),
                );
            }
            mir::Rvalue::Len(place) => {
                message.push_str(format!(" Len: place: {:?}", place).as_str());
            }
            mir::Rvalue::Cast(cast_kind, operand, ty) => {
                message.push_str(
                    format!(
                        " Cast: cast_kind: {:?}, operand: {:?}, ty: {:?}",
                        cast_kind, operand, ty
                    )
                    .as_str(),
                );
            }
            mir::Rvalue::BinaryOp(bin_op, _) => {
                message.push_str(format!(" BinaryOp: bin_op: {:?}", bin_op).as_str());
            }
            mir::Rvalue::NullaryOp(null_op, ty) => {
                message
                    .push_str(format!(" NullaryOp: null_op: {:?}, ty: {:?}", null_op, ty).as_str());
            }
            mir::Rvalue::UnaryOp(un_op, operand) => {
                message.push_str(
                    format!(" UnaryOp: un_op: {:?}, operand: {:?}", un_op, operand).as_str(),
                );
            }
            mir::Rvalue::Discriminant(place) => {
                message.push_str(format!(" Discriminant: place: {:?}", place).as_str());
            }
            mir::Rvalue::Aggregate(aggregate_kind, index_vec) => {
                message.push_str(
                    format!(
                        " Aggregate: aggregate_kind: {:?}, index_vec: {:?}",
                        aggregate_kind, index_vec
                    )
                    .as_str(),
                );
            }
            mir::Rvalue::ShallowInitBox(operand, ty) => {
                message.push_str(
                    format!(" ShallowInitBox: operand: {:?}, ty: {:?}", operand, ty).as_str(),
                );
            }
            mir::Rvalue::CopyForDeref(place) => {
                message.push_str(format!(" CopyForDeref: place: {:?}", place).as_str());
            }
        }
        log::trace!("{}", message);
    }
}
