<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  type Panel = "menu" | "systemInfo" | "virtualization" | "disable";
  type SystemInfoItem = { category: string; item: string; value: string };
  type VirtItem = {
    category: string;
    status: string;
    details: string;
    recommendation: string;
  };
  type DisableResult = { task: string; success: boolean; message: string };
  type ProgressEvent = {
    step: number;
    total: number;
    message: string;
    success: boolean;
  };
  type DisableOptions = {
    hyperv: boolean;
    wsl: boolean;
    vbs: boolean;
    core_isolation: boolean;
  };

  let currentPanel = $state<Panel>("menu");
  let status = $state("준비됨");
  let version = $state("dev");

  // 시스템 정보
  let systemItems = $state<SystemInfoItem[]>([]);
  let systemLoading = $state(false);
  let systemLoaded = $state(false);

  // 가상화 점검
  let virtItems = $state<VirtItem[]>([]);
  let virtLoading = $state(false);
  let virtChecked = $state(false);

  // 비활성화
  let disableLog = $state<string[]>([]);
  let disableRunning = $state(false);
  let disableComplete = $state(false);

  onMount(async () => {
    try {
      version = await invoke<string>("get_app_version");
    } catch {
      version = "dev";
    }
  });

  function showPanel(panel: Panel) {
    currentPanel = panel;
  }

  async function loadSystemInfo() {
    showPanel("systemInfo");
    if (systemLoaded) return;
    systemLoading = true;
    status = "시스템 정보 수집 중...";
    try {
      systemItems = await invoke<SystemInfoItem[]>("get_system_info");
      systemLoaded = true;
      status = "시스템 정보 수집 완료";
    } catch (e) {
      status = `오류: ${e}`;
    } finally {
      systemLoading = false;
    }
  }

  async function refreshSystemInfo() {
    systemLoaded = false;
    await loadSystemInfo();
  }

  async function loadVirtStatus() {
    showPanel("virtualization");
    virtLoading = true;
    status = "가상화 설정 점검 중...";
    try {
      virtItems = await invoke<VirtItem[]>("get_virtualization_status");
      virtChecked = true;
      status = "가상화 설정 점검 완료";
    } catch (e) {
      status = `오류: ${e}`;
    } finally {
      virtLoading = false;
    }
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

    const unlisten = await listen<ProgressEvent>("disable-progress", (e) => {
      const { step, total, message } = e.payload;
      disableLog = [...disableLog, `  [${step}/${total}] ${message}`];
    });

    try {
      const options: DisableOptions | null = virtChecked
        ? computeDisableOptions(virtItems)
        : null;
      const results = await invoke<DisableResult[]>("execute_disable", {
        options,
      });
      for (const r of results) {
        disableLog = [
          ...disableLog,
          "",
          `${r.success ? "✅" : "⚠️"} ${r.task}`,
          r.message,
        ];
      }
      disableLog = [
        ...disableLog,
        "",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        "✅ 모든 작업이 완료되었습니다.",
        "변경 사항을 적용하려면 재부팅이 필요합니다.",
      ];
      status = "비활성화 완료 — 재부팅 필요";
      disableComplete = true;
    } catch (e) {
      disableLog = [...disableLog, `❌ 오류: ${e}`];
      status = "오류 발생";
    } finally {
      disableRunning = false;
      unlisten();
    }
  }

  async function requestReboot() {
    const ok = window.confirm(
      "지금 재부팅하시겠습니까?\n\n확인을 클릭하면 5초 후 자동으로 재부팅됩니다."
    );
    if (!ok) return;
    try {
      await invoke("request_reboot");
    } catch (e) {
      status = `재부팅 오류: ${e}`;
    }
  }

  /// 가상화 점검 결과로부터 비활성화 옵션 계산
  function computeDisableOptions(items: VirtItem[]): DisableOptions {
    const find = (cat: string) => items.find((i) => i.category === cat);
    const isActive = (item: VirtItem | undefined) =>
      item !== undefined &&
      (item.status.includes("활성화됨") || item.status.includes("설치됨 (활성)"));

    const hypervActive =
      isActive(find("Hyper-V (전체)")) ||
      isActive(find("Hyper-V 하이퍼바이저"));

    const hypervisorLaunch = find("Hypervisor 시작 유형");
    const hypervisorActive =
      hypervisorLaunch !== undefined &&
      !hypervisorLaunch.status.toLowerCase().includes("off") &&
      hypervisorLaunch.status !== "확인 불가";

    return {
      hyperv: hypervActive || hypervisorActive,
      wsl:
        isActive(find("WSL")) ||
        isActive(find("가상 머신 플랫폼 (WSL2)")),
      vbs:
        isActive(find("VBS (가상화 기반 보안)")) ||
        isActive(find("CredentialGuard")) ||
        isActive(find("LSA 보호")),
      core_isolation: isActive(find("HVCI (코어 격리)")),
    };
  }

  // 상태에 따른 상태 표시 색상
  function statusColor(s: string): string {
    if (s.includes("활성화됨") || s.includes("활성)") || s.includes("설치됨 (활성)"))
      return "text-red-600 font-semibold";
    if (s.includes("비활성") || s.includes("미설치") || s.includes("Off") || s.includes("지원됨"))
      return "text-green-600";
    return "text-gray-700";
  }
</script>

<div class="flex flex-col h-screen bg-gray-50 select-none">
  <!-- Header -->
  <header class="bg-slate-800 text-white px-6 py-3 flex items-center justify-between">
    <span class="text-lg font-bold">VM Compatibility Tool</span>
    {#if currentPanel !== "menu"}
      <button
        onclick={() => showPanel("menu")}
        class="text-sm text-slate-300 hover:text-white transition-colors"
      >
        ← 메인 메뉴
      </button>
    {/if}
  </header>

  <!-- Content -->
  <main class="flex-1 overflow-auto p-5">

    <!-- 메인 메뉴 -->
    {#if currentPanel === "menu"}
      <div class="flex flex-col items-center justify-center h-full gap-5">
        <p class="text-gray-400 text-sm mb-2">작업을 선택하세요</p>
        <button
          onclick={loadSystemInfo}
          class="w-72 py-4 text-base font-bold bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors shadow"
        >
          🖥️ 시스템 사양 체크
        </button>
        <button
          onclick={loadVirtStatus}
          class="w-72 py-4 text-base font-bold bg-amber-500 hover:bg-amber-600 text-white rounded-lg transition-colors shadow"
        >
          🔍 가상화 설정 점검
        </button>
        <button
          onclick={() => showPanel("disable")}
          class="w-72 py-4 text-base font-bold bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors shadow"
        >
          ⚙️ VBS 및 Hyper-V 비활성화
        </button>
      </div>

    <!-- 시스템 정보 -->
    {:else if currentPanel === "systemInfo"}
      <div class="flex flex-col gap-3 h-full">
        <div class="flex items-center justify-between shrink-0">
          <h2 class="text-base font-bold text-gray-800">시스템 상세 정보</h2>
          <div class="flex gap-2">
            <button
              onclick={refreshSystemInfo}
              disabled={systemLoading}
              class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded transition-colors"
            >
              새로고침
            </button>
            <button
              onclick={exportSystemCsv}
              disabled={systemLoading || systemItems.length === 0}
              class="px-3 py-1.5 text-xs bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded transition-colors"
            >
              CSV 내보내기
            </button>
          </div>
        </div>

        {#if systemLoading}
          <div class="flex-1 flex items-center justify-center text-gray-400 text-sm">
            수집 중...
          </div>
        {:else}
          <div class="flex-1 overflow-auto">
            <table class="w-full text-sm border-collapse bg-white rounded shadow-sm">
              <thead class="bg-gray-100 sticky top-0">
                <tr>
                  <th class="text-left px-3 py-2 border-b w-28 font-semibold text-gray-700">항목</th>
                  <th class="text-left px-3 py-2 border-b w-36 font-semibold text-gray-700">세부 정보</th>
                  <th class="text-left px-3 py-2 border-b font-semibold text-gray-700">값</th>
                </tr>
              </thead>
              <tbody>
                {#each systemItems as item, i}
                  <tr class={i % 2 === 0 ? "bg-white" : "bg-gray-50"}>
                    <td class="px-3 py-1.5 border-b font-medium text-gray-800">{item.category}</td>
                    <td class="px-3 py-1.5 border-b text-gray-600">{item.item}</td>
                    <td class="px-3 py-1.5 border-b text-gray-900">{item.value}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

    <!-- 가상화 점검 -->
    {:else if currentPanel === "virtualization"}
      <div class="flex flex-col gap-3 h-full">
        <div class="flex items-center justify-between shrink-0">
          <h2 class="text-base font-bold text-gray-800">가상화 설정 점검</h2>
          <div class="flex gap-2">
            <button
              onclick={loadVirtStatus}
              disabled={virtLoading}
              class="px-3 py-1.5 text-xs bg-gray-200 hover:bg-gray-300 disabled:opacity-50 rounded transition-colors"
            >
              재점검
            </button>
            <button
              onclick={exportVirtCsv}
              disabled={virtLoading || virtItems.length === 0}
              class="px-3 py-1.5 text-xs bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded transition-colors"
            >
              CSV 내보내기
            </button>
          </div>
        </div>

        {#if virtLoading}
          <div class="flex-1 flex items-center justify-center text-gray-400 text-sm">
            점검 중...
          </div>
        {:else}
          <div class="flex-1 overflow-auto">
            <table class="w-full text-sm border-collapse bg-white rounded shadow-sm">
              <thead class="bg-gray-100 sticky top-0">
                <tr>
                  <th class="text-left px-3 py-2 border-b w-44 font-semibold text-gray-700">항목</th>
                  <th class="text-left px-3 py-2 border-b w-28 font-semibold text-gray-700">상태</th>
                  <th class="text-left px-3 py-2 border-b font-semibold text-gray-700">상세 정보</th>
                  <th class="text-left px-3 py-2 border-b w-48 font-semibold text-gray-700">권장사항</th>
                </tr>
              </thead>
              <tbody>
                {#each virtItems as item, i}
                  <tr class={i % 2 === 0 ? "bg-white" : "bg-gray-50"}>
                    <td class="px-3 py-2 border-b font-medium text-gray-800">{item.category}</td>
                    <td class="px-3 py-2 border-b">
                      <span class={statusColor(item.status)}>{item.status}</span>
                    </td>
                    <td class="px-3 py-2 border-b text-gray-500 text-xs">{item.details}</td>
                    <td class="px-3 py-2 border-b text-amber-700 text-xs">{item.recommendation}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

    <!-- 비활성화 -->
    {:else if currentPanel === "disable"}
      <div class="flex flex-col gap-4 h-full">
        <h2 class="text-base font-bold text-gray-800 shrink-0">VBS 및 Hyper-V 비활성화</h2>

        <div class="bg-red-50 border border-red-200 rounded p-4 text-sm text-red-800 shrink-0">
          <p class="font-bold mb-2">⚠️ 이 작업은 다음을 수행합니다</p>
          <ul class="list-disc list-inside space-y-1 text-xs">
            <li>Hyper-V 및 관련 기능 제거 (DISM)</li>
            <li>WSL2 제거 (DISM)</li>
            <li>VBS (가상화 기반 보안) 레지스트리 비활성화</li>
            <li>코어 격리 비활성화</li>
            <li>hypervisorlaunchtype off (bcdedit)</li>
          </ul>
          {#if virtChecked}
            <p class="mt-2 text-green-700 font-semibold">✅ 가상화 점검 결과 기반으로 선택적 실행됩니다</p>
          {:else}
            <p class="mt-2 text-amber-700">ℹ️ 가상화 점검 없이 모든 항목을 일괄 처리합니다</p>
          {/if}
        </div>

        <div class="flex gap-3 shrink-0">
          <button
            onclick={runDisable}
            disabled={disableRunning}
            class="px-5 py-2.5 font-bold bg-red-500 hover:bg-red-600 disabled:bg-gray-300 text-white rounded transition-colors"
          >
            {disableRunning ? "실행 중..." : "비활성화 실행"}
          </button>
          {#if disableComplete}
            <button
              onclick={requestReboot}
              class="px-5 py-2.5 font-bold bg-slate-700 hover:bg-slate-800 text-white rounded transition-colors"
            >
              🔄 지금 재부팅
            </button>
          {/if}
        </div>

        <div class="flex-1 overflow-hidden">
          <pre class="h-full bg-gray-900 text-green-400 rounded p-4 text-xs font-mono overflow-y-auto whitespace-pre-wrap"
          >{disableLog.join("\n") || "비활성화 실행 버튼을 클릭하면 작업이 시작됩니다."}</pre>
        </div>
      </div>
    {/if}
  </main>

  <!-- Status Bar -->
  <footer class="bg-gray-200 px-4 py-1.5 flex justify-between text-xs text-gray-600 shrink-0">
    <span>{status}</span>
    <span class="text-gray-400">{version}</span>
  </footer>
</div>
