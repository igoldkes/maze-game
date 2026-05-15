//! Persistence-oriented accessors for maze internals.
//! Keep non-generation data access methods here to avoid bloating `maze/mod.rs`.

use super::Maze;

impl Maze {
    /// Raw row-major wall-bit storage for persistence/replay.
    pub fn cells(&self) -> &[u8] {
        &self.cells
    }
}
