# RFC 093 — A contrast gate cannot see 1.4.1, and ours never could

**Status.** Done — shipped in v0.43.0 (2026-09-02).
[Handoff](../handoffs/093-a-contrast-gate-cannot-see-1-4-1/implementation-handoff.md).
**Tracks.** Accessibility / test reach. **Severity: High.**
**Found by** tekstide and orbok independently, replying to the 0.42.0 letter.
**Touches.** `crates/snora/src/toast/`, `crates/snora-widgets/src/design/`,
`docs/src/guides/accessibility.md`.
**Release target.** 0.43.0.

## The finding

**Contrast is a property of a colour pair. WCAG 1.4.1 (Use of Colour) is a
property of everything else on the surface.** No enumeration of colour pairs
reaches it, however exhaustive, and making ours more exhaustive would not have
helped.

We published a false 1.4.1 claim — *"toast intents and notice tones are
distinguishable by more than colour alone"* — and withdrew it in 0.41.1. At the
time we had **seven** contrast RFCs: six guarding the token layer, one the widget
layer, with exhaustive destructuring so no role could be added unmeasured. Not
one of them could have caught it, and no amount of strengthening them would.

**tekstide proved this from outside, on their own code.** They built the same
class of gate — an exhaustive destructure over every theme colour pair — and
reported:

> "It measures 1.4.3. It cannot see 1.4.1 at all — whether a pair is the *only*
> channel is a claim about what else is rendered, which no colour-pair
> enumeration reaches. **Ours would have passed your toasts at every intent.**"

Their own surfaces were clean **by accident**: their theme exposes no semantic
colour roles — no `warning`, `danger`, `success` — so a surface has nothing to
encode status in but text. As they put it, that protection *"ends with the first
semantic role, and nothing would fail: the new pair measures fine and the old
surfaces have not changed."*

We have had intent families all along, which is plausibly why the claim survived
here and not there.

**orbok found the same shape from the other end.** All three claims we have
withdrawn to them — `text_muted` (0.34.0), the dialog-card border (0.39.0), 1.4.1
(0.41.1) — were **documentation claims about rendering behaviour where the code
was right and only the prose was wrong.** Their remedy, offered without
expectation: a comment asserting a behaviour should be paired with a test that
checks it.

## Scope, measured rather than inherited from the withdrawal

The withdrawal named toasts and notices. **The property is broader.** Every
semantic-variant surface in snora carries the distinction in colour alone:

| Surface | Varies by | Non-colour channel |
|---|---|---|
| `snora::toast` | `ToastIntent` (5) via `intent_colors` | **none** |
| `snora_widgets::design::notice` | `Tone` (6) — `notice.rs:115-120` maps each to one colour | **none** |
| `snora_widgets::design::progress` | `Tone` via `snora-style` | **none** |

Checked, not assumed: every `Tone::` arm in `snora-style` resolves to a colour,
and `button`/`chip` do not take `Tone` at all.

**This is not necessarily a defect.** snora's prefabs contribute colour; the
consumer supplies the words. Every adopting team that checked came back safe for
exactly that reason — orbok's `UserNotice` variants each carry their own title
and body, knotra passes distinct strings in two languages, aaai's toasts carry
per-call text. That is a legitimate division of labour.

**What is a defect is that we never said so**, and asserted the opposite.

## Proposal — a channel register, asserted by test

For each surface that varies by semantic variant, record which channels carry the
distinction, and assert the record against the code:

- the register states, per surface, that the widget contributes **colour only**;
- a test enumerates the variants exhaustively (RFC-063's pattern, already used by
  `toast/contrast_tests.rs`) and asserts nothing but colour differs between them;
- `accessibility.md` states the division of labour: **snora's prefab surfaces
  distinguish semantic variants by colour alone; a consumer relying on them for
  1.4.1 must supply a non-colour channel themselves** — with the adopting teams'
  own patterns named as the worked examples they already are.

The test's job is not to prove 1.4.1 conformance, which it cannot. Its job is to
**fail the day someone adds an icon or a prefix and does not update the claim** —
and equally, to fail the day someone removes one. It pins the documented state to
the real one, which is the only part a test can reach.

This is RFC-092's thesis in the place it costs the most. RFC-092 mechanized claims
about *what changed*; this is a claim about *what the code does*, and it needs a
different mechanism.

## Non-goals

- **Not adding a non-colour cue.** That is an appearance change on the default
  path for every consumer, and it is Q-1, not a foregone conclusion.
- **Not claiming 1.4.1 conformance for snora.** A framework that renders text
  supplied by its caller cannot conform on the caller's behalf. The register says
  what we contribute, not what the application achieves.
- **Not a general "every documented behaviour gets a test" rule.** orbok's
  version generalises that far; adopting it wholesale would be unbounded. This
  RFC is scoped to claims about *semantic variants*, where we have a withdrawal
  on the record.

## Open questions

**Q-1 — do we add a non-colour cue to toasts and notices?** An icon per intent,
or a textual prefix. It would let a consumer rely on the prefab rather than
supply the channel themselves. It is also an appearance change on the default
path — the fourth this quarter by aaai's count — and every adopting team is
already safe without it. **Suggest: register and document first, decide the cue
separately**, so the honest statement ships now and the appearance change is not
smuggled in behind it.

**Q-2 — does the register belong in code or in the guide?** Code makes it
testable; the guide makes it findable. **Suggest both**, with the code as the
source of truth and the guide quoting it, which is the shape
`overlay-interaction-semantics.md` already uses for Law 8.

**Q-3 — does this reach `Emphasis` as well as `Tone`?** `Emphasis::Solid/Soft/
Outline` also varies appearance and is not a status channel, so probably not —
but it was not checked, and saying so is cheaper than assuming.

## Acceptance criteria

1. The register exists, exhaustive over `ToastIntent` and `Tone`, and fails to
   compile if a variant is added without an entry.
2. A test asserts the register matches the code, **demonstrated failing** — add a
   per-intent icon in a scratch edit, confirm the test refuses, restore.
3. `accessibility.md` states the division of labour and stops implying the
   prefabs carry a non-colour channel.
4. Whatever Q-1 rules, the RFC's own text records which way and why.
5. CHANGELOG entry, or one line saying why not.

---

## Q-1 ruled, 2026-09-12 — no cue, and the condition that was waiting on one is withdrawn

Criterion 4 requires this RFC to record which way Q-1 went and why. It went
**no**, and the more useful half of the answer is why the deferral that carried
it was never going to fire.

### The condition could not see the evidence that mattered

Q-1 was deferred on *"a consumer asks for it."* All six teams were asked and
none did. But **orbok had already built one** — per-variant `title()`/`body()`
in their own code, dating to that file's first commit (`67c4378`, 2026-06-17),
three months before we withdrew the 1.4.1 claim, so their solution is
independent of anything we published rather than an echo of it.

**A consumer who built the channel themselves is stronger evidence of need than
one who asks for it, and the condition counted it as silence.** That is this
project's recurring defect in its native habitat: a rule with nothing to fire
it, where the case that should fire it is the case it is blind to. Two more of
the six (apimokka, tekstide) asked to be recorded as **absent, not "no"** —
apimokka compiles without `snora-widgets`, so the prefabs are unavailable to
them rather than declined; tekstide is not a consumer. A condition polling six
teams, four of whom said their answer was not demand evidence, was measuring
its own reachability.

### Ruling anyway, rather than restating the condition and waiting again

The condition was wrong, so the honest move is to decide on the merits now —
not to replace it with a better condition and defer a third time. Renewing a
deferral because its condition was poorly drawn is how the `check-*` scripts'
manual-to-gate deferral survived three renewals with its condition true the
whole time (RFC-087).

**On the merits: no cue.** The reasoning is not "nobody asked":

- **The gap Q-1 would close is already closed, by division of labour rather
  than by API.** `accessibility.md` states that snora's prefabs distinguish
  variants by colour alone and that a consumer relying on them for 1.4.1 must
  supply the non-colour channel — and that statement is asserted by the channel
  register's test, exhaustive over each variant enum, so it fails the day the
  claim and the code diverge in either direction. The consumer is told, and the
  telling is checked.
- **The beneficiary of a built-in cue is a future adopter who has not arrived.**
  Every present team either supplies its own channel or cannot use the prefabs.
  Shipping an opt-in API that every current consumer would ignore is building
  for nobody — and the pre-ruled shape (**opt-in, not default-on**, which
  stands) guarantees nobody is moved onto it by default either.
- **The honest form of the cue is not obvious, and that is a reason to wait for
  a real requirement rather than guess at one.** A textual prefix is not ours to
  localise — we do not know the application's language. An icon per intent
  drags an icon source onto a default path that is deliberately free of one.
  Shape or border differentiation avoids both but needs evidence it is
  perceivable, which we do not have. **Three plausible designs, no requirement
  to choose between them**, is the RFC-078 shape: a design question with no
  forcing case.

### What replaces the deferral

Nothing, as an open question. **Q-1 is closed, not deferred**, and its register
row is discharged.

The residual risk is not "consumers lack a non-colour channel" — it is **"a
future adopter assumes the prefab carries one."** That is a documentation
failure, not an API gap, and it has a different tell: somebody reporting that
they expected the prefab to carry the channel. If that ever arrives it reopens
this on its own terms, with a real requirement attached and a named surface
that failed to communicate — which is exactly what the three candidate designs
above are missing today.

**Reversibility is cheap and is part of the ruling.** Adding a cue later is
additive under RFC-036; nothing here forecloses it. Shipping one now, and
discovering the wrong form was chosen, would not be — removal is the priced
direction.
