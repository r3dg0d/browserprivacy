use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub include_flatpak: bool,
    pub extra_profile_roots: Vec<String>,
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self {
                include_flatpak: true,
                ..Default::default()
            });
        };
        if !path.exists() {
            return Ok(Self {
                include_flatpak: true,
                ..Default::default()
            });
        }
        let raw = fs::read_to_string(path)?;
        if raw.trim_start().starts_with('{') {
            return Ok(serde_json::from_str(&raw)?);
        }
        let mut cfg = Self {
            include_flatpak: true,
            ..Default::default()
        };
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim().trim_matches('"').trim_matches('\'');
            match k {
                "include_flatpak" => cfg.include_flatpak = v == "true" || v == "1",
                "extra_profile_roots" => cfg.extra_profile_roots.push(v.to_string()),
                _ => {}
            }
        }
        Ok(cfg)
    }
}
