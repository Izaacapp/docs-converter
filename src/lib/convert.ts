import { Command } from "@tauri-apps/plugin-shell";

export type Format = "md" | "html" | "tex" | "docx" | "pdf";

export interface ConvertOpts {
  input: string;
  to: Format;
  output: string;
  onPhase?: (line: string) => void;
}

export interface ConvertResult {
  code: number;
  stderr: string;
}

// The converter runs on the server; the bundled sidecar just forwards to it.
const API_URL = import.meta.env.VITE_CONVERTER_API_URL ?? "";

/**
 * Invoke the bundled `doc-convert` sidecar in client mode: it POSTs the PDF to
 * the converter server (VITE_CONVERTER_API_URL) and writes the result to
 * `output`. No conversion happens on this machine. Progress phases arrive on
 * stderr as `>> phase=…`.
 */
export async function convert(opts: ConvertOpts): Promise<ConvertResult> {
  const args = ["-i", opts.input, "-t", opts.to, "-o", opts.output];
  if (API_URL) args.push("--api-url", API_URL);

  const cmd = Command.sidecar("binaries/doc-convert", args);

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
