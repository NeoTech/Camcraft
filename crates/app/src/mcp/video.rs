//! `render_simulation_video`: MCP and script door for the built-in 3D
//! multi-setup stock-simulation movie renderer (`crate::sim_video`).
//!
//! Provenance: exposes `sim_video::render_simulation_video` over the same
//! JSON-RPC tool table used by both `camcraft-cli mcp` and `cam.call(...)`.

use std::path::PathBuf;

use serde_json::{json, Map, Value};

use super::tools::{opt_str, req_str};
use crate::script::Env;
use crate::sim_video::{
    render_simulation_video, CameraView, VideoFormat, VideoOptions, DEFAULT_FPS, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    MAX_EDGE, MIN_EDGE,
};

pub(super) fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "output": { "type": "string", "description": "Output movie path (.mp4, .webm, .gif) or frame directory (for format=\"frames\")." },
            "format": { "type": "string", "enum": ["auto", "mp4", "webm", "gif", "frames"], "description": "Output format; default auto (detected from extension, or PNG frames when none)." },
            "setup": { "type": "string", "description": "Render only one setup (earlier setups are still simulated silently so incoming stock is exact)." },
            "per_setup": { "type": "boolean", "description": "Also write one video (or frame folder) per setup alongside the combined output (default false)." },
            "width": { "type": "integer", "minimum": MIN_EDGE, "maximum": MAX_EDGE, "description": "Frame width in pixels (default 960; must be even for mp4/webm)." },
            "height": { "type": "integer", "minimum": MIN_EDGE, "maximum": MAX_EDGE, "description": "Frame height in pixels (default 540; must be even for mp4/webm)." },
            "fps": { "type": "integer", "minimum": 1, "maximum": 120, "description": "Frames per second (default 30)." },
            "cell_mm": { "type": "number", "description": "Voxel pitch in mm (auto-sized around ~14M voxels if omitted)." },
            "view": { "type": "string", "enum": ["iso", "front-right", "top", "side"], "description": "Camera angle preset (default iso)." },
            "toolpath": { "type": "boolean", "description": "Overlay live depth-tested 3D toolpath ribbon trail (default true)." },
            "hud": { "type": "boolean", "description": "Overlay operation HUD, DRO readout, tool badge, and progress bar (default true)." },
            "seconds": { "type": "number", "minimum": 0.0, "maximum": 600.0, "description": "Approximate total machining playback seconds across rendered setups (auto if omitted)." },
            "transition_seconds": { "type": "number", "minimum": 0.0, "maximum": 600.0, "description": "Setup re-orientation flip/turn animation duration in seconds (default 1.1)." },
            "spin_seconds": { "type": "number", "minimum": 0.0, "maximum": 600.0, "description": "End-of-program turntable inspection spin duration in seconds (default 3.0)." },
            "keyframes_dir": { "type": "string", "description": "Optional directory to write representative PNG keyframes during the render." }
        },
        "required": ["output"],
        "additionalProperties": false,
    })
}

fn opt_u32(a: &Map<String, Value>, key: &str, default: u32, min: u32, max: u32) -> Result<u32, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| (min..=max).contains(n))
            .ok_or_else(|| format!("\"{key}\" must be an integer from {min} to {max}, got {v}")),
    }
}

fn opt_bool(a: &Map<String, Value>, key: &str, default: bool) -> Result<bool, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(format!("\"{key}\" must be a boolean, got {other}")),
    }
}

fn opt_f64(a: &Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_f64()
            .filter(|x| x.is_finite())
            .map(Some)
            .ok_or_else(|| format!("\"{key}\" must be a finite number, got {v}")),
    }
}

pub(super) fn render_video(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let output = PathBuf::from(req_str(a, "output")?);
    let format = match opt_str(a, "format")? {
        None => VideoFormat::Auto,
        Some(s) => VideoFormat::parse(s)?,
    };
    let view = match opt_str(a, "view")? {
        None => CameraView::Iso,
        Some(s) => CameraView::parse(s)?,
    };
    let opts = VideoOptions {
        output,
        format,
        width: opt_u32(a, "width", DEFAULT_WIDTH, MIN_EDGE, MAX_EDGE)?,
        height: opt_u32(a, "height", DEFAULT_HEIGHT, MIN_EDGE, MAX_EDGE)?,
        fps: opt_u32(a, "fps", DEFAULT_FPS, 1, 120)?,
        cell_mm: opt_f64(a, "cell_mm")?,
        setup: opt_str(a, "setup")?.map(str::to_string),
        per_setup: opt_bool(a, "per_setup", false)?,
        toolpath: opt_bool(a, "toolpath", true)?,
        hud: opt_bool(a, "hud", true)?,
        view,
        seconds: opt_f64(a, "seconds")?,
        transition_seconds: opt_f64(a, "transition_seconds")?,
        spin_seconds: opt_f64(a, "spin_seconds")?,
        keyframes_dir: opt_str(a, "keyframes_dir")?.map(PathBuf::from),
    };
    let report = render_simulation_video(&env.world, &opts)?;
    serde_json::to_value(report).map_err(|e| format!("serialise video report: {e}"))
}
