## #1075 - STEP records may nest at most 64 parentheses deep

**Decided:** before ruststep parses a STEP file, one linear byte scan
(`geometry::import::step_depth`) tracks parenthesis depth outside strings
(`'...'`, `''` escaped) and comments (`/* ... */`) and refuses the file past
`MAX_NESTING = 64`, naming the cap, the record's `#id` and the line
(`\n`-counted, so CRLF and LF agree). Both functions that hand bytes to
ruststep call it first: `step::parse_with_chord` (every model import:
`ImportModel`, `ImportFixture` / fixture models, the tool-body mesh, the
MCP `import_model` and script import all reach it through
`import::load_model[_graded]`) and `step_csys::read_tool_csys` (the tool
body's coordinate systems).

**Why 64 - the evidence (2026-11-12, Windows 11, this machine):**

| ruststep (pinned rev 90fdc935), one record nested N deep | 1 MiB stack | 2 MiB stack | 8 MiB stack |
|---|---|---|---|
| debug build, last N that survives | 400 | 821 | 3341 |
| release build, last N that survives | 530 | 1086 | 4418 |

About 2.5 KB of stack per level in debug, 1.9 KB in release. The smallest
stack an import runs on by default is 1 MiB: the Windows main thread, where
the GUI's import falls back to running inline when its worker thread cannot
be spawned (the worker and test threads get Rust's 2 MiB). Every committed
STEP fixture (`testdata/models/*.step`, a real AP214 export included) nests
3 deep; real AP203/AP214 records rarely pass 4-5. 64 is about 20x the
deepest real record and about 6x below the debug crash on 1 MiB, leaving
room for whatever the caller already has on the stack. It also matches the
cap the solidcore STEP importer (#916, PR #941) already uses.

**Considered and dropped:**
- (b) parse on a thread with a large, bounded stack and treat its death as
  the refusal: a stack overflow aborts the whole process on every platform,
  it cannot be caught on a thread; a bigger stack only moves the depth.
- Hosting the scan in PR #1006's `step_diag.rs`: it is not merged; a new
  file keeps the two PRs from colliding (both touch `parse_with_chord`, in
  different hunks).
- A cap near the crash depth (e.g. 256): no real file needs it and it eats
  the headroom on a 1 MiB stack.

**Cost to change later:** one constant; the measured table above says how
far it can move. A ruststep that parses iteratively would make the scan
redundant but harmless.
