<script lang="ts">
  interface Props {
    inputPath: string | null;
    busy?: boolean;
    onpick: () => void;
  }
  let { inputPath, busy = false, onpick }: Props = $props();

  const baseName = $derived(
    inputPath ? inputPath.split(/[\\/]/).pop()!.replace(/\.pdf$/i, "") : "",
  );
</script>

<button class="drop" class:has={!!inputPath} onclick={onpick} disabled={busy}>
  {#if inputPath}
    <strong>{baseName}.pdf</strong>
    <span class="path">{inputPath}</span>
  {:else}
    <strong>Choose a PDF…</strong>
    <span>Click to select a document</span>
  {/if}
</button>

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
    transition: border-color 0.15s, background 0.15s;
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
</style>
