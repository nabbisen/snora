//! Anchored source for `docs/src/guides/icons.md` (RFC-136). The lucide and SVG fragments are compiled with their features
//! (`--all-features`, as CI builds the workspace).

#[rustfmt::skip]
fn icons_text() {
use snora::Icon;
// ANCHOR: icons_text
let i: Icon = "★".into();                // From<&str>
let i: Icon = String::from("★").into();  // From<String>
let i = Icon::Text("✓".into());           // explicit
// ANCHOR_END: icons_text
}

#[cfg(feature = "lucide-icons")]
#[rustfmt::skip]
fn icons_lucide() {
// ANCHOR: icons_lucide
use snora::Icon;
use snora::lucide;                     // re-exported variants

let i: Icon = lucide::Home.into();
let i = Icon::Lucide(lucide::Settings);
// ANCHOR_END: icons_lucide
}

#[cfg(feature = "lucide-icons")]
mod lucide_font {
    #[derive(Debug, Clone)]
    enum Message {}

    #[derive(Default)]
    struct App;

    impl App {
        fn update(&mut self, _message: Message) {}

        fn view(&self) -> iced::Element<'_, Message> {
            snora::widget::icon_element(&snora::lucide::Home.into())
        }
    }

    #[rustfmt::skip]
    fn main() -> iced::Result {
// ANCHOR: icons_lucide_font
iced::application(App::default, App::update, App::view)
    // Once, at start-up: without it every `Icon::Lucide` draws as a stray glyph or a box.
    .font(snora::lucide::LUCIDE_FONT_BYTES)
    .run()
// ANCHOR_END: icons_lucide_font
    }
}

#[cfg(feature = "svg-icons")]
#[rustfmt::skip]
fn icons_svg() {
use snora::Icon;
// ANCHOR: icons_svg
let i = Icon::Svg(std::path::PathBuf::from("assets/logo.svg"));
// ANCHOR_END: icons_svg
}

#[rustfmt::skip]
fn icons_sized() {
use snora::Icon;
// ANCHOR: icons_sized
use snora::widget::icon::icon_element_sized;

let big_logo = icon_element_sized(&Icon::Text("✓".into()), 24.0);
// ANCHOR_END: icons_sized
let _: iced::Element<'_, ()> = big_logo;
}
