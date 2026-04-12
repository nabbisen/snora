use strum_macros::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewId {
    Home,
    Search,
    Settings,
}

impl std::fmt::Display for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ViewId::Home => write!(f, "Home"),
            ViewId::Search => write!(f, "Search"),
            ViewId::Settings => write!(f, "Settings"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum MenuId {
    File,
    Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemId {
    File(FileMenuItemId),
    Settings(SettingsMenuItemId),
}

impl std::fmt::Display for MenuItemId {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuItemId::File(x) => write!(w, "{}", x),
            MenuItemId::Settings(x) => write!(w, "{}", x),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum FileMenuItemId {
    New,
}

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum SettingsMenuItemId {
    About,
}
