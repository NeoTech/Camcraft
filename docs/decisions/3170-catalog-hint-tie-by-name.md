## #3170 - The catalog hint breaks a size tie by name

**Decided:** `tool_gate::catalog_suggestion` keeps its order: the entry of
the family's first stocked kind nearest 6 mm, then the smaller size. When
two entries still tie, the entry with the lower name wins. This is a
deterministic rule, not a preference for one tool line. It makes the hint
independent of the file order of `assets/tools/catalog.json`.

**Why:** #3159 added "Router ball nose 6 mm" before "BN-0600" in the
catalog file. The old tie kept the first entry in file order, so the
refusal for a ball-nose operation changed from 'BN-0600' to the router
cutter, and `the_refusal_names_a_fitting_catalog_entry` failed on main.
The taper kind has the same risk: "Router V-bit 60 deg 6 mm" and "Router
V-bit 90 deg 6 mm" come before "CM-0600-90". The name rule gives
'BN-0600', 'EM-0600-4FL' and 'CM-0600-90' for any file order. The pin
`a_size_tie_picks_by_name_not_file_order` covers the two orders of the
ball-nose pair and the reversed catalog for ball-nose, end mill and taper.

**Dropped:**
- Change the test expectation to 'Router ball nose 6 mm'. The hint would
  still depend on file order, and the next catalog edit could move it
  again.
- Move the router entries after the metal ladder in `catalog.json`. This
  hides the fault; the order stays load-bearing.
- Prefer an entry by its material group. The suggestion has no material
  context, and this is new behaviour for a P2 fix.
