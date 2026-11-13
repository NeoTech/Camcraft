# 1467 - Solidcraft reader: coordinate limit and body fuzzing

## Coordinate limit of a `.sc` final body: 1e6 mm

- **Chosen:** `camcraft_kernel::solidcraft::MAX_COORD_MM = 1.0e6` (1 km). The
  reader checks every vertex of the decoded final body before the kernel
  builds it: a non-finite coordinate or one past ±1e6 mm is refused by name
  ("the final body's vertex i ..."). Tolerances are not checked again: the
  record's own validator already refuses any outside (0, 1e3] mm.
- **Why:** solidcore-record's decoder already refuses NaN/infinities and
  anything past 1e12, but 1e12 mm is far past any part a mill can hold; a
  body out there is damage, and its geometry would reach the kernel and the
  stock maths. 1 km is past every machine envelope and leaves room for
  designs drawn far from their origin.
- **Considered and dropped:** 1e5 mm (100 m) - tighter, but a design placed
  in a site or assembly frame can sit tens of metres out; the decoder's own
  1e12 only - lets values through that no CAM path should see; reading the
  limit from the machine file - the kernel crate has no machine, and the
  import runs before a setup exists.

## Fuzzing the body, not only the container

- **Chosen:** the new `solidcraft_hostile` test splices damaged or edited
  bodies into the sample with the slice table, both SHA-256 digests and the
  ZIP CRC-32s made to agree, so each case reaches `native::decode` and the
  vertex check; every refusal must name the final body (a case stopping at
  the container fails the test). Values the encoder will not write (NaN,
  infinities, 1e13) are patched over a sentinel coordinate's bytes.
- **Also pinned:** a deflate entry that inflates past its declared size is
  cut off at size + 1 and refused (the reader's existing `take` bound).
