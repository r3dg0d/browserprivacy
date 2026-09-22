use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct XdgPaths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
    pub data_dir: PathBuf,
}

#[allow(dead_code)]
impl XdgPaths {
    pub fn new() -> Result<Self> {
        let dirs = ProjectDirs::from("dev", "r3dg0d", "browserprivacy")
            .context("unable to resolve XDG directories")?;
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
            state_dir: dirs.state_dir().map(|p| p.to_path_buf()).unwrap_or_else(|| dirs.data_dir().join("state")),
            data_dir: dirs.data_dir().to_path_buf(),
        })
    }

    pub fn ensure_all(&self) -> Result<()> {
        for d in [&self.config_dir, &self.cache_dir, &self.state_dir, &self.data_dir] {
            fs::create_dir_all(d)?;
        }
        Ok(())
    }

    pub fn default_config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }
}
