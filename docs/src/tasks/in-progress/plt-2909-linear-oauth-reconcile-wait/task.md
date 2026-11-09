# PLT-2909 — wait for Linear OAuth reconcile before issue creation

- Linear: PLT-2909
- Repository: tachyon-sdk
- Scope: Tachyon CLI Linear issue creation

## Goal

When the Tachyon API rejects Linear issue creation before the provider request
with a structured `reconcile_pending` recovery action, wait for background
reconcile and retry issue creation. Do not retry any response or failure that
does not prove the request stopped at Linear OAuth client resolution.

## Investigation

All supported command surfaces converge on `pm_cli::run_create`:

- `tachyon pm issue create`
- `tachyon issue create`
- `tachyon linear issue create`

The current implementation calls the shared `ApiClient::post`. That helper
retries every POST once after a Tachyon API 401 and therefore is not an
appropriate duplicate-safe boundary for issue creation.

Tachyon Apps PLT-2908 (`#7464`) returns HTTP 424 for request-time Linear OAuth
resolution failures. Its recovery contract is a tagged object:

```json
{
  "provider": "linear",
  "operation": "linear_oauth_client_resolve",
  "retryable": true,
  "token_refresh_required": false,
  "recovery": {
    "type": "reconcile_pending",
    "retry_after_seconds": 600,
    "reconcile_interval_seconds": 600
  }
}
```

`reconnect_required` is a separate, non-retryable recovery type. The recovery
object does not contain OAuth token or secret values.

## Design

Issue creation uses a one-attempt POST primitive that never performs the
shared 401 replay. `run_create` retries only a successfully parsed HTTP 424
whose provider, operation, and recovery type identify the pre-provider Linear
OAuth resolution boundary.

The first delay comes from `retry_after_seconds`; subsequent delays use
`reconcile_interval_seconds`. Waiting is bounded by a deterministic deadline.
Progress is emitted to stderr, while successful `--json` output remains
exclusively on stdout. With `--json`, a deadline failure also writes a
structured `linear_oauth_reconcile_timeout` result to stdout before the
operational error is reported on stderr.

Transport errors, success-body parse failures, `reconnect_required`, and all
other HTTP errors return immediately without replay. A wait timeout advises
checking reconcile worker operation and connection state without claiming that
Reconnect is required.

## Tests

- `reconcile_pending` followed by success
- wait timeout without Reconnect guidance
- `reconnect_required` and unrelated failures are not retried
- transport failure after request arrival is not retried
- JSON success remains on stdout and wait progress remains on stderr

All fixtures use fake identifiers and contain no OAuth token or secret values.
