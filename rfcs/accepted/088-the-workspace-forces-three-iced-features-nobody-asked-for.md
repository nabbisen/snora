# RFC 088 — The workspace forces three iced features, one used only under a feature that already enables it

**Status.** Accepted (owner, 2026-09-01). Handoff written — see
[`handoffs/088-…`](../handoffs/088-the-workspace-forces-three-iced-features-nobody-asked-for/implementation-handoff.md).
**Tracks.** Dependency layering / build cost. **Severity: High.**
**Found by** the external audit, 2026-09-01 (F-26). Usage verified by the
architect before opening.
**Touches.** `Cargo.toml` (one line).
**Release target.** **0.42.0** — **minor.** It changes what a consumer's build
resolves. *(Corrected 2026-09-02: this header said 0.41.0, which shipped without
this RFC, as did 0.41.1. The handoff has said 0.42.0 throughout — the
implementer flagged the discrepancy rather than picking one silently, and was
right to build against the handoff. Second time an RFC header has gone stale
this way; RFC-085's did the same.)*

## This is RFC-083 again, on the next line of the same file

`Cargo.toml:55`:

```toml
iced = { version = "0.14", features = ["canvas", "svg", "tokio"] }
```

RFC-083 removed `lucide-icons`' unused `iced` feature from **line 56**. Nobody
read line 55. Verified now:

| feature | used in `crates/`? |
|---|---|
| `canvas` | **zero occurrences.** Nothing draws a canvas |
| `svg` | only `snora-widgets/src/icon.rs:45`, **and only under our own `svg-icons` feature — which already declares `"iced/svg"`** (`crates/snora/Cargo.toml:59`) |
| `tokio` | **no direct use.** It forces an executor choice on every consumer |

So `canvas` is dead, `svg` is **redundant and ungated** — the gated path already
turns it on, and the workspace line turns it on for everyone who did not ask —
and `tokio` is a policy decision made silently on the consumer's behalf.

The audit measures the first alone at **−1.83 MiB and −34 crates** on every
consumer's build.

## What RFC-083 should have taught us and did not

RFC-083's own finding was *"a workspace feature nobody uses, dragging in a
dependency tree"*. It fixed the instance it was handed and **did not sweep the
adjacent line for the same shape.** The gate it added guards `snora-core`
against iced specifically; it says nothing about features iced itself is asked
for.

**One RFC per instance is how a class survives.**

## Non-goals

- **No change to `svg-icons`' behaviour.** Consumers who enable it keep working
  identically — `"iced/svg"` is already declared there.
- **No new feature.** If someone needs `canvas` or `tokio`, they enable it on
  their own iced dependency; that is what feature unification is for.
- **Do not touch line 56.** RFC-083 settled it.

## Open questions

**Q-1 — is `tokio` removable, or does something need an executor?** iced needs
*an* executor; `tokio` is one choice. **Measure whether the workspace builds and
the examples run without it** before removing — and if something does need it,
gate it behind a feature rather than forcing it.

**Q-2 — is this breaking?** A consumer relying on `iced/canvas` or `iced/tokio`
arriving transitively stops compiling — the identical shape RFC-083 treated as
breaking. **Suggest the same treatment**, for consistency and because the
precedent is one release old.

**Q-3 — should the gate generalise?** RFC-083's gate asserts a crate has no
iced. The general property here is *"the workspace enables no iced feature no
crate uses"*, which is derivable: for each feature, grep. **Suggest yes** — it
is the mechanism that would have caught this on the day RFC-083 shipped.

## Acceptance criteria

1. `canvas` removed; `svg` removed from the workspace line and confirmed still
   reaching `svg-icons` consumers; `tokio` resolved per Q-1.
2. **Before/after dependency count and binary size stated** — the audit's
   −1.83 MiB / −34 crates re-derived, not quoted.
3. Q-2's breaking-change treatment: migration guide names the reliance.
4. Q-3 ruled; if a gate is added, a perturbation demo proves it fires.
5. `cargo test --workspace --all-features` and the examples unaffected.

## Compatibility and security

**Compatibility.** Removes transitively-enabled features. **Minor**, with a
guide. **Security.** Removes 34 crates from the default dependency tree — a
supply-chain reduction, not a risk.
