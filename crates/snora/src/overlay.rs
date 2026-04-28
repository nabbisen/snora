//! Internal renderers for modal overlays. Not part of the public API —
//! the engine ([`crate::render`]) calls into these.

pub(crate) mod dialog;
pub(crate) mod sheet;
