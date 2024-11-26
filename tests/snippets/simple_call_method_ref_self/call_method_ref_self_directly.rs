struct T {
    _value: i32,
}

impl T {
    fn test(&self) {
        let _ = self;
    }
}

fn main() {
    let x = T { _value: 10 };
    (&x).test();
}