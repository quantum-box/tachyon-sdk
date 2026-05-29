# Tachyon Agent Skills

This directory contains distributable agent skills for Tachyon workflows.

## Install

Install the bundled skills into the default agent skill directory:

```bash
./scripts/install-agent-skills.sh
```

Install into a custom directory:

```bash
TACHYON_AGENT_SKILLS_DIR="$HOME/.codex/skills" ./scripts/install-agent-skills.sh
```

## Skills

| Skill | Purpose |
| --- | --- |
| `cloud-app-deploy` | Work with Tachyon Cloud Apps, `tachyon.yml`, env vars, builds, logs, deployments, and user feedback reports through the Tachyon CLI. |

After installing, restart the agent runtime if it only scans skills at startup.
