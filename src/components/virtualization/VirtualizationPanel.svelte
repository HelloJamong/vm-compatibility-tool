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

  let actionItems = $derived(
    virtItems.filter(item => item.action_required || item.manifest_id === "whfb_check")
  );
</script>

<div class="flex flex-col gap-2.5 h-full">
  <div class="flex items-center justify-between shrink-0">
    <h2 class="text-base font-bold text-gray-800">가상화 설정 점검</h2>
    <div class="flex gap-2">
      <button
        onclick={onReload}
        disabled={virtLoading}
        class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded-lg transition-colors"
      >
        재점검
      </button>
      <button
        onclick={onExport}
        disabled={virtLoading || virtItems.length === 0}
        class="px-3 py-1.5 text-xs bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded-lg transition-colors"
      >
        CSV 내보내기
      </button>
    </div>
  </div>

  {#if virtLoading && virtItems.length === 0}
    <div class="flex-1 flex items-center justify-center text-sm text-gray-400">
      가상화 상태를 점검하고 있습니다...
    </div>
  {:else}
    {#if virtItems.length > 0}
      <div class="grid grid-cols-3 gap-2 shrink-0">
        <SummaryCard title="조치 필요" value={actionItemTotal} description="실제 항목 수" tone="danger" />
        <SummaryCard title="정상/비활성" value={healthyItemTotal} description="추가 조치 불필요" tone="success" />
        <SummaryCard title="확인 불가" value={unknownItemTotal} description="수동 확인 권장" tone="neutral" />
      </div>
    {/if}

    <div class="flex-1 overflow-auto">
      {#if actionItems.length === 0}
        <div class="h-full flex items-center justify-center rounded-xl border border-green-200 bg-green-50">
          <p class="text-sm text-green-700 font-semibold">✅ 모든 항목이 VM 호환 상태입니다. 조치가 필요하지 않습니다.</p>
        </div>
      {:else}
        <div class="rounded-xl border border-gray-200 shadow-sm bg-white overflow-hidden">
          <table class="w-full table-fixed text-sm border-collapse bg-white">
            <thead class="bg-gray-100 sticky top-0 z-10">
              <tr>
                <th class="text-left px-3 py-2 border-b w-44 font-semibold text-gray-700">항목</th>
                <th class="text-left px-3 py-2 border-b w-32 font-semibold text-gray-700">상태</th>
                <th class="text-left px-3 py-2 border-b font-semibold text-gray-700">권장사항</th>
              </tr>
            </thead>
            <tbody>
              {#each actionItems as item, i}
                {@const isWhfbCheck = item.manifest_id === "whfb_check"}
                <tr class="{i % 2 === 0 ? 'bg-white' : 'bg-gray-50'} {isWhfbCheck ? 'hover:bg-amber-50' : 'hover:bg-red-50'} transition-colors align-top">
                  <td class="px-3 py-2 border-b font-medium text-gray-800 text-xs align-top break-words">{item.category}</td>
                  <td class="px-3 py-2 border-b align-top">
                    <StatusBadge
                      label={item.status}
                      tone={isWhfbCheck ? "info" : "danger"}
                      className="font-medium"
                    />
                  </td>
                  <td class="px-3 py-2 border-b text-xs align-top break-words">
                    {#if item.recommendation}
                      <span class={isWhfbCheck ? "text-blue-700" : "text-amber-700"}>{item.recommendation}</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        <p class="mt-2 text-[11px] text-gray-400 text-right">전체 점검 결과는 CSV 내보내기로 확인하세요.</p>
      {/if}
    </div>

    {#if virtChecked && virtItems.length > 0}
      <div class="shrink-0 flex items-center justify-between rounded-xl px-4 py-2 text-sm {actionGroupCount > 0 ? 'bg-amber-50 border border-amber-200' : 'bg-green-50 border border-green-200'}">
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
