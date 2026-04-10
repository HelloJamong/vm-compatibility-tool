<script lang="ts">
  type Props = {
    open: boolean;
    complete: boolean;
    progressPercent: number;
    actionSummaries: string[];
    savedFilename: string | null;
    saveError: string | null;
    version: string;
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
    version,
    onStartAction,
    onClose,
  }: Props = $props();

  const canStartAction = $derived(complete && actionSummaries.length > 0);

  function issueTone(summary: string): "danger" | "warning" {
    return summary.includes("확인 불가") ? "warning" : "danger";
  }
</script>

{#if open}
  <div class="inspection-screen">
    <div class="inspection-panel">
      <div class="inspection-content">
        <div class="status-icon-wrap" aria-hidden="true">
          <svg viewBox="0 0 48 48" class="status-icon">
            <circle cx="21" cy="21" r="15" fill="#2f67ea" opacity="0.12"></circle>
            <circle cx="21" cy="21" r="10.5" fill="#2f67ea"></circle>
            <path d="M28.5 28.5L38 38" stroke="#2f67ea" stroke-width="4.2" stroke-linecap="round"></path>
            <path d="M17 16.5V25.5M21 14V28M25 18V25" stroke="white" stroke-width="2.4" stroke-linecap="round"></path>
          </svg>
        </div>

        <h2 class="title">{complete ? "시스템 점검 완료" : "시스템 점검 중"}</h2>

        <p class="description">
          {#if complete}
            최초 실행에 따른 하드웨어 및 가상화 설정 검사가 완료되었습니다. 아래 항목에 대한 조치 필요 여부를 확인하세요.
          {:else}
            최초 실행에 따른 하드웨어 및 가상화 설정 검사를 진행하고 있습니다. 잠시만 기다려주세요.
          {/if}
        </p>

        {#if !complete}
          <section class="summary-card">
            <div class="summary-header">
              <span class="summary-title">점검 상태</span>
              <span class="summary-count summary-count--info">점검 중 {progressPercent}%</span>
            </div>
            <div class="progress-track" aria-hidden="true">
              <div class="progress-fill" style={`width: ${Math.max(0, Math.min(100, progressPercent))}%`}></div>
            </div>
            <p class="summary-message">시스템 정보와 가상화 관련 항목을 순차적으로 확인하고 있습니다.</p>
          </section>
        {:else if actionSummaries.length === 0}
          <section class="summary-card">
            <div class="summary-header">
              <span class="summary-title">점검 결과 요약</span>
              <span class="summary-count summary-count--info">특이사항 없음</span>
            </div>
            <p class="summary-message">조치가 필요한 항목이 없습니다.</p>
            {#if savedFilename}
              <p class="summary-message">점검 결과가 <strong>"{savedFilename}"</strong> 로 저장되었습니다.</p>
            {:else if saveError}
              <p class="summary-message summary-message--error">점검 결과 저장 실패: {saveError}</p>
            {/if}
          </section>
        {:else}
          <section class="summary-card">
            <div class="summary-header">
              <span class="summary-title">점검 결과 요약</span>
              <span class="summary-count">조치 대상 {actionSummaries.length}건 확인</span>
            </div>
            <div class="issue-list">
              {#each actionSummaries as summary}
                <div class="issue-item">
                  <span class={`issue-dot ${issueTone(summary)}`}>!</span>
                  <span>{summary}</span>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <div class="actions">
          <button
            class="button button-primary"
            onclick={onStartAction}
            disabled={!canStartAction}
          >
            조치 시작
          </button>
          <button class="button button-secondary" onclick={onClose}>닫기</button>
        </div>
      </div>

      <div class="inspection-footer">
        <p class="version">{version}</p>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
  }

  .inspection-screen {
    min-height: 100vh;
    display: flex;
    align-items: stretch;
    justify-content: center;
    padding: 0;
    background: linear-gradient(180deg, #eef3fa 0%, #f5f7fb 100%);
    box-sizing: border-box;
  }

  .inspection-panel {
    width: 100%;
    min-height: 100vh;
    background: #ffffff;
    display: flex;
    flex-direction: column;
  }

  .inspection-content {
    flex: 1;
    width: 100%;
    padding: 28px 10px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-sizing: border-box;
  }

  .status-icon-wrap {
    width: 84px;
    height: 84px;
    border-radius: 50%;
    background: #eef2f7;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 22px;
  }

  .status-icon {
    width: 42px;
    height: 42px;
    display: block;
  }

  .title {
    margin: 0;
    font-size: 24px;
    line-height: 1.35;
    font-weight: 800;
    letter-spacing: -0.03em;
    color: #1d2740;
    word-break: keep-all;
  }

  .description {
    margin: 14px 0 0;
    width: 100%;
    max-width: none;
    font-size: 14px;
    line-height: 1.8;
    letter-spacing: -0.02em;
    color: #7f8ea3;
    word-break: keep-all;
  }

  .summary-card {
    width: 100%;
    max-width: none;
    margin-top: 34px;
    border: 1px solid #dce5f0;
    border-radius: 18px;
    background: rgba(245, 247, 251, 0.82);
    padding: 18px 18px 16px;
    box-sizing: border-box;
    text-align: left;
  }

  .summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 14px;
  }

  .summary-title {
    font-size: 14px;
    font-weight: 700;
    color: #4d5d73;
    letter-spacing: -0.02em;
  }

  .summary-count {
    white-space: nowrap;
    font-size: 14px;
    font-weight: 700;
    color: #ef4b4b;
    letter-spacing: -0.02em;
  }

  .summary-count--info {
    color: #2f67ea;
  }

  .summary-message {
    margin: 0;
    font-size: 13px;
    line-height: 1.7;
    letter-spacing: -0.02em;
    color: #4a5970;
    word-break: keep-all;
  }

  .summary-message + .summary-message {
    margin-top: 8px;
  }

  .summary-message--error {
    color: #ef4b4b;
  }

  .progress-track {
    width: 100%;
    height: 10px;
    border-radius: 999px;
    background: #dce5f0;
    overflow: hidden;
    margin-bottom: 12px;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    background: #2f67ea;
    transition: width 0.25s ease;
  }

  .issue-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .issue-item {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    line-height: 1.5;
    color: #4a5970;
    letter-spacing: -0.02em;
    word-break: keep-all;
  }

  .issue-dot {
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 11px;
    font-weight: 700;
    color: #ffffff;
  }

  .issue-dot.danger {
    background: #ef4b4b;
  }

  .issue-dot.warning {
    background: #f2a316;
  }

  .actions {
    width: 100%;
    max-width: none;
    margin-top: 36px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .button {
    width: 100%;
    min-height: 54px;
    border-radius: 14px;
    border: 1px solid transparent;
    font-size: 17px;
    font-weight: 800;
    letter-spacing: -0.02em;
    cursor: pointer;
    transition:
      transform 0.15s ease,
      box-shadow 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease,
      color 0.15s ease;
  }

  .button:active {
    transform: translateY(1px);
  }

  .button-primary {
    background: #2f67ea;
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(47, 103, 234, 0.18);
  }

  .button-primary:hover {
    background: #2557cf;
  }

  .button-primary:disabled {
    cursor: not-allowed;
    background: #cbd5e1;
    color: #94a3b8;
    box-shadow: none;
  }

  .button-secondary {
    background: rgba(255, 255, 255, 0.22);
    border-color: #d7e0eb;
    color: #697a92;
  }

  .button-secondary:hover {
    background: rgba(255, 255, 255, 0.5);
  }

  .inspection-footer {
    min-height: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5px 20px 8px;
    background: #e8edf4;
  }

  .version {
    margin: 0;
    font-size: 12px;
    line-height: 1;
    font-weight: 800;
    letter-spacing: 0.12em;
    color: #9aa8bc;
    text-transform: uppercase;
    text-align: center;
  }

  @media (min-width: 640px) {
    .inspection-content {
      padding: 36px 12px 0;
    }
  }
</style>
