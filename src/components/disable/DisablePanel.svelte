<script lang="ts">
  import StatusBadge from "../common/StatusBadge.svelte";
  import type { DisableOptions } from "../../lib/app-types";

  type Props = {
    virtChecked: boolean;
    disableRunning: boolean;
    disableComplete: boolean;
    disableLog: string[];
    disableOptions: DisableOptions;
    selectedTaskCount: number;
    optionalCandidateCount: number;
    whfbDetected: boolean;
    onRunDisable: () => void;
    onRequestReboot: () => void;
    onLoadVirtStatus: () => void;
    logLineClass: (line: string) => string;
  };

  let {
    virtChecked,
    disableRunning,
    disableComplete,
    disableLog,
    disableOptions,
    selectedTaskCount,
    optionalCandidateCount,
    whfbDetected,
    onRunDisable,
    onRequestReboot,
    onLoadVirtStatus,
    logLineClass,
  }: Props = $props();
</script>

<div class="flex flex-col gap-2.5 h-full">
  <h2 class="text-base font-bold text-gray-800 shrink-0">VBS 및 Hyper-V 비활성화</h2>

  {#if whfbDetected}
    <div class="bg-amber-50 border border-amber-300 rounded-xl p-3 text-sm shrink-0">
      <p class="font-bold text-amber-800 mb-1">⚠️ Windows Hello for Business 감지됨</p>
      <p class="text-xs text-amber-700">VBS 레지스트리는 재부팅 후 복구될 수 있습니다. 가상화 점검 결과의 <span class="font-semibold">Windows Hello</span> 항목을 먼저 확인하세요.</p>
    </div>
  {/if}

  <div class="bg-red-50 border border-red-200 rounded-xl p-3 text-sm text-red-800 shrink-0">
    <p class="font-bold mb-1.5">⚠️ 이 작업은 다음을 수행합니다</p>
    <ul class="list-disc list-inside space-y-0.5 text-xs text-red-700">
      <li>Hyper-V 및 관련 기능 제거 (DISM)</li>
      <li>WSL 기능 제거 (DISM)</li>
      <li>VBS (가상화 기반 보안) 레지스트리 비활성화</li>
      <li>코어 격리 비활성화</li>
      <li>Hypervisor 시작 유형 비활성화 (bcdedit)</li>
    </ul>
  </div>

  {#if virtChecked}
    <div class="bg-blue-50 border border-blue-200 rounded-xl p-3 shrink-0">
      <div class="flex items-center justify-between gap-3 mb-2">
        <p class="text-xs font-semibold text-blue-800">점검 결과 기반 실행 예정 항목</p>
        <StatusBadge
          label={`${selectedTaskCount}개 작업 선택`}
          tone={selectedTaskCount > 0 ? "info" : "neutral"}
        />
      </div>
      <div class="grid grid-cols-2 gap-1.5">
        {#each [
          { label: "Hyper-V 비활성화", on: disableOptions.hyperv },
          { label: "WSL 비활성화", on: disableOptions.wsl },
          { label: "VBS 레지스트리 비활성화", on: disableOptions.vbs },
          { label: "코어 격리 비활성화", on: disableOptions.core_isolation },
        ] as task}
          <div class="flex items-center gap-1.5 text-xs">
            <span class="{task.on ? 'text-red-500' : 'text-gray-300'}">●</span>
            <span class="{task.on ? 'text-gray-800 font-medium' : 'text-gray-400 line-through'}">
              {task.label}
            </span>
          </div>
        {/each}
      </div>
      {#if optionalCandidateCount > 0}
        <p class="mt-2 text-[11px] text-amber-700">
          추가 레지스트리 조치 {optionalCandidateCount}개는 실행 전 확인 창에서 체크박스로 선택할 수 있습니다.
        </p>
      {/if}
    </div>
  {:else}
    <div class="bg-amber-50 border border-amber-200 rounded-xl px-3 py-2 text-xs text-amber-700 shrink-0">
      ℹ️ 가상화 점검 없이 모든 항목을 일괄 처리합니다.
      <button onclick={onLoadVirtStatus} class="ml-1 underline hover:text-amber-900">
        지금 점검하기
      </button>
    </div>
  {/if}

  <div class="flex gap-3 shrink-0">
    <button
      onclick={onRunDisable}
      disabled={disableRunning}
        class="px-5 py-2.5 font-bold bg-red-500 hover:bg-red-600 disabled:bg-gray-300 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
    >
      {disableRunning ? "실행 중..." : "비활성화 실행"}
    </button>
    {#if disableComplete}
      <button
        onclick={onRequestReboot}
        class="px-5 py-2.5 font-bold bg-slate-700 hover:bg-slate-800 text-white rounded-lg transition-colors"
      >
        🔄 지금 재부팅
      </button>
    {/if}
  </div>

  <div class="flex-1 overflow-hidden">
    <div class="h-full bg-gray-900 rounded-xl p-4 text-xs font-mono overflow-y-auto">
      {#if disableLog.length === 0}
        <span class="text-gray-500">비활성화 실행 버튼을 클릭하면 작업이 시작됩니다.</span>
      {:else}
        {#each disableLog as line}
          <div class="{logLineClass(line)} leading-5">{line || "\u00A0"}</div>
        {/each}
      {/if}
    </div>
  </div>
</div>
