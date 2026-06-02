<script lang="ts">
  import type { InputItem } from "../files";

  interface Props {
    inputs: InputItem[];
    busy?: boolean;
    onfiles: () => void;
    onfolder: () => void;
    onclear: () => void;
  }
  let { inputs, busy = false, onfiles, onfolder, onclear }: Props = $props();
</script>

<div class="picker">
  <div class="drop" class:has={inputs.length > 0}>
    {#if inputs.length === 0}
      <strong>Choose files or a folder…</strong>
      <span>PDF · Markdown · HTML · LaTeX · Word — or drag &amp; drop onto the window</span>
    {:else}
      <strong>{inputs.length} file{inputs.length === 1 ? "" : "s"} selected</strong>
      <ul>
        {#each inputs.slice(0, 8) as it (it.path)}
          <li><span class="badge">{it.from}</span> {it.name}</li>
        {/each}
        {#if inputs.length > 8}<li class="more">+{inputs.length - 8} more…</li>{/if}
      </ul>
    {/if}
  </div>
  <div class="row">
    <button type="button" onclick={onfiles} disabled={busy}>Choose files…</button>
    <button type="button" onclick={onfolder} disabled={busy}>Choose folder…</button>
    {#if inputs.length}
      <button type="button" class="ghost" onclick={onclear} disabled={busy}>Clear</button>
    {/if}
  </div>
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .drop {
    border: 1.5px dashed var(--border);
    background: var(--bg-2);
    color: var(--text);
    border-radius: 14px;
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
  }
  .drop.has {
    border-style: solid;
    border-color: var(--accent-2);
  }
  .drop strong {
    font-size: 1.02rem;
  }
  .drop > span {
    color: var(--muted);
    font-size: 0.82rem;
  }
  ul {
    margin: 4px 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 200px;
    overflow: auto;
  }
  li {
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  li.more {
    color: var(--muted);
  }
  .badge {
    display: inline-block;
    min-width: 2.6em;
    text-align: center;
    padding: 1px 6px;
    margin-right: 6px;
    border-radius: 6px;
    background: var(--card);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.72rem;
    text-transform: uppercase;
  }
  .row {
    display: flex;
    gap: 10px;
  }
  .row button {
    appearance: none;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--text);
    border-radius: 9px;
    padding: 9px 14px;
    font-size: 0.88rem;
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .row button:hover:not(:disabled) {
    border-color: var(--accent);
  }
  .row button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .row .ghost {
    margin-left: auto;
    background: transparent;
  }
</style>
