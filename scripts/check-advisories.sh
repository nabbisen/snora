#!/usr/bin/env bash
set -euo pipefail

# Scans the resolved dependency graph with cargo-deny, pinned to an exact
# version (RFC-097 Unit 2).
#
# Until this existed, snora gated compilation, clippy, rustdoc warnings,
# feature combinations, iced-freedom, workflow syntax, doc links, version
# snippets, migration-guide coverage, docs-only claims, and whether
# upstream still compiles after a fresh resolve -- and **nothing about
# whether a dependency is known vulnerable**, across 310 packages and
# five published crates. `unpinned-build` is the sharpest version of that
# gap: it exists to watch upstream movement and watches only the compile
# axis.
#
# ## What is fatal here and what is not (Q-1/Q-2 rulings, see deny.toml)
#
#   * advisories                -- FATAL. Exit 1 on any advisory.
#   * licenses, bans, sources   -- reported, never fatal. Their exit
#                                  status is deliberately discarded.
#
# Tightening the second group is a later decision to be taken with
# evidence in hand. Running them now is what produces that evidence.
#
# ## Why this is a script and not inline workflow YAML
#
# So the failing-first demonstration exercises the same code CI runs. A
# gate whose logic is transcribed by hand into a review transcript is a
# gate demonstrated in copy, not in fact.
#
# Usage: scripts/check-advisories.sh
#
# Resolution order, matching scripts/check-workflows.sh exactly:
#   1. `cargo-deny` already on PATH -- used as-is, whatever version, and
#      the version actually used is printed so a transcript is honest
#      about what ran.
#   2. Otherwise the pinned version's release binary for this machine's
#      OS/arch, verified against a pinned SHA-256 before it is executed,
#      cached under target/cargo-deny-cache/.
#
# If neither is possible -- unsupported OS/arch, no network, checksum
# mismatch -- this script says so on stderr and exits non-zero. It does
# NOT report success with nothing scanned. An unrunnable scanner
# reporting green is the defect this project has now found five times
# (RFC-087 D-1, RFC-086's alpha-blind assertion, RFC-088's silent set -e
# exit, RFC-090's short-SHA gap, RFC-094's untested subscription), and a
# security scanner is the worst place to add a sixth.
#
# Exit codes:
#   0  cargo-deny ran and the advisories check passed
#   1  cargo-deny is unavailable and could not be obtained, OR
#      the advisories check found something

CARGO_DENY_VERSION="0.20.2"

# Pinned SHA-256 sums for this version's release assets, from the
# per-asset .sha256 files at
# https://github.com/EmbarkStudios/cargo-deny/releases/tag/0.20.2
# Only the OS/arch combinations this project's contributors and CI runner
# (ubuntu-latest = linux/amd64) actually use are listed. Add a line here,
# from that same source, before adding support for another platform --
# do not download unverified.
CHECKSUMS="
linux_amd64   9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f
linux_arm64   995c82be0defc7a025cae49a2aa2644ce8245c9a3318fc4103907c6a285e8c7d
darwin_amd64  248da7f581724e470071990c088ffc55c811981715f4cbdb258621fb79f8b7a6
darwin_arm64  fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3
"

# Release-asset target triple per platform. cargo-deny ships musl builds
# for Linux, which is what makes the downloaded binary runnable on the
# CI image without matching its glibc.
TRIPLES="
linux_amd64   x86_64-unknown-linux-musl
linux_arm64   aarch64-unknown-linux-musl
darwin_amd64  x86_64-apple-darwin
darwin_arm64  aarch64-apple-darwin
"

cd "$(git rev-parse --show-toplevel)"

resolve_cargo_deny() {
  if command -v cargo-deny >/dev/null 2>&1; then
    command -v cargo-deny
    return 0
  fi

  local os arch platform
  case "$(uname -s)" in
    Linux) os="linux" ;;
    Darwin) os="darwin" ;;
    *)
      echo "error: no cargo-deny on PATH, and this OS ($(uname -s)) has no pinned download entry" >&2
      return 1
      ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64) arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)
      echo "error: no cargo-deny on PATH, and this architecture ($(uname -m)) has no pinned download entry" >&2
      return 1
      ;;
  esac
  platform="${os}_${arch}"

  local expected_sha triple
  expected_sha=$(echo "$CHECKSUMS" | awk -v p="$platform" '$1 == p { print $2 }')
  triple=$(echo "$TRIPLES" | awk -v p="$platform" '$1 == p { print $2 }')
  if [[ -z "$expected_sha" || -z "$triple" ]]; then
    echo "error: no cargo-deny on PATH, and no pinned checksum/triple for $platform" >&2
    return 1
  fi

  local cache_dir="target/cargo-deny-cache/${CARGO_DENY_VERSION}"
  local bin_path="${cache_dir}/cargo-deny"
  if [[ -x "$bin_path" ]]; then
    bin_path="$(cd "$(dirname "$bin_path")" && pwd)/$(basename "$bin_path")"
    echo "$bin_path"
    return 0
  fi

  mkdir -p "$cache_dir"
  local stem="cargo-deny-${CARGO_DENY_VERSION}-${triple}"
  local archive="${cache_dir}/${stem}.tar.gz"
  local url="https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/${stem}.tar.gz"

  echo "no cargo-deny on PATH; downloading pinned v${CARGO_DENY_VERSION} for ${platform}..." >&2
  if ! curl -fsSL -o "$archive" "$url"; then
    echo "error: failed to download cargo-deny from $url" >&2
    return 1
  fi

  local actual_sha
  actual_sha=$(sha256sum "$archive" | cut -d' ' -f1)
  if [[ "$actual_sha" != "$expected_sha" ]]; then
    echo "error: cargo-deny download checksum mismatch -- expected $expected_sha, got $actual_sha" >&2
    echo "refusing to run an unverified binary" >&2
    rm -f "$archive"
    return 1
  fi

  # The archive contains <stem>/cargo-deny alongside its licence files.
  tar -xzf "$archive" -C "$cache_dir" --strip-components=1 "${stem}/cargo-deny"
  rm -f "$archive"
  chmod +x "$bin_path"
  bin_path="$(cd "$(dirname "$bin_path")" && pwd)/$(basename "$bin_path")"
  echo "$bin_path"
}

CARGO_DENY_BIN=$(resolve_cargo_deny) || {
  echo "REFUSED: cargo-deny is unavailable and could not be obtained -- this graph is unscanned, not clean." >&2
  exit 1
}

echo "Using cargo-deny: $CARGO_DENY_BIN ($("$CARGO_DENY_BIN" --version 2>&1 | head -1))"
echo

# ---------------------------------------------------------------------
# Non-fatal group first, so its output is visible even when the fatal
# group below fails the job. `|| true` is load-bearing under `set -e`:
# without it these checks would abort the script and the advisories
# result -- the one that matters -- would never run.
# ---------------------------------------------------------------------
echo "== licenses, bans, sources (reported, not enforced) =="
"$CARGO_DENY_BIN" check licenses bans sources || true

echo
echo "== advisories (fatal) =="
# `--deny advisory-not-detected` is the load-bearing half of the ruling
# that permits `deny.toml` to carry any `ignore` entry at all (RFC-097
# Unit 2, R-1).
#
# cargo-deny emits `advisory-not-detected` when an ignored advisory no
# longer matches anything in the graph -- i.e. when the reason for
# accepting it has expired. By default that is a warning and the job
# stays green, which would let an accepted risk outlive its own
# justification indefinitely: a rule with nothing to fire it, this
# project's most-repeated defect. Promoted to an error, the gate goes
# red the moment upstream fixes one of the accepted advisories, and
# names the entry to delete.
#
# There is no dated-expiry alternative at this pin: cargo-deny 0.20.2's
# ignore schema is exactly ["id", "reason"], and an `expiration` key
# fails to deserialise.
"$CARGO_DENY_BIN" check advisories --deny advisory-not-detected
