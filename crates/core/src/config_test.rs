#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_config_parse() {
        let toml = r#"
            reserve_mb = 256
            soft_threshold_pct = 10
            hard_threshold_pct = 3
            mode = "kill"
            scan_interval_sec = 1
            exclude_names = ["sshd", "systemd"]
            max_actions_per_min = 2
        "#;
        let path = Path::new("/tmp/test_memsentinel.toml");
        let mut file = File::create(&path).unwrap();
        file.write_all(toml.as_bytes()).unwrap();
        let cfg = crate::config::Config::load_from(path).unwrap();
        assert_eq!(cfg.reserve_mb, 256);
        assert_eq!(cfg.soft_threshold_pct, 10);
        assert_eq!(cfg.hard_threshold_pct, 3);
        assert_eq!(cfg.mode, "kill");
        assert_eq!(cfg.scan_interval_sec, 1);
        assert_eq!(cfg.exclude_names, vec!["sshd", "systemd"]);
        assert_eq!(cfg.max_actions_per_min, 2);
    }
}
