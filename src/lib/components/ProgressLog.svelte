<script lang="ts">
  let { phases }: { phases: string[] } = $props();
</script>

{#if phases.length}
  <ul class="phases">
    {#each phases as p}<li>{p}</li>{/each}
  </ul>
{/if}

<style>
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
</style>
