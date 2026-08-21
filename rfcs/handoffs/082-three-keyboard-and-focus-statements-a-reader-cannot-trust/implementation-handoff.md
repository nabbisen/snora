# Developer Handoff — RFC-082 three statements a reader cannot trust

**Governing RFC.** [RFC-082](../../accepted/082-three-keyboard-and-focus-statements-a-reader-cannot-trust.md)
**Status.** Inherited from RFC-082 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.2 — documentation only. **No code.**
**Implementation units.** Three.

---

## 1. §3 (menus) — unheld, and narrowed

The hold is lifted. **This RFC's own account of the blocker was wrong** and the
RFC now records the correction: snora does not always draw menu items.

| path | who builds the dropdown | state today |
|---|---|---|
| `AppLayout::header_menu(Node)` / `context_menu(Node)` | the **application** — an already-built `Option<Node>` | in-menu traversal is **already possible**; nothing needed from snora |
| `snora_widgets::menu::render_menu(...)` | **snora** | no way for the application to express a highlighted item |

**What you write in `menus.md`:** the ruling — in-menu keyboard traversal is the
application's own — **and the distinction between the two paths**, so a reader on
the engine path learns they can do it today and a reader on the widgets path
learns they currently cannot.

**Do not write that it is deferred**, and **do not promise the widgets-path
fix.** A separate RFC owns that; it is a public API change with a real design
question, and naming it here as forthcoming would commit a design nobody has
chosen.

**Verify both halves before writing them.** `layout.rs:121` and `:124` for the
slot types, `snora-widgets/src/menu.rs:42` for `render_menu`. If either reads
differently to you, stop — this RFC has already been wrong about it once.

## 2. Unit 1 — the decision register is wrong about who is asking

`docs/src/contributing/design-decisions.md:32`:

> Focus trapping deferred | Deferred — trigger fired, Q-1 (RFC-060) now the
> blocker | **Concrete app: met (tekstide)**

**tekstide withdrew as a demand signal on 2026-08-18** — *"we would not switch
to trapping even if you shipped it"* — and our own 0.36.1 note recorded the
consequence: *"the demand column for modal focus trapping is empty."*

Correct the row so it reflects that **no consumer is a demand signal**, citing
tekstide's withdrawal.

**Two things it must not do.** It must **not** claim to resolve RFC-060 Q-1 —
the owner ruled that is answered by measurement (RFC-078), which is open and
unstarted. And it must **not** silently drop the row; the decision is still
"deferred", only the evidence changed.

**Then check the rest of the register the same way.** RFC-075 fixed one register
being wrong about itself and never looked at this one. **Read every row's
evidence column and say which ones you verified**, including the ones that were
already right.

## 3. Unit 2 — a mandatory bullet that cannot pass

`docs/src/design/high-contrast.md:59-61`:

> - The focus ring at `3.0` width must be visible when tabbing through controls.
>   (In iced 0.14 the focus ring is not rendered through `button::Style` — see
>   Semantic accessibility.)

**A mandatory checklist item whose own parenthetical says it cannot be
satisfied.** Every other statement of that limitation in our docs carries a
marker — `BLOCKED`, `INFO`, or a documented exception.

Either mark it the way the others are, or reword it so it can be checked as
written. **Match the existing convention rather than inventing a fourth
marker** — find how the others are marked first, and say which you followed.

## 4. Unit 1's second half — Q-2 ruled: date the evidence

The register's demand column depends on what consumers say, which **no check can
derive** — do not try to mechanise it.

**Give each row's evidence the date it was last confirmed.** Staleness then
shows without pretending it is derivable, which is the honest version of the
guarantee RFC-079's check gives for guides.

## 5. Explicit non-change scope

- **No promise of the widgets-path fix in `menus.md`** (§1) — a separate RFC owns it.
- **No resolution of RFC-060 Q-1**, and no touching RFC-078.
- **No new marker vocabulary** (§3).
- **No code.** `git diff -- crates/` must be empty.

## 6. Required evidence

- The corrected register row, quoted before and after
- **Every register row's evidence column, listed, with whether you verified it**
  — including rows that needed nothing
- The `high-contrast.md` bullet before and after, plus the convention you matched
  and where you found it
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py`
- `git diff --stat -- crates/` — **expected empty**

## 7. Acceptance criteria

1. The focus-trapping row says no consumer is a demand signal, cites tekstide's
   withdrawal, keeps the decision as deferred, and does not resolve Q-1.
2. Every register row carries the date its evidence was last confirmed.
3. Every row's evidence reported as verified or not — a null result stated, not
   omitted.
4. The `high-contrast.md` bullet is marked or reworded, matching an existing
   convention you name.
5. `menus.md` states the ruling and distinguishes the two paths; it promises no future API and does not call the matter deferred.
6. `CHANGELOG.md` `[Unreleased]`, crediting tekstide.

## 8. Required review-request format

`.git-exclude/review-request/082-three-keyboard-and-focus-statements-a-reader-cannot-trust/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: §2's second half** — the rest of the register. One row
was wrong because a consumer told us. The others have had nobody checking them
at all.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
