<script lang="ts">
  import type { SystemInfoItem } from "../../lib/app-types";

  type Props = {
    systemLoading: boolean;
    systemItems: SystemInfoItem[];
    onRefresh: () => void;
    onExport: () => void;
  };

  let {
    systemLoading,
    systemItems,
    onRefresh,
    onExport,
  }: Props = $props();
</script>

<div class="flex flex-col gap-2.5 h-full">
  <div class="flex items-center justify-between shrink-0">
    <h2 class="text-base font-bold text-gray-800">시스템 상세 정보</h2>
    <button
      onclick={onRefresh}
      disabled={systemLoading}
      class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded-lg transition-colors"
    >
      재수집
    </button>
  </div>

  {#if systemLoading && systemItems.length === 0}
    <div class="flex-1 flex items-center justify-center text-sm text-gray-400">
      데이터를 수집하고 있습니다...
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center">
      <div class="w-full max-w-sm rounded-xl border border-green-200 bg-green-50 px-6 py-8 text-center flex flex-col gap-3">
        <p class="text-2xl">✅</p>
        <p class="text-sm font-semibold text-green-800">시스템 정보 수집 완료</p>
        <p class="text-xs text-green-700">총 {systemItems.length}개 항목 수집됨</p>
        <p class="text-[11px] text-gray-500 mt-1">상세 내용은 CSV로 내보내기하여 확인하세요.</p>
        <button
          onclick={onExport}
          disabled={systemLoading || systemItems.length === 0}
          class="mt-2 px-4 py-2 text-xs bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded-lg transition-colors font-semibold"
        >
          CSV 내보내기
        </button>
      </div>
    </div>
  {/if}
</div>
