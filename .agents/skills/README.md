# Tachyon Agent Skills

This directory contains distributable agent skills for Tachyon workflows.

## Install

Install the bundled skills into the default agent skill directory:

```bash
tachyon skills install
```

Install into a custom directory:

```bash
tachyon skills install --target-dir "$HOME/.custom-agent/skills"
```

Install into Codex:

```bash
tachyon skills install --codex
```

## Skills

| Skill | Purpose |
| --- | --- |
| `tachyon-cloud` | Work with Tachyon Cloud Apps, `tachyon.yml`, env vars, builds, logs, deployments, and user feedback reports through the Tachyon CLI. |

After installing, restart the agent runtime if it only scans skills at startup.
