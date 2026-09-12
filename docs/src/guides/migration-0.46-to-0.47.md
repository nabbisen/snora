# Migration 0.46 → 0.47

> **Not breaking. Nothing in your application needs to change.** No public
> API change, no appearance change, no feature resolution change, no MSRV
> change. This guide exists because every minor ships one, *even to say
> nothing is required* (RFC-079) — and because this release is one you may
> want to cite rather than merely install.

## Who is affected

**For code: nobody.** Upgrading is a version bump.

**If your team maintains a security questionnaire, an assurance record,
or an SBOM review that mentions snora, this is the release to re-read.**
Two of its changes are the kind a reviewer asks about, and until now the
honest answer to both was "by habit, not by mechanism."

## What changed

### `#![forbid(unsafe_code)]` in all five crates

snora's own source has contained zero `unsafe` for its whole life — 13,331
lines, verified. That was true and unenforced. It is now a property of the
crates you depend on that the compiler holds: an `unsafe` block anywhere in
snora's own source fails to build.

**Nothing about the current code changed**, which is the point — the
property was already true, so adopting it could not regress anything. What
changed is that it can no longer be lost quietly.

This says nothing about our dependencies, which contain a great deal of
`unsafe`, as any GPU-backed renderer must. See
[the threat model](../reference/threat-model.md) for where that line sits.

### Dependency advisory scanning, and what its first run found

A weekly job now scans the resolved dependency graph against the RustSec
advisory database. Before this, nothing in the repository did — seven
workflows checked that snora compiles, lints, documents and resolves, and
none checked whether any of 310 packages was known vulnerable.

**Its first run found five advisories, and that is disclosed here rather
than absorbed quietly.**

**Two were vulnerabilities, and they are fixed.** `RUSTSEC-2026-0194` and
`RUSTSEC-2026-0195` in `quick-xml`, both denial-of-service. Two things
matter for you:

- **You were almost certainly never exposed.** The patched release was
  already inside the version range our dependency declares, so anyone
  resolving snora fresh had been getting it. What was behind was *our*
  committed lockfile, which does not bind your build.
- In snora's graph `quick-xml` is reached only by a **build-time
  proc-macro**, parsing protocol XML that ships with the build. It was
  never parsing anything of yours at run time.

**Three are accepted, and none is a vulnerability.** They are
unmaintained-crate advisories against `paste` (build-time, macOS-only),
`rustybuzz` and `ttf-parser` (both run time, no upgrade published). Each
is recorded with its reachability, the reasoning for accepting it, and the
condition that ends the acceptance. If you run your own advisory scan
against a graph containing snora, you will see these three; this is our
statement of what they are, so a scan result is not a surprise.

**Nothing here obliges you to act.** It is written down so that a reviewer
asking "what does snora do about dependency advisories" has an answer
with dates and numbers in it.

## What did not change

- No public API, no rendered appearance, no feature resolution, no MSRV
  (still `1.88`).
- **No visual-regression baseline is invalidated** — the fourth release
  running.
- Your own dependency resolution. The lockfile that moved is ours; a
  library's lockfile does not bind the applications that depend on it.

## If you are jumping more than one minor

**0.45** removed `snora_design::{Emphasis, Size}` — breaking, though all
six adopting teams confirmed beforehand that neither appeared in their
code. **0.41** and **0.42** carry real behavioural and dependency
changes. The [migration index](migrations.md) lists them all.
