# Compilation CLI


```bash
clear && cargo clean && cargo build --manifest-path ../../../Cargo.toml && RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links --color-log  --print-mir --print-rl-graph > mir
```

```bash
rm -rf target && cargo build --manifest-path ../../Cargo.toml && RUSTC_BOOTSTRAP=1 CARGO_PRIMARY_PACKAGE=1 RUST_LOG_STYLE=always RUST_LOG=trace LD_LIBRARY_PATH=/home/fcb/dev/rusty-links/tests/rust/build/x86_64-unknown-linux-gnu/stage1/lib/rustlib/x86_64-unknown-linux-gnu/lib ../../target/debug/cargo-rusty-links --color-log  --print-mir --print-rl-graph --filter-with-file "compiler/rustc/src/main.rs" > mir
```

# Notes

Copy is better than move. (This phrase assume that the code is written in a way that only the type that needs the copy operator will have it.)

| Operator Kind  |
|----------------|
| move           |
| copy           |
| const          |

We can trait the operator `kind` as a multiplier on the `kind` of the place it is annotating.

- `move` a place which is a `Clone` -> bad
- `move` a place which is an `Adt` -> good
- `move` a place which is a `Const` -> top



## 2024-11-26

It the type is not `Copy` we will always have a `move` operator in the MIR.

```rust
#[derive(Clone)]
pub struct T {
    _value: i32,
}

impl T {
    fn test_method(self) {
        let _ = self;
    }
}
```

```rust
bb3: {
    _6 = T { _value: const 10_i32 };
    _9 = &_6;
    _8 = <T as std::clone::Clone>::clone(move _9) -> [return: bb4, unwind continue];
}

bb4: {
    _7 = test_move(move _8) -> [return: bb5, unwind continue];
}

bb5: {
    _10 = T::test_method(move _6) -> [return: bb6, unwind continue];
}
```

## 2024-11-26

There are another cases when we transfer the ownership of `self` to the method call.
In this case we have a `copy` operator used directly.


```rust
bb7: {
    _16 = U { _value: const 10_i32 };
    _17 = test_for_u(copy _16) -> [return: bb8, unwind continue];
}

bb8: {
    _18 = U::test_method(copy _16) -> [return: bb9, unwind continue];
}
```


## 2024-11-25

If `self` does not implement `Copy` we will always have a `move` operator in the MIR in the method call.
Note that in the case of ownership transfer we will have a `move` preceded by a `copy` operator.

```rust
trait Trait {
    fn test_self_ref_mut(&mut self);
    fn test_self_ref(&self);
    fn test_self(self);
}

impl Trait for U {
    fn test_self_ref_mut(&mut self) {
        let _ = self;
    }

    fn test_self_ref(&self) {
        let _ = self;
    }

    fn test_self(self) {
        let _ = self;
    }
}
```

```rust
bb8: {
    _18 = U { _value: const 10_i32 };
    _20 = &mut _18;
    _19 = <U as Trait>::test_self_ref_mut(move _20) -> [return: bb9, unwind continue];
}

bb9: {
    _22 = &_18;
    _21 = <U as Trait>::test_self_ref(move _22) -> [return: bb10, unwind continue];
}

bb10: {
    _24 = copy _18;
    _23 = <U as Trait>::test_self(move _24) -> [return: bb11, unwind continue];
}
```

## 2024-11-25

Example in which we prefer to work with the optmiized version of the MIR.
We have the `copy` keyword in the optimized version of the MIR.

Unoptimized:
```rust
bb2: {
    StorageDead(_3);
    StorageDead(_2);
    StorageLive(_5);
    _5 = T { _value: const 10_i32 };
    StorageLive(_6);
    StorageLive(_7);
    StorageLive(_8);
    _8 = &_5;
    _7 = &(*_8);
    _6 = test_borrow(move _7) -> [return: bb3, unwind continue];
}
```

Optimized:
```rust
bb2: {
    _5 = T { _value: const 10_i32 };
    _7 = &_5;
    _6 = test_borrow(copy _7) -> [return: bb3, unwind continue];
}
```


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

