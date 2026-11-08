#!/usr/bin/env bash
set -euo pipefail

# Validates every file under .github/workflows/ with actionlint, pinned to
# an exact version. Runnable locally -- the whole point, after the
# incident this script exists for: `183cc70` added a step name containing
# an unquoted ": ", making the entire ci.yaml unparseable. GitHub created
# zero jobs for it; nobody could have caught this before pushing, because
# nobody on this project could parse a workflow file at all. RFC-090's own
# submission said so in writing a week earlier ("no YAML parser was
# available in this environment... no pyyaml, no js-yaml, no actionlint")
# and nothing acted on the gap until it cost a red main.
#
# actionlint over a bare YAML parser deliberately: a plain parse would
# have caught the incident above and little else. actionlint also catches
# unknown `runs-on` labels, bad `${{ }}` expressions, and invalid `needs:`
# references -- GitHub-specific mistakes no YAML-only tool can see.
#
# Usage: scripts/check-workflows.sh
#
# Resolution order:
#   1. `actionlint` already on PATH -- used as-is, whatever version.
#   2. Otherwise, uses the pinned version's release **archive** for this
#      machine's OS/arch, from target/actionlint-cache/ if it is there and
#      from GitHub releases if it is not (a few retries on a server error:
#      the release server answered 504 twice in two days, with nothing wrong
#      in the workflows). **The pinned SHA-256 of the archive is checked on
#      every run, cached or just downloaded, before anything is unpacked**;
#      the unpacked binary is made in a temporary directory and thrown away.
#
# Why the archive and not the binary is what is cached: a cached, already
# unpacked binary would run with no check at all, and so would a cache that
# had been poisoned. A cached archive that fails its checksum is deleted, said
# so loudly (a `::warning::` in CI), and replaced by a fresh download checked
# against the same pin; a fresh download that fails refuses. An
# `actionlint` left unpacked in the cache directory by an earlier version of
# this script is an unverified binary too; it is never run, and is removed.
#
# If neither is possible -- unsupported OS/arch, no network, checksum
# mismatch -- this script says so on stderr and exits non-zero. It does
# NOT silently report success with nothing actually checked: an
# unvalidatable tree reporting itself clean is the defect this project
# has now hit four times (RFC-087 D-1, RFC-086's alpha-blind assertion,
# RFC-088's silent set -e exit, RFC-090's own short-SHA gap).
#
# Exit codes:
#   0  actionlint ran and found nothing to report
#   1  actionlint is unavailable and could not be obtained, OR
#      actionlint ran and found problems

ACTIONLINT_VERSION="1.7.12"

# The release server's base URL. Overridable **only to test the failure paths
# against a local server** (a 5xx, a corrupt body); the workflow does not set it.
RELEASES_BASE="${SNORA_ACTIONLINT_RELEASES_BASE:-https://github.com/rhysd/actionlint/releases/download}"

# Pinned SHA-256 sums for this version's release assets, from
# https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_checksums.txt
# Only the OS/arch combinations this project's contributors and CI
# runner (ubuntu-latest = linux/amd64) actually use are listed. Add a
# line here (from that same checksums file) before adding support for
# another platform -- do not download unverified.
CHECKSUMS="
linux_amd64   8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8
linux_arm64   325e971b6ba9bfa504672e29be93c24981eeb1c07576d730e9f7c8805afff0c6
darwin_amd64  5b44c3bc2255115c9b69e30efc0fecdf498fdb63c5d58e17084fd5f16324c644
darwin_arm64  aba9ced2dee8d27fecca3dc7feb1a7f9a52caefa1eb46f3271ea66b6e0e6953f
"

cd "$(git rev-parse --show-toplevel)"

resolve_actionlint() {
  if command -v actionlint >/dev/null 2>&1; then
    command -v actionlint
    return 0
  fi

  local os arch platform
  case "$(uname -s)" in
    Linux) os="linux" ;;
    Darwin) os="darwin" ;;
    *)
      echo "error: no actionlint on PATH, and this OS ($(uname -s)) has no pinned download entry" >&2
      return 1
      ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64) arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)
      echo "error: no actionlint on PATH, and this architecture ($(uname -m)) has no pinned download entry" >&2
      return 1
      ;;
  esac
  platform="${os}_${arch}"

  local expected_sha
  expected_sha=$(echo "$CHECKSUMS" | awk -v p="$platform" '$1 == p { print $2 }')
  if [[ -z "$expected_sha" ]]; then
    echo "error: no actionlint on PATH, and no pinned checksum for $platform" >&2
    return 1
  fi

  local cache_dir="target/actionlint-cache/${ACTIONLINT_VERSION}"
  local archive="${cache_dir}/actionlint_${ACTIONLINT_VERSION}_${platform}.tar.gz"
  local url="${RELEASES_BASE}/v${ACTIONLINT_VERSION}/actionlint_${ACTIONLINT_VERSION}_${platform}.tar.gz"
  mkdir -p "$cache_dir"
  # An unpacked binary an earlier version of this script left here has never
  # been checked on this run: never run it.
  rm -f "${cache_dir}/actionlint"

  local actual_sha
  if [[ -f "$archive" ]]; then
    echo "using the cached actionlint archive (it is verified below, as every run)" >&2
    actual_sha=$(sha256sum "$archive" | cut -d' ' -f1)
    if [[ "$actual_sha" != "$expected_sha" ]]; then
      # A bad cache entry is immutable on GitHub, so refusing here would block
      # every later run until someone deleted it by hand. A fresh download is
      # exactly as trustworthy as the first one: it is checked against the same
      # pinned SHA-256. So: say so, loudly, and fetch afresh.
      local key="actionlint-archive-${ACTIONLINT_VERSION}-${os}-${arch}"
      local msg="the cached actionlint archive failed its pinned SHA-256 (expected $expected_sha, got $actual_sha). It was NOT used and has been deleted; a fresh verified download follows. Delete the cache entry '$key' (repository Actions caches) so the next run does not restore it again."
      if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
        # To the script's own stdout (fd 3), not this function's, whose stdout
        # is the path of the binary and nothing else.
        echo "::warning::$msg" >&3
      fi
      echo "WARNING: $msg" >&2
      rm -f "$archive"
    fi
  fi

  if [[ ! -f "$archive" ]]; then
    echo "no actionlint on PATH; downloading pinned v${ACTIONLINT_VERSION} for ${platform}..." >&2
    # Retries on any error, a 5xx included, and says so (no --silent: curl
    # reports each retry on stderr). Written to a .part file and renamed, so a
    # download that was cut off is never taken for a cached archive.
    if ! curl -fL --no-progress-meter --retry 4 --retry-all-errors --retry-delay 2 \
        --connect-timeout 20 --max-time 180 -o "${archive}.part" "$url"; then
      rm -f "${archive}.part"
      echo "error: failed to download actionlint from $url (after retries)" >&2
      return 1
    fi
    mv "${archive}.part" "$archive"
    actual_sha=$(sha256sum "$archive" | cut -d' ' -f1)
    if [[ "$actual_sha" != "$expected_sha" ]]; then
      echo "error: actionlint archive checksum mismatch -- expected $expected_sha, got $actual_sha" >&2
      echo "refusing to run an unverified binary; the archive is deleted" >&2
      rm -f "$archive"
      return 1
    fi
  fi

  tar -xzf "$archive" -C "$WORK" actionlint
  chmod +x "${WORK}/actionlint"
  echo "${WORK}/actionlint"
}

# The unpacked binary lives here for this run only.
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

exec 3>&1
ACTIONLINT_BIN=$(resolve_actionlint) || {
  echo "REFUSED: actionlint is unavailable and could not be obtained -- this tree is unvalidated, not clean." >&2
  exit 1
}

echo "Using actionlint: $ACTIONLINT_BIN ($("$ACTIONLINT_BIN" -version 2>&1 | head -1))"
# No file arguments: actionlint's own documented default is to discover
# every .github/workflows/*.yml|*.yaml under the current directory, which
# is why `cd` to the repo root above matters.
"$ACTIONLINT_BIN"
