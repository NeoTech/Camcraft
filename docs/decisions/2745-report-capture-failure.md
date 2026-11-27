## #2745 - Deliver a failed window read-back to the report status

**Decided:** Send both a picture and a failed read-back through the existing
capture completion channel. A failure retains its request tokens. The frame
consumes it before drawing the status strip. Only the report token changes
the report status. A failure sends no screenshot event or clipboard image.
Map, mapped-range and pixel-decoding failures use this same completion path.

Keep a capture busy until the frame consumes its completion, not only until
the callback finishes. Give each shot its own completion flag. After a
timeout, an old callback cannot clear a new shot or answer a new request.
Requests that arrived during a shot remain queued for the next shot.

**Why:** The old callback cleared its flag and dropped the tokens on failure.
The timeout could no longer run, and the frame could stop repainting. The
user saw a promise to copy a picture that could never arrive.

**Considered and dropped:** Do not send an empty image as a failure. That
would claim a successful copy and could replace the clipboard. Do not add a
new screenshot service, dependency or persistent fault-injection option.
Do not change the simulation preview read-back or machining policy.

**Verification:** The callback/channel regression checks the rendered status
and absence of a clipboard command. A second test cancels a real pending
wgpu map with `Buffer::unmap`. This exercises the production map-error
callback on Linux without losing a device. Tests also cover successful
multi-token delivery, a foreign token, a closed dialog, timeouts, obsolete
callbacks and a queued request after failure. A temporary rendering harness
uses the real report button, capture callback and status renderer for the
before/after evidence. It is not part of the source change.

**Cost to change later:** A different screenshot consumer can interpret its
own failed token at the existing frame-consumption seam. A change to the
report message needs the corresponding visible-status assertion.
