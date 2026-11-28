"""Provenance: merged CamCraft PRs listed in #3909, guide batch 3."""

from pathlib import Path
import re
import unittest


GUIDE = Path(__file__).resolve().parents[1] / "content" / "guide"


def section(page, heading):
    text = (GUIDE / page).read_text()
    marker = f"## {heading}\n"
    assert marker in text, f"Missing guide section: {heading}"
    body = text.split(marker, 1)[1].split("\n## ", 1)[0]
    return re.sub(r"\s+", " ", body)


class Release027Batch3(unittest.TestCase):
    def facts(self, text, *facts):
        for fact in facts:
            with self.subTest(fact=fact):
                self.assertIn(fact, text)

    def test_3833_outside_contour_entry_names_limits(self):
        text = section("operations/operations-2d.md", "2D Contour")
        self.facts(text, "outside", "lead-in and ring", "each level",
                   "Helix", "tabs", "too short", "compensation")

    def test_3861_cli_ui_targets_existing_history(self):
        text = section("automate/cli.md", "Control a running window")
        self.facts(text, "--ui windows", "--window", "call", "mcp",
                   "Ctrl+Z", "exit 3", "exit 2", "exit 1", "exit 4",
                   "never falls back", "CAMCRAFT_UI_LINK=off")

    def test_3900_library_is_a_window_not_a_dock_tab(self):
        text = section("set-up/tools.md", "Create and edit a tool")
        self.facts(text, "**Tool library** window", "**Document**", "**Catalog**",
                   "dock tab is gone", "Properties")
        first_run = (GUIDE / "getting-started/first-run.md").read_text()
        self.assertNotIn("**Tool library** pane", first_run)

    def test_3903_editor_keeps_separate_drafts(self):
        text = section("set-up/tools.md", "Create and edit a tool")
        self.facts(text, "**Tool** tab", "**Details** tab", "**All tools**",
                   "own draft", "one undo step", "part number")

    def test_3496_rotary_zigzag_closes_walls_without_double_seam(self):
        text = section("multi-axis/operations-multiaxis.md", "Rotary Offset")
        self.facts(text, "**Zigzag**", "closing ring", "seam", "cusps",
                   "twice")

    def test_3516_pocket_protection_covers_full_assembly(self):
        text = section("operations/operations-2d.md", "2D Pocket")
        self.facts(text, "**Do not touch**", "**Pick protected**", "**Clearance**",
                   "flutes, shank and holder", "rest machining", "refused")

    def test_3579_missed_relief_names_boundary_corner(self):
        text = section("operations/operations-2d.md", "2D Pocket")
        self.facts(text, "corner relief at (x, y) is not cut: no pass vertex reaches the relief spur",
                   "boundary corner", "once", "material remains")

    def test_3762_compute_fallback_is_not_window_recovery(self):
        text = section("reference/troubleshooting.md", "GPU compute falls back to the CPU")
        self.facts(text, "uncaptured GPU error", "GPU compute device lost",
                   "first", "rest of the process", "CAMCRAFT_GPU_COMPUTE=off",
                   "window", "graphics driver")


if __name__ == "__main__":
    unittest.main()
