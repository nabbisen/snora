# Adding a new overlay kind

Use this page when you want to add an overlay surface that does not
fit `Dialog`, `Sheet`, or `context_menu`. Examples that have come up
in discussion (none implemented yet): a **command palette** centered
like a dialog but with `Escape` to close, an **anchored popover**
attached to a specific widget, an **inline banner** that drops in
from the top of the body region rather than over the whole window.

Note that side panels and edge-anchored drawers are *already*
covered by [`Sheet`] — choose `SheetEdge::Start` / `End` rather
than introducing a separate "drawer" type.

[`Sheet`]: ../guides/overlays.md#sheet

## Decision tree first

Before writing code, ask:

1. **Is this really a new overlay, or could it be a `Dialog` with
   different inner content?** A command palette is often best built
   as a `Dialog` whose `content` is your search input + result list.
   You get the dim layer and the `on_close_modals` plumbing for
   free, and there is no new vocabulary.
2. **Is this a `Sheet` at a non-default edge?** If so, just use
   `Sheet::new(...).at(SheetEdge::Top)` (or `Start`/`End`). No new
   overlay needed.
3. **Is it modal or transient?** Modal → a sibling of `Dialog` /
   `Sheet`. Transient → a sibling of `header_menu` / `context_menu`.
4. **Does it have configuration that does not fit existing
   vocabulary?** A command palette has search history, a popover
   has an anchor element. New configuration is the strongest
   reason to introduce a new type.

If you can answer "use a `Dialog`" or "use a `Sheet` at a different
edge" to (1) or (2), stop here.

## Steps if you do need a new overlay

### 1. Add the data type to `snora-core`

Place it in `src/overlay.rs` next to `Dialog` and `Sheet`. Keep
the same shape:

```rust
pub struct CommandPalette<Node, Message> {
    pub content: Node,
    pub recent_count: usize,
    _marker: PhantomData<Message>,
}

impl<Node, Message> CommandPalette<Node, Message> {
    pub fn new(content: Node) -> Self { /* sane default */ }

    #[must_use]
    pub fn with_recent_count(mut self, n: usize) -> Self {
        self.recent_count = n;
        self
    }
}
```

Notes:

- No `on_close` field — outside-click dismissal is wired via
  `AppLayout::on_close_modals` (modal) or `on_close_menus`
  (transient). Keep the rule consistent across overlay kinds.
- The `_marker: PhantomData<Message>` keeps the `Message` type
  parameter available for future expansion (animations, lifecycle
  hooks) without being a breaking change.

### 2. Add any new vocabulary enums

If your overlay has configuration that does not fit a primitive
(palette mode, popover anchor type), add a small enum next to the
struct. Use *logical* terms (`Start` / `End`) for axis-aligned
variants — never `Left` / `Right` directly.

### 3. Add an `AppLayout` field + builder method

In `snora-core/src/layout.rs`:

```rust
pub struct AppLayout<Node, Message> {
    // existing fields...
    pub command_palette: Option<CommandPalette<Node, Message>>,
}

impl<Node, Message: Clone> AppLayout<Node, Message> {
    #[must_use]
    pub fn command_palette(
        mut self,
        palette: CommandPalette<Node, Message>,
    ) -> Self {
        self.command_palette = Some(palette);
        self
    }
}
```

Update `AppLayout::new` to initialize it to `None`.

### 4. Add the renderer in `snora`

Create `snora/src/overlay/command_palette.rs`:

```rust
pub(crate) fn render_command_palette<'a, Message>(
    palette: CommandPalette<Element<'a, Message>, Message>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where Message: Clone + 'a,
{
    // physical anchoring resolved here, not in snora-core
    // ...
}
```

Wire it into `render` in `snora/src/render.rs`. Decide which
existing layer it joins (above the dim, with the modals; or below,
with the menus) and put the `if let Some(palette)` block in the right
place. Update the doc comment listing the layer order.

### 5. Add tests in `snora-core`

In the new overlay's `tests` module:

- Default constructor produces sensible defaults.
- Builder methods compose (`.at(...).at(...)` keeps the second).
- Any new vocabulary enums partition cleanly (cf. the
  `ToastPosition::is_top` / `is_bottom` pair).

### 6. Document

- `docs/reference/vocabulary.md` — add the new enums.
- `docs/guides/overlays.md` — add a section to the table at top and
  a paragraph below explaining when to reach for it.
- If it changes behavior of close sinks, update
  `docs/guides/overlays.md`'s "one close sink, two channels" section.

### 7. Add an example

Under `examples/<n>/` create a tiny crate that demonstrates
the overlay in isolation. Existing examples are 100–200 lines each;
follow the same length budget.

Add the new example to:

- The workspace `Cargo.toml` `members` list.
- The "Examples" section of the root `README.md`.
- `docs/README.md` if it is something a beginner should see early.

## Things to *not* do

- **Do not put physical anchoring (`Left` / `Right`) in
  `snora-core`.** That breaks ABDD. Always use logical edges and
  resolve to physical sides in the engine renderer.
- **Do not let the new overlay carry its own close hook.** Outside-
  click is wired once at `AppLayout` level. Per-overlay close
  buttons inside the content are fine; outside-click sinks are not.
- **Do not change the dim layer's behavior.** If your overlay needs
  no dim, classify it as a *menu* (uses transparent backdrop, fired
  by `on_close_menus`), not a modal. The dim layer is shared by all
  modals.
