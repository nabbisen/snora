use iced::{Element, widget::text};
use snora_core::contract::ui::Icon;

pub fn render_icon<'a, Message>(icon: &Icon) -> Element<'a, Message>
where
    Message: 'a,
{
    match icon {
        Icon::Text(s) => text(s.to_owned()).size(14).into(),
        #[cfg(feature = "lucide-icons")]
        Icon::Lucide(lucide_constant) => lucide_constant.widget().size(14).into(),
        #[cfg(feature = "svg-icons")]
        Icon::Svg(path) => iced::advanced::svg::Svg::new(path)
            // .width(Length::Fixed(14.0))
            .height(iced::Length::Fixed(14.0))
            .into(),
    }
}
