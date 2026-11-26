## #3459 - Check the merged wave before the first merge

**Decided:** Add `cargo xtask merge-plan --check`. Keep the default text-only
planner for fast inspection. The merge-ready skill requires `--check` before
the first merge. The check uses the recorded base and PR heads.

The tool checks the simulated commit in a unique detached worktree under
`.claude/worktrees/`. It does not merge into a real branch. It removes its
worktree after success, a compiler error, or an operational error. It reuses
the caller's `target/` and runs `cargo check --tests`, not `cargo test`.

Cargo metadata supplies the workspace package names. Each changed package
is checked separately. Shared build inputs select all workspace packages.
Documentation outside packages selects none. The check does not unify the
features of unrelated packages in one Cargo call.

Compiler diagnostic paths identify the PRs to exclude. If ownership is not
clear, the tool excludes the whole candidate wave. Stacked children of an
excluded PR leave the plan too. The tool plans and checks the remaining PRs
again. Text and JSON reports list exclusions and the recorded heads. An
exclusion returns exit 1. An operational error returns exit 2 without a plan.

**Why:** A text-clean merge can add a struct field and retain an initializer
without that field. Two individually compiling heads can fail together.
The regression uses real Git branches and real Cargo checks of a tiny crate.
It also checks worktree cleanup and preservation of caller changes.

**Considered and dropped:** A full test build would link test binaries and
use more disk. A heavy gate would exceed the requested compile check. A
persistent trial branch would add shared state. Guessing one owner for an
unattributed diagnostic could admit a broken wave. None is necessary here.

**Cost to change later:** The validation module and the merge-ready skill
can add another check policy. This command does not prove runtime behavior,
review independence, or machine safety. Wave 2 still requires its normal
refresh and review. A moved base or head requires another trial check.
