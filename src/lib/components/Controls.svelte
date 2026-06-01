<script lang="ts">
  import type { Format, OcrMode } from "../convert";

  interface Props {
    format: Format;
    ocr: OcrMode;
    serveUrl: string;
    doclingBin: string;
    busy?: boolean;
  }
  let {
    format = $bindable(),
    ocr = $bindable(),
    serveUrl = $bindable(),
    doclingBin = $bindable(),
    busy = false,
  }: Props = $props();

  let showAdvanced = $state(false);

  const FORMATS: { v: Format; label: string }[] = [
    { v: "md", label: "Markdown" },
    { v: "html", label: "HTML" },
    { v: "json", label: "JSON" },
    { v: "tex", label: "LaTeX" },
    { v: "docx", label: "Word (.docx)" },
    { v: "pdf", label: "PDF" },
  ];
</script>

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

<style>
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
</style>
