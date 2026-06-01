//! Markdown -> {tex, docx, pdf} via pandoc (+ a LaTeX engine for pdf).
//!
//! md / html / json come straight from Docling and never reach this module —
//! pandoc only ever runs the Markdown->{tex,docx,pdf} leg. For LaTeX targets
//! we NFKC-fold compatibility ligatures (ﬁ/ﬂ → fi/fl) that break pdflatex;
//! xelatex/lualatex are Unicode-native.

use crate::cli::{Args, PdfEngine, Target};
use crate::error::{AppError, Result};
use crate::tools;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use unicode_normalization::UnicodeNormalization;

/// Returns Some(bytes) for text targets (caller emits) or None for binary
/// targets (already written to `--output`).
pub fn run(md: &str, args: &Args) -> Result<Option<Vec<u8>>> {
    match args.to {
        Target::Tex => {
            let norm = normalize_for_latex(md);
            let mut extra: Vec<String> = Vec::new();
            if args.standalone {
                extra.push("-s".into());
            }
            Ok(Some(pandoc_stdout(&norm, "latex", &extra)?))
        }
        Target::Docx => {
            let out = require_output(args)?;
            pandoc_file(md, "docx", out, &[])?;
            Ok(None)
        }
        Target::Pdf => {
            let out = require_output(args)?;
            tools::require(args.pdf_engine.bin(), tools::LATEX_HINT)?;
            let body = if matches!(args.pdf_engine, PdfEngine::Pdflatex) {
                normalize_for_latex(md)
            } else {
                md.to_string()
            };
            let extra = vec![
                format!("--pdf-engine={}", args.pdf_engine.bin()),
                "-V".into(),
                "geometry:margin=1in".into(),
            ];
            pandoc_file(&body, "", out, &extra)?;
            Ok(None)
        }
        other => Err(AppError::Usage(format!(
            "convert::run called for non-pandoc target '{}'",
            other.as_str()
        ))),
    }
}

fn require_output(args: &Args) -> Result<&Path> {
    match &args.output {
        Some(p) => Ok(p.as_path()),
        None => Err(AppError::Usage(format!(
            "--output is required for binary target '{}'",
            args.to.as_str()
        ))),
    }
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

/// Write `md` to the child's stdin on a thread so `wait_with_output` can drain
/// stdout/stderr concurrently — avoids a pipe-buffer deadlock on big books.
fn write_stdin(child: &mut std::process::Child, md: &str) {
    if let Some(mut si) = child.stdin.take() {
        let owned = md.to_string();
        std::thread::spawn(move || {
            let _ = si.write_all(owned.as_bytes());
        });
    }
}
