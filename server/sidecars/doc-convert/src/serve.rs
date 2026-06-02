//! Tiny HTTP service (sync, no async runtime). Runs the converter on the
//! server so neither the client's CPU nor the GPU is touched.
//!
//!   GET  /health                  -> "ok"
//!   POST /convert?from=pdf&to=md  body = raw document bytes -> converted bytes
//!
//! `from` ∈ pdf|md|html|tex|docx (sniffed from the body if omitted).
//! `to`   ∈ md|html|tex|docx|pdf (default md). Errors -> 4xx/5xx with a message.

use crate::cli::{PdfEngine, Target};
use crate::convert;
use crate::error::AppError;
use std::process::ExitCode;
use tiny_http::{Method, Response, Server};

pub fn run(port: u16) -> ExitCode {
    let server = match Server::http(("0.0.0.0", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("doc-convert serve: bind :{port} failed: {e}");
            return ExitCode::from(1);
        }
    };
    eprintln!(
        "doc-convert serving on http://0.0.0.0:{port}  (GET /health, POST /convert?from=&to=)"
    );

    for mut req in server.incoming_requests() {
        let method = req.method().clone();
        let url = req.url().to_string();

        if method == Method::Get && url.starts_with("/health") {
            let _ = req.respond(Response::from_string("ok"));
            continue;
        }
        if method == Method::Post && url.starts_with("/convert") {
            let to = parse_fmt(&url, "to").unwrap_or(Target::Md);
            let mut body = Vec::new();
            if req.as_reader().read_to_end(&mut body).is_err() {
                let _ = req.respond(Response::from_string("read error").with_status_code(400));
                continue;
            }
            // The desktop app always sends `from`; sniff keeps raw curl handy.
            let from = parse_fmt(&url, "from").unwrap_or_else(|| sniff(&body));
            // Tectonic on the server (lightweight, no full TeX Live install).
            match convert::run(&body, from, to, PdfEngine::Tectonic, false) {
                Ok(bytes) => {
                    let _ = req.respond(Response::from_data(bytes));
                }
                Err(e) => {
                    let code = if matches!(e, AppError::Input(_) | AppError::Usage(_)) {
                        400
                    } else {
                        500
                    };
                    let _ = req.respond(
                        Response::from_string(format!("doc-convert: {e}")).with_status_code(code),
                    );
                }
            }
            continue;
        }
        let _ = req.respond(Response::from_string("not found").with_status_code(404));
    }
    ExitCode::SUCCESS
}

/// Parse a `?from=`/`?to=` format off the query string.
fn parse_fmt(url: &str, key: &str) -> Option<Target> {
    let prefix = format!("{key}=");
    url.split('?')
        .nth(1)?
        .split('&')
        .find_map(|kv| kv.strip_prefix(&prefix))
        .and_then(|v| match v {
            "md" | "markdown" => Some(Target::Md),
            "html" | "htm" => Some(Target::Html),
            "tex" | "latex" => Some(Target::Tex),
            "docx" => Some(Target::Docx),
            "pdf" => Some(Target::Pdf),
            _ => None,
        })
}

/// Best-effort input detection when `?from=` is omitted.
fn sniff(body: &[u8]) -> Target {
    if body.starts_with(b"%PDF") {
        return Target::Pdf;
    }
    if body.starts_with(b"PK\x03\x04") {
        return Target::Docx; // zip container — assume .docx
    }
    let head = String::from_utf8_lossy(&body[..body.len().min(2048)]).to_lowercase();
    let trimmed = head.trim_start();
    if trimmed.starts_with("<!doctype")
        || trimmed.starts_with("<html")
        || head.contains("<body")
        || head.contains("<div")
        || head.contains("<p>")
    {
        Target::Html
    } else if head.contains("\\documentclass") || head.contains("\\begin{document}") {
        Target::Tex
    } else {
        Target::Md
    }
}
