struct T {
    _value: i32,
}

fn main() {
    let dummy = 10;
    let x = T { _value: 10 };

    let lambda = || {
        if dummy == 10 {
            test(x);
        } else {
            test2(x);
        }
    };

    lambda();
}

fn test(t: T) {
    let _ = t;
}

fn test2(t: T) {
    let _ = t;
}
