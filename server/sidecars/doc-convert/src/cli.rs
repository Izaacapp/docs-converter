//! Command-line surface (clap derive).

use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "doc-convert",
    version,
    about = "Convert between PDF/Markdown/HTML/LaTeX/DOCX — pure Rust (pdf_oxide) for PDF input, pandoc for everything else."
)]
pub struct Args {
    /// Source file (pdf/md/html/tex/docx).
    #[arg(short, long)]
    pub input: PathBuf,

    /// Target format.
    #[arg(short, long, value_enum)]
    pub to: Target,

    /// Input format. Defaults to the input file's extension.
    #[arg(long, value_enum)]
    pub from: Option<Target>,

    /// Write output here. If omitted, text targets stream to stdout and binary
    /// targets (docx/pdf) are a usage error.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Convert via a doc-convert server instead of locally (overrides
    /// $CONVERTER_API_URL). The server does the work; no local CPU is used.
    #[arg(long)]
    pub api_url: Option<String>,

    /// Produce a full standalone document (pandoc -s) for html/tex.
    #[arg(long)]
    pub standalone: bool,

    /// LaTeX engine used for `--to pdf`.
    #[arg(long, value_enum, default_value_t = PdfEngine::Xelatex)]
    pub pdf_engine: PdfEngine,

    /// Emit machine-readable NDJSON progress on stderr.
    #[arg(long)]
    pub json_progress: bool,

    /// Suppress progress output (errors still print).
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Target {
    Md,
    Html,
    Tex,
    Docx,
    Pdf,
}

impl Target {
    pub fn is_binary(self) -> bool {
        matches!(self, Target::Docx | Target::Pdf)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Target::Md => "md",
            Target::Html => "html",
            Target::Tex => "tex",
            Target::Docx => "docx",
            Target::Pdf => "pdf",
        }
    }

    /// Pandoc reader name when this format is the *input*. PDF is handled by
    /// pdf_oxide (pandoc cannot read PDF), so its reader is never used.
    pub fn pandoc_reader(self) -> &'static str {
        match self {
            Target::Md => "markdown",
            Target::Html => "html",
            Target::Tex => "latex",
            Target::Docx => "docx",
            Target::Pdf => "pdf", // unused — PDF input goes through pdf_oxide
        }
    }

    /// Infer a format from a file extension (PDF/MD/HTML/TeX/DOCX).
    pub fn from_ext(path: &Path) -> Option<Target> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "pdf" => Some(Target::Pdf),
            "md" | "markdown" | "mdown" => Some(Target::Md),
            "html" | "htm" => Some(Target::Html),
            "tex" | "latex" => Some(Target::Tex),
            "docx" => Some(Target::Docx),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum PdfEngine {
    Xelatex,
    Lualatex,
    Pdflatex,
    /// Self-contained Rust XeTeX — single binary, fetches packages on demand.
    Tectonic,
}

impl PdfEngine {
    pub fn bin(self) -> &'static str {
        match self {
            PdfEngine::Xelatex => "xelatex",
            PdfEngine::Lualatex => "lualatex",
            PdfEngine::Pdflatex => "pdflatex",
            PdfEngine::Tectonic => "tectonic",
        }
    }
}
