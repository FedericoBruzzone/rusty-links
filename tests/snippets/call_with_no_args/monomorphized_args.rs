fn outline<F: FnOnce() -> R, R>(f: F) -> R { 
    f() 
}

fn main() { 
    outline(|| { 
        10; 
    }); 
}