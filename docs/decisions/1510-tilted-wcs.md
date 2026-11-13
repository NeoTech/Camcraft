## #1510 - A milling WCS tilted off the machine's Z is refused at generation

**Decided:** a milling operation whose setup's WCS has its Z axis tilted
off the machine's +Z (more than 1e-9 rad, the post's identity tolerance)
is refused by name at the generation door (`collect_job`), and a facing
pick on a face the tilted WCS calls level says the WCS is why. The
refusal tells the user to turn the part with the setup's Part alignment
(#326) and keep the WCS Z up. A WCS turned only about Z still generates;
its post refusal stays #315's.

**Why:** every milling generator cuts along the machine's Z. A tilted WCS
was drawn, but ignored by generation: the reporter's facing machined the
face that lies level in the model, not the one the WCS showed, and the
op reported OK. The post already refuses any rotated offset (#315), so no
wrong G-code could post - but a toolpath shown as OK on the wrong face
is the "silent wrong answer" the Prime Directive forbids; refusing at
the door makes it visible where the user works.

**Considered and dropped:** generating in the WCS frame (the WCS as a
3+2 plane, or as an implicit part alignment) - new behaviour across the
generators, the post and the sim (effort:max), and it would duplicate
Part alignment, which already turns the part for a setup. A note instead
of a refusal - an OK status with a note is still easy to miss. Refusing
yaw-only WCS rotations too - they keep the tool axis; #315 already holds
them at the post.

**Scope:** milling machines only (`Kind::Mill`); lathe, wire and other
kinds read their own frames and are untouched.

**Cost to change later:** one module (`crates/app/src/setup_wcs_tilt.rs`);
carrying a tilted WCS for real replaces its refusal with the transform.
