# HEIDENHAIN TNC dialects by NC software version

Research notes for the CamCraft HEIDENHAIN posts (`assets/posts/heidenhain.cps` for DIN/ISO, `assets/posts/heidenhain-klartext.cps` for Klartext). The notes tell which words, cycles and cycle parameters each HEIDENHAIN control accepts, and from which NC software version. They do not change code.

Collected in November 2026.

## Files

| File | Content |
|---|---|
| [`dialects.md`](dialects.md) | Proposal: `controller.identity` values, capability flags, profiles (control + NC software → flags), what each post must branch on, what the posts assume today |
| [`differences.md`](differences.md) | A: the common language of all controls. B: what the oldest manual of each series adds. B2: post-relevant features with "introduced / changed / removed" and references. B3: Q-parameter lists of cycles 200, 203, 205, 207 and 209 by version |
| [`differences-by-version.md`](differences-by-version.md) | Section C: per control, each NC software version in release order, with the entries of its new-function list (generated, short pointers only) |
| [`cycles.md`](cycles.md) | Each cycle number in these notes: HEIDENHAIN name, purpose, DIN/ISO code, software option, versions, reference |
| [`inventory.md`](inventory.md) | Each manual used: control, NC software, title, HEIDENHAIN ID, edition date, pages, download URL; the overview documents; dead links |
| [`picomax-55-review.md`](picomax-55-review.md) | Review of the two HEIDENHAIN posts and the machine file `fehlmann-picomax55.mch` against the Fehlmann programming manual and the machine parameters (MP) of a Picomax 55 with iTNC 530 340490-03 |

## Method

1. The source is the public HEIDENHAIN TNCguide portal: the English user manuals of 6 series, 15 control models and 84 NC software pages (TNC 124 to TNC7), plus the "Overview of New and Modified Software Functions" documents and some brochures. `inventory.md` lists each document.
2. The text of each PDF was extracted with `pdftotext -layout`. Scripts searched the text. Nobody read the manuals page by page.
3. The main sources are the "New functions" / "Changed functions" lists of each manual and the overview documents. The Q-parameter lists of the cycles come from the example block that starts with the cycle's own number and name (section B3 of `differences.md`).
4. Section C assigns an entry to a control only when the manual covers that control. The DIN/ISO manual 345 940-22 is shared by the TNC 410, TNC 426 and TNC 430. Its "not TNC 410" marks were applied, so its 280 476 list counts for the TNC 426/430 only.
5. Short excerpts were read to confirm a detail. Each fact has a document ID and a PDF page (`<ID> p<N>`).
6. A gap is marked "unsure" or "not found in the extracted text". "Not found" does not prove that a control lacks a function.

## How complete

- All six series of the portal are covered. Every manual with a new-function list was processed: iTNC 530 340 420 to 60642x-04 SP8, TNC 320, TNC 620, TNC 640, TNC7 and TNC 128.
- The old manuals without such lists (TNC 124, TNC 310, TNC 410, TNC 426/430) are covered by keyword search only. The TNC 426 280 460 manual is a scan without text and was not analysed.
- The TNC 406 / TNC 416 are EDM controls. They are out of scope.
- Three portal links are dead (HTTP 404). Three brochures were blocked (HTTP 403). The most useful missing document is "Functions of the TNC7".

## Copyright and clean-room note

The HEIDENHAIN manuals are © HEIDENHAIN. These files hold facts about control behaviour with document references. They do not hold manual text, except short phrases with the document ID and the page. No manual and no extracted text is in this repository. Use the URLs in `inventory.md` to get a manual. HEIDENHAIN manuals are allowed reference material for behaviour facts (AGENTS.md, "Clean-Room Protocol"). No HSMWorks material was used.

## Top findings for the posts

1. **Three dialect groups.** TNC 124 and TNC 128 are paraxial: one axis in each block, no `L`, no arcs, `R+`/`R-`. TNC 310 is contouring Klartext only. All other controls add DIN/ISO from a version on.
2. **DIN/ISO is not on all controls.** None on TNC 310, TNC 128 and TNC 124. TNC 320 from 340 55x-04, TNC 620 from 340 56x-02, TNC 426/430 from 280 472, TNC7 only from 81762x-17.
3. **Cycle 205 (UNIVERSAL PECKING) grew in steps.** Q379/Q253 from iTNC 340 422. Q208 and Q395 only from iTNC 60642x-04, TNC 640 34059x-04, TNC 620 81760x-01 and TNC 320 77185x-01 (1096959-20 p9). Cycle 200 (DRILLING) has Q395 from the same releases. The TNC 426/430 gets cycle 205 only from 280 474. The TNC 410 has no cycle 205. The DIN/ISO post writes Q395/Q208 today, so its output is not valid on iTNC 530 340 49x software (#4094).
4. **`FUNCTION DWELL` is on no iTNC 530 version.** It comes with TNC 640 34059x-07, TNC 620 81760x-04 and TNC 320 77185x-04. Use `G04 F..` (cycle 9 DWELL TIME) in DIN/ISO (#4093).
5. **Cycle 247 (PRESETTING) changed meaning in iTNC 340 422.** Before, it set a datum value. After, it activates a row of the preset table.
6. **PLANE** comes with iTNC 340 422, TNC 620 340 56x-02 and TNC 320 340 55x-04. PLANE AXIAL comes with iTNC 34049x-03. The TNC7 keeps cycle 19 (WORKING PLANE) only to edit old programs (from 81762x-17), and converts cycle 7 (DATUM SHIFT) into TRANS DATUM.
7. **FUNCTION TCPM in ISO programs** comes only with TNC 640 34059x-10 and TNC 620 81760x-07. Before that, and on the iTNC 530 ISO side, use M128. The TNC 320 has no 5-axis option.
8. **Tool calls.** The tool name in quotes comes with TNC 426 280 462 (not TNC 410 or TNC 310). The name length went from 16 to 32 characters in TNC 640 34059x-02. FZ/FU come with iTNC 34049x-01. TNC7 81762x-19 refuses `L`/`R` in `TOOL DEF`.
9. **Options.** On the TNC 620, cycles 201-205, 208 and 209 need option 19. Tilting needs option 8 and TCPM needs option 9. The existing `installed_options` field can carry this.
10. **Number format.** TNC 310, TNC 410 and TNC 124 take 1 µm (3 decimals). TNC 426/430, iTNC 530, TNC 320/620/640 and TNC7 take 0.1 µm (4 decimals). TNC 640 34059x-11 changed the operator priority in Q formulas: write parentheses.
11. **TNC 410.** The shared ISO manual 345 940-22 marks cycles 205-209, 247, 19 (G80), 32 (G62), the SL II cycles, M128 and the tilted working plane as "not TNC 410". The TNC 410 profile in `dialects.md` follows these marks.

## Related issues

- #4095: the HEIDENHAIN posts write the dialect of the control and NC software in the machine file. `dialects.md` is the proposal for it.
- #4093: `FUNCTION DWELL` in DIN/ISO (finding 4).
- #4094: Q395 and Q208 in cycles 200/205 (finding 3).
- #4085, #4086, #4087, #4088: defects from the Picomax 55 review (tool change without a stop, arc tolerance, tool numbers above 254, preset row 6).
- #3095: verify the DIN/ISO output on a real iTNC 530 340490-03.
