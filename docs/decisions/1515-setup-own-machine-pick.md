## #1515 - A machine picked on one setup never re-mounts another

**What was decided.** The machine verbs (library combo, Load other, Edit
apply, Clear, the Machines pane's "Use for this setup", the ribbon's
machine-mode switch) and the post pick route by one law
(`State::machine_target` / `post_target`, `crates/ui/src/state_setup_mount.rs`):

- the active setup owns its machine (post) - it edits that, as before;
- else, when the document has **two or more setups**, the pick becomes the
  active setup's OWN machine (post) through the existing undoable
  `SetSetupMachine` / `SetSetupPost` - the document's mount, which every
  other following setup reads, is not written;
- else (no setups, or the only one) it sets the document's machine
  (post), byte for byte as before.

The setup's Machine and Post pages say by name, under the own/document
radios, where a pick lands: "A machine picked on this page is Setup2's
own - no other setup changes." or "... sets the document's machine -
every setup without its own uses it."

**Why.** The report: Setup1 with no machine, a pick on Setup2's Machine
page, and Setup1 showed it too - Setup2 followed the document, so the
verb wrote the document's machine (`LoadMachine`), which Setup1 also
followed. A setup reading another setup's machine gets the wrong travel
and spindle refusals (Prime Directive). The maintainer's triage asked to
fix the ownership, not the one symptom, so the post pick (the same
document-level write, wrong post = wrong G-code) takes the same law.

**Considered and dropped.**
- "Bind own only when ANOTHER setup follows the document": also safe, but
  a later new setup (which follows the document) would still inherit a
  pick made on one setup's page - the "default into another setup" the
  user called dangerous. "Two or more setups" is the simpler rule to state.
- Refuse the pick and ask the user to choose "This setup has its own
  machine" first: one more click for the common case, and a refusal for
  a request whose meaning is clear.
- Changing the MCP `load_machine`: it already takes `setup` (that setup's
  own machine) and names the document explicitly without it ("without,
  it is the document's"), so an agent always names its target; nothing
  implicit to fix. Pinned by `mcp_setup_mount_trace.rs`
  (`a_setups_own_machine_picked_over_mcp_reaches_its_own_posted_file_and_undoes`:
  Setup1 keeps the VMC while Setup2 takes its own 770M).

**Cost to change later.** One function (`shared_mount_target`). In a
multi-setup document the document's machine can no longer be changed from
a setup page; a setup that wants it again picks "Use the document's
machine". A document-level machine page, if ever wanted, would write
`LoadMachine` and name every setup that follows it.

**Left as is.** Post OPTIONS (`SetPostOptions`, #389) still write the
document's options for a setup that follows them - the same shape;
filed as a follow-up rather than widened here.
