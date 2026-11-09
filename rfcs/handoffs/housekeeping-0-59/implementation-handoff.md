# Implementation handoff — 0.59: remove the 0.56 deprecations

**Ruled.** The versioning policy: *a deprecated item is removed in the third minor after the
one that deprecated it.* The owner put it in 0.59's scope on 2026-11-09.
**Release target.** 0.59.0. **One commit, one short review.** It may start now.
**Branch.** `housekeeping-0-59`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## Remove (deprecated since 0.56.0, each note says "removed in 0.59.0")

| Item | Replacement |
|---|---|
| `AppLayout::on_toast_hover`, the method | `AppLayout::on_toast_attention` |
| `AppLayout`'s `on_toast_hover` field | `on_toast_attention` |
| `snora_core::ToastHover` (re-exported from `snora`) | `ToastAttention` (`PointerEntered`, `PointerLeft`; plus `FocusGained`, `FocusLost`) |
| `snora::toast::Holds::hover` | `Holds::attend` |

- **The "both callbacks set" path goes with them.** With only `on_toast_attention`, the
  rule that sent pointer crossings to it alone has nothing left to decide. Remove that
  code, its test (`with_both_callbacks_set_a_crossing_reports_once`) and the deprecated-path
  test (`on_toast_hover_alone_still_reports_the_pointer`). The `#[expect(deprecated)]`
  `Holds::hover` tests go too.
- **Any example or doc comment** that still names a removed item moves to its replacement.

## Check

- `grep -rn "on_toast_hover\|ToastHover\b\|Holds::hover\|\.hover(" crates examples` finds
  nothing outside history.
- **Every remaining `#[deprecated]` names its removal version.** One remains:
  `dismiss_on_escape`, removed in 0.60.0.
- **The hold still works by pointer and by keyboard:** `Holds::attend`'s tests, and
  `focus_on_a_toasts_close_control_is_reported`, pass unmodified.
- **CHANGELOG `[Unreleased]`:** **Removed:** the four items, each with its replacement,
  citing 0.56.0 and the rule.

## Acceptance criteria

All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, and the feature
matrix. Then the branch CI. The migration guide is the architect's.
