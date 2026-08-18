//! The semantic color [`Palette`].

use crate::Color;

/// Semantic color roles for a theme.
///
/// Roles are named by *meaning*, not by a color scale. Status background roles
/// (`success`/`warning`/`danger`/`info`) each have a paired on-status
/// foreground (`*_text`) so status surfaces — starting with the v0.20 danger
/// button — have a contrast-tested foreground rather than borrowing
/// `accent_text`.
///
/// `Palette` is `#[non_exhaustive]`: new roles (the documented future roles
/// such as `surface_sunken`, `overlay`, `selection`, `separator`) can be added
/// without a breaking change. Construct one through a [`crate::Tokens`] preset
/// rather than a struct literal; you may still mutate individual fields
/// (`tokens.palette.accent = ...`).
///
/// `text_muted` is the lowest-contrast text role, for non-essential text,
/// and — like every other text role — meets WCAG AA body-text contrast
/// (4.5:1) against all three surfaces, asserted in the contrast suite
/// (RFC-058).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Window / app background.
    pub background: Color,
    /// Primary surface (cards, panels).
    pub surface: Color,
    /// Raised surface (elevated cards, popovers).
    pub surface_raised: Color,

    /// Primary body text.
    pub text_primary: Color,
    /// Secondary text. Meets WCAG AA body-text contrast (4.5:1) against all
    /// three surfaces.
    pub text_secondary: Color,
    /// Muted text for non-essential content. Meets WCAG AA body-text
    /// contrast (4.5:1) against all three surfaces (RFC-058), same as every
    /// other text role.
    pub text_muted: Color,

    /// Borders and separators.
    pub border: Color,
    /// Accent / primary action color.
    pub accent: Color,
    /// Foreground used on top of `accent`.
    pub accent_text: Color,

    /// Success background.
    pub success: Color,
    /// Foreground used on top of `success`.
    pub success_text: Color,
    /// Warning background.
    pub warning: Color,
    /// Foreground used on top of `warning`.
    pub warning_text: Color,
    /// Danger / destructive background.
    pub danger: Color,
    /// Foreground used on top of `danger`.
    pub danger_text: Color,
    /// Informational background.
    pub info: Color,
    /// Foreground used on top of `info`.
    pub info_text: Color,

    /// Focus-ring color.
    pub focus: Color,
}

/// Threshold class a role's contrast pairs are measured against
/// (RFC-063). Explicit per role, not inferred from the name — `focus`
/// and `border` are both non-text and share no naming convention with
/// the `*_text` roles, and "something was implicit" is the defect this
/// RFC exists to close.
///
/// `Focus` and `NonText` map to `tests.rs`'s separate `FOCUS_MIN` and
/// `NON_TEXT_MIN` constants (RFC-058) — kept distinct here for the same
/// reason those constants are: `focus` and `border` contrast are
/// allowed to diverge later without one silently following the other.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThresholdClass {
    /// WCAG AA body text — `tests.rs`'s `AA_TEXT`.
    Text,
    /// Focus-ring contrast — `tests.rs`'s `FOCUS_MIN`.
    Focus,
    /// Non-text boundary contrast other than focus — `tests.rs`'s
    /// `NON_TEXT_MIN`.
    NonText,
}

/// One `Palette` role's declared intended-usage surfaces and threshold
/// class (RFC-063).
///
/// `class` and `surfaces` are both empty/`None` together for a role
/// that renders as a surface or fill itself and is never measured as a
/// foreground (`background`, `accent`, …) — declared that way
/// explicitly by [`Palette::usages`], not omitted.
#[cfg(test)]
pub(crate) struct RoleUsage {
    /// The role's name, for assertion failure messages.
    pub(crate) label: &'static str,
    /// The role's own color.
    pub(crate) fg: Color,
    /// `None` for a role with no measurable foreground usage.
    pub(crate) class: Option<ThresholdClass>,
    /// `(surface label, surface color)` pairs this role is *intended*
    /// to render on — not the cross-product of every surface it could
    /// technically sit on. `accent_text` declares only `accent`, not
    /// `background`, because `accent_text` exists to sit on `accent`;
    /// asserting it against surfaces it never renders on would be noise
    /// in an accessibility gate, which is how gates get ignored.
    pub(crate) surfaces: Vec<(&'static str, Color)>,
}

impl Palette {
    /// Declares every role's intended rendering surfaces and threshold
    /// class (RFC-063). Crate-private and test-only, matching the
    /// visibility this replaces.
    ///
    /// **Exhaustive destructuring, deliberately.** `Palette` is
    /// `#[non_exhaustive]` for *other* crates; that attribute does not
    /// constrain the crate that defines it. Adding a nineteenth field
    /// to `Palette` makes the `let Palette { .. } = *self;` below fail
    /// to compile (`E0027: pattern does not mention field ...`) until
    /// this function says where the new role renders and what it must
    /// be measured against — that is the whole enforcement mechanism.
    /// **Do not add `..` to the pattern below.** The compiler will
    /// suggest it on a missing-field error; that suggestion silently
    /// defeats the entire point of this function.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn usages(&self) -> Vec<RoleUsage> {
        let Palette {
            background,
            surface,
            surface_raised,
            text_primary,
            text_secondary,
            text_muted,
            border,
            accent,
            accent_text,
            success,
            success_text,
            warning,
            warning_text,
            danger,
            danger_text,
            info,
            info_text,
            focus,
        } = *self;

        let neutral_surfaces = || {
            vec![
                ("background", background),
                ("surface", surface),
                ("surface_raised", surface_raised),
            ]
        };

        vec![
            // Fill/surface roles: rendered as a surface itself, never
            // measured as a foreground on top of another surface.
            // Declared explicitly, empty, rather than silently omitted.
            RoleUsage {
                label: "background",
                fg: background,
                class: None,
                surfaces: vec![],
            },
            RoleUsage {
                label: "surface",
                fg: surface,
                class: None,
                surfaces: vec![],
            },
            RoleUsage {
                label: "surface_raised",
                fg: surface_raised,
                class: None,
                surfaces: vec![],
            },
            // RFC-063 review (R-1): `accent`/`danger` back two *filled,
            // borderless* interactive controls (`snora_style::button::
            // {primary, danger}` — `Border::default()`, width 0). With no
            // border, the fill itself is the button's identifying
            // boundary against whatever neutral surface it sits on —
            // the identical argument RFC-058 used for the dialog card's
            // border, applied to a fill instead of a stroke.
            RoleUsage {
                label: "accent",
                fg: accent,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "danger",
                fg: danger,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            // `success`/`warning`/`info` have no filled-button counterpart
            // in `button.rs`; their only usage
            // (`snora-widgets/src/design/notice.rs`) is a 4px left tone bar
            // and a 1px border on an *informational* panel that also carries
            // the same tone in its title/body text.
            //
            // **Declared anyway (architect's call, RFC-063 round 2).** The
            // question a declaration answers is "does this role render on
            // this surface", not "does WCAG compel a ratio there" — and it
            // demonstrably does render on all three. Leaving it undeclared
            // is precisely the silently-breakable state this RFC exists to
            // close: a later palette edit could drop the tone bar below 3:1
            // with nothing to catch it.
            //
            // The obligation is nonetheless **redundant-signal**, not
            // sole-identifier: unlike accent/danger's borderless filled
            // buttons, the tone is also carried in text, which is snora's own
            // non-colour-status-encoding principle. So if a future edit ever
            // trips one of these, that is a prompt for a judgement call, not
            // an automatic defect — the opposite of the accent/danger case
            // above. Current worst case 4.63:1 (`success`/`surface`, light),
            // clearing NON_TEXT_MIN by 54%.
            RoleUsage {
                label: "success",
                fg: success,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "warning",
                fg: warning,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "info",
                fg: info,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            // Body text — rendered directly on any of the three
            // neutral surfaces.
            RoleUsage {
                label: "text_primary",
                fg: text_primary,
                class: Some(ThresholdClass::Text),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "text_secondary",
                fg: text_secondary,
                class: Some(ThresholdClass::Text),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "text_muted",
                fg: text_muted,
                class: Some(ThresholdClass::Text),
                surfaces: neutral_surfaces(),
            },
            // Non-text boundaries — a border or focus ring can appear
            // around a control drawn on any of the three neutral
            // surfaces.
            RoleUsage {
                label: "border",
                fg: border,
                class: Some(ThresholdClass::NonText),
                surfaces: neutral_surfaces(),
            },
            RoleUsage {
                label: "focus",
                fg: focus,
                class: Some(ThresholdClass::Focus),
                surfaces: neutral_surfaces(),
            },
            // On-status text — each exists to sit on exactly its own
            // status background, not the neutral surfaces.
            RoleUsage {
                label: "accent_text",
                fg: accent_text,
                class: Some(ThresholdClass::Text),
                surfaces: vec![("accent", accent)],
            },
            RoleUsage {
                label: "success_text",
                fg: success_text,
                class: Some(ThresholdClass::Text),
                surfaces: vec![("success", success)],
            },
            RoleUsage {
                label: "warning_text",
                fg: warning_text,
                class: Some(ThresholdClass::Text),
                surfaces: vec![("warning", warning)],
            },
            RoleUsage {
                label: "danger_text",
                fg: danger_text,
                class: Some(ThresholdClass::Text),
                surfaces: vec![("danger", danger)],
            },
            RoleUsage {
                label: "info_text",
                fg: info_text,
                class: Some(ThresholdClass::Text),
                surfaces: vec![("info", info)],
            },
        ]
    }
}
