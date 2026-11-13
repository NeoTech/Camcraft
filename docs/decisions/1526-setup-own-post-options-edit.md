## #1526 - A post option changed on one setup never changes another's program

**What was decided.** The post-option edits (Skip pauses and homes,
Output, Threading, End units - every arm of `State::route_post_options`,
`crates/ui/src/state_setup_post_options.rs`) route by the #1515 law
through one new reader, `State::post_options_target`:

- the active setup owns its post options - the edit lands there, as before;
- else, when the document has **two or more setups**, the edit becomes the
  active setup's OWN options through the existing undoable
  `SetSetupPostOptions` (a copy of the options it posts with now, with the
  one change) - the document's options, which every other following setup
  posts with, are not written;
- else (no setups, or the only one) it sets the document's options
  (`SetPostOptions`), exactly as before.

It reuses #1515's `shared_mount_target` (made `pub(super)`), so the
machine, the post and the post options follow one stated rule. The
setup's Post page says by name, under the options' own/document radios,
where an edit lands: "A post option changed on this page is Setup2's own -
no other setup changes." or "... sets the document's post options - every
setup without its own uses them."

**Why.** The report: with two setups following the document's options, a
change on Setup2's Post page wrote the document's options, so Setup1's
posted program changed too (e.g. its optional stops and trips home
silently dropped by "Skip pauses and homes") - wrong posted G-code from a
page that names one setup (P1, Prime Directive).

**Considered and dropped.**
- "Bind own only when ANOTHER setup follows the document": dropped for the
  reason #1515 gives - a setup created later still inherits the edit.
- Refuse the edit until the user picks "This setup has its own post
  options": one more click for a request whose meaning is clear.
- A separate document-level options page: none exists; the document's
  options in a multi-setup document stay reachable by the project file
  and by `SetPostOptions` with one setup.

**Consequence, accepted.** Output (one file per operation) and Threading
are program-level: `setup_post_options::program_problems` refuses a
program whose setups disagree on them ("post the setups one at a time").
Changing Output on Setup2 of a two-setup document therefore now makes the
whole-program post refuse by name until Setup1 is set the same way (or
Setup2 follows the document again), where it used to silently switch both.
A visible refusal is the Prime Directive's other legal outcome; a silent
change to Setup1's output is not.

**MCP.** There is no post-options tool on the MCP server at all
(`crates/app/src/mcp/` never names `PostOptions`), so the MCP path has no
bleed; the missing parity is filed as a P5 `MCP:` issue (named in the PR).

**Cost to change later.** One function (`post_options_target`).
