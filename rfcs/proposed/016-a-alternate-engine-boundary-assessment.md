# RFC-016-A — Alternate Engine Boundary Assessment

Status: Proposed  
Target release: v0.16+ design assessment only  
Priority: Low-medium  
Type: Strategic architecture assessment

## 1. Summary

Assess whether the iced-free `snora-core` boundary has practical alternate-engine value, or whether it should be treated mainly as an isolation layer for iced upgrades and testing.

## 2. Motivation

The crate architecture keeps `snora-core` independent of iced. This is valuable even if no alternate engine ever exists: it protects vocabulary from iced churn and makes logic easier to test. However, documents may imply that another engine could drive the same vocabulary. That possibility should be assessed honestly before it becomes an accidental promise.

## 3. Goals

- Clarify the value of iced-free vocabulary.
- Avoid promising WGPU/HTML/other engine support without evidence.
- Identify what an alternate engine would actually require.
- Protect 1.0 messaging from portability overclaim.
- Keep the current crate boundary regardless of outcome.

## 4. Non-Goals

- Do not implement an alternate engine.
- Do not introduce a renderer abstraction trait.
- Do not make `snora-core` generic over rendering details.
- Do not support web/HTML as a roadmap item.
- Do not weaken iced integration for a hypothetical future.

## 5. External Design

Assessment questions:

| Question | Evidence needed |
|---|---|
| Can `AppLayout` vocabulary map to another renderer? | Prototype or design sketch. |
| Are overlay semantics renderer-independent? | z-stack/backdrop/dismissal mapping. |
| Are widgets portable? | likely no; `snora-widgets` is iced-specific. |
| Would alternate engine users accept missing prefab widgets? | downstream signal required. |
| Does portability justify API constraints? | only if concrete demand exists. |

Recommended public wording:

> `snora-core` is iced-free to keep vocabulary stable, testable, and insulated from iced upgrades. It may be useful for alternate engines in the future, but Snora does not currently promise alternate renderer support.

## 6. Internal Design

Optional assessment artifact:

- Add `docs/src/contributing/alternate-engine-boundary.md`.
- Include a non-code mapping table from `AppLayout` concepts to renderer requirements.

Required renderer capabilities:

- compose base skeleton;
- stack overlays deterministically;
- capture pointer events on backdrop layers;
- position logical start/end surfaces under LTR/RTL;
- manage transient toast sweep outside rendering;
- accept app-supplied content nodes.

If a tiny prototype is ever attempted, it should live outside main crates or behind an experimental folder, not in public API.

## 7. Testing and Acceptance

Acceptance criteria for assessment:

- Document clearly states no alternate engine is promised.
- No new public abstraction is added.
- The assessment identifies which parts are portable and which are iced-specific.
- Public README wording avoids overclaiming.

## 8. Documentation Updates

Update:

- architecture docs
- crate-level docs
- design decisions
- README if it mentions alternate-engine potential

Use conservative wording.

## 9. Compatibility and Migration

Compatible.

The outcome may influence 1.0 messaging but should not change code.

## 10. Open Questions

- Is alternate-engine potential worth mentioning publicly at all?
- Would a test-double engine help internal testing, or would iced_test/headless tests be enough?
- Are there downstream users who want the vocabulary without iced?
