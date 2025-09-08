using System;
using System.Diagnostics;
using System.IO;
using System.Management;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using Microsoft.Win32;

namespace VmCompatibilityTool
{
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();
        }

        private void ShowPanel(string panelName)
        {
            MenuPanel.Visibility = System.Windows.Visibility.Collapsed;
            SystemInfoPanel.Visibility = System.Windows.Visibility.Collapsed;
            DisablePanel.Visibility = System.Windows.Visibility.Collapsed;

            switch (panelName)
            {
                case "Menu":
                    MenuPanel.Visibility = System.Windows.Visibility.Visible;
                    break;
                case "SystemInfo":
                    SystemInfoPanel.Visibility = System.Windows.Visibility.Visible;
                    break;
                case "Disable":
                    DisablePanel.Visibility = System.Windows.Visibility.Visible;
                    break;
            }
        }

        private void SystemCheckButton_Click(object sender, RoutedEventArgs e)
        {
            ShowPanel("SystemInfo");
            LoadDetailedSystemInfo();
        }

        private void DisableVbsHyperVButton_Click(object sender, RoutedEventArgs e)
        {
            ShowPanel("Disable");
        }

        private void BackToMenuButton_Click(object sender, RoutedEventArgs e)
        {
            ShowPanel("Menu");
        }

        private async void LoadDetailedSystemInfo()
        {
            SystemInfoTextBox.Text = "시스템 정보를 수집하고 있습니다...\n";
            StatusTextBlock.Text = "시스템 정보 수집 중...";

            try
            {
                await Task.Run(() => CollectSystemInfo());
            }
            catch (Exception ex)
            {
                SystemInfoTextBox.Text = $"시스템 정보 수집 중 오류가 발생했습니다: {ex.Message}";
            }
            finally
            {
                StatusTextBlock.Text = "시스템 정보 수집 완료";
            }
        }

        private void CollectSystemInfo()
        {
            var result = new StringBuilder();
            result.AppendLine("=== 시스템 상세 정보 ===");
            result.AppendLine();

            try
            {
                // Windows 버전 정보 (상세)
                result.AppendLine("[운영체제 정보]");
                var osInfo = GetWindowsVersionInfo();
                result.AppendLine(osInfo);
                result.AppendLine();

                // CPU 정보
                result.AppendLine("[프로세서 정보]");
                var cpuInfo = GetCpuInfo();
                result.AppendLine(cpuInfo);
                result.AppendLine();

                // 메모리 정보
                result.AppendLine("[메모리 정보]");
                var memoryInfo = GetMemoryInfo();
                result.AppendLine(memoryInfo);
                result.AppendLine();

                // 디스크 정보
                result.AppendLine("[디스크 정보]");
                var diskInfo = GetDiskInfo();
                result.AppendLine(diskInfo);
                result.AppendLine();

                // 가상화 지원 정보
                result.AppendLine("[가상화 지원]");
                var virtualizationInfo = GetVirtualizationInfo();
                result.AppendLine(virtualizationInfo);
                result.AppendLine();

                // 부팅 시간
                result.AppendLine("[부팅 정보]");
                var bootTime = GetBootTime();
                result.AppendLine(bootTime);
                result.AppendLine();

                Dispatcher.Invoke(() =>
                {
                    SystemInfoTextBox.Text = result.ToString();
                });
            }
            catch (Exception ex)
            {
                Dispatcher.Invoke(() =>
                {
                    SystemInfoTextBox.Text = $"오류 발생: {ex.Message}";
                });
            }
        }

        private string GetWindowsVersionInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                // 레지스트리에서 상세 Windows 버전 정보 가져오기
                using (var key = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\Microsoft\Windows NT\CurrentVersion"))
                {
                    if (key != null)
                    {
                        var productName = key.GetValue("ProductName")?.ToString() ?? "알 수 없음";
                        var displayVersion = key.GetValue("DisplayVersion")?.ToString() ?? "알 수 없음";
                        var currentBuild = key.GetValue("CurrentBuild")?.ToString() ?? "알 수 없음";
                        var ubr = key.GetValue("UBR")?.ToString() ?? "0";
                        
                        // 빌드 넘버로 Windows 10/11 구분
                        string osVersion = "Windows";
                        if (int.TryParse(currentBuild, out int buildNumber))
                        {
                            if (buildNumber >= 22000)
                            {
                                osVersion = "Windows 11";
                            }
                            else if (buildNumber >= 10240)
                            {
                                osVersion = "Windows 10";
                            }
                            else if (buildNumber >= 9200)
                            {
                                osVersion = "Windows 8";
                            }
                            else
                            {
                                osVersion = "Windows";
                            }
                        }
                        
                        // Windows 버전 정보를 정확히 표시
                        string finalVersion = displayVersion;
                        
                        // Windows 11의 경우 24H2 빌드 번호 업데이트
                        if (osVersion == "Windows 11" && int.TryParse(currentBuild, out int win11Build))
                        {
                            if (win11Build >= 26100 && displayVersion == "23H2")
                            {
                                finalVersion = "24H2"; // 빌드 번호가 24H2 범위인데 DisplayVersion이 23H2인 경우 수정
                            }
                        }
                        
                        result.AppendLine($"운영체제: {osVersion}");
                        result.AppendLine($"버전: {finalVersion}");
                        result.AppendLine($"빌드: {currentBuild}.{ubr}");
                    }
                }
                
                return result.ToString();
            }
            catch
            {
                return "Windows 버전 정보를 가져올 수 없습니다.";
            }
        }

        private string GetCpuInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_Processor"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        result.AppendLine($"모델: {obj["Name"]}");
                        result.AppendLine($"제조사: {obj["Manufacturer"]}");
                        result.AppendLine($"코어 수: {obj["NumberOfCores"]}");
                        result.AppendLine($"논리 프로세서 수: {obj["NumberOfLogicalProcessors"]}");
                        result.AppendLine($"최대 클럭: {obj["MaxClockSpeed"]} MHz");
                        break;
                    }
                }
                
                return result.ToString();
            }
            catch
            {
                return "CPU 정보를 가져올 수 없습니다.";
            }
        }

        private string GetMemoryInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT TotalPhysicalMemory FROM Win32_ComputerSystem"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var totalMemory = Convert.ToInt64(obj["TotalPhysicalMemory"]);
                        var memoryGB = Math.Round(totalMemory / (1024.0 * 1024.0 * 1024.0), 2);
                        result.AppendLine($"총 물리적 메모리: {memoryGB:F2} GB");
                        break;
                    }
                }
                
                // 사용 가능한 메모리 (WMI 사용)
                try
                {
                    using (var perfSearcher = new ManagementObjectSearcher("SELECT AvailableBytes FROM Win32_PerfRawData_PerfOS_Memory"))
                    {
                        foreach (ManagementObject perfObj in perfSearcher.Get())
                        {
                            var availableBytes = Convert.ToInt64(perfObj["AvailableBytes"]);
                            var availableGB = Math.Round(availableBytes / (1024.0 * 1024.0 * 1024.0), 2);
                            result.AppendLine($"사용 가능한 메모리: {availableGB:F2} GB");
                            break;
                        }
                    }
                }
                catch
                {
                    result.AppendLine("사용 가능한 메모리: 확인 불가");
                }
                
                return result.ToString();
            }
            catch
            {
                return "메모리 정보를 가져올 수 없습니다.";
            }
        }

        private string GetDiskInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_LogicalDisk WHERE DriveType=3"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var driveLetter = obj["DeviceID"].ToString();
                        var totalSize = Convert.ToInt64(obj["Size"]);
                        var freeSpace = Convert.ToInt64(obj["FreeSpace"]);
                        var usedSpace = totalSize - freeSpace;
                        
                        var totalGB = Math.Round(totalSize / (1024.0 * 1024.0 * 1024.0), 2);
                        var usedGB = Math.Round(usedSpace / (1024.0 * 1024.0 * 1024.0), 2);
                        var freeGB = Math.Round(freeSpace / (1024.0 * 1024.0 * 1024.0), 2);
                        
                        result.AppendLine($"드라이브 {driveLetter}");
                        result.AppendLine($"  총 용량: {totalGB:F2} GB");
                        result.AppendLine($"  사용 중: {usedGB:F2} GB");
                        result.AppendLine($"  여유 공간: {freeGB:F2} GB");
                        
                        // SSD/HDD 구분
                        var mediaType = GetDriveMediaType(driveLetter.Replace(":", ""));
                        result.AppendLine($"  타입: {mediaType}");
                        result.AppendLine();
                    }
                }
                
                return result.ToString();
            }
            catch
            {
                return "디스크 정보를 가져올 수 없습니다.";
            }
        }

        private string GetDriveMediaType(string driveLetter)
        {
            try
            {
                using (var searcher = new ManagementObjectSearcher($"SELECT * FROM Win32_DiskDrive"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var mediaType = obj["MediaType"]?.ToString();
                        if (mediaType != null && mediaType.Contains("SSD"))
                            return "SSD";
                    }
                }
                return "HDD";
            }
            catch
            {
                return "알 수 없음";
            }
        }

        private string GetVirtualizationInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT VirtualizationFirmwareEnabled FROM Win32_Processor"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var isEnabled = obj["VirtualizationFirmwareEnabled"];
                        if (isEnabled != null && (bool)isEnabled)
                        {
                            result.AppendLine("하드웨어 가상화: 활성화됨");
                        }
                        else
                        {
                            result.AppendLine("하드웨어 가상화: 비활성화됨");
                        }
                        break;
                    }
                }
                
                // Hyper-V 상태 확인
                var hyperVStatus = CheckHyperVInstalled();
                result.AppendLine($"Hyper-V: {hyperVStatus}");
                
                // VBS 상태 확인
                var vbsStatus = CheckVbsStatus();
                result.AppendLine($"VBS (가상화 기반 보안): {vbsStatus}");
                
                return result.ToString();
            }
            catch
            {
                return "가상화 정보를 가져올 수 없습니다.";
            }
        }

        private string CheckHyperVInstalled()
        {
            try
            {
                using (var key = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\OptionalFeatures\Microsoft-Hyper-V-All"))
                {
                    if (key != null)
                    {
                        var enabled = key.GetValue("Enabled");
                        return enabled != null && enabled.ToString() == "1" ? "설치됨 (활성화)" : "설치됨 (비활성화)";
                    }
                }
                return "설치되지 않음";
            }
            catch
            {
                return "확인 불가";
            }
        }

        private string CheckVbsStatus()
        {
            try
            {
                using (var key = Registry.LocalMachine.OpenSubKey(@"SYSTEM\CurrentControlSet\Control\DeviceGuard"))
                {
                    if (key != null)
                    {
                        var enabled = key.GetValue("EnableVirtualizationBasedSecurity");
                        return enabled != null && enabled.ToString() == "1" ? "활성화됨" : "비활성화됨";
                    }
                }
                return "비활성화됨";
            }
            catch
            {
                return "확인 불가";
            }
        }

        private string GetBootTime()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT LastBootUpTime FROM Win32_OperatingSystem"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var bootTimeStr = obj["LastBootUpTime"].ToString();
                        var bootTime = ManagementDateTimeConverter.ToDateTime(bootTimeStr);
                        var upTime = DateTime.Now - bootTime;
                        
                        result.AppendLine($"마지막 부팅 시간: {bootTime:yyyy-MM-dd HH:mm:ss}");
                        result.AppendLine($"시스템 가동 시간: {upTime.Days}일 {upTime.Hours}시간 {upTime.Minutes}분");
                        break;
                    }
                }
                
                return result.ToString();
            }
            catch
            {
                return "부팅 시간 정보를 가져올 수 없습니다.";
            }
        }

        private async void ExecuteDisableButton_Click(object sender, RoutedEventArgs e)
        {
            var result = MessageBox.Show(
                "이 작업은 시스템의 보안 기능을 비활성화하며, 완료 후 재부팅이 필요합니다.\n\n" +
                "계속하시겠습니까?",
                "경고",
                MessageBoxButton.YesNo,
                MessageBoxImage.Warning);

            if (result != MessageBoxResult.Yes)
                return;

            ExecuteDisableButton.IsEnabled = false;
            DisableResultTextBox.Text = "VBS 및 Hyper-V 비활성화 작업을 시작합니다...\n";
            StatusTextBlock.Text = "비활성화 작업 실행 중...";

            try
            {
                await Task.Run(() => DisableVbsAndHyperV());
            }
            finally
            {
                ExecuteDisableButton.IsEnabled = true;
                StatusTextBlock.Text = "비활성화 작업 완료";
            }
        }

        private void DisableVbsAndHyperV()
        {
            var result = new StringBuilder();
            result.AppendLine("=== VBS 및 Hyper-V 비활성화 작업 ===");
            result.AppendLine();

            // 1. Hyper-V 기능 비활성화
            result.AppendLine("1. Hyper-V 기능 비활성화 중...");
            var hyperVResult = DisableHyperVFeatures();
            result.AppendLine(hyperVResult);
            result.AppendLine();

            // 2. WSL2 비활성화
            result.AppendLine("2. WSL2 비활성화 중...");
            var wslResult = DisableWSL2();
            result.AppendLine(wslResult);
            result.AppendLine();

            // 3. VBS 비활성화
            result.AppendLine("3. VBS (가상화 기반 보안) 비활성화 중...");
            var vbsResult = DisableVBS();
            result.AppendLine(vbsResult);
            result.AppendLine();

            // 4. 코어 격리 비활성화
            result.AppendLine("4. 코어 격리 비활성화 중...");
            var coreIsolationResult = DisableCoreIsolation();
            result.AppendLine(coreIsolationResult);
            result.AppendLine();

            result.AppendLine("모든 비활성화 작업이 완료되었습니다.");
            result.AppendLine("변경사항을 적용하려면 시스템을 재부팅해야 합니다.");

            Dispatcher.Invoke(() =>
            {
                DisableResultTextBox.Text = result.ToString();
                ShowRebootDialog();
            });
        }

        private string DisableHyperVFeatures()
        {
            try
            {
                var result = new StringBuilder();
                
                // DISM을 사용하여 Hyper-V 기능들 비활성화
                var hyperVFeatures = new[]
                {
                    "Microsoft-Hyper-V-All",
                    "Microsoft-Hyper-V",
                    "Microsoft-Hyper-V-Tools-All",
                    "Microsoft-Hyper-V-Management-PowerShell",
                    "Microsoft-Hyper-V-Hypervisor",
                    "Microsoft-Hyper-V-Services",
                    "Microsoft-Hyper-V-Management-Clients"
                };

                foreach (var feature in hyperVFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /disable-feature /featurename:{feature} /norestart",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        process.WaitForExit();
                        
                        if (process.ExitCode == 0)
                        {
                            result.AppendLine($"   ✓ {feature} 비활성화 성공");
                        }
                        else
                        {
                            result.AppendLine($"   ○ {feature} 이미 비활성화되어 있음");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"   ✗ {feature} 비활성화 실패");
                    }
                }
                
                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"   ✗ Hyper-V 비활성화 중 오류 발생: {ex.Message}";
            }
        }

        private string DisableWSL2()
        {
            try
            {
                var result = new StringBuilder();
                
                var wslFeatures = new[]
                {
                    "Microsoft-Windows-Subsystem-Linux",
                    "VirtualMachinePlatform"
                };

                foreach (var feature in wslFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /disable-feature /featurename:{feature} /norestart",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        process.WaitForExit();
                        
                        if (process.ExitCode == 0)
                        {
                            result.AppendLine($"   ✓ {feature} 비활성화 성공");
                        }
                        else
                        {
                            result.AppendLine($"   ○ {feature} 이미 비활성화되어 있음");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"   ✗ {feature} 비활성화 실패");
                    }
                }
                
                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"   ✗ WSL2 비활성화 중 오류 발생: {ex.Message}";
            }
        }

        private string DisableVBS()
        {
            try
            {
                var result = new StringBuilder();
                
                // VBS 관련 레지스트리 설정
                var vbsKeys = new[]
                {
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "EnableVirtualizationBasedSecurity", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "RequirePlatformSecurityFeatures", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "WasEnabledBy", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard", "Enabled", 0)
                };

                foreach (var (keyPath, valueName, value) in vbsKeys)
                {
                    try
                    {
                        using (var key = Registry.LocalMachine.CreateSubKey(keyPath))
                        {
                            key?.SetValue(valueName, value, RegistryValueKind.DWord);
                            result.AppendLine($"   ✓ {keyPath}\\{valueName} = {value} 설정 완료");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"   ✗ {keyPath}\\{valueName} 설정 실패");
                    }
                }
                
                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"   ✗ VBS 비활성화 중 오류 발생: {ex.Message}";
            }
        }

        private string DisableCoreIsolation()
        {
            try
            {
                var result = new StringBuilder();
                
                // 코어 격리 관련 레지스트리 설정
                var coreIsolationKeys = new[]
                {
                    (@"SYSTEM\CurrentControlSet\Control\CI\Config", "VulnerableDriverBlocklistEnable", 0),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "EnableVirtualizationBasedSecurity", 0),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HypervisorEnforcedCodeIntegrity", 0),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HVCIMATRequired", 0)
                };

                foreach (var (keyPath, valueName, value) in coreIsolationKeys)
                {
                    try
                    {
                        using (var key = Registry.LocalMachine.CreateSubKey(keyPath))
                        {
                            key?.SetValue(valueName, value, RegistryValueKind.DWord);
                            result.AppendLine($"   ✓ {keyPath}\\{valueName} = {value} 설정 완료");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"   ✗ {keyPath}\\{valueName} 설정 실패");
                    }
                }
                
                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"   ✗ 코어 격리 비활성화 중 오류 발생: {ex.Message}";
            }
        }

        private void ShowRebootDialog()
        {
            var result = MessageBox.Show(
                "모든 변경사항을 적용하려면 시스템을 재부팅해야 합니다.\n\n" +
                "지금 재부팅하시겠습니까?",
                "재부팅 필요",
                MessageBoxButton.YesNo,
                MessageBoxImage.Question);

            if (result == MessageBoxResult.Yes)
            {
                // 지금 재부팅
                try
                {
                    var process = new Process
                    {
                        StartInfo = new ProcessStartInfo
                        {
                            FileName = "shutdown.exe",
                            Arguments = "/r /t 5 /c \"VMFort Compatibility Tool에 의한 시스템 재부팅\"",
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    
                    process.Start();
                    MessageBox.Show("5초 후 시스템이 재부팅됩니다.", "재부팅 예약됨", MessageBoxButton.OK, MessageBoxImage.Information);
                    Application.Current.Shutdown();
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"재부팅 실행 중 오류가 발생했습니다: {ex.Message}", "오류", MessageBoxButton.OK, MessageBoxImage.Error);
                }
            }
            else
            {
                MessageBox.Show("나중에 수동으로 시스템을 재부팅해 주세요.", "알림", MessageBoxButton.OK, MessageBoxImage.Information);
            }
        }

    }
}