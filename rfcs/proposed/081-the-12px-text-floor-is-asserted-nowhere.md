# RFC 081 — The 12-pixel text floor is the one mandatory number nothing asserts

**Status.** Proposed
**Tracks.** Accessibility assertions.
**Found by** **tekstide**, 2026-08-20. Verified against source before opening.
**Touches.** `crates/snora-design/src/tests.rs`,
`docs/src/guides/readability.md`, `docs/src/design/typography.md`.
**Release target.** 0.39.2 — test and documentation only. **No values.**

## Summary

`readability.md:75` states it as flatly as anything in our docs:

> **The floor is 12 logical pixels.** Nothing else.

**Nothing asserts it.** Verified: no test in `crates/snora-design/src/tests.rs`
constrains any `typography.*.size`, and `Typography` is a plain struct with
public `f32` fields — `tokens.typography.body.size = 8.0;` compiles.

## Its two neighbours are both enforced

tekstide put the comparison better than we would have:

| mandatory floor | enforcement |
|---|---|
| 24px pointer target | asserted per role **and** padding step — `pointer_target_height_meets_24px_for_every_role_and_padding_step` (`tests.rs:312`) |
| contrast thresholds | **compile error** if a role is added without a threshold class — `Palette::usages` (RFC-063) |
| **12px text size** | **nothing** |

This is RFC-058's shape exactly — a threshold stated in prose with nothing to
fire it — and it is the third instance this project has found in its own
accessibility numbers.

**It also already cost a consumer.** `readability.md` records that an earlier
wording of this floor led knotra to a remediation at roughly twice the size it
needed. The number has a history of being misread, and it stands alone.

## Be precise about what an assertion can and cannot do

**It can prove our built-in presets comply.** Our six roles are 14–32px, so the
assertion passes today and would catch a future preset edit that dropped one
below 12.

**It cannot constrain a consumer's own `Tokens`.** `Typography`'s fields are
public and RFC-036's covenant freezes that surface, so a consumer writing
`body.size = 8.0` is unreachable by any test we ship. Enforcing at construction
would mean private fields — a breaking change to a frozen surface, and not
proposed here.

**Say that limit out loud in the RFC and in the docs.** An assertion that proves
less than a reader assumes is how "verified" becomes a word that stops meaning
anything, and this project has spent a fortnight removing claims of exactly that
shape.

## Open questions

**Q-1 — assert over presets only, or add a public validator too?** A
`Typography::check_floor()`-shaped helper would let a consumer with custom tokens
check their own, and is additive so the covenant permits it. **Suggest presets
only for now**: nobody has asked for a validator, the owner's stated prior is to
keep snora simple, and a helper nobody calls is a third thing to keep true.

**Q-2 — what is the floor's authority?** `readability.md` states 12px as
snora's own rule. It is not a WCAG number — SC 1.4.4 is about *resize*, not a
minimum. **The assertion's message must not imply a standard it does not have.**
Cite the guide, not a specification.

## Acceptance criteria

1. A test asserts every role in **all four built-in presets** is ≥ 12.0.
2. **A perturbation demo:** drop one preset role below 12, watch it fail naming
   the preset and the role, restore.
3. The test's failure message cites `readability.md`, not WCAG.
4. The docs state what the assertion covers — **built-in presets** — and that a
   custom `Tokens` is the application's own to check.
5. No preset value changes. No field is made private.

## Compatibility and security

**Compatibility.** A test and documentation. **Security.** None.
