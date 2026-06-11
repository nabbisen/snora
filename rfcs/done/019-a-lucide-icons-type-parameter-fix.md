# RFC-019-A — Lucide Icons Type-Parameter Fix

**Status.** Implemented (v0.18.1)
**Tracks.** Bug fix / downstream compatibility.
**Touches.** `crates/snora-widgets/src/icon.rs`.

## 1. Bug report

A downstream application (nabbisen's project) failed to build
`snora-widgets v0.18.0` with `lucide-icons` enabled:

```
error[E0277]: the trait bound
  `Element<'_, Message, Theme, Renderer<Renderer, Renderer>>:
   From<Text<'_, Theme, ()>>` is not satisfied
  --> snora-widgets/src/icon.rs:27
   = note: there are multiple different versions of crate `iced_core`
           in the dependency graph
```

## 2. Root cause

`snora-widgets/src/icon.rs` called `lucide_const.widget()` to render a
Lucide icon. `lucide-icons` v1.x declares `iced = "0.*"` as a dependency,
which Cargo resolves independently from snora's `iced = "0.14"` pin. Even
when both resolve to `0.14.x`, subtle differences in Cargo resolution can
produce two entries for `iced_core` in the dependency graph.

`lucide_icons::Icon::widget()` returns `iced::widget::Text<'a>` from
*lucide-icons'* iced_core. When that `Text` is passed to `.into()` to
produce snora-widgets' `Element<'_, Message, Theme, Renderer>`, the
conversion requires `From<Text<'_, Theme, Renderer>>` — but Cargo sees
two different `Text` types (different crate instances), so the trait bound
is not satisfied.

## 3. Fix

Replace the `lucide_const.widget().size(size).into()` call with:

```rust
let glyph = char::from(*lucide_const).to_string();
text(glyph)
    .font(iced::Font::with_name("lucide"))
    .size(size)
    .into()
```

`From<Icon> for char` is defined in lucide-icons with no iced dependency
at all — it is a pure mapping from icon variant to unicode codepoint. We
then construct the `Text` widget using snora-widgets' own `iced::text()`
import, which is always the correct iced_core version.

The visual output is identical: both paths produce a text widget using the
`"lucide"` font at the given codepoint.

## 4. Classification

Per the versioning policy (RFC-015-A): this is a **Fixed** entry. The
documented behavior (Lucide icons render correctly) was broken by an
implementation detail. No API change.

## 5. Evidence

This is the first downstream build failure reported by a real application.
It satisfies part of Gate 3 (third-party app using snora). The app is
nabbisen's project `logolig`.

## 6. Acceptance criteria

- `snora-widgets` compiles with `lucide-icons` feature regardless of
  iced_core duplication in the dependency graph.
- All existing tests pass.
- No visual change to icon rendering.
