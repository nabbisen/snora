# Handoff — 0.62 audit: is every keyboard and overlay claim in the book guarded by a test?

**Ruled.** The owner, 2026-11-12 ("1. Yes."), on the architect's proposal while scoping 0.62.
**What it produces.** A table and findings, in two review requests. **No code and no book change in
this work.** The architect corrects the book. Tests for unguarded claims are a follow-up the
review rules on, and what it rules becomes 0.62.
**Branch.** None needed for the audit. A probe that has to run code goes in a scratch test on a
throwaway branch, and is described in the request rather than landed.

## Why

**Three times a sentence in the book said snora did something, and nothing tested it, and it was
false:**
- **0.54:** the book said snora's buttons took Enter and Space. They were iced buttons, which take
  no keyboard input (corrected `0c6e59e`, RFC-114).
- **0.35 to 0.61:** F6 zone navigation, as documented, moved no focus (RFC-126).
- **0.55 to 0.61:** "set initial focus with `widget::Id`" could not be done with a snora control
  (RFC-127).

**Two more claims were broader than their evidence:**
- 0.55.0's "nothing changes for an app that doesn't route Tab" (RFC-121);
- 0.59's "a page covered by an open menu gets no pointer", which held for the header dropdown
  only (RFC-125).

Each was found by a reader or by an unrelated piece of work. **This audit looks for the next
one deliberately.**

## Scope

**Pages.**

| Page | Lines |
|---|---|
| `docs/src/reference/overlay-interaction-semantics.md` (Laws 1–8) | 397 |
| `docs/src/guides/keyboard.md` | 130 |
| `docs/src/guides/menus.md` | 197 |
| `docs/src/guides/overlays.md` | 251 |
| `docs/src/guides/accessibility.md` | 277 |
| `docs/src/design/buttons.md` (its keyboard and accessibility notes) | 142 |
| `docs/src/design/notices.md` | 58 |
| `docs/src/contributing/semantic-accessibility.md` (the ownership table) | 235 |

**Rustdoc of the public keyboard and overlay surface:**
- `snora::keyboard` (`traverse`, `cycle_zones`, `focus_zone`, `Shortcuts`, `Escape`);
- `snora::control`;
- `snora::design::control`;
- `AppLayout`'s `on_close_modals`, `on_close_menus`, `escape`, `shortcuts`.

**Not in scope:**
- migration guides and released changelog entries (records of their time);
- history notes and *Corrected* lines;
- design rationale;
- "future wanted";
- RFCs.

## What counts as a claim

**A sentence a reader could act on that says what snora does:** a behaviour, a guarantee, a rule,
or a limit. For example:
- *"Escape publishes the overlay's escape message if one is set"*;
- *"a disabled control is not a focus target"*;
- *"a wheel over the dim does nothing"*;
- *"the ring clears 3:1 in every preset"*;
- *"snora installs no subscription"*.

One sentence can hold several claims. Split them.

## The verdict for each claim

| Verdict | Meaning | What to give |
|---|---|---|
| **Guarded** | a test exists whose assertion is this claim, and it would fail if the claim were false | the test's file and name. **For one claim in five, chosen by you, show it fail by a breakage** (named `cp` backups, `sha256sum -c`; never `git checkout` or `restore`). The rest by reading, said so |
| **Guarded in part** | a test covers some of the claim, for example one renderer, one path, one overlay | what is covered and what is not |
| **Unguarded, true** | no test, and a probe shows it holds | the probe, briefly, and a proposed test |
| **Unguarded, false** | no test, and a probe shows it does not hold | **stop on that claim and report it at once** in the request's first lines. It is a defect, and the architect decides whether it is fixed or the sentence is withdrawn |
| **Not testable here** | about iced, a platform (macOS: RFC-116 L-8), or a real window | why. If a real-window check would settle it, say so |

**A claim guarded by a test that cannot fail is not guarded.** Four of this project's handoffs
specified one, so look for the pattern:
- an assertion that passes whether or not the behaviour holds;
- a test that reads the frame after an operation that repairs the thing it is checking
  (RFC-123);
- a second mechanism that produces the same result (RFC-125).

## Two review requests, in this order

1. **`.git-exclude/review-request/audit-0-62-a/`:**
   - `overlay-interaction-semantics.md`, which is normative and the largest;
   - `keyboard.md`;
   - the rustdoc listed above.
2. **`.git-exclude/review-request/audit-0-62-b/`:** the five guides and the ownership table.

**Each has a `claims.md`:** one row per claim, with the page and line, the claim quoted short, the
verdict, and the evidence. The README carries:
- a summary of counts by verdict;
- every **false** claim first;
- the proposed tests for unguarded claims, grouped by the suite they would join.

## Acceptance criteria

1. Every normative sentence of the scoped pages is in a `claims.md` row, or listed as not a
   claim with a one-word reason (history, rationale, future, example).
2. Every "Guarded" names its test. One in five is shown failing by a breakage.
3. Every "Unguarded" has a probe result.
4. No product code or book page changed.
