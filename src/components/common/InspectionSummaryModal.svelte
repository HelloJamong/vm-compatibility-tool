<script lang="ts">
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

        <div class="mb-3">
          <div class={`text-sm font-semibold mb-2 ${complete ? "text-emerald-600" : "text-blue-600"}`}>
            {complete ? "점검 완료" : `점검 중 ${progressPercent}%`}
          </div>
          <h2 class="text-[38px] leading-tight font-bold tracking-[-0.03em] text-slate-900">
            {complete ? "시스템 점검 완료" : "시스템 점검 중"}
          </h2>
        </div>

        <p class="text-[22px] leading-[1.65] text-slate-500 mb-8">
          {#if complete}
            자동 점검이 완료되었습니다. 아래 요약을 확인한 뒤 필요한 경우 바로 조치를 시작할 수 있습니다.
          {:else}
            시스템 정보와 가상화 설정을 자동으로 점검하고 있습니다. 잠시만 기다려주세요.
          {/if}
        </p>

        {#if !complete}
          <div class="max-w-[320px] mx-auto">
            <div class="flex items-center justify-between text-sm font-semibold text-slate-500 mb-2">
              <span>점검 진행률</span>
              <span>{progressPercent}%</span>
            </div>
            <div class="w-full h-2.5 rounded-full bg-slate-200 overflow-hidden">
              <div
                class="h-full rounded-full transition-[width] duration-300 bg-blue-600"
                style={`width: ${Math.max(0, Math.min(100, progressPercent))}%`}
              ></div>
            </div>
          </div>
        {/if}
      </div>

      <div class="px-8 py-6">
        {#if !complete}
          <div class="rounded-[22px] border border-slate-200 bg-slate-50 p-6 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 상태</span>
              <span class="text-xs font-bold text-blue-600">진행 중</span>
            </div>
            <p class="text-base leading-7 text-slate-700">하드웨어 및 가상화 관련 항목을 순차적으로 확인하고 있습니다.</p>
          </div>
        {:else if actionSummaries.length === 0}
          <div class="rounded-[22px] border border-slate-200 bg-slate-50 p-6 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 결과 요약</span>
              <span class="text-xs font-bold text-emerald-600">특이사항 없음</span>
            </div>
            <p class="text-lg font-semibold text-slate-900">조치가 필요한 항목이 없습니다.</p>
            {#if savedFilename}
              <p class="mt-3 text-base leading-7 text-slate-700">점검 결과가 <span class="font-semibold text-slate-900">"{savedFilename}"</span> 로 저장되었습니다.</p>
            {:else if saveError}
              <p class="mt-3 text-base leading-7 text-amber-700">점검 결과 저장 실패: {saveError}</p>
            {/if}
          </div>
        {:else}
          <div class="rounded-[22px] border border-slate-200 bg-slate-50 p-6 text-left">
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-bold text-slate-600">점검 결과 요약</span>
              <span class="text-xs font-bold text-red-500">조치 대상 {actionSummaries.length}건 확인</span>
            </div>
            <div class="space-y-2">
              {#each actionSummaries as summary}
                <div class="flex items-start gap-3 text-[15px] leading-7 text-slate-700">
                  <span class={`mt-2 h-2.5 w-2.5 rounded-full ${summary.includes("확인 불가") ? "bg-amber-400" : "bg-red-500"}`}></span>
                  <span>{summary}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <div class="px-8 py-5 flex flex-col gap-4">
        <button
          onclick={onStartAction}
          disabled={!canStartAction}
          class="w-full py-4 rounded-2xl text-xl font-bold text-white transition-all disabled:bg-slate-200 disabled:text-slate-400 disabled:cursor-not-allowed bg-blue-600 hover:bg-blue-700 shadow-[0_16px_30px_-18px_rgba(37,99,235,0.8)]"
        >
          자동 최적화 및 조치 시작
        </button>
        <button
          onclick={onClose}
          class="w-full py-4 rounded-2xl text-xl font-bold text-slate-500 border border-slate-200 bg-white hover:bg-slate-50 transition-colors"
        >
          {canStartAction ? "대시보드로 이동" : "닫기"}
        </button>
      </div>

      <div class="px-8 py-4 bg-slate-100 text-center">
        <p class="text-[11px] font-semibold tracking-[0.22em] text-slate-400 uppercase">VM Compatibility Tool {complete ? "inspection flow" : "startup scan"}</p>
      </div>
    </div>
  </div>
{/if}
