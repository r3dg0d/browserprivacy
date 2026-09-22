use super::types::*;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn audit_firefox(profile: &ProfileRef) -> Result<ProfileAudit> {
    let mut audit = ProfileAudit {
        profile: profile.clone(),
        notes: vec![
            "Read-only audit. Passwords/cookies/token DBs are never opened for values.".into(),
        ],
        ..Default::default()
    };

    let prefs = read_prefs_js(&profile.path.join("prefs.js")).unwrap_or_default();
    let user = read_prefs_js(&profile.path.join("user.js")).unwrap_or_default();
    // user.js overrides prefs.js for our reporting preference
    let get = |key: &str| -> Option<SettingValue> {
        user.get(key)
            .map(|v| SettingValue {
                key: key.into(),
                value: v.clone(),
                source: "user.js".into(),
            })
            .or_else(|| {
                prefs.get(key).map(|v| SettingValue {
                    key: key.into(),
                    value: v.clone(),
                    source: "prefs.js".into(),
                })
            })
    };

    audit.webrtc = get("media.peerconnection.enabled").or_else(|| get("media.peerconnection.ice.default_address_only"));
    audit.doh = get("network.trr.mode").or_else(|| get("network.trr.uri"));
    audit.proxy = get("network.proxy.type");
    audit.cookie_policy = get("network.cookie.cookieBehavior");
    audit.third_party_cookies = get("network.cookie.cookieBehavior");

    // local storage presence counts — count files/dirs under storage/default, not contents
    let storage = profile.path.join("storage/default");
    if storage.is_dir() {
        let count = walkdir_count_dirs(&storage);
        audit.local_storage_entries = Some(count);
    } else {
        audit.notes.push("no storage/default directory".into());
    }

    // service workers
    let sw = profile.path.join("storage/default");
    // Also check serviceworker.txt or similar — count *ls directories
    let sw_count = count_name_contains(&profile.path.join("storage"), "sw");
    audit.service_worker_regs = Some(sw_count);

    // permissions.sqlite — we deliberately do NOT query secrets; try permissions.json if present
    audit.permissions = read_firefox_permissions_json(&profile.path);

    // extensions
    audit.extensions = read_firefox_extensions(&profile.path);

    audit.suggestions = firefox_suggestions(&audit);
    let _ = sw;
    Ok(audit)
}

fn read_prefs_js(path: &Path) -> Result<std::collections::HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    Ok(parse_prefs_js(&content))
}

/// Parse user_pref("key", value); lines.
pub fn parse_prefs_js(content: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let re = Regex::new(r#"user_pref\(\s*"([^"]+)"\s*,\s*(.+?)\s*\);"#).unwrap();
    for cap in re.captures_iter(content) {
        let key = cap[1].to_string();
        let val = cap[2].trim().trim_matches('"').to_string();
        map.insert(key, val);
    }
    map
}

fn walkdir_count_dirs(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.path() != path)
        .count() as u64
}

fn count_name_contains(root: &Path, needle: &str) -> u64 {
    if !root.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .contains(needle)
        })
        .count() as u64
}

fn read_firefox_permissions_json(profile: &Path) -> Vec<PermissionInfo> {
    // Firefox stores site permissions in permissions.sqlite — we skip SQLite credential-adjacent DBs.
    // If a JSON export/extension file exists, read that; otherwise note skip.
    let _ = profile;
    vec![]
}

fn read_firefox_extensions(profile: &Path) -> Vec<ExtensionInfo> {
    let path = profile.join("extensions.json");
    let Ok(content) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else {
        return vec![];
    };
    let mut out = Vec::new();
    if let Some(addons) = v.get("addons").and_then(|a| a.as_array()) {
        for a in addons {
            let id = a
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                continue;
            }
            // Skip themes optionally? include all
            out.push(ExtensionInfo {
                id,
                name: a
                    .get("defaultLocale")
                    .and_then(|d| d.get("name"))
                    .and_then(|x| x.as_str())
                    .map(str::to_string)
                    .or_else(|| {
                        a.get("name")
                            .and_then(|x| x.as_str())
                            .map(str::to_string)
                    }),
                version: a
                    .get("version")
                    .and_then(|x| x.as_str())
                    .map(str::to_string),
                enabled: a.get("active").and_then(|x| x.as_bool()),
            });
        }
    }
    out
}

fn firefox_suggestions(audit: &ProfileAudit) -> Vec<String> {
    let mut s = Vec::new();
    if let Some(w) = &audit.webrtc {
        if w.value == "true" || w.value == "1" {
            s.push(
                "WebRTC appears enabled — consider media.peerconnection.enabled=false or resistFingerprinting for stricter OPSEC"
                    .into(),
            );
        }
    } else {
        s.push("WebRTC pref not found — check about:config media.peerconnection.*".into());
    }
    if let Some(d) = &audit.doh {
        if d.key == "network.trr.mode" && matches!(d.value.as_str(), "0" | "5") {
            s.push("DoH/TRR mode is off or off-by-choice — enable TRR mode 2/3 if you want encrypted DNS".into());
        }
    }
    if let Some(c) = &audit.cookie_policy {
        // 0=accept all, 1=reject foreign, 2=reject all, 4=reject trackers, 5=reject trackers+partition
        if c.value == "0" {
            s.push(
                "cookieBehavior=0 accepts all cookies — prefer 4 or 5 (Total Cookie Protection)"
                    .into(),
            );
        }
    }
    s.push("Review extension inventory; remove unused add-ons.".into());
    s.push("Never audit Login Data / key4.db / cookies.sqlite values with this tool — by design.".into());
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prefs() {
        let content = r#"
user_pref("media.peerconnection.enabled", false);
user_pref("network.trr.mode", 2);
user_pref("network.cookie.cookieBehavior", 5);
"#;
        let m = parse_prefs_js(content);
        assert_eq!(m.get("media.peerconnection.enabled").map(|s| s.as_str()), Some("false"));
        assert_eq!(m.get("network.trr.mode").map(|s| s.as_str()), Some("2"));
    }
}
