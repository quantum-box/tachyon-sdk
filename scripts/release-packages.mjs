// Single source of truth for the packages that have a release pipeline.
//
// Both the pull-request guard (lint-release-version-bump.mjs) and the
// scheduled drift detector (check-release-version-drift.mjs) read this
// list. Keeping one copy is what makes the two agree: a package added
// here is guarded and monitored at the same time, instead of silently
// getting only one of the two.
export const releasePackages = [
  {
    name: "tachyon-cli",
    root: "cli",
    versionFile: "cli/Cargo.toml",
    versionFormat: "cargo",
    workflow: ".github/workflows/auto-release-cli.yml",
    // Not an npm package. It ships as a GitHub release carrying the
    // prebuilt binaries, which @tachyon-sdk/cli downloads on install.
    registry: "github-release",
    releaseTagPrefix: "tachyon-cli-v",
  },
  {
    name: "@tachyon-sdk/cli",
    root: "packages/cli",
    versionFile: "packages/cli/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-cli-npm.yml",
    registry: "npm",
  },
  {
    name: "@tachyon-sdk/agent",
    root: "packages/agent",
    versionFile: "packages/agent/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-agent.yml",
    registry: "npm",
  },
  {
    name: "@tachyon-sdk/agent-chat",
    root: "packages/agent-chat",
    versionFile: "packages/agent-chat/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-agent-chat.yml",
    registry: "npm",
  },
  {
    name: "@tachyon-sdk/storage",
    root: "packages/storage",
    versionFile: "packages/storage/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-storage.yml",
    registry: "npm",
  },
  {
    name: "@tachyon-sdk/storekit",
    root: "packages/storekit",
    versionFile: "packages/storekit/package.json",
    versionFormat: "npm",
    workflow: ".github/workflows/publish-storekit.yml",
    registry: "npm",
  },
];

export function readVersion(body, format, versionFile) {
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
