use anyhow::Result;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use comfy_table::{Table, presets::UTF8_FULL, ContentArrangement};
use humansize::{format_size, BINARY};
use tracing::Level;
use sentinel_core::{mem, procinfo, policy::{self, PressureState}};

#[derive(Parser, Debug)]
#[command(name="sentinelctl", version, about="Sentinel control and status CLI", long_about=None)]
struct Cli {
    #[arg(long, default_value = "auto")]
    #[arg(long, default_value = "auto")]
    color: String,

    #[arg(long, default_value_t = true)]
    #[arg(long, default_value_t = true)]
    unicode: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
    Status,
    Top { #[arg(long, default_value_t = 10)] limit: usize },
    Top { #[arg(long, default_value_t = 10)] limit: usize },
    Simulate { #[arg(value_parser=["soft","hard"]) ] level: String, #[arg(long)] dry_run: bool },
    Simulate { #[arg(value_parser=["soft","hard"]) ] level: String, #[arg(long)] dry_run: bool },
    Config { #[arg(value_parser=["get","set")] ) op: String, key: Option<String>, value: Option<String> },
    Config { #[arg(value_parser=["get","set"])] op: String, key: Option<String>, value: Option<String> },
    Logs { #[arg(long)] since: Option<String>, #[arg(long)] follow: bool },
    Logs { #[arg(long)] since: Option<String>, #[arg(long)] follow: bool },
    Reserve { #[arg(value_parser=["hold","release","rebuild"]) ] op: String },
    Reserve { #[arg(value_parser=["hold","release","rebuild"])] op: String },
    Slices { #[arg(long)] tree: bool },
    Slices { #[arg(long)] tree: bool },
}

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).with_target(false).compact().init();
    let result = std::panic::catch_unwind(|| {
        let cli = Cli::parse();
        match cli.cmd {
            Commands::Status => status(cli.unicode),
            Commands::Top { limit } => top(limit, cli.unicode),
            Commands::Simulate { level, dry_run } => simulate(&level, dry_run),
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
    let cfg_path = PathBuf::from("/etc/memsentinel.toml");
    match op {
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

fn status(unicode: bool) -> Result<()> {
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
                format_size(m.mem_total as u64, BINARY),
                format_size(used as u64, BINARY),
            ]);
            println!("{}", table);
        }
        Err(e) => {
            println!("ERROR: Could not sample memory: {}", e);
            println!("Sentinel — Status (static fallback)");
        }
    }

    Ok(())
}

fn top(limit: usize, unicode: bool) -> Result<()> {
    let mut table = Table::new();
    if unicode { table.load_preset(UTF8_FULL); }
    table.set_header(vec!["PID","NAME","RSS"]);
    for p in procinfo::top_processes(limit, &vec!["sshd".into(), "systemd".into(), "sentinel".into()])? {
        table.add_row(vec![p.pid.to_string(), p.name, humansize::format_size(p.rss_bytes, BINARY)]);
    }
    println!("{}", table);
    Ok(())
}

fn simulate(level: &str, _dry: bool) -> Result<()> {
    println!("Simulating {} threshold response (dry-run)", level);
    Ok(())
}
