<script lang="ts">
  interface ToastData {
    kind: "ok" | "err";
    msg: string;
  }
  let { toast }: { toast: ToastData | null } = $props();
</script>

{#if toast}
  <div class="toast {toast.kind}">{toast.msg}</div>
{/if}

<style>
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
