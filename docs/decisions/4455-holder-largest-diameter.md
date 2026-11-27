## #4455 - Holder form: the largest diameter, and a body that the outline does not show

**What was decided.**

1. The cause of "the form reads 50.0, the drawing shows Ø63" is a label.
   The form field `Largest diameter` held the body steel below the flange
   (`HolderShape::largest_d`: the ER32 nut Ø50 of a BT40 ER32). The drawing,
   and the read-only view of a built-in holder, state the largest diameter
   of the whole outline: the flange or the body, whichever is wider (the
   BT40 flange Ø63). The outline itself was correct in both views.
   - The editable field is now `Body diameter`.
   - The form has a read-only row `Largest diameter`. It shows the drawing's
     value (`holder_drawing::largest_d` of the drawn entry), which is 2 x
     the largest radius of the outline.
2. A second defect was under the safety floor. A shrink-fit body that the
   flange envelope covers shows nowhere in the outline. In that case
   `HolderShape::of` read back the narrowest body that draws the outline
   (the Ø38.748558 for a Ø44 body that PR #4438 observed). The copy of
   "BT40 Shrink 20 mm" read back a body of about Ø41.5, not the table's
   Ø42. When the user then made the gauge length longer, the body was
   drawn smaller than the steel.
   - `of` now returns `None` when a wider body or a narrower nose draws the
     same outline. The holder then opens in the generic component editor,
     as an outline from elsewhere does.
   - The exception is a library shrink chuck at its library gauge length.
     It reads back as the library's own nose and body: those are the values
     that the user saw in the read-only view.
   - `HolderShape::component` builds the kind and the geometry, and refuses
     (`ShapeError::NotKept`) a shape whose outline does not read back as
     the same values. The form's check and Apply use it. Thus the form
     never writes an outline that it cannot show again.

**Why.** The Prime Directive: the holder outline goes into the collision
checks. A value that the form shows must be the steel that the outline
keeps. A refusal by name is better than a quiet guess, and the outline that
a guess draws can be smaller than the real steel.

**Dropped.**

- Show the outline's largest diameter in the editable field. That field
  sets the body. The flange comes from the taper, so the value cannot be
  edited there.
- Read back the widest body that draws the outline. The set of such bodies
  has no upper limit below the flange, and the value is a guess too.
- Accept a covered body at any gauge length when it is the library's body.
  A user holder that was written before this fix, at another gauge length,
  can have another body. It now opens in the generic editor.

**Cost to change later.** Small. If a holder component gets its own
recorded shape (for example a STEP import in #4390), `of` can read that
record first and the refusal can go away for those holders.
