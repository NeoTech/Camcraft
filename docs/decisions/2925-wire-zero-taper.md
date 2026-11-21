## #2925 - Accept an explicit zero wire taper

Use the Wire Contour's typed taper angle when checking an operation parameter patch.
Zero selects a vertical cut and the project writer omits it from the saved fields.
Changing project serialization would change existing files when saved. Accepting all
zero fields would hide spelling errors. A later change to how a wire taper is stored
needs an update to this check and its MCP regression test.
