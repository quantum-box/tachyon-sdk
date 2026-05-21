#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = path.resolve(new URL("..", import.meta.url).pathname);
const lintScript = path.join(repoRoot, "scripts", "lint-release-tag-prefix.mjs");

function writeFile(root, relativePath, body) {
  const target = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, body);
}

function createFixture(overrides = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-tag-prefix-lint-"));
  writeFile(
    root,
    "release-tag-prefixes.json",
    JSON.stringify(
      {
        releaseTags: [
          {
            name: "tachyon-cli",
            packageName: "tachyon",
            tagPrefix: overrides.tagPrefix ?? "tachyon-cli-v",
            versionFile: "cli/Cargo.toml",
            selfUpdateSource: "cli/src/install_cli.rs",
            selfUpdateConstant: "TAG_PREFIX",
            releaseWorkflows: [
              ".github/workflows/release-cli.yml",
              ".github/workflows/auto-release-cli.yml",
            ],
          },
        ],
      },
      null,
      2,
    ),
  );
  writeFile(root, "cli/Cargo.toml", '[package]\nname = "tachyon"\nversion = "0.6.2"\n');
  writeFile(
    root,
    "cli/src/install_cli.rs",
    `const TAG_PREFIX: &str = "${overrides.selfUpdatePrefix ?? "tachyon-cli-v"}";\n`,
  );
  writeFile(
    root,
    ".github/workflows/release-cli.yml",
    "on:\n  push:\n    tags:\n      - 'tachyon-cli-v*'\n",
  );
  writeFile(
    root,
    ".github/workflows/auto-release-cli.yml",
    'run: |\n  echo "tag=tachyon-cli-v$VERSION" >> "$GITHUB_OUTPUT"\n',
  );
  return root;
}

function runLint(root, ...args) {
  return spawnSync(process.execPath, [lintScript, "--root", root, ...args], {
    encoding: "utf8",
  });
}

{
  const root = createFixture();
  const result = runLint(root, "--check-config", "--tag", "tachyon-cli-v1.2.3");
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /passed/);
}

{
  const root = createFixture();
  const result = runLint(root, "--tag", "v1.2.3");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not start with an allowed release prefix/);
}

{
  const root = createFixture({ selfUpdatePrefix: "v" });
  const result = runLint(root, "--check-config");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not match configured tagPrefix/);
}

{
  const root = createFixture();
  writeFile(
    root,
    ".github/workflows/release-cli.yml",
    "on:\n  push:\n    tags:\n      - 'v*'\n",
  );
  const result = runLint(root, "--check-config");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not mention configured tag prefix 'tachyon-cli-v'/);
}

console.log("release tag prefix lint tests passed");
