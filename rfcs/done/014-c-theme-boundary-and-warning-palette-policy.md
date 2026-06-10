# RFC-014-C — Theme Boundary and Warning Palette Policy

**Status.** Implemented (v0.14.0)
**Tracks.** Design boundary / documentation / minor code cleanup.
**Touches.** `crates/snora/src/toast.rs` (extract const + comment),
`docs/src/contributing/design-decisions.md` (theme-boundary record),
`docs/src/reference/vocabulary.md` (ToastIntent note).

## 1. [Decisions]

### Warning fallback: extract as named private const

The inline `Color::from_rgb8(0xD9, 0x77, 0x06)` in `toast.rs` is
correct but undocumented. Extract to:

```rust,ignore
/// Fallback color for [`ToastIntent::Warning`].
///
/// iced's extended palette has no `warning` semantic pair (unlike
/// `primary`, `success`, and `danger`). This stable amber/orange is
/// chosen to remain readable against both light and dark iced themes.
/// It is a Snora implementation detail — applications cannot configure
/// it through the theme API, and it may change when iced adds a warning
/// semantic. See RFC-014-C.
const WARNING_COLOR: Color = Color::from_rgb(0.851, 0.467, 0.024);
```

(`from_rgb` takes 0..1 floats; `0xD9/0xFF ≈ 0.851`, `0x77/0xFF ≈ 0.467`,
`0x06/0xFF ≈ 0.024`.)

The const stays `const` (not `pub const`) — private implementation detail.

### Style review checklist: add to design-decisions.md

Record the four-question review checklist for future style changes:
1. Does the change add a public color/token type?
2. Does it derive from iced `Theme` where possible?
3. Does it add a dependency?
4. Does it affect binary size?

### No `warning_fallback_pair` helper

The warning path is a single `match` arm. Extracting to a function would
add indirection without value. The named const plus the doc comment is
sufficient.

## 2. vocabulary.md update

Near `ToastIntent`, add a note:

> `ToastIntent::Warning` uses a private fallback color because iced's
> extended palette has no warning semantic pair. The color is stable across
> iced versions but is an implementation detail, not a public design token.

## 3. ABDD check

Not direction-sensitive. ABDD does not apply.

## 4. Acceptance criteria

- `WARNING_COLOR` const exists in `toast.rs` with doc comment.
- `toast.rs` warning match arm references the const.
- `vocabulary.md` has the warning fallback note.
- `design-decisions.md` has the theme-boundary and style-checklist records.
- No public API change.
