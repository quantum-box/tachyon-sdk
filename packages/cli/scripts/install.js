#!/usr/bin/env node
"use strict";

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { Readable } = require("node:stream");
const { pipeline } = require("node:stream/promises");
const { resolvePlatformTarget } = require("./platform");

const packageRoot = path.resolve(__dirname, "..");
const packageJson = require(path.join(packageRoot, "package.json"));
const vendorDir = path.join(packageRoot, "vendor");
const binaryPath = path.join(vendorDir, "tachyon");

async function main() {
  if (process.env.TACHYON_CLI_SKIP_DOWNLOAD === "1") {
    console.log("Skipping Tachyon CLI binary download because TACHYON_CLI_SKIP_DOWNLOAD=1");
    return;
  }

  const target = resolvePlatformTarget();
  const version = normalizeVersion(process.env.TACHYON_CLI_VERSION ?? packageJson.version);
  const tag = process.env.TACHYON_CLI_TAG ?? `tachyon-cli-v${version}`;
  const repo = process.env.TACHYON_CLI_REPOSITORY ?? "quantum-box/tachyon-sdk";
  const assetName = `${target.artifactName}.tar.gz`;
  const downloadUrl =
    process.env.TACHYON_CLI_DOWNLOAD_URL ??
    `https://github.com/${repo}/releases/download/${tag}/${assetName}`;

  fs.mkdirSync(vendorDir, { recursive: true });

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "tachyon-cli-"));
  const archivePath = path.join(tmpDir, assetName);

  try {
    console.log(`Downloading Tachyon CLI ${version} for ${target.os}/${target.arch}...`);
    await download(downloadUrl, archivePath);
    execFileSync("tar", ["-xzf", archivePath, "-C", tmpDir], {
      stdio: "inherit",
    });

    const extractedBinary = path.join(tmpDir, "tachyon");
    if (!fs.existsSync(extractedBinary)) {
      throw new Error(`Downloaded archive did not contain the tachyon binary: ${downloadUrl}`);
    }

    fs.copyFileSync(extractedBinary, binaryPath);
    fs.chmodSync(binaryPath, 0o755);
    console.log(`Installed Tachyon CLI to ${binaryPath}`);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

function normalizeVersion(version) {
  return version.replace(/^v/, "").trim();
}

async function download(url, destinationPath) {
  const response = await fetch(url, {
    headers: {
      "User-Agent": `@tachyon-sdk/cli/${packageJson.version}`,
    },
    redirect: "follow",
  });

  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.status} ${response.statusText}`);
  }
  if (!response.body) {
    throw new Error(`Failed to download ${url}: response body is empty`);
  }

  await pipeline(Readable.fromWeb(response.body), fs.createWriteStream(destinationPath));
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
