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
#   2. Otherwise, downloads the pinned version's release binary for this
#      machine's OS/arch from GitHub releases, verifies it against a
#      pinned SHA-256 (not merely fetched and trusted), and caches it
#      under target/actionlint-cache/ so repeat runs don't re-download.
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
  local bin_path="${cache_dir}/actionlint"
  if [[ -x "$bin_path" ]]; then
    bin_path="$(cd "$(dirname "$bin_path")" && pwd)/$(basename "$bin_path")"
    echo "$bin_path"
    return 0
  fi

  mkdir -p "$cache_dir"
  local archive="${cache_dir}/actionlint_${ACTIONLINT_VERSION}_${platform}.tar.gz"
  local url="https://github.com/rhysd/actionlint/releases/download/v${ACTIONLINT_VERSION}/actionlint_${ACTIONLINT_VERSION}_${platform}.tar.gz"

  echo "no actionlint on PATH; downloading pinned v${ACTIONLINT_VERSION} for ${platform}..." >&2
  if ! curl -fsSL -o "$archive" "$url"; then
    echo "error: failed to download actionlint from $url" >&2
    return 1
  fi

  local actual_sha
  actual_sha=$(sha256sum "$archive" | cut -d' ' -f1)
  if [[ "$actual_sha" != "$expected_sha" ]]; then
    echo "error: actionlint download checksum mismatch -- expected $expected_sha, got $actual_sha" >&2
    echo "refusing to run an unverified binary" >&2
    rm -f "$archive"
    return 1
  fi

  tar -xzf "$archive" -C "$cache_dir" actionlint
  rm -f "$archive"
  chmod +x "$bin_path"
  bin_path="$(cd "$(dirname "$bin_path")" && pwd)/$(basename "$bin_path")"
  echo "$bin_path"
}

ACTIONLINT_BIN=$(resolve_actionlint) || {
  echo "REFUSED: actionlint is unavailable and could not be obtained -- this tree is unvalidated, not clean." >&2
  exit 1
}

echo "Using actionlint: $ACTIONLINT_BIN ($("$ACTIONLINT_BIN" -version 2>&1 | head -1))"
# No file arguments: actionlint's own documented default is to discover
# every .github/workflows/*.yml|*.yaml under the current directory, which
# is why `cd` to the repo root above matters.
"$ACTIONLINT_BIN"
