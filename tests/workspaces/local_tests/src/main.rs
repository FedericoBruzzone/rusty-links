// Optionally: --use-unoptimized-mir
// cargo clean && cargo build --manifest-path ../../../Cargo.toml && RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links --color-log  --print-mir --print-rl-graph > mir

// TODO:
// - Find a way to pass two mir::ConstValue::ZeroSized to the same function/closure (see line 305 of rl_visitor.rs)

#[derive(Clone)]
struct T {
    _value: i32,
}

fn test(t: T) {
    let _ = t;
}

fn main() {
    let mut x = test_own as fn(T);
    x = test as fn(T);
    x(T { _value: 10 });
    x(T { _value: 10 });

    // let x = T { _value: 10 };
    // test_own(x.clone());

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
}

fn test_fn(t: &dyn Fn() -> ()) {
    t();
}

fn test_own(t: T) {
    let _ = t;
}

// mod Test {
//     use super::T;
//     pub fn test_own(t: T) {}
// }
