# RFC-013-C — Tooltip Vocabulary and Persistent Toast Helper

Status: Proposed discussion  
Target release: v0.13 if real signal appears  
Priority: Low-medium  
Type: Small API ergonomics

## 1. Summary

Discuss two small ergonomic candidates:

1. shared tooltip vocabulary;
2. a persistent-toast acknowledgement helper.

Both should remain gated by real usage. Neither is urgent.

## 2. Motivation

Snora should avoid broad widget growth, but small vocabulary/ergonomic additions may be justified when repeated patterns
appear. The handoff already identifies tooltip vocabulary as a near-term candidate triggered by a second consumer, and
persistent-toast acknowledgement as a maybe candidate triggered by recurring real use.

## 3. Goals

- Define acceptance criteria before adding either feature.
- Keep additions small and vocabulary-consistent.
- Avoid turning Snora into a widget library.
- Preserve ABDD and accessibility discipline.

## 4. Non-Goals

- Do not add rich tooltip widgets.
- Do not add hover timing/placement engine.
- Do not add toast action buttons beyond dismiss in this RFC.
- Do not add notification center/history.
- Do not add toast deduplication or replacement.

## 5. Candidate A — Tooltip Vocabulary

### 5.1 Current Situation

`SideBarItem.tooltip: String` is currently the only typed tooltip-like field. A shared type is not justified until at
least one more consumer needs tooltip semantics.

### 5.2 Trigger

Add shared tooltip vocabulary only when a second real consumer appears, for example:

- tab bar item tooltip;
- icon button helper tooltip;
- breadcrumb overflow tooltip;
- future popover/anchor docs requiring tooltip semantics.

### 5.3 Proposed Type

```rust
pub struct Tooltip {
    pub text: String,
    pub side: Edge,
}
```

Possible builder:

```rust
impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), side: Edge::End }
    }

    pub fn at(mut self, side: Edge) -> Self {
        self.side = side;
        self
    }
}
```

### 5.4 Design Notes

- `side` is logical `Edge`, not physical left/right.
- `text` remains plain string; no rich content.
- Empty text should be discouraged in docs.
- Do not implement hover rendering unless a real widget needs it.

### 5.5 Migration

If adopted, change:

```rust
pub tooltip: String
```

to:

```rust
pub tooltip: Tooltip
```

This is breaking unless bridged. Because Snora is pre-1.0, a minor release may carry this with a migration guide.
Alternative: add `tooltip: impl Into<Tooltip>` builders while preserving field type only if feasible.

## 6. Candidate B — Persistent Toast Acknowledgement Helper

### 6.1 Current Situation

Applications can already create persistent toasts:

```rust
Toast::new(id, ToastIntent::Success, title, body, on_dismiss).persistent()
```

This is sufficient. A helper is justified only if repeated examples show the same pattern.

### 6.2 Trigger

Add helper only if at least two examples or real apps repeat an acknowledgement pattern such as:

- export complete;
- background job failed;
- settings saved with manual dismiss;
- file operation requires acknowledgement.

### 6.3 Proposed API

Option 1: named constructor:

```rust
impl<Message: Clone> Toast<Message> {
    pub fn persistent_ack(
        id: u64,
        intent: ToastIntent,
        title: impl Into<String>,
        message: impl Into<String>,
        on_dismiss: Message,
    ) -> Self {
        Self::new(id, intent, title, message, on_dismiss).persistent()
    }
}
```

Option 2: no new API; improve docs only.

Recommended initial decision: docs only, unless repeated code becomes noisy.

## 7. Internal Design

If tooltip vocabulary is adopted:

- add `crates/snora-core/src/tooltip.rs`;
- export from `snora-core/src/lib.rs`;
- update `SideBarItem`;
- update `snora-widgets` sidebar builder/rendering;
- update docs and examples;
- add doctests for `Tooltip::new` and `.at`.

If persistent helper is adopted:

- modify `crates/snora-core/src/toast.rs` only;
- add doctest;
- update toast guide.

## 8. Testing Plan

Tooltip:

- `Tooltip::new` sets default side;
- `.at(Edge::Start)` updates side;
- sidebar examples compile;
- ABDD checklist completed.

Persistent helper:

- helper returns `ToastLifetime::Persistent`;
- fields match input;
- doctest compiles.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Tooltip type causes breaking change too early. | Wait for second consumer; provide migration guide. |
| Tooltip rendering scope expands. | Keep type as vocabulary only. |
| Toast helper adds API clutter. | Prefer docs unless repetition is clear. |
| Empty tooltip strings remain possible. | Document; optional future `NonEmptyText` is out of scope. |

## 10. Acceptance Criteria

- Do not implement either candidate without trigger evidence.
- If tooltip vocabulary is implemented, it uses logical `Edge` and has docs/tests.
- If persistent toast helper is implemented, it is a small additive constructor/helper with tests.
- Changelog explains why the addition was justified.
