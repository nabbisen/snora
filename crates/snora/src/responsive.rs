//! Exposes the layout's available width to the application (RFC-046).
//!
//! `AppLayout` is an application shell — header, sidebar, body, footer —
//! and adapting that composition to available width is close to the
//! definition of what a shell does. Before this, snora had **no
//! window-size awareness of any kind**: an application wanting
//! breakpoints had to write window observation itself.
//!
//! # Exposure, not behavior
//!
//! [`responsive_render`] prescribes nothing: it hands the application a
//! width and lets it build whatever `AppLayout` it wants from that. There
//! is no `Breakpoint` type, no threshold, no auto-collapse — those decide
//! *for* the application, which is exactly the kind of decision snora has
//! consistently declined (no theming layer, no form widgets, no
//! prescribed layout beyond the skeleton itself). The application decides
//! its own thresholds and what changes at them; snora supplies the
//! number.
//!
//! # `f32` width, not `Size` (RFC-046 Q-2)
//!
//! Width is what the downstream request actually asked for, and is the
//! narrower contract — `iced::widget::Responsive`'s closure receives the
//! full `Size`, but only `.width` is threaded through here. Height would
//! cost nothing extra to also expose, but RFC-046 asks for a decision
//! stated plainly rather than defaulting to "give them everything the
//! widget happens to have" — narrowing to what was asked for is the
//! chosen answer; see the review request for the full reasoning.
//!
//! # Reuses the existing z-stack — does not duplicate it
//!
//! [`responsive_render`]'s closure calls [`crate::render::render`]
//! directly — the same public entry point applications call today. There
//! is no second copy of the layer-composition logic; RFC-039 already
//! extracted the shared path this reuses.
//!
//! # This is the engine path (RFC-053)
//!
//! [`responsive_render`] renders through [`crate::render::render`]
//! **unconditionally** — a `design`-path application that adopts it
//! loses the styled dialog card and the token-derived modal dim, since
//! neither exists on the engine path. Use
//! [`crate::design::render::responsive_render`] (`snora::design::responsive_render`)
//! instead if your application calls [`crate::design::render::render`]
//! (`snora::design::render`) elsewhere.

use iced::widget::Responsive;
use iced::{Element, Size};
use snora_core::AppLayout;

/// Renders an [`AppLayout`] that may depend on the available width.
///
/// `build` receives the width available to the layout (in logical
/// pixels) and returns the `AppLayout` to render at that width. It may
/// be called again whenever the available size changes — see
/// [`iced::widget::Responsive`]'s own documentation for the underlying
/// mechanism.
///
/// A sibling to [`crate::render::render`], not a replacement — this is
/// engine capability, not `design`-gated, so it lives in the default
/// surface next to `render`. Applications not calling this entry point
/// are unaffected; `render`'s own behavior and signature are unchanged.
///
/// **Renders through the engine path unconditionally** (RFC-053) — see
/// the module documentation's "This is the engine path" section. A
/// `design`-path application wants
/// [`crate::design::render::responsive_render`] instead, or this
/// function silently drops the styled dialog card and derived modal
/// dim.
///
/// ```rust,ignore
/// use snora::{AppLayout, responsive_render};
///
/// fn view(state: &State) -> Element<'_, Message> {
///     responsive_render(move |width| {
///         let layout = AppLayout::new(body(state));
///         if width < 600.0 {
///             layout // application's own choice: no sidebar below 600px
///         } else {
///             layout.side_bar(sidebar(state))
///         }
///     })
/// }
/// ```
#[must_use]
pub fn responsive_render<'a, Message, F>(build: F) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a,
{
    Responsive::new(move |size: Size| {
        let layout = build(size.width);
        crate::render::render(layout)
    })
    .into()
}

#[cfg(test)]
mod tests;
