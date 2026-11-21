## #3213 - Remove unsupported characters from Sodick comments

**Decided:** Replace `#`, `[`, `]` and backslash with spaces in the Sodick
post's `note()` function.

**Why:** Every comment route, including program notes, passes through this
function. This keeps the existing comment lengths and diagnostic text.
The existing `clean()` function handles names and limits them to 48
characters. Applying that limit to all comments would shorten static warnings.

**Dropped:** Changing the shared sanitizer would affect other dialects.
Rejecting names or shortening all comments would add restrictions that this
fix does not need.

**Cost to change later:** Change the Sodick post and its comment golden.
