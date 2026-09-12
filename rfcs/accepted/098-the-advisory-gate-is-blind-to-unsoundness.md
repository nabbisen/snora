# RFC-098 — The advisory gate is blind to an entire advisory class

**Status.** Accepted 2026-09-12.
**Raised.** 2026-09-12, architect.
**Release target.** 0.49.0 — and one part of it is not releasable work at all
(see *Correspondence*, which is owed now rather than at a release).
**Found by.** **tekstide** — who is not a consumer, and whom I deliberately
excluded from the 0.46→0.48 letter.

---

## The finding

`scripts/check-advisories.sh` reports `advisories ok`. Its configuration
denies advisories with no severity threshold and three stated exceptions. It
has been the project's first and only security mechanism since 0.47.0.

**It cannot report an `unsound` advisory, and three are in our graph.**

cargo-deny's `[advisories]` table has an `unsound` key that does not take a
lint level — it takes a **scope**: `"all"`, `"workspace"`, `"transitive"`, or
`"none"`. We never set it, and **its default excludes transitive
dependencies.** Every one of snora's 394 resolved packages is transitive; we
declare five direct dependencies and none of them is implicated. So the
default silently excludes the entire population the gate exists to watch.

Setting `unsound = "all"` turns `advisories ok` into three errors:

| Advisory | Crate | What | Fixable by us today? |
|---|---|---|---|
| `RUSTSEC-2026-0253` | `lru` 0.16.4 | Use-after-free / double-free in `LruCache::pop()` when a key's `Drop` panics (CWE-416, CWE-415) | **No** — `cargo update -p lru` and `-p cryoglyph` both lock 0 packages; 0.18.2 is patched but `cryoglyph` will not take it |
| `RUSTSEC-2026-0186` | `memmap2` 0.9.10 | Unchecked pointer offset | **Yes** — `cargo update -p memmap2` → 0.9.11, which is the patched version |
| `RUSTSEC-2026-0221` | `event-listener` 5.4.1 | `!Send` tags cross thread boundaries via `StackSlot` | **Yes** — `cargo update -p event-listener` → 5.4.2, which is the patched version |

**Two of the three are one command away and have been the whole time.**

### Proven by perturbation, not inferred from a config reference

The mechanism was established by controlled experiment, because "the default
must exclude it" is a guess until something demonstrates it:

1. **The advisory is in the database the gate reads.** `RUSTSEC-2026-0253` is
   present at DB HEAD (`2026-09-09`), added 2026-08-11.
2. **The crate is in the graph cargo-deny itself sees** — `cargo-deny list`
   enumerates 394 crates and `lru 0.16.4` is one of them.
3. **The gate says it never saw the advisory.** With `0253` placed in `ignore`
   and `--deny advisory-not-detected` on, the run fails with
   `error[advisory-not-detected]` — cargo-deny's own statement that the
   advisory matched nothing.
4. **It is not the patched range.** Rewriting the advisory's
   `patched = [">= 0.18.2"]` to `[">= 0.16.5"]` in a scratch copy of the
   database changes nothing; `0.16.4` still goes unreported.
5. **It is the class, and only the class.** Taking an advisory the gate *does*
   report — `RUSTSEC-2026-0192`, ttf-parser — and changing **only**
   `informational = "unmaintained"` to `"unsound"` makes it vanish: three
   reported errors become two, with zero `error[unsound]`.
6. **Control that the experiment measured anything:** the same scratch
   database with an empty `ignore` list reports all three unmaintained
   advisories and exits 1, so the database was genuinely being read.

Step 5 is the one that settles it. Everything else about that advisory was
held constant.

### Not a version problem

We pin `cargo-deny 0.20.2`, which is **the latest published version**
(2026-07-09). There is no upgrade that fixes this. It is a configuration
default, and defaults are what we failed to read.

### What else was checked, so this is not fixed by halves

- `unmaintained = "all"` surfaces nothing new — that key's default already
  covers our transitive graph, which is precisely why three unmaintained
  advisories *were* reported and why the gate looked like it worked.
- `notice` has been **removed** as a key upstream (cargo-deny PR 611);
  setting it is now an error.
- `yanked` takes a lint level rather than a scope, and nothing in the graph is
  yanked.

So `unsound` is the only blind class — but that is a statement about today's
graph, not about the configuration, which is the point of R-2 below.

## Why this matters more than three advisories

**This is the failure mode the project has spent RFC-090 to RFC-097
eliminating, in the mechanism built to eliminate it.** A gate that cannot fail
for a whole category, whose green is read as coverage. RFC-097's own script
header says an unrunnable scanner reporting green is *"the defect this project
has now found five times… and a security scanner is the worst place to add a
sixth."* It was the worst place, and this is the sixth.

Three statements now on record are wrong or incomplete:

1. **`docs/src/reference/threat-model.md`** (shipped 0.47.0, written by me)
   says the gate scans the resolved graph and *"refuses rather than reporting
   clean."* It does report clean, over unsoundness.
2. **The 0.47.0 CHANGELOG** discloses *"five advisories, two fixed, three
   accepted"* as the complete first-scan result. It was not complete.
3. **The 0.46→0.48 letter, already sent to five teams**, says the three
   accepted are *"unmaintained-crate advisories, not vulnerabilities."*
   tekstide's reply names the gap exactly: *"unsound is a different class from
   unmaintained"*, and *"if your gate does not surface it, that is worth
   knowing more than the advisory itself is."* They were right on both counts.

**A downstream non-consumer audited our disclosure more effectively than our
own gate did.** That is the finding to keep.

## Proposal

**R-1 — set the scope explicitly, and state why the value is a scope.**
`unsound = "all"` in `deny.toml`, with a comment recording that this key is a
scope rather than a lint level and that its default excludes transitive
dependencies — i.e. excludes everything snora has.

**R-2 — assert the configuration, not just the outcome.** A gate whose
correctness depends on unstated defaults must not depend on anyone rereading
the vendor's reference. `scripts/check-advisories.sh` should fail if
`deny.toml` does not set every advisory class explicitly, so a class added or
renamed upstream is a build failure rather than a silent narrowing. **This is
the half that stops a seventh instance**; R-1 alone fixes today's graph and
leaves the shape intact.

**R-3 — clear the two that are clearable.** `cargo update -p memmap2 -p
event-listener`. Clear first, gate second (RFC-079), and this time the
clearing is genuinely available rather than reduced to recording.

**R-4 — rule the residue.** `lru` is not fixable by us: `cryoglyph` holds it
below the patched version and `cryoglyph` has no newer release. Under RFC-097's
established form that means an `ignore` entry with reachability, reasoning and
a retiring condition — but the reachability is worse than the three we already
accepted: it is **runtime, on the default rendering path, and the advisory is
memory-corruption**, not unmaintainedness. **Owner's decision, not mine**, and
the honest framing is that accepting it is not the same kind of act as
accepting an unmaintained crate.

**R-5 — correct the three statements**, and correct them in the order they
reach people: the letter first, then the threat model, then the CHANGELOG's
historical entry annotated rather than rewritten.

## Correspondence — owed now, not at a release

This is the part that should not wait for 0.49.0.

Five teams hold a letter from us that understates our own graph. Two of them
(**orbok**, **arama**) keep conformance or contrast records that cite us;
orbok's reply says they have already adopted our construction for their own
gate, including *"the gate that fails when a row stops matching the graph."*
**They copied a practice from a gate we now know was incomplete**, and they
should hear that from us.

**A correction letter is warranted under the existing bar** — this is a
withdrawn claim that teams have acted on, which is the one case the
correspondence rule names outright.

**Also owed: a reply to orbok on a separate error of theirs.** They wrote that
they will record the non-colour cue as *"ruled at 0.47.0"*. **It was ruled at
0.48.0** — the RFC-093 Q-1 entry is in 0.48.0's CHANGELOG, not 0.47.0's. A
wrong version in a record that cites us nineteen times is worth one line, and
it is the kind of error that propagates.

## Non-goals

- **Not tightening `bans`, `sources` or `licenses`.** Still non-fatal, still a
  separate decision with its own evidence.
- **Not claiming the gate is now complete.** R-2 exists because the honest
  claim after this RFC is *"every class cargo-deny exposes is set explicitly
  and a new one breaks the build"*, which is a different and much weaker claim
  than *"we watch the graph."*
- **Not blaming cargo-deny.** The key is documented. We did not read it, and
  the gate's own review — mine — checked that it refused when the tool was
  missing and when the database was unobtainable, and never asked whether its
  defaults matched its purpose.

## Open questions

**Q-1 — does `lru` get an `ignore` entry, or does it hold the gate red?**
Unlike the three accepted at 0.47.0 this is memory-corruption on the default
runtime path. Suggest: accept with a stated reason, because the alternative is
a standing red we cannot act on — but **flag it in the threat model by name**,
which the other three are not, and say why it is different.

**Q-2 — should R-2 assert against a hardcoded class list, or discover it?**
A hardcoded list is a hand-maintained duplicate of cargo-deny's schema, which
this project keeps replacing with mechanisms. Discovering the list means
parsing vendor output. Suggest: hardcode, with the classes named in one place
and a comment pointing at the version they were read from — a dated
duplicate beats an undated blind spot, and the pin makes it checkable.

**Q-3 — do the measurement CSVs need a re-run?** `cargo update -p memmap2 -p
event-listener` moves the shipped graph. 0.47.0's unexplained −640 B is still
open; a lockfile change here means the next row is not comparable to it.
Suggest: note it in the binary-size row rather than attempting attribution.

## Acceptance criteria

1. `unsound = "all"` set, with the scope-not-lint-level reasoning in the file.
2. **R-2 exists and is demonstrated failing** — remove one class from
   `deny.toml` in a scratch edit, confirm the script refuses, restore.
3. `memmap2` and `event-listener` updated; the gate reports the `lru` advisory
   and nothing else.
4. Q-1 ruled by the owner, and whatever it rules is written into `deny.toml`
   with a retiring condition.
5. Correspondence: correction letter drafted for five teams; orbok's 0.47.0 /
   0.48.0 error answered.
6. `threat-model.md` corrected; the 0.47.0 CHANGELOG entry annotated, not
   rewritten — the disclosure was made in good faith and the record should
   show both what we said and what we later found.
7. CHANGELOG entry for 0.49.0, disclosing this the way 0.47.0 disclosed its
   own first scan.
