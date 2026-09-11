# Developer Handoff — 0.47.0: four mechanisms from the 2026-09-12 audit

**Governing document.** Not an RFC — four small mechanisms. The audit is at
`.git-exclude/reviewed/audit-2026-09-12/audit-result.md`; the schedule is in
[`ROADMAP.md`](../../../ROADMAP.md) under *"Scheduled — the 2026-09-12 internal
audit"*.
**Status.** Approved by the owner, 2026-09-12.
**Release target.** **0.47.0.**
**Touches.** Five `lib.rs` files (attribute only), `.github/workflows/ci.yaml`,
one new `scripts/check-*.sh`, and three comments. **No behaviour change.**

---

## What these four have in common

Each is a property that **is already true and is enforced by nothing.** None is
a bug fix. The audit found four instances of the defect this project spent
RFC-090 to RFC-096 naming, and these are them.

That shapes the evidence: for every unit, the demonstration is **break the
property and watch the new mechanism refuse**. If a mechanism cannot be shown
refusing, it has not been adopted — it has been added.

---

## Unit 1 — `#![forbid(unsafe_code)]` in all five crates

`grep -rnw unsafe crates/ examples/` returns **one hit, in a prose comment**. The
codebase is unsafe-free across 13,331 lines and nothing stops the next line.

**Already verified for you, so you do not have to discover it:** I added the
attribute to all five `lib.rs` files and `cargo check --workspace --all-features`
passed. No dependency macro expands to `unsafe` inside our crates, which is the
one thing that could have made `forbid` (stricter than `deny`, and
un-overridable by an inner attribute) fail. Restored afterwards; the tree you get
is unchanged.

Use `forbid`, not `deny`. `deny` can be locally overridden by
`#[allow(unsafe_code)]`; `forbid` cannot, and an un-overridable rule is the whole
point.

**Evidence:** add `fn _probe() { unsafe { } }` to one crate, confirm
`error: usage of an `unsafe` block`, restore. I have done this on `snora-core`
and it refuses; do it on whichever crate you like so the transcript is yours.

## Unit 2 — gate that the two leaves stay independent

`docs/src/reference/architecture.md:4` states `snora-core` and `snora-design` are
*"independent, iced-free leaves."*

**Iced-freedom is gated twice** in `design-isolation` — one step per crate,
RFC-083. **Independence is gated by nothing.** A `snora-design → snora-core`
edge, or the reverse, would pass every check in the repository while falsifying
the sentence.

Not hypothetical: a v0.25.1 handoff asserted that exact dependency existed when
it did not, and nothing settled it.

Add a step beside the existing two, same shape: `cargo tree -p snora-core
--all-features` must not contain `snora-design`, and `cargo tree -p snora-design
--all-features` must not contain `snora-core`. **Both directions** — the sentence
claims mutual independence, not one-way.

**Evidence:** add the dependency to one crate's `Cargo.toml`, confirm the step
fails and names which direction, restore.

## Unit 3 — verify the MSRV against the lockfile we actually ship

`rust-version = "1.88"` ships in all five crates' metadata. Consumers resolve
against it. Verified today: **0 resolved dependencies declare a higher
rust-version**, so the claim currently holds.

What does not exist is the gate:

- Every job in `ci.yaml` uses `dtolnay/rust-toolchain@stable` — four occurrences,
  no MSRV toolchain anywhere.
- `unpinned-build.yaml` checks the MSRV **weekly, against the freshly
  `cargo update`d graph** — deliberately not the graph we publish.
- `release-process.md:395` has it as a **manual** checklist step.

So between releases nothing automated verifies the floor, and the automated check
that exists tests a different dependency graph than consumers get.

Add a job to `ci.yaml` that reads the MSRV from `Cargo.toml` — **do not hardcode
`1.88`**; `unpinned-build.yaml:53` already has the extraction and it fails loudly
when the field is absent, which is the behaviour to copy — installs that
toolchain, and runs `cargo check --workspace --all-features --locked`.

**`--locked` is the point of this unit.** Without it cargo may re-resolve and you
have rebuilt `unpinned-build`'s weekly check on a different schedule.

**Evidence:** this is the awkward one. Installing a toolchain that does not exist
proves nothing about MSRV. Introduce code using a language or library feature
stabilised **after** 1.88, confirm the new job fails while the stable jobs pass,
restore. If you cannot find one cheaply, say so and propose the alternative
rather than shipping the job unproven.

## Unit 4 — the WCAG floors are in four copies, guarded by a comment that is wrong

```
snora-design/src/tests.rs:24              AA_TEXT = 4.5
snora-widgets/src/contrast_tests.rs:91    AA_TEXT = 4.5
snora/src/toast/contrast_tests.rs:45      AA_TEXT = 4.5

snora-design/src/tests.rs:26              NON_TEXT_MIN = 3.0
snora-widgets/src/contrast_tests.rs:95    NON_TEXT_MIN = 3.0
snora/src/toast/contrast_tests.rs:49      NON_TEXT_MIN = 3.0
snora/src/design/render/tests.rs:84       NON_TEXT_MIN = 3.0    <- fourth
```

All values currently agree. **The duplication is correct and must stay** — the
engine depends on neither `snora-design` nor `snora-widgets` by design, so these
genuinely cannot be shared.

**The guard is the problem.** `snora/src/toast/contrast_tests.rs:40` names two
siblings and says *"Check all three if this value ever changes."* `NON_TEXT_MIN`
has **four** copies; the one in `snora/src/design/render/tests.rs` is not named.

And that fourth copy's own comment records why it matters:

> *"nothing links them beyond this comment, which is exactly the gap that let
> `1.3` drift from `3.0` unnoticed here in the first place (RFC-071 review,
> round 1)."*

**This value has already drifted once.** The remedy adopted was a comment, and
the comment network is now itself wrong about its own scope.

Write `scripts/check-wcag-floors.sh`, in the family that already exists. It should
extract every `const AA_TEXT`/`NON_TEXT_MIN`/`FOCUS_MIN`/`TEXT_SIZE_MIN` under
`crates/` and fail if any name carries two different values.

**Two design points worth your judgement, not mine:**

- **Should it also fail when the number of copies changes?** A fifth copy
  appearing with the *right* value is still a fact nobody decided. I lean yes,
  with the expected counts written into the script, but a pinned count is a
  maintenance cost and you are closer to it. Decide and say which.
- **Then fix the comments** so they describe the real set. The toast file says
  "three"; make it say what is true.

**Evidence:** change one copy to a different value, confirm the script names both
files and both values, restore.

## Required across all four

Each unit demonstrated failing, transcript in the review package. **Four
mechanisms, four demonstrations** — not one representative sample.

## Acceptance criteria

1. All five crates carry `#![forbid(unsafe_code)]`, demonstrated refusing.
2. A `design-isolation` step gates leaf independence **both directions**,
   demonstrated failing.
3. An MSRV job checks the **committed lockfile** with the MSRV read from
   `Cargo.toml`, demonstrated failing — or a stated reason the demonstration was
   not achievable.
4. A WCAG-floor script exists, Q on copy-counting answered either way, the
   comments corrected, demonstrated failing.
5. **No CHANGELOG entry** — no consumer-observable change in any of the four.
   Stated so the omission is a decision, per RFC-092.
