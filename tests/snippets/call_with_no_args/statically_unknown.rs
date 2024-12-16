fn outline(f: impl Fn()) {
    f()
}
fn main() {
    outline(|| {
        10;
    });
}