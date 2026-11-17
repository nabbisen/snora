//! Scaffolding shared by the compiled book snippets (RFC-136): the application a snippet is a fragment of.
//!
//! A book page shows the lines of a fragment (`let t = &self.tokens;`, `Message::ToggleDrafts`, …) and
//! the fragment names things only an application has: its state, its `Message`. They are defined here,
//! **outside every anchor**, so the page shows exactly what it showed and the fragment still compiles
//! against the API it calls.

use iced::Element;
use snora::design::Tokens;

/// The application's messages, as the fragments name them.
#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Cancel,
    Skip,
    ChooseFolder,
    Delete,
    DoSomething,
    ToggleDrafts,
    ToggleActive,
    ToggleRust,
    RemoveRust,
    RemoveTag(String),
    OpenSync,
    DismissNotice,
    FreeSpace,
    Toggle,
    Remove,
}

/// The application's state, as the fragments name it.
pub struct App {
    pub tokens: Tokens,
    pub show_drafts: bool,
    pub show_active: bool,
    pub rust_selected: bool,
    pub active_tags: Vec<String>,
    pub selected_id: Option<u32>,
    pub can_save: bool,
    pub can_delete: bool,
    pub show_notice: bool,
}

/// One item of a list the fragment draws.
pub struct Item {
    pub id: u32,
}
