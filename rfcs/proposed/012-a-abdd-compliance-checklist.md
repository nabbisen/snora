# RFC-012-A — ABDD Compliance Checklist

Status: Proposed  
Target release: v0.12  
Priority: Medium-high  
Type: Accessibility discipline / Contribution process

## 1. Summary

Add an ABDD compliance checklist to Snora's contribution process. Every direction-sensitive feature, widget, or overlay
must demonstrate logical-edge correctness before merge.

## 2. Motivation

ABDD is one of Snora's strongest differentiators. However, direction correctness can regress through small changes:
using `Left`/`Right` instead of `Start`/`End`, forgetting RTL examples, or assuming breadcrumb/menu orientation.

A checklist makes ABDD repeatable rather than dependent on reviewer memory.

## 3. Goals

- Make layout-direction correctness a formal review gate.
- Clarify that Snora owns layout direction, not full i18n.
- Require examples/tests for LTR and RTL where relevant.
- Keep the checklist small enough to be used.

## 4. Non-Goals

- Do not add translation catalogs.
- Do not add locale/date/number formatting.
- Do not claim full screen-reader accessibility.
- Do not add vertical writing-mode support.
- Do not add a complete a11y audit framework.

## 5. External Design

Add a new contributing page:

```text
docs/src/contributing/abdd-checklist.md
```

Add a concise version to the PR template if the repository adopts one.

## 6. ABDD Checklist

```text
ABDD checklist for Snora changes

Scope:
[ ] Does this change position, align, order, mirror, anchor, or label a UI surface?
[ ] If no, explain why ABDD does not apply.

Logical direction:
[ ] Uses Start/End or LayoutDirection instead of hardcoded Left/Right when the concept is logical.
[ ] Documents any physical Left/Right use as intentionally physical.
[ ] Resolves Start/End consistently under LayoutDirection::Ltr and LayoutDirection::Rtl.

Public API:
[ ] New public names avoid LTR-only assumptions.
[ ] New enum variants are logical where appropriate.
[ ] Defaults are sensible under both LTR and RTL.

Examples and docs:
[ ] At least one example or doc snippet shows RTL behavior if the surface is direction-sensitive.
[ ] Docs say Snora handles layout direction, not full translation/localization.

Tests:
[ ] Pure logic tests cover LTR and RTL if possible.
[ ] Render-semantics tests cover mirroring if the behavior is in the engine.

Accessibility wording:
[ ] The change does not overclaim full accessibility or full i18n.
[ ] Tooltip/label text is required or documented where visual-only controls appear.
```

## 7. Internal Design

### 7.1 Docs Update

Add the checklist page and link it from:

- `docs/src/SUMMARY.md`;
- `docs/src/contributing/design-decisions.md`;
- `docs/src/guides/direction.md`;
- `docs/src/contributing/adding-an-overlay.md`.

### 7.2 Source Comments

For direction-sensitive helpers, use comments that distinguish logical and physical meaning:

```rust
// Resolve logical Start/End to the physical side required by iced layout.
```

Avoid comments such as “left sidebar” unless the side is intentionally physical.

### 7.3 Optional PR Template

If the repository wants a PR template, add:

```markdown
## ABDD

- [ ] This change is not direction-sensitive.
- [ ] This change is direction-sensitive and the ABDD checklist has been completed.
```

## 8. Testing Plan

- Existing `LayoutDirection` and `Edge` tests remain.
- Add tests for any direction helper introduced by future changes.
- Link the checklist from render-semantics testing RFC so RTL behavior becomes a test category.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Checklist becomes bureaucracy. | Keep it short; require only when direction-sensitive. |
| Contributors confuse ABDD with full i18n. | Include a repeated “layout direction only” statement. |
| Physical left/right becomes taboo even when correct. | Allow physical directions when explicitly documented. |

## 10. Acceptance Criteria

- `abdd-checklist.md` exists and is linked.
- Direction guide references the checklist.
- Overlay-adding guide references the checklist.
- Future direction-sensitive RFCs are expected to include an ABDD section.
