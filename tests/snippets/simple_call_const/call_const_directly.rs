struct T {
    _value: i32,
}

fn main() {
    let x = T { _value: 10 };
    TEST(x);
}

const TEST: fn(T) = |t| {
    let _ = t;
};
