# Adding a new overlay kind

Use this page when you want to add an overlay surface that does not
fit `Dialog`, `BottomSheet`, or `context_menu`. Examples that have
come up in discussion (none implemented yet): a **side drawer**
sliding from the start edge, a **command palette** centered like a
dialog but with `Escape` to close, an **anchored popover** attached
to a specific widget.

## Decision tree first

Before writing code, ask:

1. **Is this really a new overlay, or could it be a `Dialog` with
   different inner content?** A command palette is often best built
   as a `Dialog` whose `content` is your search input + result list.
   You get the dim layer and the `on_close_modals` plumbing for
   free, and there is no new vocabulary.
2. **Is it modal or transient?** Modal → a sibling of `Dialog` /
   `BottomSheet`. Transient → a sibling of `header_menu` /
   `context_menu`.
3. **Does it have configuration that does not fit existing
   vocabulary?** A side drawer would want a `DrawerEdge` (start /
   end). A command palette would not.

If you can answer "use a `Dialog`" to (1), stop here.

## Steps if you do need a new overlay

### 1. Add the data type to `snora-core`

Place it in `src/overlay.rs` next to `Dialog` and `BottomSheet`. Keep
the same shape:

```rust
pub struct SideDrawer<Node, Message> {
    pub content: Node,
    pub edge: DrawerEdge,
    _marker: PhantomData<Message>,
}

impl<Node, Message> SideDrawer<Node, Message> {
    pub fn new(content: Node) -> Self { /* sane default */ }

    #[must_use]
    pub fn at(mut self, edge: DrawerEdge) -> Self {
        self.edge = edge;
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

If your overlay has configuration (drawer edge, palette mode,
anchor type), add a small enum next to the struct. Use *logical*
terms (`Start` / `End`) for axis-aligned variants — never `Left` /
`Right` directly.

### 3. Add an `AppLayout` field + builder method

In `snora-core/src/layout.rs`:

```rust
pub struct AppLayout<Node, Message> {
    // existing fields...
    pub side_drawer: Option<SideDrawer<Node, Message>>,
}

impl<Node, Message: Clone> AppLayout<Node, Message> {
    #[must_use]
    pub fn side_drawer(mut self, drawer: SideDrawer<Node, Message>) -> Self {
        self.side_drawer = Some(drawer);
        self
    }
}
```

Update `AppLayout::new` to initialize it to `None`.

### 4. Add the renderer in `snora`

Create `snora/src/overlay/side_drawer.rs`:

```rust
pub(crate) fn render_side_drawer<'a, Message>(
    drawer: SideDrawer<Element<'a, Message>, Message>,
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
with the menus) and put the `if let Some(drawer)` block in the right
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
