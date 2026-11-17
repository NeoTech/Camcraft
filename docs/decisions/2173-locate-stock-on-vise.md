## #2173 - Locate the stock on an existing vise

**Decided.** The locate takes a held body: the part (as before) or the
stock of the fixture's setup (`LocateBody::Stock`). With the stock held,
the first pick of each mate is a stock face or point. The solve moves the
part, and the stock follows it by the stock-follow law, so the part keeps
its place in the blank and the stock lands on the mates. The Fixtures page
and the Model page hold the stock by default when the fixture's setup has
one ("Place by: Stock faces / Part faces"). The API and the MCP
`locate_part` keep the part as the default (`"body": "stock"` opts in).
`list_features` lists the faces of a stock, so MCP can name the mates.

**Why.** A vise holds the blank, not the finished part. The fixture
collision checks and the carried-workholding check read the stock where
it is, so a stock that cannot be seated on the vise leaves those checks
judging a guessed place. Moving the part with the stock keeps the
part-in-blank relation that the toolpaths use.

**Refusals.** A stock locate that names no part, a setup with no stock,
and a mate on the wrong body are refused by name. A square stock re-fits
round a turned part with the allowance of each setup side; when that
re-fit would leave the stock faces off the mates, the locate is refused
("would not land on the mates") instead of posting a stock off its seat.

**The mate check judges each picked face, not the box (review of
c492f7006).** Equal boxes do not prove a seat: a D20 x H20 cylinder whose
cap is mated to a vertical stop re-fits upright, and its cap stays
perpendicular to the stop with the same box. Now, after the box check,
each picked stock face is placed where the mates put it. The followed
stock must hold facets that face the same way (unit normals agree within
`NORMAL_TOL` = 1e-4, about 0.8 degree) and lie on that plane (within
`LEN_TOL_MM` = 1e-4 mm). These facets must cover the face's area (less at
most `FACE_AREA_REL_TOL` = 0.1 %) and each centre of its facets. A picked
stock vertex must be a corner of the followed stock within `LEN_TOL_MM`.
A rigid carry (identical facets) passes by construction. A failure is
refused in the command with the stock face, the fixture face and a hint.
- Dropped: re-fit the stock in the turned frame so that the cap lands.
  It changes the stock-follow law for every turned part, which is a larger
  change than this issue.
- Dropped: compare the outward normal of the face with the same face id
  on the re-fitted stock. A re-fit can renumber its faces.

**Considered and dropped.**
- Move the stock alone: it detaches the part from its blank, and every
  stock-relative height and the rest-material model would then be wrong.
- Infer the body from the first pick: a hidden choice, and a part inside
  the stock is never under the cursor first. The picks pass over the parts
  while the stock is held, so a stock face round a part can be picked.
- Default the API to the stock: it would change the result of every
  existing script and project test that locates a part.

**Cost to change.** Small: `LocateBody` is one enum on `LocateSpec`; a
turned stock locate could later re-fit the stock in the turned frame
instead of the refusal.
