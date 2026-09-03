<script lang="ts">
  type Props = {
    version: string;
    onStartInspection: () => void;
    onStartActionDirectly: () => void;
  };

  let { version, onStartInspection, onStartActionDirectly }: Props = $props();
</script>

<div class="launch-screen">
  <div class="launch-panel" role="dialog" aria-modal="true" aria-labelledby="launch-dialog-title">
    <div class="launch-content">
      <div class="status-icon-wrap" aria-hidden="true">
        <svg viewBox="0 0 48 48" class="status-icon">
          <circle cx="21" cy="21" r="15" fill="#2f67ea" opacity="0.12"></circle>
          <circle cx="21" cy="21" r="10.5" fill="#2f67ea"></circle>
          <path d="M28.5 28.5L38 38" stroke="#2f67ea" stroke-width="4.2" stroke-linecap="round"></path>
          <path d="M17 16.5V25.5M21 14V28M25 18V25" stroke="white" stroke-width="2.4" stroke-linecap="round"></path>
        </svg>
      </div>

      <h2 id="launch-dialog-title" class="title">VM 호환성 도구</h2>

      <p class="description">
        시스템 점검을 먼저 실행하거나, 점검을 건너뛰고 바로 조치를 진행할 수 있습니다.
      </p>

      <section class="choice-card">
        <p class="choice-title">검사 시작</p>
        <p class="choice-body">
          하드웨어·가상화·설치 프로그램을 점검하고 결과를 CSV로 저장합니다.
          점검 후 조치가 필요하면 조치 시작 버튼이 제공됩니다.
        </p>
      </section>

      <section class="choice-card">
        <p class="choice-title">조치 시작 (점검 건너뛰기)</p>
        <p class="choice-body">
          점검 없이 비활성화 조치 화면으로 바로 이동합니다. 일부 환경에서
          점검이 진행되지 않을 때 사용하세요. 조치 항목은 다음 화면에서 선택합니다.
        </p>
      </section>

      <div class="actions">
        <button class="button button-primary" onclick={onStartInspection}>검사 시작</button>
        <button class="button button-secondary" onclick={onStartActionDirectly}>조치 시작</button>
      </div>
    </div>

    <div class="launch-footer">
      <p class="version">{version}</p>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
  }

  .launch-screen {
    min-height: 100vh;
    display: flex;
    align-items: stretch;
    justify-content: center;
    background: linear-gradient(180deg, #eef3fa 0%, #f5f7fb 100%);
    box-sizing: border-box;
  }

  .launch-panel {
    width: 100%;
    min-height: 100vh;
    background: #ffffff;
    display: flex;
    flex-direction: column;
  }

  .launch-content {
    flex: 1;
    width: 100%;
    padding: 24px 12px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-sizing: border-box;
  }

  .status-icon-wrap {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: #eef2f7;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
  }

  .status-icon {
    width: 38px;
    height: 38px;
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
    margin: 10px 0 0;
    width: 100%;
    font-size: 14px;
    line-height: 1.6;
    letter-spacing: -0.02em;
    color: #7f8ea3;
    word-break: keep-all;
  }

  .choice-card {
    width: 100%;
    margin-top: 16px;
    border: 1px solid #dce5f0;
    border-radius: 18px;
    background: rgba(245, 247, 251, 0.82);
    padding: 14px 16px;
    box-sizing: border-box;
    text-align: left;
  }

  .choice-title {
    margin: 0 0 6px;
    font-size: 14px;
    font-weight: 700;
    color: #1d2740;
    letter-spacing: -0.02em;
  }

  .choice-body {
    margin: 0;
    font-size: 13px;
    line-height: 1.7;
    letter-spacing: -0.02em;
    color: #4a5970;
    word-break: keep-all;
  }

  .actions {
    width: 100%;
    margin-top: 20px;
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .button {
    width: 100%;
    min-height: 48px;
    border-radius: 14px;
    border: 1px solid transparent;
    font-size: 16px;
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

  .button-secondary {
    background: rgba(255, 255, 255, 0.22);
    border-color: #d7e0eb;
    color: #697a92;
  }

  .button-secondary:hover {
    background: rgba(255, 255, 255, 0.5);
  }

  .launch-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px 20px 6px;
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
    .launch-content {
      padding: 32px 20px 0;
    }
  }
</style>
