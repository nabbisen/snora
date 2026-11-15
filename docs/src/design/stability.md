# Stability

If you are deciding whether to adopt `snora-design`'s token surface, this
page answers the question directly: **does it churn?**

**No.** The token surface — `Tokens`, `Palette` and its 18 role fields,
`Color`, `Spacing`, `Typography`/`TextRole`, `Radius`, `FocusTokens`,
`Tone`/`Density`, and the three contrast functions
(`relative_luminance`, `contrast_ratio`, `composite_over`) — is under a
contractual additive-only covenant. It cannot be removed, renamed, or
retyped without an explicit, recorded process failure on our side: reopening
two closed stability gates in the same change, with no allowance to proceed
and rationalize afterward.

## What is frozen

RFC-036's additive-only covenant, normatively recorded in
[API governance § Additive-only covenant](../contributing/api-governance.md#additive-only-covenant-design-surface),
freezes two surfaces by name:

- **The token surface** — `Tokens` and its six constructors (`light`,
  `dark`, `light_aaa`, `dark_aaa`, `high_contrast_light`,
  `high_contrast_dark`) and `presets()`, `Palette` and its 18 role fields
  and `text_contrast`, `TextContrast`, `Color`, `Spacing`,
  `Typography`/`TextRole`, `Radius`, `FocusTokens`, `TargetTokens`,
  `Tone`/`Density`, and — named
  individually — the `contrast` module's `relative_luminance`,
  `contrast_ratio`, `composite_over`.
- **The style-bridge surface** — every public function of `snora_style`
  (the `color`, `button`, `chip`, `container`, `progress`, `tab`, `text`, and
  `theme` modules) — see [API governance § Additive-only
  covenant](../contributing/api-governance.md#additive-only-covenant-design-surface)
  for the current function list rather than a second, hand-maintained
  copy here; a per-function list on this page went stale once already
  (RFC-075: missing `text::*_line_height` and `theme::theme`).

Removing, renaming, or retyping any item on either list, or changing the
signature of any style-bridge function, is **forbidden** without reopening
gates D-3 and D-4 (`api-freeze-review.md`) in the same change, stated
explicitly. New items — new token fields on the `#[non_exhaustive]` types,
new presets, new style-bridge functions — can be added freely; that is what
"additive-only" permits.

**The reopening path has been used once, at 0.45.0** — not a hypothetical.
`Emphasis` and `Size`, two variant enums, were removed after many minors in
which nothing read either, confirmed by every adopting team rather than
inferred from their silence. Gates D-3 and D-4 (`api-freeze-review.md`)
reset to open in that same change, per the covenant's own condition, and
were re-earned at 0.47.0; both are closed again. See
[migration 0.44 → 0.45](../guides/migration-0.44-to-0.45.md) for what that
means for your own code — for nearly everyone, nothing, since nothing read
either type.

One narrow exception exists for preset **values** specifically, and it is
the only one that does not require reopening a gate: a value **inside** a
preset (a specific `Color`, not a type or field) may change where a
contrast test proves the change fixes an accessibility defect, recorded as
**Fixed** in the CHANGELOG. This is not a loophole in the freeze — it is
the mechanism that lets a defect like the one in
[0.33→0.34](../guides/migration-0.33-to-0.34.md) get fixed without
reopening the gates. See the governance page linked above for the full
rule and its forbidden alternatives.

## You do not need to re-check our contrast

Every `Palette` role declares which surfaces it renders on and at what
threshold, and the contrast pairs are *derived* from that declaration rather
than maintained beside it. A role cannot be added without answering the
question — the compiler refuses (`E0027`). So the roles you consume are
asserted by snora's own test suite, on every CI run of snora (not when you
build your application). **It covers the declared pairs.** Text a widget
draws in a colour of its own is measured separately, on the drawn pixels:
buttons, chips, links and a notice's action since 0.62, and the rest by
RFC-131. *(Corrected 2026-11-15, from the 0.63 audit: this said "on every build".)*

If you keep your own pair list over snora's roles, you are duplicating that
work. Your list only needs to cover **your own** roles and **your own**
surfaces.

Two consumers have asked whether they can copy the mechanism itself. You can —
onto your own palette type, in your own crate. You cannot apply it to *snora's*
`Palette` from outside: `#[non_exhaustive]` permits exhaustive destructuring
only inside the defining crate, so the compiler will require `..`
(`E0638`), and `..` silently defeats the enforcement. That is a language
boundary, not a mistake on your side.

## What is not frozen

The design **primitives** — the `button`, `card`, `notice`, `chip`,
`progress` prefab widget helpers in `snora-widgets` — are **deliberately
excluded** from this covenant and run on a different, more permissive
lifecycle (Recipe → Experimental helper → Stable primitive → Deprecated →
Removed). If your integration depends on a primitive's exact behaviour or
signature, that guarantee does not extend as far as the token surface's
does. [API governance](../contributing/api-governance.md) describes the
stages; it does not record which stage each primitive is at, so do not assume
one is Stable.

## What this does not mean

**This is not a claim that upgrading is mechanically painless.** snora is
pre-1.0, and `0.33` → `0.34` is an incompatible bump to Cargo regardless of
what changed — SemVer arithmetic does not know about this covenant and
this page does not override it. What the covenant answers is narrower and,
for most adopters, the more important question: whether the token
**surface** — the types and functions your code calls — will still be
there, with the same shape, after an upgrade. It is contractually forbidden
to remove or reshape it except by the reopening path above (used once, at
0.45.0), or by the accessibility exception, which covers preset values. That
is not the same guarantee as "your code compiles unchanged against every
future minor" — always read the per-version
[migration guide](../guides/migrations.md) — but it does mean the surface
itself is not a moving target while you decide whether to depend on it.
