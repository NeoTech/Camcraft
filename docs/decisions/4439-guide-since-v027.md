## #4439 - User guide refresh for the changes since v0.27.0

**What was decided:** Describe each user-visible change that merged after
`v0.27.0` in the user guide. The guide is the documentation site,
`site/content/guide/`, and the topic pages in `docs/user-guide/`. Most
text goes to the site pages, because the previous refreshes (#3909) wrote
there and the release publishes them. The SolidCraft face-name change goes
to `docs/user-guide/solidcraft-camcraft-mcp.md`, which describes the
SolidCraft link. Each change gets one to five short sentences in the
section that covers its area. New sections are added only where no
section covers the area:

- `set-up/tools.md`: "Recipes tab", "Tool components and stacks".
- `check-and-post/machines.md`: "Laser".
- `check-and-post/posting.md`: "Heidenhain DIN/ISO and Klartext".
- `multi-axis/checks.md`: "Playback on a work-turning rotary".
- `fundamentals/proving-out.md`: "Stock simulation in CamCraft".
- `getting-started/keyboard-and-mouse.md`: "Screen readers and field order".
- `reference/troubleshooting.md`: "Operations go stale after an update".
- `docs/user-guide/solidcraft-camcraft-mcp.md`: "Face picks follow
  SolidCraft names".

The facts come from the `Changelog:` line and the body of each merged PR.
Each bold UI label was checked against a string in `crates/ui/src`.
Mode names that have no confirmed label (the Wire Contour compensation
modes, the spindle power field) are plain text, not bold.

The pin `site/tests/test_release_028_guide.py` asserts the key facts of
each change in its section, in the style of
`site/tests/test_release_027_batch3.py`.

The PR also adds the `manualToolChange` row to the manifest table of
`reference/post-processors.md`. #4114 added that manifest property, and
the existing test `test_manifest_properties_have_reference_rows` was red
on `main` without the row.

**Why:** The issue asks that each user-visible change since `v0.27.0`
has its description in the guide before the next cut. One short entry in
the existing section keeps each page in its current form and keeps the
merge conflicts with open guide PRs small.

**Considered and dropped:**

- Internal changes get no text: the `camcraft-ui-kit` crate (#4397, no
  visible change), gate and xtask changes (#4388, #4333, #4197), the
  Klartext reader in `camcraft-gcode` (#4361, no app caller yet), and
  mesh bodies for tool components (#4323, no UI or MCP path yet).
- Changes that the guide already described get no new text: the theme
  tokens (#4208, #4073, #4072, #3993, #3992), the MCP idle timeout
  (#4190), the `edm/sinker` example (#4409), the LN2W post and taper
  (#4065, #4171), the Agie name and reentrant openings (#4022, #4040),
  the Haas VF-2SS YT row in the post table (#4186), and the MCP
  component tools (#4268).
- A separate "What's new in 0.28" page was dropped. The release notes
  hold that list. A guide page describes behaviour by area.
- New screenshots were dropped: this is a prose refresh, and a PR does
  not change snapshot PNGs.

**Cost to change later:** Edit the named sections and the facts in the
pin. No code, project format, machine file, post or golden changes.
