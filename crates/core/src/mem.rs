use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct MemSample {
    pub mem_total: u64,
    pub mem_available: u64,
    pub avail_pct: f64,
    pub total_kb: u64,
}

pub fn sample() -> Result<MemSample> {
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    let mut mem_free = 0u64;
    for line in reader.lines() {
        let l = line?;
        if l.starts_with("MemTotal:") {
            mem_total = l.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        } else if l.starts_with("MemAvailable:") {
            mem_available = l.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        } else if l.starts_with("MemFree:") {
            mem_free = l.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
    // MemAvailable was added in kernel 3.14 - fall back to MemFree if missing
    let avail = if mem_available > 0 { mem_available } else { mem_free };
    let pct = if mem_total > 0 { (avail as f64 / mem_total as f64) * 100.0 } else { 0.0 };
    Ok(MemSample { mem_total, mem_available: avail, avail_pct: pct, total_kb: mem_total })
}
