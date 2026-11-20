## #2651 - Refresh the continuous 4-axis example project

**What was decided:** Regenerate the example project with the existing test.
Keep the posted NC program unchanged.
Add a direct test that compares the committed version with `PROJECT_VERSION`.

**Why:** The DXF import change raised the project format to version 71.
The example still used version 70, so the existing comparison failed.
The new test identifies this mismatch without generating a toolpath.
Regeneration changes only the version field.

**What was dropped:** Do not relax the project comparison or change the serializer.
The comparison correctly detected a stale fixture.
Do not stack on #2563. The maintainer explicitly accepted this overlap on #2651.
That PR must retain the current version when it refreshes.

**Cost to change later:** A future project format change must regenerate this example.
The version test reads `PROJECT_VERSION`, so it needs no version literal update.
