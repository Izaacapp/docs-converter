//! Markdown -> {md,html,tex,docx,pdf}. `md` is passthrough; html/tex via
//! pandoc to stdout; docx/pdf via pandoc to a temp file (then read back as
//! bytes). LaTeX targets NFKC-fold compatibility ligatures (ﬁ/ﬂ → fi/fl).
//!
//! Returning bytes for every target lets both the CLI and the HTTP server share
//! one code path.

use crate::cli::{PdfEngine, Target};
use crate::error::{AppError, Result};
use crate::tools;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use unicode_normalization::UnicodeNormalization;

pub fn to_bytes(md: &str, to: Target, pdf_engine: PdfEngine, standalone: bool) -> Result<Vec<u8>> {
    match to {
        Target::Md => Ok(md.as_bytes().to_vec()),

        Target::Html => {
            let mut extra: Vec<String> = Vec::new();
            if standalone {
                extra.push("-s".into());
            }
            pandoc_stdout(md, "html5", &extra)
        }

        Target::Tex => {
            let norm = normalize_for_latex(md);
            let mut extra: Vec<String> = Vec::new();
            if standalone {
                extra.push("-s".into());
            }
            pandoc_stdout(&norm, "latex", &extra)
        }

        Target::Docx => {
            let tmp = tmpfile(".docx")?;
            pandoc_file(md, "docx", tmp.path(), &[])?;
            std::fs::read(tmp.path()).map_err(|e| AppError::Convert(format!("read docx: {e}")))
        }

        Target::Pdf => {
            tools::require(pdf_engine.bin(), tools::LATEX_HINT)?;
            let body = if matches!(pdf_engine, PdfEngine::Pdflatex) {
                normalize_for_latex(md)
            } else {
                md.to_string()
            };
            let tmp = tmpfile(".pdf")?;
            let extra = vec![
                format!("--pdf-engine={}", pdf_engine.bin()),
                "-V".into(),
                "geometry:margin=1in".into(),
            ];
            pandoc_file(&body, "", tmp.path(), &extra)?;
            std::fs::read(tmp.path()).map_err(|e| AppError::Convert(format!("read pdf: {e}")))
        }
    }
}

fn tmpfile(suffix: &str) -> Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .map_err(|e| AppError::Convert(format!("tempfile: {e}")))
}

/// NFKC folds compatibility ligatures (U+FB01 ﬁ → "fi", …) that break pdflatex.
pub fn normalize_for_latex(md: &str) -> String {
    md.nfkc().collect()
}

fn pandoc_stdout(md: &str, to: &str, extra: &[String]) -> Result<Vec<u8>> {
    tools::require("pandoc", tools::PANDOC_HINT)?;
    let mut c = Command::new("pandoc");
    c.arg("-f").arg("markdown").arg("-t").arg(to);
    for e in extra {
        c.arg(e);
    }
    c.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c
        .spawn()
        .map_err(|e| AppError::Convert(format!("pandoc spawn: {e}")))?;
    write_stdin(&mut child, md);
    let out = child
        .wait_with_output()
        .map_err(|e| AppError::Convert(format!("pandoc wait: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Convert(format!(
            "pandoc -t {to}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out.stdout)
}

fn pandoc_file(md: &str, to: &str, output: &Path, extra: &[String]) -> Result<()> {
    tools::require("pandoc", tools::PANDOC_HINT)?;
    let mut c = Command::new("pandoc");
    c.arg("-f").arg("markdown");
    if !to.is_empty() {
        c.arg("-t").arg(to);
    }
    c.arg("-o").arg(output);
    for e in extra {
        c.arg(e);
    }
    c.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = c
        .spawn()
        .map_err(|e| AppError::Convert(format!("pandoc spawn: {e}")))?;
    write_stdin(&mut child, md);
    let out = child
        .wait_with_output()
        .map_err(|e| AppError::Convert(format!("pandoc wait: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Convert(format!(
            "pandoc -> {}: {}",
            output.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

/// Feed `md` to pandoc's stdin on a thread so we can drain stdout/stderr
/// concurrently — avoids a pipe-buffer deadlock on big books.
fn write_stdin(child: &mut std::process::Child, md: &str) {
    if let Some(mut si) = child.stdin.take() {
        let owned = md.to_string();
        std::thread::spawn(move || {
            let _ = si.write_all(owned.as_bytes());
        });
    }
}
