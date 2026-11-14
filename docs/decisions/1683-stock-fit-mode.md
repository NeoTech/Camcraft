## #1683 - Fit controls follow the stock shape

**Decided:** Hide the fit group and Fit to Models button for From model
stock. Keep the command's existing refusal for callers outside the page.
Clear Setup feedback when the draft shape changes, including its status
echo when that echo still matches. Dispatch footer actions by their labels
so removing Fit to Models leaves Delete attached to deletion.

**Why:** From model stock follows its source and cannot use fit margins.
A refusal about the previous shape no longer describes the draft. Box
and Cylinder retain their fit controls, and Apply and Delete remain
available for source stock.

**Dropped:** Disabled fit controls with a hover explanation would keep
unusable controls on the page. Removing the command refusal would weaken
validation. Dispatching by fixed button positions would make Delete invoke
Fit after the middle button disappears.

**Cost to change later:** Showing disabled controls instead requires a
page-only change and structural pin updates. Stock fitting and posted
output remain unchanged.
