#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_hold_release() {
        use crate::reserve;
        reserve::hold(1);
        assert!(reserve::is_held());
        reserve::release();
        assert!(!reserve::is_held());
    }
}
