## #688 - Documentation site on GitHub Pages, built with Zola (first cut)

**Decided:** a Zola project at `site/` with original Tera templates and one
hand-written stylesheet (no third-party theme, no CSS framework, no
JavaScript), a user guide (install, first run, setups, operations,
posting, reading a refusal) and a short developer section (crate map,
commands and undo, test gates, releases). It is built and deployed by two
new jobs in the existing tag-triggered `release.yml`: `site` (install a
pinned Zola, build) and `pages` (deploy with `actions/deploy-pages`). No
trigger was added. The `/app/` browser build waits for #31; the front page
says it is coming.

**Why, choice by choice:**
- **No theme.** A theme would add a licence to track in
  `assets/ui/LICENSES.md` and styling we would have to read for
  clean-room fit; four small templates and ~60 lines of CSS do the job and
  stay readable. Light and dark follow `prefers-color-scheme`.
- **Zola pinned to v0.23.6**, the musl Linux binary from the official
  release, checked against its SHA-256 (the release asset's published
  digest) - the same pattern the Linux job uses for linuxdeploy. musl,
  because it is static and does not depend on the runner's glibc.
- **Version from the workspace manifest, not from Zola.** Zola's
  `load_data` refuses paths outside the site directory, so the templates
  cannot read `../Cargo.toml`. The `site` job reads
  `[workspace.package] version`, refuses a tag that does not name it
  (as `publish` does), and writes `site/version.toml`, which the templates
  load with `required=false`. A local build without the file says it is a
  local preview instead of showing a version it was not built from. A
  committed symlink into the site was dropped: Windows checkouts without
  symlink support would get a text file and a broken build.
- **Base URL from the repository**, `https://<owner>.github.io/<repo>`,
  passed as `zola build --base-url`, so the same job deploys correctly on
  NeoTech/Camcraft and on a fork's own Pages. `actions/configure-pages`
  was dropped: it needs Pages API access in the build job and fails when
  Pages is not yet enabled.
- **Deploy after `publish`.** `pages` needs `site` and `publish` and runs
  only on a tag when both succeeded, so the site's "Download X.Y.Z" link
  never points at a release page that is not up. A run whose publish
  fails leaves the previous site live. The `site` job also runs on a
  manual `workflow_dispatch` as a build check; nothing is deployed then.
- **Permissions scoped to the deploy job.** The workflow default stays
  `contents: read`; only `pages` gets `pages: write` and `id-token: write`,
  with a `pages` concurrency group so two tag runs never deploy at once.
- **Nothing generated is committed:** `site/public/` and
  `site/version.toml` are gitignored.

**Content sources:** README, AGENTS.md, RELEASE_NOTES.md,
`docs/MACHINE-ONBOARDING.md`, and the UI and app code for button labels,
tab names, operation captions (`operation::KindInfo::ALL`) and refusal
messages (`posting.rs`, `ui/egui_app/post_refusal.rs`). Only CamCraft's own
behaviour is described; no vendor text was read or copied.

**Not done here (follow-ups on #688):**
- AGENTS.md's release steps do not name the site step yet. That is a
  process-file change (a `PROCESS_VERSION` bump and a #681 note), so it is
  left for the maintainer or a process PR once this job has run once.
- Repository settings the deploy needs, which a workflow cannot set:
  Pages source "GitHub Actions", and the `github-pages` environment's
  deployment rules must allow tags matching `v*.*.0` (the default allows
  only protected branches, which would refuse a tag deploy).
- The Actions minutes of the first tag run are measured and reported on
  #688 after that run.

**Cost to change later:** low. Content is plain Markdown; switching the
generator or adding the `/app/` folder (#31) touches only `site/` and the
`site` job.
