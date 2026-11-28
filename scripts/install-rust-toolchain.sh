#!/usr/bin/env bash
set -euo pipefail

# RFC-140. Installs a Rust toolchain on a GitHub runner, in place of `dtolnay/rust-toolchain`: the one third-party action in the
# release path that has no releases to pin (its `stable` and `master` are branches), and that runs inside `publish`. What that
# action did is a few `rustup` commands; they are here, in a script a reviewer can read, and what it did beyond them is
# listed below with the reason each was kept or dropped (read at `dtolnay/rust-toolchain@df2a88e`, `action.yml`).
#
#   KEPT
#   * the install itself: `rustup toolchain install <t> --profile minimal --no-self-update`, then `rustup default <t>`;
#   * the retry, five attempts 30 s apart, **only** when rustup says the checksum failed "likely due to an ongoing update of
#     the official release server" (a real race at every Rust release); any other failure stops at once;
#   * `RUSTUP_PERMIT_COPY_RENAME=1` (rustup falls back to copying where a rename across filesystems fails);
#   * `CARGO_INCREMENTAL=0` and `CARGO_TERM_COLOR=always`, exported for the later steps when unset: the first because the
#     build-cost CSV and every CI cache were produced with incremental compilation off, the second because CI's logs are read
#     with colour codes in them;
#   * the final `rustc +<t> --version --verbose`, which is the evidence that a job ran the toolchain it asked for.
#   CHANGED
#   * `rustup default` is not allowed to fail silently (the action ran it with `continue-on-error`, for a Windows issue): a
#     failed default would leave the job on whatever the runner shipped, with a green tick.
#   DROPPED (nothing here uses it)
#   * the `1.NN` arithmetic that turns a YAML-float `1.1` back into `1.10`, and `stable N weeks ago` forms; the MSRV arrives as
#     a string from `grep`, and `stable` is passed as it is;
#   * installing rustup when missing, `$CARGO_HOME` export, and the Windows branches: the Ubuntu runner has both set;
#   * the `cachekey` and `name` outputs: no step reads them (`Swatinem/rust-cache` keys on `rustc -vV` itself);
#   * the workarounds for toolchains 1.66 to 1.71 (sparse registry, curl multiplexing): the floor is 1.88.
#
# **It changes the machine's default toolchain**, so it refuses to run anywhere that is not a GitHub runner.
#
# Usage: scripts/install-rust-toolchain.sh <toolchain> [component[,component...]]
#        (RETRY_SLEEP seconds between attempts, default 30; the test sets it to 0)

TOOLCHAIN="${1:?usage: install-rust-toolchain.sh <toolchain> [component,component]}"
COMPONENTS="${2:-}"

if [ "${GITHUB_ACTIONS:-}" != "true" ] && [ "${SNORA_TOOLCHAIN_SCRIPT_TEST:-}" != "1" ]; then
  echo "REFUSED: this changes rustup's default toolchain, and is for a GitHub runner (GITHUB_ACTIONS is not 'true')" >&2
  exit 1
fi
command -v rustup >/dev/null || { echo "REFUSED: rustup is not on PATH (the Ubuntu runner has it)" >&2; exit 1; }

flags=()
for c in ${COMPONENTS//,/ }; do flags+=(--component "$c"); done
export RUSTUP_PERMIT_COPY_RENAME=1
sleep_s="${RETRY_SLEEP:-30}"

log=$(mktemp)
trap 'rm -f "$log"' EXIT
status=0
for attempt in 1 2 3 4 5; do
  if rustup toolchain install "$TOOLCHAIN" ${flags[@]+"${flags[@]}"} --profile minimal --no-self-update 2>&1 | tee "$log"; then
    status=0
    break
  else
    status=$?
  fi
  if [ "$attempt" = 5 ] || ! grep -Fq 'checksum failed for ' "$log" \
      || ! grep -Fq 'this is likely due to an ongoing update of the official release server' "$log"; then
    echo "REFUSED: rustup toolchain install $TOOLCHAIN failed (attempt $attempt, exit $status)" >&2
    exit "$status"
  fi
  echo "Retrying rustup toolchain install in $sleep_s seconds (retry $attempt of 4)" >&2
  sleep "$sleep_s"
done

rustup default "$TOOLCHAIN"

if [ -n "${GITHUB_ENV:-}" ]; then
  [ -z "${CARGO_INCREMENTAL+set}" ] && echo "CARGO_INCREMENTAL=0" >> "$GITHUB_ENV"
  [ -z "${CARGO_TERM_COLOR+set}" ] && echo "CARGO_TERM_COLOR=always" >> "$GITHUB_ENV"
fi
rustc +"$TOOLCHAIN" --version --verbose
exit 0
