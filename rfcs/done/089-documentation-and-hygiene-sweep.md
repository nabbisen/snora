# RFC 089 — Documentation and hygiene sweep from the external audit

**Status.** Done — shipped in v0.41.1 (2026-09-02).
[Handoff](../handoffs/089-documentation-and-hygiene-sweep/implementation-handoff.md).
**Tracks.** Documentation / hygiene. **Severity: Medium and Low.**
**Found by** the external audit, 2026-09-01 (F-07, F-16–F-25, F-28, F-33, F-34,
F-38).
**Release target.** 0.41.1 — documentation and non-behavioural. **No public API.**

## Why one RFC and not twelve

None of these changes behaviour. Each is a sentence, a manifest key, or a dead
match arm. **Splitting them would cost more process than the fixes cost work**,
and this project has already been told once this month that it reaches for an
RFC by reflex.

**They are grouped by being cheap and independent — not by being related.** An
implementer may land them in any order, and any one may be dropped without
affecting the rest.

## The set

| # | What | Where |
|---|---|---|
| F-07 | Published docs describe animation/transitions that do not exist | `snora-core/src/overlay.rs` |
| F-16 | `#![warn(missing_docs)]` absent | `snora-style/src/lib.rs` |
| F-17 | No `[package.metadata.docs.rs]` | `snora-widgets/Cargo.toml` |
| F-18 | Missing metadata key | `snora/Cargo.toml` |
| F-19 | `CONTRIBUTING.md` is 12 generic lines and never links the 17-page handbook | `.github/CONTRIBUTING.md` |
| F-20 | Accessibility guide overstates what is verified | `docs/src/guides/accessibility.md` |
| F-22 | **"Four crates, one umbrella" — there are five** | `README.md:99` |
| F-23 | README quick-start compiles by luck, nothing checks it | `README.md` |
| F-25 | No troubleshooting page; five already-diagnosed errors uncollected | `docs/src/guides/` |
| F-28 | Stale crate-count / layering claims | 3 of 4 places |
| F-33 | Dead match with identical arms | — |
| F-34 | `portions()` duplicated inline | — |
| F-38 | 46 rustdoc warnings, no `cargo doc` CI gate | workspace |

## The two worth reading twice

**F-22 is in the README**, which is the crates.io front page. RFC-074 fixed that
file's *version snippets* by script a fortnight ago and nobody read the sentence
beside them. **A check that looks for one kind of staleness teaches you nothing
about the others**, and this is the second time this month that lesson has
arrived from outside.

**F-38 — 46 rustdoc warnings with no gate.** RFC-083 shipped because a docs.rs
build failed on a published crate. Forty-six warnings is the state that failure
came out of, and nothing watches it.

## Non-goals

- **No behaviour change.** If a fix here changes what the library does, it does
  not belong in this RFC.
- **No new documentation pages beyond F-25's troubleshooting collection**, which
  is assembling text that already exists in RFCs and CHANGELOG entries.
- **Do not fix F-23 by removing the README example.** Compile-check it or leave
  it; deleting the quick start to satisfy a check is the wrong direction.

## Open questions

**Q-1 — does `cargo doc` become a CI gate (F-38)?** 46 warnings must be cleared
first or the gate ships red — the trap RFC-079's check was designed around.
**Suggest: clear, then gate, in that order, and state the count after clearing.**

**Q-2 — is F-25's troubleshooting page a page or an index?** Five diagnosed
errors is thin for a page and useful as an error index. **Suggest an index**,
since each entry already has a home and duplicating them creates the drift this
project keeps finding.

**Q-3 — which of these can be dropped?** Explicitly: any of them. **If effort
runs short, land F-22, F-38 and F-19 and defer the rest** — the README is the
front page, the warnings are a shipped-artifact risk, and CONTRIBUTING is the
first thing a contributor reads.

## Acceptance criteria

1. Each item fixed **or** explicitly deferred with a reason — no silent drops.
2. F-22 corrected everywhere the claim appears, **not only the README** — sweep
   for it, per this month's repeated lesson.
3. Q-1's order respected: clear the warnings, then gate, count stated.
4. No behaviour change; `git diff` on rendering paths empty.

## Compatibility and security

**Compatibility.** Documentation and metadata. **Security.** None.
