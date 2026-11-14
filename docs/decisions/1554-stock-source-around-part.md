## #1554: a stock-source 2D Pocket, Adaptive or Offset clears the stock around the parts

**Chosen:** when the source is the stock and the setup holds parts that
stand in the ladder, the region is the stock's outline minus every part
section standing at the ladder planes (`area::stock_pocket`). This is
the island law of a whole-model source (#12), with the setup's parts
(`setup_align::parts_facets_in`) as the islands. When no part stands in
the ladder, the stock source keeps its own region, byte for byte. When
the parts fill the whole stock outline at a plane, generate refuses by
name.

**Why:** the issue's first expected behaviour is "it roughs the stock
minus the part". A refusal at generate would block the common job
(rough the outside of a plate from the stock) that the user tried. The
keep law (#1603) and `verify` stay as the backstop: an area offset or
extension into the part is still a blocking gouge on the op.

**Considered and dropped:**
- Refuse a stock source that covers a part at generate. Safe, but it
  refuses a legal and common intent.
- Subtract the parts for every plane (no "None" fallback). This would
  move the region of stock-source paths that cut no part (the projected
  outline is not always the region the stock source builds), and so
  change paths and goldens with no safety gain.

**Known limit (same as #12):** the islands are the union over all
ladder planes, so a level above the part's top does not clear the stock
over the part. This leaves material; it never cuts the part.

**Split:** the issue's second finding (a model-source adaptive skips an
outside band narrower than the tool with no note) needs a note or a cut
in the motion crate's pocket generator. A note there can change the CL
notes of existing pockets, so it goes to its own issue.
