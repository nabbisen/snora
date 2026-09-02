# Developer Handoff — Workflow files are validated by GitHub or by nobody

**Governing document.** Not an RFC — one script and one small workflow. The
incident is recorded in `0b382a6`.
**Status.** Approved by the owner, 2026-09-02.
**Release target.** **0.43.0.** No crate code.
**Touches.** `scripts/check-workflows.sh` (new),
`.github/workflows/workflow-lint.yaml` (new), `scripts/README.md`.

---

## What happened

`183cc70` added a step whose name was an unquoted YAML scalar containing `": "`:

    - name: Check any "Docs-only: yes" commits in this push

A plain scalar may not contain `": "` — YAML reads it as a mapping. The whole of
`.github/workflows/ci.yaml` became unparseable. GitHub created **zero jobs**: the
run appeared under the file's path rather than under `CI`, concluded `failure`,
and had nothing in it to inspect. Fixed in `0b382a6` by quoting.

Nobody was careless. **RFC-090's own submission had already stated this gap in
writing** — *"no YAML parser was available in this environment to fully lint it
(no pyyaml, no js-yaml, no actionlint) — noted as a real, not hidden, gap."*
That was true, it was declared, and a week later it cost a red `main`.

## The actual missing capability

**Nobody on this project can check a workflow file before pushing it.** GitHub is
the first and only validator, and it does not report until after the push. That
is the thing to fix — the quoting bug is just what it produced this time.

## Unit 1 — `scripts/check-workflows.sh`

Validates every file under `.github/workflows/`. Runnable locally, which is the
whole point: the same shape as `check-version-snippets.sh` and the rest.

**Use `actionlint`, pinned to an exact version.** It catches the YAML parse error
above *and* GitHub-specific mistakes a plain YAML parser cannot — unknown
`runs-on` labels, bad `${{ }}` expressions, invalid `needs:` references. A bare
`pyyaml` parse would have caught this incident and little else; prefer the tool
that catches the next one too.

The script installs or locates `actionlint` and runs it across the directory.
If a contributor has no `actionlint`, it must **say so and exit non-zero**, not
silently pass — an unvalidatable tree is the state this exists to end, and a
check that quietly does nothing when its tool is missing is the defect this
project has now hit four times.

## Unit 2 — `.github/workflows/workflow-lint.yaml`

**It must be its own file. It cannot go in `ci.yaml`.**

That is the whole structural point: a broken workflow produces *no jobs*, so a
validator inside `ci.yaml` would have been just as absent as everything else
during this incident. The validator has to survive the failure it validates.

Small, and triggered on `push` and `pull_request` — no path filter, because a
workflow file can break from a change anywhere in the directory and the job is
cheap.

**State the residual limit in the file's own header comment**, do not hide it:
if `workflow-lint.yaml` *itself* becomes unparseable, nothing reports. There is
no escape from that regress — every validator lives in a file that can break.
What this buys is that the file which actually gets edited (`ci.yaml`: three
times this cycle) is covered by one that rarely does. Keep this file minimal so
that stays true.

## Required evidence

**Prove it fails.** Reintroduce the exact defect — an unquoted step name
containing `": "` — in a scratch branch or worktree, and show:

1. `scripts/check-workflows.sh` refuses locally, naming the file and line;
2. the `workflow-lint` job refuses in CI;
3. both pass once the value is quoted.

Case 2 needs a real run. If reaching it requires pushing a branch, **ask
first** — same call you made on RFC-090's scratch tag, which was the right one.

## Acceptance criteria

1. `scripts/check-workflows.sh` exists, documented in `scripts/README.md`, and
   fails loudly when `actionlint` is unavailable.
2. `.github/workflows/workflow-lint.yaml` exists as its **own** file and states
   the self-validation limit in its header.
3. All three evidence cases demonstrated.
4. Running the script on the current tree passes — six workflow files, clean.
5. CHANGELOG entry, or one line saying why not.
