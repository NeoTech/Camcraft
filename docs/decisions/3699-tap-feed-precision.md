## #3699 - Exact G84 tap feed in six mill posts

**Decided.** The posts `fanuc`, `doosan-dnm`, `mazak`, `sinumerik`,
`tormach` and `linuxcnc` write the G84 F word at three decimals in mm/min
and four decimals in in/min, with trailing zeros removed. Each post then
checks the pitch that the control reads (written F / written S) against
the stream's pitch (feed / rpm). A difference of more than 0.01 percent
refuses the tap by name, with the operation name. The approach and the
tolerance are the same as the Mach4 fix (#3701, PR #3731).

**Why.** The stream keeps the tap feed at 0.001 mm/min (pitch x rpm). A
whole-number F word changes the pitch that the control cuts: M1.4 x 0.3 at
S115 needs F34.5, and F35 cuts a pitch that is 1.45 percent too long. The S
word is a whole number, so a fractional rpm also changes the pitch; the
check refuses that case too.

**Decimal point.** The Fanuc-family posts (`fanuc`, `doosan-dnm`, `mazak`)
write the tap F word with its point (`F600.`), by their own length-word
rule (#1836). A point in an F word reads the same at DPI=0 and DPI=1.
`sinumerik`, `tormach` and `linuxcnc` read a decimal F as written and keep
no forced point (`F600`). The F words of the other moves do not change:
#3690 owns the decimal point of those.

**Dropped.**
- Change `ff()` for all feeds. Dropped: it changes every golden program,
  and a feed that is not a tap has no pitch to hold. #3690 decides the
  general F form.
- Round S to a decimal. Dropped: the S words of these posts are whole rpm;
  the refusal names the case.
- A shared helper. Dropped: each post is one self-contained script.

**Cost to change.** One helper and two lines in each post. The test
`crates/post/tests/tap_feed_mill_posts.rs` pins each post.
