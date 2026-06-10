# RFC-016-C — Downstream Adoption and Feedback Program

**Status.** Implemented (v0.16.0)
**Tracks.** Product maturity / evidence gathering.
**Touches.** `.github/ISSUE_TEMPLATE/downstream-feedback.yml` (new),
`.github/ISSUE_TEMPLATE/feature-request.yml` (new),
`docs/src/contributing/feedback-and-scope.md` (new),
`README.md` (contribution section update), `docs/src/SUMMARY.md`.

## 1. [Decision] Open questions answered

### Q: What qualifies as a "third-party production app" for the 1.0 gate?

A project that:
- is not in the `snora` repository itself;
- is used by someone other than the primary maintainer;
- ships or is actively developed with the intent to ship.

Maintainer-owned production apps **do** count if they are separate repos
and serve real users. The spirit of the gate is "someone built something
real with Snora."

### Q: Should feedback records be public when app details are private?

Feedback can be anonymized: "a desktop tool in category X" is enough.
The point is to demonstrate real usage, not to expose app internals.

## 2. Issue templates

Both use GitHub's YAML issue template format (`.github/ISSUE_TEMPLATE/*.yml`).

`downstream-feedback.yml` — the adoption evidence template from the
planning draft, rendered as a GitHub form.

`feature-request.yml` — includes the scope-triage question:
"Does this belong in Snora's framework layer (skeleton/overlay/direction),
or in application code / a separate crate?" Applications must answer
before the request is triaged.

## 3. Feedback-and-scope page

`docs/src/contributing/feedback-and-scope.md` covers:
- what counts as a framework-level issue vs an app-level issue;
- the feature-request triage table from the planning draft;
- how evidence affects the 1.0 gate;
- the conservative "Snora does not grow into a widget library" commitment.

## 4. README update

Add a "Contributing and feedback" section to README pointing to the
issue templates and the feedback-and-scope guide.

## 5. Acceptance criteria

- Both issue templates exist and render in GitHub's issue-creation UI.
- `feedback-and-scope.md` exists and is linked from SUMMARY and README.
- README has a contribution/feedback section.
