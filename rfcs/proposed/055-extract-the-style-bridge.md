# RFC 055 — Extract the iced style bridge into its own crate

**Status.** Proposed
**Tracks.** Crate boundaries. Implements option **B′** from
[RFC-054](./054-design-requires-widgets.md), whose investigation supplies every
number here.
**Touches.** New `crates/snora-style/`; `crates/snora-widgets/`;
`crates/snora/{Cargo.toml, src/design.rs, src/design/render.rs}`;
`docs/src/reference/architecture.md`; `docs/src/design/feature-flags.md`.
**Release target.** 0.32.0 (minor — new crate; no public path changes).

## Summary

`snora-widgets/src/design/style/` is a **style** layer: five modules that take
`&Tokens` and return `iced` style structs. No `Element`, no layout, no message.

RFC-054 established it is **structurally below the widget layer inside its own
crate** — it imports nothing from the widget layer, and five widget-layer
modules import it. Its placement in `snora-widgets` is the accident.

This RFC moves it to a peer crate, `snora-style`, which makes `design`
independent of `widgets` and makes `architecture.md`'s "strict dependency
direction" claim true rather than approximately true.

## Why — and it is not size

RFC-054 was framed around binary size, and that framing was wrong. The
measurement stands (a design-path consumer with no widget call sites could
recover **48,128 of 50,432 bytes**, ~95%) but 48 KB against a ~15.7 MB binary
is **0.3%**, and no consumer decision turns on it.

The reasons that do carry weight:

**0. The style layer already has three consumers, and widgets is one of
them.** `card_raised` is called by the card *widget*
(`snora-widgets/src/design/card.rs:81`), by the *engine chrome*
(`snora/src/design/render.rs:173`, the dialog card), and by *applications*
directly — `snora::design::style::*` re-exports all five style modules, so an
application can style its own iced widgets with them.

There are not two kinds of style with a boundary between them. There is **one
style vocabulary with three consumers**, physically located inside one of the
three. That is the whole finding; everything below is consequence.

**1. The misplacement compounds.** Two RFCs have already reached across the
boundary: RFC-039 for `card_raised`, RFC-053 via `design::render`. Every future
engine-surface styling feature reaches again, and every style function added
under `snora-widgets/src/design/style/` deepens it.

**2. snora cannot express a configuration it advertises.** `feature-flags.md`
guarantees *"engine-only builds remain green"*; `design` is documented as
opt-in. Their combination is unreachable — and `examples/responsive_body`,
shipped in v0.30.0 to match a specific consumer's architecture, cannot be
compiled by that consumer for exactly this reason.

**3. Declining has a permanent documentation cost.** Option C requires
`feature-flags.md` to carry, forever, an explanation of a coupling we know to
be a misplacement. B′ removes the caveat instead of explaining it.

**Option B is unavailable**, not merely worse: `snora` depends on
`snora-widgets` and not the reverse, so relocating the bridge into `snora` and
re-exporting it from `snora-widgets` would be a dependency cycle.

## Design

### The new crate

`snora-style` — the iced style bridge. Dependencies: **`iced` and
`snora-design` only.** Verified: the style modules import nothing else, in
particular not `snora-core`.

Contents: the whole of `snora-widgets/src/design/style/` —
`color`, `button`, `container`, `text`, `progress`.

**Move the layer, not the two functions the engine surface happens to use.**
Relocating only `to_iced_color` and `card_raised` would split the style layer
across two crates and leave the misplacement half-fixed, which is worse than
either endpoint.

### The dependency direction, after

```text
snora-core     (no dependencies)
snora-design   (no dependencies)
snora-style    → iced, snora-design
snora-widgets  → iced, snora-core, snora-design, snora-style
snora          → iced, snora-core, [snora-design], [snora-style], [snora-widgets]
```

Still strict, still acyclic, one layer longer. `architecture.md` opens *"Snora
is four crates with a strict dependency direction"* and becomes five.

### Public paths do not move

`snora-widgets` re-exports `snora_style` under its existing
`design::style::*` paths, so its API is unchanged. `snora::design::style::*`
is unchanged for consumers. **Nobody's import breaks**, and no migration guide
is required.

### The feature graph

```toml
design = ["dep:snora-design", "dep:snora-style", "snora-widgets?/design"]
```

`"widgets"` is gone from the list. **`"snora-widgets?/design"` must stay, in
its conditional form.** RFC-054's investigation found this the hard way: drop
it, and enabling `widgets` + `design` together stops activating
`snora-widgets`'s own `design` submodule, failing with `E0433`. The `?` means
it applies only when `snora-widgets` is already present.

## The cost this RFC does not hide

**`snora::design` becomes partially available.** Its style and token
re-exports work without `widgets`; its widget-layer re-exports —
`design::widget`, `button`, `card`, `notice`, `chip`, `progress` — do not, and
must be `#[cfg(feature = "widgets")]` inside `design.rs`.

That is a real complexity cost, and it is the same objection RFC-054 raised
against option A. It is unavoidable for any option that makes design-without-
widgets reachable: if the configuration exists, the parts that need widgets
must be gated. The difference from A is that B′ also *fixes the layering*,
where A would add the configuration and leave the misplacement in place.

Two further costs, both one-time:

- **A fifth crate to publish**, in order. `cargo publish --workspace` resolves
  order itself, so the release process changes only in that one more crate
  appears in its output.
- **`architecture.md`'s crate count and diagram** change, and that document is
  load-bearing — RFC-035 corrected it once already for being wrong about the
  crate graph.

## Non-goals

- **No public path changes.** Every `snora::design::*` and
  `snora_widgets::design::*` path in use today keeps working.
- **No move of widget-layer modules.** `widget`, `button`, `card`, `notice`,
  `chip`, `progress` stay in `snora-widgets`.
- **No change to `snora-design`.** It is iced-free by hard constraint;
  `snora-style` exists precisely because the bridge cannot live there.
- **No duplication of the card mapping.** RFC-039's reuse decision stands —
  `design::render` keeps calling `card_raised`, now from `snora-style`.
- **No rendering change.** `render_semantics` must pass unmodified.
- **No new capability.** This is a relocation; `design::render` and
  `design::responsive_render` behave identically.

## Open questions

**Q-1 — the crate name.** Recommended: **`snora-style`**. snora's family
convention is plain functional suffixes — `core`, `design`, `widgets` — and
`snora-style` slots into the chain reading correctly. `snora-style-bridge`
names the implementation rather than the layer, and a published crate name is
permanent in a way a module path is not.

**Nothing is deprecated by this RFC.** The style layer is a *module inside*
`snora-widgets` today, not a published crate — extraction adds a fifth crate,
it does not replace a fourth. `snora-widgets` keeps re-exporting the same
paths, so even the module path survives. No crate, module, or import is
retired, and no deprecation notice is needed anywhere.

**Q-4 — when does the `snora-widgets` re-export get deprecated?**
`snora_widgets::design::style::*` becomes a compatibility shim the moment
`snora-style` exists, and shims accumulate.

**Not in this release.** `snora::design::style::*` — the path applications
actually use — is re-exported *through* `snora-widgets`, so deprecating the
widgets path first would warn consumers who did nothing wrong. The order is:
re-point `snora`'s own re-export at `snora-style`, document `snora-style` as a
consumer-facing path, *then* deprecate.

Recorded here so the obligation stays visible rather than being discovered as
debt later.

*(Q-2 and Q-3 were resolved by the owner, 2026-08-15, and are now acceptance
criteria 5 and 9 rather than open questions.)*

## Acceptance criteria

1. `snora-style` exists with `iced` and `snora-design` as its only
   dependencies, containing the five style modules.
2. `snora-widgets` re-exports them at their existing paths; its public API is
   unchanged.
3. `design` no longer requires `widgets`, and retains
   `"snora-widgets?/design"` conditionally.
4. **`snora --no-default-features --features design` compiles**, and a probe
   in that configuration builds — the configuration this RFC exists to create.
5. **The default configuration must not start compiling style code.** Today
   `snora-widgets` gates its whole `design` module — which contains `style/` —
   behind its own opt-in `design` feature, so a consumer with `widgets` and
   without `design` compiles none of it. `snora-style` must therefore stay an
   **optional** dependency of `snora-widgets`, activated by that same feature:
   `design = ["dep:snora-design", "dep:snora-style"]`. Prove it by measuring:
   `size_probe_widgets` must not grow, and `size_probe_design` must not
   regress.
6. `render_semantics` passes **unmodified**.
7. `architecture.md` describes five crates with the corrected diagram;
   `feature-flags.md` **loses** the "Requires `widgets`" caveat.
8. No consumer-visible import path changes; no migration guide needed.
9. **`design` and `widgets` are independent**, giving four expressible
   configurations where three exist today: neither (engine only), `widgets`
   alone (today's default), **`design` alone (new)**, and both. Confirm no
   existing configuration loses a path it has today — the new one is purely
   additional.

## Compatibility and security

**Compatibility.** Additive at the crate level, invisible at the API level.
The only consumers affected are those who would newly *choose*
design-without-widgets.

**Security.** No new data flow or third-party dependency; `snora-style` takes
`iced` and `snora-design`, both already in the graph.

## Release implications

**0.32.0, minor.** A new published crate, sharing the workspace version.
`CHANGELOG.md` under **Changed**, stating that no import path moved — that is
the sentence most likely to prevent an unnecessary migration scare.
