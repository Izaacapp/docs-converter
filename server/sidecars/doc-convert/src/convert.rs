//! Any-to-any document conversion. PDF input is extracted to Markdown by
//! pdf_oxide (pandoc can't read PDF); every other input is read by pandoc
//! directly. Output: md (text), html/tex (text), docx/pdf (binary).
//!
//! One entry point — `run(bytes, from, to, …)` — is shared by the CLI and the
//! HTTP server.

use crate::cli::{PdfEngine, Target};
use crate::error::{AppError, Result};
use crate::tools;
use std::path::Path;
use std::process::{Command, Stdio};
use unicode_normalization::UnicodeNormalization;

/// Pandoc input dialect for PDF-extracted Markdown: standard Markdown with the
/// "live passthrough" extensions disabled. Extracted prose routinely contains a
/// stray `\n`, `$`, or `<tag>` (e.g. a book quoting `"123 Main St,\nNot…"`).
/// With `raw_tex`/`tex_math`/`raw_html` on, pandoc treats those as live LaTeX,
/// math, or HTML — which makes the `pdf` target fail to compile (`Undefined
/// control sequence`) and silently *drops* the text from `html`/`docx`. Off,
/// they become escaped literal characters, so every target round-trips the text.
const PANDOC_FROM: &str = "markdown-raw_tex-raw_html-raw_attribute-tex_math_dollars-tex_math_single_backslash-tex_math_double_backslash";

/// Convert `input` (raw bytes of format `from`) into `to`, returning the bytes.
pub fn run(
    input: &[u8],
    from: Target,
    to: Target,
    pdf_engine: PdfEngine,
    standalone: bool,
) -> Result<Vec<u8>> {
    // PDF can't be read by pandoc — extract it to Markdown with pdf_oxide first.
    if matches!(from, Target::Pdf) {
        let mut md = crate::extract::to_markdown_bytes(input)?;
        if matches!(to, Target::Tex | Target::Pdf) {
            md = normalize_for_latex(&md);
        }
        if matches!(to, Target::Md) {
            return Ok(md.into_bytes());
        }
        return pandoc(md.as_bytes(), PANDOC_FROM, to, pdf_engine, standalone);
    }

    // Identical input and output formats: nothing to convert.
    if from == to {
        return Ok(input.to_vec());
    }

    // Markdown is always read as plain prose (the same safe dialect as
    // extracted text), so a stray `\n`/`$`/`<tag>` in a user's .md never
    // becomes live LaTeX and breaks `-> pdf`.
    let reader = if matches!(from, Target::Md) {
        PANDOC_FROM
    } else {
        from.pandoc_reader()
    };
    pandoc(input, reader, to, pdf_engine, standalone)
}

/// NFKC folds compatibility ligatures (U+FB01 ﬁ → "fi", …) that break pdflatex.
pub fn normalize_for_latex(md: &str) -> String {
    md.nfkc().collect()
}

/// Run pandoc: read `input` as `reader`, write `to`. Text targets are captured
/// from stdout; binary targets (docx/pdf) are written to a temp file and read
/// back. The input always goes through a temp file so binary sources (docx) and
/// text sources are handled the same way — and there's no stdin/stdout deadlock.
fn pandoc(
    input: &[u8],
    reader: &str,
    to: Target,
    pdf_engine: PdfEngine,
    standalone: bool,
) -> Result<Vec<u8>> {
    tools::require("pandoc", tools::PANDOC_HINT)?;
    let src = tmpfile("")?;
    std::fs::write(src.path(), input)
        .map_err(|e| AppError::Convert(format!("write tmp input: {e}")))?;

    let s = if standalone {
        vec!["-s".to_string()]
    } else {
        Vec::new()
    };
    match to {
        Target::Md => capture(src.path(), reader, "markdown", &s),
        Target::Html => capture(src.path(), reader, "html5", &s),
        Target::Tex => capture(src.path(), reader, "latex", &s),
        Target::Docx => {
            let out = tmpfile(".docx")?;
            to_file(src.path(), reader, "docx", out.path(), &[])?;
            std::fs::read(out.path()).map_err(|e| AppError::Convert(format!("read docx: {e}")))
        }
        Target::Pdf => {
            tools::require(pdf_engine.bin(), tools::LATEX_HINT)?;
            let out = tmpfile(".pdf")?;
            let extra = [
                format!("--pdf-engine={}", pdf_engine.bin()),
                "-V".to_string(),
                "geometry:margin=1in".to_string(),
            ];
            to_file(src.path(), reader, "", out.path(), &extra)?;
            std::fs::read(out.path()).map_err(|e| AppError::Convert(format!("read pdf: {e}")))
        }
    }
}

/// pandoc with stdout captured (text targets: md/html/tex).
fn capture(input: &Path, reader: &str, writer: &str, extra: &[String]) -> Result<Vec<u8>> {
    let mut c = Command::new("pandoc");
    c.arg(input).arg("-f").arg(reader).arg("-t").arg(writer);
    for e in extra {
        c.arg(e);
    }
    c.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = c
        .spawn()
        .map_err(|e| AppError::Convert(format!("pandoc spawn: {e}")))?
        .wait_with_output()
        .map_err(|e| AppError::Convert(format!("pandoc wait: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Convert(format!(
            "pandoc -t {writer}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out.stdout)
}

/// pandoc writing to a file (binary targets: docx/pdf; `writer` empty = infer
/// from the output extension, used for pdf via `--pdf-engine`).
fn to_file(input: &Path, reader: &str, writer: &str, output: &Path, extra: &[String]) -> Result<()> {
    let mut c = Command::new("pandoc");
    c.arg(input).arg("-f").arg(reader);
    if !writer.is_empty() {
        c.arg("-t").arg(writer);
    }
    c.arg("-o").arg(output);
    for e in extra {
        c.arg(e);
    }
    c.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let out = c
        .spawn()
        .map_err(|e| AppError::Convert(format!("pandoc spawn: {e}")))?
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

fn tmpfile(suffix: &str) -> Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .map_err(|e| AppError::Convert(format!("tempfile: {e}")))
}
