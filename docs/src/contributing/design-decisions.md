# Design decisions

A snora API decision is rarely a free choice — most of them have a
shape that closes off other shapes. This page records the reasoning
so that future contributors don't relitigate decisions whose
trade-offs are still valid.

## Why no `PageContract` trait

Early drafts (≤ 0.3) defined a trait that page-like objects implemented:

```rust,ignore
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

## Why `Tab` and `Crumb` are separate vocabulary, not one navigation type

In 0.7 we added [`TabBar`] and [`Crumb`] as independent types
rather than collapsing them into a single `Navigation` enum.

- They communicate different shapes of UI affordance. Tabs imply
  *peer-level switching* — three to seven views the user expects
  to flip among. Breadcrumbs imply *ancestor-level navigation* —
  a path showing depth, only the parents are interactable.
  Conflating them in one type forces every consumer to handle
  both shapes; keeping them separate lets each screen pick
  exactly the affordance it wants.
- The `id` types have different semantics. A `TabId` is a small
  closed set (3–7 values, typically all variants of an enum) and
  `active` is one of them. A `CrumbId` is a path-element id —
  potentially open-ended in the wider application even if any
  single trail is short. The semantic difference would have
  required generics either way; collapsing types only saves a
  module and gains nothing for the caller.
- The `is_leaf` flag on `Crumb` would be meaningless on a tab.
  Tabs do not have a leaf concept; one of them is "active", but
  pressing any of them is symmetric.

The cost of two types is two short modules. Each is around 60
lines of vocabulary and 80 lines of widget code. We are not at
risk of vocabulary explosion in this corner of the API.

## Why widget feature gating is coarse, not per-widget

Snora 0.7 ships **one** `widgets` feature on the `snora` crate.
There is no `widget-tab-bar` / `widget-breadcrumb` / `widget-header`
distinction. We deliberately stop at the coarse boundary.

- The current widget set is small (seven prefab elements at 0.7).
  Compile time savings from gating any one widget out are
  negligible compared to the iced compile, which dominates
  cold-cache time.
- A wider feature matrix multiplies documentation surface — every
  combination is something a user might trip over and a
  maintainer must keep coherent.
- Fine-grained gates are *additive*. We can add them later
  without breaking anything; the inverse (removing them after
  shipping) breaks downstream code. Default to the simpler shape.

The criteria that would justify revisiting the decision are
documented separately in
[contributing/feature-gating-criteria.md](feature-gating-criteria.md).
That document records the indicators (compile time threshold,
binary size threshold, heavy optional deps, platform-specific
deps, field requests) so future maintainers do not have to
reconstruct the reasoning.

## Why `AppLayout` has both fields and a builder

[`TabBar`]: ../reference/vocabulary.md
[`Crumb`]: ../reference/vocabulary.md

Both are supported; the builder is the stable, documented canonical path.
Reasoning:

- The builder (`AppLayout::new(body).header(h).footer(f)`) is the
  recommended path because each setter has a clear name and you read
  the building site top-to-bottom.
- Direct struct-literal construction from *outside* `snora-core` is
  no longer permitted (see below). Fields remain `pub` for in-crate
  access and for reading by the engine.

We are *not* going to add a `Default` impl that requires `body:
Option<Node>` — `body` is mandatory by construction; `AppLayout::new`
exists precisely to enforce that.

## Why `AppLayout` is `#[non_exhaustive]` (v0.11)

Added in v0.11.0. Three later planned features (anchored popover,
optional focus policy, and possible new overlay surfaces) may each
add a top-level field to `AppLayout`. Without `#[non_exhaustive]`,
every such addition would break downstream code that constructs the
struct with a literal.

The decision was made concrete by an in-tree audit: **no downstream
code constructs `AppLayout` by literal** — every example already uses
`AppLayout::new(body)` plus builders. The change broke nothing in
practice and unblocks future additive extensions.

Rule: any future PR adding a field to `AppLayout` must add a matching
`#[must_use]` builder method in the same PR (see RFC-011-C).

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

## Why tooltip vocabulary is deferred (v0.13)

`SideBarItem.tooltip: String` is the only typed tooltip-like field in
the current API. A shared `Tooltip { text: String, side: Edge }` type
would be justified when a second consumer appears. As of v0.12 no second
consumer exists. The trigger and the proposed type are documented in
RFC-013-C. When the trigger is met, `SideBarItem.tooltip` changes from
`String` to `Tooltip` — a minor-release breaking change with a migration
guide.

## Why the persistent-toast helper is deferred (v0.13)

`Toast::new(…).persistent()` is the current idiom. A `persistent_ack`
named constructor would be justified when two separate examples or apps
repeat this exact pattern. As of v0.12 no example calls `.persistent()`.
The trigger is documented in RFC-013-C. If/when met, `persistent_ack` is
a small additive constructor with a doctest — no migration needed.

## Why Snora is theme-aware but not theme-owning (v0.14)

Snora reads iced's active `Theme` (extended palette) in prefab widgets
and toast rendering. It does not define a parallel theming layer. This
is intentional and permanent: adding a `SnoraTheme` struct would
duplicate iced's system, force applications to configure theming twice,
and create a maintenance surface with no commensurate value.

The `ToastIntent::Warning` color uses a private fallback
(`WARNING_COLOR` in `crates/snora/src/toast.rs`) because iced's
extended palette has no `warning` semantic pair. This is a narrow
exception, not a token system. When iced adds a warning pair the
fallback will be removed.

Style review checklist for future changes: (1) Does the change add a
public color/token type? If so, reject or escalate. (2) Does it derive
from iced `Theme` where possible? (3) Does it add a dependency? Evaluate
feature-gating. (4) Does it affect binary size? Measure.

## Why focus trapping is deferred (v0.14)

Snora's modal dim provides visual modality and pointer blocking. It does
not trap keyboard focus (Law 8, RFC-011-E). iced 0.14's `operate`
machinery and `widget::Id` make programmatic focus queries possible, but
a reliable cross-platform focus trap at the framework level is unproven.

Reconsideration trigger: a concrete downstream app demonstrates the need
and iced provides a stable, cross-platform focus API. Any focus
implementation must be additive — a new optional `Dialog`/`Sheet` field
per RFC-011-C rules.
