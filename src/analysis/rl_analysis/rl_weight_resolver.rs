// use rustc_middle::mir;

// use super::rl_visitor::CallKind;

// pub struct RLWeightResolver;

// impl RLWeightResolver {
//     pub fn resolve_weights<'tcx>(call_kind: CallKind, args: Vec<mir::Operand<'tcx>>) -> Vec<f32> {
//         match call_kind {
//             CallKind::Function => Self::resolve_function_weights(args),
//             CallKind::Closure => Self::resolve_closure_weights(args),
//             CallKind::Method => Self::resolve_method_weights(args),
//             CallKind::Clone => unreachable!(),
//             CallKind::Unknown => unreachable!(),
//         }
//     }

//     fn resolve_function_weights<'tcx>(args: Vec<mir::Operand<'tcx>>) -> Vec<f32> {
//         todo!()
//     }

//     fn resolve_closure_weights<'tcx>(args: Vec<mir::Operand<'tcx>>) -> Vec<f32> {
//         todo!()
//     }

//     fn resolve_method_weights<'tcx>(args: Vec<mir::Operand<'tcx>>) -> Vec<f32> {
//         todo!()
//     }
// }
