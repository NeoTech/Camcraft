//! `camcraft-sim check` (#2586): a posted 3-axis G-code file judged
//! headless against a machine file, a post manifest, a stock box and
//! the tools. Each check passes, fails with NC line numbers, or is named
//! as not judged; the exit code says which (`crate::EXIT_*`).
//! Provenance: our own command over the gcode, post and sim crates.

pub mod args;
mod fixtures;
pub mod frames;
pub mod inputs;
mod limits;
mod model;
pub mod program;
pub mod report;
mod sat;
mod stock;

use crate::{die, EXIT_OK};
use args::{Args, Parsed};
use inputs::{MachineFile, PostManifest};
use program::Program;
use report::Report;

const POST: &str = "post";

/// Run the subcommand with its arguments (the word `check` removed).
#[must_use]
pub fn run(raw: &[String]) -> u8 {
    let a = match args::parse(raw) {
        Ok(Parsed::Run(a)) => a,
        Ok(Parsed::Help) => {
            print!("{}", args::USAGE);
            return EXIT_OK;
        }
        Err(e) => return die(&e),
    };
    match judge(&a) {
        Ok(report) => {
            println!("{}", report.text());
            report.exit_code()
        }
        Err(e) => die(&e),
    }
}

/// Read the inputs and run each check. An `Err` is bad input.
fn judge(a: &Args) -> Result<Report, String> {
    let post = a.post.as_deref().map(inputs::read_post).transpose()?;
    let machine = a.machine.as_deref().map(inputs::read_machine).transpose()?;
    if let (Some(p), Some(m)) = (&post, &machine)
        && !inputs::serves(p, m.kind)
    {
        return Err(format!("post {} does not serve the {:?} machine {} (its serves: {:?})", p.name, m.kind, m.name, p.serves));
    }
    let dialect = inputs::choose_dialect(a.dialect.as_deref(), post.as_ref())?;
    let dialect_name = dialect.name.clone();
    let text = std::fs::read_to_string(&a.program).map_err(|e| format!("cannot read {}: {e}", a.program))?;
    let program = program::read(&text, dialect, &a.wcs).map_err(|e| format!("{}: {e}", a.program))?;
    // An unmapped T refuses: a guessed tool gives a wrong verdict (#2213).
    for s in &program.sections {
        if let Some(n) = s.tool
            && !a.tools.contains_key(&n)
        {
            let line = s.lines.first().copied().unwrap_or_default();
            return Err(format!("{}: line {line}: the program uses T{n}, and --tools does not state it", a.program));
        }
    }

    let mut r = Report::default();
    r.note(&format!("program: {} ({} sections, dialect {dialect_name})", a.program, program.sections.len()));
    header(&mut r, post.as_ref(), machine.as_ref());
    if let Some(line) = program.first_line {
        r.note(&format!("line {line}: the first move starts where the machine stands; only its end point is judged"));
    }
    post_checks(&program, post.as_ref(), machine.as_ref(), &mut r);
    let skipped = program.skipped_lines();
    if !skipped.is_empty() {
        let list: Vec<String> = skipped.iter().map(ToString::to_string).collect();
        r.not_judged(
            "reference moves",
            &format!(
                "lines {}: a G28, G30 or G53 move (or the move after it) in a work offset with no --wcs has no known position",
                list.join(", ")
            ),
        );
    }
    stock::check(&program, a.stock, &a.tools, a.cell, &mut r);
    fixtures::check(&program, &a.fixtures, &a.tools, &mut r);
    limits::check(&program, machine.as_ref(), &mut r);
    model::check(&program, machine.as_ref(), &mut r);
    Ok(r)
}

fn header(r: &mut Report, post: Option<&PostManifest>, machine: Option<&MachineFile>) {
    if let Some(p) = post {
        let units = p.units.as_ref().map_or_else(|| "not stated".to_owned(), |u| u.join(" "));
        r.note(&format!(
            "post: {} (extension {}, programUnits {units}, work offsets 1..{})",
            p.name, p.extension, p.work_offset_max
        ));
    }
    if let Some(m) = machine {
        r.note(&format!("machine: {} ({:?})", m.name, m.kind));
    }
}

/// What the post manifest states against what the program asks: the
/// machine kind, the program units and the work offsets.
fn post_checks(program: &Program, post: Option<&PostManifest>, machine: Option<&MachineFile>, r: &mut Report) {
    let Some(p) = post else {
        return r.not_judged(POST, "no post is given (--post): the units, work offsets and machine kind are not judged");
    };
    let before = r.failed();
    let units = p.units.clone().unwrap_or_else(|| vec!["mm".to_owned()]);
    for (asks, word, g) in [(program.inch, "inch", "G20"), (program.mm, "mm", "G21")] {
        if asks && !units.iter().any(|u| u == word) {
            r.fail(POST, &format!("the program states {g} ({word}), and {} writes {} programs only", p.name, units.join(" ")));
        }
    }
    for s in &program.sections {
        if u32::from(s.offset) > p.work_offset_max {
            let line = s.lines.first().copied().unwrap_or_default();
            r.fail(
                POST,
                &format!("line {line}: the program uses G{}, and {} posts work offsets 1..{} only", 53 + s.offset, p.name, p.work_offset_max),
            );
        }
    }
    match machine {
        Some(m) => r.note(&format!("{} serves the {:?} machine {}", p.name, m.kind, m.name)),
        None => r.not_judged(POST, "no machine file is given (--machine): the machine kind that the post serves is not judged"),
    }
    if !r.failed() || before {
        r.pass(POST, &format!("the units and the work offsets agree with {}", p.name));
    }
}
