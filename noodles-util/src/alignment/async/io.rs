//! Async alignment format I/O.

pub mod reader;
pub mod writer;

pub use self::{reader::Reader, writer::Writer};
