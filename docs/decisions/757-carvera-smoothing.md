## #757 - Carvera post names a smoothing ask instead of dropping it

**Decided.** `assets/posts/carvera.cps` now reads the `Smoothing` /
`SmoothingMode` PARAMs (#616) and, when an operation asks for smoothing,
writes one comment per program - `(SMOOTHING NOT WRITTEN - NO CARVERA FORM
STATED)` - at the end of that section's preamble (after the coolant word,
before the first move), exactly where and how `mazak.cps` writes its note.
The ask resets at section end. Nothing else in the program changes; with
smoothing off the output is byte-identical to before (the Carvera golden
`crates/app/tests/golden/nc/carvera/two_tool_pocket.nc` is unchanged).

**Why.** The Carvera firmware's supported-codes tables (Makera's
Smoothieware-based firmware, as documented at carvera-community.gitbook.io,
firmware -> supported commands -> GCodes / MCodes) list no path-blending or
smoothing word: no G64, G61, G5.1 or similar. Writing any of them would be a
guessed word the controller may halt on or ignore; writing nothing silently
lets the user believe smoothing is on. The one-time note is the established
pattern for a dialect that cannot state the word (Fanuc, Mazak).

**Considered and dropped.**
- A G64 P<tol> like Tormach/LinuxCNC: not in the firmware's tables - an
  invented word, refused by the issue's own brief.
- A refusal (`message()`), blocking the post: smoothing is an opt-in
  optimisation, not a safety need; the path the simulation verified is
  posted exactly, so a note is honest and a refusal would be too strict.
  Fanuc and Mazak already set the note as the precedent.
- A note per section: one per program matches Fanuc/Mazak and avoids noise.

**Cost to change later.** If a later firmware documents a blending word,
`smoothOn` writes it and `smoothOff` undoes it like the other dialects; the
`smoothing.rs` Carvera case is updated with it.
