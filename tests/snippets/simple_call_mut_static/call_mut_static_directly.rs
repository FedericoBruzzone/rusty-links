struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    unsafe { TEST(x); }
}

static mut TEST: fn(T) = |t| {
    let _ = t;
};
