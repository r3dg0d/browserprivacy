use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BrowserFamily {
    #[default]
    Firefox,
    Chromium,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileRef {
    pub browser: String,
    pub family: BrowserFamily,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingValue {
    pub key: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionInfo {
    pub site: String,
    pub permission: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileAudit {
    pub profile: ProfileRef,
    pub webrtc: Option<SettingValue>,
    pub doh: Option<SettingValue>,
    pub proxy: Option<SettingValue>,
    pub cookie_policy: Option<SettingValue>,
    pub third_party_cookies: Option<SettingValue>,
    pub local_storage_entries: Option<u64>,
    pub service_worker_regs: Option<u64>,
    pub permissions: Vec<PermissionInfo>,
    pub extensions: Vec<ExtensionInfo>,
    pub notes: Vec<String>,
    pub suggestions: Vec<String>,
}
