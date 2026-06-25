#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const packageRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(packageRoot, "..", "..");
const packageJson = require(path.join(packageRoot, "package.json"));
const cargoToml = fs.readFileSync(path.join(repoRoot, "cli", "Cargo.toml"), "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (!cargoVersion) {
  console.error("Could not read cli/Cargo.toml package version");
  process.exit(1);
}

if (packageJson.version !== cargoVersion) {
  console.error(
    `@tachyon-sdk/cli version ${packageJson.version} must match cli/Cargo.toml version ${cargoVersion}`,
  );
  process.exit(1);
}
