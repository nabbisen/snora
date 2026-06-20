// Notice is a builder that produces an Element — not testable without a
// renderer. These tests verify compile-time correctness and the builder
// contract (all variants accepted, no panic on all preset tokens).
//
// Runtime behavior (tone colors, button reachability) is covered by the
// visual-fit workbench (snora-example-design-workbench).

use super::*;
use snora_design::{Tokens, Tone};

// If this compiles, the builder chain and all Tone variants are accepted.
#[allow(dead_code)] // compile-only: verifies builder accepts all Tone × preset combinations
fn _notice_compiles_for_all_tones_and_presets() {
    for tokens in [
        Tokens::light(), Tokens::dark(),
        Tokens::high_contrast_light(), Tokens::high_contrast_dark(),
    ] {
        for tone in [
            Tone::Info, Tone::Success, Tone::Warning,
            Tone::Danger, Tone::Accent, Tone::Neutral,
        ] {
            let _: iced::Element<'_, ()> = Notice::new(&tokens, tone, "body").render();
            let _: iced::Element<'_, ()> = Notice::new(&tokens, tone, "body")
                .title("Title")
                .render();
            let _: iced::Element<'_, ()> = Notice::new(&tokens, tone, "body")
                .action("Act", ())
                .render();
            let _: iced::Element<'_, ()> = Notice::new(&tokens, tone, "body")
                .dismiss(())
                .render();
            let _: iced::Element<'_, ()> = Notice::new(&tokens, tone, "body")
                .title("T")
                .action("A", ())
                .dismiss(())
                .render();
        }
    }
}
