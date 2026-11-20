## #2858 - Publish catalog images for each operation

**What was decided:** Capture one model image without toolpaths and one image for each operation.
Use selection commands to show one operation's toolpath at a time.
Keep the same camera for all images of a case.
Use `0-blank.png` for the model image.
Use the document order and an operation name with sanitized characters for each operation filename.
Collect every PNG in each case's output directory.
Commit the images to `catalog-screenshots` and push that branch to `origin` without force.
Link the published seed images from its README and the catalog documentation.

**Why:** Separate images let a reader inspect each operation without other toolpaths covering it.
Stable paths keep the README links valid when the command runs again without a source change.
An images-only branch keeps PNG files outside the product tree.
The user forbids pushing to `upstream`, so publication uses the fork's `origin` remote.
A normal push refuses a conflicting remote history and reports the error.

**Considered and dropped:** A combined toolpath image does not show each operation separately.
PNG files in the product tree violate the requested image storage rule.
Pushing to `upstream` violates the user's instruction.
An automatic force push could remove remote commits.
Reusing the review screenshots branch would mix catalog images with review evidence.

**Cost to change later:** A different path scheme requires updates to the renderer, collection tests, and README links.
Moving publication to another remote requires a deliberate remote choice and updated image links.
