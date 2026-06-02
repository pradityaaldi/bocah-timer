#!/usr/bin/env node
// Downloads the latest Bocah Timer .app via the terminal (so it is never
// browser-quarantined), strips quarantine, installs to /Applications, launches.
// macOS-only. No Gatekeeper warning, no paid Apple account.

const { execFileSync } = require("child_process");
const os = require("os");
const fs = require("fs");
const path = require("path");

if (process.platform !== "darwin") {
  console.error("Bocah Timer is macOS-only.");
  process.exit(1);
}

const REPO = "pradityaaldi/bocah-timer";
const ASSET = "Bocah.Timer_universal.app.tar.gz";
const URL = `https://github.com/${REPO}/releases/latest/download/${ASSET}`;
const DEST = "/Applications";

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

  try {
    run("xattr", ["-dr", "com.apple.quarantine", path.join(DEST, app)]);
  } catch {
    /* nothing to strip */
  }

  console.log("==> Done. Launching Bocah Timer 🎉");
  run("open", [path.join(DEST, app)]);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}
