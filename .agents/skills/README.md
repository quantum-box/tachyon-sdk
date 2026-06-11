# Tachyon Agent Skills

This directory contains distributable agent skills for Tachyon workflows.

## Install

Install the bundled skills interactively:

```bash
tachyon skills install
```

Install into Codex user scope without prompts:

```bash
tachyon skills install tachyon-cloud --codex --scope user --non-interactive
```

Install into Codex workspace scope without prompts:

```bash
tachyon skills install tachyon-cloud --codex --scope workspace --non-interactive
```

## Skills

| Skill | Purpose |
| --- | --- |
| `tachyon-cloud` | Work with Tachyon Cloud Apps, `tachyon.yml`, env vars, builds, logs, deployments, and user feedback reports through the Tachyon CLI. |

After installing, restart the agent runtime if it only scans skills at startup.
