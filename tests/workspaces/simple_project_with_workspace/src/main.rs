use crate_a;
use crate_b;

fn main() {
    let _ = crate_a::add(1, 1);
    let _ = crate_b::sub(1, 1);
    println!("Hello, world!");
}
