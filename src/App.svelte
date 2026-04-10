<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { save } from "@tauri-apps/plugin-dialog";
  import { onDestroy, onMount } from "svelte";
  import AppHeader from "./components/layout/AppHeader.svelte";
  import StatusBar from "./components/layout/StatusBar.svelte";
  import ConfirmDialog from "./components/common/ConfirmDialog.svelte";
  import InspectionSummaryModal from "./components/common/InspectionSummaryModal.svelte";
  import MenuPanel from "./components/menu/MenuPanel.svelte";
  import SystemInfoPanel from "./components/system/SystemInfoPanel.svelte";
  import VirtualizationPanel from "./components/virtualization/VirtualizationPanel.svelte";
  import DisablePanel from "./components/disable/DisablePanel.svelte";
  import type {
    DisableGroup,
    DisableOptions,
    DisableOutput,
    Panel,
    ProgressEvent,
    SystemInfoItem,
    VirtItem,
  } from "./lib/app-types";

  type InspectionExportOutput = {
    system_csv_path: string;
    virtualization_csv_path: string;
  };

  let currentPanel = $state<Panel>("menu");
  let status = $state("준비됨");
  let version = $state("dev");

  let systemItems = $state<SystemInfoItem[]>([]);
  let systemLoading = $state(false);
  let systemLoaded = $state(false);

  let virtItems = $state<VirtItem[]>([]);
  let virtLoading = $state(false);
  let virtChecked = $state(false);
  let inspectionModalOpen = $state(true);
  let inspectionSystemResultPath = $state<string | null>(null);
  let inspectionVirtResultPath = $state<string | null>(null);
  let inspectionResultSaveError = $state<string | null>(null);
  let inspectionProgress = $state(0);
  let inspectionStage = $state<"idle" | "collecting" | "saving" | "complete">("idle");
  let systemDone = $state(false);
  let virtDone = $state(false);
  let inspectionCurrentAction = $state("점검 준비 중...");

  let disableLog = $state<string[]>([]);
  let disableRunning = $state(false);
  let disableComplete = $state(false);
  let rebootConfirmOpen = $state(false);
  let progressValue = $state<number | null>(null);
  let inspectionProgressTimer: ReturnType<typeof setInterval> | null = null;

  onDestroy(() => {
    if (inspectionProgressTimer) {
      clearInterval(inspectionProgressTimer);
    }
  });

  onMount(async () => {
    try {
      version = await invoke<string>("get_app_version");
    } catch {
      version = "dev";
    }
    // 앱 시작 시 자동 점검 — 패널 전환 없이 백그라운드 실행
    status = "자동 점검 시작 중...";
    inspectionStage = "collecting";
    startInspectionProgressTimer();
    await Promise.all([fetchSystemInfo(), fetchVirtStatus()]);

    inspectionStage = "saving";
    inspectionCurrentAction = "점검 결과 CSV 저장 중...";
    try {
      const exportResult = await invoke<InspectionExportOutput>("export_inspection_csvs_auto", {
        systemItems,
        virtItems,
      });
      inspectionSystemResultPath = exportResult.system_csv_path;
      inspectionVirtResultPath = exportResult.virtualization_csv_path;
    } catch (e) {
      inspectionResultSaveError = `${e}`;
    }

    inspectionStage = "complete";
    inspectionProgress = 100;
    inspectionCurrentAction = "점검 완료";
    stopInspectionProgressTimer();

    status = inspectionSystemResultPath && inspectionVirtResultPath
      ? `점검 완료 — ${basename(inspectionSystemResultPath)}, ${basename(inspectionVirtResultPath)} 저장됨`
      : "점검 완료 — 비활성화 준비됨";
  });

  function showPanel(panel: Panel) {
    currentPanel = panel;
  }

  async function fetchSystemInfo() {
    if (systemLoaded) return;
    systemLoading = true;
    progressValue = null;
    systemDone = false;
    try {
      systemItems = await invoke<SystemInfoItem[]>("get_system_info");
      systemLoaded = true;
      systemDone = true;
    } catch (e) {
      status = `시스템 정보 오류: ${e}`;
    } finally {
      systemLoading = false;
      progressValue = null;
    }
  }

  async function fetchVirtStatus() {
    virtLoading = true;
    progressValue = null;
    virtDone = false;
    try {
      virtItems = await invoke<VirtItem[]>("get_virtualization_status");
      virtChecked = true;
      virtDone = true;
    } catch (e) {
      status = `가상화 점검 오류: ${e}`;
    } finally {
      virtLoading = false;
      progressValue = null;
    }
  }

  async function loadSystemInfo() {
    showPanel("systemInfo");
    if (systemLoaded) return;
    status = "시스템 정보 수집 중...";
    await fetchSystemInfo();
    status = "시스템 정보 수집 완료";
  }

  async function refreshSystemInfo() {
    systemLoaded = false;
    showPanel("systemInfo");
    status = "시스템 정보 수집 중...";
    await fetchSystemInfo();
    status = "시스템 정보 수집 완료";
  }

  async function loadVirtStatus() {
    showPanel("virtualization");
    status = "가상화 설정 점검 중...";
    await fetchVirtStatus();
    status = "가상화 설정 점검 완료";
  }

  async function exportSystemCsv() {
    const path = await save({
      filters: [{ name: "CSV 파일", extensions: ["csv"] }],
      defaultPath: `SystemInfo-${new Date().toISOString().slice(0, 10)}.csv`,
    });
    if (!path) return;
    try {
      await invoke("export_csv", {
        filePath: path,
        dataType: "system",
        systemItems,
        virtItems: null,
      });
      status = "CSV 내보내기 완료";
    } catch (e) {
      status = `CSV 오류: ${e}`;
    }
  }

  async function exportVirtCsv() {
    const path = await save({
      filters: [{ name: "CSV 파일", extensions: ["csv"] }],
      defaultPath: `VirtCheck-${new Date().toISOString().slice(0, 10)}.csv`,
    });
    if (!path) return;
    try {
      await invoke("export_csv", {
        filePath: path,
        dataType: "virtualization",
        systemItems: null,
        virtItems,
      });
      status = "CSV 내보내기 완료";
    } catch (e) {
      status = `CSV 오류: ${e}`;
    }
  }

  async function runDisable() {
    disableRunning = true;
    disableComplete = false;
    disableLog = ["▶ 비활성화 작업을 시작합니다..."];
    status = "비활성화 실행 중...";
    progressValue = 0;

    const unlisten = await listen<ProgressEvent>("disable-progress", (e) => {
      const { step, total, message, success } = e.payload;
      const icon = success ? "⏳" : "⚠️";
      disableLog = [...disableLog, `  [${step}/${total}] ${icon} ${message}`];
      progressValue = total > 0 ? step / total : null;
    });

    try {
      const options: DisableOptions | null = virtChecked ? computeDisableOptions(virtItems) : null;
      const output = await invoke<DisableOutput>("execute_disable", { options });
      for (const result of output.results) {
        disableLog = [
          ...disableLog,
          "",
          `${result.success ? "✅" : "⚠️"} ${result.task}`,
          result.message,
        ];
      }
      disableLog = [
        ...disableLog,
        "",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        "✅ 모든 작업이 완료되었습니다.",
        "변경 사항을 적용하려면 재부팅이 필요합니다.",
      ];
      if (output.log_path) {
        disableLog = [...disableLog, `📄 로그: ${output.log_path}`];
      }
      if (output.backup_path) {
        disableLog = [...disableLog, `💾 레지스트리 백업: ${output.backup_path}`];
      }
      status = output.log_path
        ? `비활성화 완료 — 로그 저장됨`
        : "비활성화 완료 — 재부팅 필요";
      disableComplete = true;
    } catch (e) {
      disableLog = [...disableLog, `❌ 오류: ${e}`];
      status = "오류 발생";
    } finally {
      disableRunning = false;
      progressValue = null;
      unlisten();
    }
  }

  function requestReboot() {
    rebootConfirmOpen = true;
  }

  async function closeInspectionModal() {
    await invoke("exit_app");
  }

  function startInspectionActions() {
    inspectionModalOpen = false;
    showPanel("disable");
  }

  async function confirmReboot() {
    try {
      await invoke("request_reboot");
      rebootConfirmOpen = false;
      status = "재부팅 예약 완료";
    } catch (e) {
      status = `재부팅 오류: ${e}`;
    }
  }

  function computeDisableOptions(items: VirtItem[]): DisableOptions {
    return {
      hyperv: hasActionForGroup(items, "hyperv"),
      wsl: hasActionForGroup(items, "wsl"),
      vbs: hasActionForGroup(items, "vbs"),
      core_isolation: hasActionForGroup(items, "core_isolation"),
    };
  }

  function hasActionForGroup(items: VirtItem[], group: DisableGroup): boolean {
    return items.some((item) => item.disable_group === group && item.action_required);
  }

  function actionCount(items: VirtItem[]): number {
    return new Set(
      items
        .filter((item) => item.disable_group !== null && item.action_required)
        .map((item) => item.disable_group)
    ).size;
  }

  function actionItemCount(items: VirtItem[]): number {
    return items.filter((item) => item.action_required && item.manifest_id !== "whfb_check").length;
  }

  function unknownItemCount(items: VirtItem[]): number {
    return items.filter((item) => item.status.includes("확인 불가") && item.manifest_id !== "whfb_check").length;
  }

  function healthyItemCount(items: VirtItem[]): number {
    return items.filter(
      (item) => !item.action_required && !item.status.includes("확인 불가") && item.manifest_id !== "whfb_check"
    ).length;
  }

  function hasWhfbWarning(items: VirtItem[]): boolean {
    return items.some((item) => item.manifest_id === "whfb_check");
  }

  function selectedTaskCount(opts: DisableOptions): number {
    return [opts.hyperv, opts.wsl, opts.vbs, opts.core_isolation].filter(Boolean).length;
  }

  function logLineClass(line: string): string {
    if (line.includes("✅") || line.startsWith("✓")) return "text-green-400";
    if (line.includes("❌") || line.includes("⚠️") || line.startsWith("✗")) {
      return "text-red-400";
    }
    if (line.startsWith("  [")) return "text-yellow-300";
    if (line.startsWith("━")) return "text-gray-500";
    if (line.startsWith("- ") || line.startsWith("  -")) return "text-gray-400";
    if (line.startsWith("▶")) return "text-blue-400";
    return "text-green-300";
  }

  const rebootBullets = [
    "실행 중인 작업은 모두 저장하세요.",
    "변경 사항 적용을 위해 시스템 재시작이 필요합니다.",
  ];

  function basename(path: string | null): string | null {
    return path?.split(/[\\/]/).pop() ?? null;
  }

  function inspectionProgressPercent(): number {
    return inspectionProgress;
  }

  function inspectionActionSummaries(items: VirtItem[]): string[] {
    return items
      .filter((item) => item.action_required && item.manifest_id !== "whfb_check")
      .slice(0, 4)
      .map((item) => {
        const detail = item.recommendation?.trim() || item.status;
        return `${item.category}: ${detail}`;
      });
  }

  function startInspectionProgressTimer() {
    stopInspectionProgressTimer();
    inspectionProgress = 3;
    inspectionCurrentAction = "시스템 점검 준비 중...";
    inspectionProgressTimer = setInterval(() => {
      if (inspectionStage === "complete") {
        inspectionProgress = 100;
        return;
      }

      if (inspectionStage === "saving") {
        const target = 97;
        if (inspectionProgress < target) {
          inspectionProgress = Math.min(
            target,
            inspectionProgress + Math.max(1, Math.ceil((target - inspectionProgress) / 8))
          );
        }
      } else if (systemDone || virtDone) {
        const target = 95;
        if (inspectionProgress < target) {
          inspectionProgress = Math.min(
            target,
            inspectionProgress + Math.max(1, Math.ceil((target - inspectionProgress) / 12))
          );
        }
      } else {
        const target = 78;
        if (inspectionProgress < target) {
          inspectionProgress = Math.min(
            target,
            inspectionProgress + Math.max(1, Math.ceil((target - inspectionProgress) / 16))
          );
        }
      }

      inspectionCurrentAction = nextInspectionAction();
    }, 180);
  }

  function stopInspectionProgressTimer() {
    if (inspectionProgressTimer) {
      clearInterval(inspectionProgressTimer);
      inspectionProgressTimer = null;
    }
  }
  function nextInspectionAction(): string {
    if (inspectionStage === "saving") return "점검 결과 CSV 저장 중...";
    if (inspectionStage === "complete") return "점검 완료";

    if (systemDone && !virtDone) {
      return pickAction([
        "Hyper-V 기능 상태 수집 중...",
        "WSL / VirtualMachinePlatform 상태 수집 중...",
        "VBS 레지스트리 수집 중...",
        "코어 격리 레지스트리 수집 중...",
      ]);
    }

    if (!systemDone && virtDone) {
      return pickAction([
        "운영체제 정보 수집 중...",
        "CPU / 메모리 정보 수집 중...",
        "디스크 / 부팅 정보 수집 중...",
        "메인보드 / GPU 정보 수집 중...",
        "이벤트 로그 수집 중...",
      ]);
    }

    return pickAction([
      "운영체제 정보 수집 중...",
      "CPU / 메모리 정보 수집 중...",
      "디스크 / 부팅 정보 수집 중...",
      "Hyper-V 기능 상태 수집 중...",
      "WSL / VirtualMachinePlatform 상태 수집 중...",
      "VBS 레지스트리 수집 중...",
      "코어 격리 레지스트리 수집 중...",
      "이벤트 로그 수집 중...",
    ]);
  }

  function pickAction(actions: string[]): string {
    return actions[Math.floor((inspectionProgress / 7) % actions.length)] ?? "점검 진행 중...";
  }
</script>

{#if inspectionModalOpen}
  <InspectionSummaryModal
    open={inspectionModalOpen}
    complete={systemLoaded && virtChecked}
    progressPercent={inspectionProgressPercent()}
    currentAction={inspectionCurrentAction}
    actionSummaries={inspectionActionSummaries(virtItems)}
    savedFilenames={[
      basename(inspectionSystemResultPath),
      basename(inspectionVirtResultPath),
    ].filter((value): value is string => Boolean(value))}
    saveError={inspectionResultSaveError}
    {version}
    onStartAction={startInspectionActions}
    onClose={closeInspectionModal}
  />
{:else}
  <div class="flex flex-col h-screen bg-gray-50 select-none">
    <AppHeader currentPanel={currentPanel} onBack={() => showPanel("menu")} />

    <main class="flex-1 overflow-hidden">
      {#if currentPanel === "menu"}
        <MenuPanel
          {systemLoading}
          {systemLoaded}
          {virtLoading}
          {virtChecked}
          actionGroupCount={actionCount(virtItems)}
          onLoadSystemInfo={loadSystemInfo}
          onLoadVirtStatus={loadVirtStatus}
          onShowDisable={() => showPanel("disable")}
        />
      {:else if currentPanel === "systemInfo"}
        <SystemInfoPanel
          {systemLoading}
          {systemItems}
          onRefresh={refreshSystemInfo}
          onExport={exportSystemCsv}
        />
      {:else if currentPanel === "virtualization"}
        <VirtualizationPanel
          {virtLoading}
          {virtChecked}
          {virtItems}
          actionGroupCount={actionCount(virtItems)}
          actionItemTotal={actionItemCount(virtItems)}
          healthyItemTotal={healthyItemCount(virtItems)}
          unknownItemTotal={unknownItemCount(virtItems)}
          onReload={loadVirtStatus}
          onExport={exportVirtCsv}
          onShowDisable={() => showPanel("disable")}
        />
      {:else if currentPanel === "disable"}
        <DisablePanel
          {virtChecked}
          {disableRunning}
          {disableComplete}
          {disableLog}
          disableOptions={computeDisableOptions(virtItems)}
          selectedTaskCount={selectedTaskCount(computeDisableOptions(virtItems))}
          whfbDetected={hasWhfbWarning(virtItems)}
          onRunDisable={runDisable}
          onRequestReboot={requestReboot}
          onLoadVirtStatus={loadVirtStatus}
          {logLineClass}
        />
      {/if}
    </main>

    <StatusBar
      {status}
      {version}
      isBusy={systemLoading || virtLoading || disableRunning}
      {progressValue}
    />

    <ConfirmDialog
      open={rebootConfirmOpen}
      title="지금 재부팅할까요?"
      message="확인을 누르면 5초 후 자동으로 재부팅됩니다."
      bullets={rebootBullets}
      confirmLabel="재부팅 예약"
      onConfirm={confirmReboot}
      onCancel={() => (rebootConfirmOpen = false)}
    />
  </div>
{/if}
