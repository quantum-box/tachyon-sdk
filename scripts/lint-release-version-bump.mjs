#!/usr/bin/env node
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const args = process.argv.slice(2);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index === -1) return undefined;
  return args[index + 1];
}

const root = path.resolve(valueAfter("--root") ?? process.cwd());
const baseRef = valueAfter("--base") ?? "origin/main";
const headRef = valueAfter("--head") ?? "HEAD";

// These roots and version files come from the release workflows named here.
const releasePackages = [
  {
    name: "tachyon-cli",
    root: "cli",
    versionFile: "cli/Cargo.toml",
    versionFormat: "cargo",
    workflow: ".github/workflows/auto-release-cli.yml",
  },
  {
    name: "@tachyon-sdk/cli",
    root: "packages/cli",
    versionFile: "packages/cli/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-cli-npm.yml",
  },
  {
    name: "@tachyon-sdk/agent",
    root: "packages/agent",
    versionFile: "packages/agent/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-agent.yml",
  },
  {
    name: "@tachyon-sdk/agent-chat",
    root: "packages/agent-chat",
    versionFile: "packages/agent-chat/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-agent-chat.yml",
  },
  {
    name: "@tachyon-sdk/storage",
    root: "packages/storage",
    versionFile: "packages/storage/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-storage.yml",
  },
  {
    name: "@tachyon-sdk/storekit",
    root: "packages/storekit",
    versionFile: "packages/storekit/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-storekit.yml",
  },
];

function git(...gitArgs) {
  const result = spawnSync("git", ["-C", root, ...gitArgs], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    const detail = result.stderr.trim() || result.stdout.trim();
    throw new Error(`git ${gitArgs.join(" ")} failed: ${detail}`);
  }
  return result.stdout;
}

function readAt(ref, relativePath) {
  return git("show", `${ref}:${relativePath}`);
}

function readVersion(body, format, versionFile) {
  if (format === "npm") {
    const version = JSON.parse(body).version;
    if (typeof version !== "string" || version.length === 0) {
      throw new Error(`${versionFile} does not define a string version`);
    }
    return version;
  }

  const match = body.match(/^\s*version\s*=\s*"([^"]+)"/m);
  if (!match) throw new Error(`${versionFile} does not define a Cargo version`);
  return match[1];
}

function withoutVersion(body, format) {
  if (format === "npm") {
    const manifest = JSON.parse(body);
    delete manifest.version;
    return JSON.stringify(manifest);
  }
  return body.replace(/^(\s*version\s*=\s*")[^"]+(".*)$/m, "$1<version>$2");
}

function isDocsOrTest(relativePath) {
  const normalized = relativePath.toLowerCase();
  const segments = normalized.split("/");
  const basename = segments.at(-1);

  if (segments.some((segment) =>
    ["doc", "docs", "test", "tests", "__tests__", "__snapshots__", "fixtures"].includes(
      segment,
    ))) {
    return true;
  }
  if (/\.(?:test|spec)\.[^/]+$/.test(basename)) return true;
  if (/^(?:readme|changelog|contributing)(?:\.|$)/.test(basename)) return true;
  if (/^(?:license|notice)(?:\.|$)/.test(basename)) return true;
  if (/\.(?:md|mdx|rst)$/.test(basename)) return true;
  if (/^(?:vitest|jest|playwright)\.config\./.test(basename)) return true;
  return false;
}

function changedFiles(mergeBase, head, packageRoot) {
  return git(
    "diff",
    "--name-only",
    "-z",
    "--diff-filter=ACDMRTUXB",
    mergeBase,
    head,
    "--",
    packageRoot,
  )
    .split("\0")
    .filter(Boolean);
}

function releaseRelevantFiles(entry, files, mergeBase, head) {
  return files.filter((file) => {
    if (file !== entry.versionFile) return !isDocsOrTest(file);

    const baseManifest = readAt(mergeBase, entry.versionFile);
    const headManifest = readAt(head, entry.versionFile);
    return (
      withoutVersion(baseManifest, entry.versionFormat) !==
      withoutVersion(headManifest, entry.versionFormat)
    );
  });
}

try {
  const base = git("rev-parse", "--verify", `${baseRef}^{commit}`).trim();
  const head = git("rev-parse", "--verify", `${headRef}^{commit}`).trim();
  const mergeBase = git("merge-base", base, head).trim();
  console.log(
    `release version guard: comparing ${head.slice(0, 12)} with merge base ${mergeBase.slice(0, 12)} (base ${base.slice(0, 12)})`,
  );

  let failed = false;
  for (const entry of releasePackages) {
    const files = changedFiles(mergeBase, head, entry.root);
    if (files.length === 0) {
      console.log(`SKIP ${entry.name}: package not changed`);
      continue;
    }

    const baseManifest = readAt(mergeBase, entry.versionFile);
    const headManifest = readAt(head, entry.versionFile);
    const baseVersion = readVersion(baseManifest, entry.versionFormat, entry.versionFile);
    const headVersion = readVersion(headManifest, entry.versionFormat, entry.versionFile);
    const relevantFiles = releaseRelevantFiles(entry, files, mergeBase, head);

    if (relevantFiles.length === 0) {
      const reason =
        baseVersion === headVersion
          ? "only docs/tests changed"
          : `version-only change (${baseVersion} -> ${headVersion})`;
      console.log(`SKIP ${entry.name}: ${reason}`);
      continue;
    }

    if (baseVersion !== headVersion) {
      console.log(
        `PASS ${entry.name}: ${entry.versionFile} changed ${baseVersion} -> ${headVersion}`,
      );
      continue;
    }

    failed = true;
    console.error(
      `FAIL ${entry.name}: release-relevant files changed but ${entry.versionFile} is still ${headVersion}`,
    );
    console.error(`  release workflow: ${entry.workflow}`);
    for (const file of relevantFiles) console.error(`  changed: ${file}`);
  }

  if (failed) {
    console.error("release version guard failed: bump every affected package version");
    process.exit(1);
  }
  console.log("release version guard passed");
} catch (error) {
  console.error(`release version guard failed: ${error.message}`);
  process.exit(1);
}
