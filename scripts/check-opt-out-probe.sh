#!/usr/bin/env bash
set -euo pipefail

# RFC-112: a consumer can turn off iced's default features.
#
# Before RFC-112 they could not. snora's own iced line carried iced's
# defaults, and Cargo features only ever add up — so an application
# writing `default-features = false` on its own iced line still got
# every one of them, including `linux-theme-detection` and the zbus and
# mundy trees behind it. Nothing failed; the opt-out was simply inert,
# which is why it took a consumer (aaai) to find it.
#
# `examples/opt_out_probe` is a manifest shaped like that application.
# This asserts what Cargo resolves for it, because that is the thing
# under test: not what a source file says, but what the dependency
# graph comes out as.
#
# The rule it encodes (RFC-112): `web-colors` and `crisp` are snora's
# own drawing choices and stay on. Everything else iced enables by
# default is the application's to decide, and reaches iced only through
# snora's `iced-default` feature.
#
# Usage: scripts/check-opt-out-probe.sh

cd "$(git rev-parse --show-toplevel)"

readonly PROBE="snora-opt-out-probe"
# Pinned: the platform whose default-feature tree carries zbus. The
# assertions below are about a specific target's resolution, and
# `cargo tree` without `--target` answers for all of them at once,
# which would make "absent" mean something weaker than it should.
readonly TARGET="x86_64-unknown-linux-gnu"

failures=()
note() { printf '  %s\n' "$1"; }

echo "== the probe builds =="
if cargo check -p "$PROBE" --target "$TARGET" >/dev/null 2>&1; then
  note "OK: $PROBE checks on $TARGET"
else
  failures+=("$PROBE does not build on $TARGET")
  note "FAIL: $PROBE does not build — running it again for the error:"
  cargo check -p "$PROBE" --target "$TARGET" 2>&1 | tail -20
fi

# `-e normal`: build- and dev-dependencies are not what a consumer
# ships. `-i <crate>` asks "who depends on this".
#
# **Test the output, not the exit code.** `cargo tree -i` exits 0 in
# two different situations: when it printed a tree, and when it had
# nothing to print because the crate is in the workspace's resolve but
# not reachable from this package. It exits 101 only when no package
# matches the name at all. An earlier version of this script read the
# exit code, so "unreachable" came back as "present" — which made the
# renderer assertions unfalsifiable and led to a confident, wrong
# conclusion about iced's own manifest. The zbus and mundy checks
# happened to survive it, because there a false "present" can only
# cause a false failure; the renderer checks did not.
in_tree() {
  cargo tree -p "$PROBE" --target "$TARGET" -e normal -i "$1" 2>/dev/null \
    | grep -q "^$1 v"
}

echo
echo "== turned off: the trees behind iced's default features =="
for crate in zbus mundy; do
  if in_tree "$crate"; then
    failures+=("$crate is in the probe's tree, so the opt-out did not take")
    note "FAIL: $crate present"
    cargo tree -p "$PROBE" --target "$TARGET" -e normal -i "$crate" 2>&1 | head -6 | sed 's/^/    /'
  else
    note "OK: $crate absent"
  fi
done

echo
echo "== kept: the renderers the probe asked for itself =="
# `iced_tiny_skia` and `iced_wgpu`, not the raw `tiny-skia` and
# `wgpu`. The raw `tiny-skia` is pulled by winit's Wayland decorations
# (`sctk-adwaita`) whichever renderer the application picks, so
# asserting it would pass with no renderer at all; iced's own renderer
# crates are the ones the probe's iced line decides.
for crate in iced_tiny_skia iced_wgpu; do
  if in_tree "$crate"; then
    note "OK: $crate present"
  else
    failures+=("$crate is missing, though the probe's iced line asks for it")
    note "FAIL: $crate absent"
  fi
done

echo
echo "== honoured: the features only the probe asks for =="
# On top of the renderers: two features snora never requests and the
# probe does, so this reads the probe's own line reaching iced rather
# than snora's.
PROBE_ONLY=$(cargo tree -p "$PROBE" --target "$TARGET" -e features -i iced 2>/dev/null || true)
for feature in x11 wayland; do
  if grep -q "iced feature \"$feature\"" <<<"$PROBE_ONLY"; then
    note "OK: $feature on, and only the probe asks for it"
  else
    failures+=("iced feature $feature is off, though the probe's own line asks for it")
    note "FAIL: $feature off"
  fi
done

echo
echo "== kept: snora's own drawing features, which the probe does not ask for =="
# `-e features` annotates each edge with the features it enables, so
# this reads what iced was actually resolved with rather than what any
# one manifest requested.
ICED_FEATURES=$(cargo tree -p "$PROBE" --target "$TARGET" -e features -i iced 2>/dev/null || true)
if [[ -z "$ICED_FEATURES" ]]; then
  failures+=("iced is not in the probe's tree at all")
  note "FAIL: no iced in the tree"
else
  for feature in web-colors crisp; do
    if grep -q "iced feature \"$feature\"" <<<"$ICED_FEATURES"; then
      note "OK: $feature on, though the probe's own iced line omits it"
    else
      failures+=("iced feature $feature is off; RFC-112 keeps it on as snora's own")
      note "FAIL: $feature off"
    fi
  done
fi

echo
if ((${#failures[@]} > 0)); then
  printf '%s\n' "${#failures[@]} failure(s):" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  exit 1
fi
echo "OK: the opt-out resolves as RFC-112 specifies."
