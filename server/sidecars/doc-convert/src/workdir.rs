//! Scratch directory for Docling's output files and pandoc intermediates.
//! Auto-removed on drop.

use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct WorkDir {
    dir: TempDir,
}

impl WorkDir {
    pub fn new() -> Result<Self> {
        let dir = TempDir::new().map_err(|e| AppError::Extract(format!("tempdir: {e}")))?;
        Ok(WorkDir { dir })
    }
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
    #[allow(dead_code)]
    pub fn join(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
}
