use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliUi {
    pub color: Option<String>,
    pub unicode: Option<String>,
    pub table_max_width: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub reserve_mb: u64,
    pub soft_threshold_pct: u8,
    pub hard_threshold_pct: u8,
    pub mode: String,
    pub scan_interval_sec: u64,
    pub exclude_names: Vec<String>,
    pub max_actions_per_min: u32,
    pub cli: Option<CliUi>,
    
    #[serde(default = "default_psi_enabled")]
    pub psi_enabled: bool,
    #[serde(default = "default_psi_soft_pct")]
    pub psi_soft_pct: f64,
    #[serde(default = "default_psi_hard_pct")]
    pub psi_hard_pct: f64,
    #[serde(default)]
    pub protected_units: Vec<String>,
}

fn default_psi_enabled() -> bool { true }
fn default_psi_soft_pct() -> f64 { 10.0 }
fn default_psi_hard_pct() -> f64 { 30.0 }

impl Default for Config {
    fn default() -> Self {
        Self {
            reserve_mb: 512,
            soft_threshold_pct: 15,
            hard_threshold_pct: 5,
            mode: "hybrid".into(),
            scan_interval_sec: 2,
            exclude_names: vec!["sshd".into(), "systemd".into(), "sentinel".into()],
            max_actions_per_min: 4,
            cli: Some(CliUi { color: Some("auto".into()), unicode: Some("auto".into()), table_max_width: Some(120) }),
            psi_enabled: true,
            psi_soft_pct: 10.0,
            psi_hard_pct: 30.0,
            protected_units: vec!["sshd.service".into(), "sentinel.service".into(), "ssh.service".into()],
        }
    }
}

impl Config {
    pub fn load_from(path: &Path) -> Result<Self> {
        let s = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;
        let cfg: Self = toml::from_str(&s).with_context(|| "parsing TOML config")?;
        Ok(cfg)
    }
}
