use crate::profile::types::{BrowserFamily, ProfileRef};
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_profiles(include_flatpak: bool, extra_roots: &[String]) -> Vec<ProfileRef> {
    let home = dirs_home();
    let mut profiles = Vec::new();

    // Firefox (legacy ~/.mozilla and XDG ~/.config/mozilla used by some NixOS setups)
    push_firefox(&mut profiles, &home.join(".mozilla/firefox"), "Firefox");
    push_firefox(
        &mut profiles,
        &home.join(".config/mozilla/firefox"),
        "Firefox (XDG)",
    );
    push_firefox(&mut profiles, &home.join(".librewolf"), "LibreWolf");
    push_firefox(&mut profiles, &home.join(".waterfox"), "Waterfox");

    // Chromium-family native
    for (rel, name) in [
        (".config/chromium", "Chromium"),
        (".config/google-chrome", "Google Chrome"),
        (".config/brave-browser", "Brave"),
        (".config/microsoft-edge", "Microsoft Edge"),
        (".config/vivaldi", "Vivaldi"),
        (".config/opera", "Opera"),
        (".config/net.imput.helium", "Helium"),
        (".config/thorium", "Thorium"),
    ] {
        push_chromium(&mut profiles, &home.join(rel), name);
    }

    if include_flatpak {
        let flatpak = home.join(".var/app");
        if flatpak.is_dir() {
            let flatpak_map = [
                ("org.mozilla.firefox", "Firefox (Flatpak)", true),
                ("org.chromium.Chromium", "Chromium (Flatpak)", false),
                ("com.google.Chrome", "Chrome (Flatpak)", false),
                ("com.brave.Browser", "Brave (Flatpak)", false),
            ];
            for (app_id, label, is_ff) in flatpak_map {
                let base = flatpak.join(app_id);
                if is_ff {
                    push_firefox(
                        &mut profiles,
                        &base.join(".mozilla/firefox"),
                        label,
                    );
                } else {
                    // Chromium flatpak config often under config/<name>
                    let config = base.join("config");
                    if config.is_dir() {
                        if let Ok(entries) = fs::read_dir(&config) {
                            for ent in entries.flatten() {
                                let p = ent.path();
                                if p.is_dir() {
                                    push_chromium(&mut profiles, &p, label);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for root in extra_roots {
        let p = PathBuf::from(shellexpand_home(root));
        if p.join("profiles.ini").exists() {
            push_firefox(&mut profiles, &p, "Firefox (extra)");
        } else {
            push_chromium(&mut profiles, &p, "Chromium (extra)");
        }
    }

    profiles
}

fn dirs_home() -> PathBuf {
    directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn shellexpand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        return format!("{}/{}", dirs_home().display(), rest);
    }
    s.to_string()
}

fn push_firefox(out: &mut Vec<ProfileRef>, root: &Path, browser: &str) {
    if !root.is_dir() {
        return;
    }
    let ini = root.join("profiles.ini");
    if ini.exists() {
        if let Ok(content) = fs::read_to_string(&ini) {
            for path in parse_firefox_profiles_ini(&content, root) {
                out.push(ProfileRef {
                    browser: browser.to_string(),
                    family: BrowserFamily::Firefox,
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("default")
                        .to_string(),
                    path,
                });
            }
            return;
        }
    }
    // Fallback: any subdirectory that looks like a profile
    if let Ok(entries) = fs::read_dir(root) {
        for ent in entries.flatten() {
            let p = ent.path();
            if p.is_dir() && (p.join("prefs.js").exists() || p.join("user.js").exists()) {
                out.push(ProfileRef {
                    browser: browser.to_string(),
                    family: BrowserFamily::Firefox,
                    name: ent.file_name().to_string_lossy().into(),
                    path: p,
                });
            }
        }
    }
}

/// Parse profiles.ini Path= entries.
pub fn parse_firefox_profiles_ini(content: &str, root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut is_relative = true;
    let mut path_val: Option<String> = None;
    let flush = |is_relative: bool, path_val: &mut Option<String>, out: &mut Vec<PathBuf>| {
        if let Some(p) = path_val.take() {
            let full = if is_relative {
                root.join(&p)
            } else {
                PathBuf::from(&p)
            };
            if full.is_dir() {
                out.push(full);
            }
        }
    };
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            flush(is_relative, &mut path_val, &mut out);
            is_relative = true;
            continue;
        }
        if let Some(v) = line.strip_prefix("IsRelative=") {
            is_relative = v.trim() != "0";
        } else if let Some(v) = line.strip_prefix("Path=") {
            path_val = Some(v.trim().to_string());
        }
    }
    flush(is_relative, &mut path_val, &mut out);
    out
}

fn push_chromium(out: &mut Vec<ProfileRef>, root: &Path, browser: &str) {
    if !root.is_dir() {
        return;
    }
    // Default + Profile N
    let candidates: Vec<PathBuf> = {
        let mut c = vec![root.join("Default")];
        if let Ok(entries) = fs::read_dir(root) {
            for ent in entries.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                if name.starts_with("Profile ") {
                    c.push(ent.path());
                }
            }
        }
        c
    };
    for p in candidates {
        if p.is_dir() && (p.join("Preferences").exists() || p.join("prefs.js").exists()) {
            out.push(ProfileRef {
                browser: browser.to_string(),
                family: BrowserFamily::Chromium,
                name: p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Default")
                    .to_string(),
                path: p,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parse_profiles_ini() {
        let dir = TempDir::new().unwrap();
        let prof = dir.path().join("abcd.default");
        fs::create_dir_all(&prof).unwrap();
        let ini = format!(
            "[Profile0]\nName=default\nIsRelative=1\nPath=abcd.default\n"
        );
        let paths = parse_firefox_profiles_ini(&ini, dir.path());
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], prof);
    }
}
