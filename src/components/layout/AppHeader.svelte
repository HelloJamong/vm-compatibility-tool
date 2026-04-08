<script lang="ts">
  import type { Panel } from "../../lib/app-types";

  type Props = {
    currentPanel: Panel;
    onBack: () => void;
  };

  let { currentPanel, onBack }: Props = $props();
</script>

<header class="bg-slate-800 text-white px-4 py-2.5 flex items-center justify-between shrink-0">
  <span class="text-base font-bold tracking-tight">VM Compatibility Tool</span>
  {#if currentPanel !== "menu"}
    <button
      onclick={onBack}
      class="text-sm text-slate-300 hover:text-white transition-colors"
    >
      ← 메인 메뉴
    </button>
  {/if}
</header>
