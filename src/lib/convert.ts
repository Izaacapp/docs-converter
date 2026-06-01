import { fetch } from "@tauri-apps/plugin-http";

export type Format = "md" | "html" | "tex" | "docx" | "pdf";

const API = (import.meta.env.VITE_CONVERTER_API_URL ?? "").replace(/\/$/, "");

export interface ConvertResult {
  ok: boolean;
  data?: Uint8Array;
  error?: string;
}

export function serverUrl(): string {
  return API;
}

/**
 * POST the PDF to the converter server and return the converted bytes. The
 * server (pdf_oxide + pandoc) does all the work; nothing runs on this machine.
 */
export async function convert(to: Format, pdf: ArrayBuffer): Promise<ConvertResult> {
  if (!API) return { ok: false, error: "VITE_CONVERTER_API_URL is not set (see .env)" };

  const url = `${API}/convert?to=${to}`;
  let res: Response;
  try {
    res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/pdf" },
      body: pdf,
    });
  } catch (e) {
    return { ok: false, error: `cannot reach ${API} — is the server up? (${e})` };
  }
  if (!res.ok) {
    const msg = await res.text().catch(() => "");
    return { ok: false, error: `server error ${res.status}: ${msg || res.statusText}` };
  }
  const buf = await res.arrayBuffer();
  return { ok: true, data: new Uint8Array(buf) };
}
