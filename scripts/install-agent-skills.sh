#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${repo_root}/.agents/skills"
target_dir="${HOME}/.agents/skills"

usage() {
  cat <<'USAGE'
Usage: install-agent-skills.sh [OPTIONS]

Install Tachyon agent skills.

Options:
  --agents              Install to ~/.agents/skills (default)
  --codex               Install to ~/.codex/skills
  --claude              Install to ~/.claude/skills
  --target-dir <dir>    Install to a custom skill directory
  -h, --help            Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --agents)
      target_dir="${HOME}/.agents/skills"
      shift
      ;;
    --codex)
      target_dir="${HOME}/.codex/skills"
      shift
      ;;
    --claude)
      target_dir="${HOME}/.claude/skills"
      shift
      ;;
    --target-dir)
      if [[ $# -lt 2 || -z "$2" ]]; then
        echo "--target-dir requires a directory" >&2
        exit 1
      fi
      target_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ! -d "${source_dir}" ]]; then
  echo "No agent skills found at ${source_dir}" >&2
  exit 1
fi

mkdir -p "${target_dir}"

if command -v git >/dev/null 2>&1 && git -C "${repo_root}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  skill_dirs="$(
    git -C "${repo_root}" ls-files ".agents/skills/*/SKILL.md" |
      sed -E 's#/SKILL\.md$##' |
      sort -u
  )"
else
  skill_dirs="$(find "${source_dir}" -mindepth 1 -maxdepth 1 -type d | sort)"
fi

printf '%s\n' "${skill_dirs}" | while IFS= read -r skill_dir; do
  [[ -n "${skill_dir}" ]] || continue
  [[ "${skill_dir}" = /* ]] || skill_dir="${repo_root}/${skill_dir}"
  [[ -d "${skill_dir}" ]] || continue
  skill_name="$(basename "${skill_dir}")"
  rm -rf "${target_dir}/${skill_name}"
  mkdir -p "${target_dir}/${skill_name}"
  cp -R "${skill_dir}/." "${target_dir}/${skill_name}/"
  echo "Installed ${skill_name} -> ${target_dir}/${skill_name}"
done

echo "Agent skills installed in ${target_dir}"
