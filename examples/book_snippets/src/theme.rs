//! Anchored source for `docs/src/design/theme.md`.

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::text("").into()
    }
}

// ANCHOR: theme_basic_usage
fn basic_usage() -> iced::Result {
    use snora::design::{Tokens, theme};

    let tokens = Tokens::high_contrast_dark();
    let iced_theme = theme(&tokens);

    iced::application(App::default, App::update, App::view)
        .theme(move |_state: &App| iced_theme.clone())
        .run()
}
// ANCHOR_END: theme_basic_usage
