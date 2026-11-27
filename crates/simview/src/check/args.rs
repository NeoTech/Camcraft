//! The command line of `camcraft-sim check` (#2586): each value is read
//! and refused by name here, before any file is judged.
//! Provenance: our own command line; G54..G59 are the RS-274/NGC work
//! offset words.

use std::collections::BTreeMap;

pub const USAGE: &str = "\
camcraft-sim check - judge a posted 3-axis G-code file (headless)

USAGE:
    camcraft-sim check <file.nc> --stock <box> --tools <T map> (--post <file.cps> | --dialect <name>)
                       [--machine <file.mch>] [--wcs G54=x,y,z]... [--fixture <box>]... [--cell <mm>]

    <box>      x0,y0,z0,x1,y1,z1 in mm, in the frame of the program's first work offset
    <T map>    T1=6,T2=10:ball,T3=8:flute=20:holder=40x50:shank=8x30
               the diameter in mm, then optional words: `ball`, `flute=<length>`,
               `shank=<diameter>x<length>`, `holder=<diameter>x<length>`
    --post     the post's manifest gives the dialect, units, work offsets and kinds served
    --dialect  rs274, fanuc, haas, tormach, grblhal, tinyg or carvera (overrides the post's)
    --machine  the machine file: travel limits and the generated machine model
    --wcs      the machine position of a work offset (one for each offset)
    --fixture  a fixture box, in the frame of the stock
    --cell     the stock grid cell in mm (default: from the smallest tool)

EXIT:
    0  each check ran and passed
    1  bad input (a file that cannot be read, a malformed value, a word the slice refuses)
    2  a check failed (each finding names its NC line)
    3  one or more checks were not judged (each is named)
";

/// One tool of the T map, in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolSpec {
    pub diameter: f64,
    pub ball: bool,
    pub flute: Option<f64>,
    /// (diameter, length)
    pub shank: Option<(f64, f64)>,
    /// (diameter, length)
    pub holder: Option<(f64, f64)>,
}

/// The parsed command line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Args {
    pub program: String,
    pub machine: Option<String>,
    pub post: Option<String>,
    pub dialect: Option<String>,
    /// x0, y0, z0, x1, y1, z1
    pub stock: [f64; 6],
    pub tools: BTreeMap<u32, ToolSpec>,
    /// Offset number (1 = G54 .. 6 = G59) to its machine position.
    pub wcs: BTreeMap<u8, [f64; 3]>,
    pub fixtures: Vec<[f64; 6]>,
    pub cell: Option<f64>,
}

/// What the command line asks for: a run, or the help text.
pub enum Parsed {
    Run(Box<Args>),
    Help,
}

/// Read the arguments after the word `check`.
pub fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut a = Args::default();
    let (mut stock, mut tools) = (None, false);
    let mut it = args.iter();
    while let Some(word) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match word.as_str() {
            "-h" | "--help" => return Ok(Parsed::Help),
            "--machine" => a.machine = Some(value("--machine")?),
            "--post" => a.post = Some(value("--post")?),
            "--dialect" => a.dialect = Some(value("--dialect")?),
            "--stock" => stock = Some(boxed("--stock", &value("--stock")?)?),
            "--fixture" => a.fixtures.push(boxed("--fixture", &value("--fixture")?)?),
            "--tools" => {
                tool_map(&value("--tools")?, &mut a.tools)?;
                tools = true;
            }
            "--wcs" => {
                let (n, at) = wcs(&value("--wcs")?)?;
                if a.wcs.insert(n, at).is_some() {
                    return Err(format!("--wcs states G{} two times", 53 + u32::from(n)));
                }
            }
            "--cell" => {
                let v = value("--cell")?;
                a.cell = Some(number("--cell", &v).filter(|c| *c > 0.0).ok_or("--cell must be a positive number")?);
            }
            o if o.starts_with("--") => return Err(format!("unknown option {o:?} (try check --help)")),
            o if a.program.is_empty() => a.program = o.to_owned(),
            o => return Err(format!("one G-code file only: {o:?} is a second one (try check --help)")),
        }
    }
    if a.program.is_empty() {
        return Err("expected a G-code file (try check --help)".into());
    }
    a.stock = stock.ok_or("--stock is required: the stock box x0,y0,z0,x1,y1,z1")?;
    if !tools {
        return Err("--tools is required: a T map such as T1=6,T2=10:ball".into());
    }
    if a.post.is_none() && a.dialect.is_none() {
        return Err("name the post (--post <file.cps>) or the dialect (--dialect <name>)".into());
    }
    Ok(Parsed::Run(Box::new(a)))
}

/// A finite number, or `None`.
fn number(_name: &str, s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Finite numbers separated by `sep`, exactly `n` of them.
fn numbers(name: &str, s: &str, sep: char, n: usize) -> Result<Vec<f64>, String> {
    let v: Vec<Option<f64>> = s.split(sep).map(|w| number(name, w)).collect();
    if v.len() != n || v.iter().any(Option::is_none) {
        return Err(format!("{name} needs {n} finite numbers separated by '{sep}', not {s:?}"));
    }
    Ok(v.into_iter().flatten().collect())
}

/// A box x0,y0,z0,x1,y1,z1 with each minimum under its maximum.
fn boxed(name: &str, s: &str) -> Result<[f64; 6], String> {
    let v = numbers(name, s, ',', 6)?;
    let b = [v[0], v[1], v[2], v[3], v[4], v[5]];
    if !(b[0] < b[3] && b[1] < b[4] && b[2] < b[5]) {
        return Err(format!("{name} needs x0 < x1, y0 < y1, z0 < z1, not {s:?}"));
    }
    Ok(b)
}

/// `G54=x,y,z`: the offset number (1..6) and its machine position.
fn wcs(s: &str) -> Result<(u8, [f64; 3]), String> {
    let (word, at) = s.split_once('=').ok_or_else(|| format!("--wcs needs G54=x,y,z, not {s:?}"))?;
    let n = match word.trim().to_ascii_uppercase().as_str() {
        "G54" => 1,
        "G55" => 2,
        "G56" => 3,
        "G57" => 4,
        "G58" => 5,
        "G59" => 6,
        other => return Err(format!("--wcs names {other:?}: use G54 to G59")),
    };
    let v = numbers("--wcs", at, ',', 3)?;
    Ok((n, [v[0], v[1], v[2]]))
}

/// A positive length, by the name of its word.
fn length(word: &str, s: &str) -> Result<f64, String> {
    number(word, s).filter(|v| *v > 0.0).ok_or_else(|| format!("--tools: {word} must be a positive length, not {s:?}"))
}

/// `<diameter>x<length>`, each positive.
fn band(word: &str, s: &str) -> Result<(f64, f64), String> {
    let (d, l) = s.split_once(['x', 'X']).ok_or_else(|| format!("--tools: {word} needs <diameter>x<length>, not {s:?}"))?;
    Ok((length(word, d)?, length(word, l)?))
}

/// `T1=6,T2=10:ball,...` into `out`. A tool stated two times refuses.
fn tool_map(s: &str, out: &mut BTreeMap<u32, ToolSpec>) -> Result<(), String> {
    if s.to_ascii_lowercase().ends_with(".camtools") {
        return Err(format!(
            "--tools {s:?}: a .camtools library is not read yet (#4465: its reader is in camcraft-app, \
             and it states no T numbers); give a T map such as T1=6,T2=10:ball"
        ));
    }
    for item in s.split(',').map(str::trim).filter(|w| !w.is_empty()) {
        let (t, rest) = item.split_once('=').ok_or_else(|| format!("--tools: {item:?} needs T<n>=<diameter>"))?;
        let n = t
            .trim()
            .strip_prefix(['T', 't'])
            .and_then(|n| n.parse::<u32>().ok())
            .ok_or_else(|| format!("--tools: {t:?} is not a tool number such as T1"))?;
        let mut words = rest.split(':');
        let d = length(&format!("T{n} diameter"), words.next().unwrap_or_default())?;
        let mut spec = ToolSpec { diameter: d, ball: false, flute: None, shank: None, holder: None };
        for w in words {
            match w.split_once('=') {
                None if w == "ball" => spec.ball = true,
                Some(("flute", v)) => spec.flute = Some(length("flute", v)?),
                Some(("shank", v)) => spec.shank = Some(band("shank", v)?),
                Some(("holder", v)) => spec.holder = Some(band("holder", v)?),
                _ => return Err(format!("--tools: T{n}: unknown word {w:?} (ball, flute=, shank=, holder=)")),
            }
        }
        if out.insert(n, spec).is_some() {
            return Err(format!("--tools states T{n} two times"));
        }
    }
    if out.is_empty() {
        return Err("--tools states no tool".into());
    }
    Ok(())
}
