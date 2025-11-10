use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::{Table, presets::UTF8_FULL, ContentArrangement};
use humansize::{format_size, BINARY};
use tracing::Level;
use sentinel_core::{mem, procinfo, policy::{self, PressureState}, psi::PSIMetrics};
use std::io::{self, Write};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(name="sentinelctl", version, about="Sentinel control and status CLI", long_about=None)]
struct Cli {
    #[arg(long, default_value = "auto")]
    color: String,

    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    unicode: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        watch: bool,
    },
    Top {
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    Simulate {
        #[arg(value_parser=["soft","hard"])]
        level: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain: bool,
    },
    Config { #[arg(value_parser=["get","set","init"])] op: String, key: Option<String>, value: Option<String> },
    Logs { #[arg(long)] since: Option<String>, #[arg(long)] follow: bool },
    Reserve { #[arg(value_parser=["hold","release","rebuild"])] op: String },
    Slices { #[arg(long)] tree: bool },
}

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).with_target(false).compact().init();
    let result = std::panic::catch_unwind(|| {
        let cli = Cli::parse();
        match cli.cmd {
            Commands::Status { json, watch } => status(cli.unicode, json, watch),
            Commands::Top { limit, json } => top(limit, cli.unicode, json),
            Commands::Simulate { level, dry_run, explain } => simulate(&level, dry_run, explain),
            Commands::Config { op, key, value } => config_cmd(&op, key, value),
            Commands::Logs { since, follow } => logs_cmd(since, follow),
            Commands::Reserve { op } => reserve_cmd(&op),
            Commands::Slices { tree } => slices_cmd(tree),
        }
    });
    match result {
        Ok(Ok(_)) => {},
        Ok(Err(e)) => {
            println!("ERROR: {}", e);
            println!("Sentinel — Status (main fallback)");
        },
        Err(_) => {
            println!("ERROR: panic in CLI main");
            println!("Sentinel — Status (panic fallback)");
        }
    }
}
fn config_cmd(op: &str, key: Option<String>, value: Option<String>) -> Result<()> {
    use sentinel_core::config::Config;
    use std::path::PathBuf;
    let _value = value;
    let cfg_path = if std::path::Path::new("/etc/memsentinel.toml").exists() {
        PathBuf::from("/etc/memsentinel.toml")
    } else {
        PathBuf::from("memsentinel.toml")
    };
    match op {
        "init" => {
            init_config_interactive()?;
        }
        "get" => {
            let cfg = Config::load_from(&cfg_path)?;
            if let Some(k) = key {
                let val = match k.as_str() {
                    "reserve_mb" => cfg.reserve_mb.to_string(),
                    "soft_threshold_pct" => cfg.soft_threshold_pct.to_string(),
                    "hard_threshold_pct" => cfg.hard_threshold_pct.to_string(),
                    "mode" => cfg.mode,
                    "scan_interval_sec" => cfg.scan_interval_sec.to_string(),
                    "exclude_names" => format!("{:?}", cfg.exclude_names),
                    "max_actions_per_min" => cfg.max_actions_per_min.to_string(),
                    _ => "unknown key".to_string(),
                };
                println!("{} = {}", k, val);
            } else {
                println!("{:?}", cfg);
            }
        }
        "set" => {
            println!("Config set not yet implemented");
        }
        _ => println!("Unknown config op: {}", op),
    }
    Ok(())
}

fn logs_cmd(since: Option<String>, follow: bool) -> Result<()> {
    let _since = since;
    let _follow = follow;
    println!("Logs command not yet implemented");
    Ok(())
}

fn reserve_cmd(op: &str) -> Result<()> {
    use sentinel_core::reserve;
    match op {
        "hold" => {
            reserve::hold(512);
            println!("Reserve held");
        }
        "release" => {
            reserve::release();
            println!("Reserve released");
        }
        "rebuild" => {
            reserve::release();
            reserve::hold(512);
            println!("Reserve rebuilt");
        }
        _ => println!("Unknown reserve op: {}", op),
    }
    Ok(())
}

fn slices_cmd(tree: bool) -> Result<()> {
    let _tree = tree;
    println!("Slices command not yet implemented");
    Ok(())
}

fn status(_unicode: bool, json: bool, watch: bool) -> Result<()> {
    loop {
        if json {
            status_json()?;
        } else {
            status_table()?;
        }
        
        if !watch {
            break;
        }
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        if !json {
            print!("\x1B[2J\x1B[1;1H");
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct StatusOutput {
    state: String,
    avail_pct: f64,
    mem_total_bytes: u64,
    mem_available_bytes: u64,
    mem_used_bytes: u64,
    psi_available: bool,
    psi_some_avg10: Option<f64>,
    psi_full_avg10: Option<f64>,
}

fn status_json() -> Result<()> {
    let m = mem::sample()?;
    let state = policy::classify(m.avail_pct, 15, 5);
    let used = m.mem_total.saturating_sub(m.mem_available);
    
    let psi_metrics = PSIMetrics::sample().ok();
    
    let output = StatusOutput {
        state: format!("{:?}", state),
        avail_pct: m.avail_pct,
        mem_total_bytes: m.mem_total * 1024,
        mem_available_bytes: m.mem_available * 1024,
        mem_used_bytes: used * 1024,
        psi_available: psi_metrics.is_some(),
        psi_some_avg10: psi_metrics.as_ref().map(|p| p.some_avg10),
        psi_full_avg10: psi_metrics.as_ref().map(|p| p.full_avg10),
    };
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn status_table() -> Result<()> {
    println!("Sentinel — Status");
    match mem::sample() {
        Ok(m) => {
            let state = policy::classify(m.avail_pct, 15, 5);
            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec!["State", "MemAvailable", "Total", "Used"]);
            let used = m.mem_total.saturating_sub(m.mem_available);
            let colorized = match state {
                PressureState::Healthy => "● Healthy".to_string(),
                PressureState::Soft => "▲ Soft".to_string(),
                PressureState::Hard => "✖ Hard".to_string(),
            };
            table.add_row(vec![
                colorized,
                format!("{:.0}%", m.avail_pct),
                format_size(m.mem_total as u64 * 1024, BINARY),
                format_size(used as u64 * 1024, BINARY),
            ]);
            println!("{}", table);
            
            if let Ok(psi) = PSIMetrics::sample() {
                println!("\nPSI Memory Pressure:");
                println!("  some avg10: {:.2}%  avg60: {:.2}%  avg300: {:.2}%",
                         psi.some_avg10, psi.some_avg60, psi.some_avg300);
                println!("  full avg10: {:.2}%  avg60: {:.2}%  avg300: {:.2}%",
                         psi.full_avg10, psi.full_avg60, psi.full_avg300);
            }
        }
        Err(e) => {
            println!("ERROR: Could not sample memory: {}", e);
            println!("Sentinel — Status (static fallback)");
        }
    }

    Ok(())
}

fn top(limit: usize, unicode: bool, json: bool) -> Result<()> {
    let procs = procinfo::top_processes(limit, &vec!["sshd".into(), "systemd".into(), "sentinel".into()])?;
    
    if json {
        #[derive(Serialize)]
        struct TopOutput {
            pid: i32,
            name: String,
            rss_bytes: u64,
        }
        
        let output: Vec<TopOutput> = procs.iter().map(|p| TopOutput {
            pid: p.pid,
            name: p.name.clone(),
            rss_bytes: p.rss_bytes,
        }).collect();
        
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let mut table = Table::new();
        if unicode { table.load_preset(UTF8_FULL); }
        table.set_header(vec!["PID","NAME","RSS"]);
        for p in procs {
            table.add_row(vec![p.pid.to_string(), p.name, humansize::format_size(p.rss_bytes, BINARY)]);
        }
        println!("{}", table);
    }
    
    Ok(())
}

fn simulate(level: &str, dry_run: bool, explain: bool) -> Result<()> {
    use sentinel_core::config::Config;
    
    println!("Simulating {} threshold response{}", level, if dry_run { " (dry-run)" } else { "" });
    
    if explain {
        let cfg = Config::default();
        let m = mem::sample()?;
        
        println!("\n=== Current Memory State ===");
        println!("Total: {} KB", m.mem_total);
        println!("Available: {} KB ({:.1}%)", m.mem_available, m.avail_pct);
        
        if let Ok(psi) = PSIMetrics::sample() {
            println!("\n=== PSI Metrics ===");
            println!("some avg10: {:.2}%", psi.some_avg10);
            println!("full avg10: {:.2}%", psi.full_avg10);
        }
        
        println!("\n=== Process Badness Scoring ===");
        match procinfo::processes_with_badness(
            &cfg.exclude_names,
            &cfg.protected_units,
            m.mem_total * 1024,
        ) {
            Ok(procs) => {
                println!("{:<8} {:<20} {:<12} {:<10} {:<15} {:<10}",
                         "PID", "NAME", "RSS (MB)", "OOM ADJ", "SLICE", "BADNESS");
                println!("{}", "-".repeat(85));
                
                for proc in procs.iter().take(10) {
                    println!("{:<8} {:<20} {:<12} {:<10} {:<15} {:<10.1}",
                             proc.pid,
                             proc.name,
                             proc.rss_bytes / (1024*1024),
                             proc.oom_score_adj,
                             format!("{:?}", proc.cgroup_slice),
                             proc.badness_score);
                }
                
                if let Some(victim) = procs.first() {
                    println!("\n→ Target selected: PID {} ({})", victim.pid, victim.name);
                    println!("  Badness score: {:.1}", victim.badness_score);
                    println!("  Cgroup: {:?} / {:?}", victim.cgroup_slice, victim.cgroup_unit);
                }
            }
            Err(e) => {
                println!("Error enumerating processes: {}", e);
            }
        }
    }
    
    Ok(())
}

fn init_config_interactive() -> Result<()> {
    use sentinel_core::config::Config;
    use std::fs;
    use std::path::Path;

    println!("🔧 Sentinel Configuration Wizard");
    println!("================================\n");

    let target_path = "/etc/memsentinel.toml";
    
    if Path::new(target_path).exists() {
        print!("Config file already exists at {}. Overwrite? (y/N): ", target_path);
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut cfg = Config::default();

    println!("\n📊 Memory Reserve Configuration");
    cfg.reserve_mb = prompt_u64("Reserve memory (MB)", cfg.reserve_mb)?;
    
    println!("\n⚠️  Threshold Configuration");
    cfg.soft_threshold_pct = prompt_u8("Soft threshold (% available)", cfg.soft_threshold_pct)?;
    cfg.hard_threshold_pct = prompt_u8("Hard threshold (% available)", cfg.hard_threshold_pct)?;
    
    println!("\n🎯 Action Mode");
    println!("  - kill: Terminate processes aggressively");
    println!("  - slow: Pause processes with SIGSTOP");
    println!("  - hybrid: Use both strategies");
    cfg.mode = prompt_string("Mode (kill/slow/hybrid)", &cfg.mode)?;
    
    println!("\n⏱️  Monitoring Configuration");
    cfg.scan_interval_sec = prompt_u64("Scan interval (seconds)", cfg.scan_interval_sec)?;
    cfg.max_actions_per_min = prompt_u32("Max actions per minute", cfg.max_actions_per_min)?;
    
    println!("\n🛡️  Protected Processes");
    println!("Current: {:?}", cfg.exclude_names);
    print!("Add more protected process names (comma-separated, or press Enter to skip): ");
    io::stdout().flush()?;
    let mut exclude_input = String::new();
    io::stdin().read_line(&mut exclude_input)?;
    if !exclude_input.trim().is_empty() {
        for name in exclude_input.trim().split(',') {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() && !cfg.exclude_names.contains(&trimmed) {
                cfg.exclude_names.push(trimmed);
            }
        }
    }

    let toml_content = toml::to_string_pretty(&cfg)?;
    
    println!("\n📝 Generated configuration:");
    println!("---");
    println!("{}", toml_content);
    println!("---");
    
    match fs::write(target_path, &toml_content) {
        Ok(_) => {
            println!("✅ Configuration written to {}", target_path);
        }
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            let local_path = "memsentinel.toml";
            fs::write(local_path, &toml_content)?;
            println!("⚠️  Permission denied for {}.", target_path);
            println!("✅ Configuration written to {} instead.", local_path);
            println!("💡 Run: sudo mv {} {}", local_path, target_path);
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

fn prompt_u64(prompt: &str, default: u64) -> Result<u64> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.parse()?)
    }
}

fn prompt_u8(prompt: &str, default: u8) -> Result<u8> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.parse()?)
    }
}

fn prompt_u32(prompt: &str, default: u32) -> Result<u32> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.parse()?)
    }
}

fn prompt_string(prompt: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}
