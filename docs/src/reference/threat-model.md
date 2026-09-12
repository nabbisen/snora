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

**Zero `unsafe`.** snora's own five crates contain no `unsafe` code —
verified 2026-09-12 across 13,331 lines. **Today that holds by habit and
not by mechanism**; `#![forbid(unsafe_code)]` is scheduled for 0.47.0,
after which it cannot be lost silently. Our dependencies contain a great
deal of `unsafe`, as any GPU-backed renderer must.

## 4. Supply chain and release

**Publishing does not go through a laptop.** Since 0.42.0, releases are
published by `release.yaml` on a signed tag push, authenticating to
crates.io with **Trusted Publishing (OIDC)**. There is no long-lived
registry token in this repository or in its secrets (RFC-090).

Before any upload the workflow refuses if the tag disagrees with
`[workspace.package].version`, or if the tagged commit has no completed,
successful CI run. Both refusals have been demonstrated firing.

**Advisory scanning:** as of this writing, **there is none.** Nothing in
this repository checks the resolved graph against a vulnerability
database — that gap is what RFC-097 was raised for, and the mechanism is
scheduled for 0.48.0. Said plainly here rather than omitted, because an
absent section in a threat model reads as a solved problem.

What does exist is `unpinned-build`, a weekly job that re-resolves the
graph and fails if it stops compiling. That watches upstream *movement*,
not upstream *vulnerability*, and the difference is the point.
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
