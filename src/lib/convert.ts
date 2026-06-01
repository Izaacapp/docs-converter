import { Command } from "@tauri-apps/plugin-shell";

export type Format = "md" | "html" | "json" | "tex" | "docx" | "pdf";
export type OcrMode = "auto" | "force" | "off";

export interface ConvertOpts {
  input: string;
  to: Format;
  output: string;
  ocr: OcrMode;
  /** docling-serve base URL (homelab). When set, conversion runs remotely. */
  serveUrl?: string;
  /** Absolute path to a local `docling` binary (e.g. a venv) for local runs. */
  doclingBin?: string;
  onPhase?: (line: string) => void;
}

export interface ConvertResult {
  code: number;
  stderr: string;
}

/**
 * Invoke the bundled `doc-convert` sidecar. The sidecar drives Docling (OCR +
 * tables + layout) and pandoc, writing the result straight to `output`.
 * Progress phases arrive on stderr as `>> phase=… …` lines.
 */
export async function convert(opts: ConvertOpts): Promise<ConvertResult> {
  const args = ["-i", opts.input, "-t", opts.to, "-o", opts.output, "--ocr", opts.ocr];
  if (opts.serveUrl && opts.serveUrl.trim()) {
    args.push("--serve-url", opts.serveUrl.trim());
  }

  const env: Record<string, string> = {};
  if (opts.doclingBin && opts.doclingBin.trim()) {
    env.DOCLING_BIN = opts.doclingBin.trim();
  }

  const cmd = Command.sidecar("binaries/doc-convert", args, { env });

  let stderr = "";
  cmd.stderr.on("data", (line: string) => {
    stderr += line;
    if (opts.onPhase) {
      for (const l of line.split("\n")) {
        const t = l.trim();
        if (t.startsWith(">>")) opts.onPhase(t);
      }
    }
  });

  const child = await cmd.execute();
  return { code: child.code ?? -1, stderr };
}
