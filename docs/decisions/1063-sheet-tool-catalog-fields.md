## #1063 - setup sheet prints a tool's catalog facts

**Decided.**
- **Where.** The setup sheet module writes one format only (plain text,
  `setup_sheet/tools.rs`), so that is the only output changed.
- **Format.** A line under each tool's geometry line:
  `PART <pn>  VENDOR <v>  COATING <c>  HELIX <deg> deg`, in that fixed order.
  Each stated fact appears alone; unstated ones are left out whole (no label,
  no "None"); a tool stating none gets no line, so existing sheets and goldens
  are byte-identical. Helix uses the sheet's `mm()` number helper so no float
  prints in scientific notation.
- **Notes not printed.** The issue asked for part number, vendor, coating and
  helix; free-text notes may span lines and break the fixed-width layout.
- **Dropped.** Extra columns on the tool row (wide, mostly empty).
- Display only: no posted G-code change.
- Stacked on PR #1070 (#898), which adds the fields.
