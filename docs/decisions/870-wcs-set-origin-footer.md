# #870: the WCS page's Set origin in its footer, and the origin said in words

- **Chosen:** *Set origin* moves out of the page body (where it sat beside *Level*, among the quarter turns) into the property pane's fixed footer, on one row with *Close*. The row is drawn by `egui_app::wcs_footer`; every other page keeps its plain *Close*.
  - Why: the user's report. The button that applies typed values was hard to find among the per-field controls. The footer stays put while the body scrolls.
  - Dropped: a second, body-level apply button kept for habit. Two buttons with one action is the clutter the report is about.
- **Chosen:** an *Origin not applied* flag in the footer, in the warning ink, while the typed fields differ from the placed WCS. The position is compared to 1e-9 mm. The angles are compared as rotations (quaternion dot product), so yaw 360 or -180/180 never reads as an edit. A non-finite field reads as pending.
  - Dropped: a confirm-on-close prompt. The drafts survive a close and reopen until the document changes (`sync_setup_ui`), so a close loses nothing silently, and the flag makes the state visible. A modal would add a click to every close.
  - Dropped: applying on close. Close then becomes a second, hidden apply, and the Prime Directive prefers a visible commit for something that moves every coordinate of the program.
- **Chosen:** the origin's place in words, from a new pure module `app::setup_origin` (`origin_words`, `words_on`). It is used by the WCS page's header line (`G54 origin: top of stock, front-left corner (X0 Y0 Z0)`) and by the setup sheet's new `origin:` line.
  - The reference box is the one the page's anchor grid uses: the stock box, else the model's bounds.
  - X is left/right, Y front/back (the operator at -Y), Z bottom/top. The 27 anchor points are named within 1 µm, the tolerance of the anchor buttons' exact writes.
  - Any other point is given as signed distances from the box's left side, front and top. The words never claim a corner the origin is not on.
  - A non-finite origin or box says "not known".
- **Viewport marker:** the WCS axis triad already marks the origin in the viewport. Nothing was added there.
- **Docs:** `docs/user-guide/work-origin.md`, a short page on where the origin is shown and how to change it.
- **Kept out of #759's way:** #759 (open, draft) splits the WCS page body into `wcs_page.rs` and has its own *Set Origin* in the quarter-turn rows. This PR touches the body in one place (the button removed) and puts its new code in its own files. Whichever lands second drops the body button and keeps the footer.
- **Left as is:** the words read the world-frame box. On a rotated or aligned setup (#326), "left" and "front" are the world's, as the anchor grid's compass names are.
