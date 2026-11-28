#!/usr/bin/env bash
set -euo pipefail

# RFC-140 R-7. Refuses unless <tag> is an annotated tag whose signature verifies against the project's release-signing key.
# Before this, `release.yaml` triggered on a tag-name pattern and never looked at a signature: anyone able to push a tag
# matching the pattern could start a publish. The key file is the PUBLIC half of the key that signs the tags; it is kept in the
# repository (`.github/release-signing-key.asc`), so what the check trusts is reviewed like code.
#
# The key is imported into a throwaway GNUPGHOME that holds nothing else, so a signature by any other key, however valid,
# fails: the keyring is the allow-list.
#
# Usage: scripts/check-tag-signature.sh <tag> [key-file]      (key-file default: .github/release-signing-key.asc)
# Exit:  0 the tag is annotated and signed by the key; 1 refused (a message says why); 2 usage.

cd "$(git rev-parse --show-toplevel)"
TAG="${1:?usage: check-tag-signature.sh <tag> [key-file]}"
KEY="${2:-.github/release-signing-key.asc}"

command -v gpg >/dev/null || { echo "REFUSED: gpg is not on PATH, so no signature can be checked" >&2; exit 1; }
[ -s "$KEY" ] || { echo "REFUSED: the release-signing key $KEY is missing or empty, so no signature can be checked" >&2; exit 1; }
[ "$(git cat-file -t "refs/tags/$TAG" 2>/dev/null || true)" = tag ] \
  || { echo "REFUSED: refs/tags/$TAG is not an annotated tag object here (a lightweight tag, a missing tag, or a checkout that fetched only the commit): nothing to verify" >&2; exit 1; }

GNUPGHOME=$(mktemp -d)
chmod 700 "$GNUPGHOME"
export GNUPGHOME
trap 'rm -rf "$GNUPGHOME"' EXIT
gpg --batch --quiet --import "$KEY" 2>/dev/null || { echo "REFUSED: $KEY could not be imported as a public key" >&2; exit 1; }

if out=$(git verify-tag --raw "refs/tags/$TAG" 2>&1); then
  fpr=$(printf '%s\n' "$out" | sed -n 's/^\[GNUPG:\] VALIDSIG \([0-9A-F]\{40\}\) .*/\1/p' | head -1)
  echo "check-tag-signature.sh: $TAG is an annotated tag with a good signature by key ${fpr:-?} (the project's release-signing key)"
else
  echo "REFUSED: $TAG has no good signature from the key in $KEY:" >&2
  printf '%s\n' "$out" | grep -E '^\[GNUPG:\] (ERRSIG|NO_PUBKEY|BADSIG|EXPKEYSIG|REVKEYSIG)|error:|no signature' | head -3 | sed 's/^/  /' >&2
  exit 1
fi
