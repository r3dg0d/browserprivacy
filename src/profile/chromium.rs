use super::types::*;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn audit_chromium(profile: &ProfileRef) -> Result<ProfileAudit> {
    let mut audit = ProfileAudit {
        profile: profile.clone(),
        notes: vec![
            "Read-only audit. Login Data / Cookies DB values are never read.".into(),
        ],
        ..Default::default()
    };

    let prefs_path = profile.path.join("Preferences");
    let prefs: serde_json::Value = match fs::read_to_string(&prefs_path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or(serde_json::json!({})),
        Err(_) => {
            audit.notes.push("Preferences file missing/unreadable".into());
            serde_json::json!({})
        }
    };

    audit.webrtc = chromium_webrtc(&prefs);
    audit.doh = chromium_doh(&prefs);
    audit.proxy = chromium_proxy(&prefs);
    audit.cookie_policy = json_setting(
        &prefs,
        "profile.default_content_setting_values.cookies",
        "Preferences",
    );
    audit.third_party_cookies = chromium_third_party_cookies(&prefs);

    // Local Storage: count LevelDB directories under Local Storage/leveldb — count only, no values
    let ls = profile.path.join("Local Storage/leveldb");
    if ls.is_dir() {
        let n = fs::read_dir(&ls).map(|rd| rd.count()).unwrap_or(0) as u64;
        audit.local_storage_entries = Some(n);
        audit.notes.push(
            "local_storage_entries counts LevelDB files (not secret values)".into(),
        );
    }

    // Service workers
    let sw = profile.path.join("Service Worker/Database");
    if sw.is_dir() {
        let n = fs::read_dir(&sw).map(|rd| rd.count()).unwrap_or(0) as u64;
        audit.service_worker_regs = Some(n);
    } else {
        audit.service_worker_regs = Some(0);
    }

    audit.permissions = chromium_permissions(&prefs);
    audit.extensions = read_chromium_extensions(&profile.path, &prefs);
    audit.suggestions = chromium_suggestions(&audit);

    // Explicitly refuse dangerous paths
    for forbidden in ["Login Data", "Cookies", "Web Data"] {
        if profile.path.join(forbidden).exists() {
            audit.notes.push(format!(
                "found '{forbidden}' (not opened — credential hygiene)"
            ));
        }
    }

    Ok(audit)
}

fn json_path<'a>(v: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut cur = v;
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur)
}

fn json_setting(prefs: &serde_json::Value, path: &str, source: &str) -> Option<SettingValue> {
    json_path(prefs, path).map(|v| SettingValue {
        key: path.into(),
        value: match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        },
        source: source.into(),
    })
}

fn chromium_webrtc(prefs: &serde_json::Value) -> Option<SettingValue> {
    // webRTCIPHandlingPolicy under profile or webrtc
    json_setting(prefs, "webrtc.ip_handling_policy", "Preferences")
        .or_else(|| json_setting(prefs, "profile.webrtc_ip_handling_policy", "Preferences"))
        .or_else(|| {
            // content settings
            json_setting(
                prefs,
                "profile.default_content_setting_values.media_stream",
                "Preferences",
            )
        })
}

fn chromium_doh(prefs: &serde_json::Value) -> Option<SettingValue> {
    json_setting(prefs, "dns_over_https.mode", "Preferences")
        .or_else(|| json_setting(prefs, "dns_over_https.templates", "Preferences"))
}

fn chromium_proxy(prefs: &serde_json::Value) -> Option<SettingValue> {
    if let Some(mode) = json_path(prefs, "proxy.mode") {
        return Some(SettingValue {
            key: "proxy.mode".into(),
            value: mode.to_string(),
            source: "Preferences".into(),
        });
    }
    None
}

fn chromium_third_party_cookies(prefs: &serde_json::Value) -> Option<SettingValue> {
    json_setting(
        prefs,
        "profile.cookie_controls_mode",
        "Preferences",
    )
    .or_else(|| {
        json_setting(
            prefs,
            "profile.block_third_party_cookies",
            "Preferences",
        )
    })
}

fn chromium_permissions(prefs: &serde_json::Value) -> Vec<PermissionInfo> {
    let mut out = Vec::new();
    let keys = [
        ("profile.default_content_setting_values.media_stream_camera", "camera"),
        ("profile.default_content_setting_values.media_stream_mic", "microphone"),
        ("profile.default_content_setting_values.geolocation", "location"),
        ("profile.default_content_setting_values.notifications", "notifications"),
    ];
    for (path, perm) in keys {
        if let Some(v) = json_path(prefs, path) {
            out.push(PermissionInfo {
                site: "* (default)".into(),
                permission: perm.into(),
                state: v.to_string(),
            });
        }
    }
    // exceptions
    if let Some(exceptions) = json_path(prefs, "profile.content_settings.exceptions") {
        for (perm_name, sites) in [
            ("media_stream_camera", "camera"),
            ("media_stream_mic", "microphone"),
            ("geolocation", "location"),
            ("notifications", "notifications"),
        ] {
            if let Some(obj) = exceptions.get(perm_name).and_then(|x| x.as_object()) {
                for (site, meta) in obj.iter().take(50) {
                    // only setting dict keys, not cookie values
                    let state = meta
                        .get("setting")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| meta.to_string());
                    out.push(PermissionInfo {
                        site: site.clone(),
                        permission: sites.into(),
                        state,
                    });
                }
            }
        }
    }
    out
}

fn read_chromium_extensions(profile: &Path, prefs: &serde_json::Value) -> Vec<ExtensionInfo> {
    let mut out = Vec::new();
    if let Some(settings) = json_path(prefs, "extensions.settings").and_then(|x| x.as_object()) {
        for (id, meta) in settings {
            // Skip component extensions optionally — include with flag
            let enabled = meta.get("state").and_then(|s| s.as_u64()).map(|s| s == 1);
            let name = meta
                .get("manifest")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(str::to_string);
            let version = meta
                .get("manifest")
                .and_then(|m| m.get("version"))
                .and_then(|n| n.as_str())
                .map(str::to_string);
            out.push(ExtensionInfo {
                id: id.clone(),
                name,
                version,
                enabled,
            });
        }
    }
    // Also list Extensions/ directory names as fallback
    let ext_dir = profile.join("Extensions");
    if out.is_empty() && ext_dir.is_dir() {
        if let Ok(rd) = fs::read_dir(ext_dir) {
            for ent in rd.flatten() {
                if ent.path().is_dir() {
                    out.push(ExtensionInfo {
                        id: ent.file_name().to_string_lossy().into(),
                        name: None,
                        version: None,
                        enabled: None,
                    });
                }
            }
        }
    }
    out
}

fn chromium_suggestions(audit: &ProfileAudit) -> Vec<String> {
    let mut s = Vec::new();
    match &audit.webrtc {
        Some(w) if w.value.contains("default") || w.value == "0" => {
            s.push(
                "Tighten WebRTC IP handling (disable_non_proxied_udp / force proxy) in chrome://flags or policy"
                    .into(),
            );
        }
        None => s.push(
            "WebRTC policy not found in Preferences — review chrome://settings/security / flags"
                .into(),
        ),
        _ => {}
    }
    if let Some(d) = &audit.doh {
        if d.value.contains("off") {
            s.push("DNS-over-HTTPS is off — consider enabling secure DNS".into());
        }
    } else {
        s.push("DoH setting not found — check Privacy and security → Security → Use secure DNS".into());
    }
    if let Some(t) = &audit.third_party_cookies {
        if t.value == "false" || t.value == "0" {
            s.push("Third-party cookies appear allowed — block them for stronger tracking resistance".into());
        }
    }
    s.push("Review site permissions for camera/mic/location/notifications.".into());
    s.push("Audit extensions; prefer open-source minimal-permission add-ons.".into());
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn audit_minimal_prefs() {
        let dir = TempDir::new().unwrap();
        let prefs = r#"{
            "dns_over_https": {"mode": "off"},
            "profile": {
                "default_content_setting_values": {
                    "cookies": 1,
                    "notifications": 2
                },
                "block_third_party_cookies": true
            },
            "extensions": {"settings": {}}
        }"#;
        fs::write(dir.path().join("Preferences"), prefs).unwrap();
        let pref = ProfileRef {
            browser: "Chromium".into(),
            family: BrowserFamily::Chromium,
            name: "Default".into(),
            path: dir.path().to_path_buf(),
        };
        let audit = audit_chromium(&pref).unwrap();
        assert!(audit.doh.is_some());
        assert!(!audit.suggestions.is_empty());
    }
}
