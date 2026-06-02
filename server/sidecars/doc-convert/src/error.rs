//! Typed errors mapped to stable process exit codes.
//!
//! Exit codes (superset of pdf-extract's 0/1/2; never reused):
//!   0   success
//!   1   input missing/unreadable        (pdf-extract parity)
//!   2   PDF extraction/parse failed (pdf_oxide)
//!   4   a required external tool is missing (pandoc / a LaTeX engine / curl)
//!   5   conversion (pandoc/LaTeX) failed
//!   64  usage error (bad/missing flags)

use std::process::ExitCode;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Input(String),
    Extract(String),
    ToolMissing { tool: String, hint: String },
    Convert(String),
}

impl AppError {
    pub fn code(&self) -> u8 {
        match self {
            AppError::Input(_) => 1,
            AppError::Extract(_) => 2,
            AppError::ToolMissing { .. } => 4,
            AppError::Convert(_) => 5,
            AppError::Usage(_) => 64,
        }
    }
    pub fn exit(&self) -> ExitCode {
        ExitCode::from(self.code())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Usage(m) => write!(f, "usage: {m}"),
            AppError::Input(m) => write!(f, "input: {m}"),
            AppError::Extract(m) => write!(f, "extraction failed: {m}"),
            AppError::ToolMissing { tool, hint } => {
                write!(f, "required tool '{tool}' not found — {hint}")
            }
            AppError::Convert(m) => write!(f, "convert failed: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
