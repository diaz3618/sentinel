mod tests {
    use crate::reserve;

    #[test]
    fn test_reserve_hold_release() {
        reserve::hold(1);
        assert!(reserve::is_held());
        reserve::release();
        assert!(!reserve::is_held());
    }
}
