//! doc-convert — convert between pdf/md/html/tex/docx.
//!
//! Two modes from one binary:
//!   doc-convert -i in.docx -t pdf -o out.pdf  # CLI (any input -> any output)
//!   doc-convert serve --port 8088             # HTTP API (deployed on the server)
//!
//! PDF input is extracted to Markdown in-process by pdf_oxide (pure CPU, no
//! GPU); every other input is read by pandoc directly. See `convert::run`.

mod cli;
mod client;
mod convert;
mod error;
mod extract;
mod progress;
mod serve;
mod tools;

use clap::Parser;
use cli::Args;
use error::{AppError, Result};
use progress::Reporter;
use std::fs;
use std::io::Write;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    // `serve` subcommand is handled before clap so the convert CLI stays flat.
    let raw: Vec<String> = std::env::args().collect();
    if raw.get(1).map(|s| s == "serve").unwrap_or(false) {
        let port = parse_port(&raw).unwrap_or(8088);
        return serve::run(port);
    }

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

fn parse_port(raw: &[String]) -> Option<u16> {
    raw.iter()
        .position(|a| a == "--port" || a == "-p")
        .and_then(|i| raw.get(i + 1))
        .and_then(|s| s.parse().ok())
}

fn run(args: &Args) -> Result<()> {
    let start = Instant::now();
    let rep = Reporter::from_args(args);

    if !args.input.exists() {
        return Err(AppError::Input(format!(
            "open input failed: {} does not exist",
            args.input.display()
        )));
    }
    let meta = fs::metadata(&args.input)
        .map_err(|e| AppError::Input(format!("open input failed: {e}")))?;
    if !meta.is_file() {
        return Err(AppError::Input(format!(
            "open input failed: {} is not a file",
            args.input.display()
        )));
    }
    if args.to.is_binary() && args.output.is_none() {
        return Err(AppError::Usage(format!(
            "--output is required for binary target '{}'",
            args.to.as_str()
        )));
    }

    let from = args
        .from
        .or_else(|| cli::Target::from_ext(&args.input))
        .ok_or_else(|| {
            AppError::Usage(format!(
                "cannot determine input format of {} — pass --from",
                args.input.display()
            ))
        })?;

    // Server mode: forward to the converter API; no local conversion.
    if let Some(api) = args
        .api_url
        .clone()
        .or_else(|| std::env::var("CONVERTER_API_URL").ok())
        .filter(|s| !s.trim().is_empty())
    {
        return client::forward(args, from, &api, &rep, start);
    }

    rep.phase("convert", &[("from", from.as_str()), ("to", args.to.as_str())]);
    let input = fs::read(&args.input).map_err(|e| AppError::Input(format!("read input: {e}")))?;
    let bytes = convert::run(&input, from, args.to, args.pdf_engine, args.standalone)?;
    emit(args, &bytes)?;

    rep.done(bytes.len(), start.elapsed().as_millis());
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
