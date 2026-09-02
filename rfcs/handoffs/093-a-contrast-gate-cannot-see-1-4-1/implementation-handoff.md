# Developer Handoff — RFC-093 the channel register

**Governing RFC.** **RFC-093** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-02).
**Release target.** **0.43.0.**
**Touches.** `crates/snora/src/toast/`, `crates/snora-widgets/src/design/`,
`docs/src/guides/accessibility.md`.
**Implementation units.** Two. Both yours; the guide's wording is yours too this
time, because it must quote the register you build.

---

## Rulings

**Q-1 — register and document now. The non-colour cue is not in this RFC.**

And the deferral gets a firing condition, because this project has learned what
happens without one (RFC-087, F-39): **a team asking for the cue *after* the
documentation lands is genuine demand and reopens it.** Before that, silence is
not evidence — nobody can ask for a channel they have been told exists. That
distinction is not theoretical: RFC-078 counted apimokka's decline as evidence
against focus trapping, and their decline turned out to be an echo of our own
constraint (`design-decisions.md`, corrected 2026-09-02).

**Q-2 — both, with the code as the source of truth.** The register lives in code
where a test can reach it; `accessibility.md` quotes it. Same shape
`overlay-interaction-semantics.md` already uses for Law 8.

**Q-3 — it does not reach `Emphasis`, and I checked rather than assumed.**
Nothing consumes it: `Emphasis` is defined in `snora-design/src/variants.rs`,
re-exported through `lib.rs` and `snora/src/design.rs`, and tested only for
inequality. No widget or style function varies anything by it. Out of scope here.
*(That a published vocabulary type has no consumer at all is a separate finding
and not yours to act on.)*

## Unit 1 — the register and its test

Three surfaces, measured rather than inherited from the withdrawal (which named
two):

| Surface | Varies by | Channels today |
|---|---|---|
| `snora::toast` | `ToastIntent` (5), via `intent_colors` | colour only |
| `snora_widgets::design::notice` | `Tone` (6), `notice.rs:115-120` | colour only |
| `snora_widgets::design::progress` | `Tone`, via `snora-style` | colour only |

**Exhaustive over the variant enums**, RFC-063's pattern — the same one
`toast/contrast_tests.rs` already uses. A sixth `ToastIntent` or a seventh `Tone`
must fail to compile until it has an entry.

The assertion is that **nothing but colour differs between variants of the same
surface**. Concretely: for each pair of variants, the non-colour properties of
the produced style are equal.

**What this test is for, since it is easy to build the wrong thing.** It does not
prove 1.4.1 conformance — nothing here can, because the text is supplied by the
caller. It exists to **fail the day the code and the claim diverge**, in either
direction: someone adds a per-intent icon and does not update the documentation,
or removes one that the documentation promises.

## Unit 2 — `accessibility.md` states the division of labour

The guide currently implies the prefabs carry a non-colour channel. It must say
what is true:

> snora's prefab surfaces distinguish semantic variants **by colour alone**. A
> consumer relying on them for WCAG 1.4.1 must supply a non-colour channel —
> typically per-variant text, which most applications already do.

**Name the worked examples**, because they exist and they are better than
invented ones. Every adopting team that checked came back safe for the same
reason: each passes its own per-variant title and body, so the outcome is carried
by words whatever colour it arrives in. Describe the pattern; do not name the
teams in published documentation.

Do **not** restate the withdrawal itself — `CHANGELOG.md` and the 0.41.1 entry
already carry it, and the guide's job is the current state, not the history.

## Required evidence

**Demonstrate the test failing.** Add a per-intent icon or a textual prefix to
one variant in a scratch edit, confirm the register test refuses and names the
surface and the variant, restore, confirm green.

A test that has only ever been seen to pass is the defect this project has hit
five times. This one guards a claim we have already published wrong once.

## Acceptance criteria

1. The register exists, exhaustive over `ToastIntent` and `Tone`, failing to
   compile on a new variant with no entry.
2. The test asserts nothing but colour differs between variants, and is
   **demonstrated failing** on a deliberately added non-colour channel.
3. `accessibility.md` states the division of labour and stops implying the
   prefabs carry a non-colour channel.
4. CHANGELOG entry, or one line saying why not — **this one probably warrants
   one**, since it corrects what a consumer can rely on, but say which and why.
