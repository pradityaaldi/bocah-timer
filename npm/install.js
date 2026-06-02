#!/usr/bin/env node
// Core installer: downloads the latest Bocah Timer .app via the terminal (so it
// is never browser-quarantined), strips quarantine, installs to /Applications.
// macOS-only. Used by `postinstall` and by the `bocah-timer` command.

const { execFileSync } = require("child_process");
const os = require("os");
const fs = require("fs");
const path = require("path");

const REPO = "pradityaaldi/bocah-timer";
const ASSET = "Bocah.Timer_universal.app.tar.gz";
const URL = `https://github.com/${REPO}/releases/latest/download/${ASSET}`;
const DEST = "/Applications";

function installApp({ open = false } = {}) {
  if (process.platform !== "darwin") {
    throw new Error("Bocah Timer is macOS-only.");
  }
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "bocah-"));
  const run = (cmd, args) => execFileSync(cmd, args, { stdio: "inherit" });
  try {
    console.log("==> Downloading Bocah Timer…");
    run("curl", ["-fsSL", URL, "-o", path.join(tmp, "app.tar.gz")]);

    console.log("==> Extracting…");
    run("tar", ["-xzf", path.join(tmp, "app.tar.gz"), "-C", tmp]);

    const app = fs.readdirSync(tmp).find((f) => f.endsWith(".app"));
    if (!app) throw new Error("no .app found in release archive");

    console.log(`==> Installing ${app} to ${DEST}…`);
    fs.rmSync(path.join(DEST, app), { recursive: true, force: true });
    run("mv", [path.join(tmp, app), `${DEST}/`]);

    const installed = path.join(DEST, app);
    try {
      run("xattr", ["-dr", "com.apple.quarantine", installed]);
    } catch {
      /* nothing to strip */
    }

    console.log(`==> Installed: ${installed}`);
    if (open) run("open", [installed]);
    return installed;
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

module.exports = { installApp };

// `postinstall` runs this directly — install without launching.
if (require.main === module) {
  try {
    installApp({ open: false });
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
