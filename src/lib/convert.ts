import { fetch } from "@tauri-apps/plugin-http";

export type Format = "md" | "html" | "tex" | "docx" | "pdf";

/** Every format, valid as both an input and an output. */
export const FORMATS: Format[] = ["md", "html", "tex", "docx", "pdf"];

/** File extensions we accept as input, mapped to their format. */
const EXTS: Record<string, Format> = {
  pdf: "pdf",
  md: "md",
  markdown: "md",
  mdown: "md",
  html: "html",
  htm: "html",
  tex: "tex",
  latex: "tex",
  docx: "docx",
};

const API = (import.meta.env.VITE_CONVERTER_API_URL ?? "").replace(/\/$/, "");

export function serverUrl(): string {
  return API;
}

/** Map a filename to an input Format (by extension), or null if unsupported. */
export function formatFromName(name: string): Format | null {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return EXTS[ext] ?? null;
}

export interface ConvertResult {
  ok: boolean;
  data?: Uint8Array;
  error?: string;
}

/**
 * POST `bytes` (a `from` document) to the converter server and return the
 * converted `to` bytes. The server (pdf_oxide + pandoc) does all the work;
 * nothing converts on this machine.
 */
export async function convert(
  from: Format,
  to: Format,
  bytes: Uint8Array,
): Promise<ConvertResult> {
  if (!API) return { ok: false, error: "VITE_CONVERTER_API_URL is not set (see .env)" };

  // A fresh ArrayBuffer (not a Uint8Array view) is an unambiguous BodyInit.
  const body = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(body).set(bytes);

  const url = `${API}/convert?from=${from}&to=${to}`;
  let res: Response;
  try {
    res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/octet-stream" },
      body,
    });
  } catch (e) {
    return { ok: false, error: `cannot reach ${API} — is the server up? (${e})` };
  }
  if (!res.ok) {
    const msg = await res.text().catch(() => "");
    return { ok: false, error: `server ${res.status}: ${msg || res.statusText}` };
  }
  return { ok: true, data: new Uint8Array(await res.arrayBuffer()) };
}
