---
name: rust-fold-history
description: Use when asked to fold, squash, simplify or clean up a Rust branch's commit history before its pull request, or when the user accepts a fold offered at a rust-refinement-loop break point
---

## Overview

A refinement loop leaves a long trail of small commits: one per design
change, confirmed fix and cleanup, plus the spec and plan edits that keep
them from reading as deviations. This skill folds that trail so the branch
reads as though it went directly to the refined design: the plan is
rewritten to that design, and the branch becomes one commit per plan task,
in plan order, after one commit for the spec and one for the plan. A short,
direct history narrows what a reviewer, or the loop's next round, has to
read.

The fold moves hunks between commits and nothing else. The folded branch's
final tree is byte-identical to the tree before the fold, every task commit
is green, and the original history stays on a backup branch.

It ends with a handoff file and a ready-to-paste `/compact` line, so the
session can drop its long context and carry on from the file. **The skill
cannot run `/compact` itself:** built-in commands are not available through
the Skill tool, and no hook can start a compaction (`PreCompact` can only
block one). So it does everything up to the compaction and hands the user
the line.

**The skill never pushes and never force-pushes**, even when asked to "push
it when you're done" and even when the branch has an upstream: each push
runs CI, so the push waits until the user decides the branch is ready. The
final message gives the command. The skill never edits settings, never
deletes the backup branch, and never changes code, the spec, the base
branch or any other branch.

## When It Runs

- **On request**, at any time: "fold the history", "simplify the branch
  history", "clean up the commits before the PR".
- **Offered by the refinement loop** at a break point, when the user says
  yes. [rust-refinement-loop](../rust-refinement-loop/SKILL.md) decides
  when a fold is worth offering and makes the offer; it never runs the fold
  on its own. When it does, the user's answers at that break point are
  already applied and committed, so they fold in like any other change.

A fold the user did not ask for is a failure even if it works.

## The Fold

In this order. Until the last move of step 6, the only changes to the
user's branch are step 3's plan commit and any plan revision from step 6.

1. **Preconditions.** All of these hold, or stop before changing anything
   and say which failed:
   - the working tree is clean: `git status --porcelain` prints nothing,
     untracked files included. Do not stash, commit or discard the user's
     change to get there; the user does that;
   - no rebase, merge, cherry-pick or revert is in progress;
   - HEAD is on a named branch, and that branch is not the base;
   - the branch has commits beyond the base;
   - no `fold/<branch>` branch or worktree is left from an earlier stopped
     fold (`git worktree list`). If one is, the stop message gives the
     cleanup: `git worktree remove --force <dir>`, then
     `git branch -D fold/<branch>`.

   Find the base as [rust-refinement-loop](../rust-refinement-loop/SKILL.md)
   finds it (its What It Reads section). The **root** of the fold is
   `git merge-base HEAD <base>`: the newest base commit the branch
   contains. A branch that merged the base in keeps what that merge
   brought in the root; the merge commit maps to no task, and only a change
   of its own (a conflict resolution) is mapped, to the task whose code it
   touches. The folded branch is linear.

   Find the spec and plan as the refinement loop does. With no plan, see
   Without A Plan below.

2. **Backup.** Before anything is written, create a local branch at the
   original HEAD named `backup/<branch>/<yyyymmdd-hhmm>`, from the local
   time now (`git branch backup/feat/x/20261101-1430`). It is never pushed
   and never deleted by the skill; the message names it, and the user drops
   it when satisfied. It is how the original history is recovered without
   the reflog.

3. **Rewrite the plan, as an ordinary commit.** Rewrite the plan file to
   the refined design, reading the final tree and `git log -p root..HEAD`:
   - each task names the final types, signatures and tests for the code it
     introduces: a type a refinement added appears in the task that owns
     the code using it, and a signature appears as it now is;
   - code that needs something only a later task introduces goes in that
     later task (an `impl` that uses another module's constant belongs to
     the task that adds the constant);
   - a task the refinements made unnecessary is removed, and a logical step
     the refinements added becomes its own task, placed where the code
     needs it;
   - the plan keeps its own format and voice: a superpowers plan keeps its
     header, Global Constraints, Files and Interfaces blocks and checkbox
     steps.

   Change only the plan file. Commit it on the branch, as an ordinary
   commit, before the fold, so the tree comparison in step 6 covers it. The
   HEAD after this commit is the one the fold must reproduce. If the plan
   already matches the refined design, make no plan commit; the original
   HEAD is then the one to reproduce.

4. **Write the fold map.** Assign every commit in `root..HEAD`, the plan
   commit included, to one target commit: the spec's docs commit, the
   plan's docs commit, or the task that, in the rewritten plan, introduces
   the code it changes. A commit whose hunks belong to two targets (a task,
   the spec, or the plan) is split by hunk (or by file) between them.
   Every commit lands in a target; there is no "leftover" commit, and no
   target that is not a docs commit or a plan task. Write the map to the
   handoff directory, `$(git rev-parse --git-dir)/fold-history/fold-map.md`,
   run in the user's checkout, before building anything: one line per
   original commit, short sha and subject, then its target, and for a
   split, which hunks or paths go where.

5. **Build the new history** on a temporary branch from the root, in a
   temporary worktree outside the repository
   (`git worktree add -b fold/<branch> <tmpdir> <root>`), so the user's
   checkout is never moved:
   - the docs commits first, as superpowers commits them: one for the spec,
     then one for the plan, each present only when the branch adds or
     changes that file, each holding the file as it is at HEAD;
   - then one commit per task, in plan order.

   Keep each original task commit's message as it is, trailers included;
   the docs commits keep the original spec and plan commits' messages. A
   task with no original commit (one the refinements added) gets a message
   written from the task, in the style of the other task commits, without
   the loop's stage-and-cycle line. Fix-up messages are not carried. The
   mechanics are free (`git cherry-pick -n`, `git diff <a>^ <a> -- <paths>
   | git apply --index`, autosquash), as long as each target holds exactly
   the hunks the map gives it.

6. **Verify, then move the branch.** Two checks, in this order:
   - **Tree.** `git rev-parse fold/<branch>^{tree}` must equal
     `git rev-parse <branch>^{tree}` (HEAD after step 3's plan commit). Any
     difference, a single byte or a whitespace change, stops the skill:
     the branch is not moved. Report the differing paths
     (`git diff --stat <branch> fold/<branch>`) and leave the temporary
     branch and its worktree for diagnosis.
   - **Green per task commit.** At each task commit, in the worktree, run
     `cargo fmt --check`, `cargo build --all-targets` and `cargo test`, for
     the workspace (`--all` for fmt, `--workspace` for the others, and
     `--target` as [rust-refinement-loop](../rust-refinement-loop/SKILL.md)'s
     Green And Commits says). Only these three: clippy, the docs build and
     the feature powerset are not run per commit, since the final tree is
     the one the loop already gated, and per-commit lint deltas would make
     the fold red for reasons no reviewer reads. Every task commit must
     pass, not only the last: an intermediate red commit breaks bisect.

   A red commit gets two attempts, each moving a hunk between task
   commits: first **later**, moving the red commit's hunk into the task
   that introduces what the hunk needs; then **earlier**, moving into the
   red commit a hunk from a later commit that it needs. Later comes first
   so a task commit never carries code its task does not introduce. After
   any move, run the green checks again on every task commit from the
   earliest one the move changed, and the tree check again. When a move
   leaves the plan wrong about which task introduces something, revise the
   plan as a new ordinary commit on the branch, delete the temporary branch
   and worktree, and restart from step 4: a new map, a new build, both
   checks again, against the new HEAD. The plan and the commits stay
   matched.

   If a commit is still red after two attempts, do not move the branch.
   Ask the user, naming the folded commit, the failing command and the
   failing test or error, and leave the temporary branch, its worktree and
   the map for them.

   When both checks pass, and only then, move the branch with the old
   value checked:

   ```
   git update-ref -m "rust-fold-history" refs/heads/<branch> <new-sha> <expected-old-sha>
   ```

   where `<expected-old-sha>` is HEAD after the last plan commit, or the
   original HEAD when there is none (no plan change, or no plan). It fails
   if the branch moved meanwhile; then stop and say so. Never `git reset
   --hard`, `git branch -f` or a rebase of the branch itself. Since the
   trees are equal, the user's checkout stays clean. Then remove the
   worktree and delete the temporary branch. The backup branch stays.

**Without a plan** (a branch that did not come through writing-plans):
propose the logical steps from the commit messages and the diff, show the
proposed fold map, and ask before step 5. Write no plan file into the
repository: step 3 is skipped, and the agreed steps live in the handoff
directory, beside the map. A restart from step 6 revises those agreed
steps there, not a plan file, and the expected old sha stays the original
HEAD. The only docs commit is the spec's, when the branch adds or changes
one. With a plan, the skill runs to the end without asking, except for a
commit still red after two attempts.

## The Handoff And The Message

**The handoff** lives in `$(git rev-parse --git-dir)/fold-history/`, run in
the user's checkout: per repository, never committed, and it survives
compaction and a new session. Beside `fold-map.md`, write `handoff.md`,
which a fresh context resumes from, only after the branch moves (the end
of step 6):

- the branch, its base, the root's sha, and the backup branch;
- the folded commits, one line each, `<sha> <subject>`, mapped to the plan
  task each one is (or to "spec" or "plan");
- the spec and plan paths;
- when the refinement loop offered the fold: the batch and cycle count, the
  path to the loop's notes (whose green baseline still holds, since the
  final tree is unchanged), the items still undecided, the answers given
  at this break point (applied and committed by the loop before the fold,
  so only listed here), and what the next batch starts with;
- when the branch has an upstream, that it now differs from it, and the
  force-push command.

**The message**, when the fold succeeds, has three parts and nothing else:

1. **One line:** commits before and after, tasks in the plan, and that the
   tree matched and every commit is green.
2. **The backup branch**, and the force-push command for when the user is
   ready: `git push --force-with-lease`. When the branch has no upstream,
   say so instead. Do not run it.
3. **The `/compact` line**, ready to paste, naming the handoff file and
   what to do next, for example:

   ```
   /compact The branch feat/x was folded to 6 commits matching the plan. Keep only what is needed to continue; read .git/fold-history/handoff.md and resume the refinement loop at batch 3.
   ```

   The handoff path is the one `git rev-parse --git-dir` printed, with
   `fold-history/handoff.md` after it: `.git/...` in a plain checkout, an
   absolute path in a linked worktree. On a fold the user asked for outside
   the loop, the line says what the user was doing next, or "and continue".

No per-commit narrative, no map in the message (it is in the file), no
restatement of the plan.

When the skill stops instead (a precondition, a tree difference, a commit
still red, a moved branch), the message says why in its first line, names
what it did write (the backup; the plan commit, by its sha, on top of the
original HEAD; the temporary branch and its worktree's path; the map),
gives the cleanup for the temporary branch (`git worktree remove --force
<dir>`, then `git branch -D fold/<branch>`), and says the branch was not
folded. A precondition stop writes nothing: no backup, no commit, no
handoff.

**Optional hook.** A `SessionStart` hook with matcher `compact` runs after a
compaction and adds its stdout to the new context, so it could print
`handoff.md` automatically. That is a change to the user's own settings,
theirs to make; tell them once if they ask how to skip pasting the path.
The skill never edits settings.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "The trees differ only in whitespace, so it's fine." | The final tree is byte-identical or the fold stops. Compare `^{tree}` ids; any difference, whitespace included, stops the skill with the branch untouched and the differing paths reported. |
| "This fix-up fits no task cleanly, so I'll keep it as its own commit." | Every commit maps to the spec, the plan, or a plan task. A fix-up that needs a later task's code goes into that later task; one that adds a logical step becomes a new task in the rewritten plan. A commit that matches no task is a fold that did not finish. |
| "The plan is close enough; the commits are what matter." | The plan is rewritten and committed first: final types, signatures and tests in the task that introduces them, added steps as tasks. A folded branch whose plan still names the old signatures reads as a deviation to the next reviewer. |
| "The branch has an upstream, so I'll push to keep it in sync." | The skill never pushes or force-pushes, even when asked to push when done. Each push runs CI; the message gives `git push --force-with-lease` for the user to run when ready. |
| "The red commit is intermediate, the final tree is green, so I'll move the branch." | Every task commit must be green, or the history does not bisect. Move a hunk later, then earlier; still red after two attempts means the branch stays where it is and the user is asked, with the commit and the failing test. |
| "Tests pass at every commit, that's green." | Green per commit is `cargo fmt --check`, a build of all targets, and the tests. A commit whose benches or examples do not build, or that is not formatted, is red. |
| "I'll fix the code while folding since I'm touching it anyway." | The fold moves hunks; it never changes what the final tree holds. A fix is a change for the refinement loop or the user, after the fold. |
| "I'll `git reset --hard` the branch onto the folded history." | The branch moves only by `git update-ref` with the expected old sha, after both checks. A reset skips the check that nothing moved the branch meanwhile. |
| "I'll delete the backup branch since the fold verified." | The backup is the user's to drop. It is how the original history comes back without the reflog. |
| "A plain backup name is enough." | `backup/<branch>/<yyyymmdd-hhmm>`: a second fold of the same branch must not collide with, or overwrite, the first one's backup. |
| "I'll stash the uncommitted change and fold." | A dirty tree stops the skill before it writes anything. The change is the user's; say which paths are dirty and let them commit or stash. |
| "I'll edit the spec to match the folded commits." | The fold never edits the spec. The refinement loop keeps it current; the fold only moves its edits into the spec's commit. |
| "I'll write clean new messages for every commit." | Task commits keep the original task commit's message; only a task with no original commit gets a message written from the task. |
| "I can run /compact at the end." | Built-in commands are not available through the Skill tool, and no hook starts a compaction. Write the handoff and give the user the `/compact` line to paste. |
