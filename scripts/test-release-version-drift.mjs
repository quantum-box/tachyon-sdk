#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { releasePackages, readVersion } from "./release-packages.mjs";
import {
  findDrift,
  githubReleaseExists,
  npmVersionExists,
  renderMarkdownTable,
} from "./check-release-version-drift.mjs";

function stubFetch(routes) {
  return async (url) => {
    const route = routes[url];
    if (!route) throw new Error(`unexpected request: ${url}`);
    return {
      status: route.status,
      json: async () => route.body,
    };
  };
}

async function testNpmVersionExists() {
  const fetchImpl = stubFetch({
    "https://registry.npmjs.org/@tachyon-sdk/storekit/0.3.1": { status: 200 },
    "https://registry.npmjs.org/@tachyon-sdk/storekit/0.3.2": { status: 404 },
  });

  assert.equal(
    await npmVersionExists("@tachyon-sdk/storekit", "0.3.1", fetchImpl),
    true,
  );
  assert.equal(
    await npmVersionExists("@tachyon-sdk/storekit", "0.3.2", fetchImpl),
    false,
  );

  // A registry outage must not be reported as "not published", which would
  // make the workflow dispatch a publish for something already released.
  await assert.rejects(
    () =>
      npmVersionExists(
        "@tachyon-sdk/storekit",
        "0.3.1",
        stubFetch({
          "https://registry.npmjs.org/@tachyon-sdk/storekit/0.3.1": { status: 503 },
        }),
      ),
    /npm returned 503/,
  );
}

async function testGithubReleaseExists() {
  const url = "https://api.github.com/repos/quantum-box/tachyon-sdk/releases/tags/tachyon-cli-v0.6.53";

  assert.equal(
    await githubReleaseExists("tachyon-cli-v0.6.53", {
      repository: "quantum-box/tachyon-sdk",
      fetchImpl: stubFetch({ [url]: { status: 200, body: { assets: [{ id: 1 }] } } }),
    }),
    true,
  );

  // Auto Release CLI bumps the version and creates the tag before the build
  // matrix uploads anything, so a release can exist with no binaries. That
  // is not a shipped release.
  assert.equal(
    await githubReleaseExists("tachyon-cli-v0.6.53", {
      repository: "quantum-box/tachyon-sdk",
      fetchImpl: stubFetch({ [url]: { status: 200, body: { assets: [] } } }),
    }),
    false,
  );

  assert.equal(
    await githubReleaseExists("tachyon-cli-v0.6.53", {
      repository: "quantum-box/tachyon-sdk",
      fetchImpl: stubFetch({ [url]: { status: 404 } }),
    }),
    false,
  );
}

async function testFindDriftReportsUnpublishedOnly() {
  const packages = [
    { name: "a", versionFile: "a", versionFormat: "npm", workflow: "wa", registry: "npm" },
    { name: "b", versionFile: "b", versionFormat: "npm", workflow: "wb", registry: "npm" },
  ];

  const entries = await findDrift({
    packages,
    readRepoVersion: (entry) => (entry.name === "a" ? "1.0.0" : "2.0.0"),
    isPublished: async (entry) => entry.name === "a",
  });

  assert.deepEqual(
    entries.map((entry) => [entry.name, entry.version, entry.published]),
    [
      ["a", "1.0.0", true],
      ["b", "2.0.0", false],
    ],
  );
  assert.deepEqual(
    entries.filter((entry) => !entry.published).map((entry) => entry.workflow),
    ["wb"],
  );
}

// The workflow dispatches entry.workflow by basename, so a typo there would
// only surface at 03:00 in a scheduled run. Assert the files exist instead.
function testEveryWorkflowReferenceResolves() {
  const repoRoot = path.resolve(new URL("..", import.meta.url).pathname);
  for (const entry of releasePackages) {
    assert.ok(
      fs.existsSync(path.join(repoRoot, entry.workflow)),
      `${entry.name} points at a missing workflow: ${entry.workflow}`,
    );
    assert.ok(
      fs.existsSync(path.join(repoRoot, entry.versionFile)),
      `${entry.name} points at a missing version file: ${entry.versionFile}`,
    );
    if (entry.registry === "github-release") {
      assert.ok(entry.releaseTagPrefix, `${entry.name} needs a releaseTagPrefix`);
    }
  }
}

// readVersion parses two different manifest formats. Point it at the real
// files so a Cargo.toml reformat or a manifest rename fails here rather than
// in a scheduled run nobody is watching.
function testRepoVersionsAreReadable() {
  const repoRoot = path.resolve(new URL("..", import.meta.url).pathname);
  for (const entry of releasePackages) {
    const body = fs.readFileSync(path.join(repoRoot, entry.versionFile), "utf8");
    const version = readVersion(body, entry.versionFormat, entry.versionFile);
    assert.match(
      version,
      /^\d+\.\d+\.\d+/,
      `${entry.name} version does not look like semver: ${version}`,
    );
  }
}

function testRenderMarkdownTable() {
  const table = renderMarkdownTable([
    {
      name: "@tachyon-sdk/storekit",
      version: "0.3.1",
      registry: "npm",
      workflow: ".github/workflows/publish-storekit.yml",
    },
  ]);
  const lines = table.split("\n");

  assert.equal(lines.length, 3, "header, separator, and one row");
  assert.equal(
    lines[2],
    "| `@tachyon-sdk/storekit` | 0.3.1 | npm | `.github/workflows/publish-storekit.yml` |",
  );

  // An empty list still has to render a valid table, because the workflow
  // pipes this straight into an issue body.
  assert.equal(renderMarkdownTable([]).split("\n").length, 2);
}

const tests = [
  testNpmVersionExists,
  testGithubReleaseExists,
  testFindDriftReportsUnpublishedOnly,
  testEveryWorkflowReferenceResolves,
  testRepoVersionsAreReadable,
  testRenderMarkdownTable,
];

let failed = false;
for (const test of tests) {
  try {
    await test();
    console.log(`PASS ${test.name}`);
  } catch (error) {
    failed = true;
    console.error(`FAIL ${test.name}: ${error.message}`);
  }
}

if (failed) process.exit(1);
console.log("release version drift lint tests passed");
