fn get_array() -> &'static [i32; 3] {
    // let _v = &[1, 2, 3]; // It creates another get_array::promoted
    &[1, 2, 3]
}

fn test(_: i32) {}

fn test2(_: &i32) {}

fn main() {
    let x = 10;
    let y = 10;
    test(x);
    test2(&y);
    println!("Hello, world!");
}
