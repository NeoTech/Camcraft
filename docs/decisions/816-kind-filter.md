## #816 - Machines pane kind dropdown filters the list

**Decided.** `MachinesPane::new_kind: usize` becomes `kind_filter: Option<usize>`
(`None` = "All", the default). One dropdown both filters the list and picks the
kind New blank starts. Under "All" New blank is disabled ("Pick a kind first");
with a kind it is enabled and accent-filled. The count label reads "k of N machines"
while filtered. A machine with no stated kind shows only under "All".

**Why.** Matches the issue; a blank file needs a kind, so "All" cannot start one.

**Chosen on my own.** If the selected row is filtered out, the selection is
cleared, so Edit / New from template / Use for this setup never act on a hidden
row. An empty filtered list shows "No machines of this kind.".

**Dropped.** Persisting the filter (ephemeral view state, like the dock);
remapping `selected` to filtered indices (it stays an index into the full list).
