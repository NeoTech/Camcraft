# 1185 - re-import keeps the model's recorded STEP facet budget

- Chosen: a STEP import with no explicit budget (GUI session setting never
  committed, or MCP/script call without `facet_budget`) lands under the
  budget recorded with the model of the same name; a committed or passed
  budget wins and is recorded. The model page says a re-import keeps it.
- Why: the session setting resets to the default at launch, so a restart
  silently re-meshed a 60M model at 30M.
- Dropped: persisting the session setting in the project (a document change
  for a UI setting); resetting "explicit" after each import (changes the
  existing "setting persists across imports" behaviour).
