use crate::types::StatsSnapshot;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn compute() -> StatsSnapshot {
    let root = match projects_root() {
        Some(p) if p.exists() => p,
        _ => {
            return StatsSnapshot {
                today_tokens: 0,
                week_tokens: 0,
                favorite_model: None,
            }
        }
    };

    let now = Utc::now();
    let start_of_today = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight")
        .and_utc();
    let week_ago = now - Duration::days(7);

    let mut today = 0u64;
    let mut week = 0u64;
    let mut by_model: HashMap<String, u64> = HashMap::new();

    for entry in WalkDir::new(&root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(f) = File::open(entry.path()) else {
            continue;
        };
        let reader = BufReader::new(f);
        for line in reader.lines().map_while(Result::ok) {
            let Ok(obj) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if obj["type"].as_str() != Some("assistant") {
                continue;
            }
            let Some(ts_str) = obj["timestamp"].as_str() else {
                continue;
            };
            let Ok(ts) = ts_str.parse::<DateTime<Utc>>() else {
                continue;
            };
            if ts < week_ago {
                continue;
            }
            let msg = &obj["message"];
            let u = &msg["usage"];
            let input = u["input_tokens"].as_u64().unwrap_or(0);
            let output = u["output_tokens"].as_u64().unwrap_or(0);
            let cache_create = u["cache_creation_input_tokens"].as_u64().unwrap_or(0);
            let total = input + output + cache_create;
            week += total;
            if ts >= start_of_today {
                today += total;
            }
            if let Some(model) = msg["model"].as_str() {
                *by_model.entry(model.to_string()).or_insert(0) += total;
            }
        }
    }

    let favorite = by_model
        .into_iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| display_model_name(&k));

    StatsSnapshot {
        today_tokens: today,
        week_tokens: week,
        favorite_model: favorite,
    }
}

fn projects_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude/projects"))
}

fn display_model_name(raw: &str) -> String {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() < 3 || parts[0] != "claude" {
        return raw.to_string();
    }
    let family = capitalize(parts[1]);
    let major = parts[2];
    if let Some(minor) = parts.get(3) {
        if minor.chars().all(|c| c.is_ascii_digit()) && minor.len() <= 2 {
            return format!("{} {}.{}", family, major, minor);
        }
    }
    format!("{} {}", family, major)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
