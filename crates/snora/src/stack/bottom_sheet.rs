use iced::{
    Background, Color, Element, Length,
    widget::{column, container, mouse_area, opaque, space},
};
use snora_core::contract::app::bottom_sheet::BottomSheet;

pub fn render_bottom_sheet<'a, Message>(
    bottom_sheet: Option<BottomSheet<Element<'a, Message>, Message>>,
) -> Option<Element<'a, Message>>
where
    Message: 'a + Clone,
{
    let bottom_sheet = if let Some(x) = bottom_sheet {
        x
    } else {
        return None;
    };

    // インタラクティブ・レイヤー
    // column 自体は画面全体を覆うが、上部の space 部分でクリックを拾うようにする
    let bottom_sheet_content = column![
        // 上部 2/3：ここをクリックしたら閉じる
        if let Some(msg) = bottom_sheet.on_close {
            mouse_area(space().width(Length::Fill).height(Length::FillPortion(2))).on_press(msg)
        } else {
            mouse_area(space().height(Length::FillPortion(2)))
        },
        // 下部 1/3：シート本体（ここはクリックを透過させない）
        container(opaque(bottom_sheet.content))
            .width(Length::Fill)
            .height(Length::FillPortion(1))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 1.0))),
                ..Default::default()
            })
    ];

    Some(bottom_sheet_content.into())
}
