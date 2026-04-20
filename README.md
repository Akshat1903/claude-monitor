# claude-monitor

A native-feel macOS menu bar app for tracking Claude AI usage limits in real time. Built with **Tauri v2 + Svelte 5 + Rust**.

Surfaces the 5-hour, weekly, and Claude Design (`omelette`) quotas from the Claude OAuth API, plus Today / Week token counts from your local Claude Code session transcripts.

> **Status:** Works end-to-end on macOS 13+. Unsigned ad-hoc builds — macOS Gatekeeper will prompt once on first launch.

---

## Features

- **Live gauges** for 5-hour window, weekly quota, and Claude Design when available
- **Pace badge** (Chill / On track / Hot) based on linear-pace comparison
- **Account + plan header** from `/api/oauth/profile`
- **Headline stats** — Today / Week tokens + favorite model, computed from `~/.claude/projects/**/*.jsonl`
- **Smart notifications** at 50 / 80 / 95% thresholds, re-armed on each window reset
- **Rate-limit backoff** — when the API 429s, the app pauses polling until `Retry-After` elapses
- **Menu bar tray icon** — click to open/close the popup; auto-refresh every 5 min

## Architecture

```
┌────────────────────────┐    invoke()     ┌─────────────────────┐
│  Svelte 5 frontend     │ ←────────────→  │  Rust backend       │
│  (SvelteKit SPA)       │                 │  (Tauri v2)         │
│                        │                 │                     │
│  • +page.svelte        │   "refresh"     │  • reqwest → API    │
│  • GaugeRow, Stats,    │   "get_snapshot"│  • walks ~/.claude  │
│    ProfileHeader       │   emits         │  • UNNotification   │
│  • Tailwind CSS        │   "snapshot-    │  • disk cache       │
└────────────────────────┘    updated"     └─────────────────────┘
          │                                          │
          │                                          │
          └─── click tray → show window ─────────────┘
```

## Requirements

- macOS 13+ (Ventura or newer — Tauri v2's minimum for tray icons)
- Claude Code installed and logged in (`claude /login`) with a Pro / Max / Team plan
- For building from source:
  - **Rust** (`rustup`): https://rustup.rs
  - **Node.js 18+** and **npm**

## Install & run

```bash
git clone git@github.com:Akshat1903/claude-monitor.git
cd claude-monitor

# Install deps
npm install

# One-liner to run in dev (hot-reload Svelte, auto-rebuild Rust on changes):
npm run tauri dev

# Or build a release binary:
npm run tauri build
# → src-tauri/target/release/claude-monitor
```

On first launch:
1. **macOS Gatekeeper will refuse to open the unsigned app.**
   Right-click the binary → **Open** → **Open Anyway** in the prompt.
2. **Keychain prompt** — "Claude Monitor wants to access 'Claude Code-credentials' in your Keychain". Click **Always Allow**.
3. **Notification permission** prompt — allow to get 50/80/95% threshold alerts.

Click the tray icon to open the popup. Click it again to close.

## Data sources

- **Anthropic OAuth endpoints** (undocumented, same ones Claude Code's `/status` uses):
  - `GET https://api.anthropic.com/api/oauth/usage` — quotas
  - `GET https://api.anthropic.com/api/oauth/profile` — account + plan
- **macOS Keychain**: `kSecAttrService = "Claude Code-credentials"`, extracts `claudeAiOauth.accessToken`
- **Local session transcripts**: `~/.claude/projects/**/*.jsonl` (used only for the stats tile)

## Codename map

The `/api/oauth/usage` response includes sub-cap pools under obfuscated codenames. Known mappings:

| Field | Meaning |
|---|---|
| `five_hour` | Overall 5-hour rate-limit window |
| `seven_day` | Overall weekly window |
| `seven_day_opus` | Weekly Opus sub-cap (null for most Pro/Max users) |
| `seven_day_sonnet` | Weekly Sonnet sub-cap |
| `seven_day_omelette` | **Claude Design** weekly quota |
| `seven_day_cowork`, `seven_day_oauth_apps`, `iguana_necktie`, `omelette_promotional` | Unknown (all null in testing) |

## Repo layout

```
claude-monitor/
├── README.md
├── CLAUDE.md                   architectural notes & roadmap
├── package.json, vite.config.js, svelte.config.js, tsconfig.json
├── src/                        Svelte frontend
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +layout.ts          (SSR off, SPA mode)
│   │   └── +page.svelte        main dropdown
│   ├── lib/
│   │   ├── types.ts
│   │   ├── format.ts           pace zones, duration/token formatters
│   │   ├── GaugeRow.svelte
│   │   ├── ProfileHeader.svelte
│   │   ├── StatsTile.svelte
│   │   └── ErrorBanner.svelte
│   └── app.css                 Tailwind entrypoint
└── src-tauri/                  Rust backend
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    └── src/
        ├── main.rs             binary entry
        ├── lib.rs              Tauri setup, tray icon, window toggle
        ├── types.rs            shared Serialize types
        ├── api.rs              reqwest client for /usage /profile
        ├── keychain.rs         reads Claude Code OAuth token
        ├── stats.rs            walks ~/.claude/projects for tokens-by-model
        ├── store.rs            usage.json + profile.json cache
        ├── notify.rs           threshold notifications
        └── commands.rs         #[tauri::command] handlers + AppState
```

## Credits

Inspired by [TokenEater](https://github.com/AThevon/TokenEater) — the original reverse-engineering work on the Anthropic OAuth usage endpoint.

## License

See [LICENSE](LICENSE).
