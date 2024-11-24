= Notes


## 2024-11-25

Example of a building block in which the rvalue of the local (_1) depends the previous building block.

Rust:
```rust
#[derive(Clone)]
struct T {
    _value: i32,
}

fn test(t: T) {
    let _ = t;
}

fn main() {
    let mut x = test_own as fn(T);
    x = test as fn(T);
    x(T { _value: 10 });
    x(T { _value: 10 });
}
```

MIR:
```rust
// WARNING: This output format is intended for human consumers only
// and is subject to change without notice. Knock yourself out.
fn <impl at src/main.rs:7:10: 7:15>::clone(_1: &T) -> T {
    debug self => _1;
    let mut _0: T;
    let mut _2: i32;
    let mut _3: &i32;
    let _4: &i32;

    bb0: {
        StorageLive(_2);
        StorageLive(_3);
        StorageLive(_4);
        _4 = &((*_1).0: i32);
        _3 = &(*_4);
        _2 = <i32 as std::clone::Clone>::clone(move _3) -> [return: bb1, unwind continue];
    }

    bb1: {
        StorageDead(_3);
        _0 = T { _value: move _2 };
        StorageDead(_2);
        StorageDead(_4);
        return;
    }
}

fn test(_1: T) -> () {
    debug t => _1;
    let mut _0: ();
    scope 1 {
    }

    bb0: {
        _0 = const ();
        return;
    }
}

fn main() -> () {
    let mut _0: ();
    let mut _1: fn(T);
    let mut _2: fn(T);
    let _3: ();
    let mut _4: fn(T);
    let mut _5: T;
    let _6: ();
    let mut _7: fn(T);
    let mut _8: T;
    scope 1 {
        debug x => _1;
    }

    bb0: {
        StorageLive(_1);
        _1 = test_own as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
        StorageLive(_2);
        _2 = test as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
        _1 = move _2;
        StorageDead(_2);
        StorageLive(_3);
        StorageLive(_4);
        _4 = copy _1;
        StorageLive(_5);
        _5 = T { _value: const 10_i32 };
        _3 = move _4(move _5) -> [return: bb1, unwind continue];
    }

    bb1: {
        StorageDead(_5);
        StorageDead(_4);
        StorageDead(_3);
        StorageLive(_6);
        StorageLive(_7);
        _7 = copy _1;
        StorageLive(_8);
        _8 = T { _value: const 10_i32 };
        _6 = move _7(move _8) -> [return: bb2, unwind continue];
    }

    bb2: {
        StorageDead(_8);
        StorageDead(_7);
        StorageDead(_6);
        _0 = const ();
        StorageDead(_1);
        return;
    }
}
```



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

