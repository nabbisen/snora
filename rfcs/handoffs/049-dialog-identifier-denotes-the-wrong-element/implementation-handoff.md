# Developer Handoff — RFC-049 `snora-dialog-card` denotes the wrong element

**Governing RFC.** **RFC-049** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-049 — Implemented (v0.29.0).
**Release target.** 0.29.0 (**minor** — an identifier rename is a minor bump
by the policy RFC-047 shipped in v0.28.0).
**Implementation units.** One. **Do not start before RFC-048 has landed** —
see §4.

---

## 1. Task title

Attach `snora-dialog` to the dialog's centring container and re-point
`snora-dialog-card` at the actual card, which currently carries no identifier
at all.

## 2. Purpose

RFC-047 shipped stable identifiers so downstream teams could observe a snora
application, and stated that **the names are the contract**. The dialog is the
one surface where that failed.

`snora-dialog-card` is attached to the `center(...)` wrapper. In iced 0.14,
`center(content)` is `container(content).center(Length::Fill)`
(`iced_widget-0.14.2/src/helpers.rs:259`) — a **window-filling** container.
So:

- **Default path:** the identifier is on a full-window transparent container.
  No card exists.
- **`design` path:** the identifier is *still* on the wrapper. The real card —
  the inner `container(...)` with padding, fill, border and radius — has **no
  identifier at all.**

A downstream test resolving `snora-dialog-card` gets **window-sized bounds**
on both paths. For screenshot-diffing and outside-driving — the use case
RFC-047 existed for — that is a wrong answer, not a vague label.

## 3. Background — read first

- `rfcs/done/049-dialog-identifier-denotes-the-wrong-element.md` in full,
  especially §"The one real risk: silent repurposing".
- `rfcs/done/047-stable-identifiers-on-rendered-surfaces.md` — the policy you
  are exercising for the first time.
- `docs/src/contributing/versioning-policy.md` §"Rendered surface
  identifiers".

**Verify the placement claim yourself before changing anything.** Read
`crates/snora/src/overlay/dialog.rs` and iced's `center` helper. If the code
has moved on, stop and escalate rather than implementing from this
description.

## 4. Ordering with RFC-048 — read before starting

RFC-048 corrects doc comments in `crates/snora/src/overlay/dialog.rs` and
ships as **0.28.1**. This RFC edits the same file and ships as **0.29.0**.

**RFC-048 lands first.** It is a patch and must not wait behind a minor. Do
not run these in parallel on the same branch.

## 5. The part to get right

The wiring is a handful of lines. **The failure mode is silent.**

`snora-dialog-card` keeps its spelling and changes its referent, so a
downstream test written against 0.28.0 would not fail — it would quietly start
receiving card bounds instead of window bounds. That is exactly what RFC-047
warned about, and it is accepted here for **one reason only: no consumer is on
0.28.0 yet** (apimokka is on 0.25.2, arama on 0.25.0).

**Before you implement, re-check that assumption** — RFC-049 Q-1. If any
consumer has adopted 0.28.0 identifiers, **stop and escalate**: the design
changes to retiring the string entirely and giving the card a third name, so
their assertion fails loudly instead of changing meaning underneath them.

## 6. Change scope

| File | Purpose |
|---|---|
| `crates/snora/src/identifiers.rs` | the new constant; split the registry (§8 step 2) |
| `crates/snora/src/overlay/dialog.rs` | attach both identifiers |
| `crates/snora/src/identifiers/tests.rs` | **the existing test will break — §7** |
| `crates/snora/src/design/render/tests.rs` | the card-identifier test |
| `docs/src/reference/rendered-surface-identifiers.md` | both rows |
| `docs/src/contributing/versioning-policy.md` | worked example |
| `docs/src/guides/migration-0.28-to-0.29.md` (new) | migration section |
| `CHANGELOG.md` | `[Unreleased]` **Changed** |

## 7. The trap that will cost you an hour

`crates/snora/src/identifiers/tests.rs:41` —
`identifiers_present_in_rendered_output` — iterates **`ALL_STATIC`** against
`crate::render::render(layout)`, the **default path**:

```rust,ignore
for name in ALL_STATIC {
    sim.find(Id::new(name))
        .unwrap_or_else(|_| panic!("expected to find a widget with id {name:?}"));
}
```

The moment `snora-dialog-card` becomes `design`-path-only, this test **fails**
— `ALL_STATIC` will contain a name the default path does not emit.

**Do not fix that by deleting the assertion or by removing the card from
`ALL_STATIC` silently.** `ALL_STATIC` also feeds
`documented_identifiers_match_emitted_set`, which compares it against the
reference page; dropping the card there would make the drift test pass while
the page documents an identifier no constant tracks.

Split the registry instead (§8 step 2).

## 8. Required implementation

### Step 1 — Attach

```rust,ignore
// crates/snora/src/overlay/dialog.rs
None => center(dialog.content).id(crate::identifiers::DIALOG).into(),
Some(card) => center(
        container(dialog.content)
            .padding(card.padding)
            .style(move |_theme| card.style)
            .id(crate::identifiers::DIALOG_CARD),   // the card itself
    )
    .id(crate::identifiers::DIALOG)
    .into(),
```

- `snora-dialog` — the centring container, **both** paths, always present.
- `snora-dialog-card` — the styled container, **`design` path only**.

### Step 2 — Split the registry

`ALL_STATIC` currently lists every static identifier and is `#[cfg(test)]`.
Split it so each test asks the right question:

- an always-emitted set — used by `identifiers_present_in_rendered_output`;
- a `design`-path-only set;
- their **union** — used by `documented_identifiers_match_emitted_set`, so the
  reference page must still list *every* identifier snora can emit.

Name them so the distinction is obvious at the call site. Keep the existing
doc comment's explanation of why the constant is `#[cfg(test)]`.

### Step 3 — Test the defect this RFC exists to fix

An identifier-presence test is **not sufficient**. The old identifier was
"present" too — on the wrong element.

Add a test asserting `snora-dialog-card` resolves to the **styled card
container** and not the window-filling wrapper. Distinguish them by a property
only the card has — its bounds being smaller than the viewport, or its
`padding`/`style` taking effect. If `iced_test`'s simulator cannot express
that distinction, **say so in the review request** and state what you asserted
instead; do not claim coverage you do not have.

This test is the point of the exercise. Without it the same defect can return.

### Step 4 — Reference page

Two rows. State plainly that `snora-dialog-card` is present **only** on the
`design` path, and that `snora-dialog` is the unconditional name for "the
dialog, whichever path".

### Step 5 — Versioning policy

Record this as the **worked example** of an identifier change being a minor
bump — the policy's first exercise. One short paragraph: what changed, why a
rename was preferred to keeping a wrong name, which release.

### Step 6 — Migration guide and CHANGELOG

`docs/src/guides/migration-0.28-to-0.29.md`, registered in
`docs/src/SUMMARY.md` and the migrations index. It must state the direction of
travel explicitly:

| Was asserting | Now |
|---|---|
| dialog presence / position | `snora-dialog` |
| the card's appearance or bounds | `snora-dialog-card` (`design` path only) — previously returned window bounds and was wrong |

`CHANGELOG.md` **Changed**: state that `snora-dialog-card`'s **referent**
changed, not merely that identifiers were added. A reader skimming for
breakage must be able to see it.

## 9. Explicit non-change scope

Do **not**:

- Touch any other identifier. The other eight and `snora-toast-{id}` are
  correct.
- Change any rendering. `render_semantics` must pass **unmodified**.
- Feature-gate the *identifier*. The `design` gate applies to **the element**
  — there is no card by default to label. Do not add `#[cfg(feature)]` around
  a constant to make the sets line up.
- Add identifiers to application content (RFC-047 N-4).
- Build a test harness. Unchanged firm non-goal.
- Run `cargo fmt --all` (§10).

## 10. Required tests

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora --lib --all-features
cargo test -p snora --lib --no-default-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features
mdbook build docs && mdbook test docs
```

**Both feature configurations are mandatory**, not optional: the whole point
is that one identifier is now conditional, and a test suite run only with
`--all-features` cannot see the difference.

### On `cargo fmt` — resolved in 0.28.1

**Superseded.** The drift described below was cleared by a workspace-wide
`cargo fmt --all` in 0.28.1, and CI now runs `cargo fmt --all --check` on
every PR and push. **Use the plain gate:**

```bash
cargo fmt --all --check      # must pass
```

The delta procedure below is kept as the historical record of why it was
needed; it is no longer the instruction.

### (historical) The standing gate was broken

`cargo fmt --check` does **not** pass on a clean tree at v0.28.0: **82 hunks**
workspace-wide, **7 under `-p snora`, all in
`crates/snora/tests/render_semantics.rs`** — the file you must leave
unmodified. No CI workflow runs `cargo fmt`. Use a delta check:

```bash
cargo fmt --check --all 2>/dev/null | grep '^Diff in' | sort > /tmp/fmt-before
# … make your changes …
cargo fmt --check --all 2>/dev/null | grep '^Diff in' | sort > /tmp/fmt-after
diff /tmp/fmt-before /tmp/fmt-after      # MUST be empty
```

New code you write must of course be `rustfmt`-clean; the delta is what
proves it without dragging in the pre-existing drift.

## 11. Acceptance criteria

RFC-049 §Acceptance criteria 1–8:

1. `snora-dialog` on the centring container, both paths.
2. `snora-dialog-card` on the styled card, `design` path only, and on nothing
   by default.
3. **A test asserts the card identifier resolves to the styled container, not
   the wrapper** (§8 step 3).
4. The drift test passes under both feature configurations (§10).
5. `render_semantics` passes unmodified.
6. Reference page documents both, with the card's conditional presence.
7. `versioning-policy.md` carries the worked example.
8. `CHANGELOG.md` states the referent change plainly.

## 12. Prohibited shortcuts

- **Do not delete or weaken `identifiers_present_in_rendered_output`** to make
  it pass (§7). Split the registry.
- Do not drop `snora-dialog-card` from the documented set to silence the drift
  test.
- Do not ship step 1 without step 3. An identifier that is present but on the
  wrong element is precisely the bug being fixed, and only step 3 catches it.
- Do not modify `render_semantics.rs`, including formatting.
- Do not reuse `snora-dialog-card` for the wrapper "for compatibility". That
  preserves the defect.

## 13. Compatibility and security

**Compatibility.** **Breaking for identifier consumers**, no Rust API change —
nothing fails to compile. This is the first deliberate exercise of the
identifier compatibility policy; the migration guide and CHANGELOG are how the
cost is paid, per the owner's ruling that a rename is acceptable where a much
better name exists.

State explicitly in the review request that the rename is a **silent** change
for anyone already asserting on 0.28.0 identifiers, and confirm §5's
re-check result.

**Security.** No new data flow, dependency, or integration.

## 14. Required evidence

- Diff of `overlay/dialog.rs` and `identifiers.rs` in full.
- The registry split, with the reasoning for the chosen names.
- **Step 3's test and its output** — the single most important artifact here.
- Test output for **both** feature configurations.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it is **empty**.
- The fmt delta (§10).
- Your §5 re-check: evidence that no known consumer asserts on 0.28.0
  identifiers, or escalation if one does.

## 15. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/049-dialog-identifier-denotes-the-wrong-element/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** step 3 — whether the test genuinely distinguishes
the card from the wrapper, or merely asserts presence again. Presence was
never the problem.
