export type Panel = "menu" | "systemInfo" | "virtualization" | "disable";
export type SystemInfoItem = { category: string; item: string; value: string };
export type DisableGroup = "hyperv" | "wsl" | "vbs" | "core_isolation";
export type VirtSource = "unknown" | "wmi" | "feature" | "bcd" | "registry";
export type VirtItem = {
  category: string;
  status: string;
  details: string;
  recommendation: string;
  disable_group: DisableGroup | null;
  source_type: VirtSource;
  action_required: boolean;
  manifest_id: string | null;
};
export type DisableResult = { task: string; success: boolean; message: string };
export type DisableOutput = {
  results: DisableResult[];
  log_path: string | null;
  backup_path: string | null;
};
export type ProgressEvent = {
  step: number;
  total: number;
  message: string;
  success: boolean;
};
export type DisableOptions = {
  hyperv: boolean;
  wsl: boolean;
  vbs: boolean;
  core_isolation: boolean;
};
