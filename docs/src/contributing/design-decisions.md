# Design decisions

A snora API decision is rarely a free choice — most of them have a
shape that closes off other shapes. This page records the reasoning
so that future contributors don't relitigate decisions whose
trade-offs are still valid.

**Status key:** *Firm boundary* — requires an RFC with a concrete scenario
to reopen. *Accepted* — current approach; open to revision with evidence.
*Deferred* — planned when trigger condition is met.

## Decision index

**The "Evidence confirmed" column dates when a row's reconsideration
trigger was last checked against current reality — not when the
decision was made.** Most of this project's triggers depend on a fact
(what a consumer says, what a check reports) that no automated check
can derive on its own (RFC-082 Q-2); this column is the honest
substitute — staleness becomes visible instead of assumed. Every row
below was read against current source or current correspondence on
2026-08-20 as part of this pass (RFC-082); rows whose trigger is a
static design threshold with nothing to measure today (most "Firm
boundary" rows) are dated by that same read-through confirming the
row's text still accurately describes the decision, not by a fresh
empirical re-check.

| Decision | Status | Reconsideration trigger | Evidence confirmed |
|---|---|---|---|
| No `PageContract` trait | Firm boundary | A trait that an engine actually consumes | 2026-08-20 |
| One close sink per channel | Firm boundary | A concrete app needing per-overlay close | 2026-08-20 |
| One `Sheet` type, not `BottomSheet`/`TopSheet` | Firm boundary | — (settled; axis-relative design is correct) | 2026-08-20 |
| Default `ToastPosition` is `TopEnd` | Accepted | User research showing another default is more ergonomic | 2026-08-20 |
| Application owns toast `Vec` | Firm boundary | Framework-owned queue that apps cannot control | 2026-08-20 |
| No `snora-test` crate | Firm boundary | A test need the `pub` fields + pure `update` pattern cannot serve | 2026-08-20 |
| Five crates (`-core`, `-design`, `-style`, `-widgets`, engine) | Accepted | A layer gains a second consumer that does not fit its crate, as the style bridge did (RFC-055) | 2026-08-20 |
| `Tab` and `Crumb` are separate vocabulary | Accepted | A combined type that handles both cleanly | 2026-08-20 |
| Coarse `widgets` feature gate | Accepted — trigger checked, not fired (RFC-062) | Two of the five feature-gating indicators are met; at most one is | 2026-08-20 — matches [feature-gating-criteria.md's own re-derivation](feature-gating-criteria.md#current-status-snora-0391-re-derived-2026-08-20-rfc-062), same date |
| `AppLayout` has both fields and builder | Firm boundary | — (the `#[non_exhaustive]` decision below) | 2026-08-20 |
| `AppLayout` is `#[non_exhaustive]` | Firm boundary | 1.0 freeze; no new overlays needed | 2026-08-20 |
| No `mod.rs` | Firm boundary | Rust edition change | 2026-08-20 |
| English-only comments | Firm boundary | Multi-language team adopts the project | 2026-08-20 |
| Tooltip vocabulary deferred | Deferred | Second consumer type in the codebase | 2026-08-20 — re-checked: `SideBarItem.tooltip: String` remains the only typed tooltip-like field in `crates/snora-core/src/*.rs` |
| Persistent-toast helper deferred | Deferred | Two separate apps repeat `.persistent()` | 2026-08-20 — re-checked: one production call site (`examples/toast/src/main.rs`), not two |
| Theme-producing, not theme-owning | Accepted; theme-*owning* stays Firm boundary | Owning: an RFC with a concrete scenario. Producing: evidence the emission approach itself needs revision. | 2026-08-20 |
| Focus trapping deferred | Deferred — Q-1 (RFC-060) is the blocker; no consumer is currently a demand signal | Concrete app: **none** — tekstide withdrew 2026-08-18 (*"we would not switch to trapping even if you shipped it"*); arama out, apimokka declined. Focus *querying* API: needs iced's `advanced` feature — the measurement that would have decided this (RFC-078) was **archived 2026-08-20, superseded by the owner's direct ruling: `advanced` will not be enabled by default, and no consumer ever requested it.** If trapping is ever built, `advanced` belongs behind its own opt-in feature, never a default — not stable-by-default today | 2026-08-20 (RFC-082) |
| Binary size measured via three feature-exercising probes | Accepted | Probe drift makes the marginal-cost diff unreliable across releases | 2026-08-20 |
| No interim accessibility tree; ABDD bounded to layout + visual | Accepted | iced exposes an accessibility API — checked via `cargo tree -p snora --all-features \| grep -i accesskit` | 2026-08-20 — re-run fresh, still empty |

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

## Why five crates

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

### A fourth crate: `snora-design`

In 0.19 we carved a fourth crate, `snora-design`, out of what would
otherwise have been `snora-widgets` code. The reasoning is the same
argument that separated `snora-core` from `snora` in the first place:
`snora-design`'s token vocabulary (`Tokens`, `Palette`, contrast
utilities) has no iced type in its signature and gains nothing from
sitting in an iced-dependent crate. Putting it in `snora-widgets`
would mean an iced upgrade could, in principle, touch pure-data token
code that has no reason to change. Keeping it separate preserves the
same iced-free guarantee (NF-1) that motivates `snora-core`, applied
one layer up. `snora-widgets` depends on `snora-design` (behind the
`design` feature) to bridge tokens into `iced::*::Style`; `snora-core`
does not depend on `snora-design`. See RFC-020 and RFC-021 for the
full design-system boundary and crate/feature architecture.

### A fifth crate: `snora-style`

In 0.32 we carved a fifth crate, `snora-style`, out of `snora-widgets`
(RFC-055). The trigger was not size — the whole style layer is ~48 KB and
0.3% of a release binary — but **consumer count**.

`card_raised` had three callers: the card *widget* in `snora-widgets`, the
*engine chrome*'s dialog card in `snora`, and *applications* directly, since
`snora::design::style::*` re-exports the style modules for use on an
application's own iced widgets. One style vocabulary with three consumers,
physically located inside one of the three. The engine therefore reached
sideways into a crate it otherwise did not need, and `design` could not be
enabled without `widgets`.

The layering test is the one that settled it: the style layer imports nothing
from the widget layer, while five widget-layer modules import *it*. It was
structurally below the widget layer already, inside the widget crate. The
theme emitter moved for the same reason — its imports are `iced`,
`snora-design` and `style::color`, with nothing from the widget layer.

The result is that **`design` and `widgets` are independent**: four
expressible configurations where three existed, the new one being design
without widgets. The default configuration is byte-for-byte unchanged, which
is what proves `snora-style` is a genuinely conditional dependency.

The cost is one more crate to publish and one more `Cargo.toml`, plus
`snora::design` being partially available — its widget-layer re-exports are
`#[cfg(feature = "widgets")]`-gated, which is unavoidable for any design
configuration that does not include widgets.

**Reconsideration trigger:** *a layer inside one crate acquires a consumer
outside it that does not depend on that crate.* That is what happened here
twice — `snora-design` in 0.19 and `snora-style` in 0.32 — and it is the
signal to check layering rather than to add a re-export.

The five-crate split remains invisible to applications that depend only on
`snora` — the `design` feature (opt-in) re-exports `snora-design`'s types and
`snora-style`'s bridge under `snora::design`, and every pre-0.32 import path
still resolves through compatibility re-exports in `snora-widgets`.

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

**The trigger has not fired, checkable against the record rather than
asserted (RFC-062).** Re-derived 2026-08-18: indicator 1 (compile time)
is currently unassessed — the CI proxy `feature-gating-criteria.md`
previously cited was measuring a different quantity and has been
retired as a stand-in; indicators 2 through 5 are each confirmed *not*
met — indicator 2 (binary size) reads 46,464 B (~45 KB) against a
150 KB threshold, and 3, 4, and 5 have no qualifying instance as of this
date. **At most one of the five indicators could be met** (indicator 1,
if a real developer-machine measurement were taken and found over
threshold) — which is short of the "two or more" the trigger requires
regardless of that one unknown value. This is a checkable conclusion,
not a restated verdict: the four confirmed-unmet indicators alone are
sufficient to establish it.

## Why `AppLayout` has both fields and a builder

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

## Why Snora is theme-producing, not theme-owning (v0.14; amended v0.26 by RFC-037)

Snora reads iced's active `Theme` (extended palette) in prefab widgets and
toast rendering, and — as of RFC-038, under the `design` feature — can
also *produce* one. The distinction that makes both true at once:

- **Theme-owning** — snora defines a parallel theming abstraction, holds
  theme state, and requires applications to configure appearance through
  snora rather than through iced. **Still declined, permanently.** Adding
  a `SnoraTheme` struct would duplicate iced's system, force applications
  to configure theming twice, and create a maintenance surface with no
  commensurate value.
- **Theme-producing** — a pure function from tokens to an `iced::Theme`
  (`snora::design::theme`, RFC-038). The application decides whether to
  call it, owns the result, and hands it to iced through iced's own
  `.theme()` hook. Snora holds no state and intercepts nothing.
  **Permitted under `design`.** Emission does not duplicate iced's
  system — it feeds it — and it *removes* a double configuration that
  exists today, where an application must separately configure tokens and
  an `iced::Theme` and keep them manually in agreement.

The `ToastIntent::Warning` color uses a private fallback
(`WARNING_COLOR` in `crates/snora/src/toast.rs`) that predates a
verification of iced's palette. **iced 0.14 does provide a `warning`
semantic pair**: `iced_core::theme::Palette::warning` (base color) and
`iced_core::theme::palette::Extended::warning` (generated three-shade
set), confirmed against the pinned dependency source at
`iced_core-0.14.0/src/theme/palette.rs:18` and `:297`. `WARNING_COLOR`
is therefore a removal candidate — but toasts render on the
design-inactive path, so removing it would change the appearance of
existing applications that do not opt into the `design` feature. Its
disposition (keep as a documented intentional fallback, or migrate to
`Extended::warning`) is deferred to RFC-038 Q-2; the constant is not
changed by this correction.

Style review checklist for future changes: (1) Does the change add a
public color/token type? If so, reject or escalate. (2) Does it derive
from iced `Theme` where possible? (3) Does it add a dependency? Evaluate
feature-gating. (4) Does it affect binary size? Measure.

## Why focus trapping is deferred (v0.14)

Snora's modal dim provides visual modality and pointer blocking. It does
not trap keyboard focus (Law 8, RFC-011-E).

**Corrected (RFC-060):** the previous text here said iced 0.14's
`operate` machinery and `widget::Id` "make programmatic focus queries
possible" — true, but incomplete in the way that matters. Verified
against snora's exact feature set (`canvas`, `svg`, `tokio`, no
`advanced`): *moving* focus (`operation::focus_next()` /
`focus_previous()` → `Task`) is reachable today without any new
feature. *Querying* which widget is focused (`focusable::find_focused()`)
is reachable only with iced's `advanced` feature, which snora does not
enable. A reader taking the old sentence at face value would conclude
the query is free; it is not.

**The reconsideration trigger fired, and nothing checked it.** The
condition was "a concrete downstream app demonstrates the need and iced
provides a stable, cross-platform focus API." tekstide is that
downstream app — they implemented zone-based focus cycling themselves
and called modal focus trapping "security machinery... proven with a
positive control," specifically because snora did not offer it. And the
API split above means the *moving* half of the condition is met today.
Nobody re-read this record when either half changed; a reconsideration
trigger with no scheduled re-check is a note, not a mechanism. RFC-060
is the response to the first half (frame-level zone navigation, which
only needs *moving* focus) — trapping itself needs the *querying* half,
which still needs a decision.

**The additive constraint survives this correction and is inherited.**
Any focus implementation must remain additive: trapping must arrive as
a new optional `Dialog`/`Sheet` field, per RFC-011-C rules, not a change
to an existing field or a new default behaviour. RFC-060 binds its own
Q-1 (whether to enable iced's `advanced` feature for trapping) to this
constraint explicitly, rather than treating it as settled by that
RFC alone — enabling `advanced` has its own compile-cost, binary-size
and API-stability consequences, measured separately, not decided as a
side effect of shipping zone navigation.

**What actually changed here:** the *reason* trapping stays deferred,
from "unproven" to "measured, scoped, and waiting on one feature
decision (Q-1)." The decision itself is not reversed — trapping is
still not shipped.

**Corrected again (RFC-082, 2026-08-20): tekstide, the concrete app
that fired the trigger above, has since withdrawn as a demand
signal.** On 2026-08-18 they told us directly: *"keystroke suppression
is stronger than trapping for our threat model and we would not
switch to trapping even if you shipped it."* Our own 0.36.1 note
already recorded the consequence — *"the demand column for modal
focus trapping is empty"* — alongside arama (out) and apimokka
(declined). **No consumer is currently a demand signal for trapping.**

**This does not resolve Q-1.** Whether to enable iced's `advanced`
feature by default is a separate decision — it does not depend on
consumer demand and is not decided by this correction. **The
measurement that would have decided it (RFC-078) was archived
2026-08-20**, superseded by the owner's direct ruling: `advanced` will
not be enabled by default, and no consumer ever requested it. If
trapping is ever built, `advanced` belongs behind its own opt-in
feature, never a default. **The decision stays deferred** — only the
evidence changed: the register no longer lists a consumer as currently
asking for it, but the blocker and the additive constraint above are
unaffected.

## Why binary size is measured via three feature-exercising probes

Three probe crates (`size_probe_engine`, `size_probe_widgets`,
`size_probe_design`) measure the marginal binary-size cost of the
`widgets` and `design` features. `widgets_diff_bytes` and
`design_diff_bytes` are computed from stripped binary sizes across the
three (v0.25, replacing an earlier hello-vs-workbench diff).

The first release under this methodology (v0.25.3, RFC-041) exposed a
defect: all three probes were byte-identical — same application code,
differing only in which `snora` features their `Cargo.toml` enabled —
and `widgets_diff_bytes` measured **0**. The probes compiled the
`widgets` feature in but never *called* any `snora::widget::*` function,
so Rust's linker stripped the entire unused feature at link time. The
diff was real, just measuring "cost of compiling but not using" rather
than "cost of adopting" — the opposite of the intended signal.

RFC-043 corrected this: each probe now shares a common baseline
application (`size_probe_engine`'s code) and adds exactly one minimal,
representative call to the feature it measures — `size_probe_widgets`
wires `app_header` and `app_side_bar` into its `AppLayout`;
`size_probe_design` additionally calls `design::button::primary` and
`design::style::container::card_surface` against `Tokens::light()`. The
probes are deliberately **not** identical anymore; "identical
application code" was the trap, not the goal. The goal — isolating the
marginal cost of one feature from unrelated application-logic
differences — is preserved by keeping every probe's *baseline* shared
and adding only the smallest representative use on top.

Reconsideration trigger: if a future methodology change causes probe
drift that makes the marginal-cost diff unreliable or non-reproducible
across consecutive releases, revisit the representative-use approach
(see RFC-043 §Risks for the accepted risk that a corrected probe can
still report an honestly small number).

## Why snora has no interim accessibility tree (v0.27)

A downstream team preparing UX acceptance sessions asked directly
whether snora has a position on AccessKit, which iced has discussed
integrating, or considers assistive technology out of scope in favour of
visual accessibility. Verified before answering:
`grep -rniE "accesskit|accessibility_tree|widget::Id|semantic_id"
crates/*/src/` returned nothing. There is no accessibility tree, no
AccessKit integration, and no semantic identifiers anywhere in the
crates — a screen reader sees nothing an application does not supply
itself.

The gap is defensible: an accessibility tree is not something a layout
framework can supply on its own, and iced 0.14 does not expose one. What
was not defensible is that snora's own framing — "Accessible By Default
and by Design" — invites a reading its implementation does not support.
A reader meets the name before the fine print, and the name was doing
the claiming.

The position, stated in
[`semantic-accessibility.md`](semantic-accessibility.md#position-on-assistive-technology-rfc-045)
and reproduced here for the register:

> snora will integrate an accessibility tree when iced exposes one.
> Until then, ABDD means layout-direction correctness and visual
> accessibility — contrast, logical edges, non-colour status encoding —
> and snora states that boundary plainly rather than implying more.
> snora will not build a parallel accessibility abstraction of its own
> in the interim.

The last clause is deliberate: a snora-owned interim accessibility layer
would repeat DEC-02's original mistake — a parallel abstraction
duplicating what the toolkit will eventually provide — in a domain where
getting it wrong is worse than in theming. When iced exposes an API,
snora integrates it, the same relationship snora already has to iced's
`Theme`.

This bounds a *claim*; it does not retreat from a *capability*. The
contrast-tested presets, the four built-in tokens, and the ABDD layout
discipline are unchanged and are genuinely strong for a framework this
size — the correction is to stop implying they cover assistive
technology too.

Reconsideration trigger: iced exposes an accessibility API. Until then
this stands, revisited deliberately rather than left to expire quietly.

**The check attached to this trigger, credit tekstide (RFC-062):** the
`grep -rniE "accesskit|accessibility_tree|widget::Id|semantic_id"
crates/*/src/` above detects **snora's own** adoption of an
accessibility API — it correctly supports the claim that snora has not
built one, but it says nothing about whether *iced* has since exposed
one for snora to integrate. The trigger is about iced's readiness, not
snora's; a different command answers that question:

```bash
cargo tree -p snora --all-features | grep -i accesskit
```

**Verified 2026-08-18 (re-run twice to confirm): empty.** `iced_core`
0.14 has no accessibility module and pulls in no `accesskit` dependency.
The trigger has **not** fired, and the position recorded above remains
accurate. Re-run this check, not just the `crates/*/src/` grep, at each
future re-read of this trigger.

[`TabBar`]: ../reference/vocabulary.md
[`Crumb`]: ../reference/vocabulary.md
