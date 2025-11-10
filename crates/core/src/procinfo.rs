use anyhow::Result;
use procfs::process::all_processes;
use crate::cgroups::{CgroupInfo, CgroupSlice};
use std::fs;

#[derive(Debug, Clone)]
pub struct ProcLite {
    pub pid: i32,
    pub name: String,
    pub rss_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ProcWithBadness {
    pub pid: i32,
    pub name: String,
    pub rss_bytes: u64,
    pub oom_score_adj: i32,
    pub cgroup_slice: CgroupSlice,
    pub cgroup_unit: Option<String>,
    pub badness_score: f64,
}

impl ProcWithBadness {
    /* Composite badness score: RSS percentage + OOM adjustment + cgroup priority.
     * Higher score = more likely to be killed. */
    pub fn calculate_badness(&mut self, total_mem: u64) {
        let rss_score = (self.rss_bytes as f64 / total_mem as f64) * 1000.0;
        
        // Negative oom_score_adj means "protect me" - reduce impact
        let oom_score = if self.oom_score_adj >= 0 {
            self.oom_score_adj as f64
        } else {
            self.oom_score_adj as f64 * 0.5
        };
        
        let cgroup_priority = self.cgroup_slice.priority_score() as f64;
        
        self.badness_score = rss_score + oom_score + cgroup_priority;
    }
}

pub fn top_processes(limit: usize, exclude: &[String]) -> Result<Vec<ProcLite>> {
    let mut procs = Vec::new();
    for pr_res in all_processes()? {
        if let Ok(pr) = pr_res {
            if let Ok(statm) = pr.statm() {
                let rss_pages = statm.resident as u64;
                let rss = rss_pages * 4096;
                let name = pr.stat().map(|s| s.comm).unwrap_or_else(|_| String::from("?"));
                if exclude.iter().any(|e| name.contains(e)) {
                    continue;
                }
                procs.push(ProcLite { pid: pr.pid(), name, rss_bytes: rss });
            }
        }
    }
    procs.sort_by_key(|p| std::cmp::Reverse(p.rss_bytes));
    procs.truncate(limit);
    Ok(procs)
}

pub fn processes_with_badness(
    exclude: &[String],
    protected_units: &[String],
    total_mem: u64,
) -> Result<Vec<ProcWithBadness>> {
    let mut procs = Vec::new();
    
    for pr_res in all_processes()? {
        if let Ok(pr) = pr_res {
            let pid = pr.pid();
            
            if let Ok(statm) = pr.statm() {
                let rss_pages = statm.resident as u64;
                let rss = rss_pages * 4096;
                
                // Skip tiny processes (< 10 MB RSS)
                if rss < 10 * 1024 * 1024 {
                    continue;
                }
                
                let name = pr.stat().map(|s| s.comm).unwrap_or_else(|_| String::from("?"));
                
                if exclude.iter().any(|e| name.contains(e)) {
                    continue;
                }
                
                let oom_score_adj = read_oom_score_adj(pid).unwrap_or(0);
                
                let cgroup_info = CgroupInfo::for_pid(pid as u32).unwrap_or_else(|_| CgroupInfo {
                    slice: CgroupSlice::Unknown,
                    unit_name: None,
                    raw_path: String::new(),
                });
                
                if cgroup_info.is_protected(protected_units) {
                    continue;
                }
                
                let mut proc = ProcWithBadness {
                    pid,
                    name,
                    rss_bytes: rss,
                    oom_score_adj,
                    cgroup_slice: cgroup_info.slice,
                    cgroup_unit: cgroup_info.unit_name,
                    badness_score: 0.0,
                };
                
                proc.calculate_badness(total_mem);
                procs.push(proc);
            }
        }
    }
    
    procs.sort_by(|a, b| b.badness_score.partial_cmp(&a.badness_score).unwrap());
    Ok(procs)
}

fn read_oom_score_adj(pid: i32) -> Result<i32> {
    let path = format!("/proc/{}/oom_score_adj", pid);
    let content = fs::read_to_string(path)?;
    Ok(content.trim().parse()?)
}

