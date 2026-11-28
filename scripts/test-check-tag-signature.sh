#!/usr/bin/env bash
set -euo pipefail

# RFC-140 R-7, Q-3. The tests of `scripts/check-tag-signature.sh`, with scratch keys and a local bare repository: no network, no
# tag is pushed anywhere, and everything is deleted on exit. It runs on every push (Workflow lint), so that `gpg` and the
# script are proven on the real runner image **before** a release depends on them. The release's own refusal publishes nothing,
# but it would also publish nothing on the day it is needed.
#
# (a) a checkout of a tag push carries the annotated tag object: `actions/checkout@d23441a` fetches
#     `git -c protocol.version=2 fetch --no-tags --prune --no-recurse-submodules --depth=1 origin +refs/tags/T:refs/tags/T`
#     (`src/git-command-manager.ts`, `src/ref-helper.ts` `getRefSpec`), replayed here;
# (b) the script passes a tag signed by the key it is given, and refuses every other shape.
#
# Usage: scripts/test-check-tag-signature.sh      Exit: 0 every row as stated; 1 one was not.

cd "$(git rev-parse --show-toplevel)"
SCRIPT="$PWD/scripts/check-tag-signature.sh"
command -v gpg >/dev/null || { echo "test-check-tag-signature.sh: FAILED: gpg is not on PATH" >&2; exit 1; }
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
wrong=0

mkkey() { # name: a GNUPGHOME with one scratch key; its public half in $W/<name>.asc
  local h="$W/gpg-$1"; mkdir -m 700 "$h"
  GNUPGHOME="$h" gpg --batch --quiet --passphrase '' --quick-generate-key "scratch $1 <$1@scratch.invalid>" ed25519 sign never 2>/dev/null
  GNUPGHOME="$h" gpg --batch --quiet --armor --export "$1@scratch.invalid" > "$W/$1.asc"
}
mkkey A; mkkey B
R="$W/src"; git init -q -b main "$R"
(
  cd "$R"
  git config user.name t; git config user.email t@scratch.invalid; git config commit.gpgsign false; git config tag.gpgsign false
  echo x > f; git add f; git commit -q -m c1
  sign() { GNUPGHOME="$W/gpg-$2" git -c user.signingkey="$2@scratch.invalid" tag -s -m "$1" "$1"; }
  git tag light
  git tag -a -m unsigned unsigned
  sign signedA A; sign signedB B
  # signedA's tag object with its message edited after it was signed
  git cat-file tag signedA | sed 's/^signedA$/signedA edited/' > "$W/t.txt"
  git update-ref refs/tags/tampered "$(git hash-object -t tag -w "$W/t.txt")"
)
BARE="$W/bare.git"; git clone -q --bare "$R" "$BARE" 2>/dev/null; git -C "$R" push -q "$BARE" --tags 2>/dev/null

fetch() { # dir tag : the checkout action's fetch, in a fresh repository
  rm -rf "$1"; git init -q "$1"
  # a tag that does not exist makes the fetch fail, which is the case being tested: tolerate it here, not in the script's `-e`
  ( cd "$1" && git remote add origin "file://$BARE" \
      && { git -c protocol.version=2 fetch -q --no-tags --prune --no-recurse-submodules --depth=1 origin "+refs/tags/$2:refs/tags/$2" 2>/dev/null || true; } )
}
echo "== (a) the checkout of a tag push carries the annotated tag object"
for t in signedA unsigned; do
  fetch "$W/co-$t" "$t"
  type=$(git -C "$W/co-$t" cat-file -t "refs/tags/$t" 2>/dev/null || echo none)
  if [ "$type" = tag ]; then echo "  ok: refs/tags/$t is a tag object after the fetch"; else echo "WRONG: refs/tags/$t is '$type' after the fetch"; wrong=1; fi
done

row() { # name expect(pass|refuse) tag keyfile [reason]
  # A refusal is checked for its REASON as well as its exit status: several refusals are stopped twice (an early explicit
  # check, and again downstream by `git verify-tag` or an empty keyring), so a test of the exit status alone cannot tell the
  # early check from its removal. The early checks are what give an adopter-facing message, so the message is pinned.
  fetch "$W/r-$1" "$3"
  set +e; out=$(cd "$W/r-$1" && bash "$SCRIPT" "$3" "$4" 2>&1); rc=$?; set -e
  local ok=1
  if [ "$2" = pass ]; then [ "$rc" -eq 0 ] || ok=0; else [ "$rc" -eq 1 ] || ok=0; fi
  if [ "$ok" = 1 ] && [ -n "${5:-}" ] && ! printf '%s' "$out" | grep -Fq -- "$5"; then ok=0; fi
  if [ "$ok" = 1 ]; then echo "  ok: $1 ($2${5:+: $5})"; else echo "WRONG: $1: expected $2${5:+ with \"$5\"}, exit $rc: $(printf '%s' "$out" | head -1)"; wrong=1; fi
}
printf 'not a key\n' > "$W/junk.asc"
echo "== (b) check-tag-signature.sh"
row "signed by A, key file A"             pass   signedA   "$W/A.asc" "good signature by key"
row "signed by B, key file A"             refuse signedB   "$W/A.asc" "no good signature from the key"
row "signed by A, key file B"             refuse signedA   "$W/B.asc" "no good signature from the key"
row "unsigned annotated tag"              refuse unsigned  "$W/A.asc" "no good signature from the key"
row "tampered after signing"              refuse tampered  "$W/A.asc" "no good signature from the key"
row "lightweight tag"                     refuse light     "$W/A.asc" "not an annotated tag object"
row "tag that does not exist"             refuse nosuch    "$W/A.asc" "not an annotated tag object"
row "key file missing"                    refuse signedA   "$W/nokey.asc" "missing or empty"
row "key file is not a key"               refuse signedA   "$W/junk.asc" "could not be imported"

if [ "$wrong" -eq 0 ]; then echo "test-check-tag-signature.sh: every row behaved as stated"; else echo "test-check-tag-signature.sh: FAILED" >&2; fi
exit "$wrong"
