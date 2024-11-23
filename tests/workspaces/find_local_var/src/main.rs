// Optionally: --use-unoptimized-mir
// cargo clean && cargo build --manifest-path ../../../Cargo.toml && RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links --color-log  --print-mir --print-rl-graph > mir



#[derive(Clone)]
struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    test_own(x.clone());

    let y = &test_own;
    y(x.clone());

    let z = test_own;
    z(x.clone());

    let lambda = |t: T, t2: T| {
        let x = T { _value: 10 };
        test_own(x);
    };
    
    test_own(x.clone());

    lambda(x.clone(), x);
}

fn test_own(t: T) {
    let _ = t;
}

// mod Test {
//     use super::T;
//     pub fn test_own(t: T) {}
// }
