import { open } from "@tauri-apps/plugin-dialog";
import { readDir, stat } from "@tauri-apps/plugin-fs";
import { formatFromName, type Format } from "./convert";

export interface InputItem {
  path: string;
  name: string; // file name with extension
  from: Format;
}

const FILTERS = [
  {
    name: "Documents",
    extensions: ["pdf", "md", "markdown", "mdown", "html", "htm", "tex", "latex", "docx"],
  },
];

export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function stripExt(name: string): string {
  return name.replace(/\.[^.]+$/, "");
}

/** Open a multi-file picker; returns selected paths (or []). */
export async function pickFiles(): Promise<string[]> {
  const sel = await open({ multiple: true, directory: false, filters: FILTERS });
  if (!sel) return [];
  return Array.isArray(sel) ? sel : [sel];
}

/** Open a folder picker; returns the chosen directory path (or null). */
export async function pickDirectory(title?: string): Promise<string | null> {
  const sel = await open({ directory: true, multiple: false, title });
  return typeof sel === "string" ? sel : null;
}

/**
 * Expand picked paths into convertible inputs: a file maps by its extension; a
 * directory contributes its top-level convertible files. Unsupported files are
 * skipped, and paths are de-duplicated.
 */
export async function expandToInputs(paths: string[]): Promise<InputItem[]> {
  const out: InputItem[] = [];
  const seen = new Set<string>();
  const push = (path: string, name: string) => {
    const from = formatFromName(name);
    if (from && !seen.has(path)) {
      seen.add(path);
      out.push({ path, name, from });
    }
  };

  for (const p of paths) {
    let isDir = false;
    try {
      isDir = (await stat(p)).isDirectory;
    } catch {
      continue; // unreadable / vanished
    }
    if (isDir) {
      let entries;
      try {
        entries = await readDir(p);
      } catch {
        continue;
      }
      for (const e of entries) {
        if (e.isFile) push(`${p}/${e.name}`, e.name);
      }
    } else {
      push(p, baseName(p));
    }
  }
  return out;
}
