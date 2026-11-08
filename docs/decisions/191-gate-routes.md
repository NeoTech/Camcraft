<!-- moved from RECOMMENDED.md for PR #227 (docs/decisions rule, PR #250) -->

## #191 - a refused Create click takes you to the fix (2026-11-07)

- **Chosen:** a dark Create cell or dropdown row that is clicked anyway
  keeps its reason on the status line and then routes (view state only,
  nothing queued): no tool / no fitting tool → the Tool library page
  (`UiTarget::Tool("")`, " - opened the tool library"); a kind the mounted
  machine cannot run → the machine door opens on its workspace list
  (" - pick a workspace"). `gate_route` mirrors `gate_note`'s cases in its
  order, so the words and the destination cannot disagree.
- **Why:** the reporter read the status line as an error to hunt for;
  the click now lands where the fix is.
- **Not done:** a WCS route - creating an operation has no WCS gate (an
  unplaced slot means G54 at the origin), so there is nowhere to send
  the user yet. Opening the tool *editor* on a catalog draft instead of
  the library page was dropped: the library shows what is racked and the
  catalog suggestion the reason already names.
