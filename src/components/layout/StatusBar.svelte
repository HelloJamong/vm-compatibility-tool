<script lang="ts">
  type Props = {
    version: string;
    status: string;
    isBusy?: boolean;
  };

  let { version, status, isBusy = false }: Props = $props();
</script>

<footer class="bg-gray-200 px-3 py-1.5 flex items-end justify-between gap-4 text-xs text-gray-600 shrink-0 border-t border-gray-300">
  <span class="text-gray-500 font-medium">{version}</span>

  <div class="min-w-[260px] max-w-[420px] w-full flex flex-col items-end gap-1">
    <span class="text-gray-600">{status}</span>
    {#if isBusy}
      <div class="w-full h-1.5 rounded-full bg-gray-300 overflow-hidden">
        <div class="h-full w-1/3 rounded-full bg-blue-500 animate-pulse"></div>
      </div>
    {/if}
  </div>
</footer>
