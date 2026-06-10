# RFC-014-B — Focus and Modal Accessibility Boundary

> **v0.11 propagation note (2026-06-10).** This RFC's caveat "do not add
> focus-related fields to `Dialog`/`Sheet` before resolving `AppLayout`
> construction stability" is now unblocked: RFC-011-C is decided/implemented
> in v0.11.0 (`#[non_exhaustive]`, builder-canonical). Any future focus
> vocabulary may be added additively when its design is proven. The
> documentation boundary this RFC defines also builds directly on the Law 8
> wording shipped in RFC-011-E (overlay-interaction-semantics) in v0.11.0.

Status: Proposed  
Target release: v0.14 documentation; implementation deferred  
Priority: Medium-high  
Type: Accessibility boundary / modal semantics

## 1. Summary

Define what Snora means by modal accessibility, what it does not yet guarantee, and how applications should handle focus, screen-reader labels, and close affordances.

## 2. Motivation

Snora is ABDD-oriented, but ABDD is a layout-direction discipline, not full accessibility. Dialogs and sheets create modal visual states, and users may expect focus trapping, initial focus, restore focus, Escape dismissal, and screen-reader semantics. If Snora overclaims, users will misunderstand the framework. If Snora underdocuments, apps may ship inaccessible modal flows. A clear boundary protects both users and maintainers.

## 3. Goals

- Clarify that Snora provides layout-direction correctness and overlay composition, not a complete accessibility stack.
- Define recommended application responsibilities for dialogs and sheets.
- Require visible close affordances when outside-click or Escape close is absent.
- Document focus-trap status honestly.
- Keep future focus support open without committing before iced APIs and real app needs are clear.

## 4. Non-Goals

- Do not implement focus trapping in this RFC.
- Do not add a screen-reader abstraction.
- Do not add localization, text shaping, or locale formatting.
- Do not claim WCAG conformance for applications using Snora.
- Do not add new modal vocabulary fields unless a later implementation RFC justifies them.

## 5. External Design

Add a public statement:

> Snora's accessibility contribution is structural and directional: logical placement, deterministic overlay layering, and examples that model clear close affordances. Full accessibility remains a responsibility shared with iced and the application.

Recommended application modal checklist:

- Dialog/sheet title is visible or programmatically represented by the app content.
- At least one explicit close/cancel action exists.
- `on_close_modals` is set when outside-click dismissal is intended.
- Escape dismissal is wired by the app if desired.
- Destructive modal actions require explicit labels and are not triggered by backdrop click.
- Focus behavior is considered by the app; Snora does not yet guarantee focus trapping.

Potential future vocabulary, not accepted now:

```rust
pub enum ModalAccessibilityPolicy {
    AppManaged,
    RequestInitialFocus,
    TrapFocus,
}
```

This future enum is intentionally not part of this RFC because the implementation route is not proven.

## 6. Internal Design

No runtime implementation is required.

If future work adds focus support, it should be designed after render-semantics tests exist. The likely placement would be in `crates/snora/src/overlay/` rather than `snora-core` until the vocabulary is stable.

Do not add focus-related fields to `Dialog` or `Sheet` before resolving `AppLayout` construction stability and public field extensibility.

Internal invariant to document:

- The modal dim/backdrop represents input blocking visually and by pointer capture.
- It does not by itself imply keyboard focus trapping.

## 7. Testing and Acceptance

Acceptance criteria:

- Documentation explicitly distinguishes visual modality, pointer blocking, keyboard dismissal, and focus trapping.
- Workbench example includes at least one dialog and one sheet with explicit close controls.
- Render-semantics tests verify pointer blocking, not focus trapping, unless future focus support is accepted.
- No documentation phrase says Snora is a complete accessibility framework.

## 8. Documentation Updates

Update or add:

- `docs/src/guides/overlays.md`
- `docs/src/guides/direction.md`
- `docs/src/reference/overlay-interaction-semantics.md`
- `docs/src/contributing/design-decisions.md`

Add a short phrase near ABDD documentation: "ABDD is a layout discipline, not a complete accessibility or localization stack."

## 9. Compatibility and Migration

Fully compatible if documentation-only.

Future focus API may be breaking if it changes modal vocabulary; therefore this RFC deliberately does not add it.

## 10. Open Questions

- Does iced provide stable enough focus primitives for Snora to eventually offer focus helpers?
- Should modal close-button guidance be a recommendation or an enforced helper in prefab examples?
- Should examples include accessibility comments even when they cannot enforce semantics?
