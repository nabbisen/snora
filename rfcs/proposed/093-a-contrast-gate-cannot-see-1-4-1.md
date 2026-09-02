# RFC 093 — A contrast gate cannot see 1.4.1, and ours never could

**Status.** Proposed (2026-09-02).
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
