struct T {
    _value: i32,
}

fn main() {
    let lambda = || {
        let x = T { _value: 10 };
        let y = TEST;
        y(x);
    };

    lambda();
}

const TEST: fn(T) = |t| {
    let _ = t;
};

