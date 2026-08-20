# RFC 074 — The release checklist names two files by hand, so every other version snippet drifts

**Status.** Accepted (owner, 2026-08-20). Handoff written — see
[`handoffs/074-…`](../handoffs/074-version-snippets-are-a-hand-maintained-list/implementation-handoff.md).
**Tracks.** Release process / documentation integrity.
**Found by** the owner, 2026-08-20, reading `crates/snora/src/lib.rs`.
**Touches.** `crates/snora/src/lib.rs`, `README.md`,
`docs/src/design/feature-flags.md`, `docs/src/reference/widgets.md`,
`docs/src/contributing/release-process.md`.
**Release target.** 0.38.3 — documentation only; **no code, no API.**

## Summary

`release-process.md:126` reads:

> `[ ] Update user-facing version snippets in install.md and icons.md to the
> new version`

**Two files, named by hand.** Every version snippet outside those two has been
drifting since the line was written (RFC-018-A). The current state:

| Location | shows | minors behind | reach |
|---|---|---|---|
| `crates/snora/src/lib.rs:72` | `0.25` | **13** | **docs.rs — every published release** |
| `docs/src/reference/widgets.md:23` | `0.6` | **32** | the book |
| `docs/src/design/feature-flags.md` | `0.25`, `0.28`×4, `0.31` | up to 13 | the book |
| `README.md:44` | `0.37` | 1 | **the repository front page and the crates.io listing** |

`install.md` and `icons.md` are current, because they are the two the checklist
names.

## The sharpest instance

**`crates/snora/src/lib.rs` ships to docs.rs.** A consumer reading snora
0.38.2's own API documentation is told, in the engine-only build section, to
write `snora = { version = "0.25", default-features = false }` — a version that
predates `snora-style` (RFC-055, 0.32.0), the border repair (0.34.0), the modal
dim (0.37.0), and the line-height helpers (0.38.0). It is the one snippet we
publish *as part of the crate itself*, and it is the most stale of all of them.

`docs/src/design/feature-flags.md` is worse per-page: **six snippets, three
different versions, none current** — a reader cannot even infer the intended
version by consensus.

## The defect is the rule, not the files

This is the RFC-063 shape sitting inside the release process itself: **a
hand-maintained list of locations, with nothing deriving it.** Adding a seventh
version snippet anywhere requires remembering to add a seventh name to a line in
a checklist, and nothing fails when you do not.

**I have now hit this twice and fixed only the symptom both times.** During the
0.38.0 cut I found `docs/src/design/overview.md` stating `0.28`, nine minors
stale, corrected the file, and did not ask why the checklist had not caught it.
That is the miss this RFC exists to close.

## Scope

1. Correct every stale snippet in the table above.
2. **Replace the enumerated checklist line with a derived one** — a check that
   finds every version-bearing snora snippet in the repository and reports any
   whose minor is not the release's, so the set is discovered rather than
   remembered.

## The hard part is the exclusion set

A naive sweep-and-rewrite would corrupt the historical record. These must
**never** be updated:

- `docs/src/guides/migration-*.md` — they document what a version *was*.
- `CHANGELOG.md` — likewise.
- `rfcs/**` — RFC-051 quotes `snora = "0.25"` as a consumer's then-current
  version; RFC-056 quotes `snora-widgets = "0.6"` as *the stale instruction it
  was reporting*. Rewriting either would destroy the finding.

**A check that cannot distinguish a live snippet from a quoted one is worse than
no check**, because it will either produce noise every release or be silenced
and stop working. Getting this boundary right is the design work; the string
replacements are not.

## Open questions

**Q-1 — what carries the check?** A script in `scripts/`, run manually at
release time, matching `check-built-links.py`'s precedent (RFC-073: committed,
runnable, **not** a CI gate). **Suggest yes, same shape, same non-gate
reasoning** — and per RFC-064's precedent the audit and rule ship before any
gate is pointed at it.

**Q-2 — how does the check know the expected value?** Snippets carry the
**minor** (`snora = "0.38"` is a caret range covering 0.38.2), so only a minor
bump requires the sweep — but reading the expected minor from
`[workspace.package].version` makes the check self-updating rather than needing
its own constant. **Suggest deriving it.** A check with a hand-written expected
version would be this defect one level down.

**Q-3 — does it cover `crates/**/*.rs` doc comments?** It must — that is where
the worst instance is, and it is the only one that ships inside the artifact.

## Acceptance criteria

1. Every location in the table states the current minor.
2. `release-process.md`'s line no longer names files; it invokes the check.
3. The check derives its expected value from `Cargo.toml` (Q-2) and covers
   crate doc comments (Q-3).
4. **The exclusion set is explicit, and justified in one line each** —
   migration guides, CHANGELOG, `rfcs/`.
5. **A perturbation demo:** stale one snippet, run the check, see it named;
   restore. And confirm the check stays silent on the excluded historical
   references — **both directions**, since a check that flags history is the
   failure mode that gets it disabled.
6. No CI gate; no code change.

## Compatibility and security

**Compatibility.** Documentation only. `crates/snora/src/lib.rs` changes a doc
comment; no API. **Security.** None.
