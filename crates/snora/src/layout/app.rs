use std::fmt::Debug;

use iced::{
    Background, Color, Element, Length,
    widget::{column, container, mouse_area, row, space, stack},
};
use snora_core::contract::{app::AppLayout, page::PageContract, rtl::LayoutDirection};

use crate::stack::{bottom_sheet::render_bottom_sheet, dialog::render_dialog, toast::render_toast};

pub fn render_app<'a, Node, Message, MenuId>(
    layout: AppLayout<Node, Message, MenuId>,
    on_close_menus: Option<Message>,  // メニュー用 (透明背景クリック)
    on_close_modals: Option<Message>, // モーダル用 (半透明背景クリック)
) -> Element<'a, Message>
where
    Node: PageContract<Node = Element<'a, Message>, Message = Message>,
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
{
    // Base Layer (Header -> Body/Sidebar -> Footer)
    let mut main_col = column![];

    if let Some(header) = layout.header {
        main_col = main_col.push(header.view());
    }

    let mut body_row = row![].width(Length::Fill).height(Length::Fill);
    let body = container(layout.body.view())
        .width(Length::Fill)
        .height(Length::Fill);

    match layout.direction {
        LayoutDirection::Ltr => {
            if let Some(sidebar) = layout.side_bar {
                body_row = body_row.push(sidebar.view());
            }
            body_row = body_row.push(body);
        }
        LayoutDirection::Rtl => {
            body_row = body_row.push(body);
            if let Some(sidebar) = layout.side_bar {
                body_row = body_row.push(sidebar.view());
            }
        }
    }

    main_col = main_col.push(body_row);

    if let Some(footer) = layout.footer {
        main_col = main_col.push(footer.view());
    }

    let base_layer = container(main_col).width(Length::Fill).height(Length::Fill);

    // layers
    let mut layers = stack![base_layer];

    let has_menu = layout.header_menu.is_some() || layout.context_menu.is_some();
    if let (true, Some(on_close_menus)) = (has_menu, on_close_menus) {
        // 透明な壁で入力を奪い、メニューを閉じる
        layers = layers.push(
            mouse_area(container(space()).width(Length::Fill).height(Length::Fill))
                .on_press(on_close_menus),
        );

        if let Some(header_menu_node) = layout.header_menu {
            layers = layers.push(header_menu_node);
        }

        if let Some(context_menu_node) = layout.context_menu {
            layers = layers.push(context_menu_node);
        }
    }

    // --- Modal Layer ---
    let has_modal = layout.dialog.is_some() || layout.bottom_sheet.is_some();
    if let (true, Some(on_close_modals)) = (has_modal, on_close_modals) {
        // 半透明（Dimmed）な壁で背景をロック
        let dimmed = container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
                ..Default::default()
            });
        layers = layers.push(mouse_area(dimmed).on_press(on_close_modals));

        // -- Dialog オーバーレイ --
        if let Some(dialog_stack) = render_dialog(layout.dialog) {
            layers = layers.push(dialog_stack);
        }

        // -- BottomSheet オーバーレイ --
        if let Some(bottom_sheet_stack) = render_bottom_sheet(layout.bottom_sheet) {
            layers = layers.push(bottom_sheet_stack);
        }
    }

    // -- Toast オーバーレイ --
    if let Some(toast_stack) = render_toast(layout.toasts) {
        layers = layers.push(toast_stack);
    }

    // layers.width(Length::Fill).height(Length::Fill).into()
    layers.into()
}
