# Migrating from 0.25 to 0.26

v0.26 adds **token-derived theme emission** — the first step of the
appearance milestone — plus a positioning amendment that permits it and a
correction to the budget measurement methodology.

**No breaking changes to any public API.** Everything new is additive and
behind the opt-in `design` feature. If your application does not enable
`design`, **snora's rendered output is unchanged from v0.25** — that is a
compatibility promise, not an aspiration.

## What changed

### `snora::design::theme(&Tokens) -> iced::Theme` (new)

Previously, an application that selected a Snora Design preset got
token-derived styling on the design primitives *it* built — and nothing
else. Its `text_input`, `pick_list`, `scrollable` and window background
followed a separately configured `iced::Theme`, which had to be kept in
agreement with the tokens by hand.

`snora::design::theme` removes that double configuration:

```rust,ignore
use snora::design::{theme, Tokens};

let tokens = Tokens::dark();

iced::application(App::default, App::update, App::view)
    .theme(move |_state| theme(&tokens))
    .run()
```

Stock iced widgets and the window background now follow the same palette
as your design primitives.

**Snora does not hold theme state.** The function is pure: it returns a
value your application owns and passes to iced itself. Nothing in snora
calls it on your behalf.

#### Why the emitted theme is not iced's default generator

The function deliberately does **not** use `Theme::custom`. iced's default
palette generator routes every colour pair through `Pair::new`, which
adjusts foreground colours heuristically until they clear an internal
contrast bar. Passing Snora Design's contrast-tested roles through that
would replace them with approximations, and the WCAG guarantees would not
transfer.

Instead the emitted palette is constructed directly from the token roles,
so the colours you selected are the colours you get. Base tiers match
their token role exactly; `weak`/`strong` tiers and the background
gradient are derived deterministically and are independently
contrast-tested.

See [`design/theme.md`](../design/theme.md) for the full 18→6 mapping,
including the two roles with no direct iced counterpart.

### Chrome colours now follow — geometry does not yet

Because snora's prefab widgets already read iced's palette, adopting the
emitted theme means the header, sidebar, footer and menus pick up your
tokens **with no code change on your part**.

What is *not* yet token-derived, and is planned for v0.27:

- the dialog card (currently unstyled — content is centred, not carded)
- the modal dim (currently a fixed 40% black)
- spacing rhythm, corner radius, and elevation across chrome

If you were expecting a fully coherent visual default, v0.26 delivers the
colour half. The geometry half follows.

### Design-system positioning amended

`design-decisions.md`'s DEC-02 previously recorded *"theme-aware, not
theme-owning"* as a **firm boundary**. That is now split:

- **Theme-owning** — a parallel theming abstraction, snora holding theme
  state, applications configuring appearance through snora rather than
  iced. **Still declined, permanently.**
- **Theme-producing** — a pure function from tokens to an `iced::Theme`.
  **Permitted under the `design` feature.**

This is a governance change, not a behavioural one. It is recorded because
the decision register is the project's account of *why* the API looks the
way it does, and shipping the emitter against an unamended register would
have left the two contradicting each other.

### Budget measurement methodology corrected

Not user-facing, but it changes numbers you may have been reading.

The size probes were byte-identical and never *called* the features they
measured, so the linker stripped the unused code and `widgets_diff_bytes`
measured `0`. The probes now exercise their features, and the diff
measures what adopting a feature actually costs. Build-cost measurement
was also running against a warm dependency cache; that cache is removed,
so those runs now take minutes rather than seconds — correctly.

1.0 gate 9 remains **open** and re-satisfies only once two releases have
been measured under the corrected methodology.

## Upgrading

1. Change `snora = "0.25"` to `snora = "0.26"` in `Cargo.toml`.
2. That is the whole migration if you do not use the `design` feature.
3. If you *do* use `design` and want the emitted theme, wire it into
   iced's `.theme()` hook as shown above. This is opt-in: not doing so
   leaves your application exactly as it was.

## Minimum supported Rust version

**1.88**, inherited from `iced` and `wgpu`. Unchanged from v0.25.3.

**If you are coming from v0.25.0, v0.25.1 or v0.25.2**, note that those
releases declared **no** `rust-version` at all — the field was added in the
v0.25.3 patch, and there is no separate migration guide for a patch. This
is the one thing in the jump that can stop your build.

To be precise about what changed: snora's *actual* floor has been 1.88 for
as long as iced 0.14 has been pinned, because `iced` and `wgpu` both
declare it. Declaring it does not narrow support — it means Cargo now
refuses cleanly up front instead of failing somewhere inside `iced`, and
its MSRV-aware resolver will hold you on an older snora rather than
breaking you outright.

If you are on a toolchain below 1.88, upgrade the toolchain. You could not
have built snora 0.26 or later regardless.
