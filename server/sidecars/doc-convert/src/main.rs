//! doc-convert — PDF -> md/html/json/tex/docx/pdf.
//!
//! Pipeline: Docling (the engine — OCR, tables, reading order) produces the
//! canonical intermediate (md/html/json); md/html/json are emitted directly,
//! while tex/docx/pdf go Markdown -> pandoc (+ LaTeX engine for pdf).
//!
//! Contract (mirrors Cupids-Sniper/server/sidecars/pdf-extract):
//!   payload -> stdout, progress/diagnostics -> stderr, typed exit codes.

mod cli;
mod convert;
mod engine;
mod error;
mod progress;
mod tools;
mod workdir;

use clap::Parser;
use cli::Args;
use error::{AppError, Result};
use progress::Reporter;
use std::fs;
use std::io::Write;
use std::process::ExitCode;
use std::time::Instant;
use workdir::WorkDir;

fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::from(64)
            } else {
                ExitCode::SUCCESS
            };
        }
    };

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("doc-convert: {e}");
            e.exit()
        }
    }
}

fn run(args: &Args) -> Result<()> {
    let start = Instant::now();
    let rep = Reporter::from_args(args);

    if !args.input.exists() {
        return Err(AppError::Input(format!(
            "open pdf failed: {} does not exist",
            args.input.display()
        )));
    }
    let meta = fs::metadata(&args.input)
        .map_err(|e| AppError::Input(format!("open pdf failed: {e}")))?;
    if !meta.is_file() {
        return Err(AppError::Input(format!(
            "open pdf failed: {} is not a file",
            args.input.display()
        )));
    }
    if args.to.is_binary() && args.output.is_none() {
        return Err(AppError::Usage(format!(
            "--output is required for binary target '{}'",
            args.to.as_str()
        )));
    }

    let work = WorkDir::new()?;

    let n = if args.to.is_native_docling() {
        // md / html / json — straight from Docling.
        let bytes = engine::run(&work, args, args.to.docling_format(), &rep)?;
        emit(args, &bytes)?;
        bytes.len()
    } else {
        // tex / docx / pdf — Docling Markdown, then pandoc.
        let md_bytes = engine::run(&work, args, "md", &rep)?;
        let md = String::from_utf8_lossy(&md_bytes).into_owned();
        rep.phase("convert", &[("target", args.to.as_str())]);
        match convert::run(&md, args)? {
            Some(bytes) => {
                emit(args, &bytes)?;
                bytes.len()
            }
            None => args
                .output
                .as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .map(|m| m.len() as usize)
                .unwrap_or(0),
        }
    };

    rep.done(n, start.elapsed().as_millis());
    Ok(())
}

fn emit(args: &Args, bytes: &[u8]) -> Result<()> {
    match &args.output {
        Some(p) => fs::write(p, bytes)
            .map_err(|e| AppError::Convert(format!("write {}: {e}", p.display()))),
        None => {
            let mut out = std::io::stdout().lock();
            out.write_all(bytes)
                .map_err(|e| AppError::Convert(format!("write stdout: {e}")))
        }
    }
}
