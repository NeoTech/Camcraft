## #1415 - Remove the unused ci.yml and nightly.yml workflows

**What was decided.** `.github/workflows/ci.yml` and `nightly.yml` are
deleted, and every live text that named them (AGENTS.md "GitHub Actions
minutes are budgeted", README.md, scripts/README.md, the site's gates page)
now says GitHub runs only `release.yml` on a pushed tag (with
`aur-package.yml` as a job it calls). `PROCESS_VERSION` goes 54 -> 55.
History files (CHANGELOG.md, RECOMMENDED.md, release notes) are left as
they were.

**Why.** The maintainer asked (2026-11-13: "so we can remove ci.yml and
nightly.yml as a docs change."). Both had only a `workflow_dispatch`
trigger; `ci.yml` last ran on 2026-11-06 and `nightly.yml` has no run in
the last 100. Every gate tier runs locally with `cargo xtask gate`, so the
files were dead weight that the docs had to keep explaining.

**Considered and dropped.** Keeping them manual "for the rare case a
clean-machine run is wanted": nobody used that path, and a clean-machine
gate run on GitHub is the subject of issue 1413 (cut pipeline on a
self-hosted Linux runner), which would replace them anyway.

**Cost to change later.** Restore from git history
(`git log -- .github/workflows/ci.yml`); no code depends on the files.
