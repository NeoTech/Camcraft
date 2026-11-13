## #688 - The cut names the documentation site as its step 5

**Decided:** AGENTS.md's release cut gets a step 5, "Check the documentation
site": the tag run's `site` and `pages` jobs regenerate and deploy the Zola
site (and `/app/`), and the cut is done once `pages` is green and the live
site shows the new version. A red `pages` job is re-run, never fixed by
moving the tag. This closes #688: the site (PR #777), the version and
download links (`site/templates/`), the Pages deploy (maintainer, 2026-11-12)
and the browser build (#1103) had all shipped; only "AGENTS.md's release
steps name the site step" was left.

**Why:** the site has no step of its own to run - the tag does it - so the
cut's only duty is to check it went out. Naming that check stops a cut from
being called done while the site still shows the previous release (v0.18.0's
`pages` job failed until the maintainer re-ran it).

**Considered and dropped:** a local `zola build` step on the release branch
(adds a tool the cut machine may not have, and the tag run already builds
from the same tree); folding the text into step 4 (step 4 is already the
tag's preconditions; the site check comes after the push).

**Cost to change:** one paragraph in AGENTS.md.
