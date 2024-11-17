use crate::analysis::utils::TextMod;

use rustc_index::IndexVec;
use rustc_middle::mir;
use rustc_middle::mir::visit::Visitor;
use rustc_middle::ty;
use rustc_span::def_id::DefId;
use rustc_span::def_id::LocalDefId;

use super::rl_graph::RLGraph;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use super::Analyzer;

pub struct RLVisitor<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone,
{
    analyzer: &'a Analyzer<'tcx, G>,

    // Stack of local_def_id and local_decls
    stack_local_def_id: Vec<(DefId, &'a IndexVec<mir::Local, mir::LocalDecl<'tcx>>)>,

    // Map of places and their rvalues
    // The value can be None when the respective local go out of scope,
    // thanks to the borrow checker semantic.
    // See `visit_local` function.
    map_place_rvalue: rustc_hash::FxHashMap<mir::Local, Option<mir::Rvalue<'tcx>>>,

    // Map from def_id to the index of the node in the graph.
    // It is used to retrieve the index of the node in the graph
    // when we need to add an edge.
    rl_index_map: rustc_hash::FxHashMap<DefId, G::Index>,

    // The graph that represents the relations between functions and their calls.
    rl_graph: G,
}

// Guardare le tre diverse tipologie di linear: copy move e borrow
impl<'tcx, 'a, G> RLVisitor<'tcx, 'a, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex> + Default + Clone,
{
    pub fn new(analyzer: &'a Analyzer<'tcx, G>) -> Self {
        Self {
            analyzer,
            stack_local_def_id: Vec::default(),
            map_place_rvalue: rustc_hash::FxHashMap::default(),
            rl_index_map: rustc_hash::FxHashMap::default(),
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

        self.stack_local_def_id
            .push((local_def_id.to_def_id(), &body.local_decls));

        // It ensures that the local variable is in the map.
        for (local, _) in body.local_decls.iter_enumerated() {
            self.map_place_rvalue.insert(local, None);
        }

        let message = self.analyzer.modify_if_needed(
            format!("Visiting the local_def_id: {:?}", local_def_id).as_str(),
            TextMod::Blue,
        );
        log::trace!("{}", message);
        self.visit_body(body);

        // Clear map_place_rvalue
        for (local, _) in body.local_decls.iter_enumerated() {
            self.map_place_rvalue.remove(&local);
        }
        self.stack_local_def_id.pop();
    }

    /// Recursively retrieve the def_id of the function.
    /// This function assumes that the `local` is a function, so
    /// it panics if it is not.
    fn retrieve_fun_def_id(&self, local: mir::Local) -> DefId {
        log::debug!("Retrieving the def_id of the function (local: {:?})", local);
        match &self.map_place_rvalue[&local] {
            // _5 = copy (*_6)
            Some(mir::Rvalue::Use(mir::Operand::Copy(place))) => {
                self.retrieve_fun_def_id(place.local)
            }
            // _5 = move _6
            Some(mir::Rvalue::Use(mir::Operand::Move(place))) => {
                self.retrieve_fun_def_id(place.local)
            }
            // _5 = &(*10)
            Some(mir::Rvalue::Ref(_region, _borrow_kind, place)) => {
                self.retrieve_fun_def_id(place.local)
            }
            // _5 = function
            // _5 = const main::promoted[0]
            Some(mir::Rvalue::Use(mir::Operand::Constant(const_operand))) => {
                match const_operand.const_.ty().kind() {
                    ty::TyKind::FnDef(def_id, _generic_args) => *def_id,
                    ty::TyKind::Ref(_region, ty, _mutability) => match ty.kind() {
                        ty::TyKind::FnDef(def_id, _generic_args) => *def_id,
                        _ => unreachable!(),
                    },
                    _ => {
                        log::error!(
                            "The local ({:?}) is not a function, but a constant: {:?}",
                            local,
                            const_operand.const_.ty()
                        );
                        unreachable!()
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn add_node_if_needed(&mut self, def_id: DefId) -> G::Index {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.rl_index_map.entry(def_id) {
            let node = RLNode::new(def_id);
            let index = self.rl_graph.rl_add_node(node);
            entry.insert(index);
        }
        self.rl_index_map[&def_id]
    }

    fn resolve_arg_weight(&self, _operand: &mir::Operand<'tcx>) -> f32 {
        1.0
    }
}

impl<'tcx, G> Visitor<'tcx> for RLVisitor<'tcx, '_, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex> + Default + Clone,
{
    // Entry point
    fn visit_body(&mut self, body: &mir::Body<'tcx>) {
        // log::trace!("Visiting the body {:?}", body);
        self.super_body(body);
    }

    // Call by the super_body
    fn visit_ty(&mut self, ty: ty::Ty<'tcx>, context: mir::visit::TyContext) {
        log::trace!("Visiting the ty: {:?}, {:?}", ty, context);
        // TODO: We should visit the `FnDef` because in `_12 = test_own(move _13) -> [return: bb5, unwind continue];`
        // `test_own` is a `FnDef`.
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
            mir::TerminatorKind::Call {
                func,
                args,
                destination,
                ..
            } => {
                let message = self.analyzer.modify_if_needed(
                    format!(
                        "Visiting the call: {:?}, {:?}, {:?}",
                        func, args, destination
                    )
                    .as_str(),
                    TextMod::Magenta,
                );
                log::trace!("{}", message);

                let fun_def_id = match func {
                    mir::Operand::Copy(place) => {
                        let fun_def_id = self.retrieve_fun_def_id(place.local);
                        self.visit_place(
                            place,
                            mir::visit::PlaceContext::NonMutatingUse(
                                mir::visit::NonMutatingUseContext::Copy,
                            ),
                            location,
                        );
                        log::debug!(
                            "Retrieving the def_id of the function (local: {:?}) that is called",
                            place.local
                        );
                        fun_def_id
                    }
                    mir::Operand::Move(place) => {
                        let fun_def_id = self.retrieve_fun_def_id(place.local);
                        self.visit_place(
                            place,
                            mir::visit::PlaceContext::NonMutatingUse(
                                mir::visit::NonMutatingUseContext::Move,
                            ),
                            location,
                        );
                        log::debug!(
                            "Retrieving the def_id of the function (local: {:?}) that is called",
                            place.local
                        );
                        fun_def_id
                    }
                    mir::Operand::Constant(const_operand) => match const_operand.const_.ty().kind()
                    {
                        ty::TyKind::FnDef(def_id, _generic_args) => *def_id,
                        _ => unreachable!(),
                    },
                };

                let fun_caller = self.rl_index_map[&self.stack_local_def_id.last().unwrap().0];
                let fun_callee = self.add_node_if_needed(fun_def_id);
                let arg_weights = args
                    .iter()
                    .map(|arg| self.resolve_arg_weight(&arg.node))
                    .collect::<Vec<f32>>();
                let edge = RLEdge::new(arg_weights);
                self.rl_graph.rl_add_edge(fun_caller, fun_callee, edge);

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
        self.map_place_rvalue
            .insert(place.local, Some(rvalue.clone()));
        self.super_assign(place, rvalue, location);
    }

    // NOT NEEDED
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

    // Call by the super_assign
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
                    let _ = self.map_place_rvalue.insert(local, None);
                }
                mir::visit::NonUseContext::StorageLive => {
                    // It is not always true that if the map contains the local,
                    // then the value is not None.
                    // For intance, the first `bb`` can have a `StorageLive` for a local
                    // that is only initialized in the local_decls, but never assigned.
                    assert!(self.map_place_rvalue.contains_key(&local))
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
            mir::Rvalue::Use(operand) => {
                match operand {
                    mir::Operand::Copy(place) => {
                        message.push_str(format!(" Use: copy operand: {:?}", place).as_str());
                    }
                    mir::Operand::Move(place) => {
                        message.push_str(format!(" Use: operand: {:?}", place).as_str());
                    }
                    mir::Operand::Constant(const_operand) => {
                        message.push_str(format!(" Use: operand: {:?}", const_operand).as_str());
                    }
                }
                // message.push_str(format!(" Use: operand: {:?}", operand).as_str());
            }
            mir::Rvalue::Repeat(operand, _) => {
                message.push_str(format!(" Repeat: operand: {:?}", operand).as_str());
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
