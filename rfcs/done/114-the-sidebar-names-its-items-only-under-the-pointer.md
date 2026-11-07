# RFC-114 — The sidebar names its items only under the pointer

**Status.** Done — shipped v0.54.0. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/114-…`](../handoffs/114-the-sidebar-names-its-items-only-under-the-pointer/implementation-handoff.md).
**Raised.** 2026-11-04, architect, from the owner's question about older users (Theme
E). The owner approved opening it.
**Release target.** 0.54.0 (Q-4). **Priority.** A usability defect for pointer, touch and
older users: it asks every reader to remember what an icon means. *(Corrected
2026-11-04: this said "a defect for keyboard users". See the correction below.)*

---

## The finding

**The rail is icons only.** `app_side_bar`, on both paths, draws each `SideBarItem` as
a 48 × 48 button holding its icon, inside a 64 px rail (`crates/snora-widgets/src/sidebar.rs:23,25`).
The item's name, `SideBarItem::tooltip`, appears only in a tooltip.

**iced opens a tooltip only while the pointer is over it.**
`iced_widget-0.14.2/src/tooltip.rs:224` moves to `Hovered` and then `Open` from
`cursor.position_over(bounds)`. Nothing else opens it: not focus, and not a key.
So:
- **a keyboard user** cannot reach the rail at all. In iced 0.14 a button is not
  focusable and takes no keyboard input, in either form of the rail (see the
  correction below). Their route to a view is an application shortcut, like orbok's
  `Ctrl/Cmd+1..6`;
- **a touch user** has no resting pointer to hover with;
- **an older user** must either remember what each icon means or hold the pointer
  still long enough for the tooltip's delay, which a tremor makes hard.

**Our rustdoc claims the opposite.** `SideBarItem::tooltip`
(`crates/snora-core/src/sidebar.rs:29`) says: "Required for accessibility — keyboard
and screen-reader users rely on it." Keyboard users cannot reach it. iced 0.14 has no
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
only in the icon form. Keyboard users cannot reach the rail in either form, because
iced 0.14's buttons take no keyboard input, and no screen reader reads it.

**R-4 — the compact form says what it costs.** The icon rail stays as the compact
choice for narrow windows (`examples/responsive` collapses the rail by width). Its
docs say that it names items only under the pointer, so touch users and anyone who
cannot hold the pointer still get no names.

## Open questions

**Q-1 — the default.** **Suggest `Visible`.** ABDD puts the accessible form first. The
icon form makes every reader recall what an icon means, and gives a touch user no
name at all.
- **Who pays:** only applications that draw snora's prefab rail. arama composes its
  own (it has no `snora-widgets` at all). From what the teams have told us, orbok and
  knotra use prefab widgets; whether either uses `app_side_bar` is **not known**.
  The migration guide names the change; we do not guess at who it reaches.
- **The cost:** every application that shows `app_side_bar` gets a wider rail. Its
  body loses that width, and screenshot baselines change.
- An application that wants the compact rail passes `SideBarLabels::Tooltip`, one
  argument.
- **The alternative:** keep `Tooltip` as the default and document `Visible`. That
  leaves recall over recognition as the default, which ABDD argues against.

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

---

## Correction, 2026-11-04 (while writing the handoff)

This RFC first argued from keyboard users: "a keyboard user who tabs along the rail
sees a focus ring around an icon, and never its name". **That was false.** iced
0.14's `button` handles mouse and touch only, and registers no focus target. A probe
showed that Tab, Enter and Space never press a button
(`.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/`). Keyboard users cannot
reach the rail in either form.

**What still holds:** the labelled form is for pointer, touch and older users, who
recognise a name rather than recall an icon. Q-1's outcome is unchanged; its keyboard
reasoning is removed. The owner accepted the RFC with the keyboard argument in it, so
the change is reported to them. The same finding withdraws the book's claim that iced
buttons take Enter / Space (six files, corrected the same day).

## Amendment, 2026-11-04 (owner's direction): three forms, the application chooses

The owner asked whether "beside or beneath" had to be a single choice, or whether
each application could make its own. It does not have to be one choice. **The
vocabulary names all three forms, and the application chooses:**

| `SideBarLabels::` | Item | Rail width |
|---|---|---|
| `Beneath` | icon, with the name centred beneath it | the wider of the icon button and the longest name, capped |
| `Beside` | icon at the start edge, the name after it, mirrored under RTL | icon + longest name, capped |
| `Tooltip` | icon only, name in a pointer tooltip (today's rail) | 64 px |

**Default: `Beneath`** (the architect's suggestion; the owner may override it before
the handoff is released). The reasons:
- it is what the one adopter with evidence asked for: orbok, 2026-11-04, older
  users;
- it keeps the rail narrowest of the two labelled forms, so the applications that
  adopt the default lose the least body width.

**Everything R-1 says of "the labelled form" applies to both `Beneath` and `Beside`:**
- the same string;
- text that follows the host or `Typography`, as orbok scales to ×2.0;
- wrapping, never clipping;
- the whole item highlighted when active;
- at least 48 px per item;
- no tooltip.

The width cap is one number for both forms, set from the dev team's measurement
(Q-2). Elsewhere in this RFC, "`Visible`" reads as "the labelled forms", and "the
default" as `Beneath`.

orbok also reported the icon form's tooltip covering the tab bar's first label when
the pointer rests on the first item. The labelled forms have no tooltip, so they do
not have the problem. The icon form keeps it, and its docs say so.

## Rulings, 2026-11-04

All suggestions accepted, as revised in `393c753`.

- **Q-1:** a labelled form is the default: `Beneath`, per the amendment above.
- **Q-2:** the rail is content-sized, capped at 240 px. The dev team measures the real
  labels before the cap is fixed.
- **Q-3:** `app_side_bar` and `design::app_side_bar` keep their signatures. The
  `_with` variants take `SideBarLabels`.
- **Q-4:** 0.54.0.
