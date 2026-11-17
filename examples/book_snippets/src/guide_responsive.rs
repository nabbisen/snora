//! Anchored source for `docs/src/guides/responsive.md` (RFC-136).

use iced::Element;
use snora::{SideBar, SideBarItem};

#[derive(Clone)]
enum Message {}

struct State;

fn body(state: &State) -> Element<'_, Message> {
    iced::widget::space().into()
}

fn sidebar(state: &State) -> Element<'_, Message> {
    iced::widget::space().into()
}

// ANCHOR: responsive_view
use snora::{AppLayout, responsive_render};

fn view(state: &State) -> Element<'_, Message> {
    responsive_render(move |width| {
        let layout = AppLayout::new(body(state));

        if width < 600.0 {
            layout // your own choice: no sidebar below 600px
        } else {
            layout.side_bar(sidebar(state))
        }
    })
}
// ANCHOR_END: responsive_view
