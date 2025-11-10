use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CgroupSlice {
    System,
    User,
    Machine,
    Init,
    Unknown,
}

impl CgroupSlice {
    /* Priority for process selection - higher score = more likely to be killed */
    pub fn priority_score(&self) -> i32 {
        match self {
            CgroupSlice::User => 100,      // user sessions
            CgroupSlice::Machine => 50,     // VMs/containers
            CgroupSlice::Unknown => 25,     
            CgroupSlice::System => 10,      // system services
            CgroupSlice::Init => 0,         // critical init processes
        }
    }
}

#[derive(Debug, Clone)]
pub struct CgroupInfo {
    pub slice: CgroupSlice,
    pub unit_name: Option<String>,
    pub raw_path: String,
}

impl CgroupInfo {
    pub fn for_pid(pid: u32) -> Result<Self> {
        let cgroup_path = format!("/proc/{}/cgroup", pid);
        let content = fs::read_to_string(&cgroup_path)
            .with_context(|| format!("reading {}", cgroup_path))?;
        
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self> {
        for line in content.lines() {
            if line.contains("0::") {
                let raw_path = line.strip_prefix("0::").unwrap_or(line).to_string();
                let (slice, unit_name) = Self::classify_path(&raw_path);
                
                return Ok(CgroupInfo {
                    slice,
                    unit_name,
                    raw_path,
                });
            }
        }
        
        Ok(CgroupInfo {
            slice: CgroupSlice::Unknown,
            unit_name: None,
            raw_path: String::new(),
        })
    }

    fn classify_path(path: &str) -> (CgroupSlice, Option<String>) {
        if path.contains("/user.slice/") {
            let unit = Self::extract_unit_name(path);
            (CgroupSlice::User, unit)
        } else if path.contains("/system.slice/") {
            let unit = Self::extract_unit_name(path);
            (CgroupSlice::System, unit)
        } else if path.contains("/machine.slice/") {
            let unit = Self::extract_unit_name(path);
            (CgroupSlice::Machine, unit)
        } else if path.contains("/init.scope") {
            (CgroupSlice::Init, Some("init.scope".to_string()))
        } else {
            (CgroupSlice::Unknown, None)
        }
    }

    fn extract_unit_name(path: &str) -> Option<String> {
        path.split('/')
            .filter(|s| s.ends_with(".service") || s.ends_with(".scope") || s.ends_with(".slice"))
            .last()
            .map(|s| s.to_string())
    }

    pub fn is_protected(&self, protected_units: &[String]) -> bool {
        if let Some(ref unit) = self.unit_name {
            protected_units.iter().any(|protected| unit == protected)
        } else {
            false
        }
    }
}

pub fn get_slice_stats() -> Result<HashMap<CgroupSlice, usize>> {
    let mut stats = HashMap::new();
    
    let proc_dir = Path::new("/proc");
    for entry in fs::read_dir(proc_dir)? {
        let entry = entry?;
        if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
            if let Ok(cgroup_info) = CgroupInfo::for_pid(pid) {
                *stats.entry(cgroup_info.slice).or_insert(0) += 1;
            }
        }
    }
    
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cgroup_user_slice() {
        let sample = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-firefox.scope";
        let info = CgroupInfo::parse(sample).unwrap();
        
        assert_eq!(info.slice, CgroupSlice::User);
        assert!(info.unit_name.is_some());
        assert!(info.raw_path.contains("user.slice"));
    }

    #[test]
    fn test_parse_cgroup_system_slice() {
        let sample = "0::/system.slice/sshd.service";
        let info = CgroupInfo::parse(sample).unwrap();
        
        assert_eq!(info.slice, CgroupSlice::System);
        assert_eq!(info.unit_name, Some("sshd.service".to_string()));
    }

    #[test]
    fn test_is_protected() {
        let sample = "0::/system.slice/sshd.service";
        let info = CgroupInfo::parse(sample).unwrap();
        
        let protected = vec!["sshd.service".to_string(), "sentinel.service".to_string()];
        assert!(info.is_protected(&protected));
        
        let not_protected = vec!["sentinel.service".to_string()];
        assert!(!info.is_protected(&not_protected));
    }

    #[test]
    fn test_slice_priority() {
        assert!(CgroupSlice::User.priority_score() > CgroupSlice::System.priority_score());
        assert!(CgroupSlice::System.priority_score() > CgroupSlice::Init.priority_score());
    }
}
