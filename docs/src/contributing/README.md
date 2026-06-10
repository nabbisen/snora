# Contributing to Snora

Welcome. This directory contains the maintainer and contributor documentation.
Pages are grouped below by reading purpose — start with the path that matches why
you are here.

## Orientation (read first if new)

| Page | What it covers |
|---|---|
| [Architecture](architecture.md) | Three-crate layout, dependency rules, z-stack, why snora-core is iced-free |
| [Design decisions](design-decisions.md) | Why the API looks the way it does; status and reconsideration triggers for each decision |
| [Alternate engine boundary](alternate-engine-boundary.md) | What snora-core's iced-free boundary means in practice |

## Design and safety

| Page | What it covers |
|---|---|
| [ABDD compliance checklist](abdd-checklist.md) | Direction-sensitivity review gate for any PR touching layout |
| [Adding a new overlay kind](adding-an-overlay.md) | Step-by-step guide for adding a new layer to the z-stack |
| [Anchored popover design](anchored-popover-design.md) | Design study for a future popover overlay; eight questions answered |
| [Feature-gating criteria](feature-gating-criteria.md) | Five indicators that justify splitting a feature behind a Cargo flag |

## Process and governance

| Page | What it covers |
|---|---|
| [Release process](release-process.md) | Full release checklist including publish order |
| [Versioning policy](versioning-policy.md) | Change-type table, Fixed vs Changed rule, deprecation bridge rules |
| [Documentation test policy](documentation-test-policy.md) | Code fence classification rules; no bare `rust` fence policy |
| [Feedback and scope](feedback-and-scope.md) | Feature-request triage; what belongs in Snora vs application code |

## Pre-1.0 readiness

| Page | What it covers |
|---|---|
| [API freeze review](api-freeze-review.md) | Living 1.0 gate tracker — current status of all ten gates |

## Recommended reading order for first-time contributors

1. Architecture — understand the three-crate split and the z-stack
2. Design decisions — read the rationale before proposing changes
3. ABDD checklist — required before any layout-affecting PR
4. Release process — needed before any release
5. API freeze review — understand where the project stands toward 1.0
