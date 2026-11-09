#!/usr/bin/env node
// Verify that every operation the SDK publishes is served by the deployed
// Tachyon core.
//
// Consumers (Field and others) call core only through this SDK, so the SDK's
// openapi.json is the single contract surface. Checking it here, once, against
// the deployed core OpenAPI replaces per-consumer source scanning: an operation
// can enter openapi.json only after core has deployed it.
//
// Operations already missing when this check was introduced are carried in a
// baseline file. The baseline may only shrink: a new entry, or an entry that
// core now serves or the SDK no longer publishes, fails the check.

import fs from "node:fs";
import { fileURLToPath } from "node:url";

const HTTP_METHODS = new Set(["get", "post", "put", "patch", "delete", "head", "options"]);

export function canonicalPath(path) {
  return path.replace(/\{[^}]+\}/g, "{}").replace(/\/+$/, "") || "/";
}

export function operationKey(method, path) {
  return `${method.toUpperCase()} ${canonicalPath(path)}`;
}

export function operations(spec) {
  if (!spec || typeof spec.paths !== "object" || spec.paths === null) {
    throw new Error("OpenAPI document has no paths object");
  }
  const result = new Set();
  for (const [path, item] of Object.entries(spec.paths)) {
    for (const method of Object.keys(item ?? {})) {
      if (HTTP_METHODS.has(method.toLowerCase())) {
        result.add(operationKey(method, path));
      }
    }
  }
  return result;
}

export function baselineKeys(baseline) {
  if (!baseline || baseline.version !== 1 || !Array.isArray(baseline.entries)) {
    throw new Error("baseline must be {\"version\": 1, \"entries\": [...]}");
  }
  const keys = new Set();
  for (const entry of baseline.entries) {
    if (typeof entry?.method !== "string" || typeof entry?.path !== "string") {
      throw new Error(`baseline entry needs method and path: ${JSON.stringify(entry)}`);
    }
    if (typeof entry.reason !== "string" || entry.reason.trim() === "") {
      throw new Error(`baseline entry needs a reason: ${entry.method} ${entry.path}`);
    }
    const key = operationKey(entry.method, entry.path);
    if (keys.has(key)) throw new Error(`duplicate baseline entry: ${key}`);
    keys.add(key);
  }
  return keys;
}

export function checkContract({ sdkSpec, coreSpec, baseline, baseBaseline }) {
  const sdk = operations(sdkSpec);
  const core = operations(coreSpec);
  const allowed = baselineKeys(baseline);
  const errors = [];

  for (const key of [...sdk].sort()) {
    if (!core.has(key) && !allowed.has(key)) {
      errors.push(`missing from deployed core: ${key}`);
    }
  }
  for (const key of [...allowed].sort()) {
    if (core.has(key)) {
      errors.push(`baseline entry is now served by core; remove it: ${key}`);
    } else if (!sdk.has(key)) {
      errors.push(`baseline entry is no longer in openapi.json; remove it: ${key}`);
    }
  }
  if (baseBaseline) {
    const previous = baselineKeys(baseBaseline);
    for (const key of [...allowed].sort()) {
      if (!previous.has(key)) {
        errors.push(`baseline may only shrink; new entry: ${key}`);
      }
    }
  }
  return errors;
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const name = argv[i];
    const value = argv[i + 1];
    if (!name?.startsWith("--") || value === undefined) {
      throw new Error(`invalid arguments near ${name}`);
    }
    args[name.slice(2)] = value;
  }
  for (const required of ["sdk-spec", "core-spec", "baseline"]) {
    if (!args[required]) throw new Error(`--${required} is required`);
  }
  return args;
}

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, "utf8"));
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const errors = checkContract({
    sdkSpec: readJson(args["sdk-spec"]),
    coreSpec: readJson(args["core-spec"]),
    baseline: readJson(args.baseline),
    baseBaseline: args["base-baseline"] ? readJson(args["base-baseline"]) : undefined,
  });
  if (errors.length > 0) {
    for (const error of errors) console.log(`::error title=Deployed core contract::${error}`);
    console.log(`FAIL: ${errors.length} problem(s)`);
    process.exit(1);
  }
  console.log("OK: every SDK operation is served by deployed core or carried in the baseline");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
