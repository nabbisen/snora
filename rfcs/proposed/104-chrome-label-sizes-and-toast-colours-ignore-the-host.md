# RFC-104 — Chrome label sizes and toast colours ignore the host

**Status.** Proposed.
**Raised.** 2026-09-26, architect. Source: REQ-002 (tekstide: *"the single item
that would keep us out regardless of the others"*). It is also a defect for
current consumers.
**Release target.** 0.52.0. **Priority.** Second: the largest defect, and it
needs a design ruling.

---

## The finding

**Chrome label sizes are literals in both the unstyled and the styled
variants:**

| Site | Literal |
|---|---|
| `tab.rs` label | 13 |
| `crumb.rs` label and separator (×3) | 13 |
| `menu.rs` header and items (×2) | 14 |
| `header.rs` title | 16, bold |
| engine `toast.rs` title / message | 16 / 14 |

The styled variants map spacing and radius to tokens, but **pass no text
size**. Only RFC-102's sidebar tooltip reads a typography token. **A host that
raises its `Typography`, for an accessibility setting for example, does not
move snora's chrome labels.** orbok offers "Text size: Default / Large /
Larger", and its tabs, menus, crumbs and header stay put.

**Colours from outside the theme:**

- the toast's Warning fill is a literal `WARNING_COLOR`, although iced 0.14's
  `Palette` has `warning`;
- toast text on fills is fixed `Color::BLACK` / `WHITE`, chosen by RFC-085 for
  measured contrast;
- the default-path modal dim is a literal 40 % black (the design path derives
  it).

**The toast has no token path at all.** The engine's design path styles chrome
and the dialog card, not toasts.

### The constraint that shapes the fix

iced 0.14 text sizes are **absolute**, and a view function **cannot read** the
application's `default_text_size`. So "take the size from the host" means
something different on each path:

- **styled:** the host's `Typography` tokens, i.e. a `TextRole` per label;
- **unstyled:** either iced's own default (by dropping `.size(..)`), or an
  explicit input.

## Proposal

**R-1 — styled chrome labels read `TextRole`.** Each label maps to a role, in
one table like the existing geometry mappings. Exact roles: Q-2.

**R-2 — the unstyled path.** Shape: Q-1.

**R-3 — the toast gets a token path and a theme path.** On the design path,
title and message take `TextRole`s. On both paths, Warning's fill comes from
the theme (`palette.warning` / the design palette). **The fill's contrast with
its text must be re-measured and asserted before the literal is dropped**:
RFC-085 chose the literal for contrast, and the theme's colour may not clear
it. If it does not, keep the literal and record why (Q-3).

**R-4 — a build-failing scan for literal sizes.** This is tekstide's
"what keeps the rule true a year later". A `scripts/check-literal-sizes.sh`, in
the existing `check-*.sh` family, wired into CI, fails on a `.size(<number>)` in
non-test widget and engine source. Named constants that are deliberate floors
or glyph metrics must be listed in the script with a reason, and the list is
reviewed. Shape of the allow-list: Q-4.

## Open questions

**Q-1 — unstyled label sizes.**

| Option | Shape | Consequence |
|---|---|---|
| **(a) Drop the literals; inherit iced's default** | `text(label)` with no `.size` | "From the host" in the purest sense: the app's `default_text_size`. **An appearance change for every unstyled app** (tabs 13 → 16 at iced's default) |
| (b) Explicit size inputs | e.g. geometry / builder parameters | API growth on every prefab, and the caller must know to use it |
| (c) Leave unstyled as it is | Styled-only fix | Unstyled stays hardcoded, so the scan needs an exemption for it, which weakens R-4 |

**Suggest (a).** It is the only option under which the unstyled path genuinely
follows the host. The appearance change is real and goes in the migration guide.
Apps that want the old compact look set `default_text_size`.

**Q-2 — which `TextRole` per styled label?** Suggest `Label` for tab, crumb and
menu items, `Title`/`Heading` for the header title and the toast title, and
`Body` for the toast message. The handoff lists the exact variants that exist,
and the table records any that do not map cleanly.

**Q-3 — the toast's Warning colour** if the theme's warning fails contrast
against the toast's text: keep the literal with a recorded reason, or pair the
theme colour with a text colour chosen for it. Decided by measurement in the
handoff, with the owner ruling if neither is clean.

**Q-4 — the scan's allow-list.** Suggest: only named `const`s with a comment
naming them as a floor or a glyph metric (e.g. `CLOSE_GLYPH_SIZE`), and no bare
numbers anywhere.

## What this is not

- **Not a covenant change.** `Typography` and `TextRole` are *read*, not
  changed. Changing their values would be forbidden; nothing here does.
- **Not a font-family change.** snora sets none, except lucide glyphs.

## Acceptance criteria

1. Every styled chrome label reads a `TextRole`, and a mapping test covers each.
2. Unstyled labels follow Q-1's ruling.
3. The toast has a design path for text sizes; Warning's fill follows Q-3, with
   contrast **asserted** for every intent on both paths.
4. `check-literal-sizes.sh` exists, is wired into CI, and is **shown failing** on
   a scratch `.size(13)`; the allow-list is as ruled in Q-4.
5. A rendered test shows a label's size changing when the host's typography (or
   default size) changes, **shown failing** against today's literals.
6. Migration guide: the appearance change, including how to restore the old
   sizes.
