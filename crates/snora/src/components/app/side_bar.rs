use iced::{
    Alignment, Length,
    widget::{
        text, {button, column, container, tooltip},
    },
};
use snora_core::contract::app::side_bar::AppSideBar;

use crate::components::icon::render_icon;

pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: AppSideBar<Message, ViewId>,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: PartialEq + 'a,
{
    let mut col = column![].spacing(16).align_x(Alignment::Center);

    for item in side_bar.items {
        // アクティブ状態の判定（今回は見た目の変化のプレースホルダーとして）
        let _is_active = item.view_id == side_bar.active_view_id;

        let icon_el = render_icon(&item.icon);

        let btn = button(icon_el)
            .on_press(item.action.clone())
            // iced 0.14 の簡略化された Length 指定
            .width(48)
            .height(48);

        let item_with_tooltip = tooltip(btn, text(item.tooltip), tooltip::Position::Right);

        col = col.push(item_with_tooltip);
    }

    container(col)
        .width(64)
        .height(Length::Fill)
        .padding(16.0)
        // TODO: App全体のテーマに合わせた背景色を指定
        .into()
}
