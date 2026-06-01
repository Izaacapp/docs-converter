//! Command-line surface (clap derive).

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "doc-convert",
    version,
    about = "Convert a PDF to Markdown/HTML/JSON/LaTeX/DOCX/PDF via Docling (OCR + tables built in), pandoc for the docx/tex/pdf leg."
)]
pub struct Args {
    /// Source PDF file.
    #[arg(short, long)]
    pub input: PathBuf,

    /// Target format.
    #[arg(short, long, value_enum)]
    pub to: Target,

    /// OCR strategy. auto = let Docling decide; force = re-OCR every page; off = no OCR.
    #[arg(long, value_enum, default_value_t = OcrMode::Auto)]
    pub ocr: OcrMode,

    /// OCR language(s), comma-separated (e.g. "en" or "en,fr"). Forwarded to Docling.
    #[arg(long, default_value = "en")]
    pub ocr_lang: String,

    /// Override Docling's OCR engine (e.g. easyocr, tesserocr, rapidocr). Omit to use Docling's default.
    #[arg(long)]
    pub ocr_engine: Option<String>,

    /// Disable table-structure recognition (faster).
    #[arg(long)]
    pub no_tables: bool,

    /// Torch device for the local Docling CLI. Default "cpu" avoids the
    /// Apple-Silicon MPS float64 bug; use "cuda" on a GPU box.
    #[arg(long, default_value = "cpu")]
    pub device: String,

    /// How images are handled in md/html output.
    #[arg(long, value_enum, default_value_t = ImageMode::Placeholder)]
    pub image_mode: ImageMode,

    /// Which Docling backend to use.
    #[arg(long, value_enum, default_value_t = EngineKind::Auto)]
    pub engine: EngineKind,

    /// docling-serve base URL (overrides $DOCLING_SERVE_URL), e.g. http://homelab:5001
    #[arg(long)]
    pub serve_url: Option<String>,

    /// Write output here. If omitted, text targets stream to stdout; binary targets (docx/pdf) error.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

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
    Json,
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
            Target::Json => "json",
            Target::Tex => "tex",
            Target::Docx => "docx",
            Target::Pdf => "pdf",
        }
    }
    /// True when Docling can emit this format directly (no pandoc leg).
    pub fn is_native_docling(self) -> bool {
        matches!(self, Target::Md | Target::Html | Target::Json)
    }
    /// The Docling output format to request as the canonical intermediate.
    pub fn docling_format(self) -> &'static str {
        match self {
            Target::Html => "html",
            Target::Json => "json",
            // tex/docx/pdf go through Markdown -> pandoc
            _ => "md",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OcrMode {
    Auto,
    Force,
    Off,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ImageMode {
    Placeholder,
    Embedded,
    Referenced,
}

impl ImageMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ImageMode::Placeholder => "placeholder",
            ImageMode::Embedded => "embedded",
            ImageMode::Referenced => "referenced",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum EngineKind {
    /// Use docling-serve if a URL is configured, else the local CLI.
    Auto,
    /// Force the local `docling` CLI.
    Cli,
    /// Force a remote docling-serve instance.
    Serve,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum PdfEngine {
    Xelatex,
    Lualatex,
    Pdflatex,
}

impl PdfEngine {
    pub fn bin(self) -> &'static str {
        match self {
            PdfEngine::Xelatex => "xelatex",
            PdfEngine::Lualatex => "lualatex",
            PdfEngine::Pdflatex => "pdflatex",
        }
    }
}
