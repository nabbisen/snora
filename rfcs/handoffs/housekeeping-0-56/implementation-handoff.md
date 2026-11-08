# Implementation handoff — 0.56 housekeeping

**Release target.** 0.56.0. **Scope.** Three small items the owner accepted as part of
0.56's core on 2026-11-08. None needs an RFC. Each is its own commit; one review covers
all three. **They may start now,** in parallel with RFC-120, which awaits rulings.
**Branch.** `housekeeping-0-56`, off `main`.

## 1. Examples: drop `dismiss_on_escape` where RFC-119 made it redundant

From 0.55, the engine answers Escape for every overlay with a click-outside close
message (`on_close_modals`, `on_close_menus`), and captures it. So the application's
own `keyboard::listen()` never sees that Escape.

- `examples/starter/src/main.rs:150`
- `examples/workbench/src/main.rs:197`

**Remove the call only where it is redundant:** every overlay it covers must have its
click-outside close message set. If an overlay in the example has none, the call is
RFC-119's legitimate remaining case. Keep it there, with a comment that says so. State
which case each example is in.

- Update `starter`'s module doc (`:12`), and the book's starter page, which the
  architect edits once you report.
- **Check:** with the call removed, Escape still closes the example's dialog and menu.
  `crates/snora/tests/modal_keyboard.rs` already covers the engine side. For the
  example, a test or a recorded run is enough. Say which.

## 2. CI: `snora-control`'s own pixel tests on wgpu too

`snora-control`'s tests include rendered ones, `Inert`'s and `Underlined`'s. The wgpu
job runs only `cargo test -p snora --all-features --tests`, so in CI they run on
tiny-skia alone (RFC-119 Unit 0 noted this).

- Add `snora-control` to the `rendered-wgpu` job, with `--test-threads=1`, as the job's
  own comments explain.
- Keep its own step, or extend the command; your choice. Say which.
- **Check:** the branch CI shows the `snora-control` tests running in the wgpu job,
  with their count.

## 3. The real-window Tab check under native Wayland (RFC-116 L-6)

RFC-116 Unit 3's real-window check ran under X11 (`WINIT_UNIX_BACKEND=x11`). Repeat it
under native Wayland: the `tab` example, then Tab, Space, Shift+Tab, and Tab past the
last control.

- **If synthetic input cannot reach a native Wayland window** on your machine, say so,
  with what you tried. `docs/src/reference/rendered-surface-identifiers.md` records an
  earlier attempt under niri. **Do not substitute XWayland and call it Wayland.**
  The owner may then do it by hand.
- Record what you saw, as in Unit 3: frames and a montage.

## Acceptance criteria

1. Each item as above, with what you could and could not do stated.
2. All of `ci.yaml`'s Rust quality gate commands, and the branch CI.
3. CHANGELOG `[Unreleased]`: nothing for items 2 and 3, which change no user-visible
   behaviour. For item 1, a line only if an example's behaviour changes.
