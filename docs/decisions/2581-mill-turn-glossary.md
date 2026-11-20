## #2581 - "Mill-turn" in the docs means the live-tooling kind; turning on a mill gets its own term

**What was decided.** The guide keeps the word "mill-turn" for CamCraft's
machine kind: a lathe (multitasking turning centre) that also mills with
live tooling, `millturn.mch`. The other meaning, a mill used as a lathe
(part in the mill spindle, lathe tools on the table, one work offset for
each tool), is called "turning on a mill". A box with the two meanings is
in `operations-turning.md` and in the Mill-turn section of `machines.md`.
The glossary has the two entries. The `load_machine` row of the
`camcraft-mcp` skill names the two meanings and tells an agent to confirm
which one a user means. Each other use of "mill-turn" in the guide and the
README says "(live tooling)" or "(a lathe with live tooling)", or is on a
page with the box.

**Why.** The UI already uses "Mill-turn" and "Mill-turning" for the
live-tooling kind (`MachineKind::MillTurn`, the built-in preset name, the
`mill-turn` kind in `.mch` files). A docs term that differs from the UI
would add a third name. "Turning on a mill" describes the setup without a
vendor name, and #2580 can use it.

**Considered and dropped.**
- Rename the kind in the docs to "multitasking lathe". Dropped: the UI,
  the preset name and the `.mch` kind value stay "mill-turn", so the docs
  would not match the screen.
- A Zola shortcode for the box. Dropped: the site has one shortcode
  (`icon`), and `_index.md` already uses a plain blockquote for a callout.
- Qualify the enum lists (`mill`, `lathe`, `mill-turn`) and the UI filter
  list in `setups.md`. Dropped: those are literal values, and the text
  next to them defines the term.

**Cost to change later.** Low: docs text only, no code or file format.
