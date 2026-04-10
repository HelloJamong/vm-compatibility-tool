<script lang="ts">
  import type { DisableOptions, VirtItem } from "../../lib/app-types";

  type Stage = "warning" | "running" | "complete";

  type Props = {
    open: boolean;
    stage: Stage;
    progressPercent: number;
    currentAction: string;
    disableOptions: DisableOptions;
    optionalRegistryCandidates: VirtItem[];
    hasErrors: boolean;
    logPath: string | null;
    backupPath: string | null;
    version: string;
    onStart: () => void;
    onToggleOptionalRegistry: (manifestId: string) => void;
    onCancel: () => void;
    onRebootNow: () => void;
    onDismiss: () => void;
  };

  let {
    open,
    stage,
    progressPercent,
    currentAction,
    disableOptions,
    optionalRegistryCandidates,
    hasErrors,
    logPath,
    backupPath,
    version,
    onStart,
    onToggleOptionalRegistry,
    onCancel,
    onRebootNow,
    onDismiss,
  }: Props = $props();

  let rebootDeclined = $state(false);

  const taskList = $derived([
    { label: "Hyper-V 및 관련 기능 제거 (DISM)", on: disableOptions.hyperv },
    { label: "WSL 기능 제거 (DISM)", on: disableOptions.wsl },
    { label: "VBS 레지스트리 비활성화", on: disableOptions.vbs },
    { label: "코어 격리 비활성화", on: disableOptions.core_isolation },
    { label: "Hypervisor 시작 유형 비활성화 (bcdedit)", on: disableOptions.hyperv },
  ]);

  const activeTasks = $derived(taskList.filter((t) => t.on));
  const selectedOptionalTasks = $derived(
    optionalRegistryCandidates.filter(
      (item) => item.manifest_id && disableOptions.optional_registry_ids.includes(item.manifest_id)
    )
  );
  const totalSelectedTaskCount = $derived(activeTasks.length + selectedOptionalTasks.length);

  function basename(path: string | null): string | null {
    return path?.split(/[\\/]/).pop() ?? null;
  }
</script>

{#if open}
  <div class="action-screen">
    <div class="action-panel">
      <div class="action-content">

        <!-- ───── WARNING ───── -->
        {#if stage === "warning"}
          <div class="icon-wrap icon-warn" aria-hidden="true">
            <svg viewBox="0 0 48 48" class="icon">
              <path d="M24 7L43 39H5L24 7Z" fill="#fef2f2" stroke="#ef4444" stroke-width="2.2" stroke-linejoin="round" />
              <rect x="22.5" y="18" width="3" height="11" rx="1.5" fill="#ef4444" />
              <circle cx="24" cy="34" r="2" fill="#ef4444" />
            </svg>
          </div>

          <h2 class="title">비활성화 조치 전 주의사항</h2>
          <p class="description">
            아래 항목들이 시스템에서 비활성화 또는 제거됩니다.<br />
            계속하기 전에 실행 중인 작업을 모두 저장하세요.
          </p>

          <section class="summary-card">
            <div class="summary-header">
              <span class="summary-title">조치 예정 항목</span>
              <span class="summary-count summary-count--danger">{totalSelectedTaskCount}개 작업</span>
            </div>
            <div class="task-list">
              {#each activeTasks as task}
                <div class="task-item">
                  <span class="task-dot">!</span>
                  <span>{task.label}</span>
                </div>
              {/each}
              {#each selectedOptionalTasks as task}
                <div class="task-item task-item--optional">
                  <span class="task-dot task-dot--optional">+</span>
                  <span>{task.category}</span>
                </div>
              {/each}
            </div>
          </section>

          {#if optionalRegistryCandidates.length > 0}
            <section class="summary-card summary-card--optional">
              <div class="summary-header">
                <span class="summary-title">추가 선택 가능한 레지스트리 조치</span>
                <span class="summary-count summary-count--neutral">{optionalRegistryCandidates.length}개 후보</span>
              </div>
              <div class="optional-list">
                {#each optionalRegistryCandidates as item}
                  {@const manifestId = item.manifest_id ?? ""}
                  <label class="optional-item">
                    <input
                      type="checkbox"
                      checked={disableOptions.optional_registry_ids.includes(manifestId)}
                      onchange={() => manifestId && onToggleOptionalRegistry(manifestId)}
                    />
                    <span class="optional-copy">
                      <span class="optional-title">{item.category}</span>
                      <span class="optional-description">{item.recommendation || item.status}</span>
                    </span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          <div class="actions">
            <button class="button button-danger" onclick={onStart} disabled={totalSelectedTaskCount === 0}>조치 시작</button>
            <button class="button button-secondary" onclick={onCancel}>취소</button>
          </div>

        <!-- ───── RUNNING ───── -->
        {:else if stage === "running"}
          <div class="icon-wrap icon-running" aria-hidden="true">
            <svg viewBox="0 0 48 48" class="icon icon-spin">
              <circle cx="24" cy="24" r="17" fill="none" stroke="#e2e8f0" stroke-width="4" />
              <path d="M24 7 A17 17 0 0 1 41 24" fill="none" stroke="#2f67ea" stroke-width="4" stroke-linecap="round" />
            </svg>
          </div>

          <h2 class="title">조치 진행 중...</h2>
          <p class="description">시스템 설정을 변경하고 있습니다. 잠시만 기다려주세요.</p>

          <section class="summary-card">
            <div class="summary-header">
              <span class="summary-title">진행 상태</span>
              <span class="summary-count summary-count--info">{Math.round(progressPercent)}%</span>
            </div>
            <div class="progress-track">
              <div class="progress-fill" style={`width: ${Math.max(0, Math.min(100, progressPercent))}%`}></div>
            </div>
            <p class="summary-message">{currentAction || "조치 준비 중..."}</p>
          </section>

        <!-- ───── COMPLETE ───── -->
        {:else if stage === "complete"}
          <div class="icon-wrap {hasErrors ? 'icon-warn' : 'icon-ok'}" aria-hidden="true">
            {#if hasErrors}
              <svg viewBox="0 0 48 48" class="icon">
                <circle cx="24" cy="24" r="17" fill="#fef2f2" />
                <path d="M16 16L32 32M32 16L16 32" stroke="#ef4444" stroke-width="3.5" stroke-linecap="round" />
              </svg>
            {:else}
              <svg viewBox="0 0 48 48" class="icon">
                <circle cx="24" cy="24" r="17" fill="#f0fdf4" />
                <path d="M15 24l7 7 11-13" stroke="#22c55e" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round" fill="none" />
              </svg>
            {/if}
          </div>

          <h2 class="title">{hasErrors ? "일부 조치 실패" : "조치 완료"}</h2>

          {#if !rebootDeclined}
            <p class="description">
              변경 사항을 적용하려면 <strong>시스템 재부팅이 필요</strong>합니다.
            </p>

            {#if logPath || backupPath}
              <section class="summary-card">
                <div class="summary-header">
                  <span class="summary-title">저장된 파일</span>
                </div>
                {#if logPath}
                  <p class="summary-message">📄 {basename(logPath)}</p>
                {/if}
                {#if backupPath}
                  <p class="summary-message">💾 {basename(backupPath)}</p>
                {/if}
              </section>
            {/if}

            <p class="reboot-question">지금 재부팅하시겠습니까?</p>

            <div class="actions">
              <button class="button button-primary" onclick={onRebootNow}>
                예 — 지금 재부팅 (5초 후)
              </button>
              <button class="button button-secondary" onclick={() => (rebootDeclined = true)}>
                아니요 — 나중에 재부팅
              </button>
            </div>

          {:else}
            <p class="description">
              재부팅 전까지 변경 사항이 적용되지 않습니다.<br />
              작업이 완료되면 직접 시스템을 재부팅해 주세요.
            </p>

            <section class="summary-card summary-card--neutral">
              <p class="summary-message">
                시작 메뉴 → 전원 → 다시 시작을 통해 재부팅할 수 있습니다.
              </p>
            </section>

            <div class="actions">
              <button class="button button-secondary" onclick={onDismiss}>닫기</button>
            </div>
          {/if}
        {/if}

      </div>

      <div class="action-footer">
        <p class="version">{version}</p>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
  }

  .action-screen {
    min-height: 100vh;
    display: flex;
    align-items: stretch;
    justify-content: center;
    background: linear-gradient(180deg, #eef3fa 0%, #f5f7fb 100%);
  }

  .action-panel {
    width: 100%;
    min-height: 100vh;
    background: #ffffff;
    display: flex;
    flex-direction: column;
  }

  .action-content {
    flex: 1;
    width: 100%;
    padding: 28px 20px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-sizing: border-box;
  }

  .icon-wrap {
    width: 84px;
    height: 84px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;
  }

  .icon-warn    { background: #fef2f2; }
  .icon-ok      { background: #f0fdf4; }
  .icon-running { background: #eef2f7; }

  .icon {
    width: 44px;
    height: 44px;
    display: block;
  }

  .icon-spin {
    animation: spin 1.2s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }

  .title {
    margin: 0 0 10px;
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.03em;
    color: #1d2740;
    word-break: keep-all;
  }

  .description {
    margin: 0 0 18px;
    font-size: 13px;
    line-height: 1.65;
    color: #5a6a82;
    word-break: keep-all;
  }

  .summary-card {
    width: 100%;
    background: #f3f6fb;
    border-radius: 14px;
    padding: 14px 16px;
    margin-bottom: 14px;
    text-align: left;
    box-sizing: border-box;
  }

  .summary-card--neutral {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
  }

  .summary-card--optional {
    background: #fffbeb;
    border: 1px solid #fcd34d;
  }

  .summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .summary-title {
    font-size: 11px;
    font-weight: 700;
    color: #3d5170;
    text-transform: uppercase;
    letter-spacing: 0.07em;
  }

  .summary-count {
    font-size: 12px;
    font-weight: 700;
    border-radius: 6px;
    padding: 2px 8px;
  }

  .summary-count--danger {
    color: #ef4444;
    background: #fef2f2;
  }

  .summary-count--info {
    color: #2f67ea;
    background: #eef2fb;
  }

  .summary-count--neutral {
    color: #92400e;
    background: #fef3c7;
  }

  .summary-message {
    margin: 4px 0 0;
    font-size: 12px;
    color: #5a6a82;
    line-height: 1.55;
  }

  .task-item--optional {
    color: #92400e;
  }

  .task-dot--optional {
    background: #f59e0b;
  }

  .optional-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .optional-item {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    font-size: 13px;
    color: #3d5170;
    cursor: pointer;
  }

  .optional-item input {
    margin-top: 2px;
    width: 16px;
    height: 16px;
    accent-color: #d97706;
    cursor: pointer;
  }

  .optional-copy {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .optional-title {
    font-weight: 700;
    color: #7c2d12;
  }

  .optional-description {
    font-size: 12px;
    line-height: 1.5;
    color: #92400e;
  }

  .task-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 12px;
    color: #3d5170;
  }

  .task-dot {
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
    background: #ef4444;
  }

  .progress-track {
    width: 100%;
    height: 8px;
    background: #e2e8f0;
    border-radius: 99px;
    overflow: hidden;
    margin: 8px 0 10px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #2f67ea, #60a5fa);
    border-radius: 99px;
    transition: width 0.3s ease;
  }

  .reboot-question {
    margin: 4px 0 14px;
    font-size: 15px;
    font-weight: 700;
    color: #1d2740;
  }

  .actions {
    width: 100%;
    margin-top: 4px;
    margin-bottom: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .button {
    width: 100%;
    min-height: 52px;
    border-radius: 14px;
    border: 1px solid transparent;
    font-size: 16px;
    font-weight: 800;
    letter-spacing: -0.02em;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .button:active { transform: translateY(1px); }
  .button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
    box-shadow: none;
  }

  .button-primary {
    background: #2f67ea;
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(47, 103, 234, 0.18);
  }
  .button-primary:hover { background: #2557cf; }

  .button-danger {
    background: #ef4444;
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(239, 68, 68, 0.18);
  }
  .button-danger:hover { background: #dc2626; }

  .button-secondary {
    background: rgba(255, 255, 255, 0.22);
    border-color: #d7e0eb;
    color: #697a92;
  }
  .button-secondary:hover { background: rgba(255, 255, 255, 0.5); }

  .action-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5px 20px 8px;
    background: #e8edf4;
  }

  .version {
    margin: 0;
    font-size: 12px;
    font-weight: 800;
    letter-spacing: 0.12em;
    color: #9aa8bc;
    text-transform: uppercase;
    text-align: center;
  }

  @media (min-width: 640px) {
    .action-content { padding: 36px 28px 0; }
  }
</style>
