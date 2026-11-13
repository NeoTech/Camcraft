## #1386 - Explain G-code: the post option, the PRELUDE helper, the manifest and the refusal

**Decided** (unattended, from the design on #1358):

- `PostOptions.explain` (default off, per-setup override through the
  existing `SetPostOptions` / `SetSetupPostOptions` commands, so undo and
  redo come for free). `posting::options_value` writes
  `postOptions.Explain = true` only when on; the project file writes an
  `explain` key (document and setup) only when on. Off, every script sees
  exactly the object it saw before and every file saves as before.
- The helper lives in a new `crates/post/src/post/explain.rs` as
  `EXPLAIN_JS`, loaded right after the core PRELUDE by `Script::load`,
  instead of growing `js.rs` (already past 400 lines). It is armed by the
  engine (`Script::arm_explain`) only when the option is on and the
  manifest states a form; unarmed, `explain()` returns `""`, so a post
  that calls it writes byte-identical programs while the option is off.
- `explain(words)` returns the trailing comment, delimiters included
  (`(TEXT)` or `; TEXT`); the post writes it after its words. A block is
  explained only when it carries a G / M / T / S word; F is explained
  beside them. Unknown G / M words are skipped, never guessed. Meanings
  are joined with ", ".
- Guards in the helper, not in each post: printable ASCII only, `( ) ; %`
  removed, the LinuxCNC active-keyword lead ("- ") in every form, and a
  cut to `commentMax` (default 60, delimiters included; 8..=255, anything
  else refused by name) - back to the last whole meaning where one fits,
  a single long meaning cut short.
- `explains = "paren" | "semi"` in the manifest; Explain on with no form
  (or an unknown one) is refused by the post's name before any hook runs,
  next to the #902 `programUnits` refusal (`units::post_name` is now shared
  by both).
- Wording: upper case, short, written fresh for this repo from RS-274 /
  ISO 6983 and the LinuxCNC G/M code lists; the unit is named once, on the
  unit word (G20 / G21).

**Considered and dropped:** a per-post property or machine setting (the
research comment: it is the operator's choice for one program);
explaining bare continuation blocks (noise - modal words are already
written only on change); putting the helper text at the end of the
script source (would keep error line numbers but would override a user
post's own `explain`); an MCP tool and the dialog switch (their own issue,
#1390).

**Cost to change later:** the wording table is one JS object in
`explain.rs`; a wording change touches the Explain-on goldens of the
posts that opt in (#1387-#1389), never an Explain-off golden.
