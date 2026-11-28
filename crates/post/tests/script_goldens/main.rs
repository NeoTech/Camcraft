//! The `script_goldens` suite of `camcraft-post` (#3188): the recorded edge-case streams through every shipped post script, each program or named refusal compared with its golden.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

mod recorded_streams;
mod script_agie_native;
mod script_carvera;
mod script_doosan_dnm;
mod script_fanuc;
mod script_fanuc_lathe;
mod script_g_edm;
mod script_grbl;
mod script_grblhal;
mod script_haas;
mod script_heidenhain;
mod script_heidenhain_klartext;
mod script_linuxcnc;
mod script_mach3;
mod script_mach4;
mod script_mazak;
mod script_richauto;
mod script_sandbox;
mod script_sinker_iso;
mod script_sinker_side;
mod script_sinumerik;
mod script_sinumerik_810m;
mod script_sodick_ln2w;
mod script_sodick_wire;
mod script_tinyg;
mod script_tormach;
mod script_tormach_lathe;
mod script_tormach_plasma;
mod script_waterjet_iso;
mod script_wire_iso;
