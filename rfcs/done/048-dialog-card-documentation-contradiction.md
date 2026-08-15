# RFC 048 — The dialog card: contradictory documentation and an undiscoverable capability

**Status.** Implemented (v0.28.1)
**Tracks.** Documentation consistency. Answers a downstream report from the
**arama** team (2026-08-15) and corrects a contradiction snora has shipped
since at least v0.25.0.
**Touches.** `crates/snora-core/src/layout.rs`,
`crates/snora/src/overlay/dialog.rs`, `crates/snora/src/render.rs`,
`docs/src/guides/overlays.md`,
`docs/src/reference/overlay-interaction-semantics.md`,
`docs/src/reference/architecture.md`,
`docs/src/contributing/feature-gating-criteria.md`. **No code.**
**Release target.** 0.28.1 (patch — documentation only).

## Summary

A downstream team reported that `render_dialog` "is documented as producing
*the centered modal card*, but draws no card." The behaviour is **correct and
deliberate**, and the card they asked for **shipped in v0.27.0** via RFC-039.

The defect is real anyway, and it is the one they named:

> The doc comment and the behaviour disagreeing is the actual defect; either
> half can move.

This RFC moves the documentation half. It does three things:

1. **Corrects six sites** that promise a card the default path does not draw.
2. **Makes the card discoverable** from the page a consumer actually reads
   about dialogs — which currently states the opposite.
3. **Writes down the rule** whose absence produced both this report and
   apimokka's accessibility-discoverability report three releases ago.

**No code changes.** Both render paths behave correctly.

## Motivation

The report is at `.git-exclude/tmp/snora-dialog-overlay-card.md`; the review
result, with full verification, is at
`.git-exclude/reviewed/arama-dialog-overlay-card/review-result.md`.

arama is an image and video browser. Its dialogs open over a gallery of
thumbnails, so dialog content lands on dark, saturated, arbitrary pixels. They
observed the same dialog reading cleanly over empty backdrop and being
illegible over thumbnails — the difference being *where the application's own
content sits*, which the dialog does not control.

They then read `crates/snora/src/overlay/dialog.rs`, whose first line says:

```rust,ignore
//! Dialog — the centered modal card.
```

and whose body centres content and does nothing else.

Their own framing of why this was worth reporting is the part worth keeping:

> the failure is **content-dependent**, so it survives testing on any
> application whose background is plain. It is invisible until someone puts a
> dialog over a photo grid.

That is why six releases shipped over it.

## The premise correction, which lands against snora

arama writes: *"the doc comment already promises the behaviour, so consumers
reasonably do not check."*

At **v0.25.0** — the version they reported against — `docs/src/guides/overlays.md`
already said, at line 43, eleven lines below the heading gloss they quote at
line 32:

> `Dialog` does not own the card chrome — you decide whether the dialog
> content is a plain `container`, a styled card with a border, an entire
> form. snora is a positioner, not a styler.

Verified with `git show 0.25.0:docs/src/guides/overlays.md`, not inferred.

So the behaviour matched the documented contract. **This is not a
"consumer missed the docs" finding and must not be recorded as one.** The
accurate reading is worse for snora:

**snora's documentation contradicts itself, in the same file, eleven lines
apart.** Line 32 promises a card; line 43 denies it. Both shipped together
for at least four releases.

A document contradicting the code is a defect a reader can resolve by reading
the code. A document contradicting *itself* leaves no way to tell which half
binds — which is precisely why arama went to the source and filed against the
module comment. Their behaviour was correct; our page was not.

## Findings

Severity: **(M)** must fix this release, **(S)** should fix.

### F-1 (M) — Six sites promise a card the default path does not draw

All present at v0.28.0:

| # | Site | Text | Audience |
|---|---|---|---|
| 1 | `crates/snora-core/src/layout.rs:114` | `/// A centered modal card.` | **Public rustdoc, docs.rs** |
| 2 | `crates/snora/src/overlay/dialog.rs:1` | `//! Dialog — the centered modal card.` | Private module, but ships in crate source — **the site arama read** |
| 3 | `crates/snora/src/render.rs:16` | `5. dialog — centered card` | Contributor |
| 4 | `docs/src/guides/overlays.md:32` | "A centered modal card." | **Consumer** |
| 5 | `docs/src/guides/overlays.md:133` | `5. dialog   centered card` | **Consumer** |
| 6 | `docs/src/reference/overlay-interaction-semantics.md:20` | `5. dialog — centered card` | **Consumer** |

**Site 1 is the most serious and the one arama did not find.** It is the
`AppLayout::dialog` field's rustdoc on a published crate, it says "card"
without qualification, and unlike `overlays.md` there is no correcting
paragraph anywhere near it. A reader on docs.rs gets the promise and never
the retraction.

**The correct wording already exists in the codebase twice.** Both are models
to copy rather than text to invent:

- `crates/snora-core/src/overlay.rs:38` — *"The engine centers this in the
  window and paints the dim backdrop around it."* No card claim.
- `crates/snora/src/render.rs:63` — *"Today's literal, unmodified: opaque
  black at 40% alpha, **no card**."*

### F-2 (M) — The card is undiscoverable from the page about dialogs

`docs/src/guides/overlays.md` is where a consumer goes to learn about
dialogs. At v0.28.0 it still says **"snora is a positioner, not a styler"**
and does not mention `design::render`, RFC-039, or the card anywhere.

That sentence became **path-specific in v0.27.0** and the page was never
updated. The card is documented only in `docs/src/design/engine-surfaces.md`
and the 0.26→0.27 migration guide.

A consumer in exactly arama's position — dialog legibility problem, opens the
dialogs guide — is told snora will never style it. **arama did not fail to
find the card; the page they would look at still says it does not exist.**

This is the **second instance of the failure mode apimokka reported**: the
content exists, in a location the audience needing it does not read. That one
was fixed for accessibility in v0.27.1 by adding a consumer-facing page. The
*class* was never swept for. F-4 addresses that.

### F-3 (S) — The z-stack tables describe a blend of two paths, and get each wrong in opposite directions

Four tables document the layer order. Since v0.27.0 they describe neither
path accurately:

| Site | Card claim | Dim claim |
|---|---|---|
| `docs/src/guides/overlays.md:132–133` | `centered card` | `40 % black click sink` |
| `docs/src/reference/overlay-interaction-semantics.md:19–20` | `centered card` | `40%-dim click sink` |
| `docs/src/reference/architecture.md:135` | *(none — bare `dialog`)* | `40 % black click sink` |
| `crates/snora/src/render.rs:15–16` | `centered card` | `40%-dim mouse_area` |

The two claims fail in **opposite** directions, which is why neither reads as
obviously wrong:

- **"centered card"** — wrong on the default path, right on the `design` path.
- **"40 % black"** — right on the default path, wrong on the `design` path.

Be precise about the dim, because half of it is still true: RFC-039 left the
**alpha unchanged at 40%** and derived only the **colour** (`Color::WHITE` on
dark presets, `Color::BLACK` on light — `crates/snora/src/design/render.rs:39–48`).
So "40%" is correct on both paths; "black" is not.

`architecture.md` carries the dim claim but **not** a card claim — its layer 5
is a bare `dialog`. Do not "fix" a card claim there that does not exist.

### F-4 (M) — The rule that was never written down

Two downstream reports, three releases apart, are the same omission:

| Report | Capability shipped | Page that still denied it |
|---|---|---|
| apimokka (2026-08-04) | focus/AT limitation documented in `contributing/` | consumer-facing docs had no route to it |
| arama (2026-08-15) | dialog card, RFC-039, v0.27.0 | `guides/overlays.md` — "not a styler" |

RFC-039 correctly documented the card in `docs/src/design/`. What neither RFC
did was **update the default-path page that tells consumers the capability
does not exist.** Adding the new page is not the whole job; retracting the old
denial is.

State it as a rule in `docs/src/contributing/feature-gating-criteria.md` —
the document that owns the gating concept, and the one a contributor reads
while deciding to gate something:

> **When a `design`-gated capability lands, every default-path page that
> states the capability is absent is part of the change scope.** Documenting
> the new behaviour in `docs/src/design/` is necessary and not sufficient: a
> consumer who never reads `design/` is left with the old denial, which now
> reads as a statement that the capability will not exist.

### F-5 (M) — `snora-dialog-card` never identifies the card, on either path

Found while verifying this report, and it is worse than a naming complaint.
It is arama's defect class reintroduced one release ago, by me.

RFC-047 attached `snora-dialog-card`
(`crates/snora/src/identifiers.rs:55`) in `overlay/dialog.rs` — on the
`center(...)` wrapper, on **both** paths:

```rust,ignore
None       => center(dialog.content).id(DIALOG_CARD),
Some(card) => center(container(dialog.content).padding(..).style(..))
                  .id(DIALOG_CARD),   // <- id is on the wrapper, not the card
```

In iced 0.14, `center(content)` is `container(content).center(Length::Fill)`
(`iced_widget-0.14.2/src/helpers.rs:259`) — a **full-window** container.
Verified by reading it, not assumed. Therefore:

- **Default path:** the identifier is on a full-window transparent container.
  No card exists.
- **`design` path:** the identifier is *still* on the full-window centring
  container. The actual card — the inner `container(...)` carrying the
  padding, fill, border and radius — **has no identifier at all.**

So the name never denotes the card, and a downstream test resolving
`snora-dialog-card` gets **window-sized bounds** rather than the card's. For
the stated use case — screenshot-diffing and driving a GUI from outside — that
is a wrong answer, not an imprecise label.

snora already documents the discrepancy while preserving the misleading name.
`docs/src/reference/rendered-surface-identifiers.md` describes it as *"The
dialog's centered container"*; `identifiers.rs:53` says the same. **The prose
is accurate and the name contradicts it** — arama's defect, one layer down,
in the surface RFC-047 introduced specifically so downstream teams could rely
on it.

Correcting this is a **minor** bump under the policy shipped in the same
release, so it cannot ride this patch. Tracked as **RFC-049**; see Q-1.

## Non-goals

- **No code change.** Both paths behave correctly. Neither the default
  centring nor the `design` card is being altered.
- **No card on the default path.** That would break the v0.25 rendering
  guarantee and RFC-037's gating invariant.
- **No renaming of `snora-dialog-card` in this release** — it is a minor-bump
  change by policy, and this is a patch. RFC-049 carries it (Q-1).
- **No disclaimer campaign.** Six sites, four tables, one guide section, one
  rule. If a seventh caveat is being added, the intent has been overshot.
- **Not a re-litigation of "positioner, not a styler."** That stance is
  correct for the default path and stays. It is being scoped, not withdrawn.

## Open questions

**Q-1 — Does `snora-dialog-card` get renamed?**
**Resolved: yes.** An earlier revision of this RFC recommended *no*, on the
grounds that churning a compatibility surface teaches downstream teams the
guarantee is soft. The owner's ruling supersedes it:

> you can rename if much better name exists. `CHANGELOG.md` or handoffs are
> available for help to app dev teams.

That is the right call, and F-5's verified placement finding — the identifier
is on the full-window wrapper and never on the card — makes it the only
defensible one. A compatibility guarantee is a promise not to move a name
*gratuitously*; it is not a reason to keep a name that returns the wrong
element. Honouring a wrong name is not stability, it is preserving a defect
with extra steps.

The cost is also near zero **right now** and only grows: 0.28.0 shipped
2026-08-04, and both known consumers (apimokka on 0.25.2, arama on 0.25.0)
are behind it. No known consumer asserts on these identifiers yet. That
window closes on their next upgrade.

Design is in **RFC-049** (minor, 0.29.0). Not in this patch.

**Q-2 — Does the module doc on a private module matter?**
`crates/snora/src/overlay/dialog.rs` is `pub(crate)`, so site 2 never reaches
docs.rs rendered output. It ships in crate source, and it is the site arama
actually read. Recommendation: **fix it** — the cost is one line and the
evidence that someone read it is in hand.

## Acceptance criteria

1. All six F-1 sites describe what each path renders. Site 1
   (`snora-core/src/layout.rs:114`) is corrected first and does not depend on
   the others.
2. `docs/src/guides/overlays.md` states that a token-styled card is available
   via `snora::design::render` as of v0.27.0, links
   `docs/src/design/engine-surfaces.md`, and scopes "positioner, not a
   styler" to the default path.
3. The four z-stack tables in F-3 distinguish default-path from `design`-path
   behaviour for **both** the card and the dim, with the dim's alpha stated
   correctly as unchanged.
4. `feature-gating-criteria.md` carries the F-4 rule.
5. `mdbook build docs` and `mdbook test docs` pass.
6. `git diff --stat -- crates/*/src/**/*.rs` shows **doc-comment lines only**;
   `cargo test -p snora --test render_semantics` passes unmodified.

## Testing and verification

```bash
mdbook build docs && mdbook test docs
cargo test -p snora --test render_semantics    # MUST pass unmodified
cargo doc --workspace --no-deps
```

Plus the grep that produced F-1 and F-3, re-run to confirm each remaining hit
is intentional and qualified:

```bash
grep -rn 'centered card\|centered modal card\|centred modal card' \
  docs/src/ crates/*/src/ README.md
grep -rn '40 %\|40%' docs/src/ crates/*/src/ README.md
```

**Do not treat an empty grep as the acceptance signal.** The criterion is
that every surviving hit reads correctly for the path it describes — several
should survive, because "40%" is accurate on both paths.

## Compatibility and security

**Compatibility.** Documentation only. No API, no rendering, no gate rows. The
corrected text describes behaviour that has been stable since v0.27.0.

**Security.** No new data flow, dependency, or integration.

## Release implications

**0.28.1, patch.** No code, no API change, no MSRV change. A migration guide
is not required — snora's convention attaches those to minors, and there is
nothing for a consumer to migrate.

`CHANGELOG.md` gets a **Changed** entry under `[Unreleased]` naming the
contradiction rather than describing it as a wording tidy-up. arama is
credited; it is their finding.

## What to tell arama

Handled in the reply, not here, but the RFC records the two points that must
survive into it:

- The card they asked for **shipped in v0.27.0**. They are on 0.25.0. No API
  break, dim backdrop unchanged on the default path, opt-in per call site.
- Their premise correction runs **against snora, not against them** — the
  contract was documented in a page that also contradicted it. They are not to
  be told they missed the docs.
