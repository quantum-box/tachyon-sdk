# Tachyon Worker Runtime

`tachyon worker` runs an OSS Coding Job worker on a customer VPS or PC. It
replaces the separately distributed `tachyond` binary while keeping the runtime
boundary simple: the worker talks to Tachyon Cloud only through public REST
APIs.

## Quick Start

Run the worker while the current shell is open:

```sh
tachyon auth login --profile work
tachyon auth use work

tachyon --profile work --tenant-id tn_xxxx worker run
```

Stop it with `Ctrl-C`. The process deregisters the worker during graceful
shutdown.

On a Linux systemd host, install it as a background service:

```sh
sudo tachyon --profile work --tenant-id tn_xxxx worker start
```

`worker start` installs `tachyon-worker.service`, writes
`/etc/tachyon/worker.env`, enables the service, and starts it through systemd.
Before installation it performs a best-effort CLI self update unless
`--no-update` is passed.

## Runtime Model

- Default provider is `containerized_codex`.
- Jobs are claimed from `POST /v1/agent/coding-jobs/claim`.
- Results are reported to
  `POST /v1/agent/coding-jobs/{coding_job_id}/worker-complete`.
- Worker registration and heartbeat use the public worker APIs under
  `/v1/agent/workers`.
- Containerized jobs run through Docker with `codex exec --json <prompt>`.

The worker does not import or link to `tachyon-apps` internals. All control
plane state, job assignment, and result reporting stay behind the public API
contract.

## Credentials

`worker run` uses the selected CLI auth profile or `TACHYON_API_KEY` directly
for the current process.

`worker start` uses the selected CLI auth profile only during installation. It
creates or reuses a service account named `tachyon-worker-<worker_id>`, issues a
dedicated API key, attaches `CodingJobWorkerPolicy`, and writes that key to
`/etc/tachyon/worker.env` with owner-only permissions.

The long-running systemd process uses that worker API key. It does not need the
installer user's OAuth refresh token.

`worker start --dry-run` prints the unit and environment shape but never prints
the installer token or generated worker API key.

## Operational Commands

Foreground mode:

```sh
tachyon --profile work --tenant-id tn_xxxx worker run
tachyon --profile work --tenant-id tn_xxxx worker run --worker-id my-shell-worker
```

Systemd mode:

```sh
sudo tachyon worker status
sudo tachyon worker logs --lines 200
sudo tachyon worker logs --follow
sudo tachyon worker restart
sudo tachyon worker stop
```

The systemd commands wrap `systemctl` and `journalctl` for the local
`tachyon-worker.service`.

## Installed Files

| Path | Purpose |
| --- | --- |
| `/etc/systemd/system/tachyon-worker.service` | systemd service definition |
| `/etc/tachyon/worker.env` | API URL, tenant, worker ID, provider, and worker API key |

## Runtime Environment

| Variable | Purpose |
| --- | --- |
| `TACHYON_API_URL` | Tachyon Cloud API base URL |
| `TACHYON_TENANT_ID` | Operator tenant ID |
| `TACHYON_API_KEY` | Dedicated worker API key |
| `TACHYON_WORKER_ID` | Stable worker identifier |
| `TACHYON_WORKER_PROVIDER` | Provider capability, currently `containerized_codex` |
| `TACHYON_WORKER_MAX_CONCURRENT_JOBS` | Advertised worker concurrency |
| `TACHYON_WORKER_POLL_INTERVAL_MS` | Poll interval for `worker run` |
| `CODEX_CONTAINER_IMAGE` | Codex runner image |
| `CODEX_CONTAINER_NETWORK` | Docker network for job containers |
| `CODEX_CONTAINER_MEMORY` | Docker memory limit |

## E2E Check

### Foreground

1. Run `tachyon --tenant-id tn_xxxx worker run`.
2. Create a Coding Job with provider `containerized_codex`.
3. Watch the shell output for claim and completion.
4. Stop the worker with `Ctrl-C`.
5. Fetch the Coding Job from the API and confirm `status` is `succeeded` or
   `failed` with the worker's result payload.

### Systemd

1. Install on a Linux host with Docker and systemd.
2. Run `sudo tachyon --tenant-id tn_xxxx worker start`.
3. Confirm `sudo tachyon worker status` is active.
4. Create a Coding Job with provider `containerized_codex`.
5. Watch `sudo tachyon worker logs --follow` for claim and completion.
6. Fetch the Coding Job from the API and confirm `status` is `succeeded` or
   `failed` with the worker's result payload.

## Security Notes

- The worker has no direct dependency on Tachyon backend crates.
- The service stores only a worker-scoped API key, not the user's OAuth profile.
- The systemd unit runs as the original `sudo` user when available.
- Job containers are labelled with `tachyon.worker.managed=true` and receive
  only the job environment plus explicitly supported host secrets.
- The default provider list is intentionally narrow. Additional providers should
  require an explicit login or setup flow before being enabled.
