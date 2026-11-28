//! camcraft-ui-kit: the design system of the camcraft shell (#4394, step 0
//! of the #4245 plan) - the theme token table and its clamps, the
//! `camcraft-theme/1` file reader, the contrast arithmetic and its
//! readability gate, and a gallery that shows them. The crate depends on
//! egui and serde_json only: it cannot see the document, so a component
//! here takes plain data and returns the user's choice, and `camcraft-ui`
//! turns that choice into an undoable command.
//! Provenance: our own crate, split out of `camcraft-ui` at the
//! maintainer's direction (#4245); no HSMWorks material.

pub mod contrast;
pub mod fold;
pub mod gallery;
pub mod theme;
pub mod theme_files;
