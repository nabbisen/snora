#!/usr/bin/env bash
set -euo pipefail

# RFC-107. The changelog is what a consumer reads to decide whether an
# upgrade costs them anything, and until now nothing checked it: entries
# were edited after release, links rotted, and the series split could
# have silently dropped or duplicated an entry. This script is the
# check. Seven rules, each one a thing that has actually gone wrong or
# that the split made possible:
#
#   1. every entry has a compare link, every endpoint is a real tag (or
#      the pending version — see below), and the left endpoint is the
#      entry immediately before it;
#   2. every relative link resolves from its own file's directory;
#   3. every minor from 0.39 carries its migration-guide line;
#   4. entries from 0.53.0 use the agreed headings and the four-label
#      lead (Q-1);
#   5. a released entry is frozen: it still says what it said at its own
#      tag, modulo the additions RFC-107 permits;
#   6. each series file holds its own series, once;
#   7. a file over 256 KB warns, so the next split is noticed before a
#      reader has to scroll through it.
#
# Rule 7 warns. Everything else fails.
#
# One subtlety runs through rules 1 and 6: **the pending version**. The
# release checklist has the architect commit the new entry, its compare
# link and a repointed `[Unreleased]` *before* the tag exists, and
# `release.yaml` will not publish unless that very commit has a green CI
# run. A gate that only knew about tags would fail that commit, so no
# release could ever publish — the gate would deadlock the thing it
# exists to protect. So the version in `Cargo.toml`, while it has no tag,
# is a legitimate endpoint and a legitimate series, and nowhere else is.
#
# Usage: scripts/check-changelog.sh

cd "$(git rev-parse --show-toplevel)"

readonly REPO_URL="https://github.com/nabbisen/snora"
readonly CURRENT_FILE="CHANGELOG.md"
readonly SIZE_WARN_BYTES=$((256 * 1024))

# Rule 3 applies from this minor: the first release with a migration
# guide line (RFC-079's adoption point).
readonly FIRST_GUIDE_MINOR=39

# Rule 4 applies from this minor's .0 and everything after it: the lead
# labels are RFC-107's own change, and entries written before it cannot
# have them.
readonly LEAD_FROM_MINOR=53

# Rule 5's exceptions: entries that differ from their own tag for a
# reason that was decided and recorded. Each is `version|commits|reason`.
# Three of them were revised after release and carry a `*Corrected ...*`
# line; for those the line's presence is itself asserted below, so an
# exception cannot outlive its disclosure.
readonly FROZEN_EXCEPTIONS=(
  "0.7.0|0a26eb8, 1c52464|link paths repaired"
  "0.8.0|1c52464|file layout repaired: 0.7.0 text had sat inside 0.8.0's section"
  "0.19.1|bd215f8|revised after release; corrected line present"
  "0.20.0|d8d3f9d|revised after release; corrected line present"
  "0.25.2|6642b94|no entry at the tag at all; corrected line present"
)
# The subset that must carry a disclosure line.
readonly CORRECTED_EXCEPTIONS=(0.19.1 0.20.0 0.25.2)

failures=()
warnings=()

fail() { failures+=("FAIL $1: $2"); }
warn() { warnings+=("WARN $1: $2"); }

# Every file that holds entries, current series first.
changelog_files() {
  printf '%s\n' "$CURRENT_FILE"
  # `changelog/` exists today; if it ever does not, the glob must not
  # expand to a literal path.
  for file in changelog/*.md; do
    [[ -e "$file" ]] && printf '%s\n' "$file"
  done
}

# Tags that should have an entry. History begins at 0.7.0, so a tag with
# a minor below 7 is from before the changelog existed.
entry_tags() {
  git tag -l '0.*' | awk -F. '($2 + 0) >= 7' | sort -V
}

# One version's entry from a file on stdin: everything after its header,
# up to the next header. The header line itself is not included.
entry_body() {
  # Reads to end of input on purpose: exiting early would SIGPIPE
  # whatever is feeding the pipe, and this script runs under
  # `set -o pipefail`.
  awk -v want="## [$1]" '
    index($0, want) == 1 { inside = 1; next }
    inside && index($0, "## [") == 1 { inside = 0 }
    inside { print }
  '
}

# Rule 5's normalisation: the differences RFC-107 permits between an
# entry as released and the same entry today.
normalise_entry() {
  sed -e '/^[[:space:]]*$/d' \
    -e '/^\[[^]]*\]: /d' \
    -e '/^\*\*Upgrading from /d' \
    -e '/^\*Corrected /d' \
    -e 's|](\.\./docs/|](docs/|g'
}

# The series file stem a version belongs to: 0.7-0.9 are "0.0x", then
# by tens — 0.4x holds 0.40 to 0.49.
series_of() {
  local minor=${1#0.}
  minor=${minor%%.*}
  printf '0.%dx' $((minor / 10))
}

# ---------------------------------------------------------------------
# Preconditions
# ---------------------------------------------------------------------

# A clone made with `--no-tags`, or a CI checkout that fetched none,
# would make rules 1, 5 and 6 vacuous. RFC-087's D-1 was exactly this:
# a gate that passed because it had nothing to check. Fail instead.
if [[ -z "$(git tag -l '0.*')" ]]; then
  echo "FAIL preconditions: no 0.* tags in this clone." >&2
  echo "  Rules 1, 5 and 6 compare entries against their tags and would" >&2
  echo "  pass vacuously. Fetch tags (CI: actions/checkout with" >&2
  echo "  fetch-depth: 0) and run again." >&2
  exit 1
fi

# The pending version: `[workspace.package].version` when no tag of that
# name exists yet — the state a release commit is in. Read exactly as
# `check-tag-matches-version.sh` reads it, so the two gates cannot
# disagree about what version this tree claims to be.
cargo_version=$(grep -m1 '^version = "[0-9]\+\.[0-9]\+\.[0-9]\+"' Cargo.toml |
  sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/' || true)

if [[ -z "$cargo_version" ]]; then
  echo "FAIL preconditions: could not read [workspace.package].version from Cargo.toml." >&2
  echo "  Rules 1 and 6 need it to recognise a release commit made before" >&2
  echo "  its tag. Refusing rather than guessing." >&2
  exit 1
fi

# Empty once the version is tagged, which is every commit but a release
# commit. `PENDING` is the only version that may stand in for a tag.
PENDING=""
git rev-parse -q --verify "refs/tags/$cargo_version" >/dev/null || PENDING=$cargo_version
readonly PENDING

mapfile -t FILES < <(changelog_files)
mapfile -t TAGS < <(entry_tags)

# version -> file that holds its entry, and the headers in file order.
declare -A ENTRY_FILE=()
declare -A ENTRY_LINE=()
duplicate_versions=()

for file in "${FILES[@]}"; do
  while IFS=: read -r lineno version; do
    if [[ -n "${ENTRY_FILE[$version]:-}" ]]; then
      duplicate_versions+=("$version (${ENTRY_FILE[$version]} and $file)")
    else
      ENTRY_FILE[$version]=$file
      ENTRY_LINE[$version]=$lineno
    fi
  done < <(grep -nE '^## \[[^]]+\]' "$file" | sed -E 's/^([0-9]+):## \[([^]]+)\].*/\1:\2/')
done

# ---------------------------------------------------------------------
# Rule 1 — compare links
# ---------------------------------------------------------------------

for version in "${!ENTRY_FILE[@]}"; do
  file=${ENTRY_FILE[$version]}
  reference=$(grep -E "^\[${version//./\\.}\]: " "$file" || true)
  if [[ -z "$reference" ]]; then
    fail rule-1 "$file:${ENTRY_LINE[$version]} [$version] has no link reference in this file"
    continue
  fi

  url=${reference#*: }
  case "$url" in
    "$REPO_URL"/compare/*)
      range=${url##*/compare/}
      left=${range%%...*}
      right=${range##*...}
      for endpoint in "$left" "$right"; do
        if [[ "$endpoint" == "HEAD" ]]; then
          if [[ "$version" != "Unreleased" ]]; then
            fail rule-1 "$file: [$version] compares against HEAD; only [Unreleased] may"
          fi
        elif [[ -n "$PENDING" && "$endpoint" == "$PENDING" ]]; then
          # The pending version is a real endpoint in exactly two
          # places: its own entry's link, and [Unreleased]. Anywhere
          # else it is a version nobody has released being cited as
          # though it had been.
          if [[ "$version" != "$PENDING" && "$version" != "Unreleased" ]]; then
            fail rule-1 "$file: [$version] compares against the pending version '$PENDING'; only [$PENDING] and [Unreleased] may"
          fi
        elif ! git rev-parse -q --verify "refs/tags/$endpoint" >/dev/null; then
          fail rule-1 "$file: [$version]'s compare endpoint '$endpoint' is not a tag"
        fi
      done
      ;;
    "$REPO_URL"/releases/tag/*)
      # The first entry has no predecessor to compare against.
      tag=${url##*/releases/tag/}
      git rev-parse -q --verify "refs/tags/$tag" >/dev/null ||
        fail rule-1 "$file: [$version] links release tag '$tag', which does not exist"
      ;;
    *)
      fail rule-1 "$file: [$version]'s link is neither a compare nor a release-tag URL: $url"
      ;;
  esac
done

# Both endpoints being real is not enough: `[0.51.0]: …/compare/
# 0.40.0...0.51.0` names two real tags and still describes the wrong
# range. The left one must be the entry immediately before this one
# (O-1). True of all 66 entries today, so it costs nothing to keep true.
mapfile -t ORDERED < <(
  for version in "${!ENTRY_FILE[@]}"; do
    [[ "$version" == "Unreleased" ]] || printf '%s\n' "$version"
  done | sort -V
)

for index in "${!ORDERED[@]}"; do
  # The first entry has no predecessor; it links its release tag
  # instead, which the case above already checked.
  ((index > 0)) || continue
  version=${ORDERED[index]}
  previous=${ORDERED[index - 1]}
  file=${ENTRY_FILE[$version]}
  reference=$(grep -E "^\[${version//./\\.}\]: " "$file" || true)
  [[ "$reference" == *"/compare/"* ]] || continue
  left=${reference##*/compare/}
  left=${left%%...*}
  [[ "$left" == "$previous" ]] ||
    fail rule-1 "$file: [$version] compares from $left, but the entry before it is $previous"
done

# [Unreleased] lives in the current file and compares from the most
# recent version the changelog claims — the pending one on a release
# commit, the newest tag at every other time.
newest_tag=$(printf '%s\n' "${TAGS[@]}" | tail -1)
unreleased_from=${PENDING:-$newest_tag}
if [[ "${ENTRY_FILE[Unreleased]:-}" != "$CURRENT_FILE" ]]; then
  fail rule-1 "$CURRENT_FILE: no [Unreleased] entry"
else
  expected="[Unreleased]: $REPO_URL/compare/$unreleased_from...HEAD"
  grep -qxF "$expected" "$CURRENT_FILE" ||
    fail rule-1 "$CURRENT_FILE: [Unreleased] should compare $unreleased_from...HEAD"
fi

# ---------------------------------------------------------------------
# Rule 2 — relative links resolve
# ---------------------------------------------------------------------

for file in "${FILES[@]}"; do
  directory=$(dirname "$file")
  # `grep -o` finds every inline link target on a line, not only the
  # first; `|| true` because a file with no links is not an error.
  while IFS=: read -r lineno target; do
    [[ -z "$target" ]] && continue
    case "$target" in
      http://* | https://* | '#'*) continue ;;
    esac
    # Strip any fragment: a heading anchor is not a path.
    path=${target%%#*}
    [[ -z "$path" ]] && continue
    [[ -e "$directory/$path" ]] ||
      fail rule-2 "$file:$lineno link target '$target' does not resolve from $directory/"
  done < <(grep -noE '\]\([^)]+\)' "$file" | sed -E 's/^([0-9]+):\]\((.*)\)$/\1:\2/' || true)
done

# ---------------------------------------------------------------------
# Rule 3 — every minor from 0.39 names its migration guide
# ---------------------------------------------------------------------

for version in "${!ENTRY_FILE[@]}"; do
  [[ "$version" == "Unreleased" ]] && continue
  [[ "$version" =~ ^0\.([0-9]+)\.0$ ]] || continue
  minor=${BASH_REMATCH[1]}
  ((minor >= FIRST_GUIDE_MINOR)) || continue

  file=${ENTRY_FILE[$version]}
  first_line=$(entry_body "$version" <"$file" | awk 'NF && !found { line = $0; found = 1 } END { print line }')
  previous=$((minor - 1))
  guide="docs/src/guides/migration-0.$previous-to-0.$minor.md"
  [[ "$file" == changelog/* ]] && guide="../$guide"
  expected="**Upgrading from 0.$previous:** read the [0.$previous → 0.$minor migration guide]($guide)."

  if [[ "$first_line" != "$expected" ]]; then
    fail rule-3 "$file: [$version]'s first line is not its navigation line"
    fail rule-3 "$file:   expected: $expected"
    fail rule-3 "$file:   found:    ${first_line:-<nothing>}"
  fi
done

# ---------------------------------------------------------------------
# Rule 4 — the agreed shape, for entries written under RFC-107
# ---------------------------------------------------------------------

readonly ALLOWED_HEADINGS=("Added" "Changed" "Deprecated" "Removed" "Fixed" "Security")
readonly LEAD_LABELS=("**In short:**" "**Code:**" "**Appearance:**" "**Security:**")

heading_is_allowed() {
  local heading=$1 allowed
  for allowed in "${ALLOWED_HEADINGS[@]}"; do
    [[ "$heading" == "$allowed" ]] && return 0
  done
  return 1
}

for version in "${!ENTRY_FILE[@]}"; do
  # [Unreleased] is exempt: its lead is written at the cut, when what
  # the release means is known.
  [[ "$version" == "Unreleased" ]] && continue
  [[ "$version" =~ ^0\.([0-9]+)\. ]] || continue
  ((BASH_REMATCH[1] >= LEAD_FROM_MINOR)) || continue

  file=${ENTRY_FILE[$version]}
  body=$(entry_body "$version" <"$file")

  while IFS= read -r heading; do
    [[ -z "$heading" ]] && continue
    heading_is_allowed "$heading" ||
      fail rule-4 "$file: [$version] has the heading '### $heading', which is not one of: ${ALLOWED_HEADINGS[*]}"
  done < <(printf '%s\n' "$body" | sed -nE 's/^### (.*)$/\1/p')

  # The lead is what a reader sees first, so it has to be before any
  # section heading, not buried under one.
  lead=$(printf '%s\n' "$body" | awk 'index($0, "### ") == 1 { past = 1 } !past { print }')
  for label in "${LEAD_LABELS[@]}"; do
    [[ "$lead" == *"$label"* ]] ||
      fail rule-4 "$file: [$version]'s lead is missing $label"
  done
done

# ---------------------------------------------------------------------
# Rule 5 — a released entry still says what it said at its tag
# ---------------------------------------------------------------------

# An exception stands only while its disclosure does: the three entries
# revised after release each carry a `*Corrected ...*` line, and losing
# the line would leave the edit unexplained while the exception kept it
# quiet.
require_disclosure() {
  local tag=$1 file=$2 reason=$3 disclosed disclosure
  for disclosed in "${CORRECTED_EXCEPTIONS[@]}"; do
    [[ "$disclosed" == "$tag" ]] || continue
    disclosure=$(entry_body "$tag" <"$file" | sed -n '/^\*Corrected /p')
    [[ -n "$disclosure" ]] ||
      fail rule-5 "tag $tag is an allowed exception ($reason) but its *Corrected line is gone — an exception may not outlive its disclosure"
  done
}

exception_reason() {
  local version=$1 row
  for row in "${FROZEN_EXCEPTIONS[@]}"; do
    [[ "${row%%|*}" == "$version" ]] && { printf '%s' "${row##*|}"; return 0; }
  done
  return 1
}

for tag in "${TAGS[@]}"; do
  file=${ENTRY_FILE[$tag]:-}
  if [[ -z "$file" ]]; then
    fail rule-5 "tag $tag has no entry in any changelog file"
    continue
  fi

  current=$(entry_body "$tag" <"$file" | normalise_entry)

  # One `git show` per tag: the entry as released lived in CHANGELOG.md
  # until the series split, and in its series file after one.
  #
  # "Found but empty" is not "not found", so the header's presence
  # decides, not the body's length. (0.25.2 is neither: at its tag the
  # entry is absent altogether. That case is handled just below.)
  released_source=$(git show "$tag:$CURRENT_FILE" 2>/dev/null || true)
  # A bash pattern, not `grep -q`: a `grep -q` on a pipe exits at the
  # first match and SIGPIPEs whatever feeds it, which under
  # `set -o pipefail` reads as failure — and only for inputs large
  # enough that the writer has not already finished.
  if [[ "$released_source" != *"## [$tag]"* ]]; then
    series_file="changelog/$(series_of "$tag").md"
    released_source=$(git show "$tag:$series_file" 2>/dev/null || true)
  fi

  if [[ "$released_source" != *"## [$tag]"* ]]; then
    # No entry at the tag at all. That is a difference from the entry
    # today, so a listed exception covers it — 0.25.2 shipped this way,
    # with no entry whatsoever. Its text was written in 6642b94 on
    # 2026-08-02, six weeks after the 2026-06-21 tag and a day before
    # the next release (0.25.3). Anything unlisted is a failure.
    if reason=$(exception_reason "$tag"); then
      require_disclosure "$tag" "$file" "$reason"
    else
      fail rule-5 "tag $tag: no entry at the tag itself, in $CURRENT_FILE or its series file, and no listed exception"
    fi
    continue
  fi
  released=$(printf '%s\n' "$released_source" | entry_body "$tag" | normalise_entry)

  if [[ "$current" == "$released" ]]; then
    if reason=$(exception_reason "$tag"); then
      fail rule-5 "tag $tag now matches its tag, so its exception ($reason) is stale — remove it"
    fi
    continue
  fi

  if reason=$(exception_reason "$tag"); then
    require_disclosure "$tag" "$file" "$reason"
    continue
  fi

  fail rule-5 "tag $tag's entry in $file differs from the entry at the tag, and is not a listed exception"
done

# ---------------------------------------------------------------------
# Rule 6 — each series in its own file, once
# ---------------------------------------------------------------------

for duplicate in "${duplicate_versions[@]:-}"; do
  [[ -z "$duplicate" ]] && continue
  fail rule-6 "version appears in two files: $duplicate"
done

# The current series follows the pending version on a release commit:
# cutting 0.60.0 is what opens the 0.6x series, and the entry is written
# before the tag exists.
newest_series=$(series_of "${PENDING:-$newest_tag}")

# A bumped version with no entry is a release commit that forgot the
# roll — 0.50.0's d524406 was that shape. Rule 1 cannot see it, because
# it only looks at entries that exist.
if [[ -n "$PENDING" && "${ENTRY_FILE[$PENDING]:-}" != "$CURRENT_FILE" ]]; then
  fail rule-6 "Cargo.toml says $PENDING, which is untagged, but $CURRENT_FILE has no [$PENDING] entry"
fi

for version in "${!ENTRY_FILE[@]}"; do
  [[ "$version" == "Unreleased" ]] && continue
  file=${ENTRY_FILE[$version]}
  series=$(series_of "$version")

  if [[ "$file" == "$CURRENT_FILE" ]]; then
    [[ "$series" == "$newest_series" ]] ||
      fail rule-6 "$CURRENT_FILE holds [$version], which belongs to the closed series changelog/$series.md"
  else
    expected="changelog/$series.md"
    [[ "$file" == "$expected" ]] ||
      fail rule-6 "$file holds [$version], which belongs in $expected"
  fi
done

for tag in "${TAGS[@]}"; do
  [[ -n "${ENTRY_FILE[$tag]:-}" ]] || fail rule-6 "tag $tag has no changelog entry"
done

# ---------------------------------------------------------------------
# Rule 7 — size, as a warning
# ---------------------------------------------------------------------

for file in "${FILES[@]}"; do
  bytes=$(wc -c <"$file")
  ((bytes > SIZE_WARN_BYTES)) &&
    warn rule-7 "$file is $((bytes / 1024)) KB, over the $((SIZE_WARN_BYTES / 1024)) KB mark — consider closing the series"
done

# ---------------------------------------------------------------------

for warning in "${warnings[@]:-}"; do
  [[ -n "$warning" ]] && echo "$warning"
done

if ((${#failures[@]} > 0)); then
  printf '%s\n' "${failures[@]}" >&2
  echo >&2
  echo "${#failures[@]} changelog failure(s). Each rule is documented in this script's header." >&2
  exit 1
fi

echo "OK: ${#TAGS[@]} released entries across ${#FILES[@]} files; 7 rules checked."
