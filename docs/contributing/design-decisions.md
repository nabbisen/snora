# Design decisions

A snora API decision is rarely a free choice — most of them have a
shape that closes off other shapes. This page records the reasoning
so that future contributors don't relitigate decisions whose
trade-offs are still valid.

## Why no `PageContract` trait

Early drafts (≤ 0.3) defined a trait that page-like objects implemented:

```rust
trait PageContract {
    type Node;
    type Message;
    fn view(&self) -> Self::Node;
    fn dialog(&self) -> Option<Dialog<Self::Node, Self::Message>>;
    fn toasts(&self) -> Vec<Toast<Self::Message>>;
    fn context_menu(&self) -> Option<Self::Node>;
    fn on_close_menus(&self) -> Option<Self::Message>;
    fn on_close_modals(&self) -> Option<Self::Message>;
}
```

The intent was that `render_app` would call each method and compose
the result. In practice the engine never consumed any method other
than `view`, so applications had to plumb the rest manually anyway —
and the trait's associated types forced all four layout slots to share
a single page type, which produced a "Section enum" boilerplate.

In 0.4 the trait was removed and overlay state was moved to plain
fields on `AppLayout`. Reasoning:

- The trait did not earn its keep — it described a contract no
  engine implemented in full.
- Plain fields make the closure of "what can be on screen" obvious
  by inspection of one struct.
- Independent slot types are recoverable any time without API
  breakage by changing `Node` to `Box<dyn Trait>` if needed.

## Why one close sink per channel, not per overlay

`Dialog` and `Sheet` could each carry an `on_outside_click:
Option<Message>`. We considered that and rejected it.

- Two overlays can be present together (a sheet under a dialog).
  With per-overlay sinks, two outside-clicks are needed to close
  both, which is unintuitive — usually the user wants the dim
  area to dismiss everything modal at once.
- The 99% case is "one CloseModals message that resets all modal
  state". Moving that into `AppLayout::on_close_modals` puts the
  user in the pit of success.
- Per-overlay sinks would also have to interact with z-order rules,
  which is engine business.

The design loses flexibility (you cannot close the dialog and keep
the sheet open via outside-click) but gains a one-place wiring rule
that is hard to misuse. Net: positive.

## Why one `Sheet` type, not `BottomSheet` / `TopSheet` / `SideSheet`

In 0.6 we generalized the bottom-anchored drawer of 0.5 into a
single `Sheet` with a `SheetEdge { Bottom, Top, Start, End }`.
The alternative — keep `BottomSheet` and add separate `TopSheet` /
`SideSheet` types — was rejected.

- `AppLayout` would need three optional fields where one suffices.
  The 99 % case is "show one sheet at a time", and the engine's
  z-order rule does not need to distinguish between edges.
- Three nearly-identical builder methods would force callers to
  remember which type matches which edge. The general `Sheet` lets
  the edge ride on the value (`Sheet::new(...).at(...)`), keeping
  one builder symbol.
- Snora's "vocabulary over flags" principle says the *enum* is the
  vocabulary. Adding a `SheetEdge` enum is the canonical way to
  express the choice; adding three types is the anti-pattern.
- The size axis is naturally edge-relative (height for vertical
  edges, width for horizontal). A single `SheetSize` reads cleanly
  in both senses without a per-type rename.

The 0.5 → 0.6 type rename (`BottomSheet` → `Sheet`,
`SheetHeight` → `SheetSize`) is breaking on paper but cushioned
with `#[deprecated]` aliases that ship in 0.6 and are removed in
0.7.

## Why default `ToastPosition` is `TopEnd`

In 0.4 the default was `BottomEnd`, mirroring OS notifications. In
0.5 we moved to `TopEnd`. Reasoning:

- snora's primary user — a local-first app with heavy background
  work — usually puts primary content (preview, editor, list) in
  the lower half of the window. Bottom-anchored toasts compete with
  primary content for screen space.
- In-app notification frameworks across languages (Material
  Snackbar, Chakra, Mantine, sonner.js) more commonly default to a
  top corner.
- The change is a one-line override for users who want the old
  behavior. We documented it in the migration guide.

## Why the toast queue is `Vec<Toast<Message>>` owned by the application

Earlier drafts had snora own the queue internally. Externalizing it:

- Lets the application persist toasts (e.g. across hot-reload or
  serialize them in tests) without an opaque framework handle.
- Keeps `update` pure — snora's framework state does not interleave
  with the application's state machine.
- Matches the iced "owned state, immutable view" idiom.

The cost is that the application clones the vec every `view` call
to pass it into `AppLayout::toasts`. We measured: with toasts under
a few dozen and `Message` types under a few hundred bytes, the clone
cost is below the noise floor in iced's render loop. We will revisit
if a large-message use case shows up.

## Why no `Cargo.toml` for `snora-test`

We considered shipping a separate crate of test helpers (Toast
inspector, mock AppLayout). Decided against:

- It would freeze internal types into the public test API. Adding a
  `Toast::is_persistent()` predicate, for instance, makes
  `lifetime: ToastLifetime` a stability commitment.
- The `Toast` / `Dialog` / etc. structs already have `pub` fields,
  so plain `assert!` against application state covers the common
  cases — see [guides/testing.md](../guides/testing.md).
- A dedicated test crate adds release coordination overhead (every
  release needs `snora`, `snora-core`, *and* `snora-test` bumped).

If the pattern becomes painful in practice, we will revisit.

## Why three crates instead of two

In 0.4 and 0.5, snora was a two-crate workspace
(`snora-core` + `snora`). In 0.6 we carved out the prefab widgets
into a third crate, `snora-widgets`. The reasoning:

- **Widget evolution should not gate engine evolution.** Adding a
  new widget (a tab bar, a breadcrumb, a status bar) is a faster
  cadence of change than adding a new overlay layer. Putting them
  in the same crate as `render` made every widget addition a
  release of the engine.
- **Engine-only applications shouldn't pay for widgets.**
  Applications that supply 100 % of their UI parts can opt out
  with `default-features = false` on `snora` and the
  `snora-widgets` compilation is skipped entirely.
- **The widget set is properly downstream of `snora-core`, not of
  `snora`.** Widgets consume the vocabulary types (`Icon`,
  `LayoutDirection`, `MenuAction<...>`) but do not need the
  engine. The dependency edge `snora-widgets → snora-core` is
  direct; the previous structure forced widgets to be in `snora`
  even though they had no logical relationship to `render`.

The cost is one more `Cargo.toml` to maintain and one extra crate
in publish order. In exchange we get clean dependency edges and a
clear ownership boundary.

The 3-crate split is invisible to applications that depend only
on `snora` — `snora`'s lib re-exports `snora-widgets` under the
familiar `snora::widget` path when the `widgets` feature is on
(the default).

## Why `AppLayout` has both fields and a builder

Both are public and both supported. Reasoning:

- The builder (`AppLayout::new(body).header(h).footer(f)`) is the
  recommended path because each setter has a clear name and you read
  the building site top-to-bottom.
- Direct struct-literal construction (`AppLayout { body, header,
  side_bar, ... }`) is available as an escape hatch when generating
  layouts programmatically (e.g. in tests where you want explicit
  field-by-field overrides).

We are *not* going to add a `Default` impl that requires `body:
Option<Node>` — `body` is mandatory by construction; `AppLayout::new`
exists precisely to enforce that.

## Why no `mod.rs`

Style preference. `my_module.rs + my_module/` is the Rust-2018+ idiom,
keeps the file tree shallow, and matches how documentation generators
present the module hierarchy (the file name appears alongside the
directory name).

## Why English-only comments

All comments are in English so that snora is reviewable by
contributors regardless of language. Documentation prose in `docs/`
follows the same rule. Translations of `docs/` into other languages
are welcome as a separate effort.
