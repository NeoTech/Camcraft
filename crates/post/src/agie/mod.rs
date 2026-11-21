//! Native Classic Cut 2S job export, from the operator's Agie Processor native-import specification.

use std::path::{Path, PathBuf};

mod contour;
mod cutoff;
mod jescript;
mod round_profile;

/// The closed cycle or the kerf side of a directed open cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgieSide {
    Punch,
    Die,
    Left,
    Right,
}

impl AgieSide {
    pub fn is_open(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Nominal geometry in setup coordinates (millimetres, stock top at zero).
pub struct AgieJob {
    pub side: AgieSide,
    pub contour: Vec<[f64; 2]>,
    pub round_profile: Option<camcraft_geometry::wire_round_profile::RoundProfile>,
    pub start: Option<[f64; 2]>,
    pub stock_min: [f64; 3],
    pub stock_max: [f64; 3],
}

/// The job name doubles as a controller file stem. Reject DOS device names
/// and directory components even when the host filesystem permits them.
pub fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    (1..=8).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        && !matches!(name, "CON" | "AUX" | "NUL" | "PRN")
        && !(bytes.len() == 4
            && (name.starts_with("COM") || name.starts_with("LPT"))
            && bytes[3].is_ascii_digit())
}

/// Write a complete controller job to a new directory.
///
/// # Errors
/// Refuses unsupported geometry, settings and destinations without publishing a partial job.
pub fn export(
    job: &AgieJob,
    destination: &Path,
    program: &str,
    separation_length_mm: f64,
) -> Result<Vec<PathBuf>, String> {
    export_works(
        std::slice::from_ref(job),
        destination,
        program,
        separation_length_mm,
    )
}

struct Work {
    side: AgieSide,
    geometry: String,
    points: Vec<[f64; 2]>,
    arc_centers: Vec<Option<[f64; 2]>>,
    entry: [f64; 2],
    exit: Option<[f64; 2]>,
    // Bounds of the captive solid, full threading corridor, separation and swept wire.
    min: [f64; 2],
    max: [f64; 2],
}

impl Work {
    fn prepare(job: &AgieJob, geometry: String, separation: f64) -> Result<Self, String> {
        if job.side.is_open() && separation != 0.0 {
            return Err("Set separationLengthMm to zero for an Agie open cutoff".into());
        }
        if !job.side.is_open() && separation != 1.0 {
            return Err("Set the Agie separation cut length (separationLengthMm) to 1.0 mm for this closed contour. A zero or unspecified separation cannot pass the control formal check".into());
        }
        if let Some(shape) = job.round_profile {
            return round_profile::prepare(job, shape, geometry);
        }
        let (points, entry, exit, reach) = if job.side.is_open() {
            let cut = cutoff::prepare(job)?;
            (cut.line.to_vec(), cut.entry, Some(cut.exit), 0.0)
        } else {
            let (mut points, reach) = contour::prepare_with_reach(job)?;
            let seam = *points.first().ok_or("Agie requires a closed contour")?;
            points.push(seam);
            (
                points,
                job.start.ok_or("Agie requires a start hole")?,
                None,
                reach,
            )
        };
        // The nominal ring encloses the retained punch. The wire can also reach
        // the longest miter, a 3 mm lead, and one wire radius. The same padded
        // rectangle includes the complete 2 mm threading hole and its return.
        let margin = reach + 3.0 + 0.125;
        let min = std::array::from_fn(|axis| {
            points.iter().fold(entry[axis], |v, p| v.min(p[axis])) - margin
        });
        let max = std::array::from_fn(|axis| {
            points.iter().fold(entry[axis], |v, p| v.max(p[axis])) + margin
        });
        Ok(Self {
            side: job.side,
            geometry,
            points,
            arc_centers: Vec::new(),
            entry,
            exit,
            min,
            max,
        })
    }
}

/// Export a single work or two independently captured punches on the same 20 mm stock.
///
/// # Errors
/// Refuses unsupported groups and shared cutting space before publishing the directory.
pub fn export_works(
    jobs: &[AgieJob],
    destination: &Path,
    program: &str,
    separation: f64,
) -> Result<Vec<PathBuf>, String> {
    let first = jobs.first().ok_or("Agie requires an active Wire Contour")?;
    if !valid_name(program) {
        return Err("Agie program name must be a non-reserved 1–8 character DOS name starting with a letter and using uppercase letters or digits".into());
    }
    if jobs.len() > 1 {
        if jobs.len() != 2
            || jobs.iter().any(|job| {
                job.side != AgieSide::Punch
                    || job.round_profile.is_some()
                    || job.stock_min != first.stock_min
                    || job.stock_max != first.stock_max
                    || job.stock_min[2] != -20.0
            })
        {
            return Err("The Agie work group requires two independent closed punches on the same 20 mm stock".into());
        }
        if program.len() > 7 {
            return Err("Choose an Agie group name up to seven characters so each work has a distinct geometry filename".into());
        }
    }
    let mut works = Vec::with_capacity(jobs.len());
    for (i, job) in jobs.iter().enumerate() {
        let geometry = if jobs.len() == 1 {
            program.to_owned()
        } else {
            format!("{program}{}", i + 1)
        };
        if !valid_name(&geometry) {
            return Err(
                "Choose an Agie group name that produces non-reserved geometry filenames".into(),
            );
        }
        works.push(Work::prepare(job, geometry, separation)?);
    }
    if let [a, b] = works.as_slice()
        && !(0..2).any(|i| a.max[i] < b.min[i] || b.max[i] < a.min[i])
    {
        return Err("Agie punch groups require separate cutting areas for both retained solids, threading holes, leads and compensated cuts".into());
    }
    let mut files = Vec::new();
    let mut resources = String::new();
    for work in &works {
        let iso_name = format!("{}.ISO", work.geometry);
        resources.push_str(&format!("AGIE.USING_{iso_name} IMPORT {iso_name};\r\n"));
        files.push((iso_name.clone(), iso(&iso_name, work)));
    }
    files.push((format!("{program}.SBR"), resources));
    files.push((
        format!("{program}.SBL"),
        jescript::piece(first, program, separation, &works),
    ));
    publish(destination, &files)
}

fn iso(iso_name: &str, work: &Work) -> String {
    let points = &work.points;
    let mut iso = format!("( {iso_name} )\r\n");
    for (i, point) in points.iter().enumerate() {
        if let (Some(Some(center)), Some(start)) = (
            work.arc_centers.get(i),
            i.checked_sub(1).and_then(|prev| points.get(prev)),
        ) {
            iso.push_str(&format!(
                "N{:04} G03 X{:.5} Y{:.5} I{:.5} J{:.5} ;\r\n",
                i + 1,
                point[0],
                point[1],
                center[0] - start[0],
                center[1] - start[1]
            ));
            continue;
        }
        let motion = if i == 0 { "G00" } else { "G01" };
        iso.push_str(&format!(
            "N{:04} {motion} X{:.5} Y{:.5} ;\r\n",
            i + 1,
            point[0],
            point[1]
        ));
    }
    iso.push_str(&format!("N{:04} M02 ;\r\n", points.len() + 1));
    iso
}

#[cfg(not(target_arch = "wasm32"))]
fn publish(destination: &Path, files: &[(String, String)]) -> Result<Vec<PathBuf>, String> {
    use std::{fs, io::Write};
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if destination.file_name().is_none() {
        return Err("Choose a new directory name for the Agie bundle".into());
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(format!(
                "Agie destination {} already exists; choose a new directory",
                destination.display()
            ));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(format!(
                "Cannot inspect Agie destination {}: {e}",
                destination.display()
            ));
        }
    }
    // The staging directory and the final name share a filesystem. The rename commits all
    // files together. Its no-replace mode also protects a destination created since the check.
    let stage = tempfile::Builder::new()
        .prefix(".camcraft-agie-")
        .tempdir_in(parent)
        .map_err(|e| format!("Cannot stage Agie job in {}: {e}", parent.display()))?;
    let mut paths = Vec::new();
    for (name, text) in files {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(stage.path().join(name))
            .map_err(|e| format!("Cannot stage {name}: {e}"))?;
        file.write_all(text.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("Cannot write {name}: {e}"))?;
        paths.push(destination.join(name));
    }
    rename_new(stage.path(), destination)
        .map_err(|e| format!("Cannot publish Agie job to {}: {e}", destination.display()))?;
    // The temporary name ceased to exist at the rename. Disarm its path-based cleanup.
    let _old_name = stage.keep();
    Ok(paths)
}

#[cfg(any(target_vendor = "apple", target_os = "linux"))]
fn rename_new(from: &Path, to: &Path) -> std::io::Result<()> {
    use rustix::fs::{CWD, RenameFlags, renameat_with};
    renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE).map_err(Into::into)
}

#[cfg(windows)]
fn rename_new(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    fn wide(path: &Path) -> std::io::Result<Vec<u16>> {
        let mut chars: Vec<_> = path.as_os_str().encode_wide().collect();
        if chars.contains(&0) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "NUL in output path",
            ));
        }
        chars.push(0);
        Ok(chars)
    }
    let from = wide(from)?;
    let to = wide(to)?;
    // SAFETY: both buffers contain terminated UTF-16 and remain live for the call.
    // MoveFileW on the same volume fails if the destination already exists.
    if unsafe { windows_sys::Win32::Storage::FileSystem::MoveFileW(from.as_ptr(), to.as_ptr()) }
        == 0
    {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(any(
    target_arch = "wasm32",
    target_vendor = "apple",
    target_os = "linux",
    windows
)))]
fn rename_new(_: &Path, _: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic Agie bundle publication is unavailable on this platform",
    ))
}

#[cfg(target_arch = "wasm32")]
fn publish(_: &Path, _: &[(String, String)]) -> Result<Vec<PathBuf>, String> {
    Err("Agie bundle output requires a local filesystem".into())
}

#[cfg(test)]
mod tests;
