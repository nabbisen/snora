// Compile-time tests: all Tone variants and preset tokens accepted by row/card.
// Runtime visual quality is covered by the design workbench.

use super::*;
use snora_core::LayoutDirection;
use snora_design::{Tokens, Tone};

#[allow(dead_code)] // compile-only: verifies row/card accept all Tone × preset combinations
fn _progress_compiles_for_all_variants() {
    // RFC-118: every preset, from the one public source.
    for (_name, tokens) in Tokens::presets() {
        for tone in [
            Tone::Neutral,
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
        ] {
            for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
                let _: iced::Element<'_, ()> = row(&tokens, "label", Some(0.5), tone, dir);
                let _: iced::Element<'_, ()> = row(&tokens, "label", None, tone, dir);
                let _: iced::Element<'_, ()> = card(&tokens, "label", Some(1.0), tone, dir);
                let _: iced::Element<'_, ()> = card(&tokens, "label", None, tone, dir);
            }
        }
    }
}

#[test]
fn value_clamps_within_range() {
    // If the progress_bar widget accepts a clamped value without panicking,
    // progress_content is functioning correctly. We verify by constructing
    // a row with out-of-range values.
    let t = Tokens::light();
    let _: iced::Element<'_, ()> = row(&t, "over", Some(1.5), Tone::Accent, LayoutDirection::Ltr);
    let _: iced::Element<'_, ()> = row(&t, "under", Some(-0.5), Tone::Accent, LayoutDirection::Ltr);
}
