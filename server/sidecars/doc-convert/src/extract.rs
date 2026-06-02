//! In-process PDF -> Markdown via **pdf_oxide** (the fastest Rust PDF crate —
//! the same engine Cupids-Sniper's pdf-extract uses). No server, no Python.
//!
//! Scanned PDFs that carry a text layer (e.g. Internet-Archive OCR) extract
//! fine; pure image-only OCR would need a Rust OCR crate (a future add).

use crate::error::{AppError, Result};
use pdf_oxide::converters::ConversionOptions;
use pdf_oxide::PdfDocument;
use std::path::Path;

pub fn to_markdown(pdf: &Path) -> Result<String> {
    let path = pdf.to_string_lossy();
    let doc = PdfDocument::open(path.as_ref())
        .map_err(|e| AppError::Extract(format!("open pdf: {e}")))?;

    let options = ConversionOptions {
        detect_headings: true,
        include_images: false,
        embed_images: false,
        ..Default::default()
    };
    let md = doc
        .to_markdown_all(&options)
        .map_err(|e| AppError::Extract(format!("pdf_oxide markdown: {e}")))?;
    Ok(strip_control_chars(&md))
}

/// Extract a PDF given as raw bytes — validates the `%PDF` magic, then writes a
/// temp file because pdf_oxide opens by path. Used by the unified `convert::run`
/// path (CLI in-process + HTTP server).
pub fn to_markdown_bytes(pdf: &[u8]) -> Result<String> {
    if pdf.len() < 5 || &pdf[..4] != b"%PDF" {
        return Err(AppError::Input("input is not a PDF".into()));
    }
    let tmp = tempfile::Builder::new()
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| AppError::Extract(format!("tempfile: {e}")))?;
    std::fs::write(tmp.path(), pdf)
        .map_err(|e| AppError::Extract(format!("write tmp pdf: {e}")))?;
    to_markdown(tmp.path())
}

/// PDF text extraction occasionally emits stray control bytes — a BEL (0x07)
/// from a mis-mapped glyph, C1 controls from bad decoding. They are invisible
/// noise in the markdown and make LaTeX abort ("Text line contains an invalid
/// character"), so drop every control char except tab / newline / return.
fn strip_control_chars(s: &str) -> String {
    if s.chars().all(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r')) {
        return s.to_string();
    }
    s.chars()
        .filter(|&c| !c.is_control() || matches!(c, '\t' | '\n' | '\r'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::strip_control_chars;

    #[test]
    fn drops_control_bytes_keeps_whitespace_and_text() {
        assert_eq!(strip_control_chars("Sentence to 2a.\u{0007} The store"), "Sentence to 2a. The store");
        assert_eq!(strip_control_chars("a\tb\nc\r\nd"), "a\tb\nc\r\nd"); // tab/nl/cr survive
        assert_eq!(strip_control_chars("x\u{0085}y\u{0000}z"), "xyz"); // C1 NEL + NUL go
        assert_eq!(strip_control_chars("plain café — 123"), "plain café — 123"); // unicode prose untouched
    }
}
