struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };

    let z = test_own;
    z(x);
}

fn test_own(t: T) {
    let _ = t;
}