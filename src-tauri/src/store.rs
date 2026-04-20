use crate::types::{ProfileResponse, UsageResponse};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUsage {
    pub usage: UsageResponse,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProfile {
    pub profile: ProfileResponse,
    pub fetched_at: DateTime<Utc>,
}

fn support_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home directory")?;
    let dir = home.join("Library/Application Support/ClaudeMonitor");
    fs::create_dir_all(&dir).ok();
    Ok(dir)
}

fn usage_path() -> Result<PathBuf> {
    Ok(support_dir()?.join("usage.json"))
}

fn profile_path() -> Result<PathBuf> {
    Ok(support_dir()?.join("profile.json"))
}

pub fn read_usage() -> Option<CachedUsage> {
    let data = fs::read(usage_path().ok()?).ok()?;
    serde_json::from_slice(&data).ok()
}

pub fn write_usage(cached: &CachedUsage) -> Result<()> {
    let data = serde_json::to_vec_pretty(cached)?;
    fs::write(usage_path()?, data)?;
    Ok(())
}

pub fn read_profile() -> Option<CachedProfile> {
    let data = fs::read(profile_path().ok()?).ok()?;
    serde_json::from_slice(&data).ok()
}

pub fn write_profile(cached: &CachedProfile) -> Result<()> {
    let data = serde_json::to_vec_pretty(cached)?;
    fs::write(profile_path()?, data)?;
    Ok(())
}
