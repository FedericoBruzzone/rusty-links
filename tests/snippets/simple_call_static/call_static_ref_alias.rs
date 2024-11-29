struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    let y = &TEST;
    y(x);
}

static TEST: fn(T) = |t| {
    let _ = t;
};

