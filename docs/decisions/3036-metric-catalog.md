## #3036 - Metric tap-drill and clearance drills, a 3 mm ball and a 6 mm bull nose in the tool catalog

**What was decided**

The starter catalog (`assets/tools/catalog.json`) gets 16 tools:

- `BN-0300`: a 3 mm ball end mill.
- `BB-0600-R1`: a 6 mm bull nose end mill with a 1 mm corner radius.
- 14 jobber drills. Each coarse thread from M3 to M12 gets a tap drill and
  a clearance drill:

  | Thread | Tap drill | Clearance drill |
  |---|---|---|
  | M3 | `DR-0250` | `DR-0340` |
  | M4 | `DR-0330` | `DR-0450` |
  | M5 | `DR-0420` | `DR-0550` |
  | M6 | `DR-0500` | `DR-0660` |
  | M8 | `DR-0680` | `DR-0900` |
  | M10 | `DR-0850` | `DR-1100` |
  | M12 | `DR-1020` | `DR-1350` |

The tap drill is the nominal diameter less the coarse pitch. The clearance
drill is the ISO 273 "medium" series. The names follow the catalog's own
scheme: the diameter in hundredths of a millimetre.

Each new tool goes into its family in diameter order. Thus the picker shows
the drills from the smallest to the largest. The other tools keep their
order.

The cutting data copies the rules of the catalog's own rows:

- Drills: the shape of `DR-0600` (118 deg point, shank equal to the
  diameter, aluminium). The flute length is 3 x D, rounded to the nearest
  0.5 mm. A length on a 0.5 mm step is exact in binary floating point.
  With a length such as 9.9 mm, the spun holder outline and the holder
  cylinder differed in the last bit (`spun_tests`,
  `a_parametric_holder_stacks_bit_identically_to_the_cylinder`). All
  older catalog rows have such lengths already. Below 6 mm, the rpm and the
  feed are interpolated between the existing drills `DR-0200`, `DR-0300`,
  `DR-0400` and `DR-0600`. From 6 mm up, the cutting speed is that of
  `DR-0600` (about 140 m/min) and the feed per revolution is 0.0025 x D,
  also from `DR-0600`. The rpm is rounded to 100 and the feed is rounded
  down to 5 mm/min.
- `BN-0300`: the shank, flute length and rpm of `EM-0300-2FL`. The feed is
  750 mm/min. That is 5/6 of the flat end mill's feed, which is the ratio
  of `BN-0600` to `EM-0600-4FL`.
- `BB-0600-R1`: the shank, flute length and feed of `BN-0600`, which has
  the same diameter and the same flute count. The rpm is that of
  `EM-0600-4FL`.

**Why**

The release QA runs had to copy a tool and change its sizes by hand for
an M6 tap drill, an M6 clearance hole and a small ball (issue #3036). A
copy that is edited by hand is easy to get wrong. The values come from
general machining knowledge and from the catalog's own rows. No vendor
catalog was used.

Interpolation below 6 mm keeps the rpm in order: a larger drill never
turns faster than a smaller drill. One surface-speed rule for all sizes
gave a 3.3 mm drill a higher rpm than the existing 3 mm drill.

**Considered and dropped**

- Add the new tools at the end of the list, like the undercut cutters of
  #1444. Dropped: the drills are one family, and a user looks for a drill
  among the drills.
- A 0.5 mm corner radius on the 6 mm bull nose. Dropped: `R1` follows the
  existing `BB-1000-R1` and its name scheme.
- Spot drills and thread-mill inserts. These have their own issues (#897,
  #893).
- Change the spun holder code so that it keeps the band length exactly.
  Dropped for this issue: that code builds the holder outline that the
  collision checks read. A change there is outside the scope of a
  catalog change.

**Cost to change later**

Low. A value is one field in `assets/tools/catalog.json`. A tool that a
user added to a project keeps its own copy of the data. A change to the
catalog does not change existing projects.
