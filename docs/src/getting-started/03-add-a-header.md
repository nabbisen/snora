# 3 — Add a header, sidebar, footer

`AppLayout` has four skeleton slots: `header`, `side_bar`, `body`,
`footer`. Each accepts any `iced::Element` — you can hand-roll your own
or use the prefab helpers in `snora::widget`.

## Adding slots one at a time

```rust
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem,
    render,
    widget::{app_footer, app_header, app_side_bar},
};

fn view(state: &State) -> iced::Element<'_, Message> {
    let header = app_header(
        "My App",
        Vec::<snora::Menu<(), ()>>::new(),  // no menus yet
        &Message::HeaderAction,
        None,                               // no menu open
        None,                               // no end-of-row controls
        LayoutDirection::Ltr,
    );

    let sidebar = app_side_bar(
        SideBar {
            items: vec![
                SideBarItem {
                    view_id: ViewId::Home,
                    icon: "🏠".into(),
                    tooltip: "Home".into(),
                    on_press: Message::Switch(ViewId::Home),
                },
                // …
            ],
            active: state.active_view,
        },
        LayoutDirection::Ltr,
    );

    let footer = app_footer(iced::widget::text("status").into());

    let body: iced::Element<'_, Message> = state.body();

    let layout = AppLayout::new(body)
        .header(header)
        .side_bar(sidebar)
        .footer(footer);

    render(layout)
}
```

## A note on direction

Every prefab widget that has a left/right asymmetry takes a
`LayoutDirection` argument. Passing the same direction everywhere is
the typical pattern; in apps that support live LTR/RTL flipping you
keep the active direction on your state and re-pass it on each `view`.

`AppLayout::direction(...)` separately controls the *body row* mirroring
(sidebar side flips). See [Direction and ABDD](../guides/direction.md)
for the full picture.

## Custom slots

You are not required to use the prefab widgets. Anything that yields an
`Element<'_, Message>` slots in:

```rust
let custom_header = iced::widget::container(my_header_row())
    .width(iced::Length::Fill)
    .padding(12)
    .into();

let layout = AppLayout::new(body).header(custom_header);
```

snora draws the skeleton; what fills each slot is your decision.

## Next

Toasts have framework-managed lifetime. See
[Toasts](04-toasts.md).
