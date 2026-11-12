//! The UI icon set, folded into the egui stack at M1.4.2 M-g: bundled
//! 32 px PNGs, recolored to white line art and loaded as egui textures.
//!
//! Provenance: the loader is our own, ported from the retired imgui
//! icons.rs. The chrome glyphs are Tabler Icons (MIT, see
//! assets/ui/LICENSES.md), unmodified except channel order; the
//! operation glyphs (#661) are our own drawings, SVG masters under
//! assets/ui/svg/ rasterised by `cargo xtask icons` (AGENTS.md clean-room:
//! never any HSMWorks or other CAM product's asset).
//!
//! The imgui loader hand-built wgpu textures and registered them as
//! external ids; egui owns that machinery, so `TextureHandle`s come from
//! `ctx.load_texture` (the renderer's texture manager uploads and
//! re-uploads as needed). The law carried over unchanged: building the
//! set is a lazy GPU job (a headless or pre-GPU frame holds `None`), a
//! glyph that fails to decode is dropped to `None` with a diagnostic -
//! never the whole ribbon - and every consumer must degrade to its
//! painted fallback, never to nothing.

use egui::{ColorImage, Context, TextureHandle, TextureOptions};

/// One table row per slot: the variant, its PNG stem under assets/ui/
/// (an `op-*` stem has an SVG master under assets/ui/svg/), and its egui
/// texture key - stable per slot, never reused for other pixels (a rename
/// would alias old uploads). The table lives once; the enum, `ALL`, the
/// bytes and the keys are all generated from it.
macro_rules! icon_table {
    ($( $(#[$m:meta])* $v:ident => $file:literal, $key:literal; )*) => {
        /// One bundled glyph. The ribbon is the first consumer; the
        /// browser tree pass added slots here, not elsewhere.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum Icon { $( $(#[$m])* $v, )* }

        impl Icon {
            /// Every slot, ribbon order - the table and the store share this list.
            pub(crate) const ALL: &'static [Icon] = &[ $( Icon::$v, )* ];

            fn png(self) -> &'static [u8] {
                match self {
                    $( Icon::$v => include_bytes!(concat!("../../../../assets/ui/", $file, ".png")), )*
                }
            }

            /// The PNG stem under assets/ui/ (pinned by the structural tests:
            /// every operation glyph is an `op-*` file with an SVG master).
            #[cfg(test)]
            pub(crate) fn file(self) -> &'static str {
                match self { $( Icon::$v => $file, )* }
            }

            fn name(self) -> &'static str {
                match self { $( Icon::$v => $key, )* }
            }
        }
    };
}

icon_table! {
    New => "file-plus", "icon-new";
    OpenModel => "folder-open", "icon-open-model";
    OpenProject => "file-import", "icon-open-project";
    Save => "device-floppy", "icon-save";
    /// Original drawing (no Tabler "post to G-code" glyph exists).
    Post => "post", "icon-post";
    Stock => "box-model", "icon-stock";
    WorkOffset => "transform", "icon-work-offset";
    Tools => "tools", "icon-tools";
    // The operation glyphs (#661): one per `OpType`, every one an
    // original SVG master in assets/ui/svg/ - a cutter and its path,
    // drawn from what the operation does. 2D milling:
    Face => "op-face", "icon-face";
    Contour => "op-contour", "icon-contour";
    Pocket => "op-pocket", "icon-pocket";
    Adaptive2D => "op-adaptive2d", "icon-adaptive2d";
    Offset2D => "op-offset2d", "icon-offset2d";
    Slot => "op-slot", "icon-slot";
    TrochoidalSlot => "op-trochoidal-slot", "icon-trochoidal-slot";
    Trace => "op-trace", "icon-trace";
    Engrave => "op-engrave", "icon-engrave";
    Chamfer2D => "op-chamfer2d", "icon-chamfer2d";
    Deburr => "op-deburr", "icon-deburr";
    Circular => "op-circular", "icon-circular";
    PlasmaProfile => "op-plasma-profile", "icon-plasma-profile";
    /// A sinker EDM burn (#541): the electrode plunging into a cavity.
    SinkerBurn => "op-sinker-burn", "icon-sinker-burn";
    // Holes and probing:
    Drill => "op-drill", "icon-drill";
    Bore => "op-bore", "icon-bore";
    Thread => "op-thread", "icon-thread";
    /// The touch probe (#108): a stylus over its ball.
    Probe => "op-probe", "icon-probe";
    // 3D milling:
    Pocket3D => "op-pocket3d", "icon-pocket3d";
    Finish3D => "op-finish3d", "icon-finish3d";
    // Multi-axis: the dome with the ball leaning on its normal; the
    // swarf blade with the tool lying on it; the rotary bar with the
    // tool riding outside it on the radial.
    Contour5D => "op-contour5d", "icon-contour5d";
    Parallel5D => "op-parallel5d", "icon-parallel5d";
    Swarf => "op-swarf", "icon-swarf";
    AdvancedSwarf => "op-advanced-swarf", "icon-advanced-swarf";
    RotaryContour => "op-rotary-contour", "icon-rotary-contour";
    RotaryParallel => "op-rotary-parallel", "icon-rotary-parallel";
    RotaryOffset => "op-rotary-offset", "icon-rotary-offset";
    MultiAxisRough => "op-multiaxis-rough", "icon-multiaxis-rough";
    Flow5D => "op-flow5d", "icon-flow5d";
    MultiAxisContour => "op-multiaxis-contour", "icon-multiaxis-contour";
    // Turning: the chuck block at the left, the bar, an insert or blade.
    Facing => "op-facing", "icon-facing";
    ProfileRough => "op-profile-rough", "icon-profile-rough";
    ProfileFinish => "op-profile-finish", "icon-profile-finish";
    GrooveRough => "op-groove-rough", "icon-groove-rough";
    GrooveFinish => "op-groove-finish", "icon-groove-finish";
    PartOff => "op-part-off", "icon-part-off";
    Threading => "op-threading", "icon-threading";
    Chamfering => "op-chamfering", "icon-chamfering";
    TaperTurning => "op-taper-turning", "icon-taper-turning";
    MultiPassTurning => "op-multipass-turning", "icon-multipass-turning";
    FormTurning => "op-form-turning", "icon-form-turning";
    BoringTurning => "op-boring-turning", "icon-boring-turning";
    Undo => "arrow-back-up", "icon-undo";
    Redo => "arrow-forward-up", "icon-redo";
    Delete => "trash", "icon-delete";
    Generate => "player-play", "icon-generate";
    FrameAll => "maximize", "icon-frame-all";
    Toolpaths => "eye", "icon-toolpaths";
    Model => "cube", "icon-model";
    EyeOff => "eye-off", "icon-eye-off";
    Simulate => "sim", "icon-simulate";
    /// Original drawing (no Tabler vise glyph exists): the two jaws,
    /// screw shaft and round knob of a machine vise.
    Fixture => "fixture", "icon-fixture";
    /// The status strip's Debug info door (#157), an original glyph.
    DebugInfo => "debug-info", "icon-debug-info";
}

/// Upload edge: 2x the token's `glyph_px` (20 display px at the dark
/// defaults - the painted path reads the token, not this number) for
/// clean minification on the button glyphs.
const UPLOAD_PX: u32 = 40;

/// Loaded icon set: one handle per slot, parallel to [`Icon::ALL`], with
/// `None` where a glyph failed to decode. A headless frame never builds
/// one at all, so every consumer takes `Option<&IconSet>`.
pub(crate) struct IconSet {
    handles: Vec<Option<TextureHandle>>,
}

impl IconSet {
    /// Decode, recolor and register every bundled glyph. Called once,
    /// lazily, from the first GPU-backed frame.
    pub(crate) fn load(ctx: &Context) -> Self {
        let handles = Icon::ALL
            .iter()
            .map(|icon| match Self::upload(ctx, *icon) {
                Ok(handle) => Some(handle),
                Err(err) => {
                    diag_warn!("icon {icon:?} unavailable: {err}");
                    None
                }
            })
            .collect();
        IconSet { handles }
    }

    fn upload(ctx: &Context, icon: Icon) -> Result<TextureHandle, String> {
        let img = image::load_from_memory(icon.png())
            .map_err(|e| e.to_string())?
            .into_rgba8();
        let small = image::imageops::resize(
            &img,
            UPLOAD_PX,
            UPLOAD_PX,
            image::imageops::FilterType::Lanczos3,
        );
        // Tabler strokes are black with alpha coverage; the chrome is dark,
        // so repaint every pixel white and let the alpha do the drawing.
        let mut rgba = small.to_vec();
        for px in rgba.as_chunks_mut::<4>().0 {
            px[0] = 255;
            px[1] = 255;
            px[2] = 255;
        }
        let color_image = ColorImage::from_rgba_unmultiplied([UPLOAD_PX as usize; 2], &rgba);
        Ok(ctx.load_texture(
            icon.name(),
            color_image,
            TextureOptions::default(),
        ))
    }

    /// Texture for a slot (`None` after a failed decode; every consumer
    /// must degrade to its painted fallback, never to nothing).
    pub(crate) fn texture(&self, icon: Icon) -> Option<&TextureHandle> {
        let i = Icon::ALL.iter().position(|s| *s == icon)?;
        self.handles.get(i)?.as_ref()
    }
}
