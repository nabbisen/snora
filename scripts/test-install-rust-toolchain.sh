#!/usr/bin/env bash
set -euo pipefail

# RFC-140. The tests of `scripts/install-rust-toolchain.sh`, against a fake `rustup` and `rustc` that record what they are asked
# and fail on demand, so the retry logic and the environment export run on every push (Workflow lint) without installing
# anything. Each case is a scratch directory; nothing outside it is touched.
#
# Usage: scripts/test-install-rust-toolchain.sh        Exit: 0 every case behaved as stated; 1 one did not.

cd "$(git rev-parse --show-toplevel)"
SCRIPT="$PWD/scripts/install-rust-toolchain.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
wrong=0

# The fake rustup: `toolchain install` consults $FAKE_INSTALL_PLAN, one outcome per line, consumed in order
# (ok | race | other); `default` just records. Everything is appended to $FAKE_LOG.
mkdir -p "$T/bin"
cat > "$T/bin/rustup" <<'FAKE'
#!/usr/bin/env bash
echo "rustup $*" >> "$FAKE_LOG"
if [ "$1 $2" = "toolchain install" ]; then
  outcome=$(head -1 "$FAKE_INSTALL_PLAN" 2>/dev/null || echo ok)
  sed -i 1d "$FAKE_INSTALL_PLAN" 2>/dev/null || true
  case "$outcome" in
    race)  echo "error: checksum failed for 'https://static.rust-lang.org/x', expected: 'a', calculated: 'b'"
           echo "this is likely due to an ongoing update of the official release server; please try again"; exit 1;;
    other) echo "error: toolchain 'nosuch' is not installable"; exit 1;;
  esac
fi
if [ "$1" = default ] && [ "${FAKE_DEFAULT_FAILS:-}" = 1 ]; then echo "error: could not set the default" >&2; exit 1; fi
exit 0
FAKE
printf '#!/usr/bin/env bash\necho "rustc $*" >> "$FAKE_LOG"\necho "rustc 1.0.0 (fake)"\n' > "$T/bin/rustc"
chmod +x "$T/bin/rustup" "$T/bin/rustc"

case_() { # name expect(pass|refuse) plan-file-content args... ; env: EXTRA_ENV
  local name=$1 expect=$2 plan=$3; shift 3
  local d="$T/$name"; mkdir -p "$d"; printf '%s' "$plan" > "$d/plan"; : > "$d/log"; : > "$d/ghenv"
  set +e
  out=$(env -i PATH="$T/bin:/usr/bin:/bin" HOME="$d" FAKE_LOG="$d/log" FAKE_INSTALL_PLAN="$d/plan" GITHUB_ENV="$d/ghenv" \
        SNORA_TOOLCHAIN_SCRIPT_TEST=1 RETRY_SLEEP=0 ${EXTRA_ENV:-} bash "$SCRIPT" "$@" 2>&1)
  rc=$?
  set -e
  CASE_OUT="$out"; CASE_DIR="$d"; CASE_RC=$rc
  if { [ "$expect" = pass ] && [ $rc -eq 0 ]; } || { [ "$expect" = refuse ] && [ $rc -ne 0 ]; }; then return 0; fi
  echo "WRONG: $name: expected $expect, got exit $rc"; echo "$out" | head -5 | sed 's/^/    /'; wrong=1
}
installs() { grep -c '^rustup toolchain install' "$CASE_DIR/log" || true; }
check() { # description condition
  if eval "$2"; then echo "  ok: $1"; else echo "WRONG: $1"; wrong=1; fi
}

echo "== the install"
case_ first-try pass $'ok\n' stable
check "one install, with the minimal profile and no self-update, then the default" \
  '[ "$(installs)" = 1 ] && grep -q "^rustup toolchain install stable --profile minimal --no-self-update$" "$CASE_DIR/log" && grep -q "^rustup default stable$" "$CASE_DIR/log"'
check "the version is printed last (the evidence a job ran what it asked for)" 'tail -1 "$CASE_DIR/log" | grep -q "^rustc +stable --version --verbose$"'
check "CARGO_INCREMENTAL=0 and CARGO_TERM_COLOR=always are exported when unset" \
  'grep -qx "CARGO_INCREMENTAL=0" "$CASE_DIR/ghenv" && grep -qx "CARGO_TERM_COLOR=always" "$CASE_DIR/ghenv"'
EXTRA_ENV="CARGO_INCREMENTAL=1 CARGO_TERM_COLOR=never" case_ env-preset pass $'ok\n' stable
check "a preset CARGO_INCREMENTAL and CARGO_TERM_COLOR are left alone" '[ ! -s "$CASE_DIR/ghenv" ]'
case_ components pass $'ok\n' stable rustfmt,clippy
check "components become --component flags" 'grep -q -- "--component rustfmt --component clippy --profile minimal" "$CASE_DIR/log"'
case_ msrv pass $'ok\n' 1.88
check "an MSRV string is passed as it is" 'grep -q "^rustup toolchain install 1.88 " "$CASE_DIR/log" && grep -q "^rustup default 1.88$" "$CASE_DIR/log"'

echo "== the retry: only for rustup's release-server race"
case_ race-twice pass $'race\nrace\nok\n' stable
check "two races then success: three installs, and the default after" '[ "$(installs)" = 3 ] && grep -q "^rustup default stable$" "$CASE_DIR/log"'
case_ race-always refuse $'race\nrace\nrace\nrace\nrace\nrace\n' stable
check "five races: five installs, then a refusal, and no default" '[ "$(installs)" = 5 ] && ! grep -q "^rustup default" "$CASE_DIR/log"'
case_ other-failure refuse $'other\nok\n' stable
check "any other failure stops at the first attempt" '[ "$(installs)" = 1 ] && ! grep -q "^rustup default" "$CASE_DIR/log"'

echo "== the default is not allowed to fail silently"
EXTRA_ENV="FAKE_DEFAULT_FAILS=1" case_ default-fails refuse $'ok\n' stable
check "a failing 'rustup default' fails the step" '[ "$CASE_RC" -ne 0 ] && ! grep -q "^rustc " "$CASE_DIR/log"'

echo "== it refuses off a runner"
set +e; out=$(env -i PATH="$T/bin:/usr/bin:/bin" HOME="$T" bash "$SCRIPT" stable 2>&1); rc=$?; set -e
check "without GITHUB_ACTIONS it refuses and runs nothing" '[ "$rc" -eq 1 ] && echo "$out" | grep -q "REFUSED"'
set +e; out=$(env -i PATH="$T/bin:/usr/bin:/bin" HOME="$T" GITHUB_ACTIONS=true FAKE_LOG="$T/l" FAKE_INSTALL_PLAN="$T/p" RETRY_SLEEP=0 bash "$SCRIPT" 2>&1); rc=$?; set -e
check "without a toolchain argument it is a usage error" '[ "$rc" -ne 0 ]'

if [ "$wrong" -eq 0 ]; then echo "test-install-rust-toolchain.sh: every case behaved as stated"; else echo "test-install-rust-toolchain.sh: FAILED" >&2; fi
exit "$wrong"
