<script lang="ts">
  type Props = {
    open: boolean;
    title: string;
    message: string;
    bullets?: string[];
    confirmLabel: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  };

  let {
    open,
    title,
    message,
    bullets = [],
    confirmLabel,
    cancelLabel = "취소",
    onConfirm,
    onCancel,
  }: Props = $props();
</script>

{#if open}
  <div class="absolute inset-0 bg-black/35 flex items-center justify-center p-4">
    <div class="w-full max-w-md rounded-xl bg-white shadow-2xl border border-gray-200 overflow-hidden">
      <div class="px-5 py-4 border-b border-gray-200">
        <h3 class="text-base font-bold text-gray-900">{title}</h3>
        <p class="mt-1 text-sm text-gray-500">{message}</p>
      </div>
      {#if bullets.length > 0}
        <div class="px-5 py-4 text-sm text-gray-700 space-y-1">
          {#each bullets as bullet}
            <div>• {bullet}</div>
          {/each}
        </div>
      {/if}
      <div class="px-5 py-4 bg-gray-50 flex justify-end gap-2">
        <button
          onclick={onCancel}
          class="px-4 py-2 text-sm rounded bg-white border border-gray-300 text-gray-700 hover:bg-gray-100"
        >
          {cancelLabel}
        </button>
        <button
          onclick={onConfirm}
          class="px-4 py-2 text-sm rounded bg-slate-800 text-white hover:bg-slate-900"
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
