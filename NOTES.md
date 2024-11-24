= Notes

## 2024-11-24

In this case main should point to test because we overwrite the function pointer.

```rust
struct T {
    _value: i32,
}
fn test(_: T) {}
fn test_own(_: T) {}

fn main() {
    let mut x = test_own as fn(T);
    x = test as fn(T);
    x(T { _value: 10 });
}
```

