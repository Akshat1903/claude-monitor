use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageBlock {
    pub utilization: f64,
    #[serde(default)]
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    #[serde(default)]
    pub monthly_limit: Option<f64>,
    #[serde(default)]
    pub used_credits: Option<f64>,
    #[serde(default)]
    pub utilization: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageResponse {
    #[serde(default)]
    pub five_hour: Option<UsageBlock>,
    #[serde(default)]
    pub seven_day: Option<UsageBlock>,
    #[serde(default)]
    pub seven_day_opus: Option<UsageBlock>,
    #[serde(default)]
    pub seven_day_sonnet: Option<UsageBlock>,
    #[serde(default)]
    pub seven_day_omelette: Option<UsageBlock>,
    #[serde(default)]
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileAccount {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub has_claude_max: Option<bool>,
    #[serde(default)]
    pub has_claude_pro: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileOrganization {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub organization_type: Option<String>,
    #[serde(default)]
    pub rate_limit_tier: Option<String>,
    #[serde(default)]
    pub subscription_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileResponse {
    pub account: ProfileAccount,
    pub organization: ProfileOrganization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSnapshot {
    pub today_tokens: u64,
    pub week_tokens: u64,
    pub favorite_model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSnapshot {
    pub usage: Option<UsageResponse>,
    pub profile: Option<ProfileResponse>,
    pub stats: Option<StatsSnapshot>,
    pub fetched_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub rate_limited_until: Option<DateTime<Utc>>,
}
