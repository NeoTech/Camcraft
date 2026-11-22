## #3276 - Redeploy the 0.25.0 site with a `site_only` dispatch of release.yml

**Decided:** `release.yml` gets a boolean input `site_only` (default false)
on its existing `workflow_dispatch`. With `site_only: true`, only the `site`
and `pages` jobs run, from the dispatched ref. The `linux`, `macos`,
`windows` and `wasm` jobs are skipped by their own condition; `aur`, `snap`
and `flatpak` follow `linux`. `publish` stays tag-only, and its condition also has `!inputs.site_only`,
so a `site_only` run never publishes, whatever its ref.
`pages` deploys on that path only when the ref is a `site/*` branch. The
site version comes from the `[workspace.package]` version of the
dispatched tree's `Cargo.toml`, as on a tag run. With `wasm` skipped, the
site goes out with app/ as its "not part of this copy" page.

The 0.25.0 site is redeployed from the branch `site/v0.25.0`: the tag
v0.25.0 plus only the site and docs part of PR #3274 that applies to that
tree (the V-Carve section, its icon, its diagram and its reference entry
are left out, because V-Carve (#3151) merged after the tag). The dispatch
is `gh workflow run release.yml --ref site/v0.25.0 -f site_only=true`.

GitHub runs a dispatch with the workflow file of the dispatched ref, not
the one on `main`. Thus the same `release.yml` change goes on
`site/v0.25.0` before the dispatch, as one commit with the file
byte-identical to the one on `main`. Without it, the tag's `release.yml`
has no `site_only` input, and the dispatch builds every platform.

**Why:** the `site` job of the v0.25.0 tag run failed on site tests that
had drifted from the shipped app (#3271), so `pages` did not deploy. A
re-run of the tag run builds the same tree and fails again. The input adds
no trigger (AGENTS.md "GitHub Actions minutes are budgeted"): only a
collaborator can dispatch, a fork PR cannot start it, and a site-only run
bills only two short Linux jobs. The `site/*` guard on `pages` keeps a
dispatch from `main` from deploying a site that names a version that is
not released.

**Owner action (a settings change, not done by an agent):** the
`github-pages` environment must permit deployments from branches named
`site/*` (Settings > Environments > github-pages > Deployment branches and
tags). Without that policy the `pages` job of the dispatch is refused, and
the 0.25.0 site content ships with the v0.26.0 deploy.

**Dropped:**
- A site build from `main`: it documents features after 0.25.0 (V-Carve)
  and names a version that is not released.
- A moved or re-pushed tag v0.25.0: AGENTS.md forbids moving an existing
  tag, and a tag push also starts the full platform build and publish.
- A patch cut v0.25.1: there are no patch cuts, and `release.yml` builds
  nothing for a patch tag.
- A separate site workflow: a second copy of the site and pages jobs that
  can drift from the release copy.

**Cost to change later:** remove the input, the four `if:` lines and the
`!inputs.site_only` term of `publish` and `pages`; the
tag path does not depend on them.
