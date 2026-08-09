#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = path.resolve(new URL("..", import.meta.url).pathname);
const lintScript = path.join(repoRoot, "scripts", "lint-release-version-bump.mjs");
const packages = [
  {
    name: "tachyon-cli",
    root: "cli",
    versionFile: "cli/Cargo.toml",
    sourceFile: "cli/src/main.rs",
    testFile: "cli/tests/main.rs",
    format: "cargo",
  },
  ...[
    ["@tachyon-sdk/cli", "cli"],
    ["@tachyon-sdk/agent", "agent"],
    ["@tachyon-sdk/agent-chat", "agent-chat"],
    ["@tachyon-sdk/storage", "storage"],
    ["@tachyon-sdk/storekit", "storekit"],
  ].map(([name, directory]) => ({
    name,
    root: `packages/${directory}`,
    versionFile: `packages/${directory}/package.json`,
    sourceFile: `packages/${directory}/src/index.ts`,
    testFile: `packages/${directory}/tests/index.test.ts`,
    format: "npm",
  })),
];

function writeFile(root, relativePath, body) {
  const target = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, body);
}

function appendFile(root, relativePath, body) {
  fs.appendFileSync(path.join(root, relativePath), body);
}

function git(root, ...args) {
  const result = spawnSync("git", ["-C", root, ...args], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout.trim();
}

function commit(root, message) {
  git(root, "add", "--", "cli", "packages", "docs");
  git(root, "commit", "-q", "-m", message);
  return git(root, "rev-parse", "HEAD");
}

function setVersion(root, entry, version) {
  const target = path.join(root, entry.versionFile);
  if (entry.format === "cargo") {
    const body = fs.readFileSync(target, "utf8");
    fs.writeFileSync(target, body.replace(/^version = "[^"]+"/m, `version = "${version}"`));
    return;
  }
  const manifest = JSON.parse(fs.readFileSync(target, "utf8"));
  manifest.version = version;
  fs.writeFileSync(target, `${JSON.stringify(manifest, null, 2)}\n`);
}

function createFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-version-bump-"));
  git(root, "init", "-q");
  git(root, "config", "user.email", "release-version-test@example.com");
  git(root, "config", "user.name", "Release Version Test");

  for (const entry of packages) {
    const manifest =
      entry.format === "cargo"
        ? '[package]\nname = "tachyon"\nversion = "1.0.0"\n'
        : `${JSON.stringify({ name: entry.name, version: "1.0.0" }, null, 2)}\n`;
    writeFile(root, entry.versionFile, manifest);
    writeFile(root, entry.sourceFile, "export const value = 1;\n");
    writeFile(root, `${entry.root}/README.md`, "# Package\n");
    writeFile(root, entry.testFile, "// package test\n");
  }
  writeFile(root, "docs/overview.md", "# Repository docs\n");
  return { root, base: commit(root, "base") };
}

function runGuard(root, base, head) {
  return spawnSync(
    process.execPath,
    [lintScript, "--root", root, "--base", base, "--head", head],
    { encoding: "utf8" },
  );
}

function showResult(name, result) {
  console.log(`--- ${name} (exit ${result.status}) ---`);
  process.stdout.write(result.stdout);
  process.stdout.write(result.stderr);
}

function runCase(name, mutate, verify) {
  const fixture = createFixture();
  try {
    mutate(fixture.root);
    const head = commit(fixture.root, name);
    const result = runGuard(fixture.root, fixture.base, head);
    showResult(name, result);
    verify(result);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
}

runCase(
  "source changed without version bump",
  (root) => {
    for (const entry of packages) appendFile(root, entry.sourceFile, "export const changed = true;\n");
  },
  (result) => {
    assert.notEqual(result.status, 0);
    for (const entry of packages) assert.match(result.stderr, new RegExp(`FAIL ${entry.name}`));
  },
);

runCase(
  "source and version changed",
  (root) => {
    for (const entry of packages) {
      appendFile(root, entry.sourceFile, "export const changed = true;\n");
      setVersion(root, entry, "1.0.1");
    }
  },
  (result) => {
    assert.equal(result.status, 0, result.stderr);
    for (const entry of packages) assert.match(result.stdout, new RegExp(`PASS ${entry.name}`));
  },
);

runCase(
  "docs and tests only",
  (root) => {
    for (const entry of packages) {
      appendFile(root, `${entry.root}/README.md`, "More docs.\n");
      appendFile(root, entry.testFile, "// another test\n");
    }
  },
  (result) => {
    assert.equal(result.status, 0, result.stderr);
    for (const entry of packages) {
      assert.match(result.stdout, new RegExp(`SKIP ${entry.name}: only docs/tests changed`));
    }
  },
);

runCase(
  "release packages untouched",
  (root) => appendFile(root, "docs/overview.md", "More repository docs.\n"),
  (result) => {
    assert.equal(result.status, 0, result.stderr);
    for (const entry of packages) {
      assert.match(result.stdout, new RegExp(`SKIP ${entry.name}: package not changed`));
    }
  },
);

runCase(
  "version bump only",
  (root) => {
    for (const entry of packages) setVersion(root, entry, "1.0.1");
  },
  (result) => {
    assert.equal(result.status, 0, result.stderr);
    for (const entry of packages) {
      assert.match(result.stdout, new RegExp(`SKIP ${entry.name}: version-only change`));
    }
  },
);

console.log("release version bump lint tests passed");
