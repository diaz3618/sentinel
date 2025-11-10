use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/* Pressure Stall Information - requires kernel 4.20+ */

#[derive(Debug, Clone, Copy, Default)]
pub struct PSIMetrics {
    pub some_avg10: f64,
    pub some_avg60: f64,
    pub some_avg300: f64,
    pub some_total: u64,
    
    pub full_avg10: f64,
    pub full_avg60: f64,
    pub full_avg300: f64,
    pub full_total: u64,
}

impl PSIMetrics {
    pub fn is_available() -> bool {
        Path::new("/proc/pressure/memory").exists()
    }

    pub fn sample() -> Result<Self> {
        let content = fs::read_to_string("/proc/pressure/memory")
            .context("failed to read /proc/pressure/memory")?;
        
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self> {
        let mut metrics = PSIMetrics::default();
        
        for line in content.lines() {
            if line.starts_with("some ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts.iter().skip(1) {
                    if let Some((key, value)) = part.split_once('=') {
                        match key {
                            "avg10" => metrics.some_avg10 = value.parse()?,
                            "avg60" => metrics.some_avg60 = value.parse()?,
                            "avg300" => metrics.some_avg300 = value.parse()?,
                            "total" => metrics.some_total = value.parse()?,
                            _ => {}
                        }
                    }
                }
            } else if line.starts_with("full ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts.iter().skip(1) {
                    if let Some((key, value)) = part.split_once('=') {
                        match key {
                            "avg10" => metrics.full_avg10 = value.parse()?,
                            "avg60" => metrics.full_avg60 = value.parse()?,
                            "avg300" => metrics.full_avg300 = value.parse()?,
                            "total" => metrics.full_total = value.parse()?,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_psi() {
        let sample = "some avg10=0.50 avg60=1.20 avg300=3.45 total=123456
full avg10=0.10 avg60=0.30 avg300=0.80 total=45678
";
        let metrics = PSIMetrics::parse(sample).unwrap();
        
        assert_eq!(metrics.some_avg10, 0.50);
        assert_eq!(metrics.some_avg60, 1.20);
        assert_eq!(metrics.some_avg300, 3.45);
        assert_eq!(metrics.some_total, 123456);
        
        assert_eq!(metrics.full_avg10, 0.10);
        assert_eq!(metrics.full_avg60, 0.30);
        assert_eq!(metrics.full_avg300, 0.80);
        assert_eq!(metrics.full_total, 45678);
    }

    #[test]
    fn test_parse_psi_zero_values() {
        let sample = "some avg10=0.00 avg60=0.00 avg300=0.00 total=0
full avg10=0.00 avg60=0.00 avg300=0.00 total=0
";
        let metrics = PSIMetrics::parse(sample).unwrap();
        
        assert_eq!(metrics.some_avg10, 0.0);
        assert_eq!(metrics.full_avg10, 0.0);
    }
}
