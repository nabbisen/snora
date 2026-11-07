#!/usr/bin/env bash
set -euo pipefail

# RFC-116 Unit 2b: plans a `cargo publish --workspace` so that it can be
# run when some of the workspace's crates are already on crates.io.
#
# THE RULE
#
#   A crate is skipped if, and only if, its exact `name@version` is
#   already on crates.io. A crate that is there at some OTHER version is
#   not skipped: this tag has a version of it to publish, and the version
#   it has not got is exactly what the release is for.
#
#   If crates.io cannot be asked, or answers anything other than "yes,
#   that version exists" (200) or "no, it does not" (404), this script
#   FAILS and prints no plan. A guess in either direction is wrong in a
#   way that matters: "publish all" fails on the crate that exists,
#   "skip all" tags a release with nothing published.
#
# WHY IT EXISTS
#
#   A new workspace crate cannot have a Trusted Publisher configured
#   until it exists on crates.io, so its first version is published from
#   the owner's machine before the tag is pushed (RFC-116, ruling of
#   2026-11-07). `cargo publish --workspace` would then fail on that
#   crate. The same skip makes a re-run after a partly failed publish
#   safe. It is general, not a one-off exclusion.
#
# USAGE
#
#   scripts/publish-plan.sh <version>
#
#   stdout:  the flags to hand to `cargo publish --workspace`, on one
#            line: `--exclude snora-core --exclude snora-design ...`
#            (an empty line when nothing is excluded).
#   stderr:  one line per publishable crate, saying whether it is
#            skipped or published and why. This is the log the release
#            run keeps.
#   $GITHUB_OUTPUT, when set, receives `excludes=<the flags>` and
#            `remaining=<how many crates are left to publish>`, so the
#            workflow can skip the publish step when `remaining` is 0.
#
#   SNORA_PUBLISH_PLAN_REGISTRY overrides the registry's API base, for
#   testing the failure paths against a local server. It is not read in
#   the release workflow.
#
# Exit codes:
#   0  a plan was printed
#   1  the plan could not be made (bad argument, registry unreachable or
#      answering something unexpected)

cd "$(git rev-parse --show-toplevel)"

VERSION="${1:?usage: publish-plan.sh <version>}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "REFUSED: '$VERSION' is not an X.Y.Z version" >&2
  exit 1
fi

REGISTRY="${SNORA_PUBLISH_PLAN_REGISTRY:-https://crates.io/api/v1/crates}"
# crates.io requires crawlers to identify themselves. This names the
# project and nobody's address.
AGENT="snora-release-plan (https://github.com/nabbisen/snora)"

# The crates `cargo publish --workspace` would publish: every workspace
# member whose `publish` is unset or lists a registry. (`publish = false`
# is `[]` in the metadata, which is how the examples and probes opt out.)
mapfile -t CRATES < <(
  cargo metadata --no-deps --format-version 1 | python3 -I -c '
import json, sys
for p in json.load(sys.stdin)["packages"]:
    if p["publish"] is None or len(p["publish"]) > 0:
        print(p["name"])
' | sort
)

if [[ ${#CRATES[@]} -eq 0 ]]; then
  echo "REFUSED: found no publishable crate in the workspace" >&2
  exit 1
fi

EXCLUDES=()
REMAINING=0

for name in "${CRATES[@]}"; do
  # `-w` so a curl failure (no route, DNS, TLS) is distinguishable from
  # an HTTP answer: curl prints 000 and exits non-zero.
  status=$(curl -sS -o /dev/null -w '%{http_code}' --retry 3 --retry-all-errors \
    --max-time 30 -A "$AGENT" "$REGISTRY/$name/$VERSION" || true)
  case "$status" in
    200)
      echo "skip     $name@$VERSION: already on crates.io" >&2
      EXCLUDES+=(--exclude "$name")
      ;;
    404)
      echo "publish  $name@$VERSION: not on crates.io" >&2
      REMAINING=$((REMAINING + 1))
      ;;
    *)
      echo "REFUSED: asking crates.io about $name@$VERSION gave '$status', not 200 or 404." >&2
      echo "         No plan is printed: guessing would either fail the publish or tag a release with nothing published." >&2
      exit 1
      ;;
  esac
done

FLAGS="${EXCLUDES[*]:-}"
echo "$FLAGS"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "excludes=$FLAGS"
    echo "remaining=$REMAINING"
  } >> "$GITHUB_OUTPUT"
fi

if [[ $REMAINING -eq 0 ]]; then
  echo "nothing left to publish: every crate is already on crates.io at $VERSION" >&2
fi
