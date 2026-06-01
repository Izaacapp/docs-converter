<script lang="ts">
  interface Props {
    fileName: string | null;
    busy?: boolean;
    onpick: (f: File) => void;
  }
  let { fileName, busy = false, onpick }: Props = $props();

  let input = $state<HTMLInputElement>();

  function change(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) onpick(f);
  }
</script>

<button type="button" class="drop" class:has={!!fileName} onclick={() => input?.click()} disabled={busy}>
  {#if fileName}
    <strong>{fileName}</strong>
    <span>Click to choose another</span>
  {:else}
    <strong>Choose a PDF…</strong>
    <span>Click to select a document</span>
  {/if}
</button>
<input bind:this={input} type="file" accept=".pdf,application/pdf" onchange={change} hidden />

<style>
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
    transition: border-color 0.15s;
    text-align: left;
    width: 100%;
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .drop span {
    color: var(--muted);
    font-size: 0.82rem;
  }
</style>
