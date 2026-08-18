# RFC 059 — Two more answers filed where consumers do not read

**Status.** Done — shipped in v0.34.0 (2026-08-18). Handoff:
[`handoffs/059-…`](../handoffs/059-answers-filed-where-consumers-do-not-read/implementation-handoff.md)
**Tracks.** Documentation. **Fourth and fifth instances** of the failure mode
RFC-045, RFC-048 and RFC-057 each fixed once.
**Touches.** `docs/src/design/iced-style-bridge.md`,
`docs/src/contributing/semantic-accessibility.md`,
`docs/src/contributing/accessibility-checklist.md`,
`crates/snora-design/src/focus.rs` (doc comment),
`docs/src/design/stability.md` (new), `docs/src/SUMMARY.md`,
`docs/src/design/{feature-flags,tokens}.md`,
`docs/src/guides/accessibility.md`,
`docs/src/contributing/feature-gating-criteria.md`,
`docs/src/contributing/release-process.md`, `CHANGELOG.md`. **No code.**
**Release target.** 0.34.0, alongside RFC-058 (documentation only in itself).

## Summary

tekstide evaluated snora end to end and declined to adopt it. Two of their
findings are not about missing capability — they are about **answers snora
already has, in places a consumer does not reach.**

1. A review label instructs people that focus rings **cannot** be rendered on
   iced 0.14. They render one today, and the label is what stopped anyone
   asking.
2. `snora-design` has a **contractual stability guarantee** naming the exact
   functions tekstide wanted. They read the whole repository and missed it. They
   named that stability statement as *"changing our calculus more than any
   feature would"* — and declined partly for want of it.

~~The second cost snora an adoption decision.~~

**Overstated — corrected by tekstide, 2026-08-18.** Their decision *"did not
turn on it."* Churn was one of four reasons, three were untouched by the
covenant, and one has since grown *stronger* — they have built and shipped their
own contrast module, so adopting would now mean deleting working, reviewed code
in order to take a dependency.

The finding stands and the fix was right: an answer we already had was filed
where a consumer could not reach it, and they asked us a question we had
answered. What does not stand is the causal claim. They told us plainly that one
of four legs was wrong and it was not the load-bearing one — **keep the defect,
drop the drama.**

## Instance 1 — `BLOCKED` is a control that stops inquiry

`semantic-accessibility.md:131` instructs recording focus-ring absence as:

> `BLOCKED (iced 0.14 — no focus variant in button::Status)`

and explicitly **"Do not file it as a bug."**

That is true for a widget that lets **iced** own focus. It is not a property of
iced 0.14. tekstide render a visible focus indicator today: their shell owns
focus as application state and their `container` style closure takes the boolean
directly —

```rust,ignore
move |_base_theme: &iced::Theme| container::Style {
    border: Border {
        color: if focused { theme.border_focused() } else { theme.border_default() },
        width: if focused { 2.0 }                  else { 1.0 },
        ..
    },
    ..
}
```

Two channels, colour *and* width, because a colour-only indicator fails their
own rule. A `container` style closure is an arbitrary `Fn(&Theme) -> Style`, so
anything the application knows is available to it.

**The accurate constraint is narrower:** *iced cannot tell a style closure that
a widget **iced** owns is focused.*

And `FocusTokens` therefore has a **present-day audience** — any application
with keyboard navigation across regions already holds the state required.
Nothing tells them.

tekstide quoted RFC-057 back at us:

> *"a control telling people not to look is why this survived several design
> releases."* A `BLOCKED` label is that kind of control.

That is the second such control found in two releases, after RFC-057's
line-height checklist item. **Two instances make it a class, not an accident:**
a review instruction that closes a question is more durable than a stale
sentence, because following the process reproduces the error.

## Instance 2 — the covenant that answers the stability question is invisible

tekstide asked whether `snora-design`'s stability differs from the framework's,
and said a statement scoped to it would change their calculus more than any
feature.

**It exists.** RFC-036's additive-only covenant, in
`contributing/api-governance.md`, freezes the **token surface** by name:

- `Tokens` and its four constructors
- `Palette` and its 18 role fields
- `Color`, `Spacing`, `Typography`/`TextRole`, `Radius`, `FocusTokens`
- `Tone`, `Emphasis`, `Size`, `Density`
- **individually named:** `relative_luminance`, `contrast_ratio`,
  `composite_over`

Removing, renaming or retyping any of it is **forbidden** without reopening
gates D-3 and D-4 *in the same change*, with an explicit prohibition on
"proceeding and rationalising afterward". The design **primitives** are
deliberately excluded and run on a more permissive lifecycle.

The three contrast functions tekstide wanted are the most strongly protected
surface snora has — and they concluded the version number could not tell them
so.

Outside `contributing/`, the covenant is mentioned **twice in passing** — once
in a migration guide, once inside `design/engine-surfaces.md`. There is no page
a prospective adopter would find it on.

### One thing the answer must not overclaim

The covenant constrains *what* may change, not cargo's version arithmetic.
`0.33` → `0.34` remains incompatible to cargo under pre-1.0 SemVer regardless.
tekstide's real question is whether the token surface **churns**, and the honest
answer is that it is contractually forbidden to — not that upgrades are
mechanically painless.

## Scope

1. **Re-scope the `BLOCKED` label** in `semantic-accessibility.md`: name the
   real constraint (iced cannot report focus for widgets it owns), and stop
   instructing reviewers not to file. Replace "do not file it" with the
   condition under which filing *is* right.
2. **Document `FocusTokens`' present-day audience** — applications that own
   focus state — in its doc comment and wherever the design docs describe it as
   awaiting future iced versions.
3. **A consumer-facing statement of the token-surface guarantee**, under
   **Snora Design**, saying what is frozen, what is not, and what changing it
   would cost. Link `api-governance.md`; do not duplicate it.
4. `guides/accessibility.md` gains a line pointing at (1)/(2), consistent with
   how RFC-057 linked readability.

## The pattern, and whether to act on it

This is the **fourth and fifth** instance:

| | Answer existed in | Found by |
|---|---|---|
| RFC-045 | `contributing/semantic-accessibility.md` | apimokka |
| RFC-048 | `docs/src/design/` | arama |
| RFC-057 | `contributing/accessibility-checklist.md` | the owner |
| **059 (1)** | a review label that closed the question | tekstide |
| **059 (2)** | `contributing/api-governance.md` | tekstide |

RFC-048 wrote a rule for capability arrivals and RFC-056 widened it to
removals. Neither covers **this**: a consumer-relevant answer living only in a
contributor document.

**Q-1 asks whether to write that rule.** The candidate: *when a governance or
policy decision answers a question a consumer would ask, it needs a
consumer-facing statement, not only a contributor record.* Five instances is
enough evidence that the class is real; whether a fifth rule helps or whether
the rules themselves are now the undiscoverable thing is a judgement.

## Non-goals

- **No code.** Doc comments only.
- **No change to the covenant itself** — this documents it, it does not amend
  it.
- **No new focus capability.** RFC-045's position stands: snora will integrate
  an accessibility tree when iced exposes one, and will not build a parallel
  abstraction. Documenting that applications can style their own focus state is
  not a retreat from that.
- **No `FocusTokens` application in snora's own widgets.** snora's primitives
  let iced own focus; that is unchanged.

## Open questions — all three now answered

**Q-1 — write a fifth documentation rule, or stop adding rules? Neither: widen
the rule that already exists.** `contributing/feature-gating-criteria.md`
§ *"Documentation scope when a capability arrives or leaves"* is that rule, and
RFC-056 already widened it once from arrivals to removals. A fifth standalone
rule would itself land in `contributing/` and reproduce the defect it describes.

Widen it a second time to cover **standing answers** — a governance guarantee,
or the true scope of a constraint — and add a pointer to it from
`release-process.md`, so following the release process reaches the rule. Five
misses while the rule sat in a page titled *feature-gating criteria* is evidence
the rule's problem is its address, not its absence.

**Q-2 — where does the stability statement live? A new
`docs/src/design/stability.md`,** registered under **Snora Design** in
`SUMMARY.md`, linked from `design/feature-flags.md` and `design/tokens.md`.
`design/overview.md` has the audience but is already long, and a prospective
adopter needs a page that can be linked at them.

**Q-3 — does the `BLOCKED` label appear in any primitive's recorded checklist?
No.** Grepped: three `BLOCKED` sites and no per-primitive record, so there is no
inherited-claim cleanup. But the grep found something the scope above had
missed —

### The most important instance was not in the original scope

`docs/src/design/iced-style-bridge.md:76–83` — under **Snora Design**, i.e. the
**consumer-facing** half of the book — carries the same over-scoped claim *and*
the "will be wired when iced exposes focus state" framing. That is the copy a
consumer reads, and it is the one that tells an application which already owns
focus state to wait for a future iced.

`contributing/accessibility-checklist.md:26` repeats it a third time.
`accessibility-checklist.md:192`, by contrast, is a correct generic definition
of the `BLOCKED` severity category and stays as it is.

Four sites, one claim, three of them needing the same narrowing. Fixing only the
contributor copy would have been this same defect one more time.

## Acceptance criteria

1. **All three over-scoped sites** — `design/iced-style-bridge.md`,
   `contributing/semantic-accessibility.md`,
   `contributing/accessibility-checklist.md:26` — state the narrow constraint,
   and `semantic-accessibility.md` no longer instructs reviewers not to file.
   `accessibility-checklist.md:192` is unchanged.
2. `FocusTokens` is documented as usable today by focus-owning applications.
3. A consumer-facing page or section states the token-surface guarantee,
   linking `api-governance.md` without duplicating it, and **does not claim
   version-upgrade compatibility** the covenant does not provide.
4. Q-3 re-confirmed by a fresh grep: no per-primitive `BLOCKED` record exists.
5. The documentation-scope rule in `feature-gating-criteria.md` covers standing
   answers, its instance table gains both tekstide rows, and
   `release-process.md` points at it.
6. `git diff --stat -- 'crates/**/*.rs'` shows doc-comment lines only.
7. `mdbook build` / `mdbook test` pass; `render_semantics` unmodified.

## Compatibility and security

**Compatibility.** Documentation only. No API, no rendering, no gate rows.

**Security.** None.
