"""Provenance: user-visible changes merged after v0.27.0, listed in #4439."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
GUIDE = ROOT / "site" / "content" / "guide"
USER_GUIDE = ROOT / "docs" / "user-guide"


def section(page, heading, base=GUIDE):
    text = (base / page).read_text()
    marker = f"## {heading}\n"
    assert marker in text, f"Missing guide section: {heading} in {page}"
    body = text.split(marker, 1)[1].split("\n## ", 1)[0]
    return re.sub(r"\s+", " ", body)


class Release028Guide(unittest.TestCase):
    def facts(self, page, heading, *facts, base=GUIDE):
        text = section(page, heading, base)
        for fact in facts:
            with self.subTest(page=page, heading=heading, fact=fact):
                self.assertIn(fact, text)

    # Tool library window (#4365, #4381, #4377, #4103, #3976, #3963).
    def test_tool_library_window(self):
        self.facts("set-up/tools.md", "Create and edit a tool",
                   "one header", "**Cutting tools**", "**Holders**",
                   "one footer", "**Duplicate**", "**Cancel**",
                   "**Search tools**", "N of M tools", "**in document**",
                   "part number and vendor", "helix angle", "stick-out",
                   "clamping range", "to scale", "**My library**",
                   "**Left-hand cutter (spindle reverse, M4)**",
                   "cannot reverse the spindle", "reserved pockets",
                   "outside the spindle range")

    # Recipes tab (#4385) and copper (#4188).
    def test_recipes_tab_and_copper(self):
        self.facts("set-up/tools.md", "Recipes tab",
                   "**Recipes** tab", "star", "surface speed", "chip load",
                   "**Add material…**", "**Suggest feeds and speeds**",
                   "**Copy recipes from tool…**")
        self.facts("set-up/tools.md", "Cutting data for wood and plastics",
                   "**Copper (C11000)**")

    # Holder library and the Holders page (#4240, #4251, #4276, #4280,
    # #4402, #4271).
    def test_holder_library_and_holders_page(self):
        self.facts("set-up/tools.md", "Holders and profiles",
                   "HSK-A63", "HSK-F63", "HSK-E40", "HSK-E32", "SK30",
                   "SK40", "SK50", "ISO30", "BT50", "CAT50",
                   "DIN 69882-8", "<taper> Shrink", "ER50", "ER11", "ER40",
                   "**Holders** page", "**Search taper, clamp, size or name**",
                   "**Copy as user holder**", "spindle nose",
                   "one undo step", "refuses an edit")

    # Tool components and stacks (#4201, #4233, #4271, #4280, #4249,
    # #4278, #4331, #4194, #4096).
    def test_tool_components_and_stacks(self):
        self.facts("set-up/tools.md", "Tool components and stacks",
                   "component stack", "**Component stack**", "Stale",
                   "components.camcomponents", "**Sync from library**",
                   "never applies", "**Show compatible parts**",
                   "**Apply stack**", "**Detach to standalone tool**",
                   "**Swap**", "indexable", "shared insert", "nose radius",
                   "nose-centre", "orientation (1 to 8)", "ISO 1832",
                   "ISO 5608", "list_tool_components")

    # .camtools version 8 (#4261) and the left-hand cutter file step.
    def test_camtools_version_8(self):
        self.facts("set-up/tools.md", "Import and export",
                   "Version 8", "tool components", "same id",
                   "version 7")

    # Machine file keys and the machine editor (#4414, #4206, #4164, #4184,
    # #4251, #4114, #3922, #3985).
    def test_machine_file_keys(self):
        self.facts("check-and-post/machines.md", "Mill",
                   "travel limits", "rigid tapping",
                   "`controller.work_offset_max`",
                   "`tooling.max_tool_number`", "`tooling.d_number: \"tool-number\"`",
                   "`tooling.spindle_mount`", "`manual-retract-top`",
                   "adds, moves and removes", "reverses a linear axis",
                   "spindle power field", "`tooling.tool_change`")

    # Laser machine kind (#4270).
    def test_laser_kind(self):
        self.facts("check-and-post/machines.md", "Laser",
                   "`laser`", "laser head", "no laser operation")

    # Wire offset on the LN2W post (#4139, #4246, #4070).
    def test_wire_offsets(self):
        self.facts("check-and-post/machines.md", "Wire and sinker EDM",
                   "G41 / G42 Hnnn", "G40", "H001", "shorter than",
                   "LN1W/LN2W", "T91")
        self.facts("operations/operations-edm.md", "Wire Contour",
                   "computer", "control or wear",
                   "wire-centre path", "lower guide", "-0.000")
        # Closing overlap, centre-line slit and open-run skims (#3617).
        self.facts("operations/operations-edm.md", "Wire Contour",
                   "**Closing overlap**", "witness mark", "**Centre**",
                   "no side is kept", "Skims also work on open runs")

    # Milling post controller details (#4382, #4373, #4367, #4363, #4185,
    # #4184, #4169, #4166, #4289, #4195, #4163, #4162, #4196, #4198, #4170,
    # #4299).
    def test_milling_post_details(self):
        self.facts("check-and-post/posting.md", "Post processors",
                   "G17 and G94", "O0001", "O9999", "G49 before",
                   "G5.1 Q0", "M89", "M84", "M74", "`T12 D1 M6`",
                   "`G0 G53 Z.. D0`", "plane axes first", "G59 P7",
                   "G85", "`P1.`", "G92", "G97", "PYRELOAD", "G64 P0",
                   "G54.1 P1", "G74", "left-hand cutter")

    # Heidenhain posts (#4387, #4242, #4172, #4158, #4114, #4248, #4395,
    # #4161, #4164).
    def test_heidenhain_posts(self):
        self.facts("check-and-post/posting.md",
                   "Heidenhain DIN/ISO and Klartext",
                   "G62", "P01", "PECK", "`G205`", "Q395", "G04 F",
                   "M92", "dialect table", "G247", "cycle 200",
                   "cycle 207", "cycle 206", "cycle 9", "MP 7431", "254")

    # 2D operations (#4136, #4416, #3832, #4008, #4014).
    def test_operations_2d(self):
        self.facts("operations/operations-2d.md", "2D Contour",
                   "**Do not touch**", "leads, links and plunges")
        self.facts("operations/operations-2d.md", "2D Chamfer",
                   "circular interpolation", "closed")
        self.facts("operations/operations-2d.md", "Bore",
                   "**Helix**", "approach ramp and helix angles", "0.025 mm")
        self.facts("operations/operations-2d.md", "2D Adaptive",
                   "clean-up", "engagement limit", "advisory")
        self.facts("operations/operations-2d.md", "2D Offset",
                   "two lobes merge")

    # DXF import (#4052, #3968).
    def test_dxf_import(self):
        self.facts("set-up/setups.md", "Place a DXF profile",
                   "INSERT", "8 levels", "1 µm", "MPOLYGON")

    # 3D operations (#4199, #4210, #4217, #4191, #4223, #4168).
    def test_operations_3d(self):
        self.facts("operations/operations-3d.md", "3D Pocket",
                   "start point", "exit point", "refused by name",
                   "short spiral", "background job")
        self.facts("operations/operations-3d.md", "3D Finish",
                   "start point", "exit point", "ends on a wall")
        self.facts("operations/operations-3d.md", "Deburr",
                   "start point", "exit point", "one chain")

    # Multi-axis operations (#4084, #4174, #4018, #4321, #4074).
    def test_operations_multiaxis(self):
        self.facts("multi-axis/operations-multiaxis.md", "5-Axis Parallel",
                   "start point", "exit point", "raster corner")
        self.facts("multi-axis/operations-multiaxis.md", "Rotary Flow",
                   "any two guide", "crossing point", "Three or more",
                   "shared", "single", "backwards")
        self.facts("multi-axis/operations-multiaxis.md", "Swarf",
                   "exit rise", "contact normal")
        self.facts("multi-axis/operations-multiaxis.md", "Rotary Contour",
                   "upright Z", "horizontal X")

    # Multi-axis checks (#4296, #4098, #4277).
    def test_multiaxis_checks(self):
        self.facts("multi-axis/checks.md",
                   "Machine check: reach and travel per axis",
                   "names the original move")
        self.facts("multi-axis/checks.md",
                   "Shank, holder and fixture collision",
                   "equator", "spindle housing strikes a fixture")
        self.facts("multi-axis/checks.md", "Playback on a work-turning rotary",
                   "work-turning", "upright", "rotary axis line")

    # Plasma picks (#3986).
    def test_plasma_picks(self):
        self.facts("operations/operations-plasma.md", "Plasma Profile",
                   "start point", "exit point", "pierce", "torch off")

    # Stock simulation (#4267, #4183, #4117, #4105, #4013, #4140, #4237,
    # #4108, #3987).
    def test_stock_simulation(self):
        self.facts("fundamentals/proving-out.md", "Stock simulation in CamCraft",
                   "in pieces", "wide tool", "marching tetrahedra",
                   "four or more facets", "closed solid", "whole number of cells",
                   "vertical", "finished part",
                   "The stock is hidden, so the material removal is not shown.")

    # WCS flip refusal (#4314).
    def test_wcs_flip_refusal(self):
        self.facts("set-up/setups.md", "Work offset",
                   "flip", "refusal", "mirrored")

    # SolidCraft face names (#4364, #4366).
    def test_solidcraft_face_names(self):
        self.facts("solidcraft-camcraft-mcp.md", "Face picks follow SolidCraft names",
                   "model cache format 10", "re-import", "review line",
                   "`picks`", "`source_faces`", base=USER_GUIDE)

    # Refusal wording, stale caches, accessibility, chat (#4187, #4286,
    # #4232, #4221, #4216, #4301, #4319).
    def test_refusals_and_ui(self):
        self.facts("reference/refusals.md", "Geometry and generation",
                   "stepdown must be greater than 0 (got -3)")
        self.facts("reference/troubleshooting.md",
                   "Operations go stale after an update",
                   "generator", "stale once")
        self.facts("getting-started/keyboard-and-mouse.md", "Screen readers and field order",
                   "accessible name", "Tab", "Enter", "1280 x 720")
        self.facts("automate/ai.md", "Ollama and offline-only mode",
                   "rename", "edit a tool", "work material")
        self.facts("reference/troubleshooting.md", "Report a problem or request a feature",
                   "picture", "text report")


if __name__ == "__main__":
    unittest.main()
