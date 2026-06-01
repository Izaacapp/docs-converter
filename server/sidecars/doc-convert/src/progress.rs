//! Progress reporting on **stderr** so it never contaminates the payload
//! on stdout. Two shapes: human (`>> phase=… k=v`) or NDJSON
//! (`{"event":"phase",…}`) when `--json-progress` is set.

use crate::cli::Args;
use std::io::Write;

pub struct Reporter {
    json: bool,
    quiet: bool,
}

impl Reporter {
    pub fn from_args(a: &Args) -> Self {
        Reporter {
            json: a.json_progress,
            quiet: a.quiet,
        }
    }

    fn emit(&self, s: &str) {
        if self.quiet {
            return;
        }
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "{s}");
    }

    pub fn phase(&self, name: &str, kv: &[(&str, &str)]) {
        if self.json {
            let mut s = format!("{{\"event\":\"phase\",\"name\":{}", jstr(name));
            for (k, v) in kv {
                s.push(',');
                s.push_str(&jstr(k));
                s.push(':');
                s.push_str(&jstr(v));
            }
            s.push('}');
            self.emit(&s);
        } else {
            let mut s = format!(">> phase={name}");
            for (k, v) in kv {
                s.push_str(&format!(" {k}={v}"));
            }
            self.emit(&s);
        }
    }

    pub fn done(&self, bytes: usize, elapsed_ms: u128) {
        if self.json {
            self.emit(&format!(
                "{{\"event\":\"done\",\"bytes\":{bytes},\"elapsed_ms\":{elapsed_ms}}}"
            ));
        } else {
            self.emit(&format!(">> done bytes={bytes} elapsed_ms={elapsed_ms}"));
        }
    }
}

/// Minimal JSON string encoder.
fn jstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
