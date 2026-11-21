## #3069 - Generic starter tools for hobby routers

**Decided:** Add 22 unbranded cutters to the embedded tool catalog. Include
up-cut, down-cut, compression and ball nose cutters at 3 mm, 6 mm, 1/8 in
and 1/4 in. Include sharp-point 60 and 90 degree V-bits with 6 mm and
1/4 in shanks. Include 25 mm and 1 in surfacing bits. Store all lengths in
mm. The inch names retain the exact conversions of 3.175, 6.35 and 25.4 mm.

Each tool states a nominal flute length and stick-out. The existing model
stores stick-out as flute length plus exposed shank length. These generic
dimensions and the holder dimensions must match the actual assembly before
machining. The cut direction appears in the name. The model does not state
the compression transition height.

Use positive baseline spindle speeds and feeds because the tool reader
requires them. Use 18000 rpm and 600 mm/min for the small cutters. Use
12000 rpm and 600 mm/min for the surfacing bits. These values are not
verified settings for a real cut. Leave material-specific cutting rows
empty until the wood cutting data in #3068 lands. Mark the surfacing bits
as unable to cut through their centre.

**Why:** The existing catalog powers the library picker and the normal
library import. Data additions make the starter set available without a
new command, dependency or file format. Separate tests preserve the legacy
tool order and check the router shapes, reach and import command.

PRs #2714 and #2836 touch other catalog rows. PR #2714 also extends the
legacy `ORDER` constant. This change keeps that constant intact and changes
the validation function. Each touches the same file, different function or
row; whichever merges second refreshes.

**Considered and dropped:** Vendor libraries remain in the community
repository. Flat-tip V-bits wait for #895; these entries describe only sharp
points. A new helix or compression transition field needs separate model
work. Wood cutting rows must not use a metal material as a substitute.
Changing the library reader or the picker adds no value for this set.

**Cost to change later:** Low for nominal sizes and baseline values. A
compression transition or a flat-tip profile needs model and validation
work. Material-specific cutting rows can follow the wood data change.
