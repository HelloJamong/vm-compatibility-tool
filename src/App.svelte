<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Panel = "menu" | "systemInfo" | "virtualization" | "disable";
  type SystemInfoItem = { category: string; item: string; value: string };
  type VirtItem = { category: string; status: string; details: string; recommendation: string };
  type DisableResult = { task: string; success: boolean; message: string };

  let currentPanel = $state<Panel>("menu");
  let status = $state("준비됨");
  let version = $state("nightly");

  // 시스템 정보
  let systemItems = $state<SystemInfoItem[]>([]);
  let systemLoading = $state(false);

  // 가상화 점검
  let virtItems = $state<VirtItem[]>([]);
  let virtLoading = $state(false);
  let virtChecked = $state(false);

  // 비활성화
  let disableLog = $state<string[]>([]);
  let disableRunning = $state(false);

  function showPanel(panel: Panel) {
    currentPanel = panel;
  }

  async function loadSystemInfo() {
    showPanel("systemInfo");
    systemLoading = true;
    status = "시스템 정보 수집 중...";
    try {
      systemItems = await invoke<SystemInfoItem[]>("get_system_info");
      status = "시스템 정보 수집 완료";
    } catch (e) {
      status = `오류: ${e}`;
    } finally {
      systemLoading = false;
    }
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

  async function runDisable() {
    disableRunning = true;
    disableLog = ["비활성화 작업을 시작합니다..."];
    status = "비활성화 실행 중...";
    try {
      const results = await invoke<DisableResult[]>("execute_disable", { selective: virtChecked });
      for (const r of results) {
        disableLog = [...disableLog, `\n[${r.task}]`, r.message];
      }
      disableLog = [...disableLog, "\n✅ 모든 작업 완료. 재부팅이 필요합니다."];
      status = "비활성화 완료";
    } catch (e) {
      disableLog = [...disableLog, `❌ 오류: ${e}`];
      status = "오류 발생";
    } finally {
      disableRunning = false;
    }
  }
</script>

<div class="flex flex-col h-screen bg-gray-50">
  <!-- Header -->
  <header class="bg-slate-800 text-white px-6 py-4 text-center text-2xl font-bold">
    VM Compatibility Tool
  </header>

  <!-- Content -->
  <main class="flex-1 overflow-auto p-6">

    <!-- 메인 메뉴 -->
    {#if currentPanel === "menu"}
      <div class="flex flex-col items-center justify-center h-full gap-6">
        <button onclick={loadSystemInfo}
          class="w-72 py-5 text-lg font-bold bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors">
          시스템 사양 체크
        </button>
        <button onclick={loadVirtStatus}
          class="w-72 py-5 text-lg font-bold bg-amber-500 hover:bg-amber-600 text-white rounded-lg transition-colors">
          가상화 설정 점검
        </button>
        <button onclick={() => showPanel("disable")}
          class="w-72 py-5 text-lg font-bold bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors">
          VBS 및 Hyper-V 비활성화
        </button>
      </div>

    <!-- 시스템 정보 -->
    {:else if currentPanel === "systemInfo"}
      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <button onclick={() => showPanel("menu")} class="text-sm text-gray-500 hover:text-gray-700">← 메인 메뉴로</button>
          <h2 class="text-lg font-bold">시스템 상세 정보</h2>
          <div class="w-24"></div>
        </div>
        {#if systemLoading}
          <p class="text-center text-gray-500 py-8">수집 중...</p>
        {:else}
          <table class="w-full text-sm border-collapse bg-white rounded shadow-sm">
            <thead class="bg-gray-100">
              <tr>
                <th class="text-left px-3 py-2 border-b w-28 font-semibold">항목</th>
                <th class="text-left px-3 py-2 border-b w-36 font-semibold">세부 정보</th>
                <th class="text-left px-3 py-2 border-b font-semibold">값</th>
              </tr>
            </thead>
            <tbody>
              {#each systemItems as item, i}
                <tr class={i % 2 === 0 ? "bg-white" : "bg-gray-50"}>
                  <td class="px-3 py-1.5 border-b font-medium">{item.category}</td>
                  <td class="px-3 py-1.5 border-b">{item.item}</td>
                  <td class="px-3 py-1.5 border-b">{item.value}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

    <!-- 가상화 점검 -->
    {:else if currentPanel === "virtualization"}
      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <button onclick={() => showPanel("menu")} class="text-sm text-gray-500 hover:text-gray-700">← 메인 메뉴로</button>
          <h2 class="text-lg font-bold">가상화 설정 점검</h2>
          <div class="w-24"></div>
        </div>
        {#if virtLoading}
          <p class="text-center text-gray-500 py-8">점검 중...</p>
        {:else}
          <table class="w-full text-sm border-collapse bg-white rounded shadow-sm">
            <thead class="bg-gray-100">
              <tr>
                <th class="text-left px-3 py-2 border-b w-40 font-semibold">항목</th>
                <th class="text-left px-3 py-2 border-b w-28 font-semibold">상태</th>
                <th class="text-left px-3 py-2 border-b font-semibold">상세 정보</th>
                <th class="text-left px-3 py-2 border-b w-48 font-semibold">권장사항</th>
              </tr>
            </thead>
            <tbody>
              {#each virtItems as item, i}
                <tr class={i % 2 === 0 ? "bg-white" : "bg-gray-50"}>
                  <td class="px-3 py-2 border-b font-medium">{item.category}</td>
                  <td class="px-3 py-2 border-b">
                    <span class={item.status.includes("활성화됨") || item.status.includes("활성)") ? "text-red-600 font-semibold" : "text-green-600"}>
                      {item.status}
                    </span>
                  </td>
                  <td class="px-3 py-2 border-b text-gray-600">{item.details}</td>
                  <td class="px-3 py-2 border-b text-amber-700 text-xs">{item.recommendation}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

    <!-- 비활성화 -->
    {:else if currentPanel === "disable"}
      <div class="flex flex-col gap-4">
        <button onclick={() => showPanel("menu")} class="text-sm text-gray-500 hover:text-gray-700 self-start">← 메인 메뉴로</button>
        <h2 class="text-lg font-bold">VBS 및 Hyper-V 비활성화</h2>

        <div class="bg-red-50 border border-red-200 rounded p-4 text-sm text-red-800">
          <p class="font-bold mb-2">⚠️ 주의: 이 작업은 다음을 수행합니다</p>
          <ul class="list-disc list-inside space-y-1">
            <li>Hyper-V 및 관련 기능 제거 (DISM)</li>
            <li>WSL2 제거 (DISM)</li>
            <li>VBS (가상화 기반 보안) 레지스트리 비활성화</li>
            <li>코어 격리 비활성화</li>
          </ul>
        </div>

        <button onclick={runDisable} disabled={disableRunning}
          class="w-48 py-2.5 font-bold bg-red-500 hover:bg-red-600 disabled:bg-gray-300 text-white rounded transition-colors">
          {disableRunning ? "실행 중..." : "비활성화 실행"}
        </button>

        <pre class="bg-gray-900 text-green-400 rounded p-4 text-xs font-mono h-64 overflow-y-auto whitespace-pre-wrap">
{disableLog.join("\n") || "비활성화 작업을 실행하려면 위의 버튼을 클릭하세요."}
        </pre>
      </div>
    {/if}
  </main>

  <!-- Status Bar -->
  <footer class="bg-gray-200 px-4 py-1.5 flex justify-between text-xs text-gray-600">
    <span>{status}</span>
    <span>{version}</span>
  </footer>
</div>
