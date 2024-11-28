struct T {
    _value: i32,
}

fn main() {
    let lambda = || {
        let x = T { _value: 10 };
        TEST(x);
    };
    
    lambda();
}

const TEST: fn(T) = |t| {
    let _ = t;
};
