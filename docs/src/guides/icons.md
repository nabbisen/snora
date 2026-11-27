# Icons

`Icon` is a single enum with feature-gated variants. Choose your icon
source per call; nothing is global.

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:icons_enum}}
```

When a feature is disabled the variant does not exist in the enum. No
runtime "unknown icon kind" branch is reachable.

## `Icon::Text` — the always-available path

```rust,ignore
{{#include ../../../examples/book_snippets/src/guide_icons.rs:icons_text}}
```

Strings can be a single Unicode glyph (`★`, `✓`, `↓`, `🛠`) or a short
label (`"OK"`). The engine renders text icons at the same default
font size as labels in built-in widgets, so they line up visually.

This variant has no asset dependency, no feature flag, and works on
every platform that iced supports.

## `Icon::Lucide` — the curated icon set

```toml
[dependencies]
snora = { version = "0.66", features = ["lucide-icons"] }
```

```rust,ignore
{{#include ../../../examples/book_snippets/src/guide_icons.rs:icons_lucide}}
```

**Load the Lucide font when your application starts.** A Lucide icon is
drawn as a glyph of the Lucide font, and snora cannot load a font for you: only
your application can, with `.font(..)` on `iced::application`. Without it, every
`Icon::Lucide` draws as a stray glyph or a missing-glyph box (▯).

```rust,ignore
{{#include ../../../examples/book_snippets/src/guide_icons.rs:icons_lucide_font}}
```

**snora's own marks do not need the font** (0.66, RFC-139). The toast's close
mark, a notice's dismiss mark, a chip's remove mark, the checkbox's tick and the
select's chevron are the text glyphs "×", "✓" and "▾" in every build. Turning on
`lucide-icons` adds the Lucide set for *your* icons and changes nothing snora
draws itself. That matters because a Cargo feature can be turned on by any crate
in your dependency graph, not only by you. *(Before 0.66, with the feature on and
the font not loaded, those marks drew a box, a "1" in a checked checkbox, and a
"3" for the select's arrow.)*

**The feature brings the whole set:** one enum with a variant for every Lucide
glyph, and one font file that holds them all. Referencing one icon does not
leave the others out of the font you load.

*(Corrected 2026-11-27, from the 0.67 audit: this said Cargo includes only the
icons you reference, and it did not say the font must be loaded.)*

## `Icon::Svg` — your own assets

```toml
snora = { version = "0.66", features = ["svg-icons"] }
```

```rust,ignore
{{#include ../../../examples/book_snippets/src/guide_icons.rs:icons_svg}}
```

The engine reads the file at render time using iced's SVG widget.
Pixel size is the same default as the other variants.

**An SVG icon keeps the file's own colours.** snora draws it with iced's
`svg` widget and no style, so neither the theme nor the tokens recolour it,
and none of snora's contrast guarantees covers it. A black icon stays black on a
dark theme. Draw it in colours that hold 3:1 against every surface it can sit
on, in every theme you ship. A `Text` or `Lucide` icon is different: it is drawn
as text, in the colour of the text around it.

### The path is yours to trust

**snora does not validate the path or the file.** `Icon::Svg` hands the path to
iced, which reads it from disk at render time and parses it as SVG. That is a
filesystem read and an XML parser, on whatever path your application supplies.

For the ordinary case — an `assets/` directory you ship — that is exactly what
you want and there is nothing to think about.

**If the path or the file can come from somewhere you do not control** — a plugin
directory, a theme a user installs, anything downloaded — then the trust boundary
is yours, not ours. snora adds no sandbox, no path restriction, and no size or
complexity limit beyond whatever the SVG renderer imposes. Validate before you
construct the `Icon`.

We state this because *"your own assets"* above is a heading, not a guarantee,
and nothing in the type signature says which of the two situations you are in.

## Sizing

**An icon is 1 em of the text it sits with** (0.55, RFC-115): `icon_element`
reads the host's `default_text_size` when the layout runs, so it is 16 px at
iced's default and 24 px if your application sets 24. Before 0.55 it was a
fixed 14 px that followed nothing. On the design path, snora's widgets draw
their icons at the label token (`Typography::label`, 14), unchanged. The
toast's × is 1.125 em. To name a size yourself:

```rust,ignore
{{#include ../../../examples/book_snippets/src/guide_icons.rs:icons_sized}}
```

## ABDD: icons should not be the only signal

Icons are a *secondary* signal. Always pair them with a **visible**
text label. A tooltip is not enough: it appears only under a resting
pointer, so touch users, keyboard users and anyone with an unsteady hand
never see it. iced 0.14 has no accessibility tree, so no screen reader
reads either. The prefab `app_side_bar` shows each item's name on the
rail by default since 0.54 (RFC-114). Do the same in your custom
widgets.

*(Corrected 2026-11-07: this said a tooltip served keyboard and
screen-reader users.)*

**A toast's intent (information, success, warning, error) is shown by
colour alone.** Nothing else in the toast changes with it, and snora's own
tests assert exactly that. So say what happened in the toast's title and
message ("Could not save", not only a red toast). Whether snora should carry
intent by more than colour is a question for 0.67.

*(Corrected 2026-11-27, from the 0.67 audit: this said the toast encodes
intent by both its background colour and the surrounding text.)*

## Why icons are feature-gated

`Icon::Text` has no extra dependencies and is always available. The
richer icon sources are optional because:

- The `lucide-icons` crate ships a constant for every Lucide glyph, and a font
  file, and contributes non-trivial compile time. Projects that only need emoji or text glyphs should not pay
  that cost.
- `Icon::Svg` requires iced's `svg` feature and a file path. Not all
  applications need SVG rendering.

Keeping both optional means engine-only builds (`--no-default-features`)
stay small and the CI feature matrix can verify each combination
independently.

## Supported feature combinations

```toml
# Default (includes widgets + text icons):
snora = { version = "0.66" }

# Engine only — no widget re-exports, no icon packs:
snora = { version = "0.66", default-features = false }

# Widgets + Lucide icon constants:
snora = { version = "0.66", features = ["widgets", "lucide-icons"] }

# Widgets + SVG icon support:
snora = { version = "0.66", features = ["widgets", "svg-icons"] }

# Widgets + both icon sources:
snora = { version = "0.66", features = ["widgets", "lucide-icons", "svg-icons"] }
```

`lucide-icons` and `svg-icons` are **subordinate** to `widgets`: they
gate widget-side rendering and are not meaningful without it. CI's feature
matrix compiles each of these combinations on every push, with two
differences: the engine-only build is compiled with `iced-default` on (an
application's own `iced` line supplies it), and the last row is compiled only
together with `design`, as part of `--all-features`.

*(Corrected 2026-11-28, from the 0.67 audit: this said CI tests all of these
combinations.)*
