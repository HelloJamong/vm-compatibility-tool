<script lang="ts">
  import StatusBadge from "./StatusBadge.svelte";

  type Props = {
    open: boolean;
    complete: boolean;
    progressPercent: number;
    actionSummaries: string[];
    savedFilename: string | null;
    saveError: string | null;
    onStartAction: () => void;
    onClose: () => void;
  };

  let {
    open,
    complete,
    progressPercent,
    actionSummaries,
    savedFilename,
    saveError,
    onStartAction,
    onClose,
  }: Props = $props();

  const canStartAction = $derived(complete && actionSummaries.length > 0);
</script>

{#if open}
  <div class="fixed inset-0 z-40 bg-slate-950/55 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-[500px] rounded-[28px] bg-white shadow-2xl border border-slate-200 overflow-hidden">
      <div class="px-8 pt-8 pb-6 text-center border-b border-slate-100">
        <div class="w-20 h-20 mx-auto mb-5 rounded-full bg-slate-100 flex items-center justify-center text-3xl">
          {#if complete}
            ✅
          {:else}
            🔎
          {/if}
        </div>

        <div class="flex items-center justify-center gap-2 mb-3">
          <h2 class="text-2xl font-bold text-slate-900">{complete ? "점검 완료" : "점검 중"}</h2>
          <StatusBadge
            label={complete ? "점검 완료" : "점검 중"}
            tone={complete ? "success" : "info"}
            className="font-semibold"
          />
        </div>

        <p class="text-sm leading-relaxed text-slate-500 mb-5">
          {#if complete}
            자동 점검이 완료되었습니다. 아래 요약을 확인한 뒤 필요한 경우 바로 조치를 시작할 수 있습니다.
          {:else}
            시스템 정보와 가상화 설정을 자동으로 점검하고 있습니다. 잠시만 기다려주세요.
          {/if}
        </p>

        <div class="max-w-[280px] mx-auto">
          <div class="flex items-center justify-between text-xs font-semibold text-slate-500 mb-2">
            <span>{complete ? "진행 완료" : "점검 진행률"}</span>
            <span>{progressPercent}%</span>
          </div>
          <div class="w-full h-2 rounded-full bg-slate-200 overflow-hidden">
            <div
              class={`h-full rounded-full transition-[width] duration-300 ${complete ? "bg-emerald-500" : "bg-blue-600"}`}
              style={`width: ${Math.max(0, Math.min(100, progressPercent))}%`}
            ></div>
          </div>
        </div>
      </div>

      <div class="px-8 py-6">
        {#if !complete}
          <div class="rounded-2xl border border-slate-200 bg-slate-50 p-5 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 상태</span>
              <span class="text-xs font-bold text-blue-600">진행 중</span>
            </div>
            <p class="text-sm text-slate-700">하드웨어 및 가상화 관련 항목을 순차적으로 확인하고 있습니다.</p>
          </div>
        {:else if actionSummaries.length === 0}
          <div class="rounded-2xl border border-emerald-200 bg-emerald-50 p-5 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 결과 요약</span>
              <span class="text-xs font-bold text-emerald-700">특이사항 없음</span>
            </div>
            <p class="text-sm font-semibold text-emerald-900">조치가 필요한 항목이 없습니다.</p>
            {#if savedFilename}
              <p class="mt-2 text-sm text-slate-700">점검 결과가 <span class="font-semibold text-slate-900">"{savedFilename}"</span> 로 저장되었습니다.</p>
            {:else if saveError}
              <p class="mt-2 text-sm text-amber-700">점검 결과 저장 실패: {saveError}</p>
            {/if}
          </div>
        {:else}
          <div class="rounded-2xl border border-red-200 bg-red-50 p-5 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 결과 요약</span>
              <span class="text-xs font-bold text-red-700">조치 필요 {actionSummaries.length}건</span>
            </div>
            <div class="space-y-2">
              {#each actionSummaries as summary}
                <div class="flex items-start gap-3 text-sm text-slate-800">
                  <span class="mt-0.5 text-red-500">●</span>
                  <span>{summary}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <div class="px-8 py-5 bg-slate-50 border-t border-slate-100 flex flex-col gap-3">
        <button
          onclick={onStartAction}
          disabled={!canStartAction}
          class="w-full py-3.5 rounded-xl font-bold text-white transition-colors disabled:bg-slate-300 disabled:text-slate-500 disabled:cursor-not-allowed bg-blue-600 hover:bg-blue-700"
        >
          조치 시작
        </button>
        <button
          onclick={onClose}
          class="w-full py-3.5 rounded-xl font-bold text-slate-600 border border-slate-200 bg-white hover:bg-slate-100 transition-colors"
        >
          닫기
        </button>
      </div>
    </div>
  </div>
{/if}
