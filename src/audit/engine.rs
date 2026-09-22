use crate::detect::paths::discover_profiles;
use crate::profile::chromium::audit_chromium;
use crate::profile::firefox::audit_firefox;
use crate::profile::types::{BrowserFamily, ProfileAudit, ProfileRef};
use crate::util::config::AppConfig;
use anyhow::Result;

pub fn list_profiles(cfg: &AppConfig) -> Vec<ProfileRef> {
    discover_profiles(cfg.include_flatpak, &cfg.extra_profile_roots)
}

pub fn audit_all(cfg: &AppConfig) -> Result<Vec<ProfileAudit>> {
    let profiles = list_profiles(cfg);
    let mut out = Vec::new();
    for p in profiles {
        match p.family {
            BrowserFamily::Firefox => out.push(audit_firefox(&p)?),
            BrowserFamily::Chromium => out.push(audit_chromium(&p)?),
        }
    }
    Ok(out)
}

#[allow(dead_code)]
pub fn audit_one(profile: &ProfileRef) -> Result<ProfileAudit> {
    match profile.family {
        BrowserFamily::Firefox => audit_firefox(profile),
        BrowserFamily::Chromium => audit_chromium(profile),
    }
}
