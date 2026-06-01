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
    doc.to_markdown_all(&options)
        .map_err(|e| AppError::Extract(format!("pdf_oxide markdown: {e}")))
}
