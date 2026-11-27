## #2324 - Pin default-machine recovery through disk reads

**Decided:** Add a disk-backed test to the existing `default_machine_missing`
suite. Copy the test executable beside a temporary `assets/machines` folder.
Run only the recovery test in that child process. Call
`default_machine_checked` after a missing file, an empty object, and the good
shipped file. Require `Err`, `Err`, `Ok`. Then break the file again and require
the good parsed machine to remain available.

**Why:** The injected-reader test does not exercise `default_mch_text`.
A fresh process isolates the directory and machine `OnceLock` state. The test
changes no shipped asset and sets no environment variable in the parent.
Production behavior already satisfies the contract. A temporary text-cache
mutation must fail the new test; the submitted diff contains no mutation.

**Dropped:** Another injected reader would not cover the disk path. A new
integration binary would increase build cost. A production change is not needed.

**Cost to change later:** Keep the child test filter and executable layout in
step with the test name and the documented data-directory lookup.
