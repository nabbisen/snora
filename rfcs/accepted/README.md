# Accepted RFCs

Design settled, owner has signed off, **implementation may start** — but the
work has not shipped.

An RFC sits here from the moment it is accepted until the release that
implements it, at which point it moves to `../done/`.

Empty is the normal resting state: nothing is in flight.

## Why this folder exists

snora adopted the lifecycle policy's **five-folder variant** on 2026-08-19.
The policy's own criterion:

> Use this variant if "the maintainer signed off" is a meaningful event
> distinct from "the implementer finished."

Here those are different events performed by **different parties**, mediated by
a written implementation handoff and a review round. They do not collapse.

Before this, accepted RFCs stayed in `../proposed/` carrying a
`Status: Accepted` line — a state the four-folder variant does not define,
sitting in a folder whose stated meaning is *"implementer should not assume the
design is final"* while a handoff told an implementer to start. The folder is
the source of truth, so the folder was wrong.
