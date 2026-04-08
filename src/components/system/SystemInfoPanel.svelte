<script lang="ts">
  import type { SystemInfoItem } from "../../lib/app-types";

  type Props = {
    systemLoading: boolean;
    systemItems: SystemInfoItem[];
    onRefresh: () => void;
    onExport: () => void;
    isCategoryStart: (items: SystemInfoItem[], index: number) => boolean;
  };

  let {
    systemLoading,
    systemItems,
    onRefresh,
    onExport,
    isCategoryStart,
  }: Props = $props();
</script>

<div class="flex flex-col gap-2.5 h-full">
  <div class="flex items-center justify-between shrink-0">
    <h2 class="text-base font-bold text-gray-800">시스템 상세 정보</h2>
    <div class="flex gap-2">
      <button
        onclick={onRefresh}
        disabled={systemLoading}
        class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded-lg transition-colors"
      >
        새로고침
      </button>
      <button
        onclick={onExport}
        disabled={systemLoading || systemItems.length === 0}
        class="px-3 py-1.5 text-xs bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded-lg transition-colors"
      >
        CSV 내보내기
      </button>
    </div>
  </div>

  {#if systemLoading && systemItems.length === 0}
    <div class="flex-1 flex items-center justify-center text-sm text-gray-400">
      데이터를 수집하고 있습니다...
    </div>
  {:else}
    <div class="flex-1 overflow-auto rounded-xl border border-gray-200 shadow-sm bg-white">
      <table class="w-full table-fixed text-sm border-collapse bg-white">
        <thead class="bg-gray-100 sticky top-0 z-10">
          <tr>
            <th class="text-left px-3 py-2 border-b w-28 font-semibold text-gray-700">분류</th>
            <th class="text-left px-3 py-2 border-b w-40 font-semibold text-gray-700">항목</th>
            <th class="text-left px-3 py-2 border-b font-semibold text-gray-700">값</th>
          </tr>
        </thead>
        <tbody>
          {#each systemItems as item, i}
            {@const isStart = isCategoryStart(systemItems, i)}
            <tr class="{isStart ? 'border-t-2 border-gray-200' : ''} {i % 2 === 0 ? 'bg-white' : 'bg-gray-50'} hover:bg-blue-50 transition-colors align-top">
              <td class="px-3 py-2 border-b {isStart ? 'font-bold text-slate-700' : 'text-transparent'} text-xs align-top">
                {isStart ? item.category : ""}
              </td>
              <td class="px-3 py-2 border-b text-gray-600 text-xs align-top">{item.item}</td>
              <td class="px-3 py-2 border-b text-gray-900 text-xs break-words align-top">{item.value}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
