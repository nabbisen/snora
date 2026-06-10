# RFC-018-A — Version Number Maintenance

**Status.** Implemented (v0.18.0)
**Tracks.** Release hygiene.
**Touches.** `docs/src/getting-started/01-install.md`,
`docs/src/guides/icons.md`.

## 1. Problem

`install.md` and `icons.md` still show `snora = "0.14"`. Per RFC-015-A
(versioning policy), user-facing version snippets must track the latest
release. These should have been updated at v0.15, v0.16, and v0.17.

## 2. Fix

Replace all `"0.14"` (in snora dependency lines) with `"0.17"` in both
files. The iced version stays at `"0.14"` — that's iced's version, not
snora's.

## 3. Release-checklist addition

Add to the release checklist: "Update user-facing version snippets in
`install.md` and `icons.md` to the new version."

## 4. Acceptance criteria

- All `snora = "..."` snippets in install.md and icons.md show `"0.17"`.
- `iced = "0.14"` remains unchanged.
- Release checklist includes version-snippet update.
