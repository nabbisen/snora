//! Tests for the chips' layout (RFC-061). Their style functions, and the tests of those,
//! moved to `snora-style` with them (RFC-129 Q-5): `snora_style::chip`.

use super::*;
use snora_design::Tokens;

// ---------------------------------------------------------------------------
// Pointer target size (RFC-061)
// ---------------------------------------------------------------------------

#[test]
fn remove_btn_target_size_clears_24px_floor_for_all_presets() {
    // RFC-118: every preset, from the one public source.
    for (_, t) in Tokens::presets() {
        let size = remove_btn_target_size(&t);
        assert!(
            size >= 24.0,
            "remove_btn_target_size {size} < 24.0 pointer-target floor"
        );
    }
}
