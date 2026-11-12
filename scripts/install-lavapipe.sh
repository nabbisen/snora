#!/usr/bin/env bash
set -euo pipefail

# Installs lavapipe (Mesa's software Vulkan) for the wgpu-rendered CI job, from **pinned, verified**
# .deb files, so that the job does not depend on Ubuntu's mirrors being up.
#
# Why: on 2026-11-11 `apt-get install mesa-vulkan-drivers` could not reach Ubuntu's mirrors for about
# three hours. Four runs failed with no snora test run, and the 0.60.0 release waited. Before that the
# job took whatever version the mirror served that day, so the rendered output could change under it.
#
# What it does, in order:
#   1. For each file in PINNED below, uses the copy in the cache directory if its SHA-256 is the pinned
#      one. A cached file that fails is said so loudly (a `::warning::` in CI), deleted, and fetched again.
#   2. A file that is not cached is fetched from SOURCES in order, and **accepted only if its SHA-256 is
#      the pinned one**. A source that serves different bytes is skipped, not trusted.
#   3. Checks that this CPU has AVX2 (see below), then installs exactly those files with `dpkg -i`. What runs is what was verified. No `apt-get update`, no
#      `apt-get install`, nothing resolved against a mirror.
#   4. Checks that the Vulkan loader now has lavapipe's ICD.
# If a file cannot be had with the pinned checksum, this exits non-zero and installs nothing.
#
# The cache is the primary source and the network is the fallback only when it is empty: the workflow
# restores `SNORA_LAVAPIPE_CACHE` with `actions/cache`, keyed on this file's hash.
#
# How the pin is bumped, and by whom: **the commit that changes PINNED.** On the target image (the
# ubuntu-latest runner; Ubuntu 26.04 "resolute" at the time of writing), `apt-get download
# mesa-vulkan-drivers libdisplay-info3` and `sha256sum` the files give the lines; also check that
# `dpkg -i` of just those files works there (any other missing dependency must be pinned too). Then
# the rendered jobs and the parity job run on the new version in the branch CI: **if the pixels differ
# from the old version's, that is a baseline change and its own decision**, not part of the bump. The
# implementer prepares it; the architect rules on a baseline change. A changed file changes the cache
# key (`hashFiles` of this script), so the new version is fetched once and cached.
#
# The pinned files are the **`amd64v3`** builds (what apt chose on the runner's CPU, an AMD EPYC 9V74), which
# need the x86-64-v3 instructions (AVX2 is the marker). On a CPU without them the install would succeed and
# lavapipe would then die inside the rendered tests with an illegal instruction, which would look like a snora
# crash. So step 3 refuses first, and names the fix: pin the plain `amd64` build (the same package, from the
# non-`v3` index) and its checksums.
#
# The last line of defence, stated: if `ubuntu-latest` moves to another Ubuntu release, `dpkg -i` here fails
# for a dependency the new image does not have, loudly, and the pin is bumped as above.
#
# Test hooks (never set by the workflow except where its comment says so):
#   SNORA_LAVAPIPE_CACHE     the cache directory (default: .lavapipe-cache in the repository root)
#   SNORA_LAVAPIPE_SOURCES   space-separated base URLs, tried in order (default below)
#   SNORA_LAVAPIPE_PINNED    a replacement for PINNED (for testing the refusal paths against local files)
#   SNORA_LAVAPIPE_VERIFY_ONLY=1   stop after step 2: verify and cache, install nothing
#   SNORA_LAVAPIPE_CPUINFO   the file the CPU flags are read from (default /proc/cpuinfo)
#   SNORA_LAVAPIPE_DRY_RUN=1   stop after step 3's CPU check: say what would be installed, install nothing

# `<sha256>  <path under /ubuntu/>`; the file name is the path's last part. Taken from the ubuntu-latest
# runner on 2026-11-12 (`apt-cache show` and `sha256sum`, both agree).
PINNED="
8248d972009babae14d67275e290e024fa4af6040d2ecbc8be56d8c75341c6c1  pool/main/libd/libdisplay-info/libdisplay-info3_0.3.0-1_amd64v3.deb
fc21b44a0b55c2c2b076ad3e95667e1d870f4826bb85d885757f701fbb7295a9  pool/main/m/mesa/mesa-vulkan-drivers_26.0.8-1ubuntu0.3_amd64v3.deb
"
PINNED="${SNORA_LAVAPIPE_PINNED:-$PINNED}"

# In order: Canonical's archive, the runner's own mirror, then the snapshot service, which keeps
# superseded versions (the archive's pool drops a version once a newer one is published).
# The snapshot's timestamp is fixed, so it names one state of the archive forever.
SOURCES="${SNORA_LAVAPIPE_SOURCES:-http://archive.ubuntu.com/ubuntu http://azure.archive.ubuntu.com/ubuntu https://snapshot.ubuntu.com/ubuntu/20261112T000000Z}"

cd "$(git rev-parse --show-toplevel)"
CACHE="${SNORA_LAVAPIPE_CACHE:-.lavapipe-cache}"
mkdir -p "$CACHE"

sha256_of() { sha256sum "$1" | cut -d' ' -f1; }

fetch_verified() { # <sha> <path>
  local want="$1" path="$2" name file src got
  name="$(basename "$path")"
  file="$CACHE/$name"
  if [ -f "$file" ]; then
    got="$(sha256_of "$file")"
    if [ "$got" = "$want" ]; then
      echo "lavapipe: $name: cached, checksum verified"
      return 0
    fi
    echo "::warning::lavapipe: cached $name has SHA-256 $got, not the pinned $want; deleted, fetching again" >&2
    rm -f "$file"
  fi
  for src in $SOURCES; do
    rm -f "$file.part"
    if curl -fsSL --retry 2 --retry-delay 2 --max-time 180 -o "$file.part" "$src/$path" 2>/dev/null; then
      got="$(sha256_of "$file.part")"
      if [ "$got" = "$want" ]; then
        mv "$file.part" "$file"
        echo "lavapipe: $name: fetched from $src, checksum verified"
        return 0
      fi
      echo "::warning::lavapipe: $src served $name with SHA-256 $got, not the pinned $want; not used" >&2
    else
      echo "::warning::lavapipe: $src could not serve $name" >&2
    fi
    rm -f "$file.part"
  done
  echo "lavapipe: $name could not be had with the pinned SHA-256 $want from any source; nothing installed" >&2
  return 1
}

files=()
while read -r sha path; do
  [ -z "${sha:-}" ] && continue
  fetch_verified "$sha" "$path"
  files+=("$CACHE/$(basename "$path")")
done <<< "$PINNED"

if [ "${SNORA_LAVAPIPE_VERIFY_ONLY:-}" = "1" ]; then
  echo "lavapipe: verified ${#files[@]} file(s); installing nothing (verify only)"
  exit 0
fi

cpuinfo="${SNORA_LAVAPIPE_CPUINFO:-/proc/cpuinfo}"
if ! grep -qm1 -w avx2 "$cpuinfo" 2>/dev/null; then
  echo "lavapipe: this CPU does not report AVX2, and the pinned packages are the amd64v3 builds, which need it" >&2
  echo "lavapipe: they would install and then fail in the rendered tests with an illegal instruction." >&2
  echo "lavapipe: fix: pin the plain amd64 build of mesa-vulkan-drivers and libdisplay-info3 (PINNED, in this file). Nothing installed." >&2
  exit 1
fi
if [ "${SNORA_LAVAPIPE_DRY_RUN:-}" = "1" ]; then
  echo "lavapipe: AVX2 present; would install ${#files[@]} pinned package(s); dry run, installing nothing"
  exit 0
fi

sudo=""
[ "$(id -u)" -ne 0 ] && sudo="sudo"
$sudo dpkg -i "${files[@]}"

icd=/usr/share/vulkan/icd.d/lvp_icd.json
if [ ! -f "$icd" ]; then
  echo "lavapipe: $icd is missing after the install" >&2
  exit 1
fi
echo "lavapipe: installed ${#files[@]} pinned package(s); $icd present"
