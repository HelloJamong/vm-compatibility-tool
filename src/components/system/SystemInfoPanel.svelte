<script lang="ts">
  import type { SystemInfoItem } from "../../lib/app-types";

  type Props = {
    systemLoading: boolean;
    systemItems: SystemInfoItem[];
    onRefresh: () => void;
  };

  let {
    systemLoading,
    systemItems,
    onRefresh,
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
        <p class="text-[11px] text-gray-500 mt-1">상세 내용은 자동 저장된 CSV 파일에서 확인하세요.</p>
      </div>
    </div>
  {/if}
</div>
