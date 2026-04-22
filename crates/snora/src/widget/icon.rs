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
pub fn icon_element_sized<'a, Message>(icon: &Icon, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    match icon {
        Icon::Text(s) => text(s.to_owned()).size(size).into(),

        #[cfg(feature = "lucide-icons")]
        Icon::Lucide(lucide_const) => lucide_const.widget().size(size).into(),

        #[cfg(feature = "svg-icons")]
        Icon::Svg(path) => iced::widget::svg(iced::widget::svg::Handle::from_path(path.clone()))
            .height(iced::Length::Fixed(size))
            .width(iced::Length::Fixed(size))
            .into(),
    }
}
