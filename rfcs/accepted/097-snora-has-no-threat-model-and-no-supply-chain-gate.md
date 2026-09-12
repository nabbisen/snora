# RFC 097 — snora has no threat model, and nothing watches 310 dependencies

**Status.** Accepted (owner, 2026-09-12). Handoff written — see
[`handoffs/097-…`](../handoffs/097-snora-has-no-threat-model-and-no-supply-chain-gate/implementation-handoff.md).
**Unit 1 (the threat model) is shipped** — `docs/src/reference/threat-model.md`.
**Q-1 ruled** — `cargo-deny`. **Q-2 ruled, and against my own lean's first form** —
scheduled job, plus a release-checklist decision point rather than a hard refusal;
see the question's own text. **Q-3 ruled** — the book, under `reference/`.
**Q-4 ruled** — fixed here.
**Tracks.** Security. **Severity: High.**
**Found by** the 2026-09-12 internal audit (findings S-1 and S-2).
**Touches.** `docs/src/reference/` (new page), `.github/SECURITY.md`,
`.github/workflows/` (new job), possibly `deny.toml`. No crate code.
**Release target.** 0.48.0.

## The finding

**Seven workflows, and not one of them is about security.**

snora gates compilation, clippy, rustdoc warnings, nine feature combinations,
iced-freedom in two crates, workflow syntax, built doc links, repo-relative
links, version snippets, migration-guide coverage, docs-only commit claims, and
whether the workspace still compiles against a freshly re-resolved dependency
graph.

It gates nothing about whether a dependency is known vulnerable. There is no
`cargo-audit`, no `cargo-deny`, no advisory database, no `deny.toml` — searched
`.github/` and `scripts/` for all of them. **310 packages** resolve under
`--all-features`; **five crates** are published to crates.io.

`unpinned-build` is the sharpest version of the gap. It exists specifically to
watch upstream movement, and it watches only the compile axis. *"Does upstream
still build"* is gated weekly. *"Is upstream known to be exploitable"* is gated
never.

**And there is no threat model.** `.github/SECURITY.md` covers reporting —
privately, 7-day acknowledgement, 30-day fix — and says nothing about what
snora's attack surface is. It asks reporters for *"scripted server responses"* as
a reproducer, which is boilerplate from a project with a network surface. snora
has none. That line tells a reporter the maintainer has not thought about this
project's actual shape.

## The part that makes this worth an RFC rather than a job

A GUI library's threat model is usually boring: no network, no persistence, no
authentication, no privilege boundary. Most of snora's is exactly that boring,
and the document should say so plainly rather than inventing risk.

**But snora owns one thing that is not boring, and it has already produced a real
defect.** snora composes the overlay z-stack and decides which layer receives
pointer input. RFC-084 found four places where it did not contain input that it
claimed to, and the 0.40→0.41 migration guide states the consequence correctly:

> *"A modal that does not block input is a UI-integrity issue: a confirmation
> dialog could be bypassed by clicking through it to the control it was meant to
> be guarding."*

**That is threat-model content, and it currently lives in a migration guide** —
findable only by someone upgrading across that exact boundary. It is the one
category where a snora bug becomes an application's security bug, and it is
written down in the least durable place we have.

The model is worth writing because we already know what belongs in it. We
learned it the expensive way and then filed it under "migration".

## Proposal — the model first, then the mechanism

**Two units, and the order is the argument.**

### Unit 1 — a threat model

A reference page stating, plainly:

- **What snora is.** A library linked into an application. No network, no
  storage, no IPC, no privilege boundary, no process of its own.
- **The surfaces that exist.** The application→snora boundary (what a caller
  hands us: `Element`s, strings, and — the one real case — `Icon::Svg`'s
  filesystem path, which iced reads and parses at render time). The
  snora→dependency boundary (310 packages, `iced`/`wgpu`/`resvg`). The build and
  release path (Trusted Publishing over OIDC, no long-lived token, signed tags).
- **UI integrity**, given its own section, because it is the category where our
  bug becomes their vulnerability, and because RFC-084 proved it is not
  theoretical.
- **What snora explicitly does not defend.** Untrusted content the application
  chooses to render, rendering cost from pathological input, and anything about
  the application's own data. Naming the non-goals is what stops the document
  growing without limit — and it is the half most threat models omit.

### Unit 2 — a supply-chain gate

Advisory scanning across the resolved graph, wired into CI. **Unit 1 first**,
because a gate adopted without a model saying what it defends is a gate nobody
can scope, and this project already has the cautionary case: RFC-094's sweep
found rows ticked on evidence nobody could name.

## Non-goals

- **Not auditing 310 dependencies by hand.** The point is a mechanism that keeps
  answering, not a one-time answer.
- **Not a formal STRIDE or LINDDUN exercise.** The surface does not justify the
  ceremony, and a template filled in for its own sake is the prose control this
  project keeps removing.
- **Not claiming conformance to anything.** We already withdrew one published
  conformance claim (1.4.1, 0.41.1). A threat model that overstates is worse than
  none.
- **Not security-hardening the code.** No defect is alleged here. If the model
  surfaces one, that is a separate RFC.

## Open questions

**Q-1 — `cargo-audit` or `cargo-deny`?** `cargo-audit` does advisories only.
`cargo-deny` does advisories **plus licences, duplicate versions, and source
allow-lists**. The audit also flagged that **licence compatibility across 310
packages is unverified**, which is the same missing tooling. **Suggest
`cargo-deny`**, so one mechanism answers both, with `deny.toml` starting
permissive and tightening later rather than blocking the adoption on policy
debate.

**Q-2 — per-push or scheduled, and does it fail or warn?** This is the question
that matters, and this project has the evidence to answer it. **A new advisory
lands against a pinned dependency without any change of ours** — the same shape
as `tinyvec 1.13.0`, which went red on `unpinned-build` and held the 0.44.0 tag
for a day. That was correct behaviour for a release gate and would be
intolerable on every push, where it would block work unrelated to the advisory.

**Ruled 2026-09-12: a scheduled job beside `unpinned-build`, failing loudly,
plus a release-checklist decision point — and deliberately NOT a hard refusal in
`release.yaml`.**

The decisive question is what blocking a push actually prevents. **Nothing
reaches a consumer on a push.** Consumers get crates from a release. So a
per-push failure costs real blocked work for zero consumer protection, and the
gate that matters is the one at the cut. That is the same reasoning that put
`unpinned-build` on a schedule and the three refusals in `release.yaml`.

**The release side is a checklist line and not a `release.yaml` refusal, which
is a change from my own first form of this suggestion.** Advisories sometimes
have no fixed version. A hard refusal would make snora unreleasable through no
fault of ours and with no remedy available — the `tinyvec 1.13.0` situation,
except that one was fixed in a day and some are not. A human holding the release
with the advisory in front of them can weigh severity, reachability and whether a
fix exists. A workflow step cannot, and would be routed around the first time it
was wrong, which is worse than not having it.

**Q-3 — where does the model live?** `SECURITY.md` is in `.github/` and is not
part of the published book, so nothing in the documentation a consumer reads
mentions security at all — it appears in **2 of 92** book pages. **Suggest the
book**, under `reference/`, with `SECURITY.md` linking to it and keeping the
reporting instructions.

**Q-4 — does the SECURITY.md wording get fixed here or separately?** The
*"scripted server responses"* line is wrong for this project and is a two-minute
fix. **Suggest here**, since anyone reading the new model will arrive at
`SECURITY.md` next and the mismatch would be immediate.

## Acceptance criteria

1. The threat model exists, names its surfaces, and **names its non-goals** —
   a model without the second half is an unbounded document.
2. UI integrity has its own section, citing RFC-084 as the demonstrated case
   rather than describing the risk abstractly.
3. The supply-chain gate runs, and is **demonstrated failing** — pin a dependency
   with a known advisory, watch it refuse, restore. A security gate that has only
   ever been seen to pass is the defect this project has now found five times.
4. Whatever Q-2 rules, the RFC's own text records which way and why.
5. `SECURITY.md` no longer asks for reproducers this project cannot have.
6. CHANGELOG entry, or one line saying why not.
