## #3180 - Use one selected DXF chain for a Wire Contour

**Decision.** The operation stores the existing sketch entity and stable chain ID. The resolver keeps the ordered analytic segments, their original primitive indices and the stock-face placement. Generic generation accepts a closed chain of straight segments with coincident joints. The existing wire generator applies the machine condition's signed kerf offset once. The post checks the nominal profile through the full height of the box stock.

**Reason.** The chain supplies the finished outline. The operator specifies the punch or die side, threading point and physical slug capture. A through cut needs stock at every height. Every unselected primitive, including rejected chain fragments, must clear the full bounding box of the cut, its lead, kerf and released piece. A primitive on a separate layer has the same protection.

**Alternatives dropped.** Reading the entire drawing can cut an unrelated chain. Chords change an analytic arc and move its offset. A stock silhouette cannot verify a sketch's nominal outline. An earlier operation can change the material that holds the piece. This first case uses fresh axis-aligned box stock, one placed drawing and a single vertical pass. The resolver refuses additional bodies and unverified boundary controls.

**Cost to change.** An analytic consumer can use the same selected chain and placement. It must bound its actual leads and kerf, verify its signed offset and prove slug retention. Additional stock shapes, sequential cuts, fixtures and tabs need corresponding material and clearance checks. The project format already stores the stable pick.

The drawing computes its exact primitive signature at construction. An operation status read uses that stored value. Reopening a project reconstructs the signature from its primitives. A coordinate edit below the chain ID quantum changes the signature.
