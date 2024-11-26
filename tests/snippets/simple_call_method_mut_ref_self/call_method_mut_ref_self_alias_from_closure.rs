struct T {
    _value: i32,
}

impl T {
    fn test(&mut self) {
        let _ = self;
    }
}

fn main() {
    let lambda = || {
        let mut x = T { _value: 10 };
        let y = T::test;
        y(&mut x);
    };

    lambda();
}
