//! Document-understanding engine = **Docling** (2026 state of the art).
//!
//! Two backends, same output contract (returns the requested format's bytes):
//!   * `cli`   — local `docling` binary (set $DOCLING_BIN to point at a venv).
//!   * `serve` — remote docling-serve (`$DOCLING_SERVE_URL`, the homelab).
//!
//! OCR, table structure and reading order are Docling's job — the sidecar
//! never touches tesseract/poppler directly.

use crate::cli::{Args, EngineKind, OcrMode};
use crate::error::{AppError, Result};
use crate::progress::Reporter;
use crate::tools;
use crate::workdir::WorkDir;
use std::process::{Command, Stdio};

/// Produce the Docling output for `fmt` ("md" | "html" | "json").
pub fn run(work: &WorkDir, args: &Args, fmt: &str, rep: &Reporter) -> Result<Vec<u8>> {
    let serve = args
        .serve_url
        .clone()
        .or_else(|| std::env::var("DOCLING_SERVE_URL").ok())
        .filter(|s| !s.trim().is_empty());

    let use_serve = match args.engine {
        EngineKind::Serve => true,
        EngineKind::Cli => false,
        EngineKind::Auto => serve.is_some(),
    };

    if use_serve {
        let url = serve.ok_or_else(|| {
            AppError::Usage("engine=serve but no --serve-url / DOCLING_SERVE_URL set".into())
        })?;
        serve_convert(&url, args, fmt, rep)
    } else {
        cli_convert(work, args, fmt, rep)
    }
}

/// Resolve the docling binary: $DOCLING_BIN, else "docling" on PATH.
fn docling_bin() -> String {
    std::env::var("DOCLING_BIN").unwrap_or_else(|_| "docling".to_string())
}

fn cli_convert(work: &WorkDir, args: &Args, fmt: &str, rep: &Reporter) -> Result<Vec<u8>> {
    let bin = docling_bin();
    if which::which(&bin).is_err() {
        return Err(AppError::ToolMissing {
            tool: bin,
            hint: tools::DOCLING_HINT.into(),
        });
    }
    rep.phase("understand", &[("engine", "docling-cli"), ("to", fmt)]);

    let outdir = work.path();
    let mut c = Command::new(&bin);
    c.arg(&args.input)
        .arg("--to")
        .arg(fmt)
        .arg("--device")
        .arg(&args.device)
        .arg("--image-export-mode")
        .arg(args.image_mode.as_str())
        .arg("--output")
        .arg(outdir);
    ocr_flags_cli(args, &mut c);
    // Belt-and-suspenders for Apple-Silicon: let unsupported MPS ops fall back
    // to CPU even if someone forces --device mps.
    c.env("PYTORCH_ENABLE_MPS_FALLBACK", "1");
    c.stdout(Stdio::null()).stderr(Stdio::piped());

    let out = c
        .output()
        .map_err(|e| AppError::Extract(format!("docling spawn: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Extract(format!(
            "docling failed: {}",
            tail(&String::from_utf8_lossy(&out.stderr))
        )));
    }

    let stem = args
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let path = outdir.join(format!("{stem}.{fmt}"));
    std::fs::read(&path)
        .map_err(|e| AppError::Extract(format!("read docling output {}: {e}", path.display())))
}

fn ocr_flags_cli(args: &Args, c: &mut Command) {
    match args.ocr {
        OcrMode::Off => {
            c.arg("--no-ocr");
        }
        OcrMode::Force => {
            c.arg("--force-ocr");
        }
        OcrMode::Auto => {
            c.arg("--ocr");
        }
    }
    if args.no_tables {
        c.arg("--no-tables");
    }
    if let Some(eng) = &args.ocr_engine {
        c.arg("--ocr-engine").arg(eng);
    }
}

fn serve_convert(url: &str, args: &Args, fmt: &str, rep: &Reporter) -> Result<Vec<u8>> {
    tools::require("curl", tools::CURL_HINT)?;
    rep.phase("understand", &[("engine", "docling-serve"), ("to", fmt)]);

    let endpoint = format!("{}/v1/convert/file", url.trim_end_matches('/'));
    let do_ocr = !matches!(args.ocr, OcrMode::Off);
    let force = matches!(args.ocr, OcrMode::Force);

    let mut c = Command::new("curl");
    c.arg("-sS")
        .arg("--fail-with-body")
        .arg("-X")
        .arg("POST")
        .arg(&endpoint)
        .arg("-H")
        .arg("accept: application/json")
        .arg("-F")
        .arg(format!(
            "files=@{};type=application/pdf",
            args.input.display()
        ))
        .arg("-F")
        .arg(format!("to_formats={fmt}"))
        .arg("-F")
        .arg(format!("do_ocr={do_ocr}"))
        .arg("-F")
        .arg(format!("force_ocr={force}"))
        .arg("-F")
        .arg(format!("do_table_structure={}", !args.no_tables))
        .arg("-F")
        .arg(format!("image_export_mode={}", args.image_mode.as_str()));
    if let Some(eng) = &args.ocr_engine {
        c.arg("-F").arg(format!("ocr_engine={eng}"));
    }
    c.stdout(Stdio::piped()).stderr(Stdio::piped());

    let out = c
        .output()
        .map_err(|e| AppError::Extract(format!("curl spawn: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Extract(format!(
            "docling-serve request failed: {}",
            tail(&String::from_utf8_lossy(&out.stdout))
        )));
    }

    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| AppError::Extract(format!("docling-serve returned non-JSON: {e}")))?;
    let doc = v
        .get("document")
        .ok_or_else(|| AppError::Extract("docling-serve: no 'document' in response".into()))?;

    if fmt == "json" {
        let jc = doc
            .get("json_content")
            .ok_or_else(|| AppError::Extract("docling-serve: no json_content".into()))?;
        return serde_json::to_vec_pretty(jc)
            .map_err(|e| AppError::Extract(format!("reserialize json_content: {e}")));
    }
    let key = if fmt == "html" {
        "html_content"
    } else {
        "md_content"
    };
    let s = doc
        .get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| AppError::Extract(format!("docling-serve: no {key} in response")))?;
    Ok(s.as_bytes().to_vec())
}

fn tail(s: &str) -> String {
    let t = s.trim();
    let max = 600;
    if t.len() > max {
        format!("…{}", &t[t.len() - max..])
    } else {
        t.to_string()
    }
}
