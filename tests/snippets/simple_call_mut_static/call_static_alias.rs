struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    let y = unsafe { TEST };
    y(x);
}

static mut TEST: fn(T) = |t| {
    let _ = t;
};
