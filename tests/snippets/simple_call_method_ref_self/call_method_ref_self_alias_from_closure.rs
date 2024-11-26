struct T {
    _value: i32,
}

impl T {
    fn test(&self) {
        let _ = self;
    }
}

fn main() {
    let lambda = || {
        let x = T { _value: 10 };
        let y = T::test;
        y(&x);
    };

    lambda();
}
