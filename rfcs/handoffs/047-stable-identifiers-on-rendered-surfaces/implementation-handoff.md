# Developer Handoff — RFC-047 stable identifiers on rendered surfaces

**Governing RFC.** [RFC-047](../../proposed/047-stable-identifiers-on-rendered-surfaces.md)
**Status.** Inherited from RFC-047 (Proposed; accepted by the owner).
**Release target.** 0.28.0, alongside RFC-046.
**Implementation units.** One. **Read §4 before starting — this and
RFC-046 both touch `render.rs`.**

---

## 1. Task title

Attach stable, documented `iced::widget::Id`s to the surfaces snora renders
itself, and record them as a compatibility surface.

## 2. Purpose

A snora application is externally unobservable. A downstream team building
scripted GUI verification found no widget identifiers, no semantic names,
and no state query — the only readable signal was the window title, which
they were using as an accessibility API because nothing else existed.

The surfaces snora renders are precisely the ones an application **cannot**
label itself: it never sees the modal dim, the menu backdrop, or the card
wrapping its dialog content. The gap is exactly snora-shaped.

## 3. Background — read first

- `rfcs/proposed/047-stable-identifiers-on-rendered-surfaces.md` in full,
  especially §"The real cost is not the code".
- `docs/src/contributing/versioning-policy.md` — you will be adding to it.

`container(...).id(...)` exists in iced 0.14
(`iced_widget-0.14.2/src/container.rs:109`) and
`iced_core::widget::Id` is public. Verified.

Conventions: English only; Rust 2018+ modules; `cargo fmt` **scoped to
`snora`** (~152 hunks of pre-existing drift).

## 4. Ordering with RFC-046 — read before starting

RFC-046 (width exposure) also modifies `crates/snora/src/render.rs`. They
are **semantically independent** — that one wraps composition, this one
labels surfaces inside it — but they conflict textually.

Do **not** run them in parallel on the same branch. **Landing this one
first is preferred**: it is the smaller edit, adding labels inside existing
composition rather than restructuring it.

Whichever lands second re-runs `render_semantics` and confirms it still
passes unmodified.

## 5. The part to get right

The wiring is a line per surface. **The names are the contract.**

Once a downstream test asserts on `snora-modal-dim`, renaming it breaks
that test **silently at runtime** — not at compile time, the way a Rust API
break would. That asymmetry is why the naming and the policy matter more
than the code here.

Treat the identifiers as public API from the first commit.

## 6. Change scope

| File | Purpose |
|---|---|
| `crates/snora/src/render.rs` | backdrops, dim, skeleton regions |
| `crates/snora/src/overlay/dialog.rs` | dialog card / centred container |
| `crates/snora/src/overlay/sheet.rs` | sheet panel |
| `crates/snora/src/toast.rs` | toast stack + per-toast |
| `crates/snora/src/…/tests.rs` | identifier + drift tests |
| `docs/src/reference/` (new page) | the identifier reference list |
| `docs/src/contributing/versioning-policy.md` | identifiers as a compatibility surface |
| `docs/src/SUMMARY.md` | register the reference page |
| `docs/src/guides/` | short note on what this does and does not provide |
| `CHANGELOG.md` | `[Unreleased]` **Added** |

## 7. Scope — what gets an identifier

**In** — surfaces snora composes:

| Surface | File |
|---|---|
| menu backdrop (transparent click sink) | `render.rs` |
| modal dim — **both** the click-capturing and non-capturing variants | `render.rs` |
| dialog card / centred container | `overlay/dialog.rs` |
| sheet panel | `overlay/sheet.rs` |
| toast stack container | `toast.rs` |
| each individual toast | `toast.rs` |
| skeleton regions (header / sidebar / body / footer) | `render.rs` |

**Out** — anything the application supplies. Slot *contents* are the
application's elements and the application's to identify.

Note the dim has **two** variants (`dim_backdrop` and
`dim_without_capture`). Both need identifying, and the review request
should say whether they share a name or differ — a test looking for "the
dim" probably wants both; a test checking click-capture behaviour may not.

## 8. Required implementation

### Step 1 — Naming convention

Propose one and apply it uniformly. Recommended: a `snora-` prefix,
kebab-case (`snora-modal-dim`, `snora-dialog-card`, `snora-toast-stack`).

The **prefix is not decoration** — it keeps snora's identifiers
distinguishable from the application's own in a tree the application also
populates, which prevents collisions.

Per-toast identifiers need a discriminator. `Toast` already carries a `u64`
id; `snora-toast-{id}` is the obvious derivation and lets a test find a
*specific* toast rather than only the stack. **Confirm the derived form is
stable across renders** before relying on it — if the id can change for the
same logical toast, say so rather than shipping an unstable identifier.

State your convention in the review request. This is RFC-047 Q-1.

### Step 2 — Attach

Always-on, not feature-gated (RFC-047 Q-2). An `Id` has no rendering effect,
and a feature gate would make tests pass or fail depending on feature
selection — the class of feature-dependent behaviour this project avoids.

If implementation reveals a real cost, flag it; the binary-size probes now
measure correctly (RFC-043) and can settle it.

### Step 3 — Reference page **and a drift test**

Add a reference page listing every identifier snora emits.

**Then write a test asserting the documented list matches the emitted
set.** This is the test that earns its keep. A hand-maintained identifier
list will drift, and a stale reference is worse than none — a downstream
test would assert on a name that no longer exists, and fail in a way that
looks like an application bug.

### Step 4 — Versioning policy

Add to `versioning-policy.md`: **snora-emitted identifiers are a
compatibility surface.** Renaming or removing one is a **minor**, not a
patch.

This is an acceptance criterion, not a follow-up. It must ship *with* the
identifiers — otherwise the first rename gets treated as a patch, and the
commitment is worthless.

### Step 5 — Guide note

Short, and honest about the boundary: this provides **labels on
snora-rendered output**, not a test harness, not a state query, and not
accessibility semantics. An `Id` is not a role.

## 9. Explicit non-change scope

Do **not**:

- Build a test harness or anything resembling `snora-test`. **N-8 is a
  firm non-goal.** Labels on output are not a harness.
- Add a state query API. Larger question, not addressed here.
- Add accessibility roles, names or states. That needs an accessibility
  tree — RFC-045, blocked on iced. **An `Id` is not a role**; do not let
  the reference page imply otherwise.
- Identify application content.
- Change any rendered appearance or behaviour.

## 10. Required tests

```bash
cargo fmt --check -p snora
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features
mdbook build docs && mdbook test docs
```

New tests:

| Test | Assertion |
|---|---|
| Identifiers present | Each surface in §7 emits its documented identifier |
| **Documentation drift** | The reference page's list and the emitted set agree |
| Toast identifier stability | The same toast yields the same identifier across renders |
| Appearance unchanged | `render_semantics` passes unmodified |

## 11. Acceptance criteria

RFC-047 §Acceptance criteria 1–5:

1. Every surface in §7 emits a documented identifier.
2. A reference page lists them, **and a test proves the list matches
   reality**.
3. `versioning-policy.md` records identifiers as a compatibility surface.
4. `render_semantics` passes unmodified; no appearance change.
5. Naming convention stated and applied uniformly.

## 12. Prohibited shortcuts

- Do not ship the reference page without the drift test. A list nobody
  checks will be wrong within two releases.
- Do not defer the versioning-policy change.
- Do not skip the two dim variants because they look alike.
- Do not modify a render-semantics test to make it pass.

## 13. Compatibility and security

**Compatibility.** No appearance or behaviour change. But state explicitly
in the review request that **from this release the identifiers are a
compatibility surface** — that is the change's lasting consequence.

**Security.** No new data flow, dependency, or integration. Worth one
sentence: identifiers make a running application more observable to
anything inspecting its widget tree. For a local-first desktop framework
that is the intent, not a leak — but say so deliberately rather than
leaving it unconsidered.

## 14. Required evidence

- Diffs of all four source files.
- The reference page in full.
- Drift-test output.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it is **empty**.
- The `versioning-policy.md` diff.
- Your naming convention and the toast-stability finding.

## 15. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/047-stable-identifiers-on-rendered-surfaces/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** the names themselves. They are the part that
cannot be changed cheaply later, and they are worth more review attention
than the code that emits them.
