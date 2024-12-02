struct T {
    _value: i32,
}

fn main() {
    let dummy = 10;
    let call_f: &fn(T);
    let x = T { _value: 10 };

    if dummy == 10 {
        call_f = &(test as fn(T));
    } else {
        call_f = &(test2 as fn(T));
    }

    call_f(x);
}

fn test(t: T) {
    let _ = t;
}

fn test2(t: T) {
    let _ = t;
}
