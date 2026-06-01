//! Integration tests for the doc-convert binary.
//!
//! Two tiers:
//!   * contract tests — exit codes / arg handling, no Docling needed (fast, CI-default).
//!   * conversion tests — actually run Docling; opt-in via DOC_CONVERT_E2E=1 and
//!     skip-don't-fail when docling/pandoc aren't present.
//!
//! Cargo exposes the built binary via env!("CARGO_BIN_EXE_doc-convert"), so these
//! exercise the same subprocess path the Tauri app uses.

use std::path::{Path, PathBuf};
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_doc-convert");

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn e2e_enabled() -> bool {
    std::env::var("DOC_CONVERT_E2E").is_ok()
}

fn docling_available() -> bool {
    let bin = std::env::var("DOCLING_BIN").unwrap_or_else(|_| "docling".into());
    if bin.contains('/') {
        Path::new(&bin).exists()
    } else {
        std::env::var("PATH")
            .map(|p| std::env::split_paths(&p).any(|d| d.join(&bin).exists()))
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Contract tests — no Docling required
// ---------------------------------------------------------------------------

#[test]
fn missing_required_flags_is_usage_error() {
    let out = Command::new(BINARY).output().expect("spawn doc-convert");
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(64), "missing flags should be usage error 64");
}

#[test]
fn nonexistent_input_exits_1() {
    let out = Command::new(BINARY)
        .args(["-i", "/tmp/nope-doc-convert-xyz.pdf", "-t", "md"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("open pdf failed"), "stderr: {err}");
}

#[test]
fn binary_target_without_output_exits_64() {
    let pdf = fixtures().join("digital_3p.pdf");
    if !pdf.exists() {
        eprintln!("skip: fixture missing {}", pdf.display());
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "pdf"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(64), "pdf without --output should be 64");
}

#[test]
fn missing_docling_exits_4() {
    let pdf = fixtures().join("digital_3p.pdf");
    if !pdf.exists() {
        eprintln!("skip: fixture missing {}", pdf.display());
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "md", "--engine", "cli"])
        .env("DOCLING_BIN", "/nonexistent/docling-xyz")
        .env_remove("DOCLING_SERVE_URL")
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(4),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("docling"));
}

// ---------------------------------------------------------------------------
// Conversion tests — need Docling; opt-in + skip-don't-fail
// ---------------------------------------------------------------------------

#[test]
fn md_conversion_produces_clean_text() {
    if !e2e_enabled() {
        eprintln!("skip: set DOC_CONVERT_E2E=1 to run Docling conversion tests");
        return;
    }
    if !docling_available() {
        eprintln!("skip: docling not available (set DOCLING_BIN)");
        return;
    }
    let pdf = fixtures().join("digital_3p.pdf");
    if !pdf.exists() {
        eprintln!("skip: fixture missing");
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "md", "-q"])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "exit {:?} stderr {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8(out.stdout).expect("md is valid UTF-8");
    assert!(s.len() > 200, "expected real extracted text, got {} bytes", s.len());
    assert!(
        !s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]),
        "leading UTF-8 BOM detected"
    );
}

#[test]
fn tex_normalizes_ligatures() {
    if !e2e_enabled() || !docling_available() {
        eprintln!("skip: DOC_CONVERT_E2E + docling required");
        return;
    }
    let pdf = fixtures().join("type3_2p.pdf");
    if !pdf.exists() {
        eprintln!("skip: fixture missing");
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "tex", "-q"])
        .output()
        .expect("spawn");
    if !out.status.success() {
        eprintln!(
            "skip: conversion unavailable (pandoc?): {}",
            String::from_utf8_lossy(&out.stderr)
        );
        return;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        !s.contains('\u{FB01}') && !s.contains('\u{FB02}'),
        "ligatures (ﬁ/ﬂ) leaked into LaTeX output"
    );
}
