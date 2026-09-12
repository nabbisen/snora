# Implementation handoff — RFC-098: the advisory gate is blind to unsoundness

**RFC.** `rfcs/accepted/098-the-advisory-gate-is-blind-to-unsoundness.md`
**Release target.** 0.49.0.
**Touch list.** `deny.toml`, `scripts/check-advisories.sh`, `Cargo.lock`,
`CHANGELOG.md`. **Nothing under `docs/`** — the threat model and the
correspondence are the architect's (criterion 5/6), and the threat-model
half is already done ahead of you, see *What is already done*.

**Read the RFC's evidence section before starting.** The mechanism was
established by controlled perturbation, and Unit 2's demonstration has to be
the same shape.

---

## What is already done, so you do not redo it

- **The root cause is established**, not hypothesised: `[advisories] unsound`
  is a **scope** key (`"all"`/`"workspace"`/`"transitive"`/`"none"`), not a
  lint level, and its default excludes transitive dependencies. Proven by
  changing only `informational = "unmaintained"` → `"unsound"` on an advisory
  the gate does report, which made it vanish (3 errors → 2, zero
  `error[unsound]`), with a control confirming the scratch database was read.
- **Ruled out:** the patched version range (rewriting `>= 0.18.2` to
  `>= 0.16.5` changes nothing) and the tool version (0.20.2 is the latest
  published; there is no upgrade that fixes this).
- **Checked for siblings:** `unmaintained`'s default already covers our
  transitive graph, `notice` has been removed upstream (cargo-deny PR 611) and
  setting it is now an error, `yanked` takes a lint level and nothing is
  yanked.
- **The threat model has already been corrected** to say the gate was blind
  and what it hid. When your work lands it needs a second, smaller edit —
  that one is also the architect's. Do not touch `docs/`.

## Unit 1 — set the scope, and say why it is a scope

`deny.toml`: add `unsound = "all"` to `[advisories]`.

The comment matters more than the line. It must record **that this key takes a
scope rather than a lint level**, and **that the default excludes transitive
dependencies — i.e. excludes every package snora has**, since we declare five
direct dependencies and 394 resolved ones. A future reader who thinks it is a
lint level will "tidy" it to `"deny"` and get
`error[unexpected-value]: expected '["all", "workspace", "transitive", "none"]'`
— which is a good failure, but the comment should mean they never try.

## Unit 2 — assert the configuration, not only the outcome

**This is the unit that matters.** Unit 1 fixes today's graph; Unit 2 is what
stops a seventh instance of a gate that cannot fail.

`scripts/check-advisories.sh` must **refuse if `deny.toml` does not set every
advisory class explicitly.** Q-2 is ruled: **hardcode the class list**, in one
place, with a comment naming the cargo-deny version it was read from
(`0.20.2`). A dated duplicate beats an undated blind spot, and the version pin
makes the duplicate checkable. Discovering the list by parsing vendor output
was considered and rejected — it makes the gate depend on output formatting,
which is a worse dependency than a pinned list.

Classes to require, as of 0.20.2: **`unsound`**, **`unmaintained`**,
**`yanked`**. Do **not** include `notice` — it is removed upstream and setting
it errors. `unsound` and `unmaintained` take scopes; `yanked` takes a lint
level. Your check only has to assert that each key is **present**, not what it
is set to — the value is a policy decision and locking it here would make the
script the policy.

**Required demonstration, and it must exercise the script rather than a
transcription:** remove one class from `deny.toml` in a scratch edit, confirm
the script exits non-zero and names the missing class, restore, confirm
byte-identical, confirm green. Do this for **two different classes**, not one
— a check that only recognises the absence of `unsound` is the same defect in a
new place.

**Watch the `set -euo pipefail` trap.** A `grep` over `deny.toml` that
legitimately matches nothing exits 1 and will kill the script before your own
error branch runs. This project has now hit that six times, including twice in
this file's own family (`check-workspace-iced-features.sh`,
`check-commit-ci-green.sh`) and once in the MSRV extraction I told you to copy
verbatim. Use `|| true` on the assignment and check emptiness yourself, the
way `check-wcag-floors.sh` already does — and say so in a comment.

## Unit 3 — clear the two that are clearable

```bash
cargo update -p memmap2 -p event-listener
```

`memmap2` 0.9.10 → 0.9.11 and `event-listener` 5.4.1 → 5.4.2 are the patched
versions for `RUSTSEC-2026-0186` and `RUSTSEC-2026-0221`. Clear first, gate
second (RFC-079); this time the clearing is genuinely available rather than
reducing to recording.

**Confirm the lockfile diff is those two packages and nothing else**, and say
so with the command in your report. A `cargo update` that moves more than you
intended is the 0.42.0 gpu-allocator shape.

Then run the MSRV gate — `cargo +1.88 check --workspace --all-features
--locked` — because a lockfile change is exactly where MSRV breaks, and the
full suite.

## Unit 4 — the residue, and the escalation that is not yours

After Units 1–3 the gate reports **one** advisory: `RUSTSEC-2026-0253`, `lru`
0.16.4, use-after-free/double-free in `LruCache::pop()`. It is **not fixable by
us** — `cargo update -p lru` and `-p cryoglyph` both lock 0 packages, and
`cryoglyph` has no release that takes `lru` 0.18.2.

**Do not write the `ignore` entry on your own judgement.** The owner's ruling
on Q-1 is pre-recorded and is this: **accept it with a stated reason, and name
it in the threat model explicitly, which the other three accepted advisories
are not.** The reason this one is different, and your entry must say so: it is
**memory-corruption on the default runtime path**, not an unmaintained crate.
Accepting it is not the same kind of act as accepting `paste`.

Write the entry in RFC-097's established form — reachability, why accepted,
and the condition that retires it. The retiring condition is a `cryoglyph`
release taking `lru >= 0.18.2`, and `--deny advisory-not-detected` will report
it the moment that lands.

**If your own verification contradicts any of the above — especially if a fix
turns out reachable after all — escalate rather than proceeding.** The RFC
exists because "the advisory says no fix" was believed once already this month
without checking the parent.

## CHANGELOG

**Yes**, and disclose it the way 0.47.0 disclosed its own first scan — the
precedent is now this project's own. It must say: the gate was blind to
`unsound`, for how long (since 0.47.0), what it hid (three advisories), what
was cleared (two), what is accepted and why it is a different kind of
acceptance, and **that a downstream team found it**. Attribute tekstide.

Do not soften it. 0.47.0's entry disclosed five advisories it could have
absorbed, and that is the entry this one is measured against.

## Gate suite

The usual, plus: `scripts/check-advisories.sh` (both the new refusal and the
real scan), `cargo +1.88 check --workspace --all-features --locked`, and
`scripts/check-workflows.sh` if you touch any workflow (you should not need to).

## Acceptance criteria

1. `unsound = "all"` set, with the scope-not-lint-level reasoning in the file.
2. Unit 2's configuration check exists and is **demonstrated failing on two
   different classes**, reverted and confirmed byte-identical each time.
3. `memmap2` and `event-listener` updated; lockfile diff confirmed to be those
   two packages only, with the command quoted.
4. The gate reports exactly one advisory before the `ignore` entry, and
   `advisories ok` after, with `--deny advisory-not-detected` still passing.
5. `lru` accepted in RFC-097's form, its reason stating that this is
   memory-corruption on the runtime path rather than unmaintainedness.
6. CHANGELOG entry, crediting tekstide.
7. Nothing under `docs/` touched.
