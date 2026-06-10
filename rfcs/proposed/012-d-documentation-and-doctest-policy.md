# RFC-012-D — Documentation and Doctest Policy

Status: Proposed  
Target release: v0.12  
Priority: Medium  
Type: Documentation quality / Test policy

## 1. Summary

Define how Snora keeps mdBook prose code blocks, `snora-core` doctests, and `snora-widgets` examples valid over time.
The goal is to reduce documentation drift without turning docs into an expensive test burden.

## 2. Motivation

Snora's public surface is intentionally small and documentation-heavy. As vocabulary changes, prose snippets can silently
become stale. `snora-core` doctests are already useful, but widget examples require iced and may need a different policy.

A clear doctest policy prevents either extreme:

- untested docs that rot;
- overly strict docs tests that make examples painful to write.

## 3. Goals

- Decide when mdBook code blocks should be testable.
- Decide how `snora-widgets` examples should be validated.
- Keep docs readable for humans.
- Avoid freezing private implementation details through tests.

## 4. Non-Goals

- Do not require every snippet to compile.
- Do not add screenshot generation.
- Do not duplicate every example in docs.
- Do not add a public test helper crate.

## 5. Proposed Policy

### 5.1 Rust API Snippets Should Prefer Doctestable Form

If a snippet demonstrates vocabulary or pure API usage and can be compiled without a full iced app, make it a real Rust
code block or crate doctest.

Good candidates:

- `LayoutDirection::flipped()`;
- `ToastLifetime::seconds()`;
- `SheetSize::Ratio` behavior;
- `Edge` resolution helpers;
- menu/tab/breadcrumb action enums.

### 5.2 Application-Shape Snippets May Be `ignore`

If a snippet is intentionally partial, event-loop-shaped, or requires a full iced app, mark it `ignore` and make it
obviously illustrative.

Use:

```markdown
```rust,ignore
fn subscription(&self) -> Subscription<Message> { ... }
```
```

### 5.3 Widget Builder Samples Should Compile Somewhere

For `snora-widgets`, prefer one of these:

1. a small example app under `examples/`;
2. an integration test if iced test support is practical;
3. an `ignore` doc block plus a nearby checked example reference.

Avoid unchecked snippets that duplicate example code.

### 5.4 mdBook Test Adoption

Run `mdbook test docs` only after classifying code blocks. The first PR should not blindly turn it on and fail on every
partial snippet.

Adoption steps:

1. Audit code blocks in `docs/src`.
2. Mark partial snippets explicitly as `ignore`.
3. Convert pure snippets to testable Rust where practical.
4. Add `mdbook test docs` to CI only when the audit is complete.

## 6. Internal Design

### 6.1 Docs Audit Script

Optional helper:

```bash
rg '^```' docs/src
```

Classify each block:

| Classification | Markdown fence |
|---|---|
| must compile | `rust` |
| illustrative partial | `rust,ignore` |
| shell command | `bash` |
| plain layout diagram | `text` |
| TOML/YAML | `toml` / `yaml` |

### 6.2 CI Integration

After cleanup, add to `ci.yaml` docs job:

```yaml
- name: Test docs
  run: mdbook test docs
```

If `mdbook test` requires extra crate setup, document the limitation and defer.

### 6.3 Widget Example Validation

Ensure examples using widget builders are included in workspace checks. If examples are separate packages, they should be
workspace members or explicitly checked in CI.

## 7. Documentation Changes

Add:

```text
docs/src/contributing/documentation-test-policy.md
```

Include:

- code fence rules;
- when to use `ignore`;
- relationship between docs snippets and examples;
- how to run docs tests locally.

## 8. Testing Plan

- Run `mdbook build docs` before and after audit.
- Run `mdbook test docs` locally after classification.
- Ensure CI docs job remains green.
- Ensure `cargo test -p snora-core` still covers vocabulary doctests.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `mdbook test` becomes noisy. | Audit and classify before enabling CI gate. |
| Examples drift despite docs tests. | Compile examples in CI. |
| Snippets become less readable due to boilerplate. | Allow `ignore` for app-shaped snippets. |
| Too much duplication. | Link to examples rather than copying long code. |

## 10. Acceptance Criteria

- Documentation test policy page exists.
- Code blocks in docs are classified.
- `mdbook test docs` is either enabled or explicitly deferred with reasons.
- Widget-builder examples compile somewhere or are linked to compiling examples.
