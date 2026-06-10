# RFC-011-B — Toast Ordering Semantics Fix

**Status.** Implemented (v0.11.0)
**Tracks.** Bugfix / render semantics. Restores the documented
`ToastPosition` stacking invariant.
**Touches.** `crates/snora/src/toast.rs`,
`docs/src/getting-started/04-toasts.md`,
`docs/src/reference/vocabulary.md`, `CHANGELOG.md`.

> Project-adopted, code-accurate version of planning-pack RFC-011-B. The
> planning analysis has been **verified against the v0.10.0 source** and is
> correct; this version records the verification and the exact edit.

## 1. Summary

The newest toast must sit closest to the configured anchor edge, as the
`ToastPosition` docs promise. The v0.10.0 engine reverses the queue for
**bottom** anchors when it should reverse for **top** anchors, so the
visible order is inverted for every position.

## 2. [Verified] Root cause

`crates/snora/src/toast.rs::render_toasts` (v0.10.0):

```rust
let mut stack_col = column![].spacing(8);
if position.is_bottom() {                 // <-- wrong predicate
    for toast in toasts.into_iter().rev() {
        stack_col = stack_col.push(render_single_toast(toast));
    }
} else {
    for toast in toasts {
        stack_col = stack_col.push(render_single_toast(toast));
    }
}
```

Application convention (documented in `snora-core::toast`): apps push new
toasts to the **back** of `Vec<Toast<_>>`, so index `0` is oldest and the
last element is newest. In a top-down iced `column!`, the first child
pushed is visually higher; the last child pushed is visually lower. The
toast container anchors vertically: `Vertical::Top` for top positions,
`Vertical::Bottom` for bottom positions (confirmed in `render_toasts`).

Tracing both branches against the documented invariant ("newest closest to
the anchor edge"):

| Anchor | Want newest at | Correct iteration | v0.10.0 does | Result |
|---|---|---|---|---|
| Top | top edge (first child) | **reverse** (newest pushed first) | chronological | newest ends up at the bottom — **inverted** |
| Bottom | bottom edge (last child) | **chronological** (newest pushed last) | reversed | newest ends up at the top — **inverted** |

So both anchor families are wrong, and the fix is to reverse on
`is_top()` rather than `is_bottom()`.

## 3. Goals

- Make visible toast order match the documented invariant.
- Clarify queue semantics in code comments and docs.
- Add regression coverage that cannot silently re-invert.
- No public API change.

## 4. Non-Goals

- No change to the `Toast` data model.
- No priority sorting, grouping, replacement, dedup, or animation.
- No toast-manager object; apps keep owning `Vec<Toast<Message>>`.

## 5. External design

No public API change. The contract becomes explicit:

```text
Applications append toasts to the back of Vec<Toast<_>> in chronological
order. Snora renders that queue so the last element is closest to the
toast anchor edge.
```

| Position | Column order (top→bottom) | Newest closest to |
|---|---|---|
| `TopStart` / `TopCenter` / `TopEnd` | newest → oldest | top edge |
| `BottomStart` / `BottomCenter` / `BottomEnd` | oldest → newest | bottom edge |

## 6. Internal design

### 6.1 Introduce a pure policy helper

So the decision is testable without introspecting an iced `Element`, add a
private helper and route `render_toasts` through it:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastRenderOrder {
    Chronological,
    ReverseChronological,
}

/// Render policy: the newest toast (back of the queue) must sit closest to
/// the anchor edge. Top anchors fill a top-aligned column, so the newest
/// must be pushed first (reverse-chronological). Bottom anchors fill a
/// bottom-aligned column, so the newest must be pushed last
/// (chronological).
fn render_order_for(position: ToastPosition) -> ToastRenderOrder {
    if position.is_top() {
        ToastRenderOrder::ReverseChronological
    } else {
        ToastRenderOrder::Chronological
    }
}
```

`render_toasts` becomes:

```rust
let mut stack_col = column![].spacing(8);
match render_order_for(position) {
    ToastRenderOrder::ReverseChronological => {
        for toast in toasts.into_iter().rev() {
            stack_col = stack_col.push(render_single_toast(toast));
        }
    }
    ToastRenderOrder::Chronological => {
        for toast in toasts {
            stack_col = stack_col.push(render_single_toast(toast));
        }
    }
}
```

### 6.2 [Design decision] Helper visibility

`render_order_for` is **private** (module-private), unit-tested in
`toast.rs`'s `#[cfg(test)]` block. It is *not* `pub(crate)` and is *not*
exposed to the `tests/` integration target, because (a) it expresses a
production render policy, not a test affordance, and (b) RFC-011-D's
integration tests exercise observable behavior via `iced_test`, not this
helper. This keeps the toast-order regression covered at the unit level
while RFC-011-D covers interaction-level behavior — no public test surface
is created (consistent with the "no `snora-test`" non-goal).

### 6.3 Module-doc correction

The `toast.rs` module header still says toasts anchor "at the bottom-*end*"
— stale since `ToastPosition` grew to six variants. Replace with:

```rust
//! 1. [`render_toasts`] — internal renderer used by [`crate::render::render`].
//!    Produces a toast stack at the requested [`ToastPosition`], with logical
//!    Start/End anchoring resolved by [`LayoutDirection`], and the newest
//!    toast rendered closest to the anchor edge.
```

The `render_toasts` doc comment already states the intended growth
direction correctly; only the implementation predicate was wrong. Add an
inline comment noting the chronological-queue assumption.

## 7. Documentation changes

- `docs/src/getting-started/04-toasts.md`: add the order table from §5 and
  the one-line queue contract.
- `docs/src/reference/vocabulary.md`: ensure the `ToastPosition` note
  states the newest-closest-to-anchor invariant.

## 8. Testing plan

Unit tests in `toast.rs` covering all six positions:

```rust
#[test]
fn top_positions_render_reverse_chronological() {
    use ToastPosition::*;
    for p in [TopEnd, TopStart, TopCenter] {
        assert_eq!(render_order_for(p), ToastRenderOrder::ReverseChronological);
    }
}

#[test]
fn bottom_positions_render_chronological() {
    use ToastPosition::*;
    for p in [BottomEnd, BottomStart, BottomCenter] {
        assert_eq!(render_order_for(p), ToastRenderOrder::Chronological);
    }
}
```

RFC-011-D additionally asserts dismissal interaction order through
`iced_test` where practical.

## 9. [Design decision] Version classification

RFC-015-A (not yet adopted) raises the open question of whether a
documented-semantics fix is patch or minor. Under the *current* handoff
convention this is a **bugfix that restores the already-documented
invariant**, with no API or feature change, shipping inside the v0.11.0
minor. It is recorded in `CHANGELOG.md` under **Fixed**, not **Changed**,
because no documented behavior changed — the implementation is brought back
into line with the docs. A short note flags that apps which happened to
rely on the inverted order will see corrected order.

## 10. Risks and mitigations

| Risk | Mitigation |
|---|---|
| An app relied on the accidental order. | It contradicted the documented invariant; note in changelog. |
| Helper looks like testing an implementation detail. | It encodes documented render policy, not arbitrary internals. |
| iced column behavior changes later. | RFC-011-D interaction tests provide a second line of defence. |

## 11. Acceptance criteria

- Top anchors render newest closest to the top edge.
- Bottom anchors render newest closest to the bottom edge.
- All six positions covered by `render_order_for` tests.
- Stale bottom-end-only module doc removed.
- `CHANGELOG.md` records the toast-ordering fix under **Fixed**.
