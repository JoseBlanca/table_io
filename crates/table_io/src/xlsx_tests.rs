//! The tests of the step of the cells of an xlsx, `read_first_sheet` and
//! its types, of `docs/specs/read.md`. They were the integration tests of
//! `tests/` while the module `xlsx` was public, and are tests of the
//! library since it is private to it; the files they share, under
//! `tests/hand_written/` and `tests/data/`, stay where they were.

#[path = "../tests/hand_written/mod.rs"]
pub(crate) mod hand_written;

mod cells;
mod data_files;
mod dates;
mod merged;
mod no_panic;
mod owner_files;
mod parts;
mod refusals;
mod sheet;
