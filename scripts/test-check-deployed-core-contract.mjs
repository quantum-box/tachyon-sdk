#!/usr/bin/env node
import assert from "node:assert/strict";

import { canonicalPath, checkContract, operations } from "./check-deployed-core-contract.mjs";

function spec(paths) {
  return { paths };
}

function baseline(entries) {
  return { version: 1, entries };
}

function entry(method, path) {
  return { method, path, reason: "test" };
}

const core = spec({
  "/v1/auth/policies/{id}": { get: {}, parameters: [] },
  "/v1/auth/policies/check": { post: {} },
});

// Parameter names differ between documents; only the shape matters.
assert.equal(canonicalPath("/v1/auth/policies/{policy_id}/"), "/v1/auth/policies/{}");
assert.deepEqual(
  [...operations(core)].sort(),
  ["GET /v1/auth/policies/{}", "POST /v1/auth/policies/check"],
);

// An SDK that publishes only served operations passes.
assert.deepEqual(
  checkContract({
    sdkSpec: spec({ "/v1/auth/policies/{policy_id}": { get: {} } }),
    coreSpec: core,
    baseline: baseline([]),
  }),
  [],
);

// An operation core has not deployed fails.
assert.deepEqual(
  checkContract({
    sdkSpec: spec({ "/v1/auth/contract-authority/resolve": { post: {} } }),
    coreSpec: core,
    baseline: baseline([]),
  }),
  ["missing from deployed core: POST /v1/auth/contract-authority/resolve"],
);

// A baseline entry carries a pre-existing gap.
assert.deepEqual(
  checkContract({
    sdkSpec: spec({ "/v1/order/quotes": { get: {} } }),
    coreSpec: core,
    baseline: baseline([entry("GET", "/v1/order/quotes")]),
    baseBaseline: baseline([entry("GET", "/v1/order/quotes")]),
  }),
  [],
);

// Stale entries must be removed so the baseline shrinks.
assert.deepEqual(
  checkContract({
    sdkSpec: spec({ "/v1/auth/policies/check": { post: {} } }),
    coreSpec: core,
    baseline: baseline([entry("POST", "/v1/auth/policies/check"), entry("GET", "/v1/gone")]),
  }),
  [
    "baseline entry is no longer in openapi.json; remove it: GET /v1/gone",
    "baseline entry is now served by core; remove it: POST /v1/auth/policies/check",
  ],
);

// The baseline cannot grow past the base revision.
assert.deepEqual(
  checkContract({
    sdkSpec: spec({ "/v1/auth/contract-authority/resolve": { post: {} } }),
    coreSpec: core,
    baseline: baseline([entry("POST", "/v1/auth/contract-authority/resolve")]),
    baseBaseline: baseline([]),
  }),
  ["baseline may only shrink; new entry: POST /v1/auth/contract-authority/resolve"],
);

// Entries without a reason are rejected.
assert.throws(
  () =>
    checkContract({
      sdkSpec: spec({}),
      coreSpec: core,
      baseline: baseline([{ method: "GET", path: "/v1/x" }]),
    }),
  /needs a reason/,
);

console.log("check-deployed-core-contract tests passed");
