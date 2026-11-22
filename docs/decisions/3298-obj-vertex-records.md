## #3298 - OBJ face references count vertex records

**Decided.** The OBJ reader keeps one entry for each `v` record, in file
order, that holds the index of its unique position. A face reference
(positive or negative) resolves against the number of records read so far,
and then maps to the unique position. Duplicate positions are still shared.

**Why.** The OBJ grammar numbers vertex records, not unique positions. The
old reader resolved references against the shortened unique list. It
refused legal files (`vertex reference '5' out of range`) and could resolve
a reference to the wrong record when the record was in range of the short
list.

**Dropped.**
- Remove the dedup and keep all records. Dropped: the dedup is documented
  behaviour, and the mesh stores positions in each triangle, so the dedup
  does not change the output.
- A new cap on the number of records. Dropped: the record list grows
  linearly with the input bytes (each record is at least 7 bytes), and the
  existing `MAX_VERTICES` cap on unique positions stays. A new cap would
  refuse files that import today.

**Cost to change later.** Low. The change is local to
`crates/app/src/geometry/import/obj.rs`.
