fn outline<F: FnOnce()>(f: F) -> F { f }
fn main() {
    let f = outline(|| {
        10;
    });
    f()
}

