# RFC-015-A — Versioning, Migration, and Deprecation Bridges

**Status.** Implemented (v0.15.0)
**Tracks.** Release engineering / public API discipline.
**Touches.** `docs/src/contributing/versioning-policy.md` (new),
`docs/src/guides/migration-template.md` (new),
`docs/src/contributing/release-process.md`,
`docs/src/guides/migrations.md`.

> v0.11 propagation (carried from filing): toast-ordering fix sets the
> "fix that restores documented invariant → Fixed" precedent. This RFC
> codifies it as the rule.

## 1. Decisions

### Open question: "at least two" minor releases for deprecated aliases

Adopted: **at least two** consecutive minor releases. "Exactly two" is too
rigid when a deprecation bridge is load-bearing for downstream apps that
move slowly. The removal commit must cite the minor where the bridge was
added and the minor that is removing it.

### Open question: behavior change classification

Adopted: a fix that makes behavior match *already-published* docs is
**Fixed**, not **Changed**. A change that updates the documented contract
is **Changed**. The toast-ordering fix in v0.11 is the canonical example.

### Open question: public API diff tool before 1.0

Deferred: `cargo-semver-checks` is a reasonable candidate but requires
a baseline. Document the intent in the versioning policy; decide when
the first downstream app provides a baseline worth checking.

## 2. deliverables

- `docs/src/contributing/versioning-policy.md` — full policy table,
  "Fixed vs Changed" rule, deprecation bridge rule, and four questions
  any PR must answer when touching public API.
- `docs/src/guides/migration-template.md` — migration guide template with
  all seven sections.
- `docs/src/contributing/release-process.md` — add "Does this PR need
  migration docs?" to checklist.
- `docs/src/guides/migrations.md` — link to the template and to the policy.

## 3. Acceptance criteria

- `versioning-policy.md` exists and is linked from release-process.
- Migration guide template exists and is linked from migrations.md.
- Release checklist has the "need migration docs?" question.
- CHANGELOG conventions are consistent with the policy.
