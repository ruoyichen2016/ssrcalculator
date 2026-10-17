# Iterative improvement loop

A reusable prompt for driving one focused improvement at a time on this
repository.  It is written to be handed to a fresh agent session with no other
context: everything needed to verify work, avoid known traps and decide when to
stop is below.

**How to run it:** paste this file as the prompt (or say *"read
`docs/LOOP_PROMPT.md` and run one iteration"*).  One iteration = one item,
implemented, verified, documented and committed.  Repeat for the next item; each
iteration must leave `master` green and the working tree clean.

---

## 1. Repository and environment

- Repo: `springsharp/` (git, branch `master`).  Reference-only decompiled C#
  sources live one level up in `raw/SpringSharpMono/`; they are **not** part of
  the repository and must never be committed.
- Core crate: zero dependencies, edition 2021, `rust-version = "1.80"`, pinned
  dev toolchain in `rust-toolchain.toml`.
- GUI: separate workspace in `gui/` (depends on eframe).  Build with
  `scripts/build-gui.sh`, which pins `CARGO_TARGET_DIR=target-gui`.
- Local CI: `bash scripts/check.sh` (add `--gui` only where the GUI can build).

Environment traps, already hit and worth not rediscovering:

- **Run cargo as `RUSTUP_TOOLCHAIN=stable cargo ...`.**  Root
  `rust-toolchain.toml` pins a named channel; rustup tries to download it and, in
  a sandboxed environment, fails with `EACCES` on `~/.rustup/tmp`.  The override
  variable is the one setting that outranks the toolchain file.
- **The GUI cannot build without a crates.io index.**  `cargo update --offline`
  in `gui/` fails with `no matching package named eframe`.  Do not "fix" that by
  hand-editing `gui/Cargo.lock` beyond a version field of the local path
  dependency.
- `git config user.name/email` are repo-local; commit identity is already set.

## 2. Mission

Raise the engineering quality and the *validated* accuracy of this Rust port of
the SpringSharp engine — one verifiable increment per iteration — without ever
overstating what is known.

This project's distinguishing value is epistemic honesty: every historical case
carries explicit provenance, in-sample fits are labelled diagnostic, and gates
that cannot be met are allowed to fail loudly.  Any change that trades that away
is a regression regardless of how good the number looks.

## 3. Non-negotiable invariants

An iteration that violates any of these is a failed iteration, even if all tests
pass.

1. **Frozen accuracy baselines hold.**  `cargo test --all-targets` must pass,
   including the thresholds in `tests/accuracy_regression.rs`.  Never loosen a
   threshold to make a change pass.  If a change genuinely regresses a baseline,
   revert the change (or prove the regression is a correction and say so
   explicitly in the commit message and `docs/VALIDATION.md`).
2. **In-sample ≠ accuracy.**  In-sample fits stay labelled diagnostic.  Only
   nested-LOOCV / holdout numbers may be quoted as accuracy evidence.
3. **Provenance is never faked.**  Do not mark a case verified, invent trial or
   model-basin data, or edit `source_type` / `verifier` / `source_url` to make
   `data-target` pass.  `data-target` exiting 4 is the *correct* state while the
   database is public-reference only.
4. **The core workspace stays dependency-free.**  Anything needing a crate
   belongs in `gui/` or does not belong.
5. **No `unsafe`** (both workspaces forbid it).
6. **Every commit compiles on its own** — verify, do not assume (see §4 step 7).
7. **Never commit** `raw/`, `target/`, `target-gui/`, `gui/target/`, or generated
   scratch files.
8. **Docs match reality in the same commit.**  Test counts, command lists, phase
   entries and baseline numbers must be true after the commit, not "later".

## 4. One iteration

**Step 0 — establish a green baseline before touching anything.**
`RUSTUP_TOOLCHAIN=stable bash scripts/check.sh` must exit 0 and
`git status --short` must be clean.  If the baseline is red, fixing that *is* the
iteration; report it instead of layering new work on top.

**Step 1 — survey (keep this cheap).**  Read: last entry and `## Remaining` in
`docs/ROADMAP.md`, the head of `docs/CHANGELOG.md`, `git log --oneline -8`,
`git status --short`.

**Step 2 — choose exactly one item.**  Take the highest-priority unblocked item
from §6, or a defect you can demonstrate.  If only blocked items remain, stop and
report (§7).  Do not bundle unrelated work: if you spot a second defect, add it to
the backlog as its own item.

**Step 3 — reproduce before changing.**  For a defect, write the failing test (or
name the exact command and current wrong output) first.  For a feature, name the
observable behaviour and the command that will demonstrate it.

**Step 4 — implement minimally.**  Smallest change that satisfies the item.  No
drive-by refactors, no reformatting of untouched code.

**Step 5 — verify.**  All of these must pass:

```bash
export RUSTUP_TOOLCHAIN=stable CARGO_INCREMENTAL=0
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets          # 116 unit + 8 regression + 2 CLI-surface at v0.3.0
RUSTUP_TOOLCHAIN=stable bash scripts/check.sh    # must exit 0
```

Then smoke-run every command you touched *with exit codes*, e.g.
`cmd >/dev/null 2>&1; echo $?` (usage errors are 2, runtime errors 1, an unmet
quality gate is 4).  A test that only asserts "not Unknown command" is not
evidence that a command works.

**Step 6 — document in the same commit.**  Add a `## Phase N — <title>` entry to
`docs/ROADMAP.md` (next number after the current last phase, 27 at v0.3.0), a
bullet to `docs/CHANGELOG.md`, and correct any count or description the change
invalidated.

**Step 7 — commit in thematic units.**  Match the existing style
(`Phase N: <imperative summary>`, body explaining why and carrying the evidence).
One concern per commit.  Then prove each new commit compiles in isolation:

```bash
for sha in <new shas>; do
  git worktree add -q --detach /tmp/wt-$sha "$sha"
  (cd /tmp/wt-$sha && RUSTUP_TOOLCHAIN=stable cargo check --all-targets -q) && echo "$sha ok"
  git worktree remove --force /tmp/wt-$sha
done
git worktree prune
```

**Step 8 — report** in the §8 format.

## 5. Definition of done for one iteration

- exactly one backlog item advanced (or one defect fixed with a test);
- working tree clean, all commits compiling in isolation;
- `scripts/check.sh` exits 0;
- docs updated in the same commits;
- accuracy baselines unchanged, or the change is explicitly justified;
- an iteration report produced.

## 6. Backlog

Blocked items exist; do not attempt them without a human decision.  Work the
unblocked list top-down unless the user names an item.

### Unblocked

- **B1 — Web console: cap simultaneous connections.**  Request-size caps, header
  caps, timeouts and per-connection threads are already in place; there is still
  no bound on concurrent connections.  Add a bounded worker/semaphore scheme,
  keep the console responsive, and cover it with a test that spawns concurrent
  clients.
- **B2 — CLI: assert behaviour, not just routing.**  `tests/cli_surface.rs`
  proves every advertised command is *routed*.  Add end-to-end assertions for
  the commands with no behavioural test (`interact` `set`/`load` paths, `html`
  output shape, `solve-speed`/`solve-range` convergence, gate exit codes).
- **B3 — GUI: expose the blended model.**  `blend-table` weights exist in the
  core; the Calibration page only offers the regime model.  Bring the blend
  alongside it (GUI crate only).
- **B4 — GUI: optional raw `.ssr` editor.**  The console uses structured inputs
  only; add a raw editor page with the same validation the CLI applies.
- **B5 — Savitsky validation on published planing data.**  Extend
  `planing-validate` beyond the current approximate cases.  If the regression
  coefficients cannot be sourced offline, say so in the docs — do not invent
  them (this was already rejected once in Phase 26; do not re-litigate without
  new sources).
- **B6 — More public literature cases for carriers/cruisers.**  Any addition
  needs full provenance fields and must be checked with `data-consistency` and
  `data-outliers`; report the LOOCV effect, including when it gets worse.
- **B7 — Confirm the MSRV job.**  The CI job now sets `RUSTUP_TOOLCHAIN`; once a
  run exists, report whether `rust-version = "1.80"` actually builds, and add any
  needed `#[allow]`/code fix rather than silently raising the declared minimum.

### Needs a human decision — escalate, do not attempt

- **C1 — Official trial / model-basin data.**  Unavailable to this project;
  `data-target` must keep failing.  Replacing the public-reference database is a
  procurement/access question, not an engineering one.
- **D1 — Licence and publishability.**  The ported formula layer is derived from
  proprietary SpringSharp 3.0 (see `NOTICE`).  Publishing requires written
  permission or a reimplementation of `model.rs` / `resistance.rs` from public
  literature only.  Do not relabel licences or soften `NOTICE`; escalate with an
  assessment of scope instead.

## 7. Stop and escalate

Stop and report instead of improvising when:

- the only remaining backlog items are in the "needs a human decision" list;
- satisfying an item would require weakening an invariant in §3;
- an improvement cannot be demonstrated by a measurement or a test;
- the baseline is red and the cause is not obvious.

Two rules that are easy to break and always wrong:

- **Never make a gate pass by changing the gate.**  Thresholds, provenance
  labels and "verified" flags are evidence, not obstacles.
- **Never present an in-sample number as accuracy evidence**, in code, docs or
  a commit message.

## 8. Iteration report format

```text
Iteration: <date> — <item id and title>
Change:    <one or two lines: what changed and why>
Evidence:  <commands run + observed results, including exit codes>
Commits:   <sha> <subject>   (each compiles in isolation: yes)
Docs:      <what was updated>
Baselines: regime LOOCV <x>% / blended <y>% / curated <z>% — unchanged | changed: <why>
Next:      <next backlog item, or "blocked: <reason>">
```

Baselines at v0.3.0 for comparison: regime **10.36%** case-weighted (curated
7.97%), blended **9.81%** (curated 7.47%), original baseline 22.44%, over 146
public-reference cases; `data-target` composite 0.393 and failing, which is
correct.

## 9. Loop driver

Repeat the prompt for successive iterations.  Terminate the loop when:

- three consecutive iterations produce no commit, or
- the backlog holds only human-decision items, or
- the user says stop.

Each iteration must leave the repository in a shippable state: `master` green,
tree clean, docs true.  Never leave work half-applied for the next iteration to
trip over — if an item cannot be finished, discard it, record it in the backlog
with what was learned, and report it.  A parked working tree is the one failure
mode that makes a loop worse than doing nothing.
