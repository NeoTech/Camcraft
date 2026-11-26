+++
title = "Model to Machine"
description = "How a part goes from a 3D model to a moving machine: CAD, CAM, post, control and machine, and where the machine file (.mch) and the post (.cps) fit."
weight = 0
+++

This page answers the questions that new users ask first. What is a post?
How is it different from the machine file? Which post fits my control?
Which file do I bring from my CAD program? Where does 3D printing fit?
Each section links to the page with the detail.

## The five steps

A part passes through five steps. Each step hands one thing to the next.

| Step | What it does | Examples | Hands on |
|---|---|---|---|
| 1. **CAD** | Draws the shape. It knows nothing about tools or machines. | SolidCraft, Fusion, SolidWorks, Onshape, FreeCAD | STEP, `.sc`, DXF (2D) |
| 2. **CAM** | Sets stock, workholding, tools and operations. Generates the toolpath, then checks it for gouges and collisions. | CamCraft | CL data (inside the project), saved as `.camproj` |
| 3. **Post** | Turns the neutral toolpath into the G-code dialect of one control: words, number format, header, tool change, coolant. | The `.cps` posts that CamCraft ships | G-code (`.nc` and other extensions) |
| 4. **Control** | Reads the G-code line by line and drives the motors. Each brand speaks its own dialect and refuses what it does not know. | PathPilot, Mach3 and Mach4, LinuxCNC, grblHAL, Fanuc, Haas, Heidenhain, Siemens, Sodick | The running program, with the operator's offsets |
| 5. **Machine** | Axes, spindle or wire, tool changer, rotary table. Its size and limits decide what is possible. | Tormach, Sienci AltMill, Haas | Chips (or sparks) |

CamCraft covers steps 2 and 3. A CAD program covers step 1. Steps 4 and 5
belong to the shop: the control comes with the machine.

The post is a translator and nothing more. If the toolpath is wrong, the post
writes wrong code faithfully. This is why CamCraft checks the program
before it posts. See [Posting](@/guide/check-and-post/posting.md) and
[Post and machine quirks](@/guide/fundamentals/post-and-machine.md).

## Conversational: the shortcut at the machine

**Conversational programming** skips steps 1 to 3. At the control the
operator fills in a form for a simple job (face a block, pocket a
rectangle, drill a bolt circle, cut a thread) and the control writes the
G-code itself. There is no CAD model, no CAM and no post.

- **Good for:** quick one-offs and simple shapes. It is fast at the
  machine, and no computer is needed.
- **Use CAD and CAM instead for:** real 3D shapes, many operations, repeat
  parts, and anything you want to simulate and check for gouges before you
  cut.

Examples of conversational controls are PathPilot conversational (Tormach),
Hurco WinMax, Mazak Mazatrol, Haas VPS and the conversational side of a
Sodick wire control. Many shops use both: conversational for quick work,
CAM for the rest. CamCraft does not replace a conversational control.

## Kinds of CNC machines

The five steps are the same for every kind of machine. What changes is how
the machine removes material, and so which operations, machine file and post
you need. [Machine types](@/guide/check-and-post/machines.md) lists what
each kind allows.

| Kind | Removes material by | Example machines | In CamCraft |
|---|---|---|---|
| Mill (3-axis) | a spinning cutter, the part fixed | Tormach 770MX and 1100MX, Haas VF-2, Makera Carvera, LinuxCNC mills | Yes |
| Mill with a 4th or 5th axis | a cutter plus a rotary table or tilting head | Tormach with a microARC 4 | Yes: 3+1, 4-axis and 5-axis (see [Multi-axis](@/guide/multi-axis/_index.md)) |
| Router | a fast spindle over a large flat table | Sienci AltMill, Tormach 24R | Yes |
| Lathe | a fixed tool, the part spinning | Tormach 8L, 15L, 15LX | Yes: turning (see [Turning](@/guide/operations/operations-turning.md)) |
| Plasma | an electric arc through sheet metal | Tormach 1300PL | Yes (see [Plasma](@/guide/operations/operations-plasma.md)) |
| Waterjet | a high-pressure water and abrasive jet | Waterjet machine files you write | The Waterjet Profile operation and the `waterjet-iso.cps` post. No waterjet machine file ships. See [Waterjet](@/guide/operations/operations-waterjet.md). |
| Wire EDM | sparks from a thin wire, the part in water | Sodick ALC400G, Agie Classic Cut 2S | Yes: Wire Contour. The wire posts are listed under [Posting](@/guide/check-and-post/posting.md#post-processors). See [EDM](@/guide/operations/operations-edm.md). |
| Sinker EDM | sparks from a shaped electrode | Any sinker with ISO-style words | A generic `sinker-edm.mch` template and the `sinker-iso.cps` post. You fill in its EDM codes. |

## 3D printing is a different path

To **print** a part instead of cutting it, the chain is
**CAD, then STL (or 3MF), then a slicer, then the printer**. The slicer cuts
the model into layers and writes the printer's G-code. This is **outside
CamCraft's scope**. CamCraft makes toolpaths for cutting, not layers for
printing.

- **Open-source slicers:** PrusaSlicer, OrcaSlicer, UltiMaker Cura,
  Bambu Studio, SuperSlicer.
- **Commercial slicers:** Simplify3D. For resin printers: Lychee Slicer and
  Chitubox.

Most printer makers point you to one of these. Use the one that has a
profile for your printer.

## Design files: native and neutral

Each CAD program saves its own **native** file. It holds the full feature
history (sketches, extrudes and fillets that you can still edit). A
**neutral** file such as STEP holds only the final shape, so any program can
read it.

| File | From | Kind | Opens in CamCraft? |
|---|---|---|---|
| `.sc`, `.solidcraft` | SolidCraft | Native | Yes, directly |
| `.step`, `.stp` | Any CAD program (export) | Neutral, shape only | Yes, directly |
| `.f3d` | Fusion | Native | No. CamCraft refuses it by name. Export STEP, or convert it with Dutchman. |
| `.sldprt`, `.sldasm` | SolidWorks | Native | No. Export STEP, or convert it with Dutchman. |
| `.ipt`, `.iam`, `.CATPart`, `.prt` | Inventor, CATIA, NX, Creo | Native | No. Export STEP, or convert it with Dutchman. |
| `.stl`, `.obj` | 3D printing, scans | Triangles only | Yes, but there are no true curves or faces. See [First run](@/guide/getting-started/first-run.md). |
| `.dxf` | 2D drawings | 2D profiles | Yes, as a profile on the stock. See [Place a DXF profile](@/guide/set-up/setups.md#place-a-dxf-profile). |
| `.svg` | Inkscape, Illustrator, logos, signs | 2D vector art | Yes, as a profile on the stock, through the MCP and script `import_svg` commands. The desktop window has no SVG menu item yet. |

There are two routes from someone else's CAD program:

- **Export STEP.** It keeps the shape only, and it works everywhere.
- **Convert with Dutchman.** It keeps editable history in SolidCraft.

**Dutchman** is a separate converter from native CAD files to SolidCraft
designs. It reads native files from several CAD programs and writes a
SolidCraft design (`.sc`). It keeps editable features where they map, and it
keeps the final shape where they do not. Then SolidCraft edits the design
and CamCraft machines it. Dutchman is in early development: check its
result for what came across. It is not part of CamCraft.

- [dutchmancad.xyz](https://dutchmancad.xyz/)
- [github.com/dutchman-cad/dutchman](https://github.com/dutchman-cad/dutchman)

## Who covers which steps

| Suite | CAD | CAM | Post | Control | Machine | Note |
|---|---|---|---|---|---|---|
| SolidCraft and CamCraft | Yes | Yes | Yes | | | Two apps. CamCraft opens `.sc` and STEP directly and ships its own posts. |
| Fusion | Yes | Yes | Yes | | | One app for design, CAM and posts. |
| SolidWorks and a CAM add-in | Yes | Yes | Yes | | | CAD from SolidWorks. CAM and posts from the add-in. |
| Tormach and PathPilot | | | | Yes | Yes | The machine maker ships the control. PathPilot is based on LinuxCNC. |
| Mach3 or Mach4 | | | | Yes | | A PC control program for many hobby and retrofit machines. It needs a motion board. |

## Machine file and post: what each one answers

Both files describe the machine side, but for different readers. The
**machine file** is for CamCraft. The **post** is for the control.

**Machine file (`.mch`): "What can this machine physically do?"**

- Axis travels and limits (X, Y, Z, rotary A and B, wire U and V).
- Kinematics: where a rotary axis sits and which way it turns.
- Spindle speed range and power, and feed limits.
- Tool changer size.
- For wire EDM: its EDM codes and wire data.
- Which post CamCraft offers for it, when the file states one.

CamCraft uses it to plan and to check. It refuses a path that the
machine cannot reach. **Show work envelope** draws the machine travel as an
outline in the view. An example is `assets/machines/tormach-770mx-microarc4.mch`. See
[Machine types](@/guide/check-and-post/machines.md).

**Post (`.cps`): "How does this control want it written?"**

- G, M and T words, and which ones the control accepts.
- Number format (decimal point, digits, inch or mm).
- Program header, tool change, coolant and end of program.
- Arcs (I/J or R), canned cycles and work offsets.
- Dialect details, such as Heidenhain Klartext against DIN/ISO.

An example is `assets/posts/tormach.cps`. See
[Posting](@/guide/check-and-post/posting.md#post-processors).

One post can serve many machines. Several machine files share the Tormach
PathPilot post, for example. One machine needs the post that its control
speaks. A machine file alone does not mean that a matching post exists.

## Tools: the third thing the post needs

Besides the part and the machine, CAM needs your **cutters**: diameter,
flutes, corner radius, length and holder. The control has its own **tool
table**: pocket numbers, and the lengths and diameters that it measured. The
tool **number** ties the two together. The post writes `T5 M6`, and the
control looks up tool 5.

| In CamCraft | Import and export |
|---|---|
| A built-in **catalog** of common tools to pick from | `.camtools`: CamCraft's own library, with holders |
| Your **library**: save tools and reuse them across projects | `.csv`: the Tormach PathPilot tool table |
| **Holders**: pick from the holder library, or draw a holder profile | `.TCM`: the Hurco WinMax tool library |
| **Tool numbers** that you set to match your machine's pockets | |

Importing a controller table brings its tool numbers along, so posted
programs call the right tools. See [Tools and tool libraries](@/guide/set-up/tools.md)
for the steps, the import modes and the limits.

## What goes in and out of CamCraft

| | Imports | Exports |
|---|---|---|
| **Part model** | STEP, `.sc`, STL, OBJ | Stays in the project |
| **Profiles** | DXF, SVG | Stays in the project |
| **Tools** | `.camtools`, PathPilot `.csv`, Hurco `.TCM`, tool body models | `.camtools`, PathPilot `.csv`, Hurco `.TCM` |
| **Machine** | `.mch` machine files | Edited and saved in the project |
| **Fixtures** | Vise and fixture models. See [Workholding](@/guide/set-up/workholding.md). | |
| **Program** | | G-code through the chosen post, and setup sheets |
| **Project** | `.camproj` | `.camproj` (everything above in one file) |

## Machine and post pairs

These machine files and posts ship with CamCraft. A machine file is a
starting point: check its travel, tools and clearance against your machine.
[Posting](@/guide/check-and-post/posting.md#post-processors) has the full list.

| Machine | Control | Machine file | Post | State |
|---|---|---|---|---|
| Tormach 770MX with microARC 4 | PathPilot | `tormach-770mx-microarc4.mch` | `tormach.cps` | Ships |
| Tormach 1300PL plasma | PathPilot | `tormach-1300pl.mch` | `tormach-plasma.cps` | Ships |
| Sienci AltMill 2x4, 4x4 and 4x8 | grblHAL | `sienci-altmill-4x4.mch` and the other two sizes | `grblhal.cps` | Ships |
| Haas mills | Haas | `haas-vf2.mch` and others | `haas.cps` | Ships |
| Mill with a Heidenhain control | Heidenhain TNC (DIN/ISO) | None dedicated | `heidenhain.cps` | Ships |
| Fehlmann Picomax 55 | iTNC 530 (Klartext) | `fehlmann-picomax55.mch` | `heidenhain-klartext.cps` | Ships |
| LinuxCNC mill | LinuxCNC | `linuxcnc-3axis.mch` | `linuxcnc.cps` | Ships |
| Mach3 mill | Mach3 | Mach3-era templates: `tormach-pcnc770.mch`, `tormach-pcnc770-s3.mch`, `tormach-pcnc1100-s1.mch`, `tormach-pcnc1100-s2.mch`, `tormach-pcnc1100-s3.mch` | `mach3.cps` | Ships. Select the post manually and check the installed control. |
| Mach4 mill | Mach4 | None dedicated | `mach4.cps` | Ships |
| Sodick ALC400G wire EDM | **Not confirmed.** The post is written for Mark 21 / EX21. | `sodick-alc400g.mch` | `sodick-wire.cps` | Ships. Select the post manually. The ALC400G control has not been confirmed to match the Mark 21 post. |
| Sodick LN2W wire EDM | LN2W | None | None | **Not shipped** ([#4045](https://github.com/NeoTech/Camcraft/issues/4045)) |

## Which post do I pick?

1. **Find your control.** Look at the control's start screen or nameplate for
   the make and model: for example Siemens 840D sl, Fanuc 0i-MF, PathPilot,
   Mach4, grblHAL or Sodick LN2W. The brand of the machine is not enough:
   one router model can ship with different controls.
2. **Use an exact match.** If CamCraft has a post for that control, use it.
   Use the matching machine file when there is one.
3. **Else pick the closest dialect.** Most Fanuc-style mills read the Fanuc
   post. LinuxCNC and PathPilot are close relatives. Siemens and Heidenhain
   have their own posts.
4. **Air cut first.** The first time you try a post, cut air (spindle off,
   Z raised). Note every line that the control rejects or warns about.
5. **Still no fit?** Ask on the [CamCraft Discord](https://discord.gg/bBWjkAgX5)
   or open an issue. Give the control model and its messages. Describe what
   the control wants. Do not send programs made by other CAM software.

## At the machine

The program is written. Three things happen at the control before you cut
metal.

1. **Load the program.** Use a USB stick, a network share or DNC. Some
   machines take an SD card. PathPilot, Mach4 and LinuxCNC open the file
   directly. Industrial controls can need a program number or a set file
   name.
2. **Set the work offset and the tool lengths.**
   - The **work offset** (`G54` and the others) says where part zero is.
     Find it with an edge finder or a probe.
   - The **tool lengths** are touched off, or measured with a tool setter,
     and stored in the control's tool table under each tool number.
   - The CamCraft setup sheet says which zero and which tools the program
     expects. See [Setup sheets](@/guide/set-up/setups.md#setup-sheets).
3. **Prove it.** Run an **air cut** above the part (a dry run). Then run in
   **single block** with the **feed override** turned down for the first
   moves, and keep a hand near feed hold. Check the first plunge and every
   tool change before you let the program run. See
   [Proving out](@/guide/fundamentals/proving-out.md).

## How you can help CamCraft support your machine

Most new machines and posts come from owners who tell us what their control
does. Real machine checks are the best evidence we can get.

**Please share:**

- **Your machine and control:** make, model, control model and software
  version, from the start screen or the nameplate.
- **The maker's manuals:** programming or operation manuals and G, M and T
  code lists from the machine or control maker, as a PDF or a link.
- **What your control accepts**, in your own words. For example: "it alarms
  on G21", "it wants Z0 before threading", "arcs need R, not I/J".
- **Exact alarm messages**, and the line that caused each one.
- **Air-cut and dry-run results** of CamCraft programs that we send you.
- **Your machine's numbers:** travels, spindle range, tool changer size,
  rotary layout, guide heights.

**Please do not share:**

- Programs made by other CAM software.
- Post processor files from other CAM products.
- Screenshots or documents of other CAM software.
- Anything under an NDA, or files that you may not pass on.

CamCraft is built clean-room. It can learn from the machine maker's
documents and from how your machine behaves. It never learns from another
CAM product's code or output.

Send it to the [CamCraft Discord](https://discord.gg/bBWjkAgX5), or open an
issue on [NeoTech/Camcraft](https://github.com/NeoTech/Camcraft/issues).

## Words you will hear

These terms are also in the [glossary](@/guide/reference/glossary.md).

| Term | Means |
|---|---|
| **STEP** (`.step`, `.stp`) | The neutral 3D model format that most CAD and CAM programs read. |
| **Native file** (`.f3d`, `.sldprt`) | A CAD program's own file, with its editable feature history. |
| **Dutchman** | A converter from native CAD files to SolidCraft designs. |
| **`.sc`** | A SolidCraft design file. CamCraft opens it directly. |
| **CL data** | Cutter location: the neutral list of tool positions that CAM makes before any post. |
| **G-code** | The text program that the control runs, one move or command per line. |
| **Dialect** | One control's flavour of G-code. Two controls can disagree on the same word. |
| **Work offset** (`G54` and others) | Where part zero is on the table, set by the operator at the machine. |
| **Tool offset** (`H`, `D`) | A tool's measured length and radius, stored in the control under its tool number. |
| **Pocket or tool number** | The `T` number that the program calls. It must match the tool loaded at that place on the machine. |
| **Single block** | The control runs one line, then waits. It is the safe way to step through a new program. |
| **Dry run or air cut** | Running the program without cutting, to find alarms and wrong moves safely. |
