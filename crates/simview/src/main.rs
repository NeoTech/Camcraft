//! `camcraft-sim`: the stand-alone simulation program (#2574, #2586).
//! Provenance: our own command line; the subcommands name their sources.

use std::process::ExitCode;

fn main() -> ExitCode {
    camcraft_simview::log_init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(camcraft_simview::run(&args))
}
