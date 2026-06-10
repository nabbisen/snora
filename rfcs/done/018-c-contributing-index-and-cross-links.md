# RFC-018-C — Contributing Index and Cross-Links

**Status.** Implemented (v0.18.0)
**Tracks.** Documentation quality.
**Touches.** `docs/src/contributing/` — new `README.md` index page,
`docs/src/SUMMARY.md`.

## 1. Problem

`docs/src/contributing/` now has 13 pages. A new contributor landing on
the directory has no overview of what exists and what order to read it.
The SUMMARY.md lists them all as a flat list with no narrative.

## 2. Fix: contributing/README.md

Add `docs/src/contributing/README.md` as a lightweight index that:
- Groups pages by reader path (orientation → design → process → reference)
- One-sentence summary of each page
- Recommended reading order for first-time contributors

This page will be the first link in the Contributing section of SUMMARY.md.

## 3. Acceptance criteria

- `docs/src/contributing/README.md` exists with grouped index.
- SUMMARY.md links it as the first item in Contributing.
- mdBook builds.
