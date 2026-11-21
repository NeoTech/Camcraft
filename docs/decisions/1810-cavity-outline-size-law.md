## #1810 - Sinker EDM: judge a body electrode against the cavity outline

**Decided:**

- Store `SinkerBurnOp::cavity_outline` as `[x, y]` points in mm relative to the burn point. Keep `cavity_x` and `cavity_y` as the outline's extents. An empty outline keeps the existing extent law.
- Carry the outline through the project DTO, the operation digest and the rough/finish cavity comparison. The additive JSON key is omitted when empty. Older files keep their previous meaning. An older reader ignores the key and uses the conservative extent law, which can refuse the diamond finish.
- Include the parameter in the MCP operation schema as an optional array with a `null` default. The `electrode_from_faces` reply supplies a nonempty array for a body electrode and `null` for a rectangle. Both outlines use the same burn point. A `null` params patch clears the outline through the existing JSON merge patch.
- Compare validated cavity and electrode regions with sharp inward wall offsets. The electrode must fit inside the cavity offset by `max(undersize - SIZE_SLACK, 0)`. A finish must cover the cavity offset by `gap + orbit_radius + SIZE_SLACK`.
- Use the existing scalar law with virtual wall widths of `cavity - 2 * undersize`. This preserves its numeric, condition, floor and orbit checks. These widths describe wall offsets; they are not the electrode's actual axis extents.
- Refuse malformed outlines and, for a rough or finish, contradictory cavity extents at command creation. A single burn still does not read the cavity sizes. Generation repeats the bounds checks defensively. Refuse consumed or split reference offsets, excess electrode material and an incomplete finish by name. A stated cavity outline requires an electrode outline. A rectangular tool cannot silently replace it.
- Refuse a positive-radius vector orbit for an outline. Directions that reach the axis-aligned walls do not prove reach on an angled wall.
- Keep the independent stock verdict. A round spark gap still leaves a recess at a sharp corner. The wall law does not remove that recess or change the stock tolerance. The result is not certified for a real EDM control.
- Stack on PR #2714 because both changes affect `stamp_cutting`. PRs #2678 and #2554 affect other functions in `mcp/job.rs`. The PR that merges second must refresh.

**Why:** The extent law measures the diamond's corners rather than its walls. An inward offset moves a sharp corner farther than the condition's undersize. The correct finish electrode can therefore fail the extent check. The wall-offset comparison follows the same Clipper2 sharp-offset construction that derives the electrode. The separate stock verdict still checks the actual burn. The DTO omits an empty outline, so the MCP patch check rejects an empty array as a field lost during serialization. A `null` patch removes the optional key and keeps that existing check intact.

**Considered and dropped:**

- Keep the extent-only finish law. It refuses the requested diamond finish.
- Increase `SIZE_SLACK`. That would also weaken the rectangular size law and permit unexplained wall errors.
- Accept nominal equality between two unvalidated outlines. That would not reject malformed geometry or an oversized electrode.
- Judge the round gap at every sharp corner as exact wall coverage. The physical recess remains, and the independent stock verdict must judge it.
- Accept vector orbits based only on their radius. Their directions do not prove coverage of angled walls.
- Add a geometry dependency. The motion crate already provides the required region operations.
- Add an exception to the generic `mcp/edit.rs` patch check. The existing `null` removal supports this optional field without a broader edit rule.
- Serialize the new key for old rectangular operations. Omitting it keeps their stored files unchanged.

**Cost to change later:** Medium. A different wall or corner law changes one motion module and its tests. Removing the parameter also changes the operation, DTO, digest and MCP schema. Directional orbit support needs a separate proof of coverage.
