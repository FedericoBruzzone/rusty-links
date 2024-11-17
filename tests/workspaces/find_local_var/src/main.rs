// cargo clean && cargo build --manifest-path ../../../Cargo.toml && RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links --color-log --print-unoptimized-mir --print-rl-graph > mir

#[derive(Clone)]
struct T {
    value: i32,
}

fn main() {
    let x = T { value: 10 };
    test_own(x.clone());

    let y = &test_own;
    y(x.clone());

    let z = test_own;
    z(x);

    let lambda = || {
        let x = T { value: 10 };
        test_own(x);
    };
    
    lambda();
}

fn test_own(t: T) {
    let _ = t;
}

// mod Test {
//     use super::T;
//     pub fn test_own(t: T) {}
// }
