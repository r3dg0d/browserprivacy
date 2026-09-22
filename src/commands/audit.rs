use crate::audit::engine::audit_all;
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use anyhow::Result;

pub fn run(out: &OutputOpts, cfg: &AppConfig, dry_run: bool) -> Result<()> {
    if dry_run {
        let profiles = crate::audit::engine::list_profiles(cfg);
        out.emit_or_human(&profiles, || {
            format!(
                "dry-run: would audit {} profile(s) (read-only)\n",
                profiles.len()
            )
        })?;
        return Ok(());
    }
    let audits = audit_all(cfg)?;
    out.print_verbose(&format!("audited {} profiles", audits.len()));
    out.emit_or_human(&audits, || format_human(&audits))?;
    Ok(())
}

pub fn format_human(audits: &[crate::profile::types::ProfileAudit]) -> String {
    let mut s = String::new();
    if audits.is_empty() {
        s.push_str("No browser profiles found.\n");
        return s;
    }
    for a in audits {
        s.push_str(&format!(
            "=== {} / {} ({:?}) ===\n{}\n",
            a.profile.browser,
            a.profile.name,
            a.profile.family,
            a.profile.path.display()
        ));
        if let Some(w) = &a.webrtc {
            s.push_str(&format!("  WebRTC: {} = {} ({})\n", w.key, w.value, w.source));
        } else {
            s.push_str("  WebRTC: (not found)\n");
        }
        if let Some(d) = &a.doh {
            s.push_str(&format!("  DoH: {} = {} ({})\n", d.key, d.value, d.source));
        } else {
            s.push_str("  DoH: (not found)\n");
        }
        if let Some(p) = &a.proxy {
            s.push_str(&format!("  Proxy: {} = {} ({})\n", p.key, p.value, p.source));
        }
        if let Some(c) = &a.cookie_policy {
            s.push_str(&format!(
                "  Cookies: {} = {} ({})\n",
                c.key, c.value, c.source
            ));
        }
        if let Some(t) = &a.third_party_cookies {
            s.push_str(&format!(
                "  3P cookies: {} = {} ({})\n",
                t.key, t.value, t.source
            ));
        }
        if let Some(n) = a.local_storage_entries {
            s.push_str(&format!("  Local storage entries (count only): {n}\n"));
        }
        if let Some(n) = a.service_worker_regs {
            s.push_str(&format!("  Service worker-related entries: {n}\n"));
        }
        if !a.permissions.is_empty() {
            s.push_str("  Permissions:\n");
            for p in a.permissions.iter().take(20) {
                s.push_str(&format!(
                    "    - {} @ {}: {}\n",
                    p.permission, p.site, p.state
                ));
            }
        }
        s.push_str(&format!("  Extensions: {}\n", a.extensions.len()));
        for e in a.extensions.iter().take(30) {
            let name = e.name.as_deref().unwrap_or("?");
            let ver = e.version.as_deref().unwrap_or("?");
            let en = e
                .enabled
                .map(|b| if b { "on" } else { "off" })
                .unwrap_or("?");
            s.push_str(&format!("    - {name} ({}) v{ver} [{en}]\n", e.id));
        }
        if !a.suggestions.is_empty() {
            s.push_str("  Suggestions:\n");
            for sug in &a.suggestions {
                s.push_str(&format!("    * {sug}\n"));
            }
        }
        for n in &a.notes {
            s.push_str(&format!("  note: {n}\n"));
        }
        s.push('\n');
    }
    s
}
