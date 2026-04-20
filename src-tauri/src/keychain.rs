use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::process::Command;

const SERVICE: &str = "Claude Code-credentials";

pub fn read_token() -> Result<String> {
    let output = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", SERVICE, "-w"])
        .output()
        .context("failed to spawn /usr/bin/security")?;

    if !output.status.success() {
        return Err(anyhow!(
            "security exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let raw = String::from_utf8(output.stdout)
        .context("security output was not valid UTF-8")?
        .trim()
        .to_string();

    if raw.is_empty() {
        return Err(anyhow!("empty keychain entry for '{}'", SERVICE));
    }

    let v: Value = serde_json::from_str(&raw)
        .context("keychain entry was not valid JSON")?;

    v["claudeAiOauth"]["accessToken"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("claudeAiOauth.accessToken not found in keychain entry"))
}
