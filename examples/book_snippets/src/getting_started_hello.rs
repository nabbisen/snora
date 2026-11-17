//! Anchored source for `docs/src/getting-started/02-hello-world.md` (RFC-136): the whole program.

// ANCHOR: hello_world
use iced::{Element, widget::text};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}

    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = text("Hello, snora!").into();
        render(AppLayout::new(body))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("Hello"))
        .run()
}
// ANCHOR_END: hello_world
