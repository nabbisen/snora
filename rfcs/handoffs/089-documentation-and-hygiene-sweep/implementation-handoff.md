# Developer Handoff — RFC-089 documentation and hygiene sweep

**Governing RFC.** [RFC-089](../../accepted/089-documentation-and-hygiene-sweep.md)
**Status.** Accepted (owner, 2026-09-01). Medium/Low.
**Release target.** **0.41.1** — documentation and metadata. **No public API,
no behaviour.**
*(Re-targeted 2026-09-01: this said 0.40.2, which is in the past. Ships with
RFC-087; both are documentation-and-CI only.)*
**Implementation units.** Thirteen, independent. Any order. **Some may be
dropped — see §4.**

---

## 1. The set

| # | What | Where |
|---|---|---|
| F-07 | Docs describe animation/transitions that do not exist | `snora-core/src/overlay.rs` |
| F-16 | `#![warn(missing_docs)]` absent | `snora-style/src/lib.rs` |
| F-17 | No `[package.metadata.docs.rs]` | `snora-widgets/Cargo.toml` |
| F-18 | Missing metadata key | `snora/Cargo.toml` |
| F-19 | `CONTRIBUTING.md` is 12 generic lines, never links the handbook | `.github/CONTRIBUTING.md` |
| F-20 | Accessibility guide overstates what is verified | `docs/src/guides/accessibility.md` |
| **F-22** | **"Four crates, one umbrella" — there are five** | **`README.md:99`** |
| F-23 | README quick start compiles by luck | `README.md` |
| F-25 | No troubleshooting collection | `docs/src/guides/` |
| F-28 | Stale crate-count / layering claims | 3 of 4 places |
| F-33 | Dead match, identical arms | — |
| F-34 | `portions()` duplicated inline | — |
| **F-38** | **46 rustdoc warnings, no `cargo doc` gate** | workspace |

## 2. F-22 is not one line

It is in the **README**, which is the crates.io front page. RFC-074 fixed that
file's *version snippets* by script a fortnight ago and nobody read the sentence
beside them.

**Sweep for the claim everywhere before fixing the one you were shown.** F-28
says three of four places carry it. This month that lesson has arrived from
outside twice; do not make it three.

## 3. Q-1 ruled — clear, then gate, in that order

46 rustdoc warnings and no `cargo doc` gate. **Clearing must come first**: a
gate that ships red is one people learn to ignore — RFC-079's trap.

**State the count after clearing.** If it is not zero, do not add the gate; say
what remains and why.

RFC-083 shipped because a docs.rs build failed on a published crate. Forty-six
warnings is the state that came out of.

## 4. Q-3 ruled — what to drop if effort runs short

**Land F-22, F-38 and F-19 first.** The README is the front page; the warnings
are a shipped-artifact risk; CONTRIBUTING is the first thing a contributor
reads.

**Anything else may be deferred — explicitly, in the report, with a reason.**
Silent drops are the failure mode here, not incompleteness.

## 5. Q-2 ruled — F-25 is an index, not a page

Five diagnosed errors is thin for a page and useful as an error index. **Point
at where each is already explained**; do not restate them. Duplicated text is
the drift this project keeps finding.

## 6. Explicit non-change scope

- **No behaviour change.** If a fix changes what the library does, it does not
  belong here — stop and report.
- **Do not delete the README quick start to satisfy F-23.** Compile-check it or
  leave it.
- **No new documentation pages** beyond F-25's index.

## 7. Required evidence

- Each item: fixed, or deferred with a reason. **Thirteen outcomes, none
  omitted.**
- F-22's sweep, showing every location found
- Rustdoc warning count before and after; the gate only if it reached zero
- `git diff` on rendering paths — **expected empty**

## 8. Acceptance criteria

1. Thirteen outcomes reported; no silent drops.
2. F-22 corrected everywhere, not only the README.
3. Warnings cleared before any gate; count stated.
4. F-25 is an index, not duplicated text.
5. No behaviour change.

## 9. Required review-request format

`.git-exclude/review-request/089-documentation-and-hygiene-sweep/`, `README.md`
entry point, evidence under `evidence/`.

**Requested review focus: F-22's sweep and the thirteen outcomes.** The
individual fixes are trivial; whether any were dropped quietly is the only thing
that can go wrong.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
