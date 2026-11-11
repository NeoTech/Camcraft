## #854 - docs-picture goldens move to the release snapshot lane

- **What was decided:**
  - `docs_images_match_the_rendered_steps` is split. `docs_example_steps_through_operations` (default suite, every OS) keeps the non-pixel checks: the example opens, each operation has a toolpath, each step draws an overlay. `docs_images_snapshot_match_the_rendered_steps` holds the PNG compare, `#[ignore]`d with its lane named.
  - The release plan gets a step `--test gpu_render --ignored snapshot`, because the existing snapshot step is `--lib` only and would never reach an integration binary. This is an `xtask/` change, so `PROCESS_VERSION` is bumped to 13.
- **Why:** AGENTS.md "Pixel snapshots are release-lane" (#255): pixel goldens drift across adapters (Linux software adapter vs the committed PNGs).
- **Considered and dropped:**
  - Cross-adapter tolerance: no principled small epsilon (6000+ pixels differ per image); it would also hide real regressions.
  - Rename only, no xtask change: the lane is `--lib`, so the test would silently never run.
  - Regenerating the PNGs: forbidden for PRs (#255).
