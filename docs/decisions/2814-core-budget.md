## #2814 - Use available CPUs for the default gate budget

**What was decided**

- Use `std::thread::available_parallelism()` for the default job budget.
  Clamp the default to 2 through 64 jobs. Use 2 if CPU detection fails.
- Keep `CAMCRAFT_GATE_JOBS` as a positive whole-number override.
  Apply no CPU floor or cap to an explicit override.
- Use the CPU default for an empty value. For a bad value, print a
  diagnostic that names the value and the chosen fallback.
- Print the source beside the chosen budget, including in `--dry-run`.
- Keep the existing batch and release queue split. It already allows
  `max(1, budget / 2)` lanes. Builds, isolated tests, UI steps and clippy
  continue to run alone with the full budget.
- Keep `plan::J` at `"6"`. It is a marker that `budget::scale` replaces
  when a step runs. It no longer describes the default budget.
- Leave `.claude/PROCESS_VERSION` unchanged. The merging session bumps
  it after the process changes merge.

**Why**

- A fixed budget of six leaves CPUs idle on larger machines.
- The standard library reports the CPUs available to the process and
  needs no new dependency.
- The floor keeps two jobs available. The cap limits automatic concurrency.
  The override lets a runner owner choose a different limit.
- The existing queue already scales with the budget. Changing the queue
  would add work without changing the required behaviour.
- Keeping the marker avoids unrelated changes to tests of raw plans.
  The lane fixtures use an explicit budget of six to remain deterministic.

**Considered and dropped**

- A free-memory guard. Portable free-memory detection adds scope and may
  need a dependency. `CAMCRAFT_GATE_JOBS` can limit a machine with little
  memory. This optional guard can be a separate change.
- A cap on explicit overrides. Existing runner settings must continue to
  choose their own budget, including one job or more than 64 jobs.
- Changes to the lane scheduler. Its limit already follows the budget.

**Cost to change later**

- Low. CPU selection and override parsing are in `budget::from_cpu_value`.
  Unit tests supply CPU counts to check the fallback, floor and cap.
  A memory guard can extend that selection without changing the queue.
