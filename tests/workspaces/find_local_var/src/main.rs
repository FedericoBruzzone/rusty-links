struct T {
    value: i32,
}

fn test_own(t: T) {
    let _ = t;
}

fn main() {
    let x = T { value: 10 };
    test_own(x);
}
