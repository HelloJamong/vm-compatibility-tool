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
  import DisableActionModal from "./components/disable/DisableActionModal.svelte";
  import type {
    DisableGroup,
    DisableOptions,
    DisableOutput,
    InstalledProgramItem,
    Panel,
    ProgressEvent,
    SystemInfoItem,
    VirtItem,
  } from "./lib/app-types";

  type InspectionExportOutput = {
    system_csv_path: string;
    virtualization_csv_path: string;
    installed_programs_csv_path: string;
  };
  type InspectionStage = "idle" | "collecting" | "saving" | "complete";
  type InspectionTask = "preparing" | "system" | "virtualization" | "installedPrograms" | "saving" | "complete";

  let currentPanel = $state<Panel>("menu");
  let status = $state("준비됨");
  let version = $state("dev");

  let systemItems = $state<SystemInfoItem[]>([]);
  let systemLoading = $state(false);
  let systemLoaded = $state(false);

  let virtItems = $state<VirtItem[]>([]);
  let virtLoading = $state(false);
  let virtChecked = $state(false);
  let installedProgramItems = $state<InstalledProgramItem[]>([]);
  let installedProgramsLoading = $state(false);
  let installedProgramsLoaded = $state(false);
  let inspectionModalOpen = $state(true);
  let inspectionSystemResultPath = $state<string | null>(null);
  let inspectionVirtResultPath = $state<string | null>(null);
  let inspectionInstalledProgramsResultPath = $state<string | null>(null);
  let inspectionResultSaveError = $state<string | null>(null);
  let inspectionProgress = $state(0);
  let inspectionStage = $state<InspectionStage>("idle");
  let inspectionActiveTask = $state<InspectionTask>("preparing");
  let inspectionSkippedItems = $state<string[]>([]);
  let systemDone = $state(false);
  let virtDone = $state(false);
  let installedProgramsDone = $state(false);
  let inspectionCurrentAction = $state("점검 준비 중...");

  let disableLog = $state<string[]>([]);
  let disableRunning = $state(false);
  let disableComplete = $state(false);
  let rebootConfirmOpen = $state(false);

  let disableActionModalOpen = $state(false);
  let disableActionStage = $state<"warning" | "running" | "complete">("warning");
  let disableActionProgress = $state(0);
  let disableActionCurrentAction = $state("");
  let disableActionHasErrors = $state(false);
  let disableActionLogPath = $state<string | null>(null);
  let disableActionBackupPath = $state<string | null>(null);
  let disableActionChangeCsvPath = $state<string | null>(null);
  let disableActionOptions = $state<DisableOptions>({
    hyperv: false,
    wsl: false,
    vbs: false,
    core_isolation: false,
    optional_registry_ids: [],
  });
  let progressValue = $state<number | null>(null);
  let inspectionProgressTimer: ReturnType<typeof setInterval> | null = null;
  const inspectionTaskProgress: Record<InspectionTask, { activeCap: number; complete: number; label: string }> = {
    preparing: { activeCap: 4, complete: 5, label: "점검 준비 중..." },
    system: { activeCap: 32, complete: 35, label: "시스템 정보 수집 중..." },
    virtualization: { activeCap: 62, complete: 65, label: "가상화 설정 점검 중..." },
    installedPrograms: { activeCap: 87, complete: 90, label: "설치된 프로그램 목록 수집 중..." },
    saving: { activeCap: 98, complete: 99, label: "점검 결과 CSV 저장 중..." },
    complete: { activeCap: 100, complete: 100, label: "점검 완료" },
  };

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
    inspectionActiveTask = "preparing";
    startInspectionProgressTimer();

    await runInspectionStep("system", fetchSystemInfo);
    await runInspectionStep("virtualization", fetchVirtStatus);
    await runInspectionStep("installedPrograms", fetchInstalledPrograms);

    inspectionStage = "saving";
    inspectionActiveTask = "saving";
    inspectionCurrentAction = inspectionTaskProgress.saving.label;
    try {
      const exportResult = await invoke<InspectionExportOutput>("export_inspection_csvs_auto", {
        systemItems,
        virtItems,
        installedProgramItems,
      });
      inspectionSystemResultPath = exportResult.system_csv_path;
      inspectionVirtResultPath = exportResult.virtualization_csv_path;
      inspectionInstalledProgramsResultPath = exportResult.installed_programs_csv_path;
    } catch (e) {
      inspectionResultSaveError = `${e}`;
    }
    inspectionProgress = Math.max(inspectionProgress, inspectionTaskProgress.saving.complete);

    inspectionStage = "complete";
    inspectionActiveTask = "complete";
    inspectionProgress = 100;
    inspectionCurrentAction = inspectionTaskProgress.complete.label;
    stopInspectionProgressTimer();

    status = inspectionSystemResultPath && inspectionVirtResultPath && inspectionInstalledProgramsResultPath
      ? `점검 완료 — ${basename(inspectionSystemResultPath)}, ${basename(inspectionVirtResultPath)}, ${basename(inspectionInstalledProgramsResultPath)} 저장됨`
      : "점검 완료 — 비활성화 준비됨";
  });

  function showPanel(panel: Panel) {
    currentPanel = panel;
  }

  async function runInspectionStep(task: InspectionTask, action: () => Promise<void>) {
    inspectionActiveTask = task;
    inspectionCurrentAction = inspectionTaskProgress[task].label;
    try {
      await action();
    } catch (e) {
      markInspectionTaskSkipped(task, `${e}`);
    }
    inspectionProgress = Math.max(inspectionProgress, inspectionTaskProgress[task].complete);
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
      const message = `${e}`;
      systemItems = [
        {
          category: "점검 실패",
          item: "시스템 정보",
          value: message,
        },
      ];
      systemLoaded = true;
      systemDone = false;
      recordInspectionSkip("시스템 정보", message);
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
      const message = `${e}`;
      virtItems = [
        {
          category: "가상화 점검",
          status: "확인 불가",
          details: message,
          recommendation: "해당 항목은 건너뛰고 나머지 점검을 계속했습니다.",
          disable_group: null,
          source_type: "unknown",
          action_required: false,
          optional_action_available: false,
          manifest_id: "virtualization_check_failed",
        },
      ];
      virtChecked = true;
      virtDone = false;
      recordInspectionSkip("가상화 점검", message);
      status = `가상화 점검 오류: ${e}`;
    } finally {
      virtLoading = false;
      progressValue = null;
    }
  }

  async function fetchInstalledPrograms() {
    if (installedProgramsLoaded) return;
    installedProgramsLoading = true;
    installedProgramsDone = false;
    try {
      installedProgramItems = await invoke<InstalledProgramItem[]>("get_installed_programs");
      installedProgramsLoaded = true;
      installedProgramsDone = true;
    } catch (e) {
      const message = `${e}`;
      installedProgramItems = [
        {
          name: "설치 프로그램 목록 수집 실패",
          publisher: message,
          install_date: "",
        },
      ];
      installedProgramsLoaded = true;
      installedProgramsDone = false;
      recordInspectionSkip("설치 프로그램 목록", message);
      status = `설치 프로그램 목록 오류: ${e}`;
    } finally {
      installedProgramsLoading = false;
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
      if (output.change_csv_path) {
        disableLog = [...disableLog, `📊 조치 전후 비교 CSV: ${output.change_csv_path}`];
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
    openDisableActionModal();
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

  function openDisableActionModal() {
    disableActionOptions = virtChecked
      ? computeDisableOptions(virtItems)
      : { hyperv: true, wsl: true, vbs: true, core_isolation: true, optional_registry_ids: [] };
    disableActionStage = "warning";
    disableActionProgress = 0;
    disableActionCurrentAction = "";
    disableActionHasErrors = false;
    disableActionLogPath = null;
    disableActionBackupPath = null;
    disableActionChangeCsvPath = null;
    disableActionModalOpen = true;
  }

  async function startDisableAction() {
    disableActionStage = "running";
    disableActionProgress = 0;
    disableActionCurrentAction = "조치 시작 중...";
    disableRunning = true;
    disableComplete = false;
    disableLog = ["▶ 비활성화 작업을 시작합니다..."];
    status = "비활성화 실행 중...";

    const unlisten = await listen<ProgressEvent>("disable-progress", (e) => {
      const { step, total, message, success } = e.payload;
      disableActionCurrentAction = message;
      disableActionProgress = total > 0 ? (step / total) * 100 : 0;
      const icon = success ? "⏳" : "⚠️";
      disableLog = [...disableLog, `  [${step}/${total}] ${icon} ${message}`];
    });

    try {
      const options: DisableOptions | null = virtChecked ? disableActionOptions : null;
      const output = await invoke<DisableOutput>("execute_disable", { options });
      disableActionHasErrors = output.results.some((r) => !r.success);
      disableActionLogPath = output.log_path;
      disableActionBackupPath = output.backup_path;
      disableActionChangeCsvPath = output.change_csv_path;
      for (const result of output.results) {
        disableLog = [...disableLog, "", `${result.success ? "✅" : "⚠️"} ${result.task}`, result.message];
      }
      disableComplete = true;
      disableActionStage = "complete";
      status = output.log_path ? "비활성화 완료 — 로그 저장됨" : "비활성화 완료 — 재부팅 필요";
    } catch (e) {
      disableActionHasErrors = true;
      disableActionStage = "complete";
      disableLog = [...disableLog, `❌ 오류: ${e}`];
      status = "오류 발생";
    } finally {
      disableRunning = false;
      unlisten();
    }
  }

  function closeDisableActionModal() {
    disableActionModalOpen = false;
    if (disableComplete) showPanel("menu");
  }

  async function rebootNow() {
    try {
      await invoke("request_reboot");
      status = "재부팅 예약 완료 (5초 후)";
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
      optional_registry_ids: [],
    };
  }

  function toggleOptionalRegistrySelection(manifestId: string) {
    const isSelected = disableActionOptions.optional_registry_ids.includes(manifestId);
    disableActionOptions = {
      ...disableActionOptions,
      optional_registry_ids: isSelected
        ? disableActionOptions.optional_registry_ids.filter((id) => id !== manifestId)
        : [...disableActionOptions.optional_registry_ids, manifestId],
    };
  }

  function toggleDisableOption(group: DisableGroup) {
    disableActionOptions = {
      ...disableActionOptions,
      [group]: !disableActionOptions[group],
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
    return items.filter((item) => item.status.includes("확인 불가") && item.manifest_id !== "whfb_check" && item.manifest_id !== "org_control_check").length;
  }

  function healthyItemCount(items: VirtItem[]): number {
    return items.filter(
      (item) => !item.action_required && !item.status.includes("확인 불가") && item.manifest_id !== "whfb_check" && item.manifest_id !== "org_control_check"
    ).length;
  }

  function hasWhfbWarning(items: VirtItem[]): boolean {
    return items.some((item) => item.manifest_id === "whfb_check");
  }

  function hasOrgWarning(items: VirtItem[]): boolean {
    return items.some((item) => item.manifest_id === "org_control_check");
  }

  function selectedTaskCount(opts: DisableOptions): number {
    return [opts.hyperv, opts.wsl, opts.vbs, opts.core_isolation].filter(Boolean).length + opts.optional_registry_ids.length;
  }

  function optionalRegistryCandidates(items: VirtItem[]): VirtItem[] {
    return items.filter((item) => item.optional_action_available && item.manifest_id !== null);
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
    const groupLabels: Record<DisableGroup, string> = {
      hyperv: "Hyper-V 관련",
      wsl: "WSL 관련",
      vbs: "VBS 관련",
      core_isolation: "코어 격리 관련",
    };
    const counts = new Map<DisableGroup, number>();

    for (const item of items) {
      if (!item.action_required || item.disable_group === null || item.manifest_id === "whfb_check") {
        continue;
      }
      counts.set(item.disable_group, (counts.get(item.disable_group) ?? 0) + 1);
    }

    return Array.from(counts.entries())
      .slice(0, 4)
      .map(([group, count]) => `${groupLabels[group]} 조치 필요 항목 ${count}건`);
  }

  function startInspectionProgressTimer() {
    stopInspectionProgressTimer();
    inspectionProgress = 1;
    inspectionCurrentAction = inspectionTaskProgress.preparing.label;
    inspectionProgressTimer = setInterval(() => {
      if (inspectionStage === "complete") {
        inspectionProgress = 100;
        inspectionCurrentAction = inspectionTaskProgress.complete.label;
        return;
      }

      const activeTask = inspectionStage === "saving" ? "saving" : inspectionActiveTask;
      const { activeCap, label } = inspectionTaskProgress[activeTask];
      inspectionCurrentAction = label;

      if (inspectionProgress < activeCap) {
        inspectionProgress = Math.min(
          activeCap,
          inspectionProgress + Math.max(1, Math.ceil((activeCap - inspectionProgress) / 12))
        );
      }
    }, 180);
  }

  function stopInspectionProgressTimer() {
    if (inspectionProgressTimer) {
      clearInterval(inspectionProgressTimer);
      inspectionProgressTimer = null;
    }
  }

  function recordInspectionSkip(label: string, message: string) {
    const summary = `${label}: ${message || "알 수 없는 오류"}`;
    if (!inspectionSkippedItems.includes(summary)) {
      inspectionSkippedItems = [...inspectionSkippedItems, summary];
    }
  }

  function markInspectionTaskSkipped(task: InspectionTask, message: string) {
    if (task === "system") {
      systemLoaded = true;
      systemDone = false;
      if (systemItems.length === 0) {
        systemItems = [{ category: "점검 실패", item: "시스템 정보", value: message }];
      }
      recordInspectionSkip("시스템 정보", message);
    } else if (task === "virtualization") {
      virtChecked = true;
      virtDone = false;
      if (virtItems.length === 0) {
        virtItems = [
          {
            category: "가상화 점검",
            status: "확인 불가",
            details: message,
            recommendation: "해당 항목은 건너뛰고 나머지 점검을 계속했습니다.",
            disable_group: null,
            source_type: "unknown",
            action_required: false,
            optional_action_available: false,
            manifest_id: "virtualization_check_failed",
          },
        ];
      }
      recordInspectionSkip("가상화 점검", message);
    } else if (task === "installedPrograms") {
      installedProgramsLoaded = true;
      installedProgramsDone = false;
      if (installedProgramItems.length === 0) {
        installedProgramItems = [
          {
            name: "설치 프로그램 목록 수집 실패",
            publisher: message,
            install_date: "",
          },
        ];
      }
      recordInspectionSkip("설치 프로그램 목록", message);
    }
  }
</script>

{#if disableActionModalOpen}
  <DisableActionModal
    open={disableActionModalOpen}
    stage={disableActionStage}
    progressPercent={disableActionProgress}
    currentAction={disableActionCurrentAction}
    disableOptions={disableActionOptions}
    optionalRegistryCandidates={optionalRegistryCandidates(virtItems)}
    hasErrors={disableActionHasErrors}
    logPath={disableActionLogPath}
    backupPath={disableActionBackupPath}
    changeCsvPath={disableActionChangeCsvPath}
    {version}
    onStart={startDisableAction}
    onToggleDisableOption={toggleDisableOption}
    onToggleOptionalRegistry={toggleOptionalRegistrySelection}
    onCancel={closeDisableActionModal}
    onRebootNow={rebootNow}
    onDismiss={closeDisableActionModal}
  />
{:else if inspectionModalOpen}
  <InspectionSummaryModal
    open={inspectionModalOpen}
    complete={systemLoaded && virtChecked && installedProgramsLoaded}
    progressPercent={inspectionProgressPercent()}
    currentAction={inspectionCurrentAction}
    actionSummaries={inspectionActionSummaries(virtItems)}
    actionItemTotal={actionItemCount(virtItems)}
    skippedItems={inspectionSkippedItems}
    savedFilenames={[
      basename(inspectionSystemResultPath),
      basename(inspectionVirtResultPath),
      basename(inspectionInstalledProgramsResultPath),
    ].filter((value): value is string => Boolean(value))}
    saveError={inspectionResultSaveError}
    {version}
    whfbDetected={hasWhfbWarning(virtItems)}
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
          optionalCandidateCount={optionalRegistryCandidates(virtItems).length}
          whfbDetected={hasWhfbWarning(virtItems)}
          orgWarningDetected={hasOrgWarning(virtItems)}
          onRunDisable={openDisableActionModal}
          onRequestReboot={requestReboot}
          onLoadVirtStatus={loadVirtStatus}
          {logLineClass}
        />
      {/if}
    </main>

    <StatusBar
      {status}
      {version}
      isBusy={systemLoading || virtLoading || installedProgramsLoading || disableRunning}
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
