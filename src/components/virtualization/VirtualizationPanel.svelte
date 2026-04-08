<script lang="ts">
  import StatusBadge from "../common/StatusBadge.svelte";
  import SummaryCard from "../common/SummaryCard.svelte";
  import type { VirtItem } from "../../lib/app-types";

  type Props = {
    virtLoading: boolean;
    virtChecked: boolean;
    virtItems: VirtItem[];
    actionGroupCount: number;
    actionItemTotal: number;
    healthyItemTotal: number;
    unknownItemTotal: number;
    onReload: () => void;
    onExport: () => void;
    onShowDisable: () => void;
  };

  let {
    virtLoading,
    virtChecked,
    virtItems,
    actionGroupCount,
    actionItemTotal,
    healthyItemTotal,
    unknownItemTotal,
    onReload,
    onExport,
    onShowDisable,
  }: Props = $props();
</script>

<div class="flex flex-col gap-3 h-full">
  <div class="flex items-center justify-between shrink-0">
    <h2 class="text-base font-bold text-gray-800">가상화 설정 점검</h2>
    <div class="flex gap-2">
      <button
        onclick={onReload}
        disabled={virtLoading}
        class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded transition-colors"
      >
        재점검
      </button>
      <button
        onclick={onExport}
        disabled={virtLoading || virtItems.length === 0}
        class="px-3 py-1.5 text-xs bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded transition-colors"
      >
        CSV 내보내기
      </button>
    </div>
  </div>

  {#if virtLoading}
    <div class="flex-1 flex flex-col items-center justify-center gap-3 text-gray-400">
      <div class="w-8 h-8 border-4 border-amber-200 border-t-amber-500 rounded-full animate-spin"></div>
      <span class="text-sm">가상화 설정 점검 중...</span>
    </div>
  {:else}
    {#if virtItems.length > 0}
      <div class="grid grid-cols-3 gap-3 shrink-0">
        <SummaryCard title="조치 필요" value={actionItemTotal} description="실제 항목 수" tone="danger" />
        <SummaryCard title="정상/비활성" value={healthyItemTotal} description="추가 조치 불필요" tone="success" />
        <SummaryCard title="확인 불가" value={unknownItemTotal} description="수동 확인 권장" tone="neutral" />
      </div>
    {/if}

    <div class="flex-1 overflow-auto rounded shadow-sm">
      <table class="w-full text-sm border-collapse bg-white">
        <thead class="bg-gray-100 sticky top-0 z-10">
          <tr>
            <th class="text-left px-3 py-2 border-b w-44 font-semibold text-gray-700">항목</th>
            <th class="text-left px-3 py-2 border-b w-28 font-semibold text-gray-700">상태</th>
            <th class="text-left px-3 py-2 border-b font-semibold text-gray-700">상세 정보</th>
            <th class="text-left px-3 py-2 border-b w-48 font-semibold text-gray-700">권장사항</th>
          </tr>
        </thead>
        <tbody>
          {#each virtItems as item, i}
            {@const isReference = item.status.includes("(참고)")}
            <tr class="{i % 2 === 0 ? 'bg-white' : 'bg-gray-50'} {isReference ? 'hover:bg-slate-50' : item.recommendation ? 'hover:bg-red-50' : 'hover:bg-green-50'} transition-colors">
              <td class="px-3 py-2 border-b font-medium text-gray-800 text-xs">{item.category}</td>
              <td class="px-3 py-2 border-b">
                <StatusBadge
                  label={item.status}
                  tone={isReference ? "neutral" : item.status.includes("활성화됨") || item.status.includes("설치됨 (활성)") ? "danger" : "success"}
                  className="font-medium"
                />
              </td>
              <td class="px-3 py-2 border-b text-gray-500 text-xs">{item.details}</td>
              <td class="px-3 py-2 border-b text-xs">
                {#if item.recommendation}
                  <span class={isReference ? "text-slate-600" : "text-amber-700"}>
                    {item.recommendation}
                  </span>
                {:else}
                  <span class="text-green-600">정상</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    {#if virtChecked && virtItems.length > 0}
      <div class="shrink-0 flex items-center justify-between rounded px-4 py-2.5 text-sm {actionGroupCount > 0 ? 'bg-amber-50 border border-amber-200' : 'bg-green-50 border border-green-200'}">
        {#if actionGroupCount > 0}
          <span class="text-amber-800 font-semibold">⚠️ {actionGroupCount}개 작업 그룹에서 조치가 필요합니다</span>
          <button
            onclick={onShowDisable}
            class="text-xs bg-red-500 hover:bg-red-600 text-white px-3 py-1.5 rounded font-bold transition-colors"
          >
            비활성화 실행 →
          </button>
        {:else}
          <span class="text-green-800 font-semibold">✅ 모든 항목이 VM 호환 상태입니다</span>
        {/if}
      </div>
    {/if}
  {/if}
</div>
