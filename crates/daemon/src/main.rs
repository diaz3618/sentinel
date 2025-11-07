use anyhow::Result;
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use tracing::{info, warn, error, Level};

use sentinel_core::{config::Config, mem, policy::{self, PressureState}, reserve};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("sentinel starting");

    let cfg_path = PathBuf::from("/etc/memsentinel.toml");
    let mut cfg = Config::load_from(&cfg_path).unwrap_or_default();
    info!(?cfg_path, "loaded config or default");

    if !reserve::is_held() {
        reserve::hold(cfg.reserve_mb);
        info!(size_mb = cfg.reserve_mb, "reserve held");
    }

    let term = Arc::new(AtomicBool::new(false));
    let hup = Arc::new(AtomicBool::new(false));
    let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP])?;
    {
        let term2 = term.clone();
        let hup2 = hup.clone();
        std::thread::spawn(move || {
            for sig in &mut signals {
                match sig {
                    SIGTERM | SIGINT => term2.store(true, Ordering::SeqCst),
                    SIGHUP => hup2.store(true, Ordering::SeqCst),
                    _ => {}
                }
            }
        });
    }

    loop {
        if term.load(Ordering::SeqCst) {
            info!("terminating");
            break;
        }
        if hup.load(Ordering::SeqCst) {
            match Config::load_from(&cfg_path) {
                Ok(newc) => {
                    cfg = newc;
                    info!("reloaded config");
                }
                Err(e) => {
                    warn!(error = %e, "failed to reload config; keeping previous");
                }
            }
            hup.store(false, Ordering::SeqCst);
        }

        let m = match mem::sample() {
            Ok(s) => s,
            Err(e) => { error!(error=%e, "meminfo read error"); std::thread::sleep(Duration::from_secs(1)); continue; }
        };
        let state = policy::classify(m.avail_pct, cfg.soft_threshold_pct, cfg.hard_threshold_pct);

        match state {
            PressureState::Healthy => {
                if !reserve::is_held() && m.avail_pct > (cfg.soft_threshold_pct as f64 + 5.0) {
                    reserve::hold(cfg.reserve_mb);
                    info!("reserve re-held");
                }
            }
            PressureState::Soft => {
                if reserve::is_held() {
                    reserve::release();
                    warn!("soft: released reserve");
                }
            }
            PressureState::Hard => {
                if reserve::is_held() {
                    reserve::release();
                    warn!("hard: released reserve");
                }
            }
        }

        std::thread::sleep(Duration::from_secs(cfg.scan_interval_sec));
    }

    Ok(())
}
