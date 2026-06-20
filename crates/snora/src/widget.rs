pub use snora_widgets::{
    app_breadcrumb, app_footer, app_header, app_side_bar, app_tab_bar, icon_element,
    icon_element_sized, render_menu,
};

/// The `icon` submodule path (kept for source-compat with 0.5.x
/// callers using `snora::widget::icon::icon_element`).
pub mod icon {
    pub use snora_widgets::{icon_element, icon_element_sized};
}
