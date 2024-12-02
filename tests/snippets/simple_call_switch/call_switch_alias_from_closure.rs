struct T {
    _value: i32,
}

fn main() {
    let lambda = || {
        let dummy = 10;
        let call_f: fn(T);
        let x = T { _value: 10 };
        if dummy == 10 {
            call_f = test;
        } else {
            call_f = test2;
        }

        call_f(x);
    };
    lambda();
}

fn test(t: T) {
    let _ = t;
}

fn test2(t: T) {
    let _ = t;
}
