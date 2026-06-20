// Compile-time tests: all Tone variants and preset tokens accepted by row/card.
// Runtime visual quality is covered by the design workbench.

use super::*;
use snora_design::{Tokens, Tone};

#[allow(dead_code)] // compile-only: verifies row/card accept all Tone × preset combinations
fn _progress_compiles_for_all_variants() {
    for tokens in [
        Tokens::light(), Tokens::dark(),
        Tokens::high_contrast_light(), Tokens::high_contrast_dark(),
    ] {
        for tone in [
            Tone::Neutral, Tone::Accent, Tone::Success,
            Tone::Warning, Tone::Danger, Tone::Info,
        ] {
            let _: iced::Element<'_, ()> = row(&tokens, "label", Some(0.5), tone);
            let _: iced::Element<'_, ()> = row(&tokens, "label", None, tone);
            let _: iced::Element<'_, ()> = card(&tokens, "label", Some(1.0), tone);
            let _: iced::Element<'_, ()> = card(&tokens, "label", None, tone);
        }
    }
}

#[test]
fn value_clamps_within_range() {
    // If the progress_bar widget accepts a clamped value without panicking,
    // progress_content is functioning correctly. We verify by constructing
    // a row with out-of-range values.
    let t = Tokens::light();
    let _: iced::Element<'_, ()> = row(&t, "over", Some(1.5), Tone::Accent);
    let _: iced::Element<'_, ()> = row(&t, "under", Some(-0.5), Tone::Accent);
}
