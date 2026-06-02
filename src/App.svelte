<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { readFile, writeFile, mkdir } from "@tauri-apps/plugin-fs";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import { convert, serverUrl, type Format } from "./lib/convert";
  import {
    pickFiles,
    pickDirectory,
    expandToInputs,
    stripExt,
    type InputItem,
  } from "./lib/files";
  import Picker from "./lib/components/Picker.svelte";
  import Controls from "./lib/components/Controls.svelte";
  import Toast from "./lib/components/Toast.svelte";

  type Row = { name: string; status: "pending" | "ok" | "err"; msg?: string };

  let inputs = $state<InputItem[]>([]);
  let to = $state<Format>("pdf");
  let busy = $state(false);
  let toast = $state<{ kind: "ok" | "err"; msg: string } | null>(null);
  let results = $state<Row[]>([]);

  const goLabel = $derived(
    inputs.length > 1
      ? `Convert ${inputs.length} files to ${to.toUpperCase()}`
      : `Convert to ${to.toUpperCase()}`,
  );

  async function add(paths: string[]) {
    if (!paths.length) return;
    const found = await expandToInputs(paths);
    const have = new Set(inputs.map((i) => i.path));
    inputs = [...inputs, ...found.filter((f) => !have.has(f.path))];
    results = [];
    toast =
      found.length === 0
        ? { kind: "err", msg: "no convertible files (pdf/md/html/tex/docx) in that selection" }
        : null;
  }

  onMount(() => {
    // Let the user literally drop files or a folder onto the window.
    let un: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((e) => {
        if (e.payload.type === "drop") add(e.payload.paths);
      })
      .then((f) => (un = f))
      .catch(() => {});
    return () => un?.();
  });

  async function chooseFiles() {
    await add(await pickFiles());
  }
  async function chooseFolder() {
    const d = await pickDirectory("Choose a folder to convert");
    if (d) await add([d]);
  }
  function clear() {
    inputs = [];
    results = [];
    toast = null;
  }

  async function run() {
    if (!inputs.length || busy) return;
    busy = true;
    toast = null;

    // Choose the destination once, up front.
    let singleOut: string | null = null;
    let outDir: string | null = null;
    if (inputs.length === 1) {
      singleOut = await save({ defaultPath: `${stripExt(inputs[0].name)}.${to}` });
      if (!singleOut) {
        busy = false;
        return;
      }
    } else {
      outDir = await pickDirectory("Choose where to save the converted files");
      if (!outDir) {
        busy = false;
        return;
      }
      try {
        await mkdir(outDir, { recursive: true });
      } catch {
        /* already exists */
      }
    }

    results = inputs.map((i) => ({ name: i.name, status: "pending" as const }));
    let ok = 0;
    for (let k = 0; k < inputs.length; k++) {
      const it = inputs[k];
      try {
        const bytes = await readFile(it.path);
        const r = await convert(it.from, to, bytes);
        if (!r.ok) {
          results[k] = { name: it.name, status: "err", msg: r.error };
          continue;
        }
        const dest = singleOut ?? `${outDir}/${stripExt(it.name)}.${to}`;
        await writeFile(dest, r.data!);
        results[k] = { name: it.name, status: "ok", msg: dest };
        ok++;
      } catch (e) {
        results[k] = { name: it.name, status: "err", msg: String(e) };
      }
    }
    toast = {
      kind: ok === inputs.length ? "ok" : "err",
      msg: `${ok}/${inputs.length} converted → ${to.toUpperCase()}`,
    };
    busy = false;
  }
</script>

<main>
  <header>
    <h1>docs-converter</h1>
    <p class="sub">
      Convert between PDF · Markdown · HTML · LaTeX · Word — one file or a whole folder, on the server
    </p>
  </header>

  <Picker {inputs} {busy} onfiles={chooseFiles} onfolder={chooseFolder} onclear={clear} />
  <Controls bind:format={to} {busy} label="Convert to" />

  <button class="go" disabled={!inputs.length || busy} onclick={run}>
    {busy ? "Converting on the server…" : goLabel}
  </button>

  {#if results.length}
    <ul class="results">
      {#each results as r (r.name)}
        <li class={r.status}>
          <span class="dot">{r.status === "ok" ? "✓" : r.status === "err" ? "✗" : "…"}</span>
          <span class="rname">{r.name}</span>
          {#if r.msg}<span class="rmsg">{r.msg}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}

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
  .results {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .results li {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 0.82rem;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--bg-2);
  }
  .results li.ok .dot {
    color: #54d18c;
  }
  .results li.err .dot {
    color: #ff6b6b;
  }
  .results .rname {
    font-weight: 600;
  }
  .results .rmsg {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
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
