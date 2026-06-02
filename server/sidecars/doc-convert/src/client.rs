//! Client mode: forward the PDF to a doc-convert server and write back the
//! response. No local conversion — the server does the work, so neither the
//! client's CPU nor the GPU is touched.

use crate::cli::{Args, Target};
use crate::error::{AppError, Result};
use crate::progress::Reporter;
use crate::tools;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

pub fn forward(args: &Args, from: Target, api: &str, rep: &Reporter, start: Instant) -> Result<()> {
    tools::require("curl", tools::CURL_HINT)?;
    let url = format!(
        "{}/convert?from={}&to={}",
        api.trim_end_matches('/'),
        from.as_str(),
        args.to.as_str()
    );
    rep.phase(
        "convert",
        &[("via", "server"), ("from", from.as_str()), ("to", args.to.as_str())],
    );

    let mut c = Command::new("curl");
    c.arg("-fsS")
        .arg("-H")
        .arg("Content-Type: application/octet-stream")
        .arg("--data-binary")
        .arg(format!("@{}", args.input.display()))
        .arg(&url);
    match &args.output {
        Some(p) => {
            c.arg("-o").arg(p).stdout(Stdio::null());
        }
        None => {
            c.stdout(Stdio::piped());
        }
    }
    c.stderr(Stdio::piped());

    let out = c
        .output()
        .map_err(|e| AppError::Convert(format!("curl spawn: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Convert(format!(
            "server convert failed ({url}): {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }

    if args.output.is_none() {
        let _ = std::io::stdout().lock().write_all(&out.stdout);
    }
    let n = args
        .output
        .as_ref()
        .and_then(|p| std::fs::metadata(p).ok())
        .map(|m| m.len() as usize)
        .unwrap_or(out.stdout.len());
    rep.done(n, start.elapsed().as_millis());
    Ok(())
}
