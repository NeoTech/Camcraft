## #1648 - Work offset control moves to the WCS page

**Decided.** The Work offset (G54-G59) picker is drawn only on the WCS page,
in its own group "Work offset (G54-G59)". The Setup page no longer draws it.
The control keeps its command (`SetSetupWorkOffset`) and the setup's own slot
(`setup::wcs_slot`). No new derivation of the slot exists.

**Why.** The maintainer said the Setup page is for the part orientation on
the machine, and the work offset belongs to the WCS page.

**Considered and dropped.**
- Move the "Set WCS from features" row too. The issue names the work offset
  only. The row stays on the Setup page, and its group is now titled
  "Set WCS from features" so the title matches its content. A later issue
  can move it.
- Keep the picker on both pages. This does not meet "Done means".

**Cost to change.** One call in `pages.rs` and one group title.
