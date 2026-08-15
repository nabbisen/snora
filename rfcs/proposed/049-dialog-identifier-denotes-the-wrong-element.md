# RFC 049 — `snora-dialog-card` denotes the wrong element

**Status.** Proposed
**Tracks.** Rendered-surface identifiers (RFC-047). Corrects a defect shipped
in v0.28.0. Split out of [RFC-048](../done/048-dialog-card-documentation-contradiction.md) F-5
because it is a **minor** bump and RFC-048 is a patch.
**Touches.** `crates/snora/src/identifiers.rs`,
`crates/snora/src/overlay/dialog.rs`,
`docs/src/reference/rendered-surface-identifiers.md`,
`docs/src/contributing/versioning-policy.md`, `CHANGELOG.md`, plus a
downstream handoff note.
**Release target.** 0.29.0 (minor — an identifier rename is a minor bump by
the policy RFC-047 shipped).

## Summary

`snora-dialog-card` is attached to the dialog's **full-window centring
container**, not to the card. On the default path no card exists; on the
`design` path the card exists and carries **no identifier at all**.

A downstream test resolving `snora-dialog-card` therefore gets window-sized
bounds on both paths. For the use case RFC-047 existed to serve —
screenshot-diffing and driving a GUI from outside — that is a wrong answer,
not an imprecise label.

This RFC renames the wrapper to what it is and gives the card its own
identifier.

## Motivation

RFC-047 introduced stable identifiers so downstream teams could observe a
snora application. It named the naming itself as the part to get right:

> The wiring is a line per surface. **The names are the contract.**

The dialog is the one surface where that did not happen. From
`crates/snora/src/overlay/dialog.rs`:

```rust,ignore
None       => center(dialog.content).id(DIALOG_CARD),
Some(card) => center(container(dialog.content).padding(..).style(..))
                  .id(DIALOG_CARD),   // <- id is on the wrapper, not the card
```

In iced 0.14, `center(content)` is `container(content).center(Length::Fill)`
(`iced_widget-0.14.2/src/helpers.rs:259`) — a container that fills the
window. Verified by reading iced's source, not assumed.

snora already documents the discrepancy while keeping the misleading name.
`docs/src/reference/rendered-surface-identifiers.md` describes the identifier
as *"The dialog's centered container"*, and `identifiers.rs:53` says the same.
**The prose is accurate; the name contradicts it.** That is precisely the
defect arama reported one layer up (RFC-048) — a name asserting something the
code does not render.

## Why rename rather than document around it

RFC-048 Q-1 originally recommended keeping the name, on the grounds that
moving a compatibility surface one release after establishing it teaches
downstream teams the guarantee is soft. The owner overruled it:

> you can rename if much better name exists. `CHANGELOG.md` or handoffs are
> available for help to app dev teams.

That is correct, and the placement finding makes it the only defensible
option. **A compatibility guarantee is a promise not to move a name
gratuitously. It is not a reason to keep a name that resolves to the wrong
element.** Honouring a wrong name is not stability; it preserves a defect and
adds a migration to whoever eventually depends on it.

The cost is near zero **now** and only grows:

- v0.28.0 shipped 2026-08-04; this RFC targets the next minor.
- Both known consumers are behind it — apimokka on 0.25.2, arama on 0.25.0.
- **No known consumer asserts on these identifiers yet.**

That window closes the moment either team upgrades. Fixing it now costs a
CHANGELOG entry; fixing it in three releases costs someone's test suite.

## Design

| Identifier | Element | Presence |
|---|---|---|
| `snora-dialog` | The dialog's centring container — the full-window layer that positions the dialog. | **Always**, both paths |
| `snora-dialog-card` | The card itself: the container carrying padding, fill, border and radius. | **`design` path only** — the element does not exist by default |

`snora-dialog-card` is **retained as a name and re-pointed** at the element it
always claimed to denote. Everything else about RFC-047 is unchanged.

### On conditional presence

RFC-047 Q-2 established that identifiers are always-on, never feature-gated,
because "a feature gate would make tests pass or fail depending on feature
selection."

`snora-dialog-card` is now conditional, and that does **not** violate the
principle. The identifier is not gated — **the element is.** There is no card
on the default path to label. Labelling a full-window wrapper "card" so that
the name is always present is what produced this defect: it bought presence
by making the answer wrong.

`snora-dialog` is always present, so a test that wants "the dialog, whichever
path" has an unconditional name to use.

### The one real risk: silent repurposing

`snora-dialog-card` keeps its spelling and changes its referent. A downstream
test written against 0.28.0 would not fail — it would silently start
receiving card bounds instead of window bounds.

That is the failure mode RFC-047 warned about, and it is accepted here for
one reason only: **no consumer is on 0.28.0.** The alternative — retiring the
string and introducing a third name so stale tests fail loudly — buys nothing
when there are no stale tests, and permanently spends the best name.

**If evidence appears that any consumer is asserting on 0.28.0 identifiers,
this decision must be revisited before release**, and the loud-failure
variant taken instead. That is Q-1.

## Non-goals

- **No change to any other identifier.** The other eight and the per-toast
  scheme are correct.
- **No rendering change.** `render_semantics` must pass unmodified.
- **No new identifiers beyond the card.** Application content stays the
  application's to label (RFC-047 N-4).
- **No test harness.** Unchanged firm non-goal.

## Open questions

**Q-1 — Is silent repurposing still acceptable at implementation time?**
Re-check before release that no consumer has adopted 0.28.0 identifiers. If
one has, retire `snora-dialog-card` instead of re-pointing it and give the
card a fresh name, so their assertion fails loudly rather than quietly
changing meaning.

**Q-2 — Does the drift test cover conditional identifiers?**
RFC-047's `documented_identifiers_match_emitted_set` compares the reference
page against the emitted set. A `design`-path-only identifier must not make
that test fail under `--no-default-features`, and must not be silently
skipped either. State how it is handled; do not let the conditional case go
untested.

## Acceptance criteria

1. `snora-dialog` is attached to the centring container on both paths.
2. `snora-dialog-card` is attached to the styled card container, `design`
   path only, and to nothing on the default path.
3. A test asserts the card identifier's element is the **styled container**,
   not the window-filling wrapper — the defect this RFC exists to fix must be
   caught by a test, not only corrected.
4. The drift test passes under both default and `--no-default-features`
   (Q-2).
5. `cargo test -p snora --test render_semantics` passes **unmodified**.
6. The reference page documents both identifiers and states the card's
   conditional presence.
7. `versioning-policy.md` records this rename as the worked example of an
   identifier change being a minor bump.
8. `CHANGELOG.md` states plainly, under **Changed**, that
   `snora-dialog-card`'s referent changed and what it now denotes.

## Compatibility and migration

**Breaking for identifier consumers only.** No Rust API changes; nothing fails
to compile.

Migration is one line per assertion, and the guide must say which direction:

| Was asserting | Now |
|---|---|
| dialog presence / position | `snora-dialog` |
| the card's appearance or bounds | `snora-dialog-card` (`design` path only) — this previously returned window bounds and was wrong |

Per the owner's ruling, **`CHANGELOG.md` and the downstream handoffs carry
this**, rather than the rename being avoided. The 0.28→0.29 migration guide
gets a section, and the app-team handoff bundle a note.

## Security considerations

None. No new data flow, dependency, or integration.

## Release implications

**0.29.0, minor** — an identifier rename is a minor bump under
`versioning-policy.md`, which shipped in v0.28.0 alongside the defect. This
RFC is the policy's first exercise, and it works as intended: the rename is
possible, priced, and announced rather than either forbidden or silent.
