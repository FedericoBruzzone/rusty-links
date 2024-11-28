struct T {
    _value: i32,
}

fn main() {
    let lambda = || {
        let x = T { _value: 10 };
        unsafe { TEST(x); }
    };

    lambda();
}

static mut TEST: fn(T) = |t| {
    let _ = t;
};
