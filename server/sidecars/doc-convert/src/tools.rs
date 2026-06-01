//! External-tool probing. Missing tools become a clean exit-4 with an
//! actionable hint instead of a cryptic spawn failure.

use crate::error::{AppError, Result};

pub fn have(bin: &str) -> bool {
    which::which(bin).is_ok()
}

pub fn require(bin: &str, hint: &str) -> Result<()> {
    if have(bin) {
        Ok(())
    } else {
        Err(AppError::ToolMissing {
            tool: bin.to_string(),
            hint: hint.to_string(),
        })
    }
}

pub const DOCLING_HINT: &str =
    "install docling (pip install docling) or set DOCLING_SERVE_URL to a docling-serve instance";
pub const PANDOC_HINT: &str = "install pandoc (brew install pandoc / apt install pandoc)";
pub const CURL_HINT: &str = "curl is required to reach docling-serve";
pub const LATEX_HINT: &str = "install a LaTeX engine (TeX Live / MacTeX) for --to pdf";
