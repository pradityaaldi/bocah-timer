# Nice Countdown

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

- `.app`: `src-tauri/target/universal-apple-darwin/release/bundle/macos/Nice Countdown.app`
- `.dmg`: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/`

> Unsigned build. First launch: right-click the app → **Open**, or
> `xattr -dr com.apple.quarantine "Nice Countdown.app"`. For distribution,
> add an Apple Developer signing identity + notarization.

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
