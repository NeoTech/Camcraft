## #1121 - crates/ui/AGENTS.md steering document for UI work

**What was decided.** A `crates/ui/AGENTS.md` addendum gathers the UI laws
already in the root `AGENTS.md` and the cited issues, each with a link back
to its source. The root file links it in the Assets section and it joins the
process-file sync list (the list and the `git checkout upstream/main -- ...`
command). `.claude/PROCESS_VERSION` goes 30 to 31.

**Why.** The maintainer asked for a layered AGENTS.md with a UI addendum
(#655, 2026-11-10). Text only: no rule is changed or invented.

**Deviations from the issue text.**
- PROCESS_VERSION is 31, not 30: upstream main was already at 30 when this
  branch was cut (another process PR merged first, so the merger takes the
  next number).
- Left out, no source: the "plain-English user text (no G-code jargon)" rule.
  It appears in no root AGENTS.md section and no decision names it as a law
  (#879 only records that strings already read in plain English). Add it
  when the maintainer states it as a rule.
- Left out, no source: "our own SVG masters" in the icon rule. The root file
  says icons are original or from a permitted set; the SVG-master wording is
  #661's format plan, not a rule.
- Included as in-progress, not as a law: the form seam (#1122), which is a
  filed plan, not a settled rule.
- Icons: the SolidCraft line cites #661 and the 2026-11-12 licence finding in
  the #655 estimate (SolidCraft's brand licence asks for written permission).
  It is recorded as the user's direction for this issue, not as a new root rule.

**Considered and dropped.**
- Duplicating the full laws in the addendum: dropped, since two copies drift.
  Each rule links to its root section instead.
- Editing the root laws to match the addendum wording: dropped; text only.

**Cost to change later.** Low: a text file and four lines in the root file.
A future rule goes into the root file first, then the addendum links it.
