# 4120 - disk_recovery test builds the machines folder one component at a time

- **Chosen:** in `crates/app/tests/default_machine_missing/disk_recovery.rs`,
  build the machines folder with `.join("assets").join("machines")` in the
  parent test and in `check_disk_recovery`.
- **Why:** `join("assets/machines")` keeps the `/` on Windows. The reader
  prints `...\assets\machines`, so the `contains` check on the path failed
  on Windows only. The reader is correct; only the test was wrong.
- **Dropped:** a change to the reader or to its message (not needed, and the
  issue says not to change it). A normalised string compare in the test
  (it would hide a reader that prints a different folder). The
  `../../assets/machines/3axis-vmc.mch` source path on line 72 stays: it is
  only opened, never compared as text, and Windows accepts `/` there.
