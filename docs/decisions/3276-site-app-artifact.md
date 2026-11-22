## #3276 - A `site_only` dispatch takes the browser build of an earlier run

**Decided:** `release.yml` gets a second optional input on its existing
`workflow_dispatch`: `app_run_id` (string, default empty). When
`site_only` is true and `app_run_id` is not empty, the `site` job:

- checks that the id has only digits, and stops the job if not;
- downloads the `wasm-app` artifact of that run
  (`actions/download-artifact@v4` with `run-id` and `github-token:
  ${{ github.token }}`), with no `continue-on-error`;
- runs `web/check-app-page.sh` on the download and again on its copy at
  app/, as before. On this path a failed download or check is an error,
  so the site job fails and `pages` deploys nothing.

The `site` job gets the permission `actions: read`, and nothing more, so
that the job token can read the artifacts of the other run. The input
goes to scripts only through `env:`, never through `${{ }}` in a `run:`
script. The tag path, a plain dispatch and a `site_only` dispatch
without `app_run_id` keep their behaviour: the same download step from
this run, the same warnings, the same "not part of this copy" page.

The 0.25.0 dispatch is `gh workflow run release.yml --ref site/v0.25.0
-f site_only=true -f app_run_id=44976953856`. The run 44976953856 is the
v0.25.0 tag run. As before, the same `release.yml` goes on
`site/v0.25.0` as one commit, byte-identical to the one on `main`,
because a dispatch uses the workflow file of the dispatched ref.

**Why:** a `site_only` run skips the `wasm` job, so run 45004413935
deployed app/ as the "not part of this copy" page. The `wasm-app`
artifact of the tag run is the exact 0.25.0 browser build: it was built
from the tagged tree and passed `web/check-app-page.sh` in that run. The
site job is strict on this path because the dispatcher asked for that
app by name: a site without it is not what was asked, and a failed job
leaves the deployed site as it is.

**Dropped:**
- A `wasm` build in the `site_only` run, from the site branch: it is a
  new build of a tree that is not the tag (the site branch has site and
  docs commits on top), it costs up to 45 minutes of runner time, and it
  can differ from the shipped app.
- A re-run of the tag run 44976953856: it runs the full platform build
  again and its `publish` job again on the existing release.
- The same warnings as the own-run path on a failed download: a deploy
  without the asked-for app would hide the failure in a green run.

**Cost to change later:** remove the input, the two steps that the
`app_run_id` condition guards, the `APP_RUN_ID` env lines and their two
`if [ -n "$APP_RUN_ID" ]` branches, and `actions: read`. The artifact of a
run expires after the retention period (90 days by default); after that,
the input cannot bring that app back.
