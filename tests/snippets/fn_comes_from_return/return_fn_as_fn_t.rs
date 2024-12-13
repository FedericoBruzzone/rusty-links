#[derive(Clone, Copy)]
struct T {
    value: i32,
}
fn test(t: T) {
    let _ = t;
}
fn return_test() -> fn(T) {
    test
}

fn main() {
    let t = T { value: 10 };
    let f = return_test();
    f(t);
}