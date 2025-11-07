#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        use crate::policy::{classify, PressureState};
        assert_eq!(classify(20.0, 15, 5), PressureState::Healthy);
        assert_eq!(classify(10.0, 15, 5), PressureState::Soft);
        assert_eq!(classify(3.0, 15, 5), PressureState::Hard);
    }
}
