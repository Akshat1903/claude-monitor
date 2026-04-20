# claude-monitor

A native-feel macOS menu bar app for tracking Claude AI usage limits in real time. Built with **Tauri v2 + Svelte 5 + Rust**.

Surfaces the 5-hour, weekly, and Claude Design (`omelette`) quotas from the Claude OAuth API, plus Today / Week token counts from your local Claude Code session transcripts.

> **Status:** Works end-to-end on macOS 13+. Unsigned ad-hoc builds — macOS Gatekeeper will prompt once on first launch.

---

## Features

- **Live gauges** for 5-hour window, weekly quota, and Claude Design when available
- **Menu bar label** shows the 5-hour % next to the tray icon (no need to open the popup just to peek)
- **Pace badge** (Chill / On track / Hot) based on linear-pace comparison
- **Account + plan header** from `/api/oauth/profile` (`Claude Max 5x`, etc.)
- **Headline stats** — Today / Week tokens + favorite model, computed from `~/.claude/projects/**/*.jsonl`
- **Smart notifications** at 50 / 80 / 95% thresholds, re-armed on each window reset
- **Rate-limit backoff** — when the API 429s, the app pauses polling until `Retry-After` elapses
- **Overlays fullscreen apps** — popup appears even when you're in fullscreen (via NSPanel)
- **Click anywhere to dismiss** — global mouse monitor closes the popup when you click outside it

## Screenshot

Popup anchored below the tray icon:

```
Claude Usage                         ↻
Akshat Gupta · Claude Max 5x

┌──────────┬──────────┬──────────┐
│  2.5m    │   23m    │ Opus 4.7 │
│  Today   │   Week   │ Favorite │
└──────────┴──────────┴──────────┘

5-hour window   [On track]         31%
████████░░░░░░░░░░░░░░░░░░░░░░░░░░░
Resets in 3h 12m

Weekly                             32%
██████████░░░░░░░░░░░░░░░░░░░░░░░░░
Resets in 3d 19h

Claude Design · weekly              0%
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

Updated just now                Close
```

---

## Installation

There are two paths. Pick one:

- **Option A — Build from source** (requires Rust + Node). The only supported option today since we haven't published signed binaries yet.
- **Option B — Download a release** (not available yet; will be added once we publish signed binaries).

### Option A: Build from source (step-by-step)

**1. Install Rust** — skip if `rustc --version` already works.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# accept the defaults by pressing Enter at each prompt
# then reload your shell so `cargo` is on PATH:
source "$HOME/.cargo/env"
rustc --version   # should print something like: rustc 1.95.0
```

**2. Install Node.js 18+** — skip if `node --version` already prints `v18` or higher.

Pick one:

```bash
# Option 1: Homebrew (recommended if you have brew)
brew install node

# Option 2: nvm (lets you switch Node versions per project)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.nvm/nvm.sh
nvm install 22
nvm use 22

# Option 3: download the installer from https://nodejs.org
```

Verify:

```bash
node --version   # v18.x or higher
npm --version
```

**3. Clone the repo**

```bash
git clone https://github.com/Akshat1903/claude-monitor.git
cd claude-monitor
```

**4. Install JS dependencies**

```bash
npm install
```

**5. Build the app**

```bash
npm run tauri build
```

First build takes **3–10 minutes** because Rust has to compile every dependency from scratch. Subsequent builds are much faster (seconds).

The command produces two artifacts:

- A binary at `src-tauri/target/release/claude-monitor` — runnable directly
- A macOS `.app` bundle at `src-tauri/target/release/bundle/macos/Claude Monitor.app` — draggable to `/Applications`

**6. Install & launch**

Pick one:

```bash
# Option 1: Install to /Applications and launch like any Mac app
cp -R "src-tauri/target/release/bundle/macos/Claude Monitor.app" /Applications/
open "/Applications/Claude Monitor.app"

# Option 2: Run the raw binary (no install step)
./src-tauri/target/release/claude-monitor
```

**7. First-launch prompts**

On first launch macOS will ask you for three things. Accept all three:

1. **Gatekeeper blocks the unsigned app.**
   You'll see: *"Claude Monitor.app cannot be opened because it is from an unidentified developer"*.
   **Close the dialog, then right-click (or Ctrl-click) the app in Finder → Open → Open Anyway.**
   You only have to do this once per machine.

2. **Keychain access prompt** — *"Claude Monitor wants to access 'Claude Code-credentials' in your Keychain."*
   Enter your Mac password and click **Always Allow**. The app reads a single item — the OAuth token Claude Code itself stored — and never writes anything to the keychain.

3. **Notifications permission** — accept so you get the 50 / 80 / 95% threshold alerts.

After that, the menu bar shows a gauge icon with your live 5-hour %. Click it to open the popup.

### Run in dev mode (optional, for contributors)

If you're changing the code and want hot-reload:

```bash
npm run tauri dev
```

This spins up Vite on localhost:1420 and rebuilds Rust on save. Frontend changes hot-reload; Rust changes trigger a rebuild-and-relaunch.

---

## Requirements

- macOS 13 (Ventura) or newer — Tauri v2's minimum for tray icons
- A **Claude Pro, Max, or Team** plan (the Free plan doesn't expose the usage endpoint)
- **Claude Code installed and logged in.** The app reads the OAuth token Claude Code stored in the macOS Keychain — it never asks you to log in separately.

  ```bash
  # If you haven't yet, install and log in to Claude Code:
  # https://docs.claude.com/en/docs/claude-code/overview
  claude /login
  ```

## How it works

- Tauri ships a single binary: Rust backend + webview that loads our Svelte bundle
- On launch, the Rust backend reads the Claude Code OAuth token from the Keychain via `/usr/bin/security`
- The API is queried on three triggers only:
  1. **App startup** (first fetch)
  2. **Every 5 minutes** — a background `tokio::time::sleep(300s)` loop
  3. **Manual refresh button** (↻ in the popup header)
- Opening the popup *does not* trigger a fetch — it reads the most recent cached snapshot
- Endpoints hit:
  - `GET https://api.anthropic.com/api/oauth/usage` → quota gauges
  - `GET https://api.anthropic.com/api/oauth/profile` → account + plan tier
- Response cache lives at `~/Library/Application Support/ClaudeMonitor/`
- Svelte frontend calls into Rust via Tauri `invoke()` for `refresh` and `get_snapshot`; backend pushes updates via `emit("snapshot-updated")`
- Today / Week token counts come from walking `~/.claude/projects/**/*.jsonl` locally — nothing leaves your machine for that

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

## Troubleshooting

**"Gatekeeper won't open the app"**
Right-click the `.app` in Finder → **Open** → confirm in the warning dialog. If that fails, open **System Settings → Privacy & Security**, scroll down and you'll see *"Claude Monitor was blocked"* — click **Open Anyway**.

**"Popup shows but all values are empty / shows a red error"**
Confirm Claude Code is authenticated:

```bash
claude --version
security find-generic-password -s "Claude Code-credentials" -w >/dev/null && echo "token present"
```

If either prints an error, run `claude /login` to re-authenticate.

**"Rate limited"**
The `/api/oauth/usage` endpoint has its own per-user throttle. The app automatically backs off when it gets a `429` response and resumes when the `Retry-After` window ends. Just wait — typically ~15–60 minutes.

**"I only see Today / Week in the stats tile — no 5-hour gauge"**
The API call failed and we're showing the cached stats (which come from local files). Click the refresh arrow in the top-right of the popup.

**"Nothing in the menu bar"**
```bash
pgrep -fl claude-monitor
```
If no process is listed, the app isn't running. Launch it again.

**Reset all cached state**
```bash
pkill -f claude-monitor
rm -rf ~/Library/Application\ Support/ClaudeMonitor/
```

## Auto-start at login

macOS doesn't auto-launch this app by default. To have it start when you log in:

1. Open **System Settings → General → Login Items & Extensions**
2. Under **Open at Login**, click `+`
3. Navigate to `/Applications/Claude Monitor.app` and add it

## Data sources

- **Anthropic OAuth endpoints** (undocumented — same ones Claude Code's `/status` uses):
  - `GET https://api.anthropic.com/api/oauth/usage` — quotas
  - `GET https://api.anthropic.com/api/oauth/profile` — account + plan
- **macOS Keychain**: `kSecAttrService = "Claude Code-credentials"`, extracts `claudeAiOauth.accessToken`
- **Local session transcripts**: `~/.claude/projects/**/*.jsonl` (for the stats tile only)

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
        ├── lib.rs              Tauri setup, tray, NSPanel conversion,
        │                       fullscreen overlay, global click monitor
        ├── types.rs            shared Serialize types
        ├── api.rs              reqwest client for /usage /profile
        ├── keychain.rs         reads Claude Code OAuth token
        ├── stats.rs            walks ~/.claude/projects for tokens-by-model
        ├── store.rs            usage.json + profile.json cache
        ├── notify.rs           threshold notifications
        └── commands.rs         #[tauri::command] handlers + AppState
```

## macOS specifics

The popup window does several macOS-only tricks to behave like a system menu bar app:

- **Activation policy**: `Accessory` — no Dock icon, no top-level app menu
- **NSPanel class swap**: the popup is runtime-cast from `NSWindow` to `NSPanel` so it can appear over fullscreen apps (the only reliable way on modern macOS)
- **`NSWindowStyleMask.nonactivatingPanel`**: panel never takes focus away from whichever app is active — important when you click the tray while in fullscreen Xcode / Safari / etc.
- **`NSScreenSaverWindowLevel` (1000)**: renders above fullscreen content
- **`CanJoinAllSpaces | FullScreenAuxiliary | Stationary`**: lives in every space including fullscreen
- **Global NSEvent click monitor**: since the panel is nonactivating it never "loses focus" in the normal sense, so we install an `NSEvent.addGlobalMonitorForEventsMatchingMask` to dismiss the popup when you click anywhere else on screen

All of this is handled in `src-tauri/src/lib.rs::set_panel_behavior` and `install_global_click_monitor`.

## Credits

Inspired by [TokenEater](https://github.com/AThevon/TokenEater) — the original reverse-engineering work on the Anthropic OAuth usage endpoint.

## License

See [LICENSE](LICENSE).
