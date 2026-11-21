## #3214 - Sodick ALC400G ships an example starter set

**Decided.** `assets/machines/sodick-alc400g.mch` states an example
starter set:

- three conditions for a 0.25 mm wire: `C110` rough (gap 0.04 mm),
  `C120` skim (gap 0.02 mm), `C130` finish (gap 0.01 mm);
- `edm.lower_guide_z` 0 and `edm.upper_guide_z` 130 mm;
- `edm.awt: true`.

Each condition's `notes` says "EXAMPLE - not a Sodick figure; replace
with the row of your control's condition table". The file's description
says the same for the whole set. The EDM guide page has the steps to
replace the examples.

**Why.** The maintainer asked that a first Wire Contour generate and post
on the built-in machine (issue "Done means"). Before, the file stated no
condition, so each Wire Contour was refused until the user wrote a
`.mch`. The ids are `C` and three digits because the Sodick post refuses
any other id. The gaps are each smaller than the 0.25 mm wire's radius,
so the reader accepts them. The guide heights put the upper guide at the
130 mm work height of the published taper figure. The Sodick post
refuses every taper, so the guide heights do not change posted output on
this machine. `awt: true` makes the post write T91 / T90, the threading
words that it already writes for a machine with a threader.

**Dropped.**

- `edm.wire_diameters`: a list would refuse a user's other wire before
  the user edits the file. With no list, any wire in range passes.
- `edm.codes`: the Sodick post does not read them.
- Tank and flushing: unstated, the post leaves them to the control, which
  is the safer default.
- Picking a default condition for a new Wire Contour: that is a command
  change, and #3183 covers conditions from the machine's own table.

**Cost to change.** Edit the `.mch` file and the pins in
`crates/app/tests/sodick_alc400g.rs`.
