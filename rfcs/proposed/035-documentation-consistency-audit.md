# RFC 035 — Documentation consistency and source-of-truth audit

**Status.** Proposed
**Tracks.** Cross-cutting documentation and release-governance hygiene.
Not tied to a feature; corrects drift between in-tree documentation and
the shipped four-crate reality.
**Touches.** `README.md`, `docs/src/reference/architecture.md`,
`docs/src/contributing/{architecture,design-decisions,release-process,api-freeze-review,alternate-engine-boundary}.md`,
`crates/snora/src/lib.rs` (module doc comment only), `rfcs/README.md`,
`.gitignore`, `CHANGELOG.md`.

## Summary

Snora shipped a fourth crate (`snora-design`) across v0.19–v0.20 and has
released five minors since. The code, feature graph, CI, and release
process all reflect four crates. **The architecture documentation does
not.** Two architecture pages, the README feature list, the `snora` crate
module doc, and the design-decision register still describe a three-crate
workspace, and `snora-design` appears zero times in either architecture
page.

Alongside that, an audit of the contributing set found one page that
instructs contributors to add code to the wrong crate, a release checklist
that has fallen behind its own CI, a decision register with a missing
section heading, and a `Cargo.lock` policy stated three different ways in
three places.

This RFC scopes a single, atomic documentation-consistency pass. It
changes **no code behavior and no public API** — the one source-file edit
is a module doc comment.

## Motivation

Snora's stated product value is legibility: "a small library the team
could have written and can fully read" (requirements §1.7). Its governance
value is preservation of the *why* (RFC-000). Both depend on in-tree
documents being trustworthy. The source-of-truth hierarchy in the project
handoff explicitly names `docs/src/contributing/architecture.md` as
authoritative for architecture and `design-decisions.md` as authoritative
for why the API looks the way it does. Those two files are currently
wrong about the crate count.

Concrete costs already visible:

1. **A contributor following `contributing/architecture.md` puts new
   widget code in the wrong crate.** The page's "Adding a new prefab
   widget" procedure points at `snora/src/widget/<name>.rs`; prefab
   widgets live in `snora-widgets/src/`.
2. **A downstream reader cannot discover `snora-design` from the
   architecture docs at all.** It is absent from both the consumer-facing
   and contributor-facing architecture pages.
3. **The release checklist under-specifies the gate suite** it is supposed
   to guarantee, and prescribes a tag format the project does not use.

None of this is hypothetical drift risk. It is drift that has already
landed and survived five releases.

## Goals

- G-1. Every architecture description in the repository states four
  crates with the correct dependency direction
  `snora-core ← snora-design ← snora-widgets ← snora`.
- G-2. Contributor procedures name the crate and path where the code
  actually belongs.
- G-3. The release checklist matches the gate suite CI actually runs and
  the tag format the project actually uses.
- G-4. The decision register is structurally well-formed and its index
  matches its sections.
- G-5. The `Cargo.lock` policy is stated once, consistently, in the place
  that owns it.

## Non-goals

- **N-1. No code behavior change.** The only source-file edit permitted is
  the module doc comment in `crates/snora/src/lib.rs`. No `pub` item is
  added, removed, renamed, or re-typed.
- **N-2. No re-litigating design decisions.** This RFC corrects statements
  that are *structurally broken or factually stale*. It does not revise
  any decision's reasoning, status, or reconsideration trigger.
- **N-3. No new documentation pages.** Corrections land in existing pages.
- **N-4. No design-system scope change.** Whether snora should emit an
  `iced::Theme` from `Tokens` is a separate strategic question and is not
  touched here.
- **N-5. No RFC renumbering or relocation.** Per RFC-000, numbers and
  locations of existing RFCs are stable.

## Findings and required corrections

Severity: **M** = must fix (states something false or misdirects a
contributor); **S** = should fix (stale, structurally broken, or
inconsistent but not misdirecting).

### F-1 (M) — Four-crate architecture is undocumented

| Location | Current state |
|---|---|
| `docs/src/reference/architecture.md:3` | "Snora is three crates with a strict dependency direction." Diagram omits `snora-design`. |
| `docs/src/contributing/architecture.md:9-45` | Source-layout tree omits `crates/snora-design/` and `crates/snora-widgets/src/design/`. |
| `README.md:98` | "**Three crates, one umbrella.**" |
| `crates/snora/src/lib.rs:12-36` | Layering diagram and the prose beneath it describe three crates. |
| `docs/src/contributing/alternate-engine-boundary.md:63` | Cross-reference text "Why three crates instead of two". |

`grep -c snora-design` returns **0** for both architecture pages.

**Expected corrected state.** All five locations describe four crates and
the strict direction `snora-core ← snora-design ← snora-widgets ← snora`,
noting that `snora-design` is iced-free and reached behind the opt-in
`design` feature.

### F-2 (M) — Contributor procedure names the wrong crate

`docs/src/contributing/architecture.md:92-97` — "Adding a new prefab
widget" instructs: add the function in `snora/src/widget/<name>.rs`,
declare the module in `snora/src/widget.rs`.

**Expected corrected state.** The procedure names `snora-widgets/src/`
for the implementation and re-export, and describes the `snora` facade
re-export as a separate, subsequent step. A parallel procedure for adding
a design primitive (`snora-widgets/src/design/`, re-exported via
`crates/snora/src/design.rs`) is added, since none exists today.

### F-3 (M) — Release checklist behind its own CI and tag convention

`docs/src/contributing/release-process.md`:

| Line(s) | Issue |
|---|---|
| `:91` | Prescribes `git tag vX.Y.Z`. Actual tags carry no `v` prefix (`0.25.2`), per the Rust crate convention the project follows. |
| `:48-55` | "Three workflows" table omits `build-cost.yaml`, which the same page's checklist depends on at `:98-102`. |
| `:92` | "all three jobs" — `ci.yaml` defines four (`rust-quality`, `feature-matrix`, `design-isolation`, `docs`). |
| `:80-83` | Gate list omits `cargo test -p snora-design`, `cargo test -p snora-widgets --features design`, and `mdbook test docs` — all of which CI runs and all of which the v0.25.1 evidence set records. |

**Expected corrected state.** Tag format `X.Y.Z` with no prefix; four
workflows tabulated; four CI jobs named; the gate list matches
`ci.yaml` plus the documented local suite.

### F-4 (S) — Decision register is structurally broken

`docs/src/contributing/design-decisions.md`:

- The section body beginning "Early drafts (≤ 0.3) defined a trait…"
  (from `:36`) has **no heading**. It is orphaned under "Decision index".
  The index row it belongs to is "No `PageContract` trait".
- `:244-245` — two link-reference definitions (`[TabBar]`, `[Crumb]`) sit
  inside the body of "Why `AppLayout` has both fields and a builder",
  between its heading and its first sentence.
- `:22` — index row reads "Three crates, not two" (see F-1).
- `:317-321` — states iced's extended palette "has no `warning` semantic
  pair". **Verified stale.** `iced_core-0.14.0/src/theme/palette.rs:18`
  and `:297` show a `warning` field on both the base `Palette` and
  `Extended`, generated by `Warning::generate`.

**Expected corrected state.** A `## Why no `PageContract` trait` heading
is restored; link definitions move to the foot of the file; the index row
becomes a four-crate statement consistent with F-1.

The warning-pair paragraph is corrected to state that iced 0.14 **does**
provide a warning pair, and that `WARNING_COLOR`
(`crates/snora/src/toast.rs:46`) is consequently a removal candidate
**whose disposition is deferred to RFC-038 Q-2** — because toasts render on
the design-inactive path, so replacing the constant would change
appearance for existing applications. The implementer confirms the finding
against the pinned iced source before editing and reports the confirmation;
they do **not** remove or alter `WARNING_COLOR`.

### F-5 (S) — Gate tracker header is stale

`docs/src/contributing/api-freeze-review.md:5` — "Current status
(v0.24.0)" at workspace version 0.25.2. D-3/D-4 rows cite "unchanged
v0.20–v0.24".

**Expected corrected state.** Header reflects the current released
version; D-gate rows reflect the current minor span. Gate *status* values
are not changed — closing D-3/D-4 requires a design-freeze review, which
is out of scope here.

### F-6 — **WITHDRAWN. The finding's premise was false.**

**This finding was wrong and is retained as a record of the error, not as
work to be done. No change is required for F-6.**

The finding originally asserted that `Cargo.lock` is tracked while
`.gitignore:5` ignores it and
`docs/src/contributing/architecture.md:99-104` denies it — a three-way
contradiction.

**Verified false.** `Cargo.lock` is **not** tracked:

```
$ git ls-files --error-unmatch Cargo.lock
error: pathspec 'Cargo.lock' did not match any file(s) known to git
$ git log --oneline -1 -- Cargo.lock
b7af344 remove Cargo.lock from vcs
```

The lockfile was deliberately removed from version control in `b7af344`
and has remained untracked since. Therefore `.gitignore:5` is **correct**,
and the "Why no `Cargo.lock` in version control" section is **correct**.

**Root cause of the error.** The author (architect) ran
`git ls-files Cargo.lock; echo "exit=$?"`, observed `exit=0`, and read it
as a match. `git ls-files` exits 0 whether or not it matches; the command
printed no filename. That misreading appeared to confirm "Lockfile
committed" inherited from the v0.25.1 handoff bundle — which this RFC
itself prohibits relying on ("re-derive every claim from source"). The
implementer caught it on re-verification, reverted edits already made
rather than shipping a new falsehood, and escalated instead of guessing.
That is the correct handling and is recorded here as such.

**Owner decision (received).** The untracked state stands. F-6 closes as
*no change needed*.

**Consequence — a live question opened elsewhere.** The rationale
originally drafted for F-6 (that a committed lockfile keeps budget
measurements attributable to snora's own changes) does not describe
reality. With no committed lockfile, CI resolves dependencies fresh on
every run, so the `resolver = "2"` → `"3"` switch in 0.25.2 could affect
comparability of binary-size and build-cost data points. **This is
followed up in RFC-041 and is explicitly out of scope here.**

### F-7 (S) — RFC index promises paths that do not resolve

`rfcs/README.md` documents `proposed/` and `archive/` directories; neither
existed before this RFC. Additionally, blank lines inside the Done table
(after the `019-A` row and after the `032` row) split one table into three
in rendered output.

**Expected corrected state.** The blank lines are removed so the Done
table renders as one table. The Proposed section lists this RFC. The
`archive/` line remains as forward-looking policy text, explicitly marked
as "created on first use".

### F-8 (M) — Published version 0.25.2 has no CHANGELOG entry

**Owner decision (received): 0.25.2 was published to crates.io.**

`CHANGELOG.md` has no `[0.25.2]` section and `[Unreleased]` reads "Nothing
yet". A published release is therefore entirely undocumented, violating
maintenance requirement M-3 and release-checklist step 3.

The audit also found that the release's contents are **understated by its
commit message**. `18bb51e` is titled "Documentation housekeeping: example
snora version", but `git diff 0.25.1 0.25.2` shows:

| Change | Assessment |
|---|---|
| `resolver = "2"` → `resolver = "3"` | **Material.** Switches the workspace to the MSRV-aware feature resolver. |
| `members` explicit 21-entry list → `crates/*`, `examples/*` globs | Behavioral for workspace membership: new directories now join automatically. |
| `[workspace.dependencies]` block relocated above `[profile.*]` | Cosmetic — content is byte-identical. |
| Version snippets in `README.md`, `design/feature-flags.md`, `design/overview.md` | Documentation, as the commit message implies. |

**Verified: zero source change.** No file under `crates/*/src/` differs
between 0.25.1 and 0.25.2, so the published crates are functionally
identical for downstream consumers. The `resolver` key governs only this
workspace's own builds and is not propagated to consumers. **Patch level
was therefore correct** under `versioning-policy.md`; this is a
documentation failure, not a versioning failure.

**Expected corrected state.** A retroactive `[0.25.2] — 2026-06-21`
section is added to `CHANGELOG.md` under **Changed**, describing the
resolver switch, the member globbing, and the version-snippet updates, and
stating explicitly that no source file changed and downstream behavior is
unaffected. crates.io releases are immutable; the remedy is accurate
retroactive documentation, not a re-release.

Add one further note, under the same entry or in
`docs/src/reference/binary-size-budget.md`: the budget trend series now
spans a resolver change. Because `Cargo.lock` is committed and was not
regenerated in `18bb51e`, resolved dependency versions did not change and
existing data points remain comparable — but the next lockfile
regeneration is the point at which comparability could break.

## Compatibility and migration

None. No public API, feature flag, persisted format, or runtime behavior
changes. Downstream applications are unaffected; no migration guide is
required. Per `versioning-policy.md`, a documentation-only change is a
**patch**-level concern at most.

## Security considerations

None. No new data flow, external integration, dependency, or auth logic.
The existing threat model (requirements §1.4, cross-cutting S-1…S-6)
remains valid and is not modified by this RFC.

## Operational considerations

The docs CI job (`mdbook build docs` + `mdbook test docs`) is the gate.
Any code fence added or moved must carry a classification tag per
`documentation-test-policy.md` — `rust,ignore` for app-shaped partials,
`rust,no_run` for type excerpts, bare `rust` only for self-contained
runnable examples.

## Testing and verification

| Check | Command | Why |
|---|---|---|
| Book renders | `mdbook build docs` | Structural correctness of edited pages |
| Fence policy | `mdbook test docs` | 63 chapters must stay green |
| Doc comment compiles | `cargo check -p snora --all-features` | `crates/snora/src/lib.rs` edit |
| Doctests unaffected | `cargo test -p snora --lib --all-features` | Guards the module-doc edit |
| Lint clean | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Standard gate |
| Claim re-verification | `grep -rn "three crates" --include='*.md' --include='*.rs' .` | Must return zero non-historical hits |

No new tests are required: this RFC asserts nothing testable at runtime.

## Alternatives considered

- **Fix opportunistically, page by page, as each is next touched.**
  Rejected: this is how five releases of drift accumulated. RFC-000's
  migration guidance explicitly prefers "a single dedicated change rather
  than spreading it across unrelated commits."
- **Add a CI documentation-consistency check instead of fixing prose.**
  Rejected as the *primary* response — a linter cannot know that four
  crates is correct. Worth revisiting as a follow-up once the prose is
  right (see Open questions, Q-3).
- **Delete the stale architecture pages and rely on the handoff bundle.**
  Rejected: the handoff bundle is explicitly a snapshot, and the
  source-of-truth hierarchy names the in-tree pages as authoritative.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Editing prose introduces a new factual error | Medium | Medium | Every corrected claim is re-derived from source, not from the handoff bundle; the handoff is a snapshot and loses to in-tree files |
| Scope creep into rewriting design rationale | Medium | Medium | N-2 forbids it; the handoff enumerates exactly which sentences may change |
| The warning-palette claim is "corrected" by guess | Medium | Low | F-4 requires API verification first, with an explicit no-guess instruction |
| `.gitignore` change alters what future contributors commit | Low | Medium | Gated on owner decision Q-2 |

## Open questions

- **Q-1 (owner) — ANSWERED.** `0.25.2` was published to crates.io. F-8 is
  unblocked and becomes a retroactive `[0.25.2]` CHANGELOG section.
- **Q-2 (owner) — ANSWERED, on corrected facts.** The question was posed
  on a false premise (see F-6). `Cargo.lock` is untracked and stays
  untracked. F-6 closes as *no change needed*.
- **Q-3 (follow-up).** Should a lightweight docs-consistency CI check be
  added (crate count, dead relative links, RFC index integrity)? Deferred
  to a separate RFC; RFC-000 §"Optional CI invariants" sketches the shape.
- **Q-4 (follow-up, raised by this audit).** `18bb51e` shipped a material
  build-configuration change under a commit message describing
  documentation work, and the gap went unnoticed until this audit. Worth
  considering whether the release checklist should require the CHANGELOG
  entry to be written *before* the version bump rather than alongside it.
  Not scoped here.

## Acceptance criteria

1. `grep -rn "three crates"` over `*.md` and `*.rs` returns hits only in
   historical contexts (CHANGELOG entries, migration guides describing
   the v0.6 split), never as a present-tense claim.
2. Both architecture pages name `snora-design`, state its iced-free
   status, and show the four-crate dependency direction.
3. `contributing/architecture.md` widget procedure names
   `snora-widgets/src/`; a design-primitive procedure exists.
4. `release-process.md` states tag format `X.Y.Z`, tabulates four
   workflows, names four CI jobs, and lists the full gate suite.
5. `design-decisions.md` has a heading for every index row; no link
   definitions sit inside a section body.
6. `api-freeze-review.md` header and D-gate rows cite the current version.
   No gate status value changed.
7. `rfcs/README.md` Done table renders as one table; Proposed lists
   RFC-035.
8. F-6 requires no change: `.gitignore` and the "Why no `Cargo.lock` in
   version control" section are both left exactly as they were. Any edit
   to either is a defect.
9. F-8 is resolved: `CHANGELOG.md` carries a `[0.25.2]` section naming the
   resolver switch and member globbing, and stating that no source file
   changed.
10. `mdbook build docs`, `mdbook test docs`, `cargo check -p snora
    --all-features`, and workspace clippy all pass.

## Implementation boundaries

One Developer Handoff, implemented as a single reviewable change. The
implementer may not: touch any `pub` item; alter any decision's Status or
reconsideration trigger; change any 1.0 or D-gate status value; add
documentation pages; or act on F-6/F-8 before the owner answers Q-1/Q-2.

## Release implications

Documentation-only. Ships as a patch or folds into the next minor at the
owner's discretion. Does not advance or close any 1.0 gate. It does
improve the evidence position for the eventual design-freeze review by
making the architecture record accurate first.
