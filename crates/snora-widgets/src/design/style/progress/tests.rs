use super::*;
use snora_design::{Tokens, Tone};

#[test]
fn all_tones_produce_valid_style_for_all_presets() {
    let tones = [
        Tone::Neutral, Tone::Accent, Tone::Success,
        Tone::Warning, Tone::Danger, Tone::Info,
    ];
    for tokens in [
        Tokens::light(), Tokens::dark(),
        Tokens::high_contrast_light(), Tokens::high_contrast_dark(),
    ] {
        for tone in tones {
            let s = toned(&tokens, tone);
            // bar and background must be Color backgrounds (not gradients for
            // the simple token path); just confirming the call doesn't panic.
            let _ = s.bar;
            let _ = s.background;
        }
    }
}
