## Description

<!-- What does this PR change and why? -->

## ABDD

- [ ] This change is not direction-sensitive (no position, alignment, mirroring, or anchoring affected).
- [ ] This change is direction-sensitive and the [ABDD checklist](../docs/src/contributing/abdd-checklist.md) has been completed.

## Documentation

- [ ] New or changed `docs/src` code fences are tagged `rust,ignore` or `rust,no_run` per the [documentation test policy](../docs/src/contributing/documentation-test-policy.md).
- [ ] No documentation changes needed.

## Checklist

- [ ] `cargo check --workspace --all-features` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test -p snora-core` passes.
- [ ] `cargo test -p snora` passes.
- [ ] `cargo check -p snora --no-default-features` passes.
