//! The `camcraft-sim` program as a library (#2586): the `cl` subcommand
//! (a CLFile cut in a stock box, moved from the sim crate) and the
//! `check` subcommand (a posted 3-axis G-code file judged against a
//! machine file, a post manifest, the stock and the tools).
//! Provenance: our own command line over the sim, post and gcode crates.

pub mod check;
pub mod cl_cmd;

/// The exit code of a run: 0 each check ran and passed, 1 bad input,
/// 2 a check failed, 3 one or more checks were not judged.
pub const EXIT_OK: u8 = 0;
pub const EXIT_BAD_INPUT: u8 = 1;
pub const EXIT_FAILED: u8 = 2;
pub const EXIT_NOT_JUDGED: u8 = 3;

const USAGE: &str = "\
camcraft-sim - CamCraft simulation (headless)

USAGE:
    camcraft-sim cl [OPTIONS] <file.cl> <x0> <y0> <z0> <x1> <y1> <z1> <tool-diameter>
    camcraft-sim check <file.nc> --stock <box> --tools <T map> (--post <file.cps> | --dialect <name>)
                       [--machine <file.mch>] [--wcs G54=x,y,z]... [--fixture <box>]... [--cell <mm>]

    camcraft-sim cl --help       the CLFile stock simulation
    camcraft-sim check --help    the G-code check

EXIT:
    0  each check ran and passed
    1  bad input
    2  a check failed
    3  one or more checks were not judged (each is named)
";

/// Run the program with its arguments (the program name removed).
#[must_use]
pub fn run(args: &[String]) -> u8 {
    match args.first().map(String::as_str) {
        Some("cl") => cl_cmd::run(&args[1..]),
        Some("check") => check::run(&args[1..]),
        Some("-h" | "--help") => {
            print!("{USAGE}");
            EXIT_OK
        }
        Some("--version") => {
            println!("camcraft-sim {}", env!("CARGO_PKG_VERSION"));
            EXIT_OK
        }
        Some(other) => die(&format!("unknown subcommand {other:?}: use `cl` or `check` (try --help)")),
        None => die("expected a subcommand, `cl` or `check` (try --help)"),
    }
}

/// Print `msg` on stderr with the program name, and give the bad-input code.
pub(crate) fn die(msg: &str) -> u8 {
    eprintln!("camcraft-sim: {msg}");
    EXIT_BAD_INPUT
}

/// #163: the library's diagnostics are `tracing` events; print them on
/// stderr, filtered by `CAMCRAFT_LOG`, else `RUST_LOG`, else `info` - the
/// app binaries' rule. A second init is a no-op, never a panic.
pub fn log_init() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("CAMCRAFT_LOG")
        .or_else(|_| tracing_subscriber::EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr).try_init();
}
