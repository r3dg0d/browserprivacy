use crate::audit::engine::audit_all;
use crate::commands::audit::format_human;
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct FullReport {
    generated_at: String,
    disclaimer: String,
    profile_count: usize,
    audits: Vec<crate::profile::types::ProfileAudit>,
    summary_suggestions: Vec<String>,
}

pub fn run(out: &OutputOpts, cfg: &AppConfig, dry_run: bool) -> Result<()> {
    if dry_run {
        out.emit_or_human(&serde_json::json!({"dry_run": true}), || {
            "dry-run: would generate full privacy report (read-only)\n".into()
        })?;
        return Ok(());
    }
    let audits = audit_all(cfg)?;
    let mut summary = Vec::new();
    for a in &audits {
        summary.extend(a.suggestions.iter().cloned());
    }
    summary.sort();
    summary.dedup();
    let report = FullReport {
        generated_at: Utc::now().to_rfc3339(),
        disclaimer: "Informational privacy audit only. Does not extract passwords, cookies, or tokens."
            .into(),
        profile_count: audits.len(),
        audits: audits.clone(),
        summary_suggestions: summary,
    };
    out.emit_or_human(&report, || {
        let mut s = format!(
            "browserprivacy report @ {}\n{}\n\n",
            report.generated_at, report.disclaimer
        );
        s.push_str(&format_human(&report.audits));
        s.push_str("\n=== Summary suggestions ===\n");
        for sug in &report.summary_suggestions {
            s.push_str(&format!("* {sug}\n"));
        }
        s
    })?;
    Ok(())
}
