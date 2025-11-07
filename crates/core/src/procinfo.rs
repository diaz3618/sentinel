use anyhow::Result;
use procfs::process::all_processes;

#[derive(Debug, Clone)]
pub struct ProcLite {
    pub pid: i32,
    pub name: String,
    pub rss_bytes: u64,
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
