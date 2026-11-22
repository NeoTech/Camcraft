//! Frame output sinks for `sim_video`: pure-Rust PNG sequences via `png` and streamed video
//! encoding (`.mp4`, `.webm`, `.gif`) via a piped `ffmpeg` process (native builds only).
//!
//! Provenance: `png` crate RGB8 encoding (W3C PNG spec) and `ffmpeg` stdin `rawvideo`/`rgb24`
//! pipe interface.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoFormat {
    Auto,
    Mp4,
    Webm,
    Gif,
    Frames,
}

impl VideoFormat {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "mp4" => Ok(Self::Mp4),
            "webm" => Ok(Self::Webm),
            "gif" => Ok(Self::Gif),
            "frames" | "png" => Ok(Self::Frames),
            other => Err(format!(
                "unsupported format \"{other}\" (expected auto, mp4, webm, gif, or frames)"
            )),
        }
    }

    pub fn resolve(self, output: &Path) -> Self {
        if self != Self::Auto {
            return self;
        }
        match output
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("mp4" | "m4v" | "mov") => Self::Mp4,
            Some("webm") => Self::Webm,
            Some("gif") => Self::Gif,
            Some("png") | None => Self::Frames,
            Some(_) => Self::Mp4,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Gif => "gif",
            Self::Frames => "frames",
        }
    }
}

/// Write one RGB24 buffer to a PNG file (pure Rust; no external dependencies).
pub(super) fn write_png(path: &Path, width: u32, height: u32, rgb: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("create dir {}: {e}", parent.display()))?;
    }
    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), width, height);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| format!("png header {}: {e}", path.display()))?;
    writer.write_image_data(rgb).map_err(|e| format!("png write {}: {e}", path.display()))?;
    Ok(())
}

pub(super) enum FrameSink {
    Frames {
        dir: PathBuf,
        width: u32,
        height: u32,
        index: usize,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Ffmpeg {
        path: PathBuf,
        child: std::process::Child,
        stdin: Option<std::process::ChildStdin>,
        frames: usize,
    },
}

impl FrameSink {
    pub fn open(path: &Path, format: VideoFormat, width: u32, height: u32, fps: u32) -> Result<Self, String> {
        let fmt = format.resolve(path);
        match fmt {
            VideoFormat::Frames => {
                fs::create_dir_all(path).map_err(|e| format!("create frame directory {}: {e}", path.display()))?;
                Ok(Self::Frames { dir: path.to_path_buf(), width, height, index: 0 })
            }
            VideoFormat::Mp4 | VideoFormat::Webm | VideoFormat::Gif | VideoFormat::Auto => {
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = (fps, width, height);
                    Err("video encoding requires a native build (use format=\"frames\" for PNG output)".into())
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Self::spawn_ffmpeg(path, fmt, width, height, fps)
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_ffmpeg(path: &Path, fmt: VideoFormat, width: u32, height: u32, fps: u32) -> Result<Self, String> {
        use std::process::{Command, Stdio};
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| format!("create output dir {}: {e}", parent.display()))?;
        }
        // libx264/yuv420p requires even dimensions; callers already validate even sizes for video.
        let mut cmd = Command::new("ffmpeg");
        cmd.args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            &format!("{width}x{height}"),
            "-r",
            &fps.to_string(),
            "-i",
            "-",
        ]);
        match fmt {
            VideoFormat::Mp4 => {
                cmd.args([
                    "-c:v", "libx264", "-preset", "medium", "-crf", "19", "-pix_fmt", "yuv420p", "-movflags",
                    "+faststart",
                ]);
            }
            VideoFormat::Webm => {
                cmd.args(["-c:v", "libvpx", "-crf", "10", "-b:v", "2200k", "-cpu-used", "2", "-an"]);
            }
            VideoFormat::Gif => {
                cmd.args([
                    "-vf",
                    "split[s0][s1];[s0]palettegen=max_colors=192[p];[s1][p]paletteuse=dither=bayer",
                ]);
            }
            VideoFormat::Frames | VideoFormat::Auto => {}
        }
        cmd.arg(path);
        cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "ffmpeg was not found on PATH - install ffmpeg to encode MP4/WebM/GIF videos, or pass format=\"frames\" for pure-Rust PNG sequences".to_string()
            } else {
                format!("failed to start ffmpeg for {}: {e}", path.display())
            }
        })?;
        let stdin = child.stdin.take();
        Ok(Self::Ffmpeg { path: path.to_path_buf(), child, stdin, frames: 0 })
    }

    pub fn write_frame(&mut self, rgb: &[u8]) -> Result<(), String> {
        match self {
            Self::Frames { dir, width, height, index } => {
                *index += 1;
                let file = dir.join(format!("frame_{:05}.png", *index));
                write_png(&file, *width, *height, rgb)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::Ffmpeg { path, stdin, frames, .. } => {
                let pipe = stdin.as_mut().ok_or_else(|| format!("ffmpeg pipe closed for {}", path.display()))?;
                pipe.write_all(rgb).map_err(|e| format!("write frame to ffmpeg ({}): {e}", path.display()))?;
                *frames += 1;
                Ok(())
            }
        }
    }

    pub fn finish(mut self) -> Result<usize, String> {
        match &mut self {
            Self::Frames { index, .. } => Ok(*index),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Ffmpeg { path, child, stdin, frames } => {
                drop(stdin.take());
                let status = child
                    .wait()
                    .map_err(|e| format!("wait for ffmpeg ({}): {e}", path.display()))?;
                if !status.success() {
                    let mut msg = String::new();
                    if let Some(mut se) = child.stderr.take() {
                        let _ = std::io::Read::read_to_string(&mut se, &mut msg);
                    }
                    let detail = msg.trim();
                    return Err(if detail.is_empty() {
                        format!("ffmpeg exited with {status} while writing {}", path.display())
                    } else {
                        format!("ffmpeg failed writing {}: {detail}", path.display())
                    });
                }
                Ok(*frames)
            }
        }
    }
}

/// Derive a clean per-setup output path alongside `base` (e.g. `sim.mp4` -> `sim_01_OP10_Top.mp4`).
pub(super) fn per_setup_path(base: &Path, format: VideoFormat, index: usize, setup_name: &str) -> PathBuf {
    let clean: String = setup_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let tag = format!("{:02}_{}", index + 1, if clean.is_empty() { "setup" } else { &clean });
    if format.resolve(base) == VideoFormat::Frames {
        return base.join(tag);
    }
    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("simulation");
    let ext = base.extension().and_then(|e| e.to_str()).unwrap_or(format.resolve(base).as_str());
    parent.join(format!("{stem}_{tag}.{ext}"))
}
