# RFC-113 — A toast disappears before a slow reader finishes it

**Status.** Done — shipped v0.54.0. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/113-…`](../handoffs/113-a-toast-disappears-before-a-slow-reader-finishes-it/implementation-handoff.md).
**Raised.** 2026-11-04, architect, from the owner's question about older users (Theme
E). The owner approved opening it.
**Release target.** 0.54.0 (Q-5). **Priority.** A defect on the default path, for
anyone who reads slowly. That covers older users, people reading in a second language
and people with low vision using magnification. It is the most serious of Theme E's
three findings, because a message that vanishes cannot be recovered.

---

## The finding

**Every toast lasts 4 seconds unless the application says otherwise.**
`Toast::new` sets `ToastLifetime::DEFAULT`, which is
`Transient(Duration::from_secs(4))` (`crates/snora-core/src/toast.rs:164`). The same
4 s applies to every intent, every length of message and every reader.

**Nothing holds a toast while it is being read.** `sweep_expired` removes a toast once
`now − created_at ≥ duration`. Nothing in `crates/snora/src/toast.rs` watches the
pointer: no `mouse_area`, no `on_enter`. A reader who moves the pointer onto a toast
to read it still loses it at 4 s.

**An error is transient by default.** `ToastIntent::Error`'s doc says it is "often
paired with `Persistent`". That is advice, not a default: `Toast::new` gives an error
the same 4 s as a debug note.

**Why this fails older readers.**
- WCAG 2.2.1 (Timing Adjustable, level A) asks that a time limit set by content can
  be turned off, adjusted or extended. We read a toast that removes itself as such a
  limit.
  snora gives the application that control (`with_lifetime`, `persistent`), but
  gives the **reader** none.
- Reading speed falls with age and with low vision. And 4 s has to cover more than
  reading: noticing that a toast appeared, moving the eyes to its corner, and then
  reading it.

**What the reader cannot do today:** keep a toast, slow it down or get it back.

## Proposal

**R-1 — an error stays until it is dismissed.** `Toast::new` gives
`ToastIntent::Error` the lifetime `ToastLifetime::Persistent`. Missing an error costs
more than dismissing one. `with_lifetime` still overrides it.

**R-2 — the default duration follows the length of the text (Q-1).** For every other
intent, `ToastLifetime::DEFAULT` is replaced by a rule computed from the title and
message, stated in one sentence a reader can apply by hand:

> *A toast stays 2 seconds, plus 1 second for every 6 characters of its title and
> message, rounded up to a whole second, and never less than 6 seconds.*

- **Characters, not words,** because Japanese and Chinese text has no spaces: a word
  count would give a whole Japanese sentence one "word" and the 6 s floor. `chars()`
  counts code points, so it needs no new dependency. A combining mark counts as an
  extra character, which only ever makes a toast stay longer.
- **Worked cases:**

  | Toast | Characters | Duration |
  |---|---|---|
  | "Saved" / "Your changes are stored." | 29 | 7 s (2 + 29 ÷ 6 = 6.8, rounded up) |
  | 120 characters of English | 120 | 22 s |
  | 「保存しました」/「変更内容を保存しました。」 | 18 | 6 s (the floor; 2 + 3 = 5) |
  | 「保存できませんでした」/「ディスクの空き容量が不足しています。」 | 28 | 7 s (2 + 4.7, rounded up) |

- **The rule gets a name:** `ToastLifetime::reading_time(title, message)`.
  `Toast::new` calls it for every intent except Error.
- **`ToastLifetime::DEFAULT` is deprecated, and its value stays 4 s.** Changing the
  value would change what an existing name means. Deprecating it tells every caller
  who names it directly, through a compiler warning, that it is no longer what
  `Toast::new` gives, and points them to `reading_time` (the rule: mark superseded
  things deprecated).

**R-3 — a toast under the pointer does not expire (Q-2).** snora owns no state, so
this takes the application's help, in the same shape as `on_dismiss`:
- `AppLayout::on_toast_hover(f)`, where `f: Fn(u64, bool) -> Message`. snora wraps
  each toast in a `mouse_area` and sends `(id, true)` on enter and `(id, false)` on
  exit. `AppLayout` is `#[non_exhaustive]`, so this is additive.
- **Held:** the sweep skips the toast.
- **Released:** its countdown **starts again in full** from `now`. That is easier to
  predict than resuming from where it stopped, and it errs long.
- How hold state is kept is Q-3.

**R-4 — say what the keyboard cannot do.** A pause on keyboard focus needs to know
which widget has focus. That takes iced's `advanced` feature, and the owner has ruled
it must never be a default (RFC-078). So a keyboard-only reader has no hold. R-1 means
they cannot miss an error, and the docs tell applications to use `persistent()` for
anything a reader must not miss. This is a stated limit, not a silent one.

**R-5 — docs.**
- The toast guide states the rule and its worked cases, the hold, and the keyboard
  limit.
- `accessibility.md` gains a row for timing.
- The migration guide names who sees a change: everyone whose toasts use
  `Toast::new`'s default.

## Open questions

**Q-1 — the numbers.** **Suggest 2 s + 1 s per 6 characters, rounded up, with a
6 s floor.** These are **assumptions, not measurements.**
- **English:** adult silent reading averages about 238 words a minute (Brysbaert,
  2019), roughly 20 characters a second at about 5.7 characters a word. 6 a second
  leaves a wide margin for older readers.
- **Japanese and Chinese:** a character carries more, and is read more slowly per
  character. I have no verified figure. *Revised in review from 8 to 6 a second*,
  because the first rate left CJK text with little margin. The owner can judge the
  Japanese worked cases directly, and that judgement is better evidence than mine.
- **The extra 2 s** covers noticing the toast and finding it.
- **Rounded up**, so a reader can do the arithmetic and get the same answer.
- **Erring long is cheap,** because the reader can dismiss a toast. Erring short loses
  the message.

The timing is a better guess, nothing more. What the reader can rely on is R-1 and
R-3.

**Q-2 — hold on hover: in or out?** **Suggest in, and steer every application to
wire it** (revised in review). The hold is the only control the reader gets. But as
first drafted it did nothing until an application chose to wire it, so readers of the
applications that never did would get none: accessible by choice, not by default.
So:
- **`sweep_expired` is deprecated,** superseded by `sweep_expired_holding` (Q-3). Every
  application that sweeps toasts then gets a compiler warning pointing it to the hold,
  and the deprecation note shows the two lines of wiring.
- **Deprecated is not removed.** `sweep_expired` keeps working. An application that
  builds with `-D warnings` will stop compiling until it moves, which is the intent,
  and the migration guide says so first.

This is the rule "mark superseded things deprecated", applied so that ABDD reaches
the default.

**Q-3 — where the hold lives.** `Toast` is **not** `#[non_exhaustive]`, so a new field
breaks every application that builds one with a struct literal.
- **Suggest:** the application keeps the held ids in a set.
  - `snora::toast::sweep_expired_holding(&mut toasts, now, &held)` skips them.
    `sweep_expired` keeps its behaviour, deprecated (Q-2).
  - `snora::toast::restart(&mut toasts, id, now)` resets `created_at`, already
    public, on release. **It cannot reorder the stack:** order comes from the
    vector (`toast.rs:168`), not from `created_at`. Checked, because a stack that
    shuffled when the pointer left would be worse than no hold.

  All of it is additive, with no vocabulary change to `snora-core`.
- **The alternative:** mark `Toast` `#[non_exhaustive]` and add a field. That is a
  breaking change to core vocabulary, and costs gate 2's "two minors without churn".

**Q-4 — Warning.** **Suggest transient, by R-2's rule.** A warning is something to
notice, not something to acknowledge. An application that disagrees calls
`persistent()`.

**Q-5 — release.** **Suggest 0.54.0.** R-1 and R-2 change behaviour on the default
path. R-3 is additive. Two deprecations, `ToastLifetime::DEFAULT` and `sweep_expired`,
warn but break nothing, except builds that deny warnings.

## Alternatives considered

- **A key that holds every toast,** bound by the application through a
  `snora::keyboard` helper, like F6 (RFC-060). It would give keyboard users a hold
  without iced's `advanced`. **Not suggested:** no platform convention exists for
  such a key, so readers would not know to press it, and a control nobody discovers
  helps nobody. Revisit if an adopter asks.
- **A fixed, longer default,** such as 8 s for every toast. It is simpler, but too
  long for "Saved" and too short for a paragraph.

## Review against snora's principles

- **ABDD:** the default moves in the reader's favour (R-1, R-2), and the one thing
  that cannot be done (a keyboard hold) is stated, with a route around it (R-4).
- **Rules a reader can predict:** one sentence for duration, rounded up so the
  arithmetic is exact, and "starts again in full" for release.
- **ABDD, second pass:** the hold was opt-in. Deprecating `sweep_expired` makes every
  application meet it (Q-2). The character rate was lowered so CJK readers get a
  margin (Q-1).
- **Mark deprecated when superseded:** `DEFAULT` is deprecated rather than silently
  redefined. This was changed in review.
- **Verify claims:** the reading-speed figure is cited and labelled as an assumption
  (Q-1).

## What this is not

- Not a change to toast appearance, position or stacking.
- Not a new dependency.
- Not a hold for keyboard focus (R-4).

## Acceptance criteria

1. **Failing first:** a test that a default Error toast is still present after 60 s,
   and that a 120-character Info toast is present at 21 s (it is due to
   expire at 22 s). Both fail on today's code.
2. The R-2 rule, tested at the floor, just above it and on a CJK string, with the
   numbers matching the worked cases.
3. Hold: a held toast survives a sweep long past its deadline, and a released toast
   expires one full duration after release. Each assertion is shown failing with
   hold disabled.
4. CHANGELOG under Changed, with the rule's sentence quoted.

---

## Rulings, 2026-11-04

All suggestions accepted, as revised in `393c753`.

- **R-1:** Error is persistent by default.
- **Q-1:** 2 s + 1 s per 6 characters, rounded up, with a 6 s floor, as
  `ToastLifetime::reading_time`. `DEFAULT` is deprecated, and its value stays 4 s.
- **Q-2:** the hold is in. `sweep_expired` is deprecated in favour of
  `sweep_expired_holding`.
- **Q-3:** the application keeps the held ids; `restart` resets `created_at`. No
  `snora-core` vocabulary change.
- **Q-4:** Warning stays transient.
- **Q-5:** 0.54.0.

## Amendment, 2026-11-04 (owner-approved): the hold is one type

Q-3's ruling ("the application keeps the held ids; `restart` resets `created_at`")
asked every application to do two things in step: insert and remove ids, and call
`restart` on pointer exit. **Forgetting the second loses the toast.** Its deadline
passed while it was held, so the next sweep removes it the moment the pointer leaves,
which is the failure the hold exists to prevent.

**Amended:** `snora::toast::Holds` owns the held ids.
- `hover(id, entered, &mut toasts, now)` restarts the countdown itself on leave.
- `sweep(&mut toasts, now)` skips held toasts, and forgets ids whose toast is gone.
- `sweep_expired_holding` and `restart` are not public, so there is one way to do it.
- `sweep_expired`'s deprecation points to `Holds`.

Still no `snora-core` vocabulary change: `Holds` lives in the `snora` engine crate.
Reviewed against the owner's fundamental philosophy: the misuse can no longer be
written, and one type replaces two functions plus a convention.
