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

  function issueTone(summary: string): "danger" | "warning" {
    return summary.includes("확인 불가") ? "warning" : "danger";
  }
</script>

{#if open}
  <div class="fixed inset-0 z-40 bg-slate-950/38 backdrop-blur-[2px] flex items-center justify-center p-4">
    <div class="w-full max-w-[472px] overflow-hidden border border-[#e8eef6] bg-white shadow-[0_10px_30px_rgba(31,61,120,0.08)]">
      <div class="flex min-h-[770px] flex-col bg-[linear-gradient(180deg,#f7f9fc_0%,#f7f9fc_83%,#e8edf4_83%,#e8edf4_100%)]">
        <div class="flex-1 px-[18px] pt-9">
          <div class="mx-auto flex max-w-[390px] flex-col items-center text-center">
            <div class="mb-[22px] flex h-[84px] w-[84px] items-center justify-center rounded-full bg-[#eef2f7]">
              <svg
                viewBox="0 0 48 48"
                class="h-[42px] w-[42px]"
                aria-hidden="true"
              >
                <circle cx="21" cy="21" r="15" fill="#2f67ea" opacity="0.12"></circle>
                <circle cx="21" cy="21" r="10.5" fill="#2f67ea"></circle>
                <path d="M28.5 28.5L38 38" stroke="#2f67ea" stroke-width="4.2" stroke-linecap="round"></path>
                <path d="M17 16.5V25.5M21 14V28M25 18V25" stroke="white" stroke-width="2.4" stroke-linecap="round"></path>
              </svg>
            </div>

            <h2 class="m-0 text-[24px] font-[800] leading-[1.35] tracking-[-0.03em] text-[#1d2740]">
              {complete ? "시스템 점검 완료" : "시스템 점검 중"}
            </h2>

            <p class="mt-[14px] max-w-[390px] break-keep text-[14px] leading-[1.8] tracking-[-0.02em] text-[#7f8ea3]">
              {#if complete}
                최초 실행에 따른 하드웨어 및 가상화 설정 검사가 완료되었습니다. 아래 항목에 대한 조치 필요 여부를 확인하세요.
              {:else}
                최초 실행에 따른 하드웨어 및 가상화 설정 검사를 진행하고 있습니다. 잠시만 기다려주세요.
              {/if}
            </p>

            {#if !complete}
              <div class="mt-6 w-full rounded-[18px] border border-[#dce5f0] bg-[rgba(245,247,251,0.82)] p-[18px] text-left">
                <div class="mb-3 flex items-center justify-between gap-3">
                  <span class="text-[14px] font-[700] tracking-[-0.02em] text-[#4d5d73]">점검 상태</span>
                  <span class="whitespace-nowrap text-[14px] font-[700] tracking-[-0.02em] text-[#2f67ea]">점검 중 {progressPercent}%</span>
                </div>
                <div class="mb-3 h-2.5 w-full overflow-hidden rounded-full bg-[#dce5f0]">
                  <div
                    class="h-full rounded-full bg-[#2f67ea] transition-[width] duration-300"
                    style={`width: ${Math.max(0, Math.min(100, progressPercent))}%`}
                  ></div>
                </div>
                <p class="m-0 text-[13px] leading-[1.7] tracking-[-0.02em] text-[#4a5970]">
                  시스템 정보와 가상화 관련 항목을 순차적으로 확인하고 있습니다.
                </p>
              </div>
            {:else if actionSummaries.length === 0}
              <div class="mt-[34px] w-full rounded-[18px] border border-[#dce5f0] bg-[rgba(245,247,251,0.82)] p-[18px] text-left">
                <div class="mb-[14px] flex items-center justify-between gap-3">
                  <span class="text-[14px] font-[700] tracking-[-0.02em] text-[#4d5d73]">점검 결과 요약</span>
                  <span class="whitespace-nowrap text-[14px] font-[700] tracking-[-0.02em] text-[#2f67ea]">특이사항 없음</span>
                </div>
                <p class="m-0 text-[13px] leading-[1.7] tracking-[-0.02em] text-[#4a5970]">
                  조치가 필요한 항목이 없습니다.
                </p>
                {#if savedFilename}
                  <p class="mt-2 text-[13px] leading-[1.7] tracking-[-0.02em] text-[#4a5970]">
                    점검 결과가 <span class="font-[700] text-[#182235]">"{savedFilename}"</span> 로 저장되었습니다.
                  </p>
                {:else if saveError}
                  <p class="mt-2 text-[13px] leading-[1.7] tracking-[-0.02em] text-[#ef4b4b]">
                    점검 결과 저장 실패: {saveError}
                  </p>
                {/if}
              </div>
            {:else}
              <div class="mt-[34px] w-full rounded-[18px] border border-[#dce5f0] bg-[rgba(245,247,251,0.82)] p-[18px] text-left">
                <div class="mb-[14px] flex items-center justify-between gap-3">
                  <span class="text-[14px] font-[700] tracking-[-0.02em] text-[#4d5d73]">점검 결과 요약</span>
                  <span class="whitespace-nowrap text-[14px] font-[700] tracking-[-0.02em] text-[#ef4b4b]">조치 대상 {actionSummaries.length}건 확인</span>
                </div>

                <div class="flex flex-col gap-[10px]">
                  {#each actionSummaries as summary}
                    <div class="flex items-center gap-[10px] text-[13px] leading-[1.5] tracking-[-0.02em] text-[#4a5970]">
                      <span
                        class={`inline-flex h-[18px] w-[18px] flex-none items-center justify-center rounded-full text-[11px] font-[700] text-white ${issueTone(summary) === "warning" ? "bg-[#f2a316]" : "bg-[#ef4b4b]"}`}
                      >
                        !
                      </span>
                      <span>{summary}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <div class="mt-9 flex w-full flex-col gap-[14px]">
              <button
                onclick={onStartAction}
                disabled={!canStartAction}
                class="min-h-[54px] w-full rounded-[14px] border border-transparent bg-[#2f67ea] text-[17px] font-[800] tracking-[-0.02em] text-white shadow-[0_8px_18px_rgba(47,103,234,0.18)] transition-all hover:bg-[#2557cf] active:translate-y-px disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-400 disabled:shadow-none"
              >
                조치 시작
              </button>

              <button
                onclick={onClose}
                class="min-h-[54px] w-full rounded-[14px] border border-[#d7e0eb] bg-white/35 text-[17px] font-[800] tracking-[-0.02em] text-[#697a92] transition-all hover:bg-white/60 active:translate-y-px"
              >
                닫기
              </button>
            </div>
          </div>
        </div>
        <div class="flex min-h-[74px] items-center justify-center px-5 pb-4 pt-[18px]">
          <p class="text-center text-[12px] font-[800] uppercase tracking-[0.12em] text-[#9aa8bc]">
            VM Compatibility Tool Startup Scan
          </p>
        </div>
      </div>
    </div>
  </div>
{/if}
