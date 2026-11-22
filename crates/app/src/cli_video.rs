//! `camcraft-cli render-video`: argv parser and generated script for rendering
//! 3D multi-setup stock-simulation movies via the live `render_simulation_video` tool.
//!
//! Provenance: thin CLI wrapper matching `cli::lane_process` (exit 0 ok, 1 refused, 2 usage).

use serde_json::{json, Map, Value};
use std::path::PathBuf;

pub(crate) const RENDER_VIDEO_USAGE: &str = "\
camcraft-cli render-video <job.camproj> -o <out.mp4|dir> [options]

    render a 3D multi-setup material-removal simulation movie showing
    spinning cutters, live depth-tested toolpath trails, HUD readout,
    and smooth setup re-orientation flips/turns

ARGUMENTS:
    <job.camproj>           the project file to simulate

OPTIONS:
    -o, --out <path>        output video (.mp4, .webm, .gif) or PNG frame directory
    --format <fmt>          auto | mp4 | webm | gif | frames (default: auto)
    --setup <name>          render only one setup (simulates prior setups first)
    --per-setup             also write one video/folder per setup alongside output
    --width <px>            frame width in pixels (default: 960; even for video)
    --height <px>           frame height in pixels (default: 540; even for video)
    --fps <n>               frames per second (default: 30)
    --cell <mm>             voxel pitch in mm (default: auto ~14M voxels)
    --view <preset>         camera angle: iso | front-right | top | side
    --seconds <n>           approximate total machining playback duration (s)
    --transition <n>        flip/turn transition duration between setups (s)
    --spin <n>              end turntable inspection spin duration (s)
    --keyframes <dir>       write representative PNG keyframes to this directory
    --no-toolpath           hide the live 3D toolpath ribbon trail
    --no-hud                hide the HUD overlay bars and DRO card
    --generate              run cam.generate() before rendering toolpaths
    -h, --help              print this usage and exit 0

Exit: 0 stood, 1 refused, 2 usage.
";

fn parse_u32(flag: &str, raw: &str) -> Result<u32, i32> {
    raw.parse::<u32>().map_err(|_| {
        eprintln!("camcraft-cli: {flag} expects a positive integer, got {raw:?}");
        2
    })
}

fn parse_f64(flag: &str, raw: &str) -> Result<f64, i32> {
    match raw.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(v),
        _ => {
            eprintln!("camcraft-cli: {flag} expects a finite number, got {raw:?}");
            Err(2)
        }
    }
}

fn next_val<'a>(it: &mut std::slice::Iter<'a, String>, flag: &str) -> Result<&'a str, i32> {
    it.next().map(String::as_str).ok_or_else(|| {
        eprintln!("camcraft-cli: {flag} needs a value");
        2
    })
}

pub(crate) fn lane_render_video(args: &[String]) -> i32 {
    let mut proj: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut run_generate = false;
    let mut opts = Map::<String, Value>::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let res: Result<(), i32> = match a.as_str() {
            "-o" | "--out" => next_val(&mut it, a).map(|p| out = Some(PathBuf::from(p))),
            "--format" => next_val(&mut it, a).map(|v| { opts.insert("format".into(), json!(v)); }),
            "--setup" => next_val(&mut it, a).map(|v| { opts.insert("setup".into(), json!(v)); }),
            "--per-setup" => { opts.insert("per_setup".into(), json!(true)); Ok(()) }
            "--width" => next_val(&mut it, a).and_then(|v| parse_u32(a, v)).map(|n| { opts.insert("width".into(), json!(n)); }),
            "--height" => next_val(&mut it, a).and_then(|v| parse_u32(a, v)).map(|n| { opts.insert("height".into(), json!(n)); }),
            "--fps" => next_val(&mut it, a).and_then(|v| parse_u32(a, v)).map(|n| { opts.insert("fps".into(), json!(n)); }),
            "--cell" => next_val(&mut it, a).and_then(|v| parse_f64(a, v)).map(|n| { opts.insert("cell_mm".into(), json!(n)); }),
            "--view" => next_val(&mut it, a).map(|v| { opts.insert("view".into(), json!(v)); }),
            "--seconds" => next_val(&mut it, a).and_then(|v| parse_f64(a, v)).map(|n| { opts.insert("seconds".into(), json!(n)); }),
            "--transition" => next_val(&mut it, a).and_then(|v| parse_f64(a, v)).map(|n| { opts.insert("transition_seconds".into(), json!(n)); }),
            "--spin" => next_val(&mut it, a).and_then(|v| parse_f64(a, v)).map(|n| { opts.insert("spin_seconds".into(), json!(n)); }),
            "--keyframes" => next_val(&mut it, a).map(|v| { opts.insert("keyframes_dir".into(), json!(v)); }),
            "--no-toolpath" => { opts.insert("toolpath".into(), json!(false)); Ok(()) }
            "--no-hud" => { opts.insert("hud".into(), json!(false)); Ok(()) }
            "--generate" => { run_generate = true; Ok(()) }
            other if other.starts_with('-') => {
                eprintln!("camcraft-cli: unknown option {other:?} (try render-video --help)");
                return 2;
            }
            other => {
                if proj.is_some() {
                    eprintln!("camcraft-cli: render-video takes one project (extra: {other:?})");
                    return 2;
                }
                proj = Some(PathBuf::from(other));
                Ok(())
            }
        };
        if let Err(code) = res {
            return code;
        }
    }
    let Some(path) = proj else {
        eprintln!("camcraft-cli: render-video needs a .camproj project");
        return 2;
    };
    if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("camproj")) {
        eprintln!("camcraft-cli: {} is not a .camproj project", path.display());
        return 2;
    }
    let Some(out_path) = out else {
        eprintln!("camcraft-cli: render-video requires -o / --out <path>");
        return 2;
    };
    opts.insert("output".into(), json!(out_path.display().to_string()));
    let gen_line = if run_generate { "cam.generate();\n" } else { "" };
    let source = format!(
        "cam.load({});\n{gen_line}cam.call(\"render_simulation_video\", {});",
        json!(path.display().to_string()),
        Value::Object(opts),
    );
    let outcome = crate::script::run(&source);
    for line in &outcome.logs {
        println!("{line}");
    }
    for line in &outcome.errors {
        eprintln!("camcraft-cli: {line}");
    }
    if !outcome.result.is_null() {
        println!("{}", outcome.result);
    }
    if outcome.ok { 0 } else { 1 }
}
