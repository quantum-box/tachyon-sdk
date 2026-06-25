#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const packageRoot = path.resolve(__dirname, "..");
const binaryPath = path.join(packageRoot, "vendor", "tachyon");

function ensureBinary() {
  if (fs.existsSync(binaryPath)) {
    return;
  }

  const installScript = path.join(packageRoot, "scripts", "install.js");
  const result = spawnSync(process.execPath, [installScript], {
    stdio: "inherit",
    env: process.env,
  });

  if (result.error) {
    console.error(`tachyon: failed to install the native binary: ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function exitFrom(result) {
  if (result.error) {
    console.error(`tachyon: failed to execute ${binaryPath}: ${result.error.message}`);
    process.exit(1);
  }

  if (result.signal) {
    const signalExitCodes = {
      SIGINT: 130,
      SIGTERM: 143,
    };
    process.exit(signalExitCodes[result.signal] ?? 1);
  }

  process.exit(result.status ?? 0);
}

ensureBinary();

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});
exitFrom(result);
