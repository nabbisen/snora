# Developer Handoff — RFC-060 frame-level keyboard navigation

**Governing RFC.** [RFC-060](../../done/060-frame-level-keyboard-navigation.md)
**Status.** Inherited from RFC-060.
**Release target.** 0.35.0 (minor — `snora-core` gains public vocabulary).
**Implementation units.** One. Independent of RFC-050, which may ship alongside.

---

## 1. Task title

Add frame-level zone navigation as pure iced-free vocabulary in `snora-core`,
a direction helper in `snora::keyboard`, and retire the fourteen-minor-stale
focus-trapping deferral across all six documentation sites.

## 2. Purpose

Snora owns the frame; applications own what is inside a pane. Snora supplies
**none** of its half of keyboard navigation today, so every multi-region
application reimplements identical logic — tekstide did, and told us so.

## 3. The two design positions that are not yours to revisit

Both are settled in the RFC. They are stated here because both are things an
implementer would reasonably "fix" on the way past.

**Snora does not take Tab or Shift+Tab.** Tab means "next control" to iced and
to every user. Claiming it for zone cycling breaks in-pane navigation for every
application with a form. Supply the **decision**, take a direction rather than a
key, and recommend **F6 / Shift+F6** in the docs.

**Snora captures no keyboard events.** No subscription is installed by snora.
`snora::keyboard::dismiss_on_escape` is the template: a pure function taking the
key and the relevant state, returning `Option<_>`, with the application wiring
`iced::keyboard::listen()` itself. Match that shape.

## 4. What to build

### 4.1 `crates/snora-core/src/focus.rs` (new)

**Iced-free.** `snora-core` has no iced dependency and this must not be the
change that adds one — that is a hard architectural constraint, not a
preference.

Vocabulary: a zone enum over the four skeleton slots, a cycle direction, and a
description of which optional slots are present. `body` is required by
`AppLayout`, so it is always present and should not be expressible as absent.

The decision function returns the next zone in **logical** order —
`Header → SideBar → Body → Footer`, wrapping — skipping absent slots.

**Do not add a `LayoutDirection` parameter.** The order is logical, not visual,
so it is direction-independent: under RTL the sidebar renders on the opposite
edge but is still the start-edge rail following the header. `ToastPosition`
needs mirroring; this does not. The RFC records this deliberately so it is not
"fixed" later.

Tabs and breadcrumbs are **not zones** — `crumb` and `tab` are content inside a
slot.

Follow `snora-core`'s existing convention: `#[cfg(test)] mod tests` inline at
the bottom of the module (see `crumb.rs`, `tab.rs`, `toast.rs`), not a separate
file — that is `snora-design`'s convention, not this crate's.

Register the module in `crates/snora-core/src/lib.rs` alongside the others.

### 4.2 Containment while an overlay is open

- **Modal open (`dialog` or `sheet`)** — cycling is **suspended**. Report it;
  do not silently return `Body`. Modal contents are an application-supplied
  `Node` that snora cannot enumerate.
- **Only a menu open (`header_menu` / `context_menu`)** — cycling is
  **unaffected**.

This mirrors `dismiss_on_escape`'s modal-before-menu priority exactly. One
overlay-precedence model, expressed twice — if your implementation disagrees
with that function's ordering, one of them is wrong.

### 4.3 `crates/snora/src/keyboard.rs`

Add a companion to `dismiss_on_escape` mapping a key to a cycle direction, so
applications are not re-deriving "F6 forward, Shift+F6 backward". Same module
doc voice, same non-capture policy, same `Option` return.

### 4.4 Documentation — six sites, machine-derived

`grep -rn "RFC-014\|focus trap\|Focus trapping" docs/src crates` was run when
scoping. **Re-run it** and confirm nothing has landed since; these are the
sites it found that are in scope:

| Site | What it says now |
|---|---|
| `contributing/semantic-accessibility.md:172` | ownership table row: *"Focus trapping in modals \| Out of v0.20 scope (deferred, RFC-014-B)"* |
| `contributing/design-decisions.md:32` | table row: *"Focus trapping deferred \| Deferred \| Concrete app + stable iced focus API"* |
| `contributing/design-decisions.md:404+` | § *"Why focus trapping is deferred (v0.14)"* — see §5 |
| `reference/overlay-interaction-semantics.md:5–6` | RFC-014-A / RFC-014-B named as future extenders |
| `reference/overlay-interaction-semantics.md:90, 98` | *"does not promise keyboard focus trapping"*; table row *"Focus trapping \| no — deferred (RFC-014-B)"* |
| `reference/overlay-interaction-semantics.md:207–210` | Law 7 *"(for now)"*, and the focus-trapping limitation note |
| `guides/accessibility.md` | consumer-facing; gains zone navigation |

**The ownership table is rewritten, not amended.** Retire the RFC-014-B row and
the v0.20 scope statement; state what is now snora's, what stays the
application's, and what is staged behind Q-1.

**`overlay-interaction-semantics.md` Law 7's *"(for now)"* is resolved** — and
resolved *narrowly*: Escape dismissal stays application-wired, zone navigation
is now snora's decision, trapping is staged. Do not overstate it into "snora
owns the keyboard."

**Do not claim containment shipped.** Per the RFC's security note, focus
escaping a modal can let a keyboard user reach controls the modal is gating.
This RFC delivers navigation, not containment.

## 5. `design-decisions.md` needs care, not a delete

This is the site the RFC's own first draft missed, and it is the most
consequential. It records:

> **Reconsideration trigger:** a concrete downstream app demonstrates the need
> and iced provides a stable, cross-platform focus API. Any focus
> implementation must be additive — a new optional `Dialog`/`Sheet` field per
> RFC-011-C rules.

Three things follow, and none is a deletion:

1. **Record that the trigger fired**, and that nothing checked it — the trigger
   lived in a decision record with no scheduled re-read. Say so; a
   reconsideration trigger with no re-check is a note, not a mechanism.
2. **The additive constraint survives and is inherited.** Trapping must arrive
   as a new optional `Dialog`/`Sheet` field under RFC-011-C. Carry it forward
   attached to Q-1, do not drop it with the deferral.
3. **Correct one sentence.** It says iced 0.14's `operate` machinery and
   `widget::Id` *"make programmatic focus queries possible."* True but
   incomplete in the way that matters: reachable **only with the `advanced`
   feature**, which snora does not enable. State the narrow form.

The decision itself is **not** fully reversed — trapping stays deferred behind
Q-1. What changes is the reason: from "unproven" to "measured, scoped, and
waiting on one feature decision."

## 6. The constraint, verified — do not re-probe or re-assert

Compile-probed against iced 0.14 with snora's exact feature set (`canvas`,
`svg`, `tokio` — **no** `advanced`):

| Capability | Path | Reachable |
|---|---|---|
| Move focus forward | `iced::widget::operation::focus_next()` → `Task` | **yes** |
| Move focus backward | `iced::widget::operation::focus_previous()` → `Task` | **yes** |
| Query focused widget | `focusable::find_focused()` | **no** — needs `advanced` |

Moving focus is available now; knowing where it is is not. **This RFC needs
only the first**, so it enables no feature.

**Do not enable the `advanced` feature.** If you conclude the task cannot be
done without it, stop and report — that is Q-1, a measured decision with
compile-cost, binary-size and API-stability consequences, not an implementation
detail. It is also the exact shape of deviation that went wrong on RFC-058:
report it in the review request, do not proceed on an answer obtained
elsewhere.

## 7. Change scope

| File | Purpose |
|---|---|
| `crates/snora-core/src/focus.rs` | **new** — vocabulary + decision (§4.1) |
| `crates/snora-core/src/lib.rs` | register the module |
| `crates/snora/src/keyboard.rs` | direction helper (§4.3) |
| the six documentation sites in §4.4 | retire the deferral |
| `docs/src/contributing/design-decisions.md` | §5 — care, not deletion |
| `CHANGELOG.md` | **Added** |

## 8. Explicit non-change scope

Do **not**:

- **Take Tab or Shift+Tab**, or install any keyboard subscription (§3).
- **Enable iced's `advanced` feature** (§6).
- **Add an iced dependency to `snora-core`.**
- **Implement modal focus trapping.** Staged behind Q-1.
- **Render a focus ring.** Staged behind Q-2 — and it is `design`-gated and an
  appearance change, so it is not a free addition.
- **Add zones for tabs or breadcrumbs.**
- **Change `AppLayout`'s fields**, the z-stack, dismissal, or
  `dismiss_on_escape`'s behaviour.
- **Hold the current zone inside snora.** Per Q-3 it is application state, like
  `toasts` and the overlay flags; the new function is pure *of* that state.
- Modify `render_semantics.rs`.

## 9. Required tests

Unit tests in `snora-core` (no renderer needed — that is the point of putting
it there):

- forward and backward cycling with all four zones present;
- wrap-around in both directions;
- every combination of absent optional slots, including **body-only**, which is
  the degenerate case an `AppLayout::new` application actually has;
- modal open → suspended; menu open → unaffected; both open → modal wins,
  matching `dismiss_on_escape`.

```bash
cargo test -p snora-core
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

Any doc example must satisfy the documentation-test policy — no bare `rust`
fences in `docs/src`.

## 10. Acceptance criteria

RFC-060 §Acceptance criteria 1–7. The two most likely to be missed:

- **4** — the ownership table **rewritten**, not amended. An added row beside
  the stale one is the failure mode.
- **7** — `render_semantics` unmodified **and no new iced feature enabled**.
  Show both.

## 11. Required evidence

- The new module and its tests, with the body-only and both-overlays cases
  visible.
- `cargo test -p snora-core` output.
- Proof `snora-core` still has no iced dependency —
  `cargo tree -p snora-core` or its `Cargo.toml`.
- Proof no feature changed — `git diff -- '**/Cargo.toml'`.
- Before/after for all six documentation sites, plus the re-run grep from §4.4.
- `render_semantics` output and `git diff --stat -- crates/snora/tests/` empty.
- The CHANGELOG entry.

## 12. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/060-frame-level-keyboard-navigation/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** whether the containment decision agrees with
`dismiss_on_escape`'s modal-before-menu precedence in every combination, and
whether `design-decisions.md` now reads as a *scoped, waiting* decision rather
than either an abandoned deferral or a delivered capability. The failure mode
is a reader concluding snora traps focus when it does not.
