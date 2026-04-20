import type { PaceZone, ProfileResponse, UsageBlock } from "./types";

export function shortDuration(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  if (h >= 24) {
    const d = Math.floor(h / 24);
    const rh = h % 24;
    return rh === 0 ? `${d}d` : `${d}d ${rh}h`;
  }
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function shortTokens(n: number): string {
  if (n >= 1_000_000) {
    const v = n / 1_000_000;
    return v >= 10 ? `${Math.round(v)}m` : `${v.toFixed(1)}m`;
  }
  if (n >= 1_000) {
    const v = n / 1_000;
    return v >= 10 ? `${Math.round(v)}k` : `${v.toFixed(1)}k`;
  }
  return `${n}`;
}

export function secondsUntil(iso: string | null): number | null {
  if (!iso) return null;
  const t = new Date(iso).getTime();
  if (isNaN(t)) return null;
  return (t - Date.now()) / 1000;
}

export function fiveHourPace(block: UsageBlock | null): PaceZone | null {
  if (!block || !block.resets_at) return null;
  const resetMs = new Date(block.resets_at).getTime();
  if (isNaN(resetMs)) return null;
  const windowStart = resetMs - 5 * 3600 * 1000;
  const elapsed = Date.now() - windowStart;
  const expectedPct = (elapsed / (5 * 3600 * 1000)) * 100;
  const delta = block.utilization - expectedPct;
  if (delta < -10) return "chill";
  if (delta > 10) return "hot";
  return "on-track";
}

export function planLabel(p: ProfileResponse): string {
  if (p.account.has_claude_max) {
    const tier = p.organization.rate_limit_tier || "";
    if (tier.includes("20x")) return "Claude Max 20x";
    if (tier.includes("5x")) return "Claude Max 5x";
    return "Claude Max";
  }
  if (p.account.has_claude_pro) return "Claude Pro";
  return "Claude";
}

export function shortName(p: ProfileResponse): string {
  return (
    p.account.display_name || p.account.full_name || p.account.email || "—"
  );
}

export function relativeFromNow(iso: string | null): string {
  if (!iso) return "";
  const t = new Date(iso).getTime();
  if (isNaN(t)) return "";
  const diffSec = Math.round((Date.now() - t) / 1000);
  if (diffSec < 10) return "just now";
  if (diffSec < 60) return `${diffSec}s ago`;
  const min = Math.round(diffSec / 60);
  if (min < 60) return `${min} min ago`;
  const h = Math.round(min / 60);
  return `${h} h ago`;
}
