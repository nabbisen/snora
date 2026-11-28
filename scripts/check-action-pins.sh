#!/usr/bin/env bash
set -euo pipefail

# RFC-140 R-1, R-4 and R-5. Two rules over the workflow files, so that the third-party code the release path runs cannot move
# under a tag or a branch, and so that no workflow holds more authority than it states:
#
#   1. Every `uses:` is `owner/repo[/path]@<40 hex characters>` (a full commit SHA), GitHub's own actions included, **followed by a
#      comment** (`# v6.1.0`) that names the release it was resolved from. A local `./` action is the one exception.
#   2. Every workflow declares `permissions:` at the top level.
#
# The comment is required because a bare SHA cannot be reviewed (nobody can tell what it is, or whether a bump is the next
# release), and because Dependabot reads the comment on the `uses:` line and rewrites it with the pin (dependabot-core,
# `github_actions/.../workflow_updater.rb`). A pin with no comment is a pin nobody will keep current by reading.
#
# Usage: scripts/check-action-pins.sh [directory]     (default: .github/workflows)
# Exit:  0 all conform; 1 a violation (each is printed with file:line); 2 usage or nothing to check.

cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DIR="${1:-.github/workflows}"
shopt -s nullglob
files=("$DIR"/*.yaml "$DIR"/*.yml)
[ "${#files[@]}" -gt 0 ] || { echo "check-action-pins.sh: no workflow files in $DIR: the glob stopped reading" >&2; exit 2; }

bad=0
uses_seen=0
for f in "${files[@]}"; do
  # Rule 2: a top-level `permissions:` key (column 0).
  if ! grep -qE '^permissions:' "$f"; then
    echo "REFUSED: $f has no top-level \`permissions:\` (state \`contents: read\`, or what the workflow needs)" >&2
    bad=1
  fi
  # Rule 1: every `uses:` line that is not itself a comment.
  while IFS= read -r hit; do
    n=${hit%%:*}; line=${hit#*:}
    uses_seen=$((uses_seen + 1))
    # the value after `uses:`, with a leading `- ` and quotes removed, and the trailing comment split off
    value=$(printf '%s' "$line" | sed -E 's/^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]*//')
    comment=""
    case "$value" in *' #'*) comment=${value#*' #'}; value=${value%%' #'*};; esac
    value=$(printf '%s' "$value" | sed -E "s/^[\"']//; s/[\"'][[:space:]]*$//; s/[[:space:]]+$//")
    case "$value" in ./*) continue;; esac
    if ! printf '%s' "$value" | grep -qE '^[A-Za-z0-9._-]+/[A-Za-z0-9._/-]+@[0-9a-f]{40}$'; then
      echo "REFUSED: $f:$n: \`uses: $value\` is not pinned to a full 40-character commit SHA" >&2
      bad=1
    elif ! printf '%s' "$comment" | grep -qE '[^[:space:]]'; then
      echo "REFUSED: $f:$n: \`uses: $value\` has no trailing comment naming the release it was resolved from (# vX.Y.Z)" >&2
      bad=1
    fi
  done < <(grep -nE '^[[:space:]]*(-[[:space:]]+)?uses:' "$f" || true)
done

[ "$uses_seen" -gt 0 ] || { echo "check-action-pins.sh: found no \`uses:\` line at all in $DIR: the pattern stopped reading" >&2; exit 2; }
if [ "$bad" -eq 0 ]; then
  echo "check-action-pins.sh: $uses_seen \`uses:\` lines in ${#files[@]} workflows: every one is a full commit SHA with a release comment, and every workflow declares \`permissions:\`"
fi
exit "$bad"
