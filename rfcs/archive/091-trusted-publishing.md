# RFC 091 — Trusted Publishing, and a deferral with a date on it

**Status.** **Archived — superseded by RFC-090, 2026-09-02, the same day it
was opened.** Never accepted; nothing was deferred in the end.
See "Why this was archived" below.
**Tracks.** Release integrity / credentials. **Severity: Medium.**
**Found by** RFC-090's Q-2, ruled 2026-09-02. Opened at the moment of the
deferral, not after it.
**Touches.** `.github/workflows/release.yaml` (once RFC-090 creates it),
`docs/src/contributing/release-process.md`.
**Firing condition.** **The first release cut through `release.yaml`.** Not a
version, not a date — the event after which the open question can actually be
answered.

## Why this exists as a document rather than a good intention

RFC-090 ruled that `release.yaml` authenticates with a **scoped crates.io API
token**, not Trusted Publishing, because the credential mechanism is orthogonal
to the three properties RFC-090 buys and an unverified improvement should not
gate a certain one.

That is a deferral. **The audit's F-39 finding was that a conditional deferral
nobody re-checks is a permanent one** — RFC-073, RFC-074 and RFC-079 each
deferred the same three gates, each citing the previous deferral, for three
releases, until an outside auditor quoted our own `ci.yaml` back at us.

RFC-087 fixed those three gates. It did not fix the habit. This RFC is the habit
being fixed once: **the deferral is a numbered document with a firing condition
before the thing it defers has even shipped.**

## The open question, stated honestly

Trusted Publishing exchanges a GitHub OIDC token for a short-lived crates.io
token, removing the long-lived secret entirely. Better, if it fits.

**Confirmed** (implementer research, 2026-09-02, sources named in
`.git-exclude/review-request/090-q2-trusted-publishing-verification/`):
a Trusted Publisher Configuration is per crate, created in the crates.io web UI,
and only after that crate has been published manually once. All five snora
crates already clear that bootstrap.

**Not confirmed:** whether one OIDC exchange in one job authorizes all five
uploads of a `cargo publish --workspace` call, or whether each crate needs its
own authenticate-then-publish step. The implementer's inference — that
authorization is decided server-side per publish request against whichever
crate's configs match the token's repo/workflow/ref claims, so one token would
cover all five if all five list the same publisher — matches the shape of
`crates-io-auth-action`, whose inputs name no crate. **It was labelled an
inference and is recorded here as one.**

If the answer is "per-crate re-auth", `cargo publish --workspace` stops being
one command, and that is a real change to RFC-090's Unit 1 shape — which is
precisely why it was not worth guessing at before the workflow existed.

## Why the firing condition is the right one

After the first cut through `release.yaml`:

- the workflow exists, so the change is an edit rather than a design,
- the three refusals are proven, so a credential change cannot silently weaken
  them,
- one real publish has happened, so the workspace-upload sequence is observed
  rather than theorised,
- and the question is the *only* thing left open, which is the cheapest a
  question ever gets.

## Non-goals

- **Not a criticism of the token path.** A token scoped to five crates,
  `publish-update` only, with an expiry, behind a protected environment, is a
  considered trade and RFC-090 records it as one.
- **Not a reason to delay RFC-090.** If this RFC is what makes someone hold
  `release.yaml` back, it has done the opposite of its job.

## Open questions

**Q-1 — does one OIDC exchange cover a five-crate workspace publish?** The
question above. Answer empirically at the firing condition, not from
documentation; the documentation is a JS-rendered SPA that could not be fetched,
and the RFC text predates whatever shipped since.

**Q-2 — if it does not, is a per-crate publish loop acceptable?** It costs the
"one command, five crates" property RFC-090 deliberately preserved. Trading a
long-lived secret for that may still be right. Owner's call, with the evidence
in hand rather than in prospect.

**Q-3 — what happens to the API token if this lands?** Revoked, not left in the
repo secrets as a fallback. An unused publishing credential is strictly a
liability. (Compare RFC-090's Q-4 ruling on the local publish path: an exception
without a named condition becomes the default.)

## Acceptance criteria

1. Q-1 answered **empirically**, against a real or scratch publish, with the
   evidence recorded.
2. If adopted: `release.yaml` uses OIDC, the API token is revoked, and
   `release-process.md` stops describing it.
3. If not adopted: **this RFC is archived with the reason**, not left proposed.
   A permanently open RFC is the same failure as a permanently renewed deferral,
   wearing the other hat.


---

## Why this was archived

**RFC-090 re-ruled Q-2 and adopted Trusted Publishing directly.** This RFC
existed to keep a deferral honest; there is no longer a deferral to keep.

The first Q-2 ruling chose a scoped API token and pushed Trusted Publishing
here, with a firing condition, so that it could not quietly become permanent —
the F-39 failure this document was written against. Hours later the owner asked
why they should configure crates.io twice. They would not have; the two are
different one-time actions and the second retires the first. But the question
exposed the real defect: **the ruling had them create a credential we already
planned to delete**, to spare the implementer an uncertainty that Unit 1 and
Unit 2 never touch. All three refusals fire before any upload, so no credential
is needed until the first real publish — by which time `release.yaml` exists and
the workflow-filename binding this RFC's sequencing worried about is settled.

So the deferral was not made more honest by this document. It was unnecessary,
and the document made it look considered.

**This is the outcome criterion 3 named** — *"If not adopted: this RFC is
archived with the reason, not left proposed. A permanently open RFC is the same
failure as a permanently renewed deferral, wearing the other hat."* Written
before there was any reason to think it would fire within the day.

### What did not get lost

Q-1 — whether one OIDC exchange authorizes all five uploads of a `cargo publish
--workspace` call — **remains unanswered and is now RFC-090's**, recorded in its
Q-2 with the risk explicitly accepted and bounded to the publish step. The
implementer's research stands: per-crate configuration confirmed, one-token
coverage not confirmed, and the inference labelled as an inference.

Q-3 is moot: no token is created, so none needs revoking.
