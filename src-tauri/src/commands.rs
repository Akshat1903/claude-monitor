use crate::api::{ApiClient, ApiError};
use crate::keychain;
use crate::notify;
use crate::stats;
use crate::store::{self, CachedProfile, CachedUsage};
use crate::types::{AppSnapshot, ProfileResponse, StatsSnapshot, UsageResponse};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime, State};
use tokio::sync::Mutex;

pub struct AppState {
    pub inner: Mutex<AppStateInner>,
    pub api: ApiClient,
}

#[derive(Default)]
pub struct AppStateInner {
    pub usage: Option<UsageResponse>,
    pub profile: Option<ProfileResponse>,
    pub stats: Option<StatsSnapshot>,
    pub fetched_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub rate_limited_until: Option<DateTime<Utc>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let s = Arc::new(Self {
            inner: Mutex::new(AppStateInner::default()),
            api: ApiClient::new(),
        });
        // Hydrate from cache on startup.
        {
            let mut g = s.inner.blocking_lock();
            if let Some(c) = store::read_usage() {
                g.usage = Some(c.usage);
                g.fetched_at = Some(c.fetched_at);
            }
            if let Some(c) = store::read_profile() {
                g.profile = Some(c.profile);
            }
        }
        s
    }
}

pub async fn refresh_impl<R: Runtime>(state: Arc<AppState>, app: AppHandle<R>) {
    {
        let g = state.inner.lock().await;
        if let Some(until) = g.rate_limited_until {
            if until > Utc::now() {
                return;
            }
        }
    }

    let token = match keychain::read_token() {
        Ok(t) => t,
        Err(e) => {
            let mut g = state.inner.lock().await;
            g.error = Some(format!("Could not read Claude token: {}", e));
            return;
        }
    };

    match state.api.fetch_usage(&token).await {
        Ok(usage) => {
            let now = Utc::now();
            let _ = store::write_usage(&CachedUsage {
                usage: usage.clone(),
                fetched_at: now,
            });
            notify::check_and_notify(&app, "five_hour", "5-hour window", &usage.five_hour);
            notify::check_and_notify(&app, "seven_day", "Weekly", &usage.seven_day);
            notify::check_and_notify(
                &app,
                "claude_design",
                "Claude Design weekly",
                &usage.seven_day_omelette,
            );

            let mut g = state.inner.lock().await;
            g.usage = Some(usage);
            g.fetched_at = Some(now);
            g.error = None;
            g.rate_limited_until = None;
        }
        Err(ApiError::RateLimited { retry_after }) => {
            let wait = retry_after.unwrap_or(300.0);
            let until = Utc::now() + chrono::Duration::seconds(wait as i64);
            let mut g = state.inner.lock().await;
            g.rate_limited_until = Some(until);
            let local = until.with_timezone(&chrono::Local);
            g.error = Some(format!("Rate limited. Retrying at {}", local.format("%H:%M")));
        }
        Err(e) => {
            let mut g = state.inner.lock().await;
            g.error = Some(e.to_string());
        }
    }

    if let Ok(p) = state.api.fetch_profile(&token).await {
        let _ = store::write_profile(&CachedProfile {
            profile: p.clone(),
            fetched_at: Utc::now(),
        });
        let mut g = state.inner.lock().await;
        g.profile = Some(p);
    }

    if let Ok(snap) = tokio::task::spawn_blocking(stats::compute).await {
        let mut g = state.inner.lock().await;
        g.stats = Some(snap);
    }

    let _ = app.emit("snapshot-updated", snapshot_inner(&state).await);
}

async fn snapshot_inner(state: &Arc<AppState>) -> AppSnapshot {
    let g = state.inner.lock().await;
    AppSnapshot {
        usage: g.usage.clone(),
        profile: g.profile.clone(),
        stats: g.stats.clone(),
        fetched_at: g.fetched_at,
        error: g.error.clone(),
        rate_limited_until: g.rate_limited_until,
    }
}

#[tauri::command]
pub async fn refresh<R: Runtime>(
    state: State<'_, Arc<AppState>>,
    app: AppHandle<R>,
) -> Result<AppSnapshot, String> {
    refresh_impl(state.inner().clone(), app).await;
    Ok(snapshot_inner(state.inner()).await)
}

#[tauri::command]
pub async fn get_snapshot(
    state: State<'_, Arc<AppState>>,
) -> Result<AppSnapshot, String> {
    Ok(snapshot_inner(state.inner()).await)
}
