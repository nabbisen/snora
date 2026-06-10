# RFC-015-C — Starter Application Template

Status: Proposed  
Target release: v0.15  
Priority: Medium  
Type: Adoption aid / example tooling

## 1. Summary

Create a starter application template that demonstrates the recommended Snora app structure without adding runtime framework scope.

## 2. Motivation

Snora is intentionally not a batteries-included application framework, but new users still need a clear starting point: Message enum, state, update, view, `AppLayout`, close sinks, toasts, and direction toggle. A starter template can reduce adoption friction while preserving the runtime library boundary.

## 3. Goals

- Provide a clean app skeleton for new projects.
- Demonstrate recommended file layout and message routing.
- Include optional toasts and RTL toggle.
- Keep template separate from Snora runtime crates.
- Avoid implying Snora owns domain architecture, persistence, or async job orchestration.

## 4. Non-Goals

- Do not add a project generator binary to the main crate.
- Do not create a full application framework.
- Do not prescribe database, settings, routing, or persistence choices.
- Do not include large assets.
- Do not depend on unstable external scaffolding.

## 5. External Design

Possible distribution options:

### Option A — `examples/starter`

A copyable example inside the repository.

Pros: simple, no new repo/tool.  
Cons: users must copy manually.

### Option B — separate `snora-template` repository

A `cargo generate` compatible template.

Pros: best user experience.  
Cons: maintenance overhead, separate versioning.

### Option C — documentation-only starter chapter

A guide that builds the starter step by step.

Pros: no maintenance overhead.  
Cons: less convenient.

Recommended initial path: Option A + Option C. Defer separate template repo until there is user demand.

Template structure:

```text
src/
  main.rs
  app.rs
  message.rs
  view.rs
  toast_state.rs
```

The template should demonstrate Snora, not architecture maximalism.

## 6. Internal Design

Repository changes:

- Add `examples/starter/` or `examples/starter.rs` depending on existing example conventions.
- Add a docs chapter: `docs/src/getting-started/06-starter-application.md`.
- Keep all code using public `snora` API.
- Avoid extra dependencies beyond iced and snora unless already required by examples.

Starter state sketch:

```rust
struct App {
    direction: LayoutDirection,
    toasts: Vec<Toast<Message>>,
    menu_open: bool,
    modal_open: bool,
}
```

Message sketch:

```rust
enum Message {
    ToggleDirection,
    OpenMenu,
    CloseMenus,
    OpenDialog,
    CloseModals,
    ToastTick(Instant),
    DismissToast(u64),
}
```

## 7. Testing and Acceptance

Acceptance criteria:

- Starter builds in CI.
- Starter demonstrates close sinks and toast sweep.
- Starter has comments explaining which parts are Snora and which are app-owned.
- Starter remains small enough for a new user to read quickly.
- No starter code reaches into private/internal modules.

## 8. Documentation Updates

Update:

- getting-started summary
- installation guide
- examples README
- "When to use Snora" guide

Docs must state that the starter is a recommended pattern, not a required architecture.

## 9. Compatibility and Migration

Compatible.

If starter evolves, treat it like documentation/examples, not public API. Still avoid churn because users may copy it.

## 10. Open Questions

- Should starter use prefab widgets or custom iced elements?
- Should starter include async/background task examples, or would that imply too much app-framework scope?
- Should a separate template repo be created after first downstream adoption?
