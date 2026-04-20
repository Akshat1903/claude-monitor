# claude-monitor

A native macOS menu bar app for tracking Claude AI usage limits in real time. Built to surface the 5-hour, weekly, and Claude Design (`omelette`) quotas so you can see where you stand before you blow through them mid-session.

> **Status:** Personal project. Works end-to-end; not distributed via App Store or Homebrew (yet).

---

## Features

- **Live gauges** for 5-hour window + weekly quota, plus Claude Design quota when available
- **Pace badge** (Chill / On track / Hot) — compares your consumption against linear pace through the window
- **Account + plan header** — pulled from `/api/oauth/profile` (e.g. `Claude Max 5x`)
- **Headline stats tile** — Today / Week token counts + favorite model, computed from your local Claude Code transcripts (`~/.claude/projects/*.jsonl`)
- **Smart notifications** at 50 / 80 / 95% thresholds, re-armed on each window reset (no spam)
- **Auto-refresh** every 5 min; force-refresh on system wake and on dropdown open
- **No separate login** — reads the OAuth token Claude Code already stores in the macOS Keychain

## How it works

Under the hood:

1. Reads your Claude Code OAuth token from the macOS Keychain via the Security framework (`kSecAttrService = "Claude Code-credentials"`)
2. Calls `GET https://api.anthropic.com/api/oauth/usage` with `Authorization: Bearer <token>` and `anthropic-beta: oauth-2025-04-20`
3. Persists the last fetched response to `~/Library/Application Support/ClaudeWidget/usage.json`
4. Walks `~/.claude/projects/**/*.jsonl` to aggregate per-event token counts for the headline stats tile

> **Note:** `/api/oauth/usage` and `/api/oauth/profile` are **not publicly documented** by Anthropic. They're the same endpoints that power Claude Code's `/status` and `/stats` commands. The response shape can change without warning. This repo treats them as a best-effort integration.

## Requirements

- macOS 14 (Sonoma) or newer — uses `MenuBarExtra`
- A Claude Pro, Max, or Team subscription (the Free plan doesn't expose the usage endpoint)
- Claude Code installed and authenticated (`claude /login`) so the OAuth token is in your Keychain
- Xcode 16+ and XcodeGen to build from source

## Build & run

```bash
brew install xcodegen

git clone git@github.com:Akshat1903/claude-monitor.git
cd claude-monitor

# Edit project.yml and replace DEVELOPMENT_TEAM with your own Apple Developer Team ID.
# (Find yours: Xcode > Settings > Accounts > your Apple ID > Personal Team.)

# You'll also need to pick your own bundle ID prefix — the default `dev.fox.claudewidget`
# is tied to the original author's cert. Change it to e.g. `dev.<yourname>.claudemonitor`
# in project.yml (the `bundleIdPrefix` line and the three `PRODUCT_BUNDLE_IDENTIFIER`
# entries).

xcodegen generate
xcodebuild -project ClaudeWidget.xcodeproj \
           -scheme ClaudeWidgetApp \
           -configuration Debug \
           -destination 'platform=macOS' \
           build

# Launch the build output (replace the UUID path with whatever xcodebuild printed):
open ~/Library/Developer/Xcode/DerivedData/ClaudeWidget-*/Build/Products/Debug/ClaudeWidgetApp.app
```

On first launch macOS will prompt you to:

1. Allow Keychain access — accept so the app can read the Claude Code credential
2. Allow notifications — accept to receive 50/80/95% threshold alerts

## First run quick checks

```bash
# Confirm the Claude Code token is in your Keychain
security find-generic-password -s "Claude Code-credentials" -w | head -c 30
# Expected: {"claudeAiOauth":...

# Confirm the app wrote its cached state
ls -la ~/Library/Application\ Support/ClaudeWidget/
# Expected: usage.json
```

## Repo layout

```
claude-monitor/
├── README.md
├── CLAUDE.md                           architectural notes, roadmap, decision log
├── project.yml                         XcodeGen spec
├── .gitignore
├── App/
│   ├── ClaudeWidgetApp.swift           @main / MenuBarExtra
│   ├── MenuBarView.swift               dropdown UI
│   ├── UsageViewModel.swift            @ObservableObject, auto-refresh, wake observer
│   └── Info.plist                      LSUIElement = true (menu bar only, no Dock icon)
└── Shared/
    ├── Models/
    │   ├── UsageResponse.swift         /api/oauth/usage response shape
    │   └── ProfileResponse.swift       /api/oauth/profile response shape
    └── Services/
        ├── APIClient.swift             usage + profile endpoints
        ├── KeychainReader.swift        SecItemCopyMatching for Claude Code token
        ├── SharedStore.swift           usage.json persistence
        ├── StatsService.swift          aggregates ~/.claude/projects/*.jsonl
        ├── NotificationService.swift   threshold alerts via UNUserNotificationCenter
        └── UsageFormatting.swift       pace zones, duration + token formatters
```

## Codename map

The `/api/oauth/usage` response includes sub-cap pools under obfuscated codenames. Known so far:

| API field | Meaning |
|---|---|
| `five_hour` | Overall 5-hour rate-limit window |
| `seven_day` | Overall weekly window |
| `seven_day_opus` | Weekly Opus sub-cap (null for most Pro/Max users) |
| `seven_day_sonnet` | Weekly Sonnet sub-cap |
| `seven_day_omelette` | **Claude Design** weekly quota |
| `seven_day_cowork`, `seven_day_oauth_apps`, `iguana_necktie`, `omelette_promotional` | Unknown (all null in testing) |

## Credits

Inspired by [TokenEater](https://github.com/AThevon/TokenEater) — the original reverse-engineering work on the Anthropic OAuth usage endpoint is theirs.

## License

See [LICENSE](LICENSE).
