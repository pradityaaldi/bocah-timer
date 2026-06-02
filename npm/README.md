# bocah-timer

Installer for **Bocah Timer** — a menu-bar countdown timer for macOS that
covers your screen with a translucent fullscreen overlay (confetti + alarm)
when time's up. Universal: Intel + Apple Silicon.

> This npm package is just the **installer**. It downloads the latest signed-by-
> nobody-but-quarantine-free `.app` from GitHub Releases and drops it in
> `/Applications`. Because the download happens in the terminal (not a browser),
> macOS shows **no "unverified developer" Gatekeeper warning**.

## Install

```bash
npm install -g bocah-timer   # postinstall downloads + installs the app
bocah-timer                  # launch it (installs first if missing)
```

One-shot, without a global install:

```bash
npx bocah-timer
```

The app lands at `/Applications/Bocah Timer.app`.

## Requirements

- **macOS** only (Intel or Apple Silicon). Install is a no-op/error elsewhere.
- `curl`, `tar` (ships with macOS).

## What it does

1. Downloads `Bocah.Timer_universal.app.tar.gz` from the latest GitHub Release.
2. Extracts it, strips the quarantine flag, moves it to `/Applications`.
3. `bocah-timer` then launches the app.

## Uninstall

```bash
rm -rf "/Applications/Bocah Timer.app"
npm uninstall -g bocah-timer
```

## Links

- Source & releases: <https://github.com/pradityaaldi/bocah-timer>
- Other install methods (Homebrew cask, curl script, `.dmg`): see the repo README.

MIT © praditya
