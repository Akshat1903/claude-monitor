# Claude Widget

A macOS menu bar app that tracks Claude AI usage limits in real time, built to prevent blowing through the 5-hour and weekly quotas during heavy Claude Code sessions.

Menu-bar only. No desktop widget (was scoped out due to free-Personal-Team App Group cross-sandbox limitations — the menu bar UX is the primary surface anyway).

## Status

**v0.2 live** — single menu bar app, signed, runs from Xcode build.

### What works
- Menu bar icon with live 5-hour %
- Dropdown showing:
  - Account name + plan tier (e.g. "Akshat Gupta · Claude Max 5x")
  - Headline stats tile: Today / Week tokens + favorite model (from local `~/.claude/projects/*.jsonl`)
  - 5-hour + weekly gauges with pace zones (chill/on-track/hot) and countdown to reset
  - Claude Design weekly gauge (from `seven_day_omelette`; hidden when API returns null)
- Manual refresh button + auto-refresh every 5 min
- macOS notifications at 50/80/95% thresholds (re-arm on window reset)
- Keychain-based auth via Security framework (no helper needed)
- Persists: `usage.json` (last cached response), `samples.json` (rolling 6h of % samples) in `~/Library/Application Support/ClaudeWidget/`

### Dev info
- Team ID: `A6UG9Q92CB` (Personal, free Apple Developer account)
- Build output: `~/Library/Developer/Xcode/DerivedData/ClaudeWidget-*/Build/Products/Debug/ClaudeWidgetApp.app`
- macOS 14+ (uses `MenuBarExtra`)

## Architecture

Single app target, unsandboxed. Three layers:

```
┌─────────────────────────────┐
│  MenuBarView (SwiftUI)      │  UI
└────────────┬────────────────┘
             │
┌────────────▼────────────────┐
│  UsageViewModel (@MainActor)│  state + auto-refresh loop
└────────────┬────────────────┘
             │
┌────────────▼────────────────┐
│  APIClient + KeychainReader │  service layer
│  + SharedStore              │
└─────────────────────────────┘
```

### Data flow

1. `KeychainReader.readClaudeCodeToken()` — uses `SecItemCopyMatching` on `kSecAttrService = "Claude Code-credentials"` → parses `claudeAiOauth.accessToken` from the JSON value
2. `APIClient.fetchUsage(token:)` — `GET https://api.anthropic.com/api/oauth/usage` with `Authorization: Bearer`, `anthropic-beta: oauth-2025-04-20`
3. `SharedStore.writeUsage(_)` — persists to `~/Library/Application Support/ClaudeWidget/usage.json`
4. `UsageViewModel.refresh()` wires it together; invoked on launch, on 5-min timer, and via refresh button

## Tech stack

- Swift 5.9 / SwiftUI `MenuBarExtra` (macOS 14+)
- XcodeGen generates `.xcodeproj` from `project.yml`
- No third-party Swift packages

## Repo layout

```
claude-widget/
├── CLAUDE.md                           ← this file
├── project.yml                         ← XcodeGen spec
├── .gitignore
├── Shared/
│   ├── Models/UsageResponse.swift      ← API response model
│   └── Services/
│       ├── APIClient.swift             ← Anthropic OAuth usage endpoint
│       ├── KeychainReader.swift        ← SecItemCopyMatching for Claude Code token
│       ├── SharedStore.swift           ← usage.json persistence
│       └── UsageFormatting.swift       ← pace zones, duration formatter
└── App/
    ├── ClaudeWidgetApp.swift           ← @main / MenuBarExtra
    ├── MenuBarView.swift               ← dropdown UI
    ├── UsageViewModel.swift            ← @ObservableObject, auto-refresh
    └── Info.plist                      ← LSUIElement = true
```

## Roadmap

### Shipped
1. ✅ Setup — Xcode, XcodeGen, signing
2. ✅ API client + Keychain pipeline
3. ✅ Menu bar app with gauges + auto-refresh
4. ✅ Simplified to menu-bar-only (dropped widget target)
5. ❌ **Opus vs Sonnet split** — built and reverted. The `seven_day_opus` / `seven_day_sonnet` fields are per-model **sub-cap** meters (plan-specific throttles), not per-model *usage* meters. On Pro they're almost always null; showing them was misleading. Only `five_hour` + `seven_day` are meaningful gauges.
6. ✅ **Smart alerts** — macOS notifications at 50/80/95% for 5-hour + weekly; re-arm on window reset; tracked via `UserDefaults` keys `alerts.<scope>.lastReset` and `alerts.<scope>.fired`
7. ❌ **Burn-rate ETA** — built then removed. Signal proved noisy in practice (false "will hit cap" alarms early in a window; "gathering…" state dominated the UI). `SampleStore` + `BurnRateCalculator` deleted to keep the app focused on *current state* rather than speculative projection.
8. ✅ **Account + plan header** — hit `/api/oauth/profile`, render `displayName · Claude Max 5x` above the gauges; flag non-`active` subscription status with an orange pill
9. ✅ **Headline stats tile** — `StatsService` walks `~/.claude/projects/**/*.jsonl`, sums `input+output+cache_creation` tokens from `assistant` events in the last 7 days; dropdown shows a 3-tile row: `Today` / `Week` / `Favorite model`
10. ✅ **Wake-aware refresh** — subscribes to `NSWorkspace.didWakeNotification` to restart the auto-refresh loop and force-refetch after lid close/open; also refreshes on dropdown open if data is >60s stale
11. ✅ **Claude Design (omelette) gauge** — `UsageResponse.sevenDayOmelette` exposes the Claude Design weekly quota (API calls it by the codename `seven_day_omelette`); rendered as a third gauge when non-null; included in smart-alert thresholds

8. ❌ **History sparkline** — built then removed. With short sample spans and usage values that change slowly, a flat line 28pt tall wasn't a useful signal. `SampleStore` still collects data (burn-rate ETA depends on it); only the UI was dropped.
9. ✅ **Burn-rate reset-awareness** — each `UsageSample` now carries its `fiveHourResetsAt` / `sevenDayResetsAt`; `recomputeBurn` filters to samples matching the current window. Also: `safeBufferSeconds` (15 min) prevents "ETA lands right at reset" false alarms.

### Next up (pick any)
- ⏳ **Launch at login** — `SMAppService` registration
- ⏳ **Per-project breakdown** — parse `~/.claude/projects/*/session.json`
- ⏳ **Polish** — app icon, about dialog, proper install to `/Applications`

## Key references

- **Anthropic OAuth endpoints** (all undocumented, internal to Claude Code/Claude.ai):
  - `GET /api/oauth/usage` — 5-hour + weekly utilization + per-model sub-cap pools. Named fields: `seven_day_opus`, `seven_day_sonnet`, `seven_day_omelette` (Claude Design). Codename-only fields: `iguana_necktie`, `omelette_promotional`, `seven_day_cowork`, `seven_day_oauth_apps` — all null for this user.
  - `GET /api/oauth/profile` — account (name, email, `has_claude_max`), organization (`organization_type`, `rate_limit_tier`, `subscription_status`), application info
  - `GET /api/oauth/account` — 35 KB verbose dump (org settings, model config, capabilities). Too much to be useful; not used by the app.
  - Headers for all: `Authorization: Bearer <token>`, `anthropic-beta: oauth-2025-04-20`, `User-Agent: claude-code/<ver>`
- **Keychain**: `kSecClass = kSecClassGenericPassword`, `kSecAttrService = "Claude Code-credentials"`. Value is JSON blob; extract `claudeAiOauth.accessToken`

## Working conventions

- Source of truth for architecture lives here. Update when decisions change.
- Prefer small, focused commits.
- No comments in code unless a non-obvious *why* is needed.
