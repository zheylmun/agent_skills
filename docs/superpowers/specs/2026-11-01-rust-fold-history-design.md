# rust-fold-history: design

Source: the conversation of 2026-11-01. A third rust-craft workflow skill,
beside [rust-spec-review](../../../skills/rust-spec-review/SKILL.md) and
[rust-refinement-loop](../../../skills/rust-refinement-loop/SKILL.md). The
refinement loop already names "the history skill" as the user's tool for
folding its commits at a break point; this is that skill.

## 1. Intent

A refinement loop leaves a long trail of small commits: one per design
change, confirmed bug fix and cleanup, plus the spec and plan edits that keep
those changes from reading as deviations. The user folds that trail so the
branch reads as though it went directly to the refined design. A short,
direct history narrows what a reviewer, or the next round of the loop, has to
read, and the user has found that this tightens the results.

The second half is context. A long refinement session fills the
conversation, and the user wants to fold the history, bring the plan up to
date, and compact the conversation as one step, so a session can tighten its
scope and keep going. A skill cannot run `/compact`: the Claude Code docs say
built-in commands such as `/compact` are not available through the Skill
tool, and no hook can start a compaction (`PreCompact` can only block one).
So the skill does everything up to the compaction, writes a handoff file,
and ends by giving the user a ready-to-paste `/compact` line that points at
that file.

## 2. Goals, non-goals, anti-goals

Goals:

- The folded branch has one commit per logical step of the plan, in plan
  order. A refinement that added a type or changed a signature folds into the
  step that owns that code, so the step count does not change. A refinement
  that clearly added or removed a logical step adds or removes a commit.
- The plan, after the fold, reads as if the refined design had been planned
  from the start: the final types, signatures and tests appear in the tasks
  that introduce them, and each folded commit matches one task.
- The final tree is byte-identical to the tree before the fold, apart from
  the plan rewrite, which is committed before the fold (section 4, step 3)
  so that even it is inside the tree being compared.
- Each folded commit builds and passes its tests, so the history bisects.
- One message at the end, and a `/compact` line that lets the session pick
  up where it was with a small context.

Non-goals:

- Pushing. The skill never pushes and never force-pushes. The user
  force-pushes a lot, but each push runs CI, so the push waits until the
  user decides the branch is ready; the message gives the command.
- Writing a PR description, or opening a PR.
- Reviewing or improving the code. The fold moves hunks between commits; it
  never changes what the final tree contains.
- Editing the spec. The refinement loop keeps it current; the fold only
  moves the spec's edits into the commit that introduces the spec.

Anti-goals (a failure even if it "works"):

- Any difference between the folded tree and the original tree.
- A lost commit: the original history must be recoverable without the
  reflog.
- A rewrite of the base branch, of any other branch, or of commits already
  on the base.
- A fold the user did not ask for. The refinement loop offers; it never runs
  the fold on its own.

## 3. When it runs

- **On request**, at any time: "fold the history", "simplify the branch
  history", "clean up the commits before the PR".
- **Offered by the refinement loop** at a break point, after the user has
  answered the trade-offs, and only when a fold looks worthwhile. A fold is
  worth offering when, since the last fold (or the start of the branch):
  - the branch's fix-up commits number at least as many as the plan has
    tasks; or
  - the loop has run 10 or more cycles; or
  - the conversation is clearly long. This one is the model's judgement; the
    skill cannot measure context size, and the first two are the observable
    tests.

  The offer is one line in the break-point message, with the counts that
  triggered it, so the user can decline without breaking stride. When none
  holds, the loop says nothing about folding. A small branch in a short
  session gains little from a fold.

The refinement loop's rule that it never rewrites history stands: it offers,
and the fold runs only on the user's yes, as this skill, not as a loop stage.

## 4. The fold

The decided parts, in order:

1. **Preconditions.** The working tree is clean (`git status --porcelain`
   is empty), no rebase, merge or cherry-pick is in progress, HEAD is on a
   named branch that is not the base, and the branch has commits beyond the
   base. Any failure stops the skill before it changes anything. The base is
   found as the refinement loop finds it (a base the user named, then local
   `main`, then `origin/HEAD`, then ask). The fold's root is the merge base
   of the branch and the base: the newest base commit the branch contains,
   so a branch that merged the base in keeps those changes in the root, not
   in a task commit.
2. **Backup.** Before anything is rewritten, a local branch
   `backup/<branch>/<yyyymmdd-hhmm>` points at the original HEAD. It is
   never pushed and never deleted by the skill; the message names it and
   the user drops it when satisfied.
3. **Plan rewrite, as an ordinary commit.** The plan is rewritten to the
   refined design: each task names the final types, signatures and tests
   for the code it introduces; a task the refinements made unnecessary is
   removed; a logical step the refinements added becomes its own task, in
   the order the code needs. The plan keeps its own format and voice
   (superpowers plans keep their header, Global Constraints and checkbox
   steps). This is committed on the original branch, before the fold, so
   the tree comparison in step 6 includes it.
4. **The fold map.** Every commit in `root..HEAD` is assigned to the task
   whose code it changes, or to a docs commit (the spec's or the plan's). A
   commit that changes code belonging to two tasks is split by hunk between
   them. The map is written to the handoff directory (section 5) before any
   commit is built, so a failure can be diagnosed.
5. **Build the new history** on a temporary branch from the root: the docs
   commits first, as superpowers commits them (one for the spec, then one
   for the plan, each present only when the branch adds or changes that
   file), then one commit per task in plan order. Each task commit's message is the
   original task commit's message when one exists, else written from the
   task. The mechanics (autosquash, cherry-pick and amend, or building each
   commit from paths) are the builder's.
6. **Verify, then move the branch.** The temporary branch's final tree must
   equal the original HEAD's tree (`git rev-parse <ref>^{tree}`); a
   difference stops the skill with the branch untouched. Each task commit
   is checked to be green, which here means `cargo fmt --check`, a build of
   all targets, and the tests, for the workspace (my call). Clippy, the docs
   build and the feature powerset are not run per commit: the final tree is
   the one the refinement loop already gated, and per-commit lint deltas
   would make the fold red for reasons no reviewer reads. A commit that is
   not green gets two attempts, each moving a hunk between adjacent task
   commits: first later, into the task that needs it, then earlier, when
   the code it needs belongs to an earlier task (my call: later first, so a
   task commit never carries code its task does not introduce). A move that
   leaves the plan's text wrong about which task introduces something
   revises the plan as a new ordinary commit on the original branch, and the
   fold restarts from step 4, so the plan and the commits stay matched. If a
   commit is still red after two attempts, the skill does not move the
   branch, and asks the user, naming the commit and the failing test. Only
   after both checks does the branch ref move to the new history,
   with the old value checked (`git update-ref` with the expected old sha),
   and the temporary branch is deleted.

Without a plan (a branch that did not come through writing-plans), the skill
proposes the logical steps from the commit messages and the diff, shows the
proposed map, and asks before step 5. It writes no plan file into the
repository (my call): step 3 is skipped, the agreed steps live in the
handoff directory, and there is a docs commit only for a spec the branch
adds or changes. With a plan, it runs to the end without asking, except for
a red commit as in step 6.

## 5. The handoff and the compaction

The handoff lives in the repository's git directory, at
`$(git rev-parse --git-dir)/fold-history/`, so it is per-repository, never
committed, and survives compaction and a new session. It holds the fold map
(step 4) and `handoff.md`, which a fresh context can resume from:

- the branch, its base and root, the backup branch, and the folded commits,
  one line each, mapped to plan tasks;
- the spec and plan paths;
- when the refinement loop offered the fold: the batch and cycle count, the
  path to the loop's notes (which hold its green baseline, still valid,
  since the final tree is unchanged), the items still undecided, and what
  the next batch starts with. The user's answers at this break point are
  applied and committed by the loop before the fold is offered (my call),
  so they are folded in like any other change, and the handoff only lists
  them;
- that the branch differs from its remote, if it has one, and the
  force-push command.

The final message is:

1. One line: commits before and after, tasks in the plan, and that the tree
   matched and every commit is green.
2. The backup branch, and the force-push command for when the user is ready:
   `git push --force-with-lease`. When the branch has no upstream, say so
   instead.
3. The `/compact` line, ready to paste, naming the handoff file and what to
   do next, for example:
   `/compact The branch feat/x was folded to 6 commits matching the plan.
   Keep only what is needed to continue; read .git/fold-history/handoff.md
   and resume the refinement loop at batch 3.`

Nothing else.

**Optional hook.** A `SessionStart` hook with matcher `compact` runs after a
compaction and adds its stdout to the new context, so it could print the
handoff file automatically. The skill mentions this once, as a settings
change the user may make; it never edits settings itself.

## 6. Changes to rust-refinement-loop

- Break Points: when a fold is worth offering (section 3), part 1's line
  gains a clause offering it, with the counts. The existing paragraph about
  "the history skill" names this skill and keeps its rule that the loop
  never runs it unasked.
- The offer comes after the user's answers are applied and committed, so
  the fold includes them.
- After a fold, the loop already re-reads the branch and keeps the merge
  base; it also reads the handoff file when it resumes after a compaction,
  and keeps its recorded green baseline rather than recording a new one.
- README lists the new skill.

## 7. Testing

The repo's method: prompt-only scenarios under `tests/scenarios/` run in a
clean-room subagent, against fixtures under `tests/fixtures/` that are never
improved.

- A fixture script, `tests/fixtures/fold_branch/setup.sh <dir>`, POSIX sh and
  hardened as `refinement_branch/setup.sh` is: a small crate on `main`, then a
  branch with a spec and a three-task plan, three task commits, and about
  eight fix-up commits seeded with:
  1. a new type that belongs to task 2's code;
  2. a signature change in task 1's code;
  3. one commit that touches code from tasks 1 and 3;
  4. two spec decisions-section edits;
  5. a fix-up that adds a logical step the plan does not have;
  6. a fix-up that only makes sense after task 3 (so a naive fold into an
     earlier task leaves a red commit).
- Scenario `fold-history-baseline`: fold the fixture branch. Pass when the
  final tree matches, every task commit is green, the commit count equals
  the rewritten plan's tasks plus the two docs commits, seed 6's fix-up sits
  in task 3's commit or later, the plan names the seeded
  type in task 2, the backup branch exists, nothing was pushed, and the
  message has the three parts and a `/compact` line naming the handoff.
- Scenario `fold-history-dirty`: the same branch with an uncommitted change.
  Pass when nothing is rewritten and the message says why.
- Scenario `refinement-loop-fold-offer`: a break point where the counts pass
  the threshold, and one where they do not. Pass when the offer appears in
  the first and is absent from the second.

## 8. Constraints

- Frontmatter `name` and `description` only, description trigger-only and
  starting "Use when", under 1024 characters together.
- Cites rust-refinement-loop for the base rule and the green command set
  rather than restating them.
- Never pushes, never edits settings, never deletes the backup branch.

## 9. Left to the builder

- The git mechanics of steps 4 and 5, and how hunks are split.
- The wording of commit messages written from a task.
- The fixture crate's content, within the seeds above.
