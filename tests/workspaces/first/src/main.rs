// enum MyEnum {
//     Variant(i32),
// }

// struct MyStruct(i32);

// fn main() {
//     let s = MyStruct(10);
//     let e = MyEnum::Variant(10);
// }


fn get_array() -> &'static [i32; 3] {
    // let _v = &[1, 2, 3]; // It creates another get_array::promoted
    &[1, 2, 3]
}

fn test(_: i32) {}

fn test2(_: &i32) {}

fn main() {
    // let x = 10;
    // let y = 10;
    test(10);
    // test2(&y);
}
