## #4496 - Generic component editor: a gauge length edit keeps the holder's own steel

**What was decided.**

1. A gauge length edit in the generic component editor still draws the
   built-in holder's outline at the new length, but only through
   `ComponentGeometry::regauged_holder` (`camcraft-tools`). It accepts the
   new outline only when both of these are true:
   - The holder's outline is the built-in holder's outline at the holder's
     own gauge length (the same points, and the same base).
   - The new outline is the old outline with one straight cylindrical
     section made longer or shorter, and nothing else changed. The nose
     keeps its shape from the nose face. The flange keeps its shape from
     the gauge line. The section keeps the radius that the old outline
     gives it. Points inside one cylinder (an ER nut and a body as wide
     as the nut) count as one section.
2. Each other case is refused by name, and nothing changes:
   - `NotLibrary`: a user holder with a library name but another outline
     (for example a wider body that shows).
   - `NoSection`: the outline shows no body section that the edit can
     make longer or shorter. This is a shrink chuck whose body the flange
     envelope covers. It includes the look-alike of #4455 (a Ø50 body under
     the "BT40 Shrink 20 mm" envelope) and the library's own copy of that
     chuck, because the two outlines are the same.
3. The library holders that show their body edit as before: the result is
   the library outline at the new length, point for point. A pinning test
   runs each library entry.

**Why.** The Prime Directive: the holder outline goes into the collision
checks. The old code found the library entry by its name and drew the
library body at the new length. When the real steel under the flange
envelope was wider than the library body, the new outline was smaller than
the steel. A gauge length change is a longer or shorter body. If the
outline shows that body as a cylinder, the new outline is known from the
old outline alone. If the outline does not show the body, the steel of the
new section is not known, and a refusal is better than a guess.

**Dropped.**

- Build the new outline from the holder's own outline with no library
  entry, and insert a cylinder at the widest radius below the flange when
  no body shows. This is safe, but it silently draws a holder much fatter
  than the user expects, and it changes the outline of each holder that
  edits correctly today.
- Accept the library entry when the holder's outline equals the library
  outline at the library gauge length. #4455 showed that a look-alike with
  a wider hidden body has the same outline, so this check cannot tell them
  apart.
- Check only that the new outline is not smaller than the old outline at
  each height below the gauge line. Each shorter gauge length fails that
  check (the nose moves up into a place where the old body was wider), and
  a longer gauge length with a hidden body passes it.

**Cost to change later.** Small. If a holder component gets its own
recorded shape or source (#4455 "Cost to change later"), the covered
library copies can take a gauge length edit again through that record.
