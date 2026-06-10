# Feedback and scope

This page explains how Snora evaluates feature requests and downstream
feedback, and how that evidence feeds into 1.0 readiness.

## What Snora's framework layer covers

Snora's value is in these areas:

- **Skeleton layout** — header, sidebar, body, footer, and their
  `LayoutDirection`-aware composition.
- **Overlay stacking** — deterministic z-stack (menus, modal dim, dialog,
  sheet, toasts), close sinks, and graceful degradation.
- **Close behavior** — `on_close_menus`, `on_close_modals`, and the
  `dismiss_on_escape` helper.
- **Layout direction (ABDD)** — logical `Edge::Start`/`End`, `LayoutDirection`,
  and automatic mirroring for sidebars, sheets, and toast anchors.
- **Operational quality** — toast lifecycle helpers, render-semantics
  guarantees, CI, and the binary-size / compile-time budgets.

## Feature-request triage

| Request type | Default response |
|---|---|
| Skeleton / overlay / direction issue | Open an RFC if signal is strong |
| Second toast variant (info, persistent ack) | Accepted when two apps need it |
| Tooltip vocabulary | Accepted when a second consumer type appears |
| Anchored popover | Accepted with a concrete consuming app |
| Form widgets / validation | Out of scope — use iced primitives |
| Table / chart / data-display | Out of scope — UI library territory |
| Decorative widgets (avatar, badge, chip) | Out of scope — trivial inline |
| Snora-owned theming layer | Firm non-goal — use iced theming |
| Translation / locale formatting beyond direction | Out of scope |
| `snora-test` public crate | Firm non-goal — `pub` fields + `iced_test` cover it |
| Game-loop / real-time rendering | Out of scope |
| Focus trap | Deferred — requires stable iced focus API |

## What counts as a "third-party production app" (1.0 gate)

The 1.0 gate requires at least one third-party production app. This means:

- Hosted outside the `snora` repository.
- Used by someone other than the primary maintainer, or owned by the
  maintainer but serving real users in a production context.
- Actively developed with the intent to ship.

If you are building such an app, please use the
[downstream feedback](https://github.com/nabbisen/snora/issues/new?template=downstream-feedback.yml)
template to share your experience.

## How evidence affects the roadmap

Concrete app stories are the strongest signal for changing Snora's scope.
They carry more weight than:
- abstract feature requests ("what if Snora had X");
- popularity votes;
- comparisons to other frameworks.

If you want to influence the roadmap:
1. Open an issue with a concrete scenario.
2. Send a PR that demonstrates the design.
3. Share a downstream feedback record for your real app.

## The "Snora does not grow into a widget library" commitment

Snora will not become a widget library. Every non-goal listed above is
intentional. The test: if a potential addition does not need to know about
`AppLayout`, the z-stack, or `LayoutDirection`, it does not belong in Snora.
