# Bocah Timer

Menu-bar countdown timer for macOS. When the countdown ends, a translucent
fullscreen overlay covers the main display (above the menu bar and Dock, across
all Spaces — *not* a separate fullscreen Space) until you dismiss it.

Universal binary — runs on Intel and Apple Silicon.

## Features

- Menu-bar (tray) app, no Dock icon
- Set duration in minutes + seconds
- Translucent fullscreen overlay on finish (main display only)
- Custom overlay message, color, and opacity
- Alarm sound (Web Audio beep loop)
- Dismiss via on-screen button or `Esc` / `Space`
- Loop / repeat (restarts the timer after dismiss)
- Start at login (toggle in settings)

## Develop

```bash
bun install
bun run app:dev      # tray icon appears; click it to open the popover
```

## Build (universal Intel + ARM)

```bash
bun run app:build
```

Output:

- `.app`: `src-tauri/target/universal-apple-darwin/release/bundle/macos/Bocah Timer.app`
- `.dmg`: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/`

> On a Mac whose default `cargo` is not rustup-managed (e.g. MacPorts/Homebrew
> Rust without the aarch64 std), prefix with the rustup toolchain:
> `PATH="$HOME/.cargo/bin:$PATH" bun run app:build`.

## Releases (GitHub Actions)

`.github/workflows/build.yml` builds the universal app on `macos-latest` and
**publishes a GitHub Release automatically** on every push to `main` (release
`Bocah Timer v<version>`, version read from `tauri.conf.json`). The `.dmg`
and `.app.tar.gz` are attached. Bump `version` in `package.json` +
`src-tauri/tauri.conf.json` to cut a new release.

## Installing (Gatekeeper)

The build is **unsigned**, so on first launch macOS shows
*"Bocah Timer cannot be opened because the developer cannot be verified."*
This is expected for any app without a paid Apple Developer ID. To open:

```bash
# easiest — clear the quarantine flag, then open normally
xattr -dr com.apple.quarantine "/Applications/Bocah Timer.app"
```

or **right-click the app → Open → Open**, or
**System Settings → Privacy & Security → Open Anyway**.

### Remove the warning entirely (optional)

Sign + notarize with an Apple Developer ID ($99/yr). Add these repo secrets and
the CI build signs automatically (no code change needed):

| Secret | What |
| --- | --- |
| `APPLE_CERTIFICATE` | base64 of your Developer ID `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | `.p12` password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Name (TEAMID)` |
| `APPLE_ID` | Apple ID email |
| `APPLE_PASSWORD` | app-specific password |
| `APPLE_TEAM_ID` | 10-char team id |

## How the overlay works

The overlay is a separate transparent Tauri window sized to the primary
monitor. On macOS it is raised with `NSWindow.setLevel(1000)`
(`kCGScreenSaverWindowLevel`) and `canJoinAllSpaces | fullScreenAuxiliary`
collection behavior — so it floats above everything without taking over a
Space. See `src-tauri/src/lib.rs` → `raise_above_everything`.

## Layout

```
src/               Svelte frontend
  App.svelte       popover: timer input + settings
  Overlay.svelte   fullscreen overlay + alarm
src-tauri/src/
  lib.rs           timer thread, tray, overlay window, NSWindow level
```
