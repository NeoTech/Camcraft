## #3206 - wire-iso writes each dimension word with a decimal point

**Decided:** the generic ISO wire post (`assets/posts/wire-iso.cps`)
always writes the decimal point on X, Y, U, V, I, J and the taper angle
word: `X5.`, `Y0.`, `U0.`, `A3.`. Trailing zeros are still removed
(`X-20.165`, `X0.5`), and a negative zero writes `0.`. Comment text
(the taper note `Z12`) keeps its plain numbers.

**Why:** many wire controls read a number without a point in their
least input unit (0.001 mm). On such a control `X5` moves 5 um, not
5 mm. A word with a point reads the same on a control of either
convention, so the point is always safe. The Sodick post already uses
the same rule.

**Dropped:** a machine or post option that switches the point on or off.
No control we know needs the point removed, and an option adds a way to
get wrong motion. If a control that refuses `X5.` appears, add the option
then, with the point as its default.

**Cost to change later:** low. The rule is one helper (`pt`) in the post
script; the eight wire-iso goldens move by the point only.
