#!/usr/bin/env bash
set -euo pipefail

# RFC-107. Prints one version's changelog entry as a GitHub Release
# body. `release.yaml` pipes this into `gh release create`, so a reader
# who lands on the Releases page sees the same notes as a reader of the
# changelog — the thing that until now existed only in the repository.
#
# Three differences from the entry as it sits in the file, each because
# a release body is not a file in the repository:
#
#   1. the `## [x.y.z] — date` header is dropped, because the Release's
#      own title carries the version;
#   2. link reference definitions are dropped: nothing in the body uses
#      them, and a trailing block of bare URLs reads as debris;
#   3. relative links are rewritten to absolute `blob/<version>` URLs,
#      because a release body has no directory to resolve them against.
#      They point at the tag, not at `main`, so a link in an old
#      release still shows what it meant at the time.
#
# Usage: scripts/extract-release-notes.sh <version>
#        scripts/extract-release-notes.sh 0.52.0 > notes.md

cd "$(git rev-parse --show-toplevel)"

readonly REPO_URL="https://github.com/nabbisen/snora"

if (($# != 1)); then
  echo "usage: scripts/extract-release-notes.sh <version>" >&2
  exit 2
fi

readonly VERSION=$1

# The file that holds this version's entry: the current series file, or
# one of the closed series files.
holder=""
for file in CHANGELOG.md changelog/*.md; do
  [[ -e "$file" ]] || continue
  if grep -qF "## [$VERSION]" "$file"; then
    holder=$file
    break
  fi
done

if [[ -z "$holder" ]]; then
  echo "no changelog entry for $VERSION in CHANGELOG.md or changelog/*.md" >&2
  exit 1
fi

directory=$(dirname "$holder")

# The entry: everything after its header, up to the next one. The awk
# reads to end of input rather than exiting at the next header, so that
# nothing upstream is left writing into a closed pipe.
body=$(
  awk -v want="## [$VERSION]" '
    index($0, want) == 1 { inside = 1; next }
    inside && index($0, "## [") == 1 { inside = 0 }
    inside { print }
  ' "$holder" | sed '/^\[[^]]*\]: /d'
)

# Rewrite each distinct relative target once, by exact string, so no
# regular expression has to be escaped against paths containing dots.
while IFS= read -r target; do
  [[ -z "$target" ]] && continue
  case "$target" in
    http://* | https://* | '#'*) continue ;;
  esac

  path=${target%%#*}
  fragment=${target#"$path"}
  [[ -z "$path" ]] && continue

  # Resolve from the holding file's own directory to a repository path:
  # `../docs/x.md` in `changelog/` and `docs/x.md` in the root are the
  # same file, and the Release body needs the repository's spelling.
  repo_path=$(realpath -m --relative-to="$PWD" "$directory/$path")
  body=${body//"]($target)"/"]($REPO_URL/blob/$VERSION/$repo_path$fragment)"}
done < <(printf '%s\n' "$body" | grep -oE '\]\([^)]+\)' | sed -E 's/^\]\((.*)\)$/\1/' | sort -u)

# Trim the blank lines the header left behind, and any trailing ones.
printf '%s\n' "$body" | awk '
  NF { blank = 0 }
  !NF { blank++ }
  NF && !started { started = 1 }
  started && NF { for (i = 0; i < pending; i++) print ""; pending = 0; print; next }
  started && !NF { pending++ }
'
