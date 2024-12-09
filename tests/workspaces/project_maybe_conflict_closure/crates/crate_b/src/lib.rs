const TEST: fn(u8) = |t| { let _ = t; };

pub fn add(left: u64, right: u64) -> u64 {
    let y = &TEST;
    y(10);
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
