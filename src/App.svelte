<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { convert, type Format } from "./lib/convert";
  import Dropzone from "./lib/components/Dropzone.svelte";
  import Controls from "./lib/components/Controls.svelte";
  import ProgressLog from "./lib/components/ProgressLog.svelte";
  import Toast from "./lib/components/Toast.svelte";

  let inputPath = $state<string | null>(null);
  let format = $state<Format>("md");
  let busy = $state(false);
  let phases = $state<string[]>([]);
  let toast = $state<{ kind: "ok" | "err"; msg: string } | null>(null);

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
    if (line.includes("via=server")) return "Converting on the server…";
    if (line.includes("phase=extract")) return "Extracting…";
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
      4: "curl is missing, or the converter server is unreachable.",
      5: "Conversion failed on the server (pandoc/LaTeX).",
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

  <Dropzone {inputPath} {busy} onpick={pick} />

  <Controls bind:format {busy} />

  <button class="go" disabled={!inputPath || busy} onclick={run}>
    {busy ? "Converting…" : `Convert to ${format.toUpperCase()}`}
  </button>

  <ProgressLog {phases} />
  <Toast {toast} />
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
</style>
