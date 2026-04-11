use iced::{
    widget::{column, row, space},
    Element,
};
use snora_core::contract::{LayoutDirection, PageLayout};

pub fn build_layout<'a, Message: 'a>(
    layout: PageLayout<Element<'a, Message>, Message>,
) -> Element<'a, Message> {
    let mut main_content = Vec::new();

    // 1. 論理的な順序でスタック
    if let Some(aside) = layout.aside {
        main_content.push(aside);
    }
    main_content.push(layout.body);

    // 2. 物理的な順序（RTL対応）
    if layout.direction == LayoutDirection::Rtl {
        main_content.reverse();
    }

    let center_row = row(main_content).spacing(10);

    column![
        layout.header.unwrap_or_else(|| space().into()),
        center_row,
        layout.footer.unwrap_or_else(|| space().into()),
    ]
    .into()
}
