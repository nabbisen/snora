# RFC-113 — A toast disappears before a slow reader finishes it

**Status.** Proposed.
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

> *A toast stays 2 seconds, plus 1 second for every 8 characters of its title and
> message, and never less than 6 seconds.*

- **Characters, not words,** because Japanese and Chinese text has no spaces: a word
  count would give a whole Japanese sentence one "word" and the 6 s floor. `chars()`
  counts code points, so it needs no new dependency. A combining mark counts as an
  extra character, which only ever makes a toast stay longer.
- **Worked cases:**

  | Toast | Characters | Duration |
  |---|---|---|
  | "Saved" / "Your changes are stored." | 29 | 6 s (the floor; 2 + 3.6 = 5.6) |
  | 120 characters of English | 120 | 17 s |
  | 「保存しました」/「変更内容を保存しました。」 | 18 | 6 s (the floor; 2 + 2.25 = 4.25) |

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

**Q-1 — the numbers.** **Suggest 2 s + 1 s per 8 characters, with a 6 s floor.**
These are **assumptions, not measurements.** Adult silent reading of English averages
about 238 words a minute (Brysbaert, 2019), roughly 20 characters a second at about
5.7 characters a word, spaces included. 8 a
second leaves room for slower readers and for scripts read more slowly per character.
The extra 2 s covers noticing the toast and finding it. If the owner or an adopter has
reading-speed data for their users, it beats these.

**Q-2 — hold on hover: in or out?** **Suggest in.** It is the only control the reader
gets, and it costs applications nothing until they wire the message. Without it, R-2
merely guesses better.

**Q-3 — where the hold lives.** `Toast` is **not** `#[non_exhaustive]`, so a new field
breaks every application that builds one with a struct literal.
- **Suggest:** the application keeps the held ids in a set.
  - `snora::toast::sweep_expired_holding(&mut toasts, now, &held)` skips them.
    `sweep_expired` itself is unchanged.
  - `snora::toast::restart(&mut toasts, id, now)` resets `created_at`, already
    public, on release.

  All of it is additive, with no vocabulary change to `snora-core`.
- **The alternative:** mark `Toast` `#[non_exhaustive]` and add a field. That is a
  breaking change to frozen core vocabulary, and costs gate 2's "two minors without
  churn".

**Q-4 — Warning.** **Suggest transient, by R-2's rule.** A warning is something to
notice, not something to acknowledge. An application that disagrees calls
`persistent()`.

**Q-5 — release.** **Suggest 0.54.0.** R-1 and R-2 change behaviour on the default
path; no code breaks. R-3 is additive.

## Review against snora's principles

- **ABDD:** the default moves in the reader's favour (R-1, R-2), and the one thing
  that cannot be done (a keyboard hold) is stated, with a route around it (R-4).
- **Rules a reader can predict:** one sentence for duration, and "starts again in
  full" for release.
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
   and that a 120-character Info toast is present at 16 s. Both fail on today's code.
2. The R-2 rule, tested at the floor, just above it and on a CJK string, with the
   numbers matching the worked cases.
3. Hold: a held toast survives a sweep long past its deadline, and a released toast
   expires one full duration after release. Each assertion is shown failing with
   hold disabled.
4. CHANGELOG under Changed, with the rule's sentence quoted.
