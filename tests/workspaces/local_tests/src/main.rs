// Optionally: --use-unoptimized-mir
// cargo clean && cargo build --manifest-path ../../../Cargo.toml && RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links --color-log  --print-mir --print-rl-graph > mir

// TODO:
// - Find a way to pass two mir::ConstValue::ZeroSized to the same function/closure (see line 305 of rl_visitor.rs)

// #[derive(Clone)]
// struct T {
//     _value: i32,
// }

// fn test(t: T) {
//     let _ = t;
// }
// fn test_fn(t: &dyn Fn() -> ()) {
//     t();
// }

// fn test_own(t: T) {
//     let _ = t;
// }

// fn test_borrow(t: &T) {
//     let _ = t;
// }

// fn test_mut_borrow(t: &mut T) {
//     let _ = t;
// }

// fn main() {
//     let x = T { _value: 10 };
//     test_own(x.clone());

//     let x = T { _value: 10 };
//     test_borrow(&x);

//     let mut x = T { _value: 10 };
//     test_mut_borrow(&mut x);

// let mut x = test_own as fn(T);
// x = test as fn(T);
// x(T { _value: 10 });

// let y = &test_own;
// y(x.clone());

// let z = test_own;
// z(x.clone());

// let lambda = |t: T, t2: T| {
//     let x = T { _value: 10 };
//     test_own(x);
// };
// lambda(x.clone(), x.clone());

// let lambda = |t: &T| {
//     let x = T { _value: 10 };
//     test_own(x);
// };
// lambda(&x.clone());

// let lambda = |bbox: std::boxed::Box<dyn Fn() -> T>| {
//     let x = T { _value: 10 };
//     test_own(x);
// };
// lambda(std::boxed::Box::new(|| x.clone()));

// // BUG
// let lambda = |llambda: &dyn Fn(T) -> T| {
//     let x = T { _value: 10 };
//     test_own(x);
// };

// lambda(&|_| x.clone());
// let lambda = |llambda: &dyn Fn() -> ()| {
//     llambda();
// };
// lambda(&|| {
//     let x = T { _value: 10 };
//     test_own(x.clone());
// });

// test_fn(&|| {
//     let x = T { _value: 10 };
//     test_own(x.clone());
// });
// }

// mod Test {
//     use super::T;
//     pub fn test_own(t: T) {}
// }

// ====================================================================================

#[derive(Clone)]
pub struct T {
    _value: i32,
}

#[derive(Clone, Copy)]
struct U {
    _value: i32,
}

// Constants
fn test_const(t: T) {
    let _ = t;
}

// Ownership transfer
fn test_move(t: T) {
    let _ = t;
}

// Borrowing
fn test_copy(t: &T) {
    let _ = t;
}

// Mutable borrowing
fn test_copy_mut(t: &mut T) {
    let _ = t;
}

fn test_const_ref(t: &T) {
    let _ = t;
}

fn main() {
    const xtmp: &T = &T { _value: 10 };
    test_const_ref(xtmp);

    const xc: T = T { _value: 10 };
    test_const(xc);

    let x = T { _value: 10 };
    test_move(x);

    let x = T { _value: 10 };
    test_move(x.clone());

    let x = T { _value: 10 };
    test_copy(&x);

    let mut x = T { _value: 10 };
    test_copy_mut(&mut x);

    let x = U { _value: 10 };
    test_for_u(x);
    let copy_x = x;

    let mut x = U { _value: 10 };
    x.test_self_ref_mut();
    x.test_self_ref();
    x.test_self();
}

fn test_for_u(u: U) {
    let _ = u;
}

trait Trait {
    fn test_self_ref_mut(&mut self);
    fn test_self_ref(&self);
    fn test_self(self);
}

impl Trait for U {
    fn test_self_ref_mut(&mut self) {
        let _ = self;
    }

    fn test_self_ref(&self) {
        let _ = self;
    }

    fn test_self(self) {
        let _ = self;
    }
}