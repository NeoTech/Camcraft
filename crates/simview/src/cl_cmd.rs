//! `camcraft-sim cl` - headless stock simulation (M13.3): cuts a box of
//! stock with a CLFile's moves through the `Stock` seam on the field
//! engine (#588, M7.3 - the column engine is being retired) and prints
//! the removed volume and a rapid/depth verdict, read column by column
//! through `Columns`. Provenance: height-field stock removal is general
//! CAM verification practice from published literature, as the
//! `stock_sim` module states; the CLI is ours. Moved from the sim crate's
//! binary to this subcommand by #2586, with the same arguments and exits.

use camcraft_sim::cl::{text, MoveKind};
use camcraft_sim::stock_sim::{auto_cell, Backend, SimOp, Stock, FIELD_BAND};
use crate::{die, EXIT_FAILED, EXIT_OK};

const USAGE: &str = "\
camcraft-sim cl - CamCraft stock simulation (headless)

USAGE:
    camcraft-sim cl [OPTIONS] <file.cl> <x0> <y0> <z0> <x1> <y1> <z1> <tool-diameter>

Cuts the stock box x0..x1, y0..y1, z0..z1 (mm) with the CLFile's moves
using a flat end mill (or --ball) of the given diameter, then prints
the removed volume and the verdict.

EXIT:
    0  simulated, no flags
    1  bad input (unreadable or malformed file, nonsense numbers)
    2  simulated, verdict flagged (a rapid that travels sideways or
       descends below the stock top inside the stock footprint, or a
       cut below the stock bottom)

OPTIONS:
    --ball            ball-nose tool instead of a flat end mill
    --cell <mm>       grid cell size (default: from the tool and box)
    -h, --help        print this text
    --version         print the version
";

/// Run the subcommand with its arguments (the word `cl` removed).
#[must_use]
pub fn run(args: &[String]) -> u8 {
    let mut ball = false;
    let mut cell: Option<String> = None;
    let mut pos: Vec<String> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return EXIT_OK;
            }
            "--version" => {
                println!("camcraft-sim {}", env!("CARGO_PKG_VERSION"));
                return EXIT_OK;
            }
            "--ball" => ball = true,
            "--cell" => match it.next() {
                Some(v) => cell = Some(v.clone()),
                None => return die("--cell needs a value"),
            },
            o if o.starts_with("--") => return die(&format!("unknown option {o:?} (try --help)")),
            o => pos.push(o.to_string()),
        }
    }
    let [path, rest @ ..] = pos.as_slice() else {
        return die("expected a CLFile path and seven numbers (try --help)");
    };
    if rest.len() != 7 {
        return die("expected a CLFile path and seven numbers (try --help)");
    }
    let mut n = [0.0f64; 7];
    for (slot, s) in n.iter_mut().zip(rest) {
        match s.parse::<f64>() {
            Ok(v) if v.is_finite() => *slot = v,
            _ => return die(&format!("not a finite number: {s:?}")),
        }
    }
    let [x0, y0, z0, x1, y1, z1, d] = n;
    if !(x1 > x0 && y1 > y0 && z1 > z0) {
        return die("stock box needs x0 < x1, y0 < y1, z0 < z1");
    }
    if d <= 0.0 {
        return die("tool diameter must be positive");
    }
    let r = d / 2.0;
    let cell = match cell.as_deref().map(str::parse::<f64>) {
        None => auto_cell(r, x1 - x0, y1 - y0),
        Some(Ok(c)) if c.is_finite() && c > 0.0 => c,
        Some(_) => return die("--cell must be a positive number"),
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return die(&format!("cannot read {path}: {e}")),
    };
    let t = match text::from_text(&src) {
        Ok(t) => t,
        Err(e) => return die(&format!("cannot parse {path}: {e}")),
    };
    let mut stock = match Stock::block(Backend::Field { band: FIELD_BAND }, x0, y0, x1, y1, z1, cell) {
        Ok(s) => s,
        Err(e) => return die(&e.to_string()),
    };
    let op = SimOp { moves: &t.moves, radius: r, ball, cone: None, corner: None };
    if let Err(e) = stock.simulate(&[op]) {
        return die(&e.to_string());
    }
    let cols = match stock.columns() {
        Ok(c) => c,
        Err(e) => return die(&e.to_string()),
    };
    // Removed volume: per column, the stock surface down to what stands,
    // floored at the stock bottom (the stock has no floor of its own). A
    // column the field gives no verdict for (NaN) counts as uncut - the
    // same abstention every column consumer makes (`Columns`).
    let a2 = cols.cell() * cols.cell();
    let (mut removed, mut cut_cells, mut lowest) = (0.0f64, 0usize, z1);
    for iy in 0..cols.ny() {
        for ix in 0..cols.nx() {
            let top = cols.top_at(ix, iy);
            if top.is_finite() && top < z1 {
                removed += (z1 - top.max(z0)) * a2;
                cut_cells += 1;
                lowest = lowest.min(top);
            }
        }
    }
    let inside = |x: f64, y: f64| x > x0 - r && x < x1 + r && y > y0 - r && y < y1 + r;
    let bad_rapids = t
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Rapid)
        .filter(|m| {
            // A straight-up retract leaves through the air it cut; a
            // rapid that travels sideways or descends below the stock
            // top inside the footprint may meet material at rapid feed.
            let sideways = m.from.x != m.to.x || m.from.y != m.to.y;
            let below = m.from.z.min(m.to.z) < z1;
            below
                && (sideways || m.to.z < m.from.z)
                && (inside(m.from.x, m.from.y) || inside(m.to.x, m.to.y))
        })
        .count();
    let through = lowest < z0;
    println!("file: {path}");
    println!("moves: {}", t.len());
    println!("grid: {} x {} cells of {:.4} mm", cols.nx(), cols.ny(), cols.cell());
    println!("stock volume: {:.3} mm^3", (x1 - x0) * (y1 - y0) * (z1 - z0));
    println!("removed volume: {removed:.3} mm^3");
    println!("cut cells: {cut_cells}");
    println!("lowest cut: {lowest:.4}");
    println!("rapids into the stock: {bad_rapids}");
    println!("cut below stock bottom: {through}");
    if bad_rapids > 0 || through {
        println!("verdict: FLAGGED");
        return EXIT_FAILED;
    }
    println!("verdict: ok (rapid and depth check only - no part, no gouge verdict)");
    EXIT_OK
}
