struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    test_own(x);
}

fn test_own(t: T) {
    let _ = t;
}