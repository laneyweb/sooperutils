# SooperUtils — Project Conventions

Tauri 2.0 (Rust) + Svelte 5 + Vite system-tray app. Global keypress counter
(day/week/month/year + per-key stats), persisted via `tauri-plugin-store`.
Frontend lives in `frontend/`, backend in `src-tauri/`.

## Build & Release — follow every time

1. **Always bump the version before building.** Keep the two version files in
   sync via the script (never edit them by hand):
   ```bash
   node scripts/bump-version.mjs            # patch (default)
   node scripts/bump-version.mjs minor      # minor
   node scripts/bump-version.mjs major      # major
   ```
   Default to `patch`. Use `minor` when the build includes new features, or
   whenever the user explicitly asks for a higher bump.

2. **Always build the DMG** (never just the bare binary):
   ```bash
   cd src-tauri && cargo tauri build
   ```
   `beforeBuildCommand` in `tauri.conf.json` rebuilds the frontend (Vite)
   automatically — do not skip it.

3. **Artifacts** (report these exact paths):
   - DMG: `src-tauri/target/release/bundle/dmg/sooperutils_<version>_aarch64.dmg`
   - App: `src-tauri/target/release/bundle/macos/sooperutils.app`

4. **Verify before reporting success:**
   - `PlistBuddy` → `CFBundleShortVersionString` in the built
     `sooperutils.app/Contents/Info.plist` equals the bumped version
   - `codesign -dv` on the `.app` shows a signature

## Signing notes

- Signed with self-signed identity `SooperUtils Dev Signing`
  (`tauri.conf.json` → `bundle.macOS.signingIdentity`).
- The keychain ACL is already configured — builds are prompt-free. Do **not**
  switch to ad-hoc signing (`-`); permission grants are keyed to the cert and
  would be invalidated on every rebuild.
- Notarization is skipped (no `APPLE_ID` / `APPLE_PASSWORD` env vars) — fine
  for local installs; Gatekeeper will flag the self-signed DMG.

## macOS permission context

Global key listening needs **Accessibility** + **Input Monitoring** grants
(System Settings → Privacy & Security). macOS silently drops events without
them — the app's Keys tab shows ✅/❌ per permission and has a link to open
the settings panes.
