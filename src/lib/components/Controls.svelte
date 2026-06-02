<script lang="ts">
  import type { Format } from "../convert";

  interface Props {
    format: Format;
    busy?: boolean;
    label?: string;
  }
  let { format = $bindable(), busy = false, label = "Format" }: Props = $props();

  const FORMATS: { v: Format; label: string }[] = [
    { v: "md", label: "Markdown" },
    { v: "html", label: "HTML" },
    { v: "tex", label: "LaTeX" },
    { v: "docx", label: "Word (.docx)" },
    { v: "pdf", label: "PDF" },
  ];
</script>

<label>
  {label}
  <select bind:value={format} disabled={busy}>
    {#each FORMATS as f}<option value={f.v}>{f.label}</option>{/each}
  </select>
</label>

<style>
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--muted);
  }
  select {
    background: var(--card);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 9px;
    padding: 10px 12px;
    font-size: 0.95rem;
  }
  select:focus {
    outline: none;
    border-color: var(--accent);
  }
</style>
