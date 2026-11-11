## #790 - Vendor vise STEP files: user-guide page and import check

- **What was decided:** the guide lives at `docs/user-guide/vendor-fixtures.md`
  (no user-guide folder existed on main; the docs site in PR #777 is still
  open). Vendor names and dimensions are kept out of the repo and the issue:
  the check results use generic names and report match or mismatch only.
- **Why:** the user asked for local test inputs that are never committed or
  copied. The guide's labels are the ones the app shows, taken from the UI
  sources, so it does not describe anything the app does not do.
- **Considered and dropped:** a separate docs site page (waits for #777);
  listing the #819 plate grid as available (it is not, so it is marked as
  planned); describing the Level button's behaviour (not verified, so only
  named).
- **Cost to change later:** low. Moving the page into the docs site is a
  file move plus a link when #777 merges.
