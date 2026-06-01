//! Tiny HTTP service (sync, no async runtime). Runs the converter on the
//! server so neither the client's CPU nor the GPU is touched.
//!
//!   GET  /health             -> "ok"
//!   POST /convert?to=md      body = raw PDF bytes -> converted bytes
//!
//! `to` ∈ md|html|tex|docx|pdf (default md). Errors -> 4xx/5xx with a message.

use crate::cli::{PdfEngine, Target};
use crate::error::AppError;
use crate::{convert, extract};
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
    eprintln!("doc-convert serving on http://0.0.0.0:{port}  (GET /health, POST /convert?to=md)");

    for mut req in server.incoming_requests() {
        let method = req.method().clone();
        let url = req.url().to_string();

        if method == Method::Get && url.starts_with("/health") {
            let _ = req.respond(Response::from_string("ok"));
            continue;
        }
        if method == Method::Post && url.starts_with("/convert") {
            let to = parse_to(&url);
            let mut body = Vec::new();
            if req.as_reader().read_to_end(&mut body).is_err() {
                let _ = req.respond(Response::from_string("read error").with_status_code(400));
                continue;
            }
            match handle(&body, to) {
                Ok(bytes) => {
                    let _ = req.respond(Response::from_data(bytes));
                }
                Err(e) => {
                    let code = if matches!(e, AppError::Input(_)) { 400 } else { 500 };
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

fn parse_to(url: &str) -> Target {
    url.split('?')
        .nth(1)
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("to=")))
        .and_then(|v| match v {
            "md" => Some(Target::Md),
            "html" => Some(Target::Html),
            "tex" => Some(Target::Tex),
            "docx" => Some(Target::Docx),
            "pdf" => Some(Target::Pdf),
            _ => None,
        })
        .unwrap_or(Target::Md)
}

fn handle(pdf_bytes: &[u8], to: Target) -> crate::error::Result<Vec<u8>> {
    if pdf_bytes.len() < 5 || &pdf_bytes[..4] != b"%PDF" {
        return Err(AppError::Input("request body is not a PDF".into()));
    }
    let tmp = tempfile::Builder::new()
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| AppError::Extract(format!("tempfile: {e}")))?;
    std::fs::write(tmp.path(), pdf_bytes)
        .map_err(|e| AppError::Extract(format!("write tmp pdf: {e}")))?;
    let md = extract::to_markdown(tmp.path())?;
    // Tectonic on the server (lightweight, no full TeX Live install needed).
    convert::to_bytes(&md, to, PdfEngine::Tectonic, false)
}
