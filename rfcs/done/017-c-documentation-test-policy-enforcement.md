# RFC-017-C — Documentation Test Policy Enforcement

**Status.** Implemented (v0.17.0)
**Tracks.** Documentation hygiene.
**Touches.** `crates/snora/src/keyboard.rs` (fence classifier fix).

## 1. Gap

The `dismiss_on_escape` doc comment in `keyboard.rs` used `ignore` (bare)
rather than `rust,ignore` as required by RFC-012-D documentation test
policy. A scan of all post-v0.12 documentation confirmed no other bare
`rust` fences exist.

## 2. Fix

Change `/// ` ` ` `ignore` to `/// ` ` ` `rust,ignore` in `keyboard.rs`.

## 3. Acceptance criteria

- No bare `ignore` (without `rust,`) in any doc comment or mdBook fence.
- `cargo test -p snora --lib` continues to pass (2 doctests ignored as
  intended, 0 newly broken).
