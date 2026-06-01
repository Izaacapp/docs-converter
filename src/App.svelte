<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { convert, type Format, type OcrMode } from "./lib/convert";

  let inputPath = $state<string | null>(null);
  let format = $state<Format>("md");
  let ocr = $state<OcrMode>("auto");
  let serveUrl = $state("");
  let doclingBin = $state("");
  let showAdvanced = $state(false);
  let busy = $state(false);
  let phases = $state<string[]>([]);
  let toast = $state<{ kind: "ok" | "err"; msg: string } | null>(null);

  const FORMATS: { v: Format; label: string }[] = [
    { v: "md", label: "Markdown" },
    { v: "html", label: "HTML" },
    { v: "json", label: "JSON" },
    { v: "tex", label: "LaTeX" },
    { v: "docx", label: "Word (.docx)" },
    { v: "pdf", label: "PDF" },
  ];

  const baseName = $derived(
    inputPath ? inputPath.split(/[\\/]/).pop()!.replace(/\.pdf$/i, "") : "document",
  );

  async function pick() {
    const sel = await open({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (typeof sel === "string") {
      inputPath = sel;
      toast = null;
    }
  }

  async function run() {
    if (!inputPath) return;
    const out = await save({ defaultPath: `${baseName}.${format}` });
    if (!out) return;

    busy = true;
    phases = [];
    toast = null;
    try {
      const res = await convert({
        input: inputPath,
        to: format,
        output: out,
        ocr,
        serveUrl: serveUrl || undefined,
        doclingBin: doclingBin || undefined,
        onPhase: (l) => (phases = [...phases, prettyPhase(l)]),
      });
      toast =
        res.code === 0
          ? { kind: "ok", msg: `Saved → ${out}` }
          : { kind: "err", msg: explain(res.code, res.stderr) };
    } catch (e) {
      toast = { kind: "err", msg: String(e) };
    } finally {
      busy = false;
    }
  }

  function prettyPhase(line: string): string {
    if (line.includes("phase=understand")) return "Understanding document (Docling, OCR + tables)…";
    if (line.includes("phase=convert")) return `Converting → ${format.toUpperCase()}…`;
    if (line.includes("done")) return "Done.";
    return line.replace(/^>>\s*/, "");
  }

  function explain(code: number, stderr: string): string {
    const last = stderr
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.startsWith("doc-convert:"))
      .pop();
    const hints: Record<number, string> = {
      1: "Could not read that PDF.",
      4: "A required tool is missing — install Docling/pandoc, or set a Docling server URL under Advanced.",
      5: "Conversion failed (pandoc/LaTeX).",
      64: "Bad options.",
    };
    return last?.replace("doc-convert:", "").trim() || hints[code] || `Failed (exit ${code}).`;
  }
</script>

<main>
  <header>
    <h1>docs-converter</h1>
    <p class="sub">PDF → Markdown · HTML · JSON · LaTeX · Word · PDF — OCR built in (Docling)</p>
  </header>

  <button class="drop" class:has={inputPath} onclick={pick} disabled={busy}>
    {#if inputPath}
      <strong>{baseName}.pdf</strong>
      <span class="path">{inputPath}</span>
    {:else}
      <strong>Choose a PDF…</strong>
      <span>Click to select a document</span>
    {/if}
  </button>

  <div class="row">
    <label>
      Format
      <select bind:value={format} disabled={busy}>
        {#each FORMATS as f}<option value={f.v}>{f.label}</option>{/each}
      </select>
    </label>
    <label>
      OCR
      <select bind:value={ocr} disabled={busy}>
        <option value="auto">Auto</option>
        <option value="force">Force</option>
        <option value="off">Off</option>
      </select>
    </label>
  </div>

  <button class="linkbtn" onclick={() => (showAdvanced = !showAdvanced)}>
    {showAdvanced ? "Hide advanced" : "Advanced"}
  </button>
  {#if showAdvanced}
    <div class="adv">
      <label>
        Docling server URL (homelab)
        <input placeholder="http://homelab:5001" bind:value={serveUrl} />
      </label>
      <label>
        Local docling binary
        <input placeholder="/path/to/.venv/bin/docling" bind:value={doclingBin} />
      </label>
    </div>
  {/if}

  <button class="go" disabled={!inputPath || busy} onclick={run}>
    {busy ? "Converting…" : `Convert to ${format.toUpperCase()}`}
  </button>

  {#if phases.length}
    <ul class="phases">
      {#each phases as p}<li>{p}</li>{/each}
    </ul>
  {/if}

  {#if toast}
    <div class="toast {toast.kind}">{toast.msg}</div>
  {/if}
</main>

<style>
  main {
    max-width: 640px;
    margin: 0 auto;
    padding: 48px 24px 64px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  header h1 {
    margin: 0;
    font-size: 2rem;
    background: var(--grad);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    width: fit-content;
  }
  .sub {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 0.9rem;
  }
  .drop {
    appearance: none;
    border: 1.5px dashed var(--border);
    background: var(--bg-2);
    color: var(--text);
    border-radius: 14px;
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    text-align: left;
  }
  .drop:hover:not(:disabled) {
    border-color: var(--accent);
  }
  .drop.has {
    border-style: solid;
    border-color: var(--accent-2);
  }
  .drop strong {
    font-size: 1.05rem;
  }
  .drop span {
    color: var(--muted);
    font-size: 0.82rem;
  }
  .drop .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--muted);
  }
  select,
  input {
    background: var(--card);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 9px;
    padding: 10px 12px;
    font-size: 0.95rem;
  }
  select:focus,
  input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .linkbtn {
    appearance: none;
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 0.82rem;
    width: fit-content;
    padding: 0;
  }
  .adv {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 11px;
  }
  .go {
    appearance: none;
    border: none;
    border-radius: 11px;
    padding: 14px;
    font-size: 1rem;
    font-weight: 600;
    color: #fff;
    background: var(--grad);
    cursor: pointer;
    box-shadow: var(--shadow);
    transition: filter 0.15s, opacity 0.15s;
  }
  .go:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .go:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .phases {
    list-style: none;
    margin: 0;
    padding: 12px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 11px;
    font-size: 0.82rem;
    color: var(--muted);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .toast {
    border-radius: 11px;
    padding: 12px 14px;
    font-size: 0.9rem;
    word-break: break-all;
  }
  .toast.ok {
    background: rgba(63, 185, 80, 0.12);
    border: 1px solid var(--ok);
    color: #8be0a0;
  }
  .toast.err {
    background: rgba(248, 81, 73, 0.12);
    border: 1px solid var(--err);
    color: #ff9b95;
  }
</style>
