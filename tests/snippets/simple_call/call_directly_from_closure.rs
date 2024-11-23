struct T {
    _value: i32,
}

fn main() {
    let lambda = || {
        let x = T { _value: 10 };
        test_own(x);
    };
    
    lambda();
}

fn test_own(t: T) {
    let _ = t;
}