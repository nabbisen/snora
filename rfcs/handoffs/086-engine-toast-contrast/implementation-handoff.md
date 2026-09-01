# Developer Handoff — RFC-086 engine toast contrast

**Governing RFC.** **RFC-086** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Accepted (owner, 2026-09-01). High.
**Release target.** **0.42.0** — **minor.** Rendered appearance changes.
*(Re-targeted 2026-09-01: this said 0.41.0, and 0.41.0 shipped without it. The
release carried RFC-084 and RFC-085 instead, so a guide named
`migration-0.40-to-0.41.md` already exists and is not yours. Write against
0.42.0.)*
**Implementation units.** One. Ships with RFC-084.

---

## 1. Measure all five before changing one

The audit reports `Warning` at **3.18:1** and the `Debug` dismiss `×` at
**1.58:1**. **Those are its figures, not ours.** The last audit's arithmetic
differed from our own once (F-29: 68 vs 57), and this project's rule is that a
figure you did not derive is one you cannot defend.

**Q-3 ruled: measure all five intents, both themes, and report the passes as
results.** Three intents nobody has measured are not three intents that pass.

## 2. The two known defects

- **F-05** `toast.rs:204` — `ToastIntent::Warning => (WARNING_COLOR,
  Color::WHITE)`. A hard-coded fill with white text.
- **F-06** `toast.rs:199` — `ToastIntent::Debug => (ep.background.strong.color,
  ep.background.strong.text)` for the body, but the dismiss `×` takes the same
  tier for both mark and fill.

## 3. The rulings

**Q-1 — darken the fill or change the text?** Measure both and choose on the
number. **State which identity property you traded**: darkening `WARNING_COLOR`
keeps the intent's colour identity; switching to near-black text changes how it
reads at a glance and may collide with `Success`/`Danger`. Either is defensible;
choosing silently is not.

**Q-2 — `Debug`'s `×` is a pairing fix, not a colour fix.** `background.strong`
already carries its own `.text`; use it. This is the same category error as
RFC-085's F-13 — a tier's `.color` used where its `.text` belongs — and fixing
it the same way keeps one lesson rather than two patches.

## 4. Why this is not part of RFC-085

RFC-085 is `snora-widgets` and its unreachable-by-tests problem. **This is the
engine**, on the default path with no features at all. Different crate,
different cause, different fix. **Do not merge the two suites** — the engine has
no token dependency and must keep none.

## 5. The assertion is the durable half

This will be **the engine's first contrast assertion.** Derive it from the
`ToastIntent` enum — exhaustively, so a sixth intent cannot be added without a
threshold — rather than listing five cases. `Palette::usages` is the pattern.

## 6. Explicit non-change scope

- **No new `ToastIntent`.**
- **No token dependency in the engine.** The `design` boundary is the point.
- **Do not route through `snora-design`.** If a fix needs a token, stop and
  report — that is a design change, not this.

## 7. Required evidence

- All five intents × both themes, before and after, your own figures
- Q-1's trade stated
- The assertion failing before the fix (perturb one intent), then passing
- `cargo test -p snora` green; appearance change recorded

## 8. Acceptance criteria

1. Five intents measured, both themes, figures stated — passes reported too.
2. `Warning` text clears 4.5:1; `Debug`'s `×` clears the non-text floor.
3. An exhaustive assertion over `ToastIntent` exists and fails on perturbation.
4. Q-1's trade recorded.
5. No new intent, no token dependency.
6. Migration note: appearance change, reference images invalidated.

## 9. Required review-request format

`.git-exclude/review-request/086-engine-toast-contrast/`, `README.md` entry
point, evidence under `evidence/`.

**Requested review focus: the three intents nobody has measured.** The two known
failures are arithmetic. Whether the other three pass is unknown, and an
unmeasured pass is not a pass.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
