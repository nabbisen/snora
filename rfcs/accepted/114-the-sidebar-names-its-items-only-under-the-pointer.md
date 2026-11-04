# RFC-114 — The sidebar names its items only under the pointer

**Status.** Proposed.
**Raised.** 2026-11-04, architect, from the owner's question about older users (Theme
E). The owner approved opening it.
**Release target.** 0.54.0 (Q-4). **Priority.** A defect for keyboard users today. For
older users it is a usability problem: it asks them to remember what an icon means.

---

## The finding

**The rail is icons only.** `app_side_bar`, on both paths, draws each `SideBarItem` as
a 48 × 48 button holding its icon, inside a 64 px rail (`crates/snora-widgets/src/sidebar.rs:23,25`).
The item's name, `SideBarItem::tooltip`, appears only in a tooltip.

**iced opens a tooltip only while the pointer is over it.**
`iced_widget-0.14.2/src/tooltip.rs:224` moves to `Hovered` and then `Open` from
`cursor.position_over(bounds)`. Nothing else opens it: not focus, and not a key.
So:
- **a keyboard user** who tabs along the rail sees a focus ring around an icon, and
  never its name;
- **a touch user** has no resting pointer to hover with;
- **an older user** must either remember what each icon means or hold the pointer
  still long enough for the tooltip's delay, which a tremor makes hard.

**Our rustdoc claims the opposite.** `SideBarItem::tooltip`
(`crates/snora-core/src/sidebar.rs:29`) says: "Required for accessibility — keyboard
and screen-reader users rely on it." Keyboard users cannot see it. iced 0.14 has no
accessibility tree, as `accessibility.md` already says correctly, so no screen reader
reads it either. The sentence is published on docs.rs, and is withdrawn by R-3.

**Recognition beats recall.** A visible label lets someone recognise an item instead
of recalling an icon's meaning. That matters most to people whose short-term memory
or vision has changed. Icons alone save width and look clean ("less is more"), but
for these readers, less becomes too little.

## Proposal

**R-1 — a labelled rail.** The rail gains a second form: each item shows its icon
**and its name**, in a row: icon at the start edge and name after it, mirrored
under RTL.
- **The name is the same string,** `SideBarItem::tooltip`. No vocabulary field
  changes.
- **Width:** the rail is as wide as its longest label, capped at 240 px, and a label
  wraps rather than clipping (RFC-105's rule, Q-2).
- **Text** uses the same size as other chrome labels: the host's `default_text_size`
  on the unstyled path, `Typography::label` on the design path (RFC-104).
- **The active highlight** and the 48 px minimum height per item are unchanged.
- **No tooltip** in the labelled form. It would only repeat what is already visible.

**R-2 — choosing the form is vocabulary, not a flag.**
`snora_widgets::SideBarLabels::{Visible, Tooltip}` chooses the form, on both
paths. **Which one is the default is Q-1**, and how a caller passes it is Q-3.

**R-3 — withdraw the false claim.** `SideBarItem::tooltip`'s doc says what is true:
the item's name, shown beside the icon in the labelled form, and on pointer hover
only in the icon form. In the icon form keyboard users do not see it, and no screen
reader reads it.

**R-4 — the compact form says what it costs.** The icon rail stays as the compact
choice for narrow windows (`examples/responsive` collapses the rail by width). Its
docs say that keyboard users get no names in it. snora cannot fix that in the icon
form: a label on focus would need to know which widget has focus, and that needs
iced's `advanced` feature, which the owner has ruled must never be a default
(RFC-078).

## Open questions

**Q-1 — the default.** **Suggest `Visible`.** ABDD puts the accessible form first, and
the icon form cannot name its items to a keyboard user at all.
- **Who pays:** only applications that draw snora's prefab rail. arama composes its
  own (it has no `snora-widgets` at all). From what the teams have told us, orbok and
  knotra use prefab widgets; whether either uses `app_side_bar` is **not known**.
  The migration guide names the change; we do not guess at who it reaches.
- **The cost:** every application that shows `app_side_bar` gets a wider rail. Its
  body loses that width, and screenshot baselines change.
- An application that wants the compact rail passes `SideBarLabels::Tooltip`, one
  argument.
- **The alternative:** keep `Tooltip` as the default and document `Visible`. That
  leaves today's keyboard defect as the default, which ABDD argues against.

**Q-2 — the width rule.** **Suggest content-sized, capped at 240 px.** A reader can
predict it: the rail fits its longest name, up to a limit. A fixed width would waste
space with short names and clip long ones. 240 is a proposal. The dev team measures
the real labels in the examples, at the default text size and at 24 px, before it is
fixed.

**Q-3 — the signature.** `app_side_bar` gains a parameter, which is a breaking change
for every caller. **Suggest:** keep `app_side_bar(side_bar, direction)`, and give it
the Q-1 default. Add `app_side_bar_with(side_bar, direction, labels)` for the other
form. Callers compile unchanged, and only what they see changes. The design path follows
the same pattern: `design::app_side_bar(tokens, side_bar, direction)` keeps its
signature, and gains `design::app_side_bar_with`. Nothing is superseded, so nothing is
deprecated.

**Q-4 — release.** **Suggest 0.54.0, with Theme E's other two RFCs.** It changes
default appearance if Q-1 is ruled `Visible`, so the migration guide leads with it.

## Review against snora's principles

- **ABDD:** Q-1 suggests the accessible form as the default, and states the cost
  plainly.
- **Rules a reader can predict:** the rail fits its longest name, up to 240 px.
- **Verify claims:** the tooltip behaviour was read in iced's source, and the false
  doc claim is withdrawn (R-3).
- **Priced against real adopters:** the cost names who draws the prefab rail, and
  says what is not known (Q-1).
- **Correspondence cost:** no letter. The default-appearance change goes in the
  migration guide, where adopters look when they bump.

## What this is not

- Not a change to `SideBar` or `SideBarItem`'s fields.
- Not an expandable or collapsible rail driven by snora state. Width-based switching
  stays with the application, as in `examples/responsive`.
- Not icon sizes, which are RFC-115's inventory.

## Acceptance criteria

1. **Failing first:** a rendered test that finds each item's name drawn at rest,
   without hover, in the default rail. It fails today.
2. Labels wrap at the cap, measured with a long label. RTL mirrors the icon and the
   label.
3. The icon form unchanged: today's sidebar tests pass unmodified with
   `SideBarLabels::Tooltip`.
4. R-3's doc rewritten. CHANGELOG under Changed (default appearance) and Fixed (the
   doc claim).
