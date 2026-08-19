# RFC 069 — Every Rust example in the book is `ignore`, and the policy blames the wrong cause

**Status.** Done — shipped in v0.38.0 (2026-08-19).
[Handoff](../handoffs/069-book-examples-cannot-be-compiled/implementation-handoff.md).
**Tracks.** Documentation integrity.
**Found by** the architect, while writing RFC-068's handoff.
**Touches.** `docs/src/contributing/documentation-test-policy.md`,
`docs/src/guides/readability.md`, `docs/book.toml` (possibly), the book's
`rust,ignore` fences.
**Release target.** 0.38.0 — documentation only.

## Correction (2026-08-19)

**This RFC originally said 110.** The correct count is **111** — one fence in
`docs/src/guides/accessibility.md` is indented two spaces and was missed by the
line-anchored grep the audit used. Caught by the dev team, who re-derived the
number instead of copying it. Non-migration figures corrected with it:
**91** candidates, **24** referencing `self`.

## Summary

**111 of 111 Rust code fences in `docs/src` are `rust,ignore`.** There are no
bare `rust` fences and no `rust,no_run` fences in the book at all. No book
example has ever reached a rung above the weakest one.

`documentation-test-policy.md` describes this as a per-snippet judgement —
authors reach for "the highest rung it can reach", and `ignore` is for
"full-app-shaped snippets, partial `impl` blocks, event-loop shapes". **That is
not the binding constraint.** `docs/book.toml` has no `[rust]` section, and CI
runs bare `mdbook test docs` with no `-L`, so **no snippet that imports snora
can compile at all** — however small, however complete. The rung is not chosen;
it is the only one available.

Meanwhile a compiled corpus already exists — **20 example crates, workspace
members, built by CI** — and the book draws from it **zero** times. mdbook's
`{{#include}}` appears nowhere in `docs/src`.

## What is *not* wrong

**RFC-064's rule is correctly scoped and must not be "fixed".** The sentence
"every fence left at `ignore` must carry a one-line reason" sits under
`## Crate doctests` → `### The three-rung ladder`, and its grep is scoped to
`crates/ --include="*.rs"`. Section and grep agree. The crate side is audited,
counted, and governed.

The gap is that **the book side has no equivalent rule at all** — and, as §"The
shape of the fix" argues, it should not get the same one.

## The four findings

**1. The reason is structural, not per-snippet.** No library path is available
to `mdbook test`, so `use snora::…` cannot resolve. A five-line snippet
constructing a token value — pure, no event loop, no renderer — is still forced
to `ignore`. The policy tells authors to climb a ladder whose upper rungs are
not attached to anything.

**2. The page contradicts itself on `no_run`.** Its classification table calls
`rust,no_run` *"highlighted but not compiled"*; its ladder table, forty lines
later, correctly calls it *"compiles, does not execute"* and *"the important
middle rung — it costs no runtime and still catches API drift."* The first
statement is wrong, and it is the one an author reads first. It removes the
incentive to use the only rung that catches drift.

**3. The policy already names a validated path, and nothing follows it.** From
the same page:

> 1. A small `examples/` crate that compiles in the workspace check; or
> 2. A `rust,ignore` block in docs **linked to the relevant example crate**.

**111 `ignore` fences; 2 references to `/examples/` anywhere in book prose.**
The rule was written, and there is nothing that fires when it is skipped. This
is the project's recurring defect shape, and this instance was already written
down.

**4. One live false claim.** `readability.md:72` reads *"Compile-checked against
the pinned iced 0.14:"* directly above a `rust,ignore` fence. Nothing compiles
it. This is the only instance found.

## Evidence of harm: none yet, and that is stated honestly

Every `snora*::` path in all 111 fences was extracted and checked against the
crate source. **Zero stale symbols.** Four flagged names — `ToastIntent::Error`,
`ToastIntent::Info`, `ToastIntent::Success`, `ToastLifetime::Persistent` — all
exist; they were regex false positives.

**The limits of that check matter more than its result.** It is symbol-level. It
catches a name that vanished. It does **not** catch a changed signature, a
changed return type, a reordered argument, a moved method, or a documented path
that is no longer reachable — and it does not verify the snippet compiles. It is
weak evidence of "not yet rotted", not evidence of correctness.

So this RFC is **not urgent**, and should not be sold as a fire. Six teams copy
these examples; the argument is that we have no mechanism, not that we have been
burned.

## The shape of the fix

**Not per-fence reasons.** Extending RFC-064's one-line-reason rule to the book
would produce **111 copies of one sentence** — and RFC-064's own rationale was
that identical justifications drift apart from each other without anyone
noticing. At 111 copies that failure mode is the design, not a risk.

The structural reason is one fact about the build. It belongs **once**, on the
policy page, not 111 times.

## Open questions

**Q-1 — which mechanism?**

- **(a) `{{#include}}` from `examples/`.** Draws prose from code CI already
  compiles. Needs anchor comments (`ANCHOR:` / `ANCHOR_END:`) in the example
  crates — **none exist today** — and only fits snippets that correspond to real
  example code.
- **(b) Give the book a library path** so `mdbook test -L …` can compile
  snippets in place. Reaches every fence, not just those with an example
  counterpart; costs a build-ordering dependency in CI (the book job would need
  the workspace built first).
- **(c) Neither** — fix findings 1, 2 and 4, and make finding 3's link
  requirement real.

**Ruled after measurement (see the handoff §3): (a), with purpose-written
source.** (b) is rejected — of the 91 non-migration fences, 24 reference `self`
and 34 reference a binding they never declare, so roughly half cannot compile
whatever the library path; and the docs CI job performs no cargo build today, so
a library path adds an iced-inclusive workspace build for partial coverage of
the half least likely to rot. (a) is right in mechanism and wrong in source: the
20 existing examples are full applications, and a 7-line median fragment does
not correspond to a region of one.

**Q-2 — how much of the 111 should ever be compiled?** Not all of it. **20 of
the 111 are in migration guides** (`migration-0.4-to-0.5.md` and successors),
which deliberately show *old* APIs. Compiling those against current source
would be wrong — they are historical records, and their staleness is the point.
That leaves **91** candidates, and probably fewer.

**Q-3 — what replaces the reason rule for the book?** Suggest: one statement of
the structural cause on the policy page, plus making finding 3's existing
requirement — an `ignore` fence links to the example crate that validates it —
actually checkable. A grep for `ignore` fences with no nearby `examples/` link
is mechanisable in the way the crate-side grep is.

## Acceptance criteria

1. The `no_run` contradiction is resolved in favour of the ladder table's
   correct description (finding 2).
2. The policy states the **structural** reason book fences are `ignore` — no
   library path — rather than attributing it to snippet shape (finding 1).
3. `readability.md`'s "Compile-checked" claim is corrected or made true
   (finding 4). **See handoff note:** RFC-068 is editing the same block.
4. Q-1 decided, with CI cost measured rather than estimated if (a) or (b) wins.
5. Migration-guide fences are explicitly excluded from any compile mechanism,
   in writing, with the reason (finding Q-2).
6. Whatever rule replaces the per-fence reason is stated once and is
   mechanically checkable, or is explicitly recorded as deferred with the
   reason — the RFC-064 precedent.
7. **RFC-064's crate-side rule and grep are unchanged.** They are correct.

## Compatibility and security

**Compatibility.** Documentation only. If Q-1 selects (b), `docs/book.toml` and
a CI job change; no crate, no public API, no rendered output.

**Security.** None.
