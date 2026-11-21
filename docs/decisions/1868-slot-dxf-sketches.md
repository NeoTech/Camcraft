## #1868 - Slot and Trochoidal Slot on DXF sketch chains

**What was decided**

- A Slot and a Trochoidal Slot read one picked chain of a DXF profile.
  They keep their two points and their width (`a`, `b`, `width`). The
  project file does not change, and no version bump is necessary.
- The pick seeds the numbers in the pick's own command
  (`CreateOperation`, `SetOperationGeometry`). One undo takes back the pick
  and the seed. This is the law that #1504 made for model picks.
  - One closed line-arc slot outline seeds `a`, `b` and `width`.
  - One open straight chain is the centerline. It seeds `a` and `b`. The
    typed `width` stays.
- Generation judges the stored slot against the picked chain, as the world
  is at that time. A slot that left its outline, or whose ends are not the
  ends of its centerline, is refused by name. A moved or re-imported
  profile cannot cut in the old position.
- A closed chain is a slot outline only when the fitted straight slot
  covers 99.5 % or more of the area of the chain. A rectangle, a circle, an
  L or a keyhole is refused by name.
- A selection with more than one chain is refused by name. The whole
  profile as the source is refused by name.
- The refusal of a Slot that is narrower than the tool now gives the two
  numbers.

**Why**

- The room law of `machining2d::SlotBound` (#1504) is reviewed and pinned.
  A sketch outline is a face room, and the existing fit and confine laws
  apply without a second algorithm. A centerline is one new room kind
  (`Room::Line`) with a small law of its own.
- Seeded numbers stay visible and editable on the operation page. The
  generator, the machine area, the WCS rotation and the verification read
  `a`, `b` and `width` in many places. A second source of slot geometry in
  the job would need a change in each of them, in safety code.
- The stock simulation and the verification use the setup stock
  (`part_solid = false`), as #1864 made for the 2D Contour.
- The area test refuses an outline that only contains a slot. The model
  face law of #1504 accepts a rectangular floor as a room, because a floor
  face is the room of a slot. A drawn outline is the slot itself, so the
  rule is stricter.

**Considered and dropped**

- Bent and arc centerlines (an open chain of lines and arcs). The plain
  Slot generator cuts a straight channel only. The Trochoidal Slot
  generator takes a point chain, but its machine area, its fit law and its
  parameters have two points. A curved channel needs a new parameter form
  and a new machine area. Such a chain is refused by name. This needs an
  issue of its own.
- Closed chains as centerlines (a circular groove). A closed chain is read
  as an outline only. One chain cannot be the two things without a new
  parameter, and a wrong guess cuts in the wrong place.
- More than one chain in one operation (a layer of slots). The operation
  holds one `a`, `b` and `width`. Each slot is one operation for now.
- New `SrcRef` or `OpParams` fields for the sketch. The existing
  `SrcRef::Selection` with `FeatureSel::SketchChain` / `SketchLayer` of
  #1864 holds all that is necessary.
- An analytic read of the outline (two lines and two half circles from the
  chain segments). The area test accepts the same outlines, and it also
  accepts an outline whose arcs a CAD system wrote as short lines.

**Cost to change later**

- Curved centerlines add a parameter form. That is a project file change
  with a version bump.
- The 99.5 % limit is one constant (`OUTLINE_COVER` in
  `operation/slot_sketch.rs`).
- The messages of a judged sketch outline say "the picked face" (the text
  of the face room). A separate room name is a small change in
  `slot_room.rs`, which is near the 400-line limit.

## Review corrections

**What was decided**

- Group natural contour chains only within one profile entity. Parent
  checks use both the entity and the chain ID.
- Key sketch highlights by the placed world pose. Keep `None` in the key
  when the sketch has no stock.

**Why**

- Independent profiles can nest in world space while both resolve to an
  outside cut. A combined region can turn the inner profile into a hole
  and reverse its cut side. Separate generation keeps the resolved sides.
- A chain ID belongs to one profile. An equal ID in another profile is
  not its parent.
- A profile rides its stock. The highlight position changes when the
  stock moves or its height changes. Without stock, the profile has no
  placement and draws no highlight.

**Considered and dropped**

- Grouping chains from separate profiles by chain ID alone. This loses
  entity identity and can change the resolved cut sides.
- Using a sketch's local `Location` as the highlight key. That value
  does not change when the stock moves, changes height, or is removed.
