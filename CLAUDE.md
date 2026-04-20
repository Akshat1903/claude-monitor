# Claude Monitor

macOS menu bar app that tracks Claude AI usage limits. Menu-bar-only surface: tray icon + small popup window.

## Tech

- **Frontend**: Svelte 5 (SvelteKit in SPA mode, `adapter-static`) + TypeScript + Tailwind CSS v4
- **Backend**: Rust + Tauri v2
- **Build**: `npm run tauri build` (Vite builds Svelte; Cargo builds Rust; Tauri wraps both into a `.app` bundle)

## Data pipeline

1. **Keychain**: `/usr/bin/security find-generic-password -s "Claude Code-credentials" -w` → extract `claudeAiOauth.accessToken` (shell-out in `src-tauri/src/keychain.rs` — simpler than `security-framework`'s item-search API, and the process isn't sandboxed so it just works)
2. **API**: `reqwest` hits `GET https://api.anthropic.com/api/oauth/usage` and `/api/oauth/profile` with `Authorization: Bearer <token>` + `anthropic-beta: oauth-2025-04-20`
3. **Persistence**: `~/Library/Application Support/ClaudeMonitor/{usage,profile,alerts}.json` — hydrates UI across launches so the popup never shows blank when the API is throttled
4. **Stats**: `walkdir` over `~/.claude/projects/**/*.jsonl`, sums `input + output + cache_creation_input_tokens` per event in the last 7 days, groups by `message.model` → favorite model
5. **UI**: Svelte renders from `AppSnapshot` returned by the `get_snapshot` / `refresh` Tauri commands; backend also pushes updates via `app.emit("snapshot-updated", snapshot)`

## Tauri specifics

- **Tray**: programmatic `TrayIconBuilder::with_id("main-tray")` in `lib.rs::run()`; left-click toggles the popup window
- **Popup window**: borderless, transparent, alwaysOnTop, `skipTaskbar`, hidden on startup (visibility driven by tray click). Positioned under the tray icon via `tray_pos` in `TrayIconEvent::Click { rect, .. }`
- **Main thread refresh loop**: `tauri::async_runtime::spawn` with `tokio::time::sleep(300s)` — runs on Tauri's built-in runtime, so no separate tokio runtime boilerplate needed
- **Rate-limit backoff**: on `APIError::RateLimited`, `AppState.rate_limited_until` is set; subsequent `refresh_impl` calls short-circuit until that instant passes

## History (condensed)

This project started as a SwiftUI `MenuBarExtra` app. After hitting friction sharing it with teammates (Xcode install + Apple Team ID setup for every new clone), we ported to Tauri. Commit history documents the old Swift version; the `src-tauri` directory is the current home.

Features shipped:
- Live 5-hour + weekly + Claude Design gauges
- Pace badge (chill / on-track / hot)
- Account + plan header (`Claude Max 5x` etc.)
- Headline stats tile (Today / Week tokens + favorite model)
- macOS notifications at 50/80/95% with re-arm on reset
- Rate-limit aware backoff
- Auto-refresh every 5 min; on-window-open refresh if cache >60s old

Features tried and removed:
- **Burn-rate ETA** — noise (false alarms early in a window)
- **History sparkline** — flat lines weren't informative
- **Per-model breakdown (Opus vs Sonnet tiles)** — misleading; those fields are sub-cap pools, not per-model usage

## Known limits / next up

- No proper tray icon artwork (uses default app icon; should be a template PNG for macOS menu bar)
- No wake-from-sleep refresh (Swift version had this; would need `objc2` + `NSWorkspace.didWakeNotification` observer in Rust, or a poll-based timer that checks last-wake time)
- No code signing → Gatekeeper prompt on first launch

## References

- Tauri v2 tray docs: https://v2.tauri.app/learn/system-tray/
- `tauri-plugin-notification`: https://v2.tauri.app/plugin/notification/
- SvelteKit as SPA for Tauri: https://v2.tauri.app/start/frontend/sveltekit/

## Conventions

- No comments in code unless a non-obvious *why* is needed
- Snake_case JSON across the Rust/TS boundary (no camelCase transform — keeps types mechanical)
- All disk cache goes to `~/Library/Application Support/ClaudeMonitor/`
