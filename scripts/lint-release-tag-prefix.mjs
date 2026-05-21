#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index === -1) return undefined;
  return args[index + 1];
}

function has(flag) {
  return args.includes(flag);
}

const root = path.resolve(valueAfter("--root") ?? process.cwd());
const configPath = path.join(root, "release-tag-prefixes.json");
const checkConfig = has("--check-config") || !has("--tag");
const tagToCheck =
  valueAfter("--tag") ??
  (process.env.GITHUB_REF_TYPE === "tag" ? process.env.GITHUB_REF_NAME : undefined);

function readText(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function fail(message) {
  console.error(`release tag prefix lint failed: ${message}`);
  process.exitCode = 1;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function loadConfig() {
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  if (!Array.isArray(config.releaseTags) || config.releaseTags.length === 0) {
    fail("release-tag-prefixes.json must define at least one releaseTags entry");
    return [];
  }
  return config.releaseTags;
}

function validateTagFormat(tag, entries) {
  const matches = entries.filter((entry) => tag.startsWith(entry.tagPrefix));
  if (matches.length === 0) {
    fail(
      `tag '${tag}' does not start with an allowed release prefix: ${entries
        .map((entry) => entry.tagPrefix)
        .join(", ")}`,
    );
    return;
  }
  if (matches.length > 1) {
    fail(`tag '${tag}' matches multiple release prefixes`);
    return;
  }

  const { tagPrefix } = matches[0];
  const semverPattern = new RegExp(
    `^${escapeRegExp(tagPrefix)}\\d+\\.\\d+\\.\\d+(?:[-+][0-9A-Za-z.-]+)?$`,
  );
  if (!semverPattern.test(tag)) {
    fail(`tag '${tag}' must match ${tagPrefix}<semver>`);
  }
}

function validateConfigEntry(entry) {
  for (const key of [
    "name",
    "packageName",
    "tagPrefix",
    "versionFile",
    "selfUpdateSource",
    "selfUpdateConstant",
  ]) {
    if (typeof entry[key] !== "string" || entry[key].length === 0) {
      fail(`releaseTags entry is missing string field '${key}'`);
    }
  }
  if (!Array.isArray(entry.releaseWorkflows) || entry.releaseWorkflows.length === 0) {
    fail(`${entry.name} must list releaseWorkflows`);
  }
  if (!entry.tagPrefix.endsWith("-v")) {
    fail(`${entry.name} tagPrefix '${entry.tagPrefix}' must end with '-v'`);
  }

  const versionFile = readText(entry.versionFile);
  const packageNameMatch = versionFile.match(/^\s*name\s*=\s*"([^"]+)"/m);
  if (!packageNameMatch) {
    fail(`${entry.versionFile} does not expose a Cargo package name`);
  } else if (packageNameMatch[1] !== entry.packageName) {
    fail(
      `${entry.versionFile} package name '${packageNameMatch[1]}' does not match configured packageName '${entry.packageName}'`,
    );
  }

  const selfUpdateSource = readText(entry.selfUpdateSource);
  const constantPattern = new RegExp(
    `const\\s+${escapeRegExp(entry.selfUpdateConstant)}\\s*:\\s*&str\\s*=\\s*"([^"]+)"`,
  );
  const constantMatch = selfUpdateSource.match(constantPattern);
  if (!constantMatch) {
    fail(`${entry.selfUpdateSource} does not define ${entry.selfUpdateConstant}`);
  } else if (constantMatch[1] !== entry.tagPrefix) {
    fail(
      `${entry.selfUpdateSource} ${entry.selfUpdateConstant} '${constantMatch[1]}' does not match configured tagPrefix '${entry.tagPrefix}'`,
    );
  }

  for (const workflowPath of entry.releaseWorkflows) {
    const workflow = readText(workflowPath);
    if (!workflow.includes(`${entry.tagPrefix}*`) && !workflow.includes(entry.tagPrefix)) {
      fail(`${workflowPath} does not mention configured tag prefix '${entry.tagPrefix}'`);
    }
    if (workflow.includes("tag=v$VERSION") || workflow.includes('"v$VERSION"')) {
      fail(`${workflowPath} appears to generate a bare v$VERSION release tag`);
    }
  }
}

function validateWorkflowTagGlobs(entries) {
  const workflowDir = path.join(root, ".github", "workflows");
  if (!fs.existsSync(workflowDir)) return;

  const allowedGlobs = new Set(entries.map((entry) => `${entry.tagPrefix}*`));
  for (const fileName of fs.readdirSync(workflowDir)) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;
    const relativePath = path.join(".github", "workflows", fileName);
    const workflow = readText(relativePath);
    for (const match of workflow.matchAll(/^\s*-\s*['"]?([^'"\s]+)['"]?\s*$/gm)) {
      const glob = match[1];
      if (!glob.includes("*") || allowedGlobs.has(glob)) continue;
      if (/^v\*$/.test(glob) || /-cli-v\*$/.test(glob)) {
        fail(`${relativePath} uses unconfigured release tag glob '${glob}'`);
      }
    }
  }
}

const entries = loadConfig();
for (const entry of entries) validateTagFormat(`${entry.tagPrefix}0.0.0`, entries);
if (checkConfig) {
  for (const entry of entries) validateConfigEntry(entry);
  validateWorkflowTagGlobs(entries);
}
if (tagToCheck) validateTagFormat(tagToCheck, entries);

if (process.exitCode) process.exit(process.exitCode);
console.log("release tag prefix lint passed");
