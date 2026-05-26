#!/usr/bin/env bash

set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
MAIN_BRANCH="${1:-main}"
CLEAN_UNTRACKED="${2:-true}"

die() {
  echo "[$SCRIPT_NAME] ERROR: $*" >&2
  exit 1
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "Required command not found: $1"
  fi
}

normalize_bool() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

require_cmd git

export GIT_PAGER=cat

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  die "Current directory is not inside a git repository"
fi

MAIN_WORKTREE="$(git rev-parse --show-toplevel)"

if ! git -C "$MAIN_WORKTREE" show-ref --verify --quiet "refs/heads/$MAIN_BRANCH"; then
  die "Main branch not found: $MAIN_BRANCH"
fi

WORKTREE_PATHS=()
while IFS= read -r worktree_path; do
  if [[ -n "$worktree_path" ]]; then
    WORKTREE_PATHS+=("$worktree_path")
  fi
done <<EOF
$(git -C "$MAIN_WORKTREE" worktree list --porcelain | awk '/^worktree /{print substr($0,10)}')
EOF

if ((${#WORKTREE_PATHS[@]} == 0)); then
  die "No worktree found"
fi

CLEAN_UNTRACKED_NORMALIZED="$(normalize_bool "$CLEAN_UNTRACKED")"

RESET_BRANCHES=()
SKIPPED_BRANCHES=()
FAILED_BRANCHES=()

git -C "$MAIN_WORKTREE" checkout "$MAIN_BRANCH"

for worktree_path in "${WORKTREE_PATHS[@]}"; do
  if [[ "$worktree_path" == "$MAIN_WORKTREE" ]]; then
    continue
  fi

  if [[ ! -d "$worktree_path" ]]; then
    echo "[$SCRIPT_NAME] Skip missing worktree: $worktree_path"
    continue
  fi

  branch_name="$(git -C "$worktree_path" rev-parse --abbrev-ref HEAD)"
  if [[ "$branch_name" == "HEAD" || -z "$branch_name" ]]; then
    echo "[$SCRIPT_NAME] Skip detached HEAD worktree: $worktree_path"
    continue
  fi

  if [[ "$branch_name" == "$MAIN_BRANCH" ]]; then
    echo "[$SCRIPT_NAME] Skip main branch worktree: $worktree_path"
    SKIPPED_BRANCHES+=("$branch_name")
    continue
  fi

  echo "[$SCRIPT_NAME] Resetting $worktree_path ($branch_name) to $MAIN_BRANCH"
  if git -C "$worktree_path" reset --hard "$MAIN_BRANCH"; then
    if [[ "$CLEAN_UNTRACKED_NORMALIZED" == "1" || "$CLEAN_UNTRACKED_NORMALIZED" == "true" || "$CLEAN_UNTRACKED_NORMALIZED" == "yes" ]]; then
      git -C "$worktree_path" clean -fd
    fi
    RESET_BRANCHES+=("$branch_name")
  else
    echo "[$SCRIPT_NAME] Reset failed: $branch_name"
    FAILED_BRANCHES+=("$branch_name")
  fi
done

git -C "$MAIN_WORKTREE" checkout "$MAIN_BRANCH"

echo "[$SCRIPT_NAME] Reset branches: ${#RESET_BRANCHES[@]}"
for branch_name in "${RESET_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   reset: $branch_name"
done

echo "[$SCRIPT_NAME] Skipped branches: ${#SKIPPED_BRANCHES[@]}"
for branch_name in "${SKIPPED_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   skipped: $branch_name"
done

echo "[$SCRIPT_NAME] Failed branches: ${#FAILED_BRANCHES[@]}"
for branch_name in "${FAILED_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   failed: $branch_name"
done

if ((${#FAILED_BRANCHES[@]} > 0)); then
  exit 1
fi

echo "[$SCRIPT_NAME] Done"
