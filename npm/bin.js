#!/usr/bin/env node
// `bocah-timer` command: launch the app, installing it first if missing.

const fs = require("fs");
const { execFileSync } = require("child_process");
const { installApp } = require("./install.js");

const APP = "/Applications/Bocah Timer.app";

try {
  if (!fs.existsSync(APP)) installApp({ open: false });
  console.log("==> Launching Bocah Timer 🎉");
  execFileSync("open", [APP], { stdio: "inherit" });
} catch (e) {
  console.error(e.message);
  process.exit(1);
}
