struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    let test_own = |t: T| {
        let _ = t;
    };
    let y = test_own;
    y(x);
}