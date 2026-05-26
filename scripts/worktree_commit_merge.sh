#!/usr/bin/env bash

set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
MAIN_BRANCH="${1:-main}"

die() {
  echo "[$SCRIPT_NAME] ERROR: $*" >&2
  exit 1
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "Required command not found: $1"
  fi
}

require_cmd git

export GIT_PAGER=cat
export GIT_MERGE_AUTOEDIT=no
export GIT_EDITOR=true

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

MERGED_BRANCHES=()
FAILED_MERGE_BRANCHES=()
FAILED_COMMIT_BRANCHES=()

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

  echo "[$SCRIPT_NAME] Processing $worktree_path ($branch_name)"
  git -C "$worktree_path" add .

  if git -C "$worktree_path" diff --cached --quiet; then
    echo "[$SCRIPT_NAME] No staged changes after add, skip commit: $branch_name"
  else
    if ! git -C "$worktree_path" commit -m "提交 $branch_name"; then
      echo "[$SCRIPT_NAME] Commit failed: $branch_name"
      FAILED_COMMIT_BRANCHES+=("$branch_name")
      continue
    fi
  fi

  git -C "$MAIN_WORKTREE" checkout "$MAIN_BRANCH"
  if ! git -C "$MAIN_WORKTREE" show-ref --verify --quiet "refs/heads/$branch_name"; then
    echo "[$SCRIPT_NAME] WARNING: Branch not found in main worktree, skip merge: $branch_name"
    continue
  fi
  if git -C "$MAIN_WORKTREE" merge-base --is-ancestor "$branch_name" "$MAIN_BRANCH"; then
    echo "[$SCRIPT_NAME] Already merged, skip merge: $branch_name"
    continue
  fi
  if git -C "$MAIN_WORKTREE" merge --no-edit --no-stat "$branch_name"; then
    echo "[$SCRIPT_NAME] Successfully merged: $branch_name"
    MERGED_BRANCHES+=("$branch_name")
  else
    echo "[$SCRIPT_NAME] Merge failed: $branch_name"
    FAILED_MERGE_BRANCHES+=("$branch_name")
    if ! git -C "$MAIN_WORKTREE" merge --abort >/dev/null 2>&1; then
      git -C "$MAIN_WORKTREE" reset --merge >/dev/null 2>&1 || true
    fi
    # 合并失败，清理状态并继续处理下一个
    echo "[$SCRIPT_NAME] WARNING: Skipping branch '$branch_name' due to merge failure."
    continue
  fi
done

git -C "$MAIN_WORKTREE" checkout "$MAIN_BRANCH"

echo "[$SCRIPT_NAME] Merged branches: ${#MERGED_BRANCHES[@]}"
for branch_name in "${MERGED_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   merged: $branch_name"
done

echo "[$SCRIPT_NAME] Failed commit branches: ${#FAILED_COMMIT_BRANCHES[@]}"
for branch_name in "${FAILED_COMMIT_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   commit_failed: $branch_name"
done

echo "[$SCRIPT_NAME] Failed merge branches: ${#FAILED_MERGE_BRANCHES[@]}"
for branch_name in "${FAILED_MERGE_BRANCHES[@]:-}"; do
  echo "[$SCRIPT_NAME]   merge_failed: $branch_name"
done

if ((${#FAILED_COMMIT_BRANCHES[@]} > 0 || ${#FAILED_MERGE_BRANCHES[@]} > 0)); then
  exit 1
fi

echo "[$SCRIPT_NAME] Done"
