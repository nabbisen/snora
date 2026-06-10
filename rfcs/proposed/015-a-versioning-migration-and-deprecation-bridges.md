# RFC-015-A — Versioning, Migration, and Deprecation Bridges

> **v0.11 propagation note (2026-06-10).** This RFC's open question "should
> behavior changes like toast ordering be patch or minor when they fix
> documented semantics?" has a concrete precedent set in v0.11.0: the
> toast-ordering fix (RFC-011-B) was classified as a **bugfix** that
> restores the already-documented invariant, recorded under CHANGELOG
> **Fixed**, and shipped inside the v0.11.0 minor (alongside other minor
> work) rather than triggering a release on its own. When this RFC is
> adopted, that precedent should be written up as the rule: a fix that makes
> behavior match existing docs is **Fixed**, not **Changed**.

Status: Proposed  
Target release: v0.15  
Priority: High  
Type: Release engineering / public API discipline

## 1. Summary

Formalize how Snora handles pre-1.0 breaking changes, deprecation bridges, migration guides, and changelog language.

## 2. Motivation

The handoff already describes pre-1.0 SemVer practice: minor releases may carry small breaking changes, patch releases are bug-fix/additive only, and deprecation bridges have been used across prior releases. This RFC turns that practice into a maintained policy so downstream users can trust migration behavior before 1.0.

## 3. Goals

- Make breaking-change policy explicit.
- Define when a deprecation bridge is required.
- Define migration-guide expectations.
- Keep patch releases safe.
- Support eventual 1.0 freeze readiness.

## 4. Non-Goals

- Do not declare 1.0.
- Do not promise zero breaking changes before 1.0.
- Do not keep deprecated aliases forever.
- Do not add a compatibility layer for every internal change.
- Do not hide breaking changes as bug fixes.

## 5. External Design

Policy:

| Change type | Version level | Migration requirement |
|---|---|---|
| Bug fix, no API change | patch | changelog note if visible |
| Additive API | minor or patch depending on scope | docs update |
| Rename public type/method | minor | deprecation alias for two minor releases when practical |
| Remove public type/method | minor pre-1.0 only | migration guide required |
| Feature flag rename | minor | old feature bridge if possible, migration guide required |
| Behavior semantics change | patch if clear bug; otherwise minor | explicit changelog note |
| 1.0+ breaking change | major | full migration guide |

Migration guide template:

```markdown
# Migration x.y to x.z

## Who is affected
## What changed
## Why it changed
## Mechanical migration
## Behavioral migration
## Deprecated aliases and removal schedule
## Examples before/after
```

## 6. Internal Design

Repository changes:

- Add `docs/src/contributing/versioning-policy.md`.
- Link it from release process and design decisions.
- Add migration guide template under `docs/src/guides/migration-template.md` or contributor docs.
- Add release checklist item: "Does this PR need migration docs?"

Internal rule:

Any PR touching public vocabulary in `snora-core/src/lib.rs` or public re-exports in `snora/src/lib.rs` must answer:

1. Is this public API?
2. Is this additive, renaming, removal, or semantic change?
3. Does it need a deprecation bridge?
4. Does it need a migration guide?

## 7. Testing and Acceptance

Acceptance criteria:

- Versioning policy is linked from release docs.
- Changelog entries use consistent labels: Added, Changed, Deprecated, Removed, Fixed.
- A sample migration guide template exists.
- Public API freeze checklist references this policy.
- CI is not required for policy itself, but docs build must pass.

## 8. Documentation Updates

Update:

- `docs/src/contributing/release-process.md`
- `docs/src/contributing/design-decisions.md`
- `docs/src/guides/migrations.md`
- `CHANGELOG.md` conventions

The docs should explicitly say that pre-1.0 minor releases may break API but must do so transparently.

## 9. Compatibility and Migration

This RFC is compatible. It constrains future changes.

If adopted, all future breaking changes must comply. Existing historical releases do not need to be rewritten unless docs already describe them.

## 10. Open Questions

- Should deprecated aliases last exactly two minor releases or "at least two"?
- Should behavior changes like toast ordering be patch or minor when they fix documented semantics?
- Should a public API diff tool be introduced before 1.0?
