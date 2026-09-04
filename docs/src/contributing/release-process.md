# Release process

The workspace uses inheritance for the version number, so a release
is fundamentally one edit. The supporting steps make sure the
release is consistent across crates, examples, and the published
artifact.

## Versioning policy

Snora is pre-1.0 and follows the conventions of pre-1.0 SemVer:

- **Patch (`0.x.y` → `0.x.(y+1)`)** — bug fixes only. No API change,
  no behavior change visible to a typical app.
- **Minor (`0.x` → `0.(x+1)`)** — feature additions, API additions,
  and *small* breaking changes when justified. The `0.4 → 0.5`
  toast-default change is an example.
- **Major (`0.x` → `0.(x+1)`)** does not exist; a true major bump
  will be `1.0` with a stability pledge.

Inside a workspace cycle, all member crates share the same version.
This is enforced by `[workspace.package].version` inheritance.

## One-edit release

```toml
# Cargo.toml at workspace root
[workspace.package]
version = "0.5.1"        # bump
```

This change propagates to every member crate's own version via
`version.workspace = true`. No per-crate *package*-version edit is
needed for that part — internal dependencies between workspace crates
declare `{ workspace = true }` too (e.g. `crates/snora/Cargo.toml`'s
`snora-core = { workspace = true }`) rather than a hand-pinned path and
version.

That covers a **patch** bump completely: the one-edit
`[workspace.package].version` change is the whole release. A **minor**
bump is not one edit, though — `[workspace.dependencies]` at the
workspace root carries its own five `version = "0.38"` pins (one per
internal crate), centralised rather than absent, and a minor must move
all of them, plus three example manifests that can't use workspace
inheritance at all. See the release checklist below for the specific
files and the exact failure mode of missing one.

## GitHub Actions workflows

Six workflows run automatically; they have distinct responsibilities:

| Workflow | File | Trigger | Responsibility |
|---|---|---|---|
| **CI** | `ci.yaml` | PR, push to `main` | Rust quality gate: `rust-quality`, `feature-matrix`, `design-isolation`, `docs` jobs. A red run now **blocks publishing mechanically** — see Release below. |
| **Release** | `release.yaml` | push of a bare `X.Y.Z` tag | Publishes the workspace to crates.io over Trusted Publishing (OIDC). Refuses before any upload if the tag disagrees with `[workspace.package].version`, or if the tagged commit has no completed, successful CI run (RFC-090). |
| **Unpinned build** | `unpinned-build.yaml` | `workflow_dispatch` | Builds against unpinned dependencies; run on `main` before tagging (RFC-041). |
| **Docs** | `docs.yaml` | Push to `main` | Build and deploy mdBook to GitHub Pages. |
| **Binary size** | `binary-size.yaml` | PR, push, tags | Measure stripped binary size; append a row to the CSV on release tags. |
| **Build cost** | `build-cost.yaml` | Push to `main`, tags | Measure compile time; append a row to the CSV on release tags. |

Do not confuse the **CI docs job** (PR gate) with the **Docs workflow**
(deployment). They run mdBook with the same `^0.5` locked version; keeping
them in sync is a release-process invariant.

## Who commits what

**Nobody commits changes they did not make.** The rule binds every role
symmetrically — the architect as much as the dev team.

- **Implementation work is committed by whoever implemented it**, one commit per
  RFC, when the submission is accepted. They wrote it; they can describe it.
- **The architect commits** review results, RFCs, handoffs, and the release
  mechanics — version bump, CHANGELOG roll, RFC folder moves, tag.

**Review the diff, not the path.** Two controls, and the first one alone is not
enough:

1. **`git add -A` is the instrument that makes the coarse violation possible.**
   Stage explicitly. **If `git status` shows paths you did not touch, stop and
   say so** rather than including them.
2. **A file you *did* touch can still contain work you did not.** Explicit
   staging does not help when someone else's uncommitted change sits in the same
   file. **Read `git diff <path>` before staging it**, or stage by hunk with
   `git add -p`.

The second control exists because the first one was not sufficient — see the
second instance below.

**A consequence, and it is intended:** if implementation work is still
uncommitted at cut time, the release is *blocked* until its author commits it.
That forces the handover to be explicit instead of implicit, and means a release
can no longer quietly contain work nobody signed for.

**The one exception, on the record rather than silent.** If an author is
genuinely unavailable and their work must land, whoever commits it **says so in
the message** — whose work it is, and why they are committing it.

### The instance this rule exists for

Commit **`df1752d`** (2026-09-01, since split) was titled *"RFC-085 header:
correct the release target to 0.42.0"* and contained **700 lines implementing
two Critical RFCs** — `contrast_tests.rs`, the dialog fix, `render.rs`, four
widget files. The architect ran `git add -A` for a one-line header edit while
the dev team's accepted-but-uncommitted work sat in the tree.

Nothing was lost and CI stayed green. **The history said something untrue**, and
it said it because implementation work was waiting in the working tree for
somebody else to commit it. It was split into three honest commits and
force-pushed the same day.

Raised as a clarification request by the dev team, who noticed that nothing
assigned commit responsibility in writing and declined to keep calling the
observed pattern a rule.

### The second instance — the commit that wrote this rule broke it

Commit **`302a83d`**, which added the section you are reading, contains the
dev team's RFC-087 checklist line (*"Has any recorded conditional deferral's
ending condition been met"*). The architect staged
`docs/src/contributing/release-process.md` **explicitly**, not with `-A`,
believing the file held only his own new section. Their accepted-but-uncommitted
line was already in it.

**Explicit staging was not enough, and that is the whole point of control 2.**
The rule as first written caught unexpected *files*; this was an unexpected
*hunk* inside a file the architect legitimately owned. Found minutes later when
the dev team's own commit (`c651e93`) turned out not to contain a line that
should have been theirs.

Left in place rather than re-split: the line is correct, it names RFC-087 in its
own text, and both commits describe their subject honestly. **Splitting history
twice in one day for one checklist line would cost more than the tidiness is
worth** — but the rule gained control 2 the same hour.

### The third authorship incident, and the first with a technical cause

`914fe92` through `3efb548` were pushed authored *and* committed as
`RFC-092 Test <test@example.com>`. The cause was not carelessness about identity:
a `git config user.email` run inside a **linked worktree** wrote to the shared
repo-level `.git/config`, because a linked worktree does **not** get its own
config unless `extensions.worktreeConfig` is enabled. It is not, here.

The worktree was removed; the identity override outlived it, and every commit in
that checkout for the next twelve minutes carried the fake identity — two the
implementer's, three the architect's, including the commit fixing an unrelated CI
break.

**If you need a throwaway identity, use `git -c user.name=… -c user.email=…
commit`, or a separate clone.** A bare `git config user.*` in a worktree is
repo-global, and it is silent about that.

Rewritten and force-pushed 2026-09-02 after the owner ruled: five tip commits, no
tag inside the range, one worktree, no PRs, and the tree hash identical before and
after. The two documents citing the old SHAs were updated in the same pass —
rewriting history invalidates every reference to it, which is its real cost.

## Deferrals waiting on a condition

The checklist step *"Has any recorded conditional deferral's ending condition
been met"* has existed since RFC-087 and had **nothing to read**. Answering it
meant recalling which deferrals were open, which is the memory-based control this
project keeps proving does not hold. This is the list it points at.

**Add a row whenever a deferral is made.** A deferral with no row here is one
nobody will re-check, which is the failure RFC-087 documented across three
releases.

| Deferral | What is deferred | Ends when | Blocked on us first? |
|---|---|---|---|
| **RFC-093 Q-1** *(obligation met 2026-09-02, ahead of the cut)* | Adding a non-colour cue (icon or textual prefix) to toast intents and notice tones | **A consumer asks for it, having been told the prefabs distinguish by colour alone.** Before that, silence is not evidence — nobody asks for a channel they believe already exists, which is exactly how RFC-078 came to count apimokka's decline as demand data | **Yes.** The 0.43.0 migration guide and its letter must carry the colour-alone statement. `accessibility.md` already says it, but consumers read guides and letters, not the reference pages. ~~If 0.43.0 ships without it, this condition can never fire~~ — **done**: `docs/src/guides/migration-0.42-to-0.43.md` carries the statement and an explicit invitation to ask. Written before the cut rather than during it, because `check-migration-guides.sh` derives its pairs from `git tag` and so cannot flag a missing guide until after the tag exists. **No 0.43.0 letter was sent** (owner, 2026-09-03): the release changed nothing anyone compiles against, so it did not clear the correspondence bar, and a letter about a release with no changes is the note tekstide asked us to stop sending. **The condition is still reachable** — every team reads the 0.42→0.43 guide when they pass it, on their own upgrade schedule rather than ours. The colour-alone paragraph is written and goes in the next letter that has its own reason to exist. |
| **Feature-gating indicator 1** | Assessing compile time at all. The table's own threshold is 30,000 ms on a developer machine, cold; the cell reads **Unassessed** because the CI proxy that used to fill it measured a different quantity and was retired (RFC-062) | **A measurement exists.** Not an RFC — RFC-078 was this exact shape, a measurement dressed as a design question, and was archived for it. Someone takes the number on a developer machine and the cell stops lying | **No.** Nothing external is needed; it has simply never been anyone's turn |
| ~~**0.44.0's tag**~~ **— discharged 2026-09-04** | Tagging and publishing 0.44.0 | **Fired.** `tinyvec` shipped 1.13.1 and then 1.13.2; a fresh resolution now builds. Verified locally (`cargo update -p tinyvec` → 1.13.2, workspace checks clean) and by re-dispatching `unpinned-build` on `main`, green. **Note 1.13.0 was never yanked**, so a consumer whose own lockfile pinned it stays broken until they update — the fix was a new version, not a withdrawal | No — held one day, cost nobody anything, and the check went green on its own exactly as the hold assumed |



**When the letter goes.** Both open questions above wait on the same letter, and
the owner ruled 2026-09-03 that it goes **after RFC-094 and RFC-095's work is
complete**, not before — so it ships with a release that has its own reason to
exist rather than as a note about pending questions. Concretely: RFC-094 lands →
0.44.0 is cut → the letter carries the colour-alone division of labour (RFC-093
Q-1) *and* the `Emphasis`/`Size` question (RFC-095 Q-1) → answers rule RFC-095's
Q-1 → 0.45.0 acts on it.

**The last column is the part worth copying.** A condition that depends on someone
else knowing something is unreachable until we tell them — so a deferral whose
trigger is "a consumer asks" carries an obligation on *us*, and it belongs in the
row rather than in whoever wrote it.

## Who cites what

**Built 2026-09-02 from all six teams' answers to a direct question.** Before
this, "which teams acted on the claim we just withdrew" was guesswork, so
withdrawals went to everyone or to nobody. orbok asked to be on a list; this is
the list.

| Team | Cites snora for | Consequence of a withdrawal |
|---|---|---|
| **orbok** | A WCAG 2.1 AA conformance record naming snora **19 times**; the *status* of **1.4.3**, **1.4.11**, **2.4.7** and **2.5.8** rests on our documentation | **Highest. Tell them first.** Has already absorbed three withdrawals: `text_muted` (0.34.0), the dialog-card border (0.39.0), and 1.4.1 (0.41.1) |
| **arama** | **AA contrast**, in RFC 010, RFC 011 and `docs/src/dev/workspace.md` | They asked explicitly: *"the AA contrast claim is a live citation of ours — that is the one to tell us about directly if it ever moves"* |
| **knotra** | Typography and the six type roles (RFC-056); readability and the **RFC-072 contrast covenant** (RFC-058), quoted as a binding rule in their review process | A covenant change reaches their review process, not just their docs |
| **tekstide** | Focus-ring documentation only. **Not a snora consumer** — no dependency, none planned | Asked to be dropped from *compatibility* letters; the design correspondence is a separate list and worth keeping |
| **aaai** | Nothing. No conformance record, no VPAT | None |
| **apimokka** | Nothing. No conformance record | None |

**Keep this current by asking, not by inferring.** Every entry above came from a
team answering a direct question; none of it was derivable from our side. Three
of the six turned out to cite us and three did not, and the split was not the one
we would have guessed.

**A withdrawal still reaches a team that cites nothing.** aaai cite us nowhere,
and the 1.4.1 correction still found a real hole on their side — their own
accessibility audit listed four surfaces as colour-independent and omitted
toasts, which they raise at ten call sites. knotra and orbok each found an
untested comment in their own code asserting what ours had asserted. **Send the
withdrawal to everyone; use this table to decide who to tell first and who needs
a direct answer.**

## Claims are checked, not trusted

Sitting beside "Who commits what" because it is the same kind of rule: about how
work gets recorded, not about the code itself.

**A claim about what changed or what was measured is either produced by a
command, or labelled as an inference.** Not "verify everything" — say which kind
of thing you are saying.

This exists because six such claims shipped wrong in one cycle (RFC-092), none
caught by a gate, because no gate reads sentences. *"No behaviour change"* over a
diff with two code changes in it. *"The fade drops four of five intents under the
floor"* when the measured answer was zero of ten. *"Confirmed passing on a clean
tree"* for a gate that could not run in CI at all. Three miscounts. **Four were
the architect's**, and the rule binds review results and release commits exactly
as it binds implementation.

The reason is cost, not care. **Verifying a summary costs as much as producing
it** — "no behaviour change" over 29 files is only checkable by reading 29 files
— so a rule demanding that every time gets skipped precisely when the diff is
large, which is when it matters.

What that means in practice:

- **"Documentation only" / "no behaviour change"** is `scripts/check-docs-only.sh
  <rev>`, not a judgement. Add a `Docs-only: yes` trailer and CI checks it for
  you; no trailer, no check.
- **Any count** — fences, findings, re-exports, packages — comes from a command
  quoted alongside it. Three of the six failures were counts made by eye.
- **A measurement names the tree it was measured on.** RFC-086's numbers were
  right; it cited a *pre-fix* table to support a *post-fix* claim.
- **Everything else gets its provenance stated.** The Q-2 research on Trusted
  Publishing separated what was confirmed from what was inferred, unprompted, and
  that is why the ruling took one pass instead of three.

## Release checklist

**The migration guide is the canonical statement of what a release
means for a consumer; a letter to a downstream team carries only what
is specific to that team (RFC-080).** Not because a letter costs more
than the guide — because of *reach*. The RFC-067 re-check obligation
below currently depended on us choosing to write to a team, so it
reached only the teams we decided to write to, and nobody else —
including no future adopter jumping through that version, who is
exactly the person most likely to be carrying a withdrawn claim
without knowing it. Put the re-check in the guide and it reaches
everyone who reads the guide, on their own schedule, whether or not we
ever wrote to them. A letter may still point at the guide; it does not
restate the guide's content.

```text
[ ] Bump [workspace.package].version
[ ] If minor: bump snora-core / snora-widgets dep versions across crates
[ ] If minor: bump the hand-pinned `snora` version in
    examples/size_probe_engine/Cargo.toml AND
    examples/responsive_body/Cargo.toml AND
    examples/size_probe_design_engine/Cargo.toml. All three use an
    explicit `default-features = false` path dependency instead of
    `snora = { workspace = true }` (workspace inheritance cannot
    override `default-features`), so none follows the workspace
    version table and each must be hand-edited on every minor bump —
    missing any fails EVERY `cargo` command in the workspace with
    "failed to select a version for the requirement `snora = "^0.NN"`"
    (this has already happened once, on the 0.26.0 bump). Grep for the
    old minor across all `Cargo.toml` files before assuming the
    workspace table alone is sufficient.
[ ] Move the [Unreleased] section in CHANGELOG.md to the new version,
    and reset [Unreleased] to "Nothing yet."
[ ] If minor: write docs/guides/migration-X.Y-to-X.Z.md. **Unconditional
    — no exceptions for "nothing broke."** A guide for a minor that
    changed nothing required says so in a sentence (RFC-079); it is
    never skipped. Run scripts/check-migration-guides.sh to confirm
    every released minor has one.
[ ] After the guide is written, decide correspondence (RFC-080): which
    teams have something **specific to them** that the guide does not
    already cover for everyone? Write only to those teams, and write
    only the team-specific part — the guide is canonical, the letter
    is specific. **If no team has anything specific, send nothing.** A
    bare "0.X.0 is out, here is the guide" letter is exactly the note
    tekstide asked us to stop sending, and the guide is on the
    published site whether or not anyone gets a letter. The
    correspondence bar itself is unchanged: broken-now, a withdrawn
    claim a team acted on, or they asked.
    # "A withdrawn claim a team acted on" is answerable now rather than
    # guessable — see "Who cites what" below.
[ ] For any capability that arrived, left, or any governance/policy
    decision that answers a question a consumer would ask: apply
    feature-gating-criteria.md § "Documentation scope when a capability
    arrives, leaves, a standing answer is invisible, or a claim is
    withdrawn" — grep the default-path docs for the claim it
    contradicts, or add a consumer-facing statement. Five misses reached
    this checklist before the rule did; this line is why it won't be a
    sixth.
[ ] Did this release withdraw, narrow, or correct anything we
    previously told consumers (RFC-067)? There is no grep for this —
    answer it yourself, since you are the person writing the note. If
    yes, the note names **what to re-check**, not only what changed —
    the same distinction a rendered-appearance change's "re-check any
    screenshot tests…" line already makes, now required of a withdrawn
    or narrowed *claim* too. A correction that does not say what to do
    about it reaches nobody who already acted on the old claim — five
    instances across four consumers (`feature-gating-criteria.md`'s
    documentation-scope table) is why this line exists. **The note
    lands in the migration guide, not (only) a letter (RFC-080)** — a
    letter to a team with a known stake may point at it in one line,
    but the guide is what reaches a future adopter who was never
    written to.
[ ] Has any recorded conditional deferral's ending condition been met
    (RFC-087)? **The open ones are listed under "Deferrals waiting on a
    condition" below — read that table, do not rely on recalling them.**
    A deferral must name the condition that ends it; this line is what
    re-checks it, so it stops being renewed by habit instead. The `check-*` scripts' own manual-to-gate deferral was
    renewed three times (RFC-073, RFC-074, RFC-079) with its condition
    ("all three pass on a clean tree") true the whole time, before
    RFC-087 finally re-checked it.
[ ] Re-evaluate feature-gating-criteria.md's "Current status" table
    (RFC-062) — the table itself says to do this and nothing pointed at
    it for ten minors, which is why it went stale in the first place.
    Re-derive indicator 2 (`widgets_diff_bytes`) against the latest
    binary-size.csv row; re-check indicators 3 and 4 against the current
    manifests rather than inheriting the prior row's answer; note any
    new field requests for indicator 5. Also re-run the accessibility-
    tree trigger's check —
    `cargo tree -p snora --all-features | grep -i accesskit` — and update
    design-decisions.md's register with the result and this release's
    date if it changed.
[ ] Read the new `design_overhead_ratio` row against
    build-cost-budget.md's watch points. Gate 9b closed at 0.37.0 on four
    rows, so this is ongoing monitoring, not a closure check — and it is
    monitoring with a stated floor: the ratio moved -4.44% across a release
    that changed no executable code, so treat a move under ~10% as noise
    and investigate above it. The absolute millisecond columns are raw
    record, never a trend (RFC-050).
[ ] Update ROADMAP.md (move shipped items off; rewrite "Near-term"
    if priorities changed)
[ ] Move v0.NN RFCs from rfcs/accepted/ to rfcs/done/ (five-folder
    variant — an accepted RFC lives in accepted/, not proposed/); update their
    Status fields and the rfcs/README.md index
[ ] Answer the four versioning-policy questions for any public API change
    (see docs/src/contributing/versioning-policy.md)
[ ] Confirm the migration guide required above was actually written —
    not conditional on whether anything broke or renamed (RFC-079; this
    line previously said "if any public API broke or renamed," which
    disagreed with the unconditional rule above and was the second copy
    of the same requirement drifting on its own)
[ ] Run scripts/check-version-snippets.sh and fix every snippet it names
    (RFC-074 — derives the expected minor from Cargo.toml itself, so this
    replaces enumerating files by hand; iced version stays unchanged)
[ ] Re-run cargo metadata; confirm every crate reports new version
[ ] Confirm Cargo.lock is current and its diff (if any) is intentional —
    `git status --porcelain Cargo.lock`; if it shows a diff, review what
    moved and why before committing. A lockfile that drifts unreviewed is
    worse than none: it carries the implied assertion that someone looked.
[ ] cargo +<declared MSRV> check --workspace --all-features
    # must pass against the committed Cargo.lock; an MSRV not re-checked
    # at release time is a claim, not a fact
[ ] Confirm no resolved dependency declares a higher rust-version than the
    declared MSRV: cargo metadata --format-version 1 --all-features
[ ] cargo fmt --all --check
    # Note --all: a bare `cargo fmt --check` misses the example crates,
    # where most drift accumulates. CI enforces this on every PR and push
    # as of 0.28.1; before that nothing ran it, and it had silently stopped
    # passing on a clean tree for several releases while still sitting in
    # this checklist. Do not tick a gate you have not seen pass.
[ ] cargo check --workspace --all-features
[ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
[ ] cargo test -p snora-core
[ ] cargo test -p snora-design
[ ] cargo test -p snora-widgets --features design
[ ] cargo test -p snora --lib --all-features
[ ] cargo test -p snora --test render_semantics   # CI hardware; may OOM locally
[ ] cargo test --workspace --all-features | grep -A2 "Doc-tests"
    # Confirm the passed/ignored counts per crate match
    # documentation-test-policy.md's "Current counts" table; update that
    # table if a doctest was added, removed, promoted, or newly ignored.
    # Every fence left at `ignore` must carry a stated reason (RFC-064) —
    # `grep -B1 '```rust,ignore\|```ignore' crates/ -r --include="*.rs"`
    # to spot-check one wasn't added without one.
[ ] cargo check -p snora --no-default-features
[ ] mdbook build docs               # validates the book renders
[ ] mdbook test docs                # validates the doc-fence policy
[ ] All examples in examples/README.md acceptance matrix compile
    (covered by workspace check above; verify no example was removed)
[ ] Workbench manual QA checklist completed (docs/src/getting-started/06-workbench.md)
[ ] cargo package --workspace
    # Inspects all five .crate archives (snora-core, snora-design,
    # snora-style, snora-widgets, snora — snora-style added in RFC-055,
    # 0.32.0). Examples are ALSO packaged (each emits "manifest has no
    # description"); `publish = false` stops them being *published*, not
    # packaged, so no exclusion flag is needed but the output is noisier
    # than "five archives". Corrected at 0.38.0 — the previous wording
    # said they were skipped, and they are not.
    # Do NOT package the five crates individually (see "Publishing").
[ ] Merge to main, then dispatch the `unpinned-build` workflow on main and
    confirm a green run BEFORE tagging.
    # WHEN IT IS RED FOR SOMEONE ELSE'S REASON: still hold. 2026-09-03,
    # cutting 0.44.0, this fired for the first time and caught an
    # upstream break, not ours — tinyvec 1.13.0, published that day and
    # not yanked, does not compile (`cannot find macro 'vec'`).
    # Our lockfile pinned 1.11.0, so CI was green and the tagged tree
    # was sound.
    # "It is not our bug" is the argument that would tag past this
    # check every time it ever fires, since a pinned-green/unpinned-red
    # split is almost always upstream. It is not the right question.
    # The right question is whether tagging makes anything better:
    # 0.44.0 had no consumer-visible change, so waiting cost nobody
    # anything and the check went green on its own once upstream moved.
    # If a release DOES carry something consumers need, weigh that
    # against shipping into a window where a fresh `cargo add` fails —
    # and record which way you went and why.
    # `workflow_dispatch` only works for workflows already on the default
    # branch, so this cannot be verified from a feature branch. A workflow
    # that has never executed is exactly the failure RFC-041 was raised to
    # fix — do not skip this.
[ ] git commit, THEN pull --rebase, THEN tag — in that order
    git tag -s X.Y.Z -m "X.Y.Z"
    # -s is REQUIRED: this repo sets `tag.gpgsign true`, so a bare
    # `git tag X.Y.Z` fails with "fatal: no tag message?" — which reads
    # like a message problem and is actually a signing one. Signed tags
    # are annotated, so -m is mandatory too.
    # Tagging before rebasing leaves the tag on a commit the rebase
    # orphans: `git merge-base --is-ancestor X.Y.Z^{commit} main` fails,
    # and the release tag is not in main's history. The measurement bots
    # append to main between your commit and your push, so a rebase is
    # NORMAL on this repo, not an exception.
[ ] Verify the tag is on main BEFORE pushing it:
    git merge-base --is-ancestor X.Y.Z^{commit} main && echo on-main
    # If it is not, delete the local tag and re-tag. If you already pushed
    # it, cancel any tag-triggered measurement runs first — otherwise
    # re-pushing the tag appends a SECOND row for the same version.
    # SAME TRAP FOR TEST TAGS: binary-size.yaml and build-cost.yaml
    # trigger on ANY tag, so a scratch tag pushed to exercise
    # release.yaml appends a row named after it to both CSVs. This
    # happened on 2026-09-02 with a `9.9.9` test tag (RFC-090 Unit 2)
    # and had to be reverted by hand. Delete the rows AND the tag after
    # any such test, and check both CSVs, not just the tag list.
[ ] git push origin main && git push origin X.Y.Z
    # Tags carry no `v` prefix, matching Rust crate convention. This is
    # now ENFORCED, not remembered: release.yaml triggers on
    # ['[0-9]+.[0-9]+.[0-9]+'], so a `v`-prefixed tag does not publish
    # at all. 0.41.1 was first tagged `v0.41.1` against 67 bare tags,
    # which is why the pattern exists (RFC-090).
    # Pushing the tag IS the publish. See "Publishing" below.
[ ] Confirm CI is green ON THE COMMIT YOU ARE ABOUT TO TAG, matched by SHA:
    bash scripts/check-commit-ci-green.sh HEAD
    # Do NOT use `gh run list --workflow CI --limit 1` for this. Right
    # after a push, the newest registered run is still the PREVIOUS
    # commit's, so `--limit 1` returns a green result for the wrong
    # commit and `gh run watch` exits 0 immediately on it.
    # That happened cutting 0.44.0 (2026-09-04): the tag was pushed on a
    # commit whose own CI was still in_progress, and release.yaml
    # refused at its second gate — correctly, and against a reviewer who
    # believed he had checked. The script above queries by head_sha and
    # cannot make that mistake.
    # Recovery if you tag early: wait for that commit's CI, then re-run
    # the failed Release run. The tag does not need deleting or moving.
[ ] After tag push: confirm a NEW ROW EXISTS for this version in
    docs/src/reference/binary-size-budget/binary-size.csv on main —
    `git show main:docs/src/reference/binary-size-budget/binary-size.csv | tail -1`
    and check its `version` column equals the tag just pushed. This is the
    falsifiable check; "the workflow run is green" is not, because a
    workflow that never triggers reports nothing rather than failing (the
    exact failure mode RFC-041 exists to fix — the tag-pattern mismatch
    that caused it went unnoticed for six releases specifically because
    only the run's status, not the row's existence, was ever checked). If
    no new row exists, treat it as a release blocker: check the Actions
    tab for whether the `binary-size` workflow triggered at all on the
    tag, not just whether a triggered run went green. If the diff column
    exceeds 150 KB, follow up per feature-gating-criteria.md indicator 2.
[ ] CONTENT-check that same binary-size row (RFC-044 — a row existing says
    nothing about whether its fields are right; this is the check that
    would have caught RFC-043's `widgets_diff_bytes = 0` at release time
    instead of a release later):
    `git show main:docs/src/reference/binary-size-budget/binary-size.csv | tail -1 | cut -d, -f8`
    must print `ubuntu-latest` — field 8, `runner_os`. If it prints
    `Linux`, the `SNORA_RUNNER_OS` override did not reach the script;
    treat it as a release blocker and investigate before publishing, do
    not just note it and move on.
    `... | cut -d, -f4` (`widgets_diff_bytes`) must be **non-zero**. A `0`
    means the probes are byte-identical again and are measuring nothing —
    treat it as a release blocker.
[ ] After tag push: confirm a NEW ROW EXISTS for this version in
    docs/src/reference/build-cost-budget/compile-time.csv on main —
    `git show main:docs/src/reference/build-cost-budget/compile-time.csv | tail -1`
    and check its `version` column equals the tag just pushed. Same
    failure-mode note as above: verify the row, not just workflow status.
    If no new row exists, treat it as a release blocker. If
    build_widgets_ms exceeds 30 000, follow up per
    feature-gating-criteria.md indicator 1.
[ ] CONTENT-check that same compile-time row (RFC-044, same rationale as
    the binary-size content-check above):
    `git show main:docs/src/reference/build-cost-budget/compile-time.csv | tail -1 | cut -d, -f9`
    must print `ubuntu-latest` — field 9, `runner_os`. This field number
    does NOT move when a column is added (RFC-050 appends
    `design_overhead_ratio` as the LAST field, field 11, specifically so
    this check and every other positional read of an existing field
    keeps working unchanged — see build-cost-budget.md's append-only
    column note). If it prints `Linux`, treat it as a release blocker,
    same as above.
    `... | cut -d, -f2` (`check_workspace_ms`) must be **at least 10 000**
    (plausibly cold — tens of seconds, not milliseconds). A value in the
    hundreds or low thousands means a dependency cache was restored and
    the "cold" build is warm — the exact defect RFC-043 fixed once
    already; treat it as a release blocker and check that
    `build-cost.yaml` still has no `Swatinem/rust-cache` step.
[ ] cargo publish --workspace   — FROM A CLEAN TREE AT THE TAG
    # cargo packages the WORKING DIRECTORY, not the tagged commit. If other
    # work is in flight, publish from a throwaway worktree:
    #   git worktree add --detach /tmp/pub X.Y.Z && cd /tmp/pub && cargo publish --workspace
    # Cargo refuses a dirty tree by default. NEVER pass --allow-dirty.
    # ONE command — cargo resolves member order itself. Do not publish the
    # five crates individually; an interrupted per-crate sequence leaves a
    # public tag with `snora` itself missing from crates.io, and anyone
    # depending on the new minor gets a resolution failure until it is
    # finished. See "Publishing" below.
[ ] Confirm all five crates report the new version on crates.io
```

### Publishing

**Pushing the tag publishes. There is no command to run here.**

`release.yaml` triggers on a bare `X.Y.Z` tag, checks out that tag, and runs
`cargo publish --workspace` — one command, five crates, as before. It
authenticates with crates.io **Trusted Publishing** (OIDC); there is no
long-lived token anywhere in this repository.

Before any upload it refuses if:

1. the tag does not match `[workspace.package].version` in `Cargo.toml`, or
2. the tagged commit has no **completed, successful** CI run — including when
   it has no run at all, which is a refusal and not a pass.

Watch the run. If it refuses, the message names what disagreed.

#### Why this is not a checklist item any more

This section used to ask three things of whoever was cutting: confirm CI is
green, publish from a clean tree at the tag, and name the tag `X.Y.Z`. All three
were rules a person had to remember, and **all three were broken during the
0.41.1 cut by the person who wrote them** — published on a red `main`, from a
laptop, under a tag first named `v0.41.1`.

They are now properties of the mechanism instead (RFC-090):

| Was | Is |
|---|---|
| Remember to check CI is green | `release.yaml` refuses a commit without a green run |
| Publish from a clean tree at the tag | The workflow's checkout of the tag *is* the clean tree |
| Tag `X.Y.Z`, not `vX.Y.Z` | The trigger pattern does not fire on anything else |

The one rule in this section that never failed was the dirty-tree rule, and it
never failed because **cargo refused** — not because it was written down well.
That is the whole argument, and it is why the list above is short.

#### Break-glass: publishing by hand

Permitted **only** when `release.yaml` itself is broken, **and** the owner says
so for that specific release. Not an equal alternative — an exception with a
named condition, because an exception without one becomes the default:

```bash
git worktree add --detach /tmp/publish-X.Y.Z X.Y.Z
cd /tmp/publish-X.Y.Z && cargo publish --workspace
```

Publish from a throwaway worktree at the tag, never from a working directory
with other work in flight: `cargo publish` packages the **working directory**,
not the tagged commit. Cargo refuses a dirty tree by default, which is the
guard; **never pass `--allow-dirty` to get past it.**

This is not hypothetical: 0.27.1 was cut while the 0.28.0 work sat uncommitted,
and cargo's dirty-tree refusal is what stopped a docs-only patch from shipping
two unreleased features' source.

Cargo computes the dependency order from the manifests
(`snora-core` and `snora-design` have no internal dependencies;
`snora-style` depends on `snora-design`; `snora-widgets` depends on
`snora-core`, `snora-design` and `snora-style`; `snora` depends on all
four) and waits
for each to become available on crates.io before publishing the next. The
order cannot drift from the manifests, because nothing restates it.

#### Why not one `cargo publish` per crate

That was this project's process until v0.27.0, and it predates
`cargo publish --workspace`. It had two costs:

- **A hand-maintained order.** A restated dependency order is a second
  source of truth that can disagree with the manifests.
- **Packaging appeared broken on every minor bump.** Once
  `[workspace.dependencies]` moves from `0.26` to `0.27`,
  `cargo package -p snora-widgets` fails with

  ```text
  error: failed to select a version for the requirement `snora-core = "^0.27"`
  ```

  because that version is not on crates.io yet. This is not a fault, and
  earlier revisions of this page carried a long explanation of why it was
  expected. `cargo publish --workspace` removes the condition rather than
  explaining it.

A half-published release is the failure this avoids: if the sequence is
interrupted partway, the tag is public while `snora` itself is missing, and
anyone depending on the new minor gets a resolution failure until it is
finished.

#### `--no-verify`

`cargo package --no-verify` skips the *build* verification step; it does
**not** skip dependency resolution. It is occasionally useful for
inspecting a `.crate` archive's contents in isolation, but it is not part
of the normal release path.

## Tarball releases (if used)

For local release artifacts shipped outside crates.io, name them
with a version suffix:

```text
snora-X.Y.Z.tar.gz
```

This was the convention adopted from 0.4.2 onward.

## Examples are not published

The `examples/*` crates set `publish = false` in their
`Cargo.toml`. They are part of the workspace for `cargo check` and
`cargo run -p` convenience but never go to crates.io.
