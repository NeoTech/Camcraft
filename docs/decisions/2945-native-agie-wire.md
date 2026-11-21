## #2945 - Native Agie wire job for a rectangular punch

Write the ISO geometry, SBR reference and SBL job in CamCraft. Use the existing post crate for the writer. Keep document checks and nominal geometry extraction in the app. The post crate uses tempfile for same-volume staging and rustix or windows-sys for an atomic rename that cannot replace another job. These dependencies already occur in the lockfile. No new dependency between CamCraft crates is needed. An external generator would add an installation requirement.

Use the Classic 2S template from #2946. The operator adds the AGIE_QN2 condition and selects the 1 mm separation length. The template stays unconfigured until then. Use nominal geometry because the resident control determines the wire offsets and trims. Applying the preview gap to the exported contour would compensate twice.

Limit the first job to one captured rectangular punch in 20 mm stock. The offline control plotted the 0.25 mm Berco wire outside the punch. Its rough centre offset was 0.245 mm. Adding the 0.125 mm wire radius and rounding outward gives a 0.4 mm stock margin. The entry needs 3 mm of straight face on either side. The control cuts the separate 1 mm interval after the long passes. Other thicknesses, conditions, profiles and separation lengths need separate control evidence before the guards change. The setup and physical release still need an operator check.

The contributor's team published the reference implementation under MIT. The notice is in `assets/ui/LICENSES.md`. The public manuals explain the file roles, object calls and quality targets. The detailed JEScript call sequence, its arguments and the QN2 inputs follow the contributor's published implementation. Some of those interface details came from observed controller examples. Whether that reference meets this project's source rules is an open review question in #2945. The PR includes no vendor runtime, controller database, firmware or vendor program. The byte goldens are CamCraft output for original synthetic geometry.

- [Agie-CAM source at 11b1bb9](https://github.com/qsmits/Agie-CAM/tree/11b1bb9ef2c752b94b6fefc1f0814acd2e030c42)
- [EZ-CAM AgieVision file workflow](https://www.ez-cam.de/downloads/v2017_edm_news.pdf)
- [Kovoprog manual, section 3.4.1](https://www.kovoprog.cz/pictures/files/manuals/MainBook_kpd_en_open.pdf)
- [Cimatron AgieVision quality targets](https://help.cimatron.com/en/2026/wizards/agievision/wedm_w_agv_step_5_7.htm)
