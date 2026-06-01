<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeFile } from "@tauri-apps/plugin-fs";
  import { convert, serverUrl, type Format } from "./lib/convert";
  import Dropzone from "./lib/components/Dropzone.svelte";
  import Controls from "./lib/components/Controls.svelte";
  import Toast from "./lib/components/Toast.svelte";

  let file = $state<File | null>(null);
  let format = $state<Format>("md");
  let busy = $state(false);
  let toast = $state<{ kind: "ok" | "err"; msg: string } | null>(null);

  const baseName = $derived(file ? file.name.replace(/\.pdf$/i, "") : "document");

  function onpick(f: File) {
    file = f;
    toast = null;
  }

  async function run() {
    if (!file) return;
    busy = true;
    toast = null;
    try {
      const pdf = await file.arrayBuffer();
      const r = await convert(format, pdf);
      if (!r.ok) {
        toast = { kind: "err", msg: r.error! };
        return;
      }
      const out = await save({ defaultPath: `${baseName}.${format}` });
      if (!out) return;
      await writeFile(out, r.data!);
      toast = { kind: "ok", msg: `Saved → ${out}` };
    } catch (e) {
      toast = { kind: "err", msg: String(e) };
    } finally {
      busy = false;
    }
  }
</script>

<main>
  <header>
    <h1>docs-converter</h1>
    <p class="sub">PDF → Markdown · HTML · LaTeX · Word · PDF — converted on the server</p>
  </header>

  <Dropzone fileName={file?.name ?? null} {busy} {onpick} />
  <Controls bind:format {busy} />

  <button class="go" disabled={!file || busy} onclick={run}>
    {busy ? "Converting on the server…" : `Convert to ${format.toUpperCase()}`}
  </button>

  {#if serverUrl()}
    <p class="server">server · {serverUrl()}</p>
  {:else}
    <p class="server warn">Set VITE_CONVERTER_API_URL in .env</p>
  {/if}

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
  .server {
    margin: 0;
    font-size: 0.78rem;
    color: var(--muted);
  }
  .server.warn {
    color: #ff9b95;
  }
</style>
