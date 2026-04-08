<script lang="ts">
  import StatusBadge from "../common/StatusBadge.svelte";

  type Props = {
    virtChecked: boolean;
    actionGroupCount: number;
    onLoadSystemInfo: () => void;
    onLoadVirtStatus: () => void;
    onShowDisable: () => void;
  };

  let {
    virtChecked,
    actionGroupCount,
    onLoadSystemInfo,
    onLoadVirtStatus,
    onShowDisable,
  }: Props = $props();
</script>

<div class="h-full flex items-center justify-center">
  <div class="w-full max-w-[420px] rounded-2xl border border-gray-200 bg-white shadow-sm px-5 py-6 flex flex-col gap-3">
    <div class="mb-1">
      <p class="text-sm font-semibold text-gray-700">작업을 선택하세요</p>
      <p class="text-xs text-gray-400">필요한 진단 또는 조치 기능을 바로 실행할 수 있습니다.</p>
    </div>

    <button
      onclick={onLoadSystemInfo}
      class="w-full px-4 py-3 text-left bg-blue-500 hover:bg-blue-600 text-white rounded-xl transition-colors shadow-sm"
    >
      <div class="text-sm font-bold">🖥️ 시스템 사양 체크</div>
      <div class="mt-1 text-[11px] text-blue-100">OS / CPU / 메모리 / 디스크 / 이벤트 로그 요약</div>
    </button>

    <button
      onclick={onLoadVirtStatus}
      class="w-full px-4 py-3 text-left bg-amber-500 hover:bg-amber-600 text-white rounded-xl transition-colors shadow-sm"
    >
      <div class="flex items-center justify-between gap-2">
        <span class="text-sm font-bold">🔍 가상화 설정 점검</span>
        {#if virtChecked}
          <StatusBadge
            label={actionGroupCount > 0 ? `${actionGroupCount}개 조치 필요` : "정상"}
            tone={actionGroupCount > 0 ? "danger" : "success"}
          />
        {/if}
      </div>
      <div class="mt-1 text-[11px] text-amber-100">Hyper-V / WSL / VBS / 코어 격리 상태 확인</div>
    </button>

    <button
      onclick={onShowDisable}
      class="w-full px-4 py-3 text-left bg-red-500 hover:bg-red-600 text-white rounded-xl transition-colors shadow-sm"
    >
      <div class="text-sm font-bold">⚙️ VBS 및 Hyper-V 비활성화</div>
      <div class="mt-1 text-[11px] text-red-100">점검 결과를 기준으로 필요한 조치만 선택 실행</div>
    </button>
  </div>
</div>
