use crate::audit::engine::list_profiles;
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ListResult {
    count: usize,
    profiles: Vec<crate::profile::types::ProfileRef>,
}

pub fn run(out: &OutputOpts, cfg: &AppConfig) -> Result<()> {
    let profiles = list_profiles(cfg);
    let result = ListResult {
        count: profiles.len(),
        profiles: profiles.clone(),
    };
    out.emit_or_human(&result, || {
        let mut s = format!("Found {} profile(s):\n", result.count);
        for p in &profiles {
            s.push_str(&format!(
                "  [{:?}] {} / {} — {}\n",
                p.family,
                p.browser,
                p.name,
                p.path.display()
            ));
        }
        if profiles.is_empty() {
            s.push_str("  (none — install Firefox/Chromium or check Flatpak paths)\n");
        }
        s
    })?;
    Ok(())
}
