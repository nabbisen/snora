# RFC-014-C — Theme Boundary and Warning Palette Policy

Status: Proposed  
Target release: v0.14  
Priority: Medium  
Type: Design boundary / documentation / minor cleanup

## 1. Summary

Document Snora as theme-aware but not theme-owning, and make the warning-color fallback policy explicit so it is not mistaken for a parallel theme system.

## 2. Motivation

Snora deliberately delegates theming to iced. Prefab chrome and toast intents consume iced `Theme` data, but Snora does not define a parallel token system. During review, the warning intent appeared to require a fallback color because the active iced palette may not expose a warning pair equivalent to other intents. This is acceptable, but it should be documented as a narrow compatibility fallback rather than a breach of the theme boundary.

## 3. Goals

- Preserve the non-goal of a Snora-owned theming layer.
- Document how prefab widgets and toast intents derive colors.
- Document any hard-coded fallback as an exception, not a new theme system.
- Provide a review checklist for future style changes.
- Avoid public theme APIs.

## 4. Non-Goals

- Do not add Snora theme tokens.
- Do not add a `SnoraTheme` struct.
- Do not add application theme configuration.
- Do not promise exact color stability across iced theme changes.
- Do not introduce custom CSS-like styling.

## 5. External Design

Public policy:

1. Applications theme iced normally.
2. Snora prefab widgets read iced `Theme` and use local style functions.
3. `ToastIntent` maps to semantic colors where iced provides them.
4. If iced lacks a semantic color pair, Snora may use a small documented fallback.
5. Fallbacks are implementation details, not public design tokens.

Documentation snippet:

```text
Snora is theme-aware, not theme-owning. It consumes iced's active Theme for prefab chrome and intent styling. It does not expose a separate theme model. A narrow fallback may exist for semantic intents not directly represented by iced's palette; such fallback is documented and may change with iced.
```

## 6. Internal Design

Recommended repository changes:

- Add comments in `crates/snora-widgets/src/style.rs` and/or `crates/snora/src/toast.rs` explaining fallback colors.
- Ensure fallback constants, if any, are private.
- Consider naming private fallback helpers clearly, e.g. `warning_fallback_pair`.
- Avoid exposing fallback colors through `snora::style` unless already public and unavoidable.

Style review checklist:

- Does the change add a public color/token type? If yes, reject or escalate.
- Does the change derive from iced `Theme` where possible?
- Does it add a dependency? If yes, evaluate feature-gating.
- Does it affect binary size? If yes, measure.

## 7. Testing and Acceptance

Acceptance criteria:

- All existing style tests/builds pass.
- If a fallback helper is factored out, add a small unit test only if it has nontrivial branching.
- Workbench example should show warning toast intent so the fallback path is visible.
- No public API exposes fallback color constants.

## 8. Documentation Updates

Update:

- `docs/src/contributing/design-decisions.md`
- `docs/src/reference/widgets.md`
- `docs/src/reference/vocabulary.md` near `ToastIntent`
- potentially README wording if theme-aware behavior is mentioned

The docs should avoid saying "Snora themes applications."

## 9. Compatibility and Migration

Compatible. This RFC is primarily documentation and private cleanup.

If fallback color details change, it should be a patch-level visual fix unless the public appearance changes dramatically enough to require a changelog note.

## 10. Open Questions

- Should warning fallback be centralized in one private helper?
- Should the docs mention exact fallback color values, or only the existence of fallback behavior?
- Should visual changes to fallback colors require changelog entries?
