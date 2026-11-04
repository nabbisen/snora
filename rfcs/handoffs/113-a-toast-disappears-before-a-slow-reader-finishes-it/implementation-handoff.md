# Implementation handoff — RFC-113: a toast disappears before a slow reader finishes it

**RFC.** `rfcs/accepted/113-a-toast-disappears-before-a-slow-reader-finishes-it.md`
**Release target.** 0.54.0.
**Rulings (2026-11-04), all as suggested:** errors persistent; `reading_time` = 2 s +
1 s per 6 characters, rounded up, at least 6 s; hold on hover; `DEFAULT` and
`sweep_expired` deprecated; Warning transient.

**Order.** Commit before RFC-114. Both touch `CHANGELOG.md`, and RFC-115's Unit 0
measures after both.

**Touch list.**
- `crates/snora-core/src/toast.rs`;
- `crates/snora-core/src/layout.rs` (`AppLayout::on_toast_hover`);
- `crates/snora/src/toast.rs`, `crates/snora/src/render.rs` (and the design
  path's render, if it builds the toast layer separately);
- every workspace caller of `sweep_expired` or `ToastLifetime::DEFAULT`:
  - `examples/{rtl,starter,toast,workbench}/src/main.rs`;
  - `crates/snora/src/lib.rs` (doc examples);
  - the rustdoc in `crates/snora-core/src/layout.rs`;
- `CHANGELOG.md` (`[Unreleased]`).

**Not:** toast appearance, position, stacking or width. The architect writes
`docs/src/getting-started/04-toasts.md`, `accessibility.md`, `vocabulary.md` and
the migration guide.

## Already established

- **Today:** `Toast::new` sets `ToastLifetime::DEFAULT` = `Transient(4 s)` for every
  intent (`snora-core/src/toast.rs:164`, `:233`).
- **Stack order comes from the vector, not from `created_at`** (`snora/src/toast.rs:168`,
  where `into_iter().rev()` is chosen by position). So `restart` resetting
  `created_at` cannot reorder the stack. **Keep it that way**, and assert it (Unit 3).
- **`Toast` is not `#[non_exhaustive]`.** Add no field. `AppLayout` is, and it derives
  nothing, so a new field there is additive.
- **iced 0.14 tooltips and focus:** there is no way to hold on keyboard focus without
  `advanced`, which is ruled out (RFC-078). Do not attempt it.

## Unit 1 — failing-first

Write these against today's code and record that they fail:
1. a default **Error** toast built by `Toast::new` is not expired at `created_at + 60 s`;
2. a default **Info** toast with a 120-character title and message is not expired at
   `+ 21 s`;
3. a toast held by `sweep_expired_holding` survives a sweep at `+ 1 h`. This fails to
   compile today; record that, then stub the function to show it fails at run time.

## Unit 2 — R-1 and R-2 (`snora-core`)

- **`ToastLifetime::reading_time(title: &str, message: &str) -> ToastLifetime`** is
  `Transient(max(6, ceil(2 + n / 6)) s)`, where `n = title.chars().count() +
  message.chars().count()`. Use integer arithmetic, so the rounding is exact:
  `2 + n.div_ceil(6)`, floored at 6.
- **Its rustdoc quotes the rule's sentence** verbatim from the RFC, and the four
  worked cases as a doctest:
  - 29 characters → 7 s;
  - 18 characters (the Japanese case) → 6 s;
  - 28 characters (the second Japanese case) → 7 s;
  - 120 characters → 22 s.
- **`Toast::new`:**
  - Error gets `Persistent`;
  - every other intent gets `reading_time(&title, &message)`, computed after the
    `Into<String>` conversions.
- **`ToastLifetime::DEFAULT`:** `#[deprecated(since = "0.54.0", note = "…")]`. The
  value stays `Transient(4 s)`. The note says that `Toast::new` no longer uses it,
  and names `reading_time`.
- **Rustdoc:**
  - `Toast::lifetime`'s field doc ("Defaults to `ToastLifetime::DEFAULT` (4-second
    transient)") and `Toast::new`'s doc now state the new defaults.
  - `ToastIntent::Error`'s "often paired with `Persistent`" becomes "persistent by
    default".

## Unit 3 — R-3, the hold (`snora` and `AppLayout`)

**`AppLayout::on_toast_hover(f: fn(u64, bool) -> Message)`:** a `#[must_use]`
builder, stored as `Option<fn(u64, bool) -> Message>`. A function pointer fits the
usual shape, `Message::ToastHover`. If you find a reason to take a closure instead,
say why.

**The engine:**
- when `on_toast_hover` is set, wrap each rendered toast in `mouse_area`, with
  `on_enter(f(id, true))` and `on_exit(f(id, false))`;
- when it is not set, the tree is unchanged: **no `mouse_area`**, so
  `render_semantics.rs` passes unmodified for applications that don't opt in.

**`snora::toast`:**
- `sweep_expired_holding(toasts: &mut Vec<Toast<M>>, now: Instant, held: &HashSet<u64>)`
  removes what `sweep_expired` would, except ids in `held`. Take whichever set type
  you judge best, as long as the doc names it.
- `restart(toasts: &mut [Toast<M>], id: u64, now: Instant)` sets that toast's
  `created_at = now`. It does nothing if `id` is absent.
- `sweep_expired`: `#[deprecated(since = "0.54.0", note = …)]`, with its behaviour
  unchanged. **The note shows the wiring in two lines:** `on_toast_hover`, and the
  `held` set with `restart` on exit.

**Workspace callers** move to `sweep_expired_holding`. At least one example (`toast`)
wires the hover and holds properly. The others may pass an empty set, with a comment
saying so. `cargo clippy -D warnings` must pass, which it cannot while any caller
still uses the deprecated items.

**Assert, in `snora/src/toast.rs` tests:**
- held across a sweep long past the deadline: survives;
- `restart` then sweep at `+ (duration − 1 ms)`: survives; at `+ duration`: gone;
- `restart` does not change the order of the vector (ids before = ids after).

## Unit 4 — the subscription

`subscription` wakes only while a transient toast exists. With Error now persistent
by default, check that an application holding only persistent toasts gets no ticks,
and that a held transient toast still keeps the subscription alive. A held toast must
expire on time after `restart`, and that needs a tick.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| Error persistent | revert `Toast::new` to `DEFAULT` for Error |
| `reading_time` worked cases | use `/ 8` instead of `div_ceil(6)`: the 29-character case gives 6, not 7 |
| floor | remove the `max(6, …)`: the 18-character case gives 5 |
| hold survives | make `sweep_expired_holding` ignore `held` |
| restart expires on time | make `restart` a no-op |
| restart keeps order | make `restart` move the toast to the back |
| no `mouse_area` when unwired | wrap unconditionally: `render_semantics.rs` must fail. **If it does not fail, say so**, and add an assertion that does |

## Acceptance criteria

1. Unit 1 shown failing, then every Unit 2 to 4 assertion passing. Each breakage above
   shown failing.
2. The doctest's four worked cases match the RFC's table exactly.
3. `render_semantics.rs` passes unmodified.
4. Clippy `-D warnings` is clean, with no `#[allow(deprecated)]` added outside a test
   that exists to exercise a deprecated item.
5. CHANGELOG:
   - Changed: the rule's sentence quoted, and errors persistent;
   - Added: `reading_time`, `on_toast_hover`, `sweep_expired_holding`, `restart`;
   - Deprecated: `DEFAULT`, `sweep_expired`.

   Do not write the migration guide.
