struct T {
    value: i32,
}

fn main() {
    let x = T { value: 10 };
    test_own(x);
    // Test::test_own(x);
}

fn test_own(t: T) {
    let _ = t;
}

// mod Test {
//     use super::T;
//     pub fn test_own(t: T) {}
// }

