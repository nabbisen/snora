# Developer Handoff — RFC-053 `design::responsive_render`

**Governing RFC.** [RFC-053](../../done/053-responsive-render-excludes-the-design-path.md)
**Status.** Inherited from RFC-053 — Implemented (v0.31.0).
**Release target.** 0.31.0 (minor — new public API).
**Implementation units.** One. Independent of RFC-052/RFC-050; any order.

---

## 1. Task title

Add `snora::design::responsive_render`, so a `design`-path consumer can use
width-aware layout without losing the design chrome.

## 2. Purpose

`snora::responsive_render` renders through `crate::render::render`,
unconditionally. A consumer on the `design` path who adopts it loses the dialog
card and the token-derived modal dim — including the `high_contrast_dark` fix
that made the dim visible at all.

Responsive layout and design chrome are **mutually exclusive as shipped**.
Reported by apimokka, whose entire 0.28 adoption existed to deliver that
accessibility fix, and who therefore cannot adopt width exposure at all.

## 3. Background — read first

- `rfcs/done/053-responsive-render-excludes-the-design-path.md` in full.
- `crates/snora/src/responsive.rs` — 80 lines, the function you are pairing.
- `crates/snora/src/design/render.rs` — `pub fn render(layout, tokens)`, which
  your new function calls.
- `rfcs/done/039-engine-surface-styling.md` §"do not duplicate the z-stack".

Conventions: English only. `cargo fmt --all --check` is CI-enforced.

## 4. The shape — it is a wrapper, and must stay one

```rust,ignore
pub fn responsive_render<'a, Message, F>(
    build: F,
    tokens: &'a Tokens,
) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a,
{
    Responsive::new(move |size: Size| {
        crate::design::render::render(build(size.width), tokens)
    })
    .into()
}
```

That is the whole function. It mirrors `crate::responsive::responsive_render`
line for line, with one call swapped.

**If your implementation grows past a wrapper, stop.** `render_with_style` is
`pub(crate)` and stays that way; RFC-039 extracted the shared composition path
precisely so a second entry point would not re-implement the z-stack. Two
copies of layer ordering is the failure this design avoids.

**`&'a Tokens`, not `Tokens` by value.** It matches `design::render(layout,
&tokens)`, and the borrow is natural in the usual `fn view(&self) ->
Element<'_, Message>` where the returned element already borrows `&self`.
`Tokens` is `Clone` and cheap, so a by-value variant stays available later
without breaking this signature — the reverse is not true.

## 5. Placement and gating

`crates/snora/src/design.rs` currently has:

```rust,ignore
pub mod render;
pub use render::render;
```

and the whole module is `#[cfg(feature = "design")]` at
`crates/snora/src/lib.rs:154`. Follow that: put the function where
`design::render` lives and re-export it the same way, so the module gate
carries it and no new `#[cfg]` is needed.

**Do not add a `#[cfg(feature = "design")]` to the function itself.** The
module is already gated; a second gate is redundant and invites the two
drifting apart.

## 6. Required implementation

### Step 1 — The function

Per §4 and §5.

### Step 2 — Document the engine one as engine-path-only

`crates/snora/src/responsive.rs`'s docs never say it renders through the engine
path. **That silence is how this defect was found** — by a consumer reading the
source, not the docs. Add one sentence and a pointer to the design pair.

**Signature and behaviour unchanged.** Documentation only.

### Step 3 — The guide

`docs/src/guides/responsive.md` states which entry point serves which path, and
that mixing them was previously impossible. Short — a table row or two
sentences, not a section.

### Step 4 — The test that earns its keep

An identifier- or width-presence test is **not sufficient here**, for the same
reason it was not sufficient in RFC-049: the defect is *"renders, but through
the wrong path"*, and a presence check cannot see a wrong path.

Assert that **design chrome survives** through `design::responsive_render` —
that its output differs from `responsive_render`'s for the same layout and
carries the styled dialog card. `crates/snora/src/design/render/tests.rs`
already has the machinery: RFC-049's
`dialog_card_identifier_resolves_to_the_card_not_the_window` renders through
`design::render::render` with a `Simulator` and resolves
`snora-dialog-card`, which is emitted **only** on the design path.

That gives you a direct probe: render a dialog-bearing `AppLayout` through
`design::responsive_render`, and assert `snora-dialog-card` resolves. Through
the engine path it would not exist at all.

**Verify the test fails before the fix.** Point it at `responsive_render`
instead and confirm it fails; restore; confirm it passes. Include both outputs.
A passing test alone is not accepted.

## 7. Explicit non-change scope

Do **not**:

- **Make `render_with_style` public.** The two entry points are the surface.
- **Change `snora::responsive_render`'s signature or behaviour.** Docs only.
- **Change `design::render`.**
- **Add a `Breakpoint` type, thresholds, or adaptive behaviour.** Unchanged
  from RFC-046. This unblocks the evidence; it does not pre-empt it.
- **Add a third responsive example.** RFC-053 Q-2 suggests documenting first;
  `examples/responsive_body` cannot demonstrate this anyway (§8).
- **Touch the `design`/`widgets` feature edge.** That is RFC-053 Q-1, out of
  scope, and needs its own RFC.

## 8. Two things that will look like bugs and are not

**`examples/responsive_body` cannot demonstrate this.** It is
`default-features = false`, and `design = ["widgets", …]`, so the design
feature is unavailable in that crate by construction. Do not "fix" the example
to use the new function — it cannot, and why it cannot is RFC-053 Q-1.

**`cargo check -p snora --no-default-features` must still pass**, and the new
function must be absent from that build. If it leaks out of the `design` gate,
that is the defect (§5).

## 9. Required tests

```bash
cargo test -p snora --lib --all-features
cargo test -p snora --lib --no-default-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features      # new fn must NOT be present
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

Both feature configurations are mandatory: the whole point is that one entry
point exists only under `design`.

## 10. Acceptance criteria

RFC-053 §Acceptance criteria 1–6:

1. `snora::design::responsive_render` exists, gated by the `design` module,
   wrapping `design::render` without duplicating composition.
2. `snora::responsive_render` documents that it renders through the engine
   path and points at the design pair; signature unchanged.
3. `guides/responsive.md` states which entry point serves which path.
4. `render_semantics` passes **unmodified**.
5. `cargo check -p snora --no-default-features` passes; the new function is
   absent there.
6. A test proves design chrome survives the new path, **with failing-first
   evidence** (§6 step 4).

## 11. Prohibited shortcuts

- Do not re-implement the z-stack (§4).
- Do not satisfy criterion 6 with a presence check. Presence was never the
  problem; the path is.
- Do not modify `render_semantics.rs` for any reason.
- Do not widen scope into the `design`/`widgets` feature edge.

## 12. Compatibility and security

**Compatibility.** Purely additive. Consumers not on `design` are unaffected;
no existing signature changes.

**Security.** No new data flow, dependency, or integration.

## 13. Required evidence

- The new function in full, and the `design.rs` re-export diff.
- The `responsive.rs` documentation diff (proving no code change).
- **The failing-first pair** from §6 step 4 — the test pointed at the engine
  path failing, and at the design path passing.
- Both feature configurations' test output.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it is empty.
- `guides/responsive.md` diff.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/053-responsive-render-excludes-the-design-path/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** criterion 6. The function is five lines and
self-evidently correct once written; whether the test actually distinguishes
the design path from the engine path — rather than confirming that something
rendered — is the part worth reviewing.
