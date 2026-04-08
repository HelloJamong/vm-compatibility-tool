<script lang="ts">
  type Props = {
    status: string;
    version: string;
  };

  let { status, version }: Props = $props();
</script>

<footer class="bg-gray-200 px-4 py-1.5 flex justify-between text-xs text-gray-600 shrink-0 border-t border-gray-300">
  <span>{status}</span>
  <span class="text-gray-400">{version}</span>
</footer>
