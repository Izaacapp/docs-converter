//! Integration tests for the doc-convert binary. Extraction is in-process
//! (pdf_oxide), so the markdown tests run by default — fast, no server. Tests
//! that need pandoc skip-don't-fail when it's absent.

use std::path::PathBuf;
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_doc-convert");

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn have(bin: &str) -> bool {
    std::env::var("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).exists()))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Contract — exit codes
// ---------------------------------------------------------------------------

#[test]
fn missing_required_flags_is_usage_error() {
    let out = Command::new(BINARY).output().expect("spawn");
    assert_eq!(out.status.code(), Some(64));
}

#[test]
fn nonexistent_input_exits_1() {
    let out = Command::new(BINARY)
        .args(["-i", "/tmp/nope-doc-convert-xyz.pdf", "-t", "md"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("open pdf failed"));
}

#[test]
fn binary_target_without_output_exits_64() {
    let pdf = fixtures().join("digital_3p.pdf");
    if !pdf.exists() {
        eprintln!("skip: fixture missing");
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "pdf"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(64));
}

// ---------------------------------------------------------------------------
// In-process extraction (pdf_oxide) — runs by default
// ---------------------------------------------------------------------------

#[test]
fn md_conversion_produces_clean_utf8_text() {
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
    assert!(s.len() > 200, "expected real text, got {} bytes", s.len());
    assert!(!s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]), "leading BOM");
}

#[test]
fn deterministic_across_runs() {
    let pdf = fixtures().join("digital_3p.pdf");
    if !pdf.exists() {
        return;
    }
    let go = || {
        Command::new(BINARY)
            .arg("-i")
            .arg(&pdf)
            .args(["-t", "md", "-q"])
            .output()
            .expect("spawn")
            .stdout
    };
    assert_eq!(go(), go(), "pdf_oxide extraction must be deterministic");
}

#[test]
fn handles_input_and_output_paths_with_spaces() {
    let src = fixtures().join("digital_3p.pdf");
    if !src.exists() {
        return;
    }
    let dir = std::env::temp_dir().join("doc convert spaces test");
    let _ = std::fs::create_dir_all(&dir);
    let input = dir.join("a book with spaces.pdf");
    std::fs::copy(&src, &input).expect("copy fixture");
    let out = dir.join("out put.md");

    let status = Command::new(BINARY)
        .arg("-i")
        .arg(&input)
        .args(["-t", "md", "-q"])
        .arg("-o")
        .arg(&out)
        .status()
        .expect("spawn");
    assert!(status.success(), "spaces-path conversion failed: {:?}", status.code());
    assert!(std::fs::metadata(&out).map(|m| m.len() > 200).unwrap_or(false));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// pandoc leg — skip-don't-fail when pandoc is absent
// ---------------------------------------------------------------------------

#[test]
fn tex_normalizes_ligatures() {
    if !have("pandoc") {
        eprintln!("skip: pandoc not installed");
        return;
    }
    let pdf = fixtures().join("type3_2p.pdf");
    if !pdf.exists() {
        return;
    }
    let out = Command::new(BINARY)
        .arg("-i")
        .arg(&pdf)
        .args(["-t", "tex", "-q"])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "exit {:?} stderr {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        !s.contains('\u{FB01}') && !s.contains('\u{FB02}'),
        "ligatures leaked into LaTeX output"
    );
}
