use anyhow::Result;
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration, fs, process};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use tracing::{info, warn, error, Level};
use clap::Parser;

use sentinel_core::{config::Config, mem, policy::{self, PressureState}, reserve};

const PID_FILE: &str = "/var/run/sentinel.pid";

#[derive(Parser, Debug)]
#[command(name="sentinel", version, about="Sentinel memory guard daemon", long_about=None)]
struct Args {
    #[arg(long, help = "Run in background (daemonize)")]
    silent: bool,

    #[arg(long, help = "Stop running daemon")]
    stop: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.stop {
        return stop_daemon();
    }

    if args.silent {
        daemonize()?;
    }

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

    // Clean up PID file on exit
    let _ = fs::remove_file(PID_FILE);

    Ok(())
}

fn daemonize() -> Result<()> {
    // Check if already running
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is still running
            if process::Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                eprintln!("Sentinel is already running (PID {})", pid);
                process::exit(1);
            }
        }
    }

    // Fork and detach
    unsafe {
        let pid = libc::fork();
        if pid < 0 {
            return Err(anyhow::anyhow!("Fork failed"));
        }
        if pid > 0 {
            // Parent process - write PID and exit
            fs::write(PID_FILE, format!("{}\n", pid))?;
            println!("Sentinel started in background (PID {})", pid);
            process::exit(0);
        }
        
        // Child process continues
        // Create new session
        if libc::setsid() < 0 {
            return Err(anyhow::anyhow!("setsid failed"));
        }

        // Change to root directory
        std::env::set_current_dir("/")?;

        // Close standard file descriptors
        libc::close(0);
        libc::close(1);
        libc::close(2);

        // Reopen them to /dev/null
        let devnull = std::ffi::CString::new("/dev/null").unwrap();
        libc::open(devnull.as_ptr(), libc::O_RDONLY);
        libc::open(devnull.as_ptr(), libc::O_WRONLY);
        libc::open(devnull.as_ptr(), libc::O_WRONLY);
    }

    Ok(())
}

fn stop_daemon() -> Result<()> {
    let pid_str = fs::read_to_string(PID_FILE)
        .map_err(|_| anyhow::anyhow!("Sentinel is not running (no PID file found)"))?;
    
    let pid = pid_str.trim().parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Invalid PID file"))?;

    // Send SIGTERM
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };
    
    if result == 0 {
        println!("Stopping sentinel (PID {})...", pid);
        
        // Wait up to 10 seconds for process to terminate
        for _ in 0..100 {
            std::thread::sleep(Duration::from_millis(100));
            let check = unsafe { libc::kill(pid, 0) };
            if check != 0 {
                // Process is gone
                let _ = fs::remove_file(PID_FILE);
                println!("Sentinel stopped successfully");
                return Ok(());
            }
        }
        
        // Force kill if still running
        println!("Process did not stop gracefully, forcing...");
        unsafe { libc::kill(pid, libc::SIGKILL) };
        let _ = fs::remove_file(PID_FILE);
        println!("Sentinel stopped (forced)");
    } else {
        let _ = fs::remove_file(PID_FILE);
        return Err(anyhow::anyhow!("Process {} not found (stale PID file removed)", pid));
    }

    Ok(())
}
