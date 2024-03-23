use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;

/// The 1Password-Fuse configuration object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The mountpoint for the filesystem
    pub mountpoint: PathBuf,

    /// The user ID that will own files in the filesystem
    #[serde(default = "default_uid_gid")]
    pub uid: u32,

    /// The group ID that will own files in the filesystem
    #[serde(default = "default_uid_gid")]
    pub gid: u32,

    /// The default mode for directories in the filesystem
    #[serde(default = "default_dir_mode")]
    pub dir_mode: u16,

    /// The default mode for files in the filesystem
    #[serde(default = "default_file_mode")]
    pub file_mode: u16,

    /// The duration to cache 1Password data for
    #[serde(default = "default_cache_duration", with = "humantime_serde")]
    pub cache_duration: Duration,

    /// The 1Password accounts to use, and their configuration
    #[serde(default)]
    pub accounts: HashMap<String, Account>,

    /// 1Password-related configuration
    #[serde(default, rename = "onepassword")]
    pub op: OnePassword,
}

fn default_uid_gid() -> u32 {
    0
}

fn default_dir_mode() -> u16 {
    0o500
}

fn default_file_mode() -> u16 {
    0o400
}

fn default_cache_duration() -> Duration {
    Duration::from_secs(5)
}

impl Config {
    /// Read a configuration file from the given path
    pub fn read(path: &Path) -> Result<Config> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

/// 1Password account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    /// The 1Password account ID
    pub id: String,

    /// The vaults to mount from this account
    #[serde(default)]
    pub vaults: HashMap<String, Vault>,
}

/// 1Password vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vault {
    /// The 1Password vault ID
    pub id: String,
}

/// 1Password configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnePassword {
    /// The 1Password command to run
    #[serde(default = "default_op_cmd")]
    pub cmd: String,
}

fn default_op_cmd() -> String {
    "op".to_string()
}

impl Default for OnePassword {
    /// The default 1Password-Fuse configuration
    fn default() -> Self {
        toml::from_str("").expect("empty object should be valid")
    }
}
