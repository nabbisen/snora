# RFC 085 — The widget layer pairs colours from different families, and no contrast suite can see it

**Status.** Accepted (owner, 2026-09-01). Handoff written — see
[`handoffs/085-…`](../handoffs/085-the-widget-layer-styles-are-unreachable-by-every-contrast-suite/implementation-handoff.md).
**Tracks.** Accessibility / test reach. **Severity: Critical.**
**Found by** an external architect's audit, 2026-09-01 (F-13, F-14, F-15, F-31).
**F-13 measured by the architect before opening.**
**Touches.** `crates/snora-widgets/src/style.rs`,
`crates/snora-widgets/src/sidebar.rs`, chrome border styles, and the contrast
suites in `crates/snora-design/src/tests.rs`.
**Release target.** 0.41.0 — **minor.** Rendered appearance changes.

## Measured, not asserted

`menu_button_style` sets `text_color` from **`ep.primary.weak.color`** — a
**background-tier** colour used as a foreground, on `background: None`, so it
renders over the dropdown surface.

| theme | at rest | hovered | `background.base.text` would give |
|---|---:|---:|---:|
| Light | **1.89:1** | 3.73:1 | **21.00:1** |
| Dark | **2.20:1** | 3.70:1 | **11.00:1** |

**No state of this widget reaches 4.5:1**, and the correctly-paired foreground
is unused in the same struct. Every consumer using the prefab header menu ships
this.

**F-14 — `sidebar_button_style` pairs across families**, and the failure
inverts by path: stock theme's active highlight is **1.89:1 against the rail**
and is the *only* active cue (also a WCAG 1.4.1 use-of-colour issue); on the
`design` path the icon on the highlight measures **2.01:1**, and **1.51:1 under
`high_contrast_dark`** — the preset that exists for low-vision users.

**F-15 — chrome borders are below the non-text floor everywhere.**

## The finding behind the findings — F-31

**These are in `snora-widgets`. Every contrast suite we have is in
`snora-design`, and tests tokens.**

`Palette::usages` declares where a *role* renders. It cannot see what
`menu_button_style` does with `primary.weak.color`, because that pairing is
invented in another crate at render time and is not a token pair at all.

**We spent this entire cycle hardening contrast assertions on the layer that was
already correct.** RFC-058, RFC-063, RFC-065, RFC-066, RFC-071, RFC-081 — every
one of them tightened `snora-design`. The layer that actually paints text has no
contrast assertion of any kind.

That is the defect worth more than the three fixes: **the suites' reach ends one
crate short of the pixels.**

## Non-goals

- **No token value changes.** The tokens are correct and heavily asserted. The
  defect is in what the widget layer pairs them with.
- **Do not "fix" this by adding roles.** RFC-036 freezes the token surface and
  nothing here needs a new role — the correct colours already exist.
- **No change to `Palette::usages`' contract.** It declares role usage and does
  that well; it is the wrong instrument for render-time pairings.

## Open questions

**Q-1 — where does a widget-layer contrast suite live?** `snora-widgets` depends
on `iced`, so it can construct real styles and measure them — `snora-design`
cannot, being iced-free by CI gate. **Suggest `snora-widgets`**, using
`snora-design`'s `contrast_ratio` as a dependency, so the maths has one home and
the measurement happens where the pairing does.

**Q-2 — what is the assertion's subject?** Not "every role against every
surface" — that is the token suite. Here it is **every `button::Style` this
crate produces, in every `Status`, on the background it is actually painted
over.** That set is enumerable from the style functions themselves, and should
be derived rather than listed, per RFC-063's precedent.

**Q-3 — stock `iced::Theme` as well as the `design` path?** The measurements
above show the stock path failing too, and most consumers start there.
**Suggest both**, and note it makes the suite dependent on iced's own palette
derivations, which can change under us — a real cost, stated rather than
discovered later.

**Q-4 — is `high_contrast_dark` at 1.51:1 a release blocker on its own?** It is
the preset that exists specifically for low-vision users, failing worse than the
default. **Suggest yes.**

## Acceptance criteria

1. `menu_button_style`, `sidebar_button_style` and the chrome borders pair
   foreground with the background actually painted, in every `Status`.
2. **Every figure re-measured after the fix**, both paths, all four presets —
   with the numbers stated, not "now passes".
3. A widget-layer contrast suite exists (Q-1/Q-2), derived rather than
   hand-listed, and **fails before the fix** on at least the three findings.
4. Q-3 ruled, with its cost recorded if stock-theme coverage is included.
5. No token value changed; `snora-design`'s suites unmodified.
6. `render_semantics` unaffected; appearance changes recorded as such.

## Compatibility and security

**Compatibility.** Rendered appearance changes on both theme paths — menu text,
sidebar highlights, chrome borders. **Minor**, with a migration guide stating
that reference images are invalidated, per the precedent 0.34.0 set.

**Security.** None, unless illegible controls count — which for an
accessibility-facing framework they arguably do.
