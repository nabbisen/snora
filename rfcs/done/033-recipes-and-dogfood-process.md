# RFC 033 — Recipes and Dogfood Process

**Status.** Implemented (v0.23.0)
**Tracks.** Snora Design System migration; v0.22+.
**Touches.** recipe docs, dogfood feedback process.

## Summary

This RFC defines recipe documentation and dogfood validation.

## Motivation

Many useful app patterns are too domain-specific to stabilize early. Recipes provide value while protecting the public API.

## Goals

- Define recipe format.
- Add initial recipes.
- Require real-app dogfooding before promotion.
- Collect feedback for later API decisions.

## Non-goals

- No stable result card component by default.
- No setup wizard state machine.
- No search framework.
- No telemetry requirement.

## Recipe format

Each recipe must include:

```text
Purpose
When to use
When not to use
Data the app owns
Snora primitives used
Accessibility notes
Code example
Customization points
Promotion status
```

## Candidate recipes

- result card;
- recent search card;
- setup wizard step card;
- onboarding card;
- empty state;
- background task card;
- friendly error recovery notice.

## Dogfood requirement

Before recipe promotion, validate Snora Design in at least one real local-first productivity application.

The dogfood app should exercise:

- tokens;
- light/dark/high contrast;
- buttons;
- cards;
- notices/chips/progress if available;
- at least one recipe;
- focus and typography fit.

## Feedback template

```text
App:
Snora version:
Patterns used:
What was easy:
What required custom code:
Where API was awkward:
Accessibility concerns:
Feature requests:
Should any recipe become a primitive:
```

## Data lifecycle

```text
recipe published
  -> app copies/adapts recipe
  -> feedback collected
  -> repeated need identified
  -> promotion RFC or recipe remains recipe
```

## Acceptance criteria

- recipe format exists;
- initial recipes exist;
- dogfood app selected;
- feedback process documented;
- no recipe promoted prematurely.
