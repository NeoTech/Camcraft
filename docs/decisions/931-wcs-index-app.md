## #931 - work offsets as an index, the post supplies the word (slice 2: app)

**Decided.** App hands the post the setup's 1-based offset index, not a
word. `camcraft_post::post::Dump::wcs` is now a `u32` (1 = the first
offset); `posting::wcs_context` returns `(index, origin)`. The six-offset
cap is gone from core: `SetSetupWorkOffset::new` checks the pick against
the setup's own post (`setup_offsets::check`, the post resolved by
`setup_mount::post_of`), the loader keeps only a sanity bound (pick
0..=254, the `u8` the height digest stores), and `setup::work_offset`
returns the stored pick unfiltered. A new verification clause,
`setup_offsets::range_problems`, judges every setup with an operation in
the program against its own post and blocks the post by setup name
(`setup Setup1 uses work offset 9, but its post Fanuc (built-in)
expresses work offsets 1-6 only ...`). The engine's slice-1 refusal stays
behind it as a second line.

**No fallback anywhere (triage point 2).** A stored pick the post cannot
express is never read as offset 1: not in `setup::work_offset`,
`wcs_context` (the travel check reads the index), the stream, the label
or the setup sheet. The label and sheet show the post's name for the
index, or "work offset n" when the post names none - never another
offset's name. Pinned by
`crates/app/tests/wcs_index.rs::offset_nine_on_a_six_offset_post_refuses_by_setup_name_and_never_posts_at_offset_one`.

**Names are display, cached.** `setup::work_offset_names(world, setup)`
(in `setup_offsets`) runs slice 1's `work_offsets(cps)` once per
(resolved post label, script text) and memoises it in a bounded map
(cleared at 64 entries, so an edited post file cannot grow it without
end). For a file post the text is re-read per call to key the cache;
the script itself is never re-run per frame. An unreadable post shows the
six legacy names (display only); its offsets above 6 are refused by the
command and the gate, since nothing can confirm them.

**File format.** Offsets 1-6 keep their words `G54`..`G59` in the project
file, so every existing file reads and writes byte-identical. Offsets 7
and up are written as their number (`"work_offset": "7"`). No format
version step: the key and its type are unchanged.

**Release note (triage point 8).** A project that uses work offset 7 or
higher cannot be opened by builds before this change - their loader
refuses the number as an unknown work offset. Offsets 1-6 open everywhere.

**`WCS_SLOTS` split (triage point 1).** The public word table is gone.
What remains is `LEGACY_WCS_NAMES`, crate-private: the legacy ENTITY
names (the implicit setup's `G54` WCS, pre-v54 `<setup>/G5x` slots) that
the name reservations and the v51/v53 migrations read. The pre-v52
migration (`project/wcs_migrate.rs`) keeps its six-slot check: those
files were written by builds that could only store slots 0-5, and a
height slot there indexes the legacy per-slot WCS list, so a higher one
is a corrupt file, not a post limit.

**Shared offsets compare indexes (point 7).** `shared_word_problems`
already compared the stored pick; it now names the offset by the
setup's post's name, so two posts calling one offset differently are
still one register.

**Slice 2 keeps UI and MCP working through app.** The Setups combo still
lists offsets 1-6, now named by the setup's post (Sinumerik shows
`G505` / `G506` for 5 and 6, matching what it posts since #934); MCP's
`word` input still takes `G54`..`G59` and its `word` output is the post's
name. Slice 3 widens the combo to the post's maximum and adds MCP's
`index`.

**Considered and dropped.** Keeping `Dump::wcs` a `&str` carrying the
number as text: it would keep the word-shaped type the issue removes.
Judging the gate against the script being run rather than each setup's
own post: every GUI and MCP flow runs the setup's own post, and a caller
that passes another script is still refused by the engine's own check.
