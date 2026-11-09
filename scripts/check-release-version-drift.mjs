#!/usr/bin/env node
// Detects release packages whose in-repo version was never published.
//
// The publish workflows are triggered only by a push touching their own
// package path, so a failed publish is never retried on its own: unless
// another commit happens to touch that same path, the version stays
// unpublished and nothing reports it. @tachyon-sdk/storekit sat at an
// unpublished 0.3.1 for almost four months that way. This script is the
// scheduled counterpart to the pull-request guard - the guard makes sure
// the version was bumped, this makes sure the bump actually shipped.
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

import { releasePackages, readVersion } from "./release-packages.mjs";

const NPM_REGISTRY = "https://registry.npmjs.org";
const GITHUB_API = "https://api.github.com";

// A specific version URL is asked for on purpose, rather than reading the
// `latest` dist-tag and comparing. Comparing would need semver ordering and
// would misread a deliberate rollback as drift; asking whether this exact
// version exists is the question we actually care about.
export async function npmVersionExists(name, version, fetchImpl = fetch) {
  const response = await fetchImpl(`${NPM_REGISTRY}/${name}/${version}`);
  if (response.status === 200) return true;
  if (response.status === 404) return false;
  throw new Error(`npm returned ${response.status} for ${name}@${version}`);
}

// A release with no assets is treated as missing: @tachyon-sdk/cli downloads
// the binaries from it at install time, so an assetless release is not a
// usable release, and publish-cli-npm.yml refuses it for the same reason.
export async function githubReleaseExists(tag, { repository, token, fetchImpl = fetch }) {
  const headers = { accept: "application/vnd.github+json" };
  if (token) headers.authorization = `Bearer ${token}`;

  const response = await fetchImpl(
    `${GITHUB_API}/repos/${repository}/releases/tags/${tag}`,
    { headers },
  );
  if (response.status === 404) return false;
  if (response.status !== 200) {
    throw new Error(`GitHub returned ${response.status} for release ${tag}`);
  }

  const release = await response.json();
  return Array.isArray(release.assets) && release.assets.length > 0;
}

export async function findDrift({ packages, readRepoVersion, isPublished }) {
  const entries = [];
  for (const entry of packages) {
    const version = readRepoVersion(entry);
    const published = await isPublished(entry, version);
    entries.push({
      name: entry.name,
      version,
      workflow: entry.workflow,
      registry: entry.registry,
      published,
    });
  }
  return entries;
}

// Rendered here rather than in the workflow: building a markdown table
// out of JSON inside a YAML-embedded shell script needs three levels of
// escaping and cannot be tested.
export function renderMarkdownTable(drifted) {
  const rows = drifted.map(
    (entry) =>
      `| \`${entry.name}\` | ${entry.version} | ${entry.registry} | \`${entry.workflow}\` |`,
  );
  return [
    "| package | version | registry | publish workflow |",
    "| --- | --- | --- | --- |",
    ...rows,
  ].join("\n");
}

function repoVersionReader(root) {
  return (entry) => {
    const body = fs.readFileSync(path.join(root, entry.versionFile), "utf8");
    return readVersion(body, entry.versionFormat, entry.versionFile);
  };
}

function publishedLookup({ repository, token }) {
  return (entry, version) => {
    if (entry.registry === "npm") return npmVersionExists(entry.name, version);
    return githubReleaseExists(`${entry.releaseTagPrefix}${version}`, {
      repository,
      token,
    });
  };
}

async function main() {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => {
    const index = args.indexOf(flag);
    return index === -1 ? undefined : args[index + 1];
  };

  const root = path.resolve(valueAfter("--root") ?? process.cwd());
  const repository = valueAfter("--repository") ?? process.env.GITHUB_REPOSITORY;
  if (!repository) {
    throw new Error("--repository or GITHUB_REPOSITORY is required");
  }

  const entries = await findDrift({
    packages: releasePackages,
    readRepoVersion: repoVersionReader(root),
    isPublished: publishedLookup({ repository, token: process.env.GITHUB_TOKEN }),
  });

  for (const entry of entries) {
    const label = entry.published ? "OK" : "MISSING";
    console.log(`${label} ${entry.name}@${entry.version} (${entry.registry})`);
  }

  const drifted = entries.filter((entry) => !entry.published);
  const outputFile = valueAfter("--output");
  if (outputFile) {
    fs.writeFileSync(outputFile, `${JSON.stringify(drifted, null, 2)}\n`);
  }

  const markdownFile = valueAfter("--markdown");
  if (markdownFile) {
    fs.writeFileSync(markdownFile, `${renderMarkdownTable(drifted)}\n`);
  }

  // Exit 0 either way. Drift is a state to report and remediate, not a
  // broken build, and the workflow needs to keep running to do both.
  if (drifted.length === 0) {
    console.log("release version drift: every package version is published");
  } else {
    console.log(`release version drift: ${drifted.length} unpublished version(s)`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(`release version drift check failed: ${error.message}`);
    process.exit(1);
  });
}
