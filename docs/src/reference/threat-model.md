# Threat model

What snora's attack surface is, and — equally — what it is not. Written
2026-09-12 (RFC-097).

If you are reporting a suspected vulnerability, see
[SECURITY.md](https://github.com/nabbisen/snora/blob/main/.github/SECURITY.md).

## What snora is, in security terms

**A library, linked into your application's process.** It has no network
client or server, no persistence, no IPC, no child processes, no
credentials, and no privilege boundary of its own. It renders what your
application hands it and returns an `iced::Element`.

That makes most of the usual categories inapplicable, and this document
says so rather than inventing risk to fill a template. **There is one
category that is not inapplicable**, and it is the reason this page
exists.

## 1. UI integrity — the category where our bug becomes your vulnerability

**snora composes the overlay z-stack and decides which layer receives
pointer input.** Your application does not do this; we do. So a defect in
containment is not a cosmetic bug in a framework — it is a security-relevant
defect in every application that trusts the framework.

**This is not hypothetical.** In 0.41.0, RFC-084 fixed four places where
snora did not contain pointer input it was documented to contain. The
consequence, stated in that release's migration guide:

> A modal that does not block input is a UI-integrity issue: a
> confirmation dialog could be bypassed by clicking through it to the
> control it was meant to be guarding.

An application showing *"Delete everything? [Cancel] [Confirm]"* over a
page containing a **Delete** button relies on the dialog to stop a click
reaching that button. That reliance is reasonable, it is what the
documentation promises, and for four surfaces it was not true.

**What we do about it now:**

- Every containment claim in
  [overlay interaction semantics](overlay-interaction-semantics.md) records
  whether a test would fail if the engine stopped obeying it (RFC-096).
- The containment assertions are *negative* — they assert input is
  **blocked**, not merely that content is reachable. Gate 5 of the
  [1.0 readiness review](../contributing/api-freeze-review.md) sat ticked
  for 24 minors on positive-only tests, which is how the four defects
  survived.

**What this means for you.** If your application guards a destructive
action behind a snora overlay, that guard is only as good as our
containment. Treat a snora upgrade that touches overlay composition as
worth re-testing those flows — the migration guides flag when one does.

## 2. The application → snora boundary

**Almost everything you pass us is inert.** `Element`s are rendered,
strings are drawn. There is no evaluation, no template expansion, no
deserialisation of caller input.

**One exception, and it is a real one.**
[`Icon::Svg`](../guides/icons.md) takes a `std::path::PathBuf`. iced reads
that file from disk at render time and parses it as SVG.

**snora validates neither the path nor the file.** For an `assets/`
directory you ship, that is exactly right. If the path or its contents can
come from somewhere you do not control — a plugin directory, a theme a
user installs, anything downloaded — **the trust boundary is yours**, and
you are handing a filesystem read and an XML parser something you did not
write. snora adds no sandbox, no path restriction, and no size or
complexity limit beyond the SVG renderer's own.

The guide says this at the call site too, because a type signature that
takes a `PathBuf` does not tell you which of the two situations you are in.

## 3. The snora → dependency boundary

**310 packages** resolve under `--all-features`. snora contributes a small
fraction of the code that ends up in your binary; `iced`, `wgpu` and the
SVG stack contribute most of it.

We do not audit those by hand, and a page claiming we did would be worth
nothing. What we do is stated in
[the supply-chain section](#supply-chain-and-release) below.

**Zero `unsafe`.** snora's own five crates contain no `unsafe` code, and
every one of them carries `#![forbid(unsafe_code)]` — so this holds
**by mechanism, not by habit**: an `unsafe` block anywhere in
snora's own source fails to compile, rather than passing review unnoticed.
Our dependencies contain a great deal of `unsafe`, as any GPU-backed
renderer must.

## 4. Supply chain and release

**Publishing does not go through a laptop.** Since 0.42.0, releases are
published by `release.yaml` on a signed tag push, authenticating to
crates.io with **Trusted Publishing (OIDC)**. There is no long-lived
registry token in this repository or in its secrets (RFC-090).

Before any upload the workflow refuses if the tag disagrees with
`[workspace.package].version`, or if the tagged commit has no completed,
successful CI run. Both refusals have been demonstrated firing.

**Advisory scanning:** the `supply-chain` workflow runs `cargo-deny`
weekly over the resolved `--all-features` graph — the same graph the
rest of CI builds. Advisories are fatal; licences, bans and
sources are reported without failing. The scanner is pinned to an exact
version and checksum-verified before it runs, and it refuses rather
than reporting clean when it cannot be obtained or when the advisory
database cannot be fetched: an unscanned graph is not a clean one.

> **Correction, 0.49.0: for two releases this gate reported clean over a
> class of advisory it could not see (RFC-098).** cargo-deny's `unsound`
> setting is a *scope* rather than a lint level, and its default
> excludes transitive dependencies — which is every package snora has.
> So from 0.47.0, when this mechanism shipped, to 0.48.0,
> `advisories ok` meant *"no vulnerability and no unmaintained-crate
> advisory"* and **not** *"no unsoundness"*. Fixed in 0.49.0 by setting
> the scope explicitly — **and the gate now refuses to run at all
> against a configuration that leaves any advisory class unset**, because
> the failure was never a wrong value but an absent key behaving as a
> permissive default.
>
> **It was hiding three**, in `lru`, `memmap2` and `event-listener`.
> `memmap2` and `event-listener` had published fixes and were taken in
> 0.49.0. The third —
> `RUSTSEC-2026-0253`, a use-after-free in `lru`'s `LruCache::pop()`
> when a stored key's `Drop` panics — is **not fixable by us**:
> `cryoglyph` 0.1.0, its only published release, declares `lru ^0.16`,
> which cannot admit the patched 0.18.2. It is reached at run time through
> `cryoglyph → iced_wgpu → iced_renderer → iced`, and unlike the three
> accepted advisories below it is **memory-corruption on the default
> rendering path, not an unmaintained crate** — which is why it is named
> here rather than only recorded in `deny.toml`.
>
> **This was found by a downstream team reading our own disclosure, not
> by us.** The paragraph below describes what the first scan found; it
> was accurate about what the gate reported and incomplete about what
> the gate could report. Both halves are left standing rather than
> rewritten, because a threat model that quietly corrects itself is
> worth less than one that shows where it was wrong.

It is scheduled rather than per-push, and it is not a refusal in
`release.yaml`. Both are deliberate. Nothing reaches a consumer on a
push, so blocking pushes would cost real work for no protection; and an
advisory sometimes has no fixed version, so a workflow step that cannot
weigh severity or reachability would make snora unreleasable through no
fault of its own, and would be routed around the first time it was
wrong. The release-time decision is a human one, taken with the advisory
in front of the person taking it.

**What the first scan found, and what it says about the older gap.** It
found five advisories. Two were vulnerabilities in `quick-xml`, reached
only through `wayland-scanner` — a build-time proc-macro parsing the
Wayland protocol XML that ships with the build, so neither was present
at run time. Both were fixed by updating the parent package, and snora's
lockfile now carries the patched version. Consumers resolving snora
fresh were never on the unpatched one.

The remaining three are **unmaintained-crate advisories, not
vulnerabilities**: `paste` (a build-time proc-macro, macOS-only, absent
from the Linux graph entirely), `rustybuzz` (run time, on the SVG path
this document already names as a consumer-owned boundary) and
`ttf-parser` (run time, font parsing on the default text path). None has
a published upgrade. Each carries an `ignore` entry in `deny.toml`
stating its reachability, why it is accepted, and the condition that
retires it — and the gate is run with `--deny advisory-not-detected`, so
an entry that stops matching the graph **fails the job and names itself
for deletion**. An accepted risk here cannot outlive its own reason
through inattention.

`unpinned-build`, the weekly job that re-resolves the graph and fails if
it stops compiling, is the older mechanism and is not a substitute. It
watches upstream *movement*, not upstream *vulnerability*, and the same
five advisories are the proof. The two `quick-xml` advisories were
published 2026-06-29. The parent release carrying the fix,
`wayland-scanner 0.31.11`, was published 2026-07-22. `unpinned-build`
re-resolves every Monday, so it spent roughly three weeks resolving the
**vulnerable** version and then roughly seven weeks resolving the
**fixed** one — and reported green for all ten, because both compile.
It could see neither the vulnerability nor the fix, and it was never
built to.

## What snora does not defend

Stated so this document has a boundary, and so nobody reads an omission as
a claim:

- **Untrusted content you choose to render.** If you draw attacker-supplied
  text, we draw it. Length limits, escaping decisions and sanitisation are
  yours.
- **Rendering cost from pathological input.** A deeply nested SVG or an
  enormous string can make rendering slow. snora imposes no budget and has
  no timeout.
- **Your application's data, at rest or in transit.** snora never sees it
  leave the process.
- **Anything about the application's own authentication, authorisation, or
  session handling.** We render the UI; we do not know what it means.
- **Dependency behaviour at runtime.** We choose dependencies and watch for
  advisories; we do not sandbox them.

## Reporting

Privately, via
[GitHub security advisories](https://github.com/nabbisen/snora/security/advisories/new).
Acknowledgement within seven days, a fix or a clear timeline within thirty
days of acknowledgement. Full policy in
[SECURITY.md](https://github.com/nabbisen/snora/blob/main/.github/SECURITY.md).
