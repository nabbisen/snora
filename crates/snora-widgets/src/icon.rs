//! Render a [`snora_core::Icon`] into an `iced::Element`.

use iced::{Element, widget::text};
use snora_core::Icon;

/// Default icon render size in pixels. Matches the text size used by
/// menus and sidebar tooltips so icons and their labels align.
const DEFAULT_ICON_SIZE: f32 = 14.0;

/// Render an icon at the default size.
pub fn icon_element<'a, Message>(icon: &Icon) -> Element<'a, Message>
where
    Message: 'a,
{
    icon_element_sized(icon, DEFAULT_ICON_SIZE)
}

/// Render an icon at a custom pixel size.
///
/// **`snora::toast` repeats this construction** for the toast close
/// button's lucide `X`: the engine does not depend on this crate, so it
/// builds its own `text(char::from(..)).font(Font::with_name("lucide"))`.
/// The two sites must agree, and
/// `crates/snora/tests/toast_close_button.rs` fails if either changes
/// codepoint or loses the font (RFC-101 Q-1 (a)).
pub fn icon_element_sized<'a, Message>(icon: &Icon, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    match icon {
        Icon::Text(s) => text(s.to_owned()).size(size).into(),

        #[cfg(feature = "lucide-icons")]
        Icon::Lucide(lucide_const) => {
            // Do NOT call lucide_const.widget() — that method returns
            // iced::widget::Text parameterised against lucide-icons' own
            // iced_core version, which may differ from snora-widgets' iced
            // dependency when the downstream app has multiple iced_core
            // versions in the graph. Instead, extract the unicode codepoint
            // via the stable `From<Icon> for char` impl (which has no iced
            // dependency) and build the Text widget using our own iced::text.
            // This replicates what lucide-icons does internally without the
            // type-parameter mismatch. See the v0.18.1 bug fix.
            let glyph = char::from(*lucide_const).to_string();
            text(glyph)
                .font(iced::Font::with_name("lucide"))
                .size(size)
                .into()
        }

        #[cfg(feature = "svg-icons")]
        Icon::Svg(path) => iced::widget::svg(iced::widget::svg::Handle::from_path(path.clone()))
            .height(iced::Length::Fixed(size))
            .width(iced::Length::Fixed(size))
            .into(),
    }
}
