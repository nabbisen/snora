//! Render a [`snora_core::Icon`] into an `iced::Element`.

use iced::{Element, widget::text};
#[cfg(feature = "svg-icons")]
use snora_control::EmBox;
use snora_control::Glyph;
use snora_core::Icon;

/// Render an icon at **1 em of the text it sits with** (RFC-115's
/// amendment).
///
/// "Em" is the host's `default_text_size`, resolved when the layout runs,
/// so the icon follows the text with no size to pass: **16 px at iced's
/// default**, and 24 px if the application sets 24. Before RFC-116's
/// Unit 3 this was a literal 14 that followed nothing, so an application
/// that enlarged its text got icons that stayed small beside it, and at
/// the default the icon sat two pixels under the label it labelled.
///
/// Use [`icon_element_sized`] to name a size yourself. The design-path
/// widgets do: their label is a token (`Typography::label`, 14), and the
/// icon is that size, unchanged.
pub fn icon_element<'a, Message>(icon: &Icon) -> Element<'a, Message>
where
    Message: 'a,
{
    icon_in_em(icon, 1.0)
}

/// An icon at `em` times the host's text size, read at layout time.
///
/// The one place the three icon kinds are built, so the unstyled sizes
/// (1 em for a pictogram) cannot drift between them.
fn icon_in_em<'a, Message>(icon: &Icon, em: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    match icon {
        Icon::Text(s) => Glyph::new(s.to_owned(), em).into(),

        #[cfg(feature = "lucide-icons")]
        Icon::Lucide(lucide_const) => {
            // Same construction as `icon_element_sized`; see the note there
            // on why `lucide_const.widget()` is not used.
            Glyph::new(char::from(*lucide_const).to_string(), em)
                .font(iced::Font::with_name("lucide"))
                .into()
        }

        // An SVG needs a number of pixels, and the number is `em` times a
        // size only layout can read: a box of that size, filled.
        #[cfg(feature = "svg-icons")]
        Icon::Svg(path) => EmBox::new(
            iced::widget::svg(iced::widget::svg::Handle::from_path(path.clone()))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            em,
        )
        .into(),
    }
}

/// The icon a widget draws next to a label of `label_size`.
///
/// `Some` is the design path, whose label is a token: the icon is that
/// size, unchanged from before RFC-116. `None` is the unstyled path,
/// whose label is the host's text: the icon is 1 em of it.
pub(crate) fn icon_beside_label<'a, Message>(
    icon: &Icon,
    label_size: Option<f32>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    match label_size {
        Some(size) => icon_element_sized(icon, size),
        None => icon_element(icon),
    }
}

/// Render an icon at a custom pixel size.
///
/// **`snora::toast` repeats this construction** for the toast close
/// button's lucide `X`: the engine does not depend on this crate, so it
/// builds its own `Glyph::new(char::from(..).to_string(), 1.125)` in the
/// lucide font. The two sites must agree on the codepoint and the font,
/// and `crates/snora/tests/toast_close_button.rs` fails if either changes
/// one or loses the other (RFC-101 Q-1 (a)).
pub fn icon_element_sized<'a, Message>(icon: &Icon, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    match icon {
        Icon::Text(s) => text(s.to_owned()).size(size).into(),

        #[cfg(feature = "lucide-icons")]
        Icon::Lucide(lucide_const) => {
            // Do NOT call lucide_const.widget() — that method returns
            // iced::widget::Text parameterised against lucide-icons' own
            // iced_core version, which may differ from snora-widgets' iced
            // dependency when the downstream app has multiple iced_core
            // versions in the graph. Instead, extract the unicode codepoint
            // via the stable `From<Icon> for char` impl (which has no iced
            // dependency) and build the Text widget using our own iced::text.
            // This replicates what lucide-icons does internally without the
            // type-parameter mismatch. See the v0.18.1 bug fix.
            let glyph = char::from(*lucide_const).to_string();
            text(glyph)
                .font(iced::Font::with_name("lucide"))
                .size(size)
                .into()
        }

        #[cfg(feature = "svg-icons")]
        Icon::Svg(path) => iced::widget::svg(iced::widget::svg::Handle::from_path(path.clone()))
            .height(iced::Length::Fixed(size))
            .width(iced::Length::Fixed(size))
            .into(),
    }
}
