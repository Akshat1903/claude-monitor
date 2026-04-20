export interface UsageBlock {
  utilization: number;
  resets_at: string | null;
}

export interface UsageResponse {
  five_hour: UsageBlock | null;
  seven_day: UsageBlock | null;
  seven_day_opus: UsageBlock | null;
  seven_day_sonnet: UsageBlock | null;
  seven_day_omelette: UsageBlock | null;
  extra_usage: unknown;
}

export interface ProfileAccount {
  full_name: string | null;
  display_name: string | null;
  email: string | null;
  has_claude_max: boolean | null;
  has_claude_pro: boolean | null;
}

export interface ProfileOrganization {
  name: string | null;
  organization_type: string | null;
  rate_limit_tier: string | null;
  subscription_status: string | null;
}

export interface ProfileResponse {
  account: ProfileAccount;
  organization: ProfileOrganization;
}

export interface StatsSnapshot {
  today_tokens: number;
  week_tokens: number;
  favorite_model: string | null;
}

export interface AppSnapshot {
  usage: UsageResponse | null;
  profile: ProfileResponse | null;
  stats: StatsSnapshot | null;
  fetched_at: string | null;
  error: string | null;
  rate_limited_until: string | null;
}

export type PaceZone = "chill" | "on-track" | "hot";
