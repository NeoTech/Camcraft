## #4133 - The site tests run in the gate, before the tag

**What was decided.**

- `cargo xtask gate` has a step kind that does not start cargo: the site tests step
  (`xtask/src/site_step.rs`). It runs `python3 -m unittest discover -s site/tests` (else `python`)
  at the root, with `ZOLA` (else `zola` on PATH), `PYTHONUTF8=1` and `node` on PATH. All calls go
  through `std::process::Command`, so the code builds on Windows, Linux and macOS.
- The batch tier and the release tier always have the step. The quick tier has it when the diff
  touches a path of `site_step::INPUTS`. The post-merge gate runs batch on each push to `main`, so a
  red site shows on `main`, not after the tag.
- A missing tool (python 3, zola, node):
  - quick and batch: a `SKIP` row, note `SKIPPED (tool missing: <tool>) - not a pass; install: ...`.
    The run does not fail for it, but the row is never `ok`.
  - release: a `FAIL` row that names the tool and its install step. The cut gate is the last check
    before the tag, and the tag must not find a red site again.
- A zola that is not the pinned 0.23.6 of `release.yml` adds a note to the row. It does not fail.
- `site_step::INPUTS` is the one list of inputs. The pin
  `site_step_tests::each_path_that_a_site_test_names_is_an_input` reads every path literal in
  `site/tests/*.py` (and the `UI + "..."` joins) and fails when a path is not covered. So a new input
  of a site test cannot be forgotten.

**Why.** The site tests ran only in the `site` job of `release.yml`, after the tag. They were red at
three cuts in a row (v0.27.0: #4106, #4109). On `origin/main` 39298012 they were red again: 112 tests,
23 failures (and 6 errors on a host without node). The cause was that no gate ran them.

**What the runners need.** No runner and no workflow changes in this PR. For the step to run (and not
SKIP or FAIL), each runner user needs on its PATH:

- the cut runner (`camcraft-cut`): python 3, zola 0.23.6 (or `ZOLA=<path>` in the runner's `.env`),
  node. Without them, the cut gate is red at the site step, by this decision.
- the post-merge runner (`camcraft-post-merge`): the same tools. Without them, the batch summary shows
  a `SKIP` row for the site step.

**Considered and dropped.**

- A missing tool is a `FAIL` in each tier: dropped. It makes the quick gate red on each contributor
  host without zola and node, for a change that only touched an input.
- A missing tool is a plain `ok` with a note: dropped. That is a silent pass.
- A change in `.github/` (install zola in `cut.yml` or `post-merge.yml`): dropped. A `.github` change
  waits for the maintainer, and the runners are self-hosted, so the tools belong to the runner image.
- Inputs as broad prefixes (`crates/app/src/`, `crates/ui/src/`): dropped for exact files where the
  tests open exact files, so quick does not run the site step for each app change.

**Gap left open.** `.github/scripts/post-merge-skip.sh` runs the post-merge gate for `site/content/*`
and `site/tests/*`, but treats a push that changes only other `site/*` paths (`site/static/`,
`site/tools/`, `site/templates/`) as docs and skips the post-merge gate. Such a push gets
the site step in its quick gate and at the cut (release tier), but not in post-merge. A fix is a
`.github` change, for the maintainer: #4433.

**Cost to change later.** Small. The tier rule is one `Missing` value for each tier in `plan.rs` and
`quick.rs`, and the inputs are one list.
