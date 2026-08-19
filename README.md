# SooperUtils

A system tray utility application built with **Tauri 2.0**, **Rust**, and **Svelte 5 + Vite**.

## Features

- System tray integration (left-click toggles the main window, right-click opens the menu)
- **Global keypress counting** — tracks day / week / month / year / total keypresses
- Data persisted locally across restarts (`tauri-plugin-store`)
- Live debug panel (listener status, event counts, macOS permission state)
- About window

## macOS Permissions

Global key listening on macOS requires two privacy permissions, granted under
**System Settings → Privacy & Security**:

- **Input Monitoring** (`kTCCServiceListenEvent`)
- **Accessibility** (`kTCCServiceAccessibility`)

> ⚠️ macOS **silently drops** keyboard events for apps without these permissions —
> the app appears to run normally but never counts anything. If keypresses stop
> registering, check the **Keys → Show Debug** panel (it shows ✅/❌ per permission)
> or click **Open Permission Settings** from the Keys tab.

### Signing & permission persistence

This repo signs with a local self-signed certificate (`SooperUtils Dev Signing` in
`tauri.conf.json` → `bundle.macOS.signingIdentity`). Because the permission grants
are keyed to the app's code signature, using the **same certificate for every build**
means permissions survive rebuilds.

If you switch back to ad-hoc signing (`signingIdentity: "-"`), the grants are keyed
to the binary's content hash and **must be re-granted after every rebuild**:

```bash
tccutil reset Accessibility com.darren.sooperutils
tccutil reset ListenEvent com.darren.sooperutils
# then relaunch the app and click Allow on the prompts
```

## Tech Stack

| Layer    | Technology                         |
| -------- | ---------------------------------- |
| Backend  | Rust, Tauri 2, `rdev`, `tauri-plugin-store` |
| Frontend | Svelte 5, TypeScript, Vite         |

### Vendored `rdev`

`rdev` is vendored under `src-tauri/vendor/rdev` and wired in via
`[patch.crates-io]` in `Cargo.toml`. Stock rdev 0.5.3 calls macOS Text Input
Services (TIS) APIs from its background event-tap thread to resolve key names,
which **aborts the process** (`SIGTRAP` / `dispatch_assert_queue`) on modern macOS.
The vendored copy removes that key-name lookup (the app only needs `EventType`,
not `event.name`) and includes a regression test
(`converts_key_down_without_crashing_off_main_thread`).

## Development

```bash
# 1. Frontend dependencies
cd frontend && npm install

# 2. Run in development mode (starts Vite + the app)
cd ../src-tauri && cargo tauri dev

# 3. Build for production (DMG + .app in src-tauri/target/release/bundle)
cargo tauri build
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
