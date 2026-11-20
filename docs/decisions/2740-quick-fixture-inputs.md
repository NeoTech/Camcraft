## #2740 - Select quick suites for changed fixtures and importers

**What was decided:** Add fixture readers to the existing quick selection.
Scan the source files of each integration suite, including declared helper modules.
Match quoted paths against directories that contain changed test data, fixtures, goldens, or snapshots.
Resolve paths from the repository root, crate root, and source directory.
Use the static prefix of a formatted fixture path to select readers of nested golden directories.
Use lexical path resolution so new and deleted files do not need to exist.
Keep all suites selected by the existing rules.

Changes to the app STEP/mesh importer or kernel Solidcraft and STEP import code also select three suites:

- `app:feature_edge_g1_join`
- `kernel:step_agreement`
- `kernel:machining_mesh`

**Why:** PR #2664 changed the importer and added a STEP fixture.
The quick gate omitted suites that read the model directory or use an independent importer.
The new rules select these checks before the change reaches main.
The directory scan is conservative and does not interpret test I/O.

**Considered and dropped:** Gate maps alone cannot detect a changed fixture that a suite reads.
A full batch gate for every fixture change adds work unrelated to the changed directory.
Coverage-based selection belongs to #2738 and is outside this fix.

**Cost to change later:** Update the importer path list when an importer moves or a required suite changes.
The text scan can select extra suites when they name a sibling fixture in the same directory.
A computed path without a fixture directory name can need an explicit rule or coverage-based selection.
Future selection can replace this helper while the plan tests preserve the required checks.
