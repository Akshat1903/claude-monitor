use crate::types::UsageBlock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

const THRESHOLDS: &[i64] = &[50, 80, 95];

#[derive(Default, Serialize, Deserialize)]
struct ThresholdState {
    #[serde(default)]
    map: HashMap<String, ScopeState>,
}

#[derive(Default, Serialize, Deserialize)]
struct ScopeState {
    #[serde(default)]
    resets_at: Option<DateTime<Utc>>,
    #[serde(default)]
    fired: Vec<i64>,
}

fn state_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join("Library/Application Support/ClaudeMonitor/alerts.json"))
}

fn read_state() -> ThresholdState {
    state_path()
        .and_then(|p| fs::read(p).ok())
        .and_then(|d| serde_json::from_slice(&d).ok())
        .unwrap_or_default()
}

fn write_state(s: &ThresholdState) {
    if let Some(p) = state_path() {
        if let Ok(data) = serde_json::to_vec_pretty(s) {
            let _ = fs::write(p, data);
        }
    }
}

pub fn check_and_notify<R: Runtime>(
    app: &AppHandle<R>,
    scope: &str,
    label: &str,
    block: &Option<UsageBlock>,
) {
    let Some(block) = block else { return };
    let pct = block.utilization as i64;

    let mut state = read_state();
    let entry = state.map.entry(scope.to_string()).or_default();

    if entry.resets_at != block.resets_at {
        entry.resets_at = block.resets_at;
        entry.fired.clear();
    }

    let to_fire: Vec<i64> = THRESHOLDS
        .iter()
        .filter(|t| pct >= **t && !entry.fired.contains(t))
        .copied()
        .collect();

    if to_fire.is_empty() {
        return;
    }

    for t in &to_fire {
        let title = format!("Claude usage at {}%", t);
        let body = format!("{} is {}% used.", label, pct);
        let _ = app.notification().builder().title(title).body(body).show();
    }

    entry.fired.extend(to_fire);
    write_state(&state);
}
