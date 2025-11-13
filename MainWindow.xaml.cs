using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Management;
using System.Reflection;
using System.Security.Principal;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Markup;
using Microsoft.Win32;

namespace VmCompatibilityTool
{
    public class SystemInfoItem
    {
        public string Category { get; set; }
        public string Item { get; set; }
        public string Value { get; set; }
    }

    public class VirtualizationInfoItem
    {
        public string Category { get; set; }
        public string Status { get; set; }
        public string Details { get; set; }
        public string Recommendation { get; set; }
    }

    public partial class MainWindow : Window
    {
        private bool isVirtualizationCheckCompleted = false;
        private ObservableCollection<VirtualizationInfoItem> lastVirtualizationCheckResult = null;

        public MainWindow()
        {
            try
            {
                // 관리자 권한 확인
                if (!IsRunAsAdministrator())
                {
                    ShowAdminRequiredDialog();
                    return;
                }

                // 문화권 관련 초기화
                InitializeCultureSettings();
                
                InitializeComponent();
                SetVersionInfo();
                SetupExceptionHandlers();
                
            }
            catch (Exception ex)
            {
                MessageBox.Show($"프로그램 초기화 중 오류 발생: {ex.Message}", "초기화 오류", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private bool IsRunAsAdministrator()
        {
            try
            {
                var identity = WindowsIdentity.GetCurrent();
                var principal = new WindowsPrincipal(identity);
                return principal.IsInRole(WindowsBuiltInRole.Administrator);
            }
            catch (Exception ex)
            {
                // 권한 확인 실패 시 로그 남기고 false 반환 (안전한 기본값)
                System.Diagnostics.Debug.WriteLine($"관리자 권한 확인 실패: {ex.Message}");
                return false;
            }
        }

        private void ShowAdminRequiredDialog()
        {
            try
            {
                var result = MessageBox.Show(
                    "이 프로그램은 시스템 정보 수집 및 Windows 기능 변경을 위해 관리자 권한이 필요합니다.\n\n" +
                    "관리자 권한으로 다시 실행해 주세요.\n\n" +
                    "프로그램을 종료하시겠습니까?",
                    "관리자 권한 필요",
                    MessageBoxButton.YesNo,
                    MessageBoxImage.Warning);

                if (result == MessageBoxResult.Yes || result == MessageBoxResult.None)
                {
                    // 애플리케이션 종료
                    Application.Current.Shutdown();
                }
                else
                {
                    // 사용자가 No를 선택한 경우에도 안전을 위해 종료
                    // (관리자 권한 없이는 대부분의 기능이 정상 작동하지 않음)
                    MessageBox.Show(
                        "관리자 권한 없이는 프로그램이 정상적으로 작동하지 않을 수 있습니다.\n" +
                        "프로그램을 종료합니다.",
                        "알림",
                        MessageBoxButton.OK,
                        MessageBoxImage.Information);
                    
                    Application.Current.Shutdown();
                }
            }
            catch (Exception ex)
            {
                // 다이얼로그 표시 실패 시에도 안전하게 종료
                System.Diagnostics.Debug.WriteLine($"관리자 권한 다이얼로그 표시 실패: {ex.Message}");
                try
                {
                    MessageBox.Show($"관리자 권한으로 실행해주세요.\n오류: {ex.Message}", "권한 오류", MessageBoxButton.OK, MessageBoxImage.Error);
                }
                catch
                {
                    // 최후 수단
                }
                Application.Current.Shutdown();
            }
        }

        private void InitializeCultureSettings()
        {
            try
            {
                // WPF 호환 문화권 설정 (InvariantCulture 회피)
                try
                {
                    // 현재 시스템의 문화권을 사용
                    var currentCulture = CultureInfo.CurrentCulture;
                    var currentUICulture = CultureInfo.CurrentUICulture;
                    
                    
                    // WPF 텍스트 렌더링을 위한 추가 초기화
                    System.Windows.FrameworkElement.LanguageProperty.OverrideMetadata(
                        typeof(System.Windows.FrameworkElement),
                        new System.Windows.FrameworkPropertyMetadata(
                            System.Windows.Markup.XmlLanguage.GetLanguage(currentCulture.IetfLanguageTag)
                        )
                    );
                }
                catch (Exception)
                {
                    // 문화권 정보 접근 실패 시 영어(미국) 문화권으로 설정 (WPF와 호환)
                    
                    try
                    {
                        // InvariantCulture 대신 en-US 사용 (WPF 호환성)
                        var fallbackCulture = new CultureInfo("en-US");
                        CultureInfo.CurrentCulture = fallbackCulture;
                        CultureInfo.CurrentUICulture = fallbackCulture;
                    }
                    catch (Exception)
                    {
                        // 최후의 수단으로 현재 문화권 유지
                    }
                }
            }
            catch (Exception)
            {
            }
        }

        private void SetupExceptionHandlers()
        {
            try
            {
                // UI 스레드의 처리되지 않은 예외 처리
                Dispatcher.UnhandledException += (sender, e) =>
                {
                    try
                    {
                        var timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
                        var errorMessage = $"[{timestamp}] 처리되지 않은 UI 예외: {e.Exception.Message}\n\n스택 추적:\n{e.Exception.StackTrace}\n\n내부 예외: {e.Exception.InnerException?.Message ?? "없음"}";
                        
                        // 로그 출력
                        System.Diagnostics.Debug.WriteLine(errorMessage);
                        
                        // 파일로 오류 로그 저장
                        SaveErrorLog(errorMessage, "UI_Exception");
                        
                        // 상세한 오류 정보를 보여주는 개선된 팝업
                        ShowDetailedErrorDialog(e.Exception, "UI 스레드 예외");
                        
                        // 예외를 처리된 것으로 표시 (프로그램 종료 방지)
                        e.Handled = true;
                    }
                    catch (Exception handlerEx)
                    {
                        // 예외 처리기에서도 오류 발생 시
                        try
                        {
                            var errorInfo = $"예외 처리기 오류: {handlerEx.Message}\n원본 예외: {e.Exception.Message}";
                            SaveErrorLog(errorInfo, "Handler_Exception");
                            MessageBox.Show($"심각한 오류가 발생했습니다. 로그 파일을 확인해주세요.\n\n{errorInfo}", "심각한 오류", MessageBoxButton.OK, MessageBoxImage.Error);
                        }
                        catch { }
                        e.Handled = false;
                    }
                };

                // 애플리케이션 도메인의 처리되지 않은 예외 처리
                AppDomain.CurrentDomain.UnhandledException += (sender, e) =>
                {
                    try
                    {
                        var exception = e.ExceptionObject as Exception;
                        var timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
                        var errorMessage = $"[{timestamp}] 처리되지 않은 도메인 예외: {exception?.Message ?? "알 수 없는 오류"}\n\n스택 추적:\n{exception?.StackTrace ?? "스택 추적 없음"}\n\n내부 예외: {exception?.InnerException?.Message ?? "없음"}";
                        
                        // 로그 출력
                        System.Diagnostics.Debug.WriteLine(errorMessage);
                        
                        // 파일로 로그 저장
                        SaveErrorLog(errorMessage, "Domain_Exception");
                        
                        // 종료되는 예외의 경우 더 상세한 정보 수집
                        if (e.IsTerminating && exception != null)
                        {
                            ShowDetailedErrorDialog(exception, "도메인 예외 (종료됨)");
                        }
                    }
                    catch (Exception domainEx)
                    {
                        // 최후 수단
                        var finalMessage = $"심각한 도메인 예외 처리기 실패: {domainEx.Message}";
                        try
                        {
                            SaveErrorLog(finalMessage, "Critical_Domain_Exception");
                        }
                        catch { }
                    }
                };
            }
            catch (Exception setupEx)
            {
                System.Diagnostics.Debug.WriteLine($"예외 처리기 설정 실패: {setupEx.Message}");
                SaveErrorLog($"예외 처리기 설정 실패: {setupEx.Message}", "Setup_Exception");
            }
        }

        private void SaveErrorLog(string errorMessage, string category)
        {
            try
            {
                var logDir = System.IO.Path.Combine(Environment.CurrentDirectory, "Logs");
                if (!System.IO.Directory.Exists(logDir))
                {
                    System.IO.Directory.CreateDirectory(logDir);
                }

                var timestamp = DateTime.Now.ToString("yyyy-MM-dd");
                var logFileName = $"VM_Error_{category}_{timestamp}.log";
                var logPath = System.IO.Path.Combine(logDir, logFileName);

                var separator = new string('=', 80);
                var logEntry = $"\n{separator}\n[{DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}] {category}\n{separator}\n{errorMessage}\n";
                
                System.IO.File.AppendAllText(logPath, logEntry, Encoding.UTF8);
                
            }
            catch (Exception logEx)
            {
                System.Diagnostics.Debug.WriteLine($"로그 저장 실패: {logEx.Message}");
            }
        }

        private void ShowDetailedErrorDialog(Exception ex, string title)
        {
            try
            {
                var detailedMessage = $"오류 유형: {ex.GetType().Name}\n" +
                                    $"오류 메시지: {ex.Message}\n\n" +
                                    $"발생 위치: {ex.TargetSite?.Name ?? "알 수 없음"}\n\n" +
                                    $"스택 추적:\n{ex.StackTrace}\n\n" +
                                    $"내부 예외: {ex.InnerException?.Message ?? "없음"}\n\n" +
                                    $"로그 파일이 Logs 폴더에 저장되었습니다.\n" +
                                    $"계속 진행하시겠습니까?";

                var result = MessageBox.Show(detailedMessage, $"상세 오류 정보 - {title}", 
                                           MessageBoxButton.YesNo, MessageBoxImage.Warning);
                
                if (result == MessageBoxResult.No)
                {
                    Application.Current.Shutdown();
                }
            }
            catch (Exception)
            {
                // 최소한의 정보라도 표시
                MessageBox.Show($"심각한 오류 발생:\n{ex.Message}\n\n로그 파일을 확인해주세요.", 
                              "오류", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }


        private void SetVersionInfo()
        {
            try
            {
                var assembly = Assembly.GetExecutingAssembly();
                var version = assembly.GetName().Version;
                VersionTextBlock.Text = $"v{version.Major}.{version.Minor}.{version.Build}";
            }
            catch
            {
                VersionTextBlock.Text = "v1.2.3";
            }
        }

        private void ShowPanel(string panelName)
        {
            MenuPanel.Visibility = System.Windows.Visibility.Collapsed;
            SystemInfoPanel.Visibility = System.Windows.Visibility.Collapsed;
            VirtualizationPanel.Visibility = System.Windows.Visibility.Collapsed;
            DisablePanel.Visibility = System.Windows.Visibility.Collapsed;

            switch (panelName)
            {
                case "Menu":
                    MenuPanel.Visibility = System.Windows.Visibility.Visible;
                    break;
                case "SystemInfo":
                    SystemInfoPanel.Visibility = System.Windows.Visibility.Visible;
                    break;
                case "Virtualization":
                    VirtualizationPanel.Visibility = System.Windows.Visibility.Visible;
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

        private void VirtualizationCheckButton_Click(object sender, RoutedEventArgs e)
        {
            ShowPanel("Virtualization");
            LoadVirtualizationInfo();
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
            try
            {
                // UI 초기화
                StatusTextBlock.Text = "시스템 정보 수집 중...";
                
                // 시스템 정보 수집 버튼 비활성화
                SystemCheckButton.IsEnabled = false;

                // CancellationToken을 사용하여 작업 취소 가능하게 함
                using (var cts = new System.Threading.CancellationTokenSource())
                {
                    // 60초 타임아웃 설정
                    cts.CancelAfter(TimeSpan.FromSeconds(60));

                    // 백그라운드에서 시스템 정보 수집
                    await Task.Run(() => CollectSystemInfo(), cts.Token);
                }
            }
            catch (OperationCanceledException)
            {
                Dispatcher.Invoke(() =>
                {
                    var errorItems = new ObservableCollection<SystemInfoItem>
                    {
                        new SystemInfoItem 
                        { 
                            Category = "오류", 
                            Item = "시간 초과", 
                            Value = "시스템 정보 수집이 시간 초과로 중단되었습니다" 
                        }
                    };
                    SystemInfoDataGrid.ItemsSource = errorItems;
                    StatusTextBlock.Text = "시스템 정보 수집 시간 초과";
                });
            }
            catch (Exception ex)
            {
                Dispatcher.Invoke(() =>
                {
                    var errorItems = new ObservableCollection<SystemInfoItem>
                    {
                        new SystemInfoItem 
                        { 
                            Category = "오류", 
                            Item = "수집 실패", 
                            Value = $"오류: {ex.Message}" 
                        }
                    };
                    SystemInfoDataGrid.ItemsSource = errorItems;
                    StatusTextBlock.Text = "시스템 정보 수집 실패";
                });
            }
            finally
            {
                // UI 복원
                Dispatcher.Invoke(() =>
                {
                    SystemCheckButton.IsEnabled = true;
                    if (StatusTextBlock.Text.Contains("중") || StatusTextBlock.Text.Contains("수집"))
                    {
                        StatusTextBlock.Text = "시스템 정보 수집 완료";
                    }
                });
            }
        }

        private void CollectSystemInfo()
        {
            var systemInfoItems = new ObservableCollection<SystemInfoItem>();

            try
            {
                // 각 정보 수집을 개별적으로 try-catch로 보호
                CollectOSInfoToTable(systemInfoItems);
                CollectCPUInfoToTable(systemInfoItems);
                CollectMemoryInfoToTable(systemInfoItems);
                CollectDiskInfoToTable(systemInfoItems);
                CollectBootInfoToTable(systemInfoItems);

                // 새로 추가된 정보 수집 메서드들
                CollectMotherboardInfoToTable(systemInfoItems);
                CollectGraphicsCardInfoToTable(systemInfoItems);
                CollectPowerManagementInfoToTable(systemInfoItems);
                CollectEventLogInfoToTable(systemInfoItems);

                // UI 업데이트를 더 안전하게 처리
                SafeUpdateSystemInfoUI(systemInfoItems, "시스템 정보 수집 완료");
            }
            catch (Exception ex)
            {
                // 예외 발생 시에도 안전하게 UI 업데이트
                systemInfoItems.Add(new SystemInfoItem
                {
                    Category = "오류",
                    Item = "시스템 정보 수집 실패",
                    Value = $"오류: {ex.Message}"
                });
                SafeUpdateSystemInfoUI(systemInfoItems, "시스템 정보 수집 실패");
            }
        }

        private void SafeUpdateSystemInfoUI(ObservableCollection<SystemInfoItem> items, string status)
        {
            try
            {
                if (Dispatcher.CheckAccess())
                {
                    SystemInfoDataGrid.ItemsSource = items;
                    StatusTextBlock.Text = status;
                }
                else
                {
                    Dispatcher.BeginInvoke(() =>
                    {
                        try
                        {
                            SystemInfoDataGrid.ItemsSource = items;
                            StatusTextBlock.Text = status;
                        }
                        catch (Exception uiEx)
                        {
                            System.Diagnostics.Debug.WriteLine($"시스템 정보 UI 업데이트 실패: {uiEx.Message}");
                        }
                    });
                }
            }
            catch (Exception dispatcherEx)
            {
                System.Diagnostics.Debug.WriteLine($"시스템 정보 Dispatcher 오류: {dispatcherEx.Message}");
            }
        }

        private void SafeUpdateUI(string content, string status)
        {
            try
            {
                // UI 스레드에서 안전하게 업데이트
                if (Dispatcher.CheckAccess())
                {
                    // 이미 UI 스레드에 있는 경우
                    UpdateUIContent(content, status);
                }
                else
                {
                    // 다른 스레드에서 호출된 경우
                    Dispatcher.BeginInvoke(() =>
                    {
                        try
                        {
                            UpdateUIContent(content, status);
                        }
                        catch (Exception uiEx)
                        {
                            // UI 업데이트 실패 시 최후 수단
                            System.Diagnostics.Debug.WriteLine($"UI 업데이트 실패: {uiEx.Message}");
                            
                        }
                    });
                }
            }
            catch (Exception dispatcherEx)
            {
                // Dispatcher 자체에 문제가 있는 경우
                System.Diagnostics.Debug.WriteLine($"Dispatcher 오류: {dispatcherEx.Message}");
            }
        }

        private void UpdateUIContent(string content, string status)
        {
            try
            {
                // 이 메소드는 더 이상 사용되지 않음 - 테이블 형식으로 변경됨
                
                if (StatusTextBlock != null)
                {
                    try
                    {
                        StatusTextBlock.Text = status ?? "상태 불명";
                    }
                    catch (Exception statusEx)
                    {
                        System.Diagnostics.Debug.WriteLine($"StatusTextBlock 업데이트 오류: {statusEx.Message}");
                    }
                }
            }
            catch (Exception updateEx)
            {
                System.Diagnostics.Debug.WriteLine($"UI 컨트롤 업데이트 전체 오류: {updateEx.Message}");
                throw; // 상위에서 처리하도록 재발생
            }
        }

        private void SafeSetTextBoxContent(System.Windows.Controls.TextBox textBox, string content)
        {
            try
            {
                // 내용이 너무 긴 경우 잘라내기 (메모리 부족 방지)
                if (content.Length > 100000) // 100KB 제한
                {
                    content = content.Substring(0, 100000) + "\n\n... [내용이 너무 길어 일부가 잘렸습니다]";
                }

                // 문제가 될 수 있는 특수 문자 제거/교체
                content = content.Replace("\0", ""); // Null 문자 제거
                
                textBox.Text = content;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"안전한 TextBox 설정 실패: {ex.Message}");
                throw;
            }
        }

        private void CollectOSInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[운영체제 정보]");
                UpdateProgress("운영체제 정보 수집 중...");
                var osInfo = GetWindowsVersionInfo();
                result.AppendLine(osInfo);
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"운영체제 정보 수집 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CollectCPUInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[프로세서 정보]");
                UpdateProgress("프로세서 정보 수집 중...");
                var cpuInfo = GetCpuInfo();
                result.AppendLine(cpuInfo);
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"프로세서 정보 수집 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CollectMemoryInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[메모리 정보]");
                UpdateProgress("메모리 정보 수집 중...");
                var memoryInfo = GetMemoryInfo();
                result.AppendLine(memoryInfo);
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"메모리 정보 수집 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CollectDiskInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[디스크 정보]");
                UpdateProgress("디스크 정보 수집 중...");
                var diskInfo = GetDiskInfo();
                result.AppendLine(diskInfo);
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"디스크 정보 수집 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CollectVirtualizationInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[가상화 지원]");
                UpdateProgress("가상화 정보 수집 중...");
                var virtualizationInfo = GetVirtualizationInfo();
                result.AppendLine(virtualizationInfo);
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"가상화 정보 수집 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CollectBootInfo(StringBuilder result)
        {
            try
            {
                result.AppendLine("[부팅 정보]");
                UpdateProgress("부팅 정보 수집 중...");
                
                // 디버깅을 위한 상세 로그 추가
                try
                {
                    result.AppendLine("부팅 정보 수집 시작...");
                    UpdateProgress("부팅 시간 계산 중...");
                    
                    var bootTime = GetBootTime();
                    
                    result.AppendLine("부팅 정보 수집 완료");
                    result.AppendLine(bootTime);
                }
                catch (Exception bootEx)
                {
                    result.AppendLine($"GetBootTime 메소드 오류: {bootEx.Message}");
                    result.AppendLine($"스택 추적: {bootEx.StackTrace}");
                    
                    // 대안: 간단한 방법으로라도 정보 제공
                    try
                    {
                        result.AppendLine("\n=== 대안 정보 ===");
                        result.AppendLine($"현재 시간: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
                        result.AppendLine($"시스템 틱: {Environment.TickCount} ms");
                        result.AppendLine($"프로세서 수: {Environment.ProcessorCount}");
                    }
                    catch (Exception altEx)
                    {
                        result.AppendLine($"대안 정보 수집도 실패: {altEx.Message}");
                    }
                }
                
                result.AppendLine();
                UpdateProgress("부팅 정보 처리 완료");
            }
            catch (Exception ex)
            {
                result.AppendLine($"부팅 정보 수집 전체 실패: {ex.Message}");
                result.AppendLine($"전체 스택 추적: {ex.StackTrace}");
                result.AppendLine();
            }
        }

        private void UpdateProgress(string message)
        {
            try
            {
                // 안전한 UI 업데이트 메소드 사용
                SafeUpdateStatus(message);
            }
            catch (Exception ex)
            {
                // 진행 상황 업데이트 실패해도 로그만 남기고 계속 진행
                System.Diagnostics.Debug.WriteLine($"진행 상황 업데이트 실패: {ex.Message}");
            }
        }

        private void SafeUpdateStatus(string message)
        {
            try
            {
                if (Dispatcher.CheckAccess())
                {
                    // UI 스레드에서 직접 업데이트
                    if (StatusTextBlock != null)
                    {
                        StatusTextBlock.Text = message;
                    }
                }
                else
                {
                    // 다른 스레드에서 BeginInvoke 사용 (블로킹 방지)
                    Dispatcher.BeginInvoke(() =>
                    {
                        try
                        {
                            if (StatusTextBlock != null)
                            {
                                StatusTextBlock.Text = message;
                            }
                        }
                        catch (Exception statusEx)
                        {
                            System.Diagnostics.Debug.WriteLine($"상태 업데이트 오류: {statusEx.Message}");
                        }
                    });
                }
            }
            catch (Exception safeEx)
            {
                System.Diagnostics.Debug.WriteLine($"안전한 상태 업데이트 오류: {safeEx.Message}");
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

        private string GetSafeProperty(ManagementObject obj, string propertyName)
        {
            try
            {
                var value = obj[propertyName];
                return value?.ToString() ?? "알 수 없음";
            }
            catch
            {
                return "알 수 없음";
            }
        }

        private string GetCpuInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                using (var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_Processor"))
                {
                    // WMI 쿼리 타임아웃 설정 (10초)
                    searcher.Options.Timeout = TimeSpan.FromSeconds(10);
                    
                    using (var collection = searcher.Get())
                    {
                        foreach (ManagementObject obj in collection)
                        {
                            try
                            {
                                result.AppendLine($"모델: {GetSafeProperty(obj, "Name")}");
                                result.AppendLine($"제조사: {GetSafeProperty(obj, "Manufacturer")}");
                                result.AppendLine($"코어 수: {GetSafeProperty(obj, "NumberOfCores")}");
                                result.AppendLine($"논리 프로세서 수: {GetSafeProperty(obj, "NumberOfLogicalProcessors")}");
                                result.AppendLine($"최대 클럭: {GetSafeProperty(obj, "MaxClockSpeed")} MHz");
                                break;
                            }
                            catch (Exception ex)
                            {
                                result.AppendLine($"CPU 속성 읽기 오류: {ex.Message}");
                            }
                            finally
                            {
                                obj?.Dispose();
                            }
                        }
                    }
                }
                
                return result.Length > 0 ? result.ToString() : "CPU 정보를 가져올 수 없습니다.";
            }
            catch (Exception ex)
            {
                return $"CPU 정보 수집 오류: {ex.Message}";
            }
        }

        private string GetMemoryInfo()
        {
            try
            {
                var result = new StringBuilder();
                
                // 총 물리적 메모리 정보
                try
                {
                    using (var searcher = new ManagementObjectSearcher("SELECT TotalPhysicalMemory FROM Win32_ComputerSystem"))
                    {
                        searcher.Options.Timeout = TimeSpan.FromSeconds(10);
                        
                        using (var collection = searcher.Get())
                        {
                            foreach (ManagementObject obj in collection)
                            {
                                try
                                {
                                    var totalMemoryStr = GetSafeProperty(obj, "TotalPhysicalMemory");
                                    if (long.TryParse(totalMemoryStr, out long totalMemory))
                                    {
                                        var memoryGB = Math.Round(totalMemory / (1024.0 * 1024.0 * 1024.0), 2);
                                        result.AppendLine($"총 물리적 메모리: {memoryGB:F2} GB");
                                    }
                                    else
                                    {
                                        result.AppendLine("총 물리적 메모리: 확인 불가");
                                    }
                                    break;
                                }
                                finally
                                {
                                    obj?.Dispose();
                                }
                            }
                        }
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"총 메모리 정보 오류: {ex.Message}");
                }
                
                // 사용 가능한 메모리 정보
                try
                {
                    using (var perfSearcher = new ManagementObjectSearcher("SELECT AvailableBytes FROM Win32_PerfRawData_PerfOS_Memory"))
                    {
                        perfSearcher.Options.Timeout = TimeSpan.FromSeconds(5);
                        
                        using (var collection = perfSearcher.Get())
                        {
                            foreach (ManagementObject perfObj in collection)
                            {
                                try
                                {
                                    var availableBytesStr = GetSafeProperty(perfObj, "AvailableBytes");
                                    if (long.TryParse(availableBytesStr, out long availableBytes))
                                    {
                                        var availableGB = Math.Round(availableBytes / (1024.0 * 1024.0 * 1024.0), 2);
                                        result.AppendLine($"사용 가능한 메모리: {availableGB:F2} GB");
                                    }
                                    else
                                    {
                                        result.AppendLine("사용 가능한 메모리: 확인 불가");
                                    }
                                    break;
                                }
                                finally
                                {
                                    perfObj?.Dispose();
                                }
                            }
                        }
                    }
                }
                catch
                {
                    result.AppendLine("사용 가능한 메모리: 확인 불가");
                }
                
                return result.Length > 0 ? result.ToString() : "메모리 정보를 가져올 수 없습니다.";
            }
            catch (Exception ex)
            {
                return $"메모리 정보 수집 오류: {ex.Message}";
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
                var debugInfo = new StringBuilder();
                debugInfo.AppendLine($"드라이브 {driveLetter}: 타입 감지 시작");
                
                // 방법 1: MSFT_PhysicalDisk (Windows 8/Server 2012 이상에서 가장 정확)
                var ssdResult = CheckSSDWithMSFTPhysicalDisk(driveLetter, debugInfo);
                if (ssdResult != "알 수 없음")
                {
                    debugInfo.AppendLine($"MSFT_PhysicalDisk 결과: {ssdResult}");
                    return ssdResult + GetDebugSuffix(debugInfo);
                }

                // 방법 2: 레지스트리를 통한 확인 (권한 이슈가 적음)
                var registryResult = CheckSSDWithRegistry(driveLetter, debugInfo);
                if (registryResult != "알 수 없음")
                {
                    debugInfo.AppendLine($"레지스트리 결과: {registryResult}");
                    return registryResult + GetDebugSuffix(debugInfo);
                }

                // 방법 3: Win32_LogicalDisk와 Win32_DiskDrive 연결
                using (var partitionSearcher = new ManagementObjectSearcher($"ASSOCIATORS OF {{Win32_LogicalDisk.DeviceID='{driveLetter}:'}} WHERE AssocClass = Win32_LogicalDiskToPartition"))
                {
                    foreach (ManagementObject partition in partitionSearcher.Get())
                    {
                        using (var diskSearcher = new ManagementObjectSearcher($"ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='{partition["DeviceID"]}'}} WHERE AssocClass = Win32_DiskDriveToDiskPartition"))
                        {
                            foreach (ManagementObject disk in diskSearcher.Get())
                            {
                                var diskIndex = disk["Index"]?.ToString();
                                debugInfo.AppendLine($"물리 디스크 인덱스: {diskIndex}");
                                
                                // Win32_DiskDrive 정보로 SSD 확인
                                var result = CheckSSDWithWin32DiskDrive(diskIndex, debugInfo);
                                if (result != "알 수 없음")
                                {
                                    debugInfo.AppendLine($"Win32_DiskDrive 결과: {result}");
                                    return result + GetDebugSuffix(debugInfo);
                                }
                                
                                // 방법 4: 성능 카운터를 통한 확인
                                var perfResult = CheckSSDWithPerformanceCounters(diskIndex, debugInfo);
                                if (perfResult != "알 수 없음")
                                {
                                    debugInfo.AppendLine($"성능 카운터 결과: {perfResult}");
                                    return perfResult + GetDebugSuffix(debugInfo);
                                }
                            }
                        }
                    }
                }
                
                debugInfo.AppendLine("모든 방법 실패");
                return "알 수 없음" + GetDebugSuffix(debugInfo);
            }
            catch (Exception ex)
            {
                return $"오류: {ex.Message}";
            }
        }

        private string GetDebugSuffix(StringBuilder debugInfo)
        {
            return "";
        }

        private string CheckSSDWithMSFTPhysicalDisk(string driveLetter, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine("MSFT_PhysicalDisk 시도");
                
                // 1단계: 드라이브 문자에서 물리 디스크 번호 찾기
                int diskNumber = GetPhysicalDiskNumber(driveLetter, debugInfo);
                if (diskNumber == -1)
                {
                    debugInfo.AppendLine("물리 디스크 번호를 찾을 수 없음, 모든 디스크 검색으로 폴백");
                    return CheckAllMSFTPhysicalDisks(driveLetter, debugInfo);
                }
                
                debugInfo.AppendLine($"드라이브 {driveLetter}는 물리 디스크 {diskNumber}번에 위치");
                
                // 2단계: 해당 물리 디스크의 MSFT_PhysicalDisk 정보 조회
                try
                {
                    using (var searcher = new ManagementObjectSearcher(@"root\Microsoft\Windows\Storage", 
                        $"SELECT * FROM MSFT_PhysicalDisk WHERE DeviceId = {diskNumber}"))
                    {
                        foreach (ManagementObject disk in searcher.Get())
                        {
                            var result = GetDiskTypeFromMSFT(disk, debugInfo, diskNumber);
                            if (result != "알 수 없음")
                                return result;
                        }
                    }
                }
                catch (Exception ex)
                {
                    debugInfo.AppendLine($"특정 디스크 MSFT 쿼리 오류: {ex.Message}");
                }
                
                // 3단계: 특정 디스크 쿼리가 실패하면 모든 디스크 검색
                debugInfo.AppendLine("특정 디스크 쿼리 실패, 모든 디스크 검색으로 폴백");
                return CheckAllMSFTPhysicalDisks(driveLetter, debugInfo);
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"MSFT_PhysicalDisk 전체 오류: {ex.Message}");
            }
            
            return "알 수 없음";
        }

        private string CheckAllMSFTPhysicalDisks(string driveLetter, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine("모든 MSFT_PhysicalDisk 검색 중");
                using (var searcher = new ManagementObjectSearcher(@"root\Microsoft\Windows\Storage", "SELECT * FROM MSFT_PhysicalDisk"))
                {
                    foreach (ManagementObject disk in searcher.Get())
                    {
                        var result = GetDiskTypeFromMSFT(disk, debugInfo, null);
                        if (result != "알 수 없음")
                        {
                            debugInfo.AppendLine("첫 번째로 식별된 디스크 타입 반환");
                            return result;
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"모든 MSFT 디스크 검색 오류: {ex.Message}");
            }
            
            return "알 수 없음";
        }

        private string GetDiskTypeFromMSFT(ManagementObject disk, StringBuilder debugInfo, int? diskNumber)
        {
            try
            {
                var mediaType = disk["MediaType"];
                var busType = disk["BusType"];
                var friendlyName = disk["FriendlyName"]?.ToString() ?? "";
                var model = disk["Model"]?.ToString() ?? "";
                
                string diskInfo = diskNumber.HasValue ? $"디스크 {diskNumber}: " : "";
                debugInfo.AppendLine($"MSFT {diskInfo}{friendlyName}, 모델: {model}");
                
                if (mediaType != null)
                {
                    var mediaTypeValue = Convert.ToUInt16(mediaType);
                    debugInfo.AppendLine($"MediaType: {mediaTypeValue}");
                    
                    // MediaType: 3 = HDD, 4 = SSD, 5 = SCM
                    switch (mediaTypeValue)
                    {
                        case 4:
                            // BusType 확인해서 NVMe인지 구분
                            if (busType != null && Convert.ToUInt16(busType) == 17) // NVMe
                                return "SSD (NVMe)";
                            return "SSD";
                        case 3:
                            return "HDD";
                        case 5:
                            return "SCM (Storage Class Memory)";
                    }
                }
                
                // MediaType가 없는 경우 모델명으로 판단 (확장된 패턴)
                var ssdIndicators = new[] { 
                    "SSD", "NVMe", "Solid State", "SHGS31", "SHGS", "GS-2", "Flash" 
                };
                
                foreach (var indicator in ssdIndicators)
                {
                    if (friendlyName.Contains(indicator, StringComparison.OrdinalIgnoreCase) ||
                        model.Contains(indicator, StringComparison.OrdinalIgnoreCase))
                    {
                        if (friendlyName.Contains("NVMe", StringComparison.OrdinalIgnoreCase) ||
                            model.Contains("NVMe", StringComparison.OrdinalIgnoreCase))
                            return "SSD (NVMe)";
                        return "SSD";
                    }
                }
                
                return "알 수 없음";
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"MSFT 디스크 정보 파싱 오류: {ex.Message}");
                return "알 수 없음";
            }
        }

        private int GetPhysicalDiskNumber(string driveLetter, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine($"드라이브 {driveLetter}의 물리 디스크 번호 찾는 중");
                
                // 방법 1: Win32_LogicalDiskToPartition 사용
                string logicalDiskQuery = $"ASSOCIATORS OF {{Win32_LogicalDisk.DeviceID='{driveLetter}'}} WHERE AssocClass=Win32_LogicalDiskToPartition";
                using (var searcher = new ManagementObjectSearcher(logicalDiskQuery))
                {
                    foreach (ManagementObject partition in searcher.Get())
                    {
                        debugInfo.AppendLine($"파티션 찾음: {partition["DeviceID"]}");
                        
                        // 파티션에서 물리 디스크 찾기
                        string partitionQuery = $"ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='{partition["DeviceID"]}'}} WHERE AssocClass=Win32_DiskDriveToDiskPartition";
                        using (var diskSearcher = new ManagementObjectSearcher(partitionQuery))
                        {
                            foreach (ManagementObject disk in diskSearcher.Get())
                            {
                                var index = disk["Index"];
                                if (index != null)
                                {
                                    int diskNumber = Convert.ToInt32(index);
                                    debugInfo.AppendLine($"물리 디스크 번호: {diskNumber}");
                                    return diskNumber;
                                }
                            }
                        }
                    }
                }
                
                // 방법 2: Win32_DiskPartition에서 직접 찾기
                debugInfo.AppendLine("대체 방법: Win32_DiskPartition에서 직접 검색");
                using (var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_DiskPartition"))
                {
                    foreach (ManagementObject partition in searcher.Get())
                    {
                        string partitionQuery2 = $"ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='{partition["DeviceID"]}'}} WHERE AssocClass=Win32_LogicalDiskToPartition";
                        using (var logicalSearcher = new ManagementObjectSearcher(partitionQuery2))
                        {
                            foreach (ManagementObject logical in logicalSearcher.Get())
                            {
                                if (logical["DeviceID"]?.ToString() == driveLetter)
                                {
                                    var diskIndex = partition["DiskIndex"];
                                    if (diskIndex != null)
                                    {
                                        int diskNumber = Convert.ToInt32(diskIndex);
                                        debugInfo.AppendLine($"대체 방법으로 찾은 물리 디스크 번호: {diskNumber}");
                                        return diskNumber;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"물리 디스크 번호 찾기 오류: {ex.Message}");
            }
            
            return -1; // 찾지 못함
        }

        private string CheckSSDWithRegistry(string driveLetter, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine("레지스트리 방법 시도");
                
                // 레지스트리에서 디스크 정보 확인
                var diskKeys = Registry.LocalMachine.OpenSubKey(@"SYSTEM\CurrentControlSet\Services\Disk\Enum");
                if (diskKeys != null)
                {
                    var count = diskKeys.GetValue("Count");
                    if (count != null)
                    {
                        for (int i = 0; i < Convert.ToInt32(count); i++)
                        {
                            var deviceId = diskKeys.GetValue(i.ToString())?.ToString();
                            if (!string.IsNullOrEmpty(deviceId))
                            {
                                debugInfo.AppendLine($"디바이스 ID: {deviceId}");
                                
                                // 디바이스 ID에서 SSD 키워드 확인
                                if (deviceId.Contains("SSD", StringComparison.OrdinalIgnoreCase) ||
                                    deviceId.Contains("NVMe", StringComparison.OrdinalIgnoreCase))
                                {
                                    return deviceId.Contains("NVMe", StringComparison.OrdinalIgnoreCase) ? "SSD (NVMe)" : "SSD";
                                }
                                
                                // 제조사별 SSD 식별자 확인
                                var ssdIds = new[] { "SAMSUNG_SSD", "INTEL_SSD", "CRUCIAL_", "WD_", "KINGSTON_", "CORSAIR_" };
                                foreach (var id in ssdIds)
                                {
                                    if (deviceId.Contains(id, StringComparison.OrdinalIgnoreCase))
                                        return "SSD";
                                }
                            }
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"레지스트리 오류: {ex.Message}");
            }
            
            return "알 수 없음";
        }

        private string CheckSSDWithPerformanceCounters(string diskIndex, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine("성능 카운터 방법 시도");
                
                // Win32_PerfRawData_PerfDisk_PhysicalDisk를 통한 확인
                using (var searcher = new ManagementObjectSearcher($"SELECT * FROM Win32_PerfRawData_PerfDisk_PhysicalDisk WHERE Name LIKE '%{diskIndex}%'"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var name = obj["Name"]?.ToString() ?? "";
                        debugInfo.AppendLine($"성능 카운터 디스크: {name}");
                        
                        // 평균 응답 시간이 매우 빠른 경우 SSD일 가능성
                        var avgDiskSecPerRead = obj["AvgDiskSecPerRead"];
                        var avgDiskSecPerWrite = obj["AvgDiskSecPerWrite"];
                        
                        if (avgDiskSecPerRead != null && avgDiskSecPerWrite != null)
                        {
                            var readTime = Convert.ToDouble(avgDiskSecPerRead);
                            var writeTime = Convert.ToDouble(avgDiskSecPerWrite);
                            
                            // SSD는 일반적으로 매우 빠른 응답시간을 가짐 (임계값 조정 가능)
                            if (readTime < 0.001 && writeTime < 0.001) // 1ms 미만
                            {
                                debugInfo.AppendLine($"빠른 응답시간 감지: R={readTime}, W={writeTime}");
                                return "SSD (성능 기반 추정)";
                            }
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"성능 카운터 오류: {ex.Message}");
            }
            
            return "알 수 없음";
        }

        private string CheckSSDWithWin32DiskDrive(string diskIndex, StringBuilder debugInfo)
        {
            try
            {
                debugInfo.AppendLine("Win32_DiskDrive 방법 시도");
                using (var searcher = new ManagementObjectSearcher($"SELECT * FROM Win32_DiskDrive WHERE Index={diskIndex}"))
                {
                    foreach (ManagementObject disk in searcher.Get())
                    {
                        var mediaType = disk["MediaType"]?.ToString() ?? "";
                        var model = disk["Model"]?.ToString() ?? "";
                        var interfaceType = disk["InterfaceType"]?.ToString() ?? "";
                        var pnpDeviceId = disk["PNPDeviceID"]?.ToString() ?? "";
                        var serialNumber = disk["SerialNumber"]?.ToString() ?? "";
                        
                        debugInfo.AppendLine($"모델: {model}, 인터페이스: {interfaceType}, PNP: {pnpDeviceId}");
                        
                        // 1. MediaType 직접 확인
                        if (mediaType.Contains("SSD", StringComparison.OrdinalIgnoreCase))
                            return "SSD";
                            
                        // 2. PNPDeviceID에서 SSD 확인 (가장 신뢰할 수 있는 방법 중 하나)
                        if (pnpDeviceId.Contains("SSD", StringComparison.OrdinalIgnoreCase) ||
                            pnpDeviceId.Contains("NVMe", StringComparison.OrdinalIgnoreCase))
                        {
                            return pnpDeviceId.Contains("NVMe", StringComparison.OrdinalIgnoreCase) ? "SSD (NVMe)" : "SSD";
                        }
                            
                        // 3. 모델명에서 SSD 키워드 확인 (확장된 키워드 목록)
                        var ssdKeywords = new[] { 
                            "SSD", "Solid State", "NVMe", "M.2", "mSATA", "PCIe",
                            "Flash", "NAND", "eUFS", "SATA SSD", "NGFF" // 추가 SSD 관련 키워드
                        };
                        foreach (var keyword in ssdKeywords)
                        {
                            if (model.Contains(keyword, StringComparison.OrdinalIgnoreCase))
                            {
                                if (keyword == "NVMe" || model.Contains("NVMe", StringComparison.OrdinalIgnoreCase))
                                    return "SSD (NVMe)";
                                return "SSD";
                            }
                        }
                        
                        // 4. 제조사별 SSD 모델 패턴 확인 (확장된 패턴)
                        var ssdPatterns = new[]
                        {
                            // 주요 제조사
                            "Samsung.*SSD", "Samsung.*NVMe", "Samsung.*EVO", "Samsung.*PRO",
                            "Intel.*SSD", "Intel.*NVMe", "Intel.*Optane",
                            "Crucial.*SSD", "Crucial.*NVMe", "Crucial.*MX", "Crucial.*BX",
                            "WD.*SSD", "WD.*NVMe", "Western Digital.*SSD", "WD_BLACK.*SSD",
                            "Kingston.*SSD", "Kingston.*NVMe", "Kingston.*A.*SSD",
                            "Corsair.*SSD", "Corsair.*NVMe",
                            "ADATA.*SSD", "ADATA.*NVMe", "XPG.*SSD",
                            "Transcend.*SSD", "Transcend.*NVMe",
                            "Micron.*SSD", "Micron.*NVMe",
                            "SK.*hynix.*SSD", "SK.*Hynix.*NVMe",
                            "LITEON.*SSD", "Plextor.*SSD",
                            // 기타 제조사
                            "Seagate.*SSD", "Seagate.*NVMe", "FireCuda.*SSD",
                            "Toshiba.*SSD", "Kioxia.*SSD",
                            "Sandisk.*SSD", "SanDisk.*NVMe",
                            "Mushkin.*SSD", "Patriot.*SSD", "PNY.*SSD",
                            "Team.*SSD", "Gigabyte.*SSD", "MSI.*SSD",
                            // 특정 모델명 패턴
                            ".*980.*PRO", ".*980.*EVO", ".*970.*EVO", ".*960.*EVO",
                            ".*P31.*Gold", ".*P41.*Plus", ".*Black.*SN.*",
                            // 일반적인 SSD 모델명 패턴 (숫자-문자-숫자 조합)
                            "SHGS.*-.*GS", "SH.*GS.*-.*", "SHGS31-500GS-2", "[A-Z]{2,5}[0-9]{2,3}-[0-9]{3}[A-Z]{1,3}-[0-9]",
                            // 더 많은 제조사별 패턴
                            "GALAX.*SSD", "ZOTAC.*SSD", "HIKVISION.*SSD", "AORUS.*SSD"
                        };
                        
                        foreach (var pattern in ssdPatterns)
                        {
                            if (System.Text.RegularExpressions.Regex.IsMatch(model, pattern, System.Text.RegularExpressions.RegexOptions.IgnoreCase))
                            {
                                debugInfo.AppendLine($"패턴 매치: {pattern}");
                                return pattern.Contains("NVMe") || pattern.Contains("980") || pattern.Contains("970") || pattern.Contains("SN") ? "SSD (NVMe)" : "SSD";
                            }
                        }
                        
                        // 5. 인터페이스 타입별 추가 확인
                        if (interfaceType.Contains("SCSI", StringComparison.OrdinalIgnoreCase))
                        {
                            // NVMe는 대부분 SCSI 인터페이스로 보고됨
                            if (model.Contains("NVMe", StringComparison.OrdinalIgnoreCase) ||
                                model.Contains("PCIe", StringComparison.OrdinalIgnoreCase))
                                return "SSD (NVMe)";
                                
                            // SCSI 인터페이스인데 모델명에 특정 키워드가 있으면 SSD 가능성
                            var scsiSsdIndicators = new[] { "980", "970", "960", "PM9", "SM9", "BG4", "BG3" };
                            foreach (var indicator in scsiSsdIndicators)
                            {
                                if (model.Contains(indicator, StringComparison.OrdinalIgnoreCase))
                                    return "SSD (NVMe)";
                            }
                        }
                        
                        // 6. 시리얼 번호나 기타 속성에서 SSD 힌트 찾기
                        if (!string.IsNullOrEmpty(serialNumber))
                        {
                            if (serialNumber.Contains("SSD", StringComparison.OrdinalIgnoreCase))
                                return "SSD";
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                debugInfo.AppendLine($"Win32_DiskDrive 오류: {ex.Message}");
            }
            
            return "알 수 없음";
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
                var methods = new List<string>();
                
                // 가장 안전한 방법부터 시도 (Environment.TickCount 먼저)
                try
                {
                    result.AppendLine("=== 안전한 방법 우선 시도 ===");
                    var envResult = GetBootTimeFromEnvironment(result);
                    if (envResult)
                    {
                        methods.Add("환경 변수 (안전 우선)");
                        return result.ToString() + $"\n[사용된 방법: {string.Join(", ", methods)}]";
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"환경 변수 방법 실행 중 오류: {ex.Message}");
                }
                
                // 방법 1: Win32_OperatingSystem을 통한 부팅 시간 확인 (기본)
                try
                {
                    result.AppendLine("=== WMI 방법 시도 ===");
                    var bootTimeResult = GetBootTimeFromWMI(result);
                    if (bootTimeResult)
                    {
                        methods.Add("WMI");
                        return result.ToString() + $"\n[사용된 방법: {string.Join(", ", methods)}]";
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"WMI 방법 실행 중 오류: {ex.Message}");
                }
                
                // 방법 2: 레지스트리를 통한 부팅 시간 확인 (성능 카운터 대신 더 안전한 방법)
                try
                {
                    result.AppendLine("=== 레지스트리 방법 시도 ===");
                    var registryResult = GetBootTimeFromRegistry(result);
                    if (registryResult)
                    {
                        methods.Add("레지스트리");
                        return result.ToString() + $"\n[사용된 방법: {string.Join(", ", methods)}]";
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"레지스트리 방법 실행 중 오류: {ex.Message}");
                }
                
                // 방법 3: 이벤트 로그를 통한 부팅 시간 확인
                try
                {
                    result.AppendLine("=== 이벤트 로그 방법 시도 ===");
                    var eventResult = GetBootTimeFromEventLog(result);
                    if (eventResult)
                    {
                        methods.Add("이벤트 로그");
                        return result.ToString() + $"\n[사용된 방법: {string.Join(", ", methods)}]";
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"이벤트 로그 방법 실행 중 오류: {ex.Message}");
                }
                
                // 방법 4: 성능 카운터 (위험하지만 시도)
                try
                {
                    result.AppendLine("=== 성능 카운터 방법 시도 (주의) ===");
                    var perfResult = GetBootTimeFromPerformanceCounter(result);
                    if (perfResult)
                    {
                        methods.Add("성능 카운터");
                        return result.ToString() + $"\n[사용된 방법: {string.Join(", ", methods)}]";
                    }
                }
                catch (Exception ex)
                {
                    result.AppendLine($"성능 카운터 방법 실행 중 오류: {ex.Message}");
                    result.AppendLine("성능 카운터 방법을 건너뛰었습니다.");
                }
                
                // 모든 방법 실패 시 기본 정보라도 제공
                result.AppendLine("\n=== 부팅 시간 정보 수집 실패 ===");
                result.AppendLine("모든 방법이 실패했지만 다음 정보를 확인할 수 있습니다:");
                result.AppendLine($"현재 시간: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
                result.AppendLine($"프로세서 수: {Environment.ProcessorCount}");
                result.AppendLine($"작업 디렉터리: {Environment.CurrentDirectory}");
                result.AppendLine("\n관리자 권한으로 실행하거나 잠시 후 다시 시도해 보세요.");
                
                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"부팅 시간 정보 수집 심각한 오류: {ex.Message}\n현재 시간: {DateTime.Now:yyyy-MM-dd HH:mm:ss}";
            }
        }

        private bool GetBootTimeFromRegistry(StringBuilder result)
        {
            try
            {
                // 레지스트리에서 시스템 부팅 시간 관련 정보 확인
                using (var key = Registry.LocalMachine.OpenSubKey(@"SYSTEM\CurrentControlSet\Control\Session Manager"))
                {
                    if (key != null)
                    {
                        var bootExecute = key.GetValue("BootExecute");
                        if (bootExecute != null)
                        {
                            result.AppendLine("레지스트리에서 부팅 관련 정보를 찾았지만 정확한 시간은 확인할 수 없습니다.");
                            result.AppendLine($"시스템 부팅 프로세스: {bootExecute}");
                            
                            // 대략적인 가동 시간 추정 (프로세스 시작 시간 기반)
                            var currentProcess = Process.GetCurrentProcess();
                            var processStartTime = currentProcess.StartTime;
                            var approximateUptime = DateTime.Now - processStartTime;
                            
                            result.AppendLine($"현재 프로세스 시작 시간: {processStartTime:yyyy-MM-dd HH:mm:ss}");
                            result.AppendLine($"대략적인 가동 시간 (추정): 최소 {approximateUptime.Days}일 {approximateUptime.Hours}시간 {approximateUptime.Minutes}분");
                            result.AppendLine("(주의: 이는 정확한 부팅 시간이 아닌 추정값입니다)");
                            
                            return true;
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                result.AppendLine($"레지스트리 부팅 시간 오류: {ex.Message}");
            }
            
            return false;
        }

        private bool GetBootTimeFromEnvironment(StringBuilder result)
        {
            try
            {
                // 환경 변수와 시스템 정보를 통한 간접적 정보 수집
                var tickCount = Environment.TickCount;
                var uptimeMilliseconds = tickCount & 0x7FFFFFFF; // 음수 방지
                var uptime = TimeSpan.FromMilliseconds(uptimeMilliseconds);
                var estimatedBootTime = DateTime.Now - uptime;
                
                result.AppendLine($"추정 부팅 시간: {estimatedBootTime:yyyy-MM-dd HH:mm:ss} (환경 변수 기반)");
                result.AppendLine($"추정 가동 시간: {uptime.Days}일 {uptime.Hours}시간 {uptime.Minutes}분");
                result.AppendLine($"시스템 틱 카운트: {tickCount:N0} ms");
                result.AppendLine("(주의: Environment.TickCount는 약 24.9일마다 재설정되므로 참고용입니다)");
                
                // 추가 시스템 정보
                result.AppendLine($"운영체제 버전: {Environment.OSVersion}");
                result.AppendLine($"64비트 운영체제: {Environment.Is64BitOperatingSystem}");
                result.AppendLine($"시스템 디렉터리: {Environment.SystemDirectory}");
                
                return true;
            }
            catch (Exception ex)
            {
                result.AppendLine($"환경 변수 부팅 시간 오류: {ex.Message}");
            }
            
            return false;
        }

        private bool GetBootTimeFromWMI(StringBuilder result)
        {
            try
            {
                using (var searcher = new ManagementObjectSearcher("SELECT LastBootUpTime FROM Win32_OperatingSystem"))
                {
                    // WMI 쿼리 타임아웃 설정
                    searcher.Options.Timeout = TimeSpan.FromSeconds(10);
                    
                    using (var collection = searcher.Get())
                    {
                        foreach (ManagementObject obj in collection)
                        {
                            try
                            {
                                var bootTimeStr = GetSafeProperty(obj, "LastBootUpTime");
                                if (string.IsNullOrEmpty(bootTimeStr) || bootTimeStr == "알 수 없음")
                                {
                                    result.AppendLine("WMI를 통한 부팅 시간 정보를 가져올 수 없습니다.");
                                    return false;
                                }

                                // ManagementDateTimeConverter 사용 시 예외 처리
                                DateTime bootTime;
                                try
                                {
                                    bootTime = ManagementDateTimeConverter.ToDateTime(bootTimeStr);
                                }
                                catch (Exception ex)
                                {
                                    result.AppendLine($"부팅 시간 변환 오류: {ex.Message}");
                                    return false;
                                }

                                var upTime = DateTime.Now - bootTime;
                                
                                // 가동 시간이 음수이거나 비정상적으로 큰 경우 체크
                                if (upTime.TotalDays < 0 || upTime.TotalDays > 365)
                                {
                                    result.AppendLine("비정상적인 부팅 시간이 감지되었습니다.");
                                    return false;
                                }
                                
                                result.AppendLine($"마지막 부팅 시간: {bootTime:yyyy-MM-dd HH:mm:ss}");
                                result.AppendLine($"시스템 가동 시간: {upTime.Days}일 {upTime.Hours}시간 {upTime.Minutes}분");
                                return true;
                            }
                            catch (Exception ex)
                            {
                                result.AppendLine($"부팅 시간 속성 처리 오류: {ex.Message}");
                                return false;
                            }
                            finally
                            {
                                obj?.Dispose();
                            }
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                result.AppendLine($"WMI 부팅 시간 쿼리 오류: {ex.Message}");
            }
            
            return false;
        }

        private bool GetBootTimeFromPerformanceCounter(StringBuilder result)
        {
            System.Diagnostics.PerformanceCounter upTimeCounter = null;
            try
            {
                result.AppendLine("성능 카운터 방법 시작");
                
                // System Up Time 성능 카운터 사용
                upTimeCounter = new System.Diagnostics.PerformanceCounter("System", "System Up Time");
                result.AppendLine("성능 카운터 생성 완료");
                
                // 첫 번째 호출 (항상 0을 반환하므로 무시)
                try
                {
                    var firstValue = upTimeCounter.NextValue();
                    result.AppendLine($"첫 번째 값: {firstValue}");
                }
                catch (Exception firstEx)
                {
                    result.AppendLine($"첫 번째 NextValue 오류: {firstEx.Message}");
                    return false;
                }
                
                // 짧은 대기 시간으로 변경 (100ms -> 50ms)
                System.Threading.Thread.Sleep(50);
                result.AppendLine("대기 완료");
                
                // 두 번째 호출로 실제 값 획득
                float uptimeSeconds;
                try
                {
                    uptimeSeconds = upTimeCounter.NextValue();
                    result.AppendLine($"두 번째 값: {uptimeSeconds}");
                }
                catch (Exception secondEx)
                {
                    result.AppendLine($"두 번째 NextValue 오류: {secondEx.Message}");
                    return false;
                }
                
                // 값 유효성 검사
                if (uptimeSeconds < 0 || uptimeSeconds > 31536000) // 1년 초과시 비정상
                {
                    result.AppendLine($"비정상적인 가동시간 값: {uptimeSeconds} 초");
                    return false;
                }
                
                var uptime = TimeSpan.FromSeconds(uptimeSeconds);
                var bootTime = DateTime.Now - uptime;
                
                result.AppendLine($"마지막 부팅 시간: {bootTime:yyyy-MM-dd HH:mm:ss} (성능 카운터 기반)");
                result.AppendLine($"시스템 가동 시간: {uptime.Days}일 {uptime.Hours}시간 {uptime.Minutes}분");
                result.AppendLine($"가동시간(초): {uptimeSeconds:F2}");
                return true;
            }
            catch (UnauthorizedAccessException)
            {
                result.AppendLine("성능 카운터 접근 권한 부족");
            }
            catch (System.ComponentModel.Win32Exception winEx)
            {
                result.AppendLine($"성능 카운터 Win32 오류: {winEx.Message} (코드: {winEx.NativeErrorCode})");
            }
            catch (InvalidOperationException invEx)
            {
                result.AppendLine($"성능 카운터 작업 오류: {invEx.Message}");
            }
            catch (Exception ex)
            {
                result.AppendLine($"성능 카운터 일반 오류: {ex.Message}");
                result.AppendLine($"오류 타입: {ex.GetType().Name}");
            }
            finally
            {
                try
                {
                    upTimeCounter?.Dispose();
                    result.AppendLine("성능 카운터 해제 완료");
                }
                catch (Exception disposeEx)
                {
                    result.AppendLine($"성능 카운터 해제 오류: {disposeEx.Message}");
                }
            }
            
            return false;
        }

        private bool GetBootTimeFromEventLog(StringBuilder result)
        {
            try
            {
                // Windows 이벤트 로그에서 시스템 부팅 이벤트 확인
                using (var eventLog = new System.Diagnostics.EventLog("System"))
                {
                    // 이벤트 로그 액세스 권한 확인
                    if (eventLog.Entries.Count == 0)
                    {
                        result.AppendLine("이벤트 로그에 접근할 수 없습니다.");
                        return false;
                    }

                    // 가장 최근 100개 이벤트 중에서 부팅 이벤트 찾기
                    System.Diagnostics.EventLogEntry bootEntry = null;
                    
                    try
                    {
                        // EventID 대신 InstanceId 사용 (deprecated 경고 해결)
                        var recentEntries = eventLog.Entries.Cast<System.Diagnostics.EventLogEntry>()
                                                   .OrderByDescending(e => e.TimeGenerated)
                                                   .Take(100)
                                                   .Where(e => e.InstanceId == 6005 || e.InstanceId == 6009 || e.InstanceId == 6013) // 시스템 시작 이벤트
                                                   .ToList();

                        bootEntry = recentEntries.FirstOrDefault();
                    }
                    catch (Exception ex)
                    {
                        result.AppendLine($"이벤트 로그 쿼리 오류: {ex.Message}");
                        return false;
                    }

                    if (bootEntry != null)
                    {
                        var bootTime = bootEntry.TimeGenerated;
                        var upTime = DateTime.Now - bootTime;
                        
                        // 가동 시간 유효성 검사
                        if (upTime.TotalDays < 0 || upTime.TotalDays > 365)
                        {
                            result.AppendLine("이벤트 로그에서 비정상적인 부팅 시간이 감지되었습니다.");
                            return false;
                        }
                        
                        result.AppendLine($"마지막 부팅 시간: {bootTime:yyyy-MM-dd HH:mm:ss} (이벤트 로그 기반)");
                        result.AppendLine($"시스템 가동 시간: {upTime.Days}일 {upTime.Hours}시간 {upTime.Minutes}분");
                        result.AppendLine($"이벤트 ID: {bootEntry.InstanceId}");
                        return true;
                    }
                    else
                    {
                        result.AppendLine("이벤트 로그에서 부팅 이벤트를 찾을 수 없습니다.");
                        return false;
                    }
                }
            }
            catch (UnauthorizedAccessException)
            {
                result.AppendLine("이벤트 로그 접근 권한이 부족합니다.");
            }
            catch (System.Security.SecurityException)
            {
                result.AppendLine("이벤트 로그 접근 보안 오류가 발생했습니다.");
            }
            catch (Exception ex)
            {
                result.AppendLine($"이벤트 로그 부팅 시간 오류: {ex.Message}");
            }
            
            return false;
        }

        private async void LoadVirtualizationInfo()
        {
            try
            {
                // UI 초기화
                StatusTextBlock.Text = "가상화 설정 수집 중...";
                
                // 가상화 점검 버튼 비활성화
                VirtualizationCheckButton.IsEnabled = false;

                // CancellationToken을 사용하여 작업 취소 가능하게 함
                using (var cts = new System.Threading.CancellationTokenSource())
                {
                    // 30초 타임아웃 설정
                    cts.CancelAfter(TimeSpan.FromSeconds(30));

                    // 백그라운드에서 가상화 정보 수집
                    await Task.Run(() => CollectVirtualizationSettings(), cts.Token);
                }
            }
            catch (OperationCanceledException)
            {
                Dispatcher.Invoke(() =>
                {
                    var errorItems = new ObservableCollection<VirtualizationInfoItem>
                    {
                        new VirtualizationInfoItem 
                        { 
                            Category = "오류", 
                            Status = "시간 초과",
                            Details = "가상화 설정 수집이 시간 초과로 중단되었습니다",
                            Recommendation = "프로그램을 다시 시도해 보세요"
                        }
                    };
                    VirtualizationDataGrid.ItemsSource = errorItems;
                    StatusTextBlock.Text = "가상화 설정 수집 시간 초과";
                });
            }
            catch (Exception ex)
            {
                Dispatcher.Invoke(() =>
                {
                    var errorItems = new ObservableCollection<VirtualizationInfoItem>
                    {
                        new VirtualizationInfoItem 
                        { 
                            Category = "오류", 
                            Status = "수집 실패",
                            Details = $"오류: {ex.Message}",
                            Recommendation = "관리자 권한으로 실행하세요"
                        }
                    };
                    VirtualizationDataGrid.ItemsSource = errorItems;
                    StatusTextBlock.Text = "가상화 설정 수집 실패";
                });
            }
            finally
            {
                // UI 복원
                Dispatcher.Invoke(() =>
                {
                    VirtualizationCheckButton.IsEnabled = true;
                    if (StatusTextBlock.Text.Contains("중") || StatusTextBlock.Text.Contains("수집"))
                    {
                        StatusTextBlock.Text = "가상화 설정 수집 완료";
                    }
                });
            }
        }

        private void CollectVirtualizationSettings()
        {
            var virtualizationItems = new ObservableCollection<VirtualizationInfoItem>();

            try
            {
                // 1. 하드웨어 가상화 지원 확인
                CheckHardwareVirtualizationToTable(virtualizationItems);
                
                // 2. WSL 설치 상태 확인
                CheckWSLInstallationToTable(virtualizationItems);
                
                // 3. Hyper-V 설치 상태 확인
                CheckHyperVInstallationDetailedToTable(virtualizationItems);
                
                // 4. bcdedit hypervisorlaunchtype 확인
                CheckHypervisorLaunchTypeToTable(virtualizationItems);
                
                // 5. VBS 상태 및 레지스트리 정보 확인
                CheckVBSStatusDetailedToTable(virtualizationItems);

                // UI 업데이트를 더 안전하게 처리
                SafeUpdateVirtualizationTableUI(virtualizationItems, "가상화 설정 수집 완료");
            }
            catch (Exception ex)
            {
                // 예외 발생 시에도 안전하게 UI 업데이트
                virtualizationItems.Add(new VirtualizationInfoItem
                {
                    Category = "오류",
                    Status = "심각한 오류",
                    Details = ex.Message,
                    Recommendation = "프로그램을 재시작하세요"
                });
                SafeUpdateVirtualizationTableUI(virtualizationItems, "가상화 설정 수집 실패");
            }
        }

        private void SafeUpdateVirtualizationUI(string content, string status)
        {
            try
            {
                if (Dispatcher.CheckAccess())
                {
                    UpdateVirtualizationUIContent(content, status);
                }
                else
                {
                    Dispatcher.BeginInvoke(() =>
                    {
                        try
                        {
                            UpdateVirtualizationUIContent(content, status);
                        }
                        catch (Exception uiEx)
                        {
                            System.Diagnostics.Debug.WriteLine($"가상화 UI 업데이트 실패: {uiEx.Message}");
                        }
                    });
                }
            }
            catch (Exception dispatcherEx)
            {
                System.Diagnostics.Debug.WriteLine($"가상화 Dispatcher 오류: {dispatcherEx.Message}");
            }
        }

        private void UpdateVirtualizationUIContent(string content, string status)
        {
            try
            {
                // VirtualizationInfoTextBox는 더 이상 사용되지 않음 - 테이블 형식으로 변경됨
                
                if (StatusTextBlock != null)
                {
                    StatusTextBlock.Text = status ?? "상태 불명";
                }
            }
            catch (Exception updateEx)
            {
                System.Diagnostics.Debug.WriteLine($"가상화 UI 컨트롤 업데이트 오류: {updateEx.Message}");
                throw;
            }
        }

        private void CheckHardwareVirtualization(StringBuilder result)
        {
            try
            {
                result.AppendLine("[1. 하드웨어 가상화 지원]");
                UpdateProgress("하드웨어 가상화 지원 확인 중...");
                
                using (var searcher = new ManagementObjectSearcher("SELECT VirtualizationFirmwareEnabled FROM Win32_Processor"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var isEnabled = obj["VirtualizationFirmwareEnabled"];
                        if (isEnabled != null && (bool)isEnabled)
                        {
                            result.AppendLine("✓ 하드웨어 가상화: 지원됨 및 활성화됨");
                            result.AppendLine("  → CPU에서 가상화 기술을 지원하며 BIOS/UEFI에서 활성화되어 있습니다.");
                        }
                        else
                        {
                            result.AppendLine("✗ 하드웨어 가상화: 비활성화됨 또는 지원되지 않음");
                            result.AppendLine("  → BIOS/UEFI 설정에서 Intel VT-x 또는 AMD-V를 활성화하세요.");
                        }
                        break;
                    }
                }
                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"✗ 하드웨어 가상화 확인 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CheckWSLInstallation(StringBuilder result)
        {
            try
            {
                result.AppendLine("[2. WSL (Windows Subsystem for Linux) 설치 상태]");
                UpdateProgress("WSL 설치 상태 확인 중...");

                var wslFeatures = new[]
                {
                    ("Microsoft-Windows-Subsystem-Linux", "WSL 1"),
                    ("VirtualMachinePlatform", "가상 머신 플랫폼 (WSL 2)")
                };

                bool anyWSLInstalled = false;

                foreach (var (featureName, displayName) in wslFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /get-featureinfo /featurename:{featureName}",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        var output = process.StandardOutput.ReadToEnd();
                        process.WaitForExit();
                        
                        if (output.Contains("State : Enabled"))
                        {
                            result.AppendLine($"✓ {displayName}: 설치됨 (활성화)");
                            anyWSLInstalled = true;
                        }
                        else if (output.Contains("State : Disabled"))
                        {
                            result.AppendLine($"○ {displayName}: 설치됨 (비활성화)");
                        }
                        else
                        {
                            result.AppendLine($"○ {displayName}: 설치되지 않음");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"? {displayName}: 확인 불가");
                    }
                }

                if (anyWSLInstalled)
                {
                    result.AppendLine("  → WSL이 설치되어 있어 가상화 성능에 영향을 줄 수 있습니다.");
                }
                else
                {
                    result.AppendLine("  → WSL이 설치되지 않아 가상화 성능에 영향을 주지 않습니다.");
                }

                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"✗ WSL 설치 상태 확인 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CheckHyperVInstallationDetailed(StringBuilder result)
        {
            try
            {
                result.AppendLine("[3. Hyper-V 설치 상태]");
                UpdateProgress("Hyper-V 설치 상태 확인 중...");

                var hyperVFeatures = new[]
                {
                    ("Microsoft-Hyper-V-All", "Hyper-V (전체)"),
                    ("Microsoft-Hyper-V", "Hyper-V 플랫폼"),
                    ("Microsoft-Hyper-V-Hypervisor", "Hyper-V 하이퍼바이저"),
                    ("Microsoft-Hyper-V-Services", "Hyper-V 서비스"),
                    ("Microsoft-Hyper-V-Management-Clients", "Hyper-V 관리 도구")
                };

                bool anyHyperVInstalled = false;

                foreach (var (featureName, displayName) in hyperVFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /get-featureinfo /featurename:{featureName}",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        var output = process.StandardOutput.ReadToEnd();
                        process.WaitForExit();
                        
                        if (output.Contains("State : Enabled"))
                        {
                            result.AppendLine($"✓ {displayName}: 설치됨 (활성화)");
                            anyHyperVInstalled = true;
                        }
                        else if (output.Contains("State : Disabled"))
                        {
                            result.AppendLine($"○ {displayName}: 설치됨 (비활성화)");
                        }
                        else
                        {
                            result.AppendLine($"○ {displayName}: 설치되지 않음");
                        }
                    }
                    catch
                    {
                        result.AppendLine($"? {displayName}: 확인 불가");
                    }
                }

                if (anyHyperVInstalled)
                {
                    result.AppendLine("  → Hyper-V가 설치되어 있어 다른 가상화 소프트웨어와 충돌할 수 있습니다.");
                }
                else
                {
                    result.AppendLine("  → Hyper-V가 설치되지 않아 다른 가상화 소프트웨어를 사용할 수 있습니다.");
                }

                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"✗ Hyper-V 설치 상태 확인 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CheckHypervisorLaunchType(StringBuilder result)
        {
            try
            {
                result.AppendLine("[4. 하이퍼바이저 시작 유형 (bcdedit)]");
                UpdateProgress("하이퍼바이저 시작 유형 확인 중...");

                var process = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = "bcdedit.exe",
                        Arguments = "/enum {current}",
                        UseShellExecute = false,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                        CreateNoWindow = true
                    }
                };
                
                process.Start();
                var output = process.StandardOutput.ReadToEnd();
                var error = process.StandardError.ReadToEnd();
                process.WaitForExit();
                
                if (process.ExitCode == 0)
                {
                    if (output.Contains("hypervisorlaunchtype") && output.Contains("Off"))
                    {
                        result.AppendLine("✓ hypervisorlaunchtype: Off (비활성화 상태)");
                        result.AppendLine("  → 부팅 시 하이퍼바이저가 로드되지 않아 가상화 소프트웨어 사용 가능");
                        result.AppendLine("  → 조치 필요없음");
                    }
                    else if (output.Contains("hypervisorlaunchtype") && output.Contains("Auto"))
                    {
                        result.AppendLine("✗ hypervisorlaunchtype: Auto (활성화 상태)");
                        result.AppendLine("  → 부팅 시 하이퍼바이저가 자동으로 로드되어 가상화 성능에 영향");
                        result.AppendLine("  → 비활성화 필요: 'VBS 및 Hyper-V 비활성화' 기능을 사용하여 Off로 변경 권장");
                    }
                    else
                    {
                        result.AppendLine("? hypervisorlaunchtype: 값이 존재하지 않음");
                        result.AppendLine("  → Windows 기본 설정 (보통 Auto와 동일하게 작동)");
                        result.AppendLine("  → 비활성화 권장: 'VBS 및 Hyper-V 비활성화' 기능을 사용하여 Off로 설정 권장");
                    }
                }
                else
                {
                    result.AppendLine($"✗ bcdedit 명령 실행 실패 (Exit Code: {process.ExitCode})");
                    if (!string.IsNullOrEmpty(error))
                    {
                        result.AppendLine($"  오류: {error.Trim()}");
                    }
                    result.AppendLine("  → 관리자 권한이 필요할 수 있습니다.");
                }

                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"✗ 하이퍼바이저 시작 유형 확인 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private void CheckVBSStatusDetailed(StringBuilder result)
        {
            try
            {
                result.AppendLine("[5. VBS (가상화 기반 보안) 상태 및 레지스트리 정보]");
                UpdateProgress("VBS 상태 및 레지스트리 확인 중...");

                var vbsRegistryKeys = new[]
                {
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "EnableVirtualizationBasedSecurity", "VBS 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "RequirePlatformSecurityFeatures", "플랫폼 보안 기능 요구"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled", "HVCI 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "WasEnabledBy", "HVCI 활성화 주체"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard", "Enabled", "시스템 가드 활성화"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "EnableVirtualizationBasedSecurity", "VBS 정책 활성화"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HypervisorEnforcedCodeIntegrity", "HVCI 정책"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HVCIMATRequired", "HVCI MAT 필요")
                };

                bool vbsEnabled = false;

                foreach (var (keyPath, valueName, description) in vbsRegistryKeys)
                {
                    try
                    {
                        using (var key = Registry.LocalMachine.OpenSubKey(keyPath))
                        {
                            if (key != null)
                            {
                                var value = key.GetValue(valueName);
                                if (value != null)
                                {
                                    string valueStr = value.ToString();
                                    string status = valueStr == "1" ? "활성화" : "비활성화";
                                    string symbol = valueStr == "1" ? "✗" : "✓";
                                    
                                    if (valueStr == "1" && (valueName.Contains("Enable") || valueName == "Enabled"))
                                    {
                                        vbsEnabled = true;
                                    }
                                    
                                    result.AppendLine($"{symbol} {description}: {status} (값: {valueStr})");
                                    result.AppendLine($"  레지스트리: HKLM\\{keyPath}");
                                    result.AppendLine($"  값 이름: {valueName}");
                                    result.AppendLine();
                                }
                                else
                                {
                                    result.AppendLine($"○ {description}: 설정되지 않음");
                                    result.AppendLine($"  레지스트리: HKLM\\{keyPath}");
                                    result.AppendLine($"  값 이름: {valueName}");
                                    result.AppendLine();
                                }
                            }
                            else
                            {
                                result.AppendLine($"○ {description}: 키가 존재하지 않음");
                                result.AppendLine($"  레지스트리: HKLM\\{keyPath}");
                                result.AppendLine();
                            }
                        }
                    }
                    catch (Exception regEx)
                    {
                        result.AppendLine($"✗ {description}: 확인 실패 ({regEx.Message})");
                        result.AppendLine($"  레지스트리: HKLM\\{keyPath}");
                        result.AppendLine();
                    }
                }

                // 종합 상태
                result.AppendLine("[VBS 종합 상태]");
                if (vbsEnabled)
                {
                    result.AppendLine("✗ VBS가 활성화되어 있어 가상화 성능에 영향을 줍니다.");
                    result.AppendLine("  → 'VBS 및 Hyper-V 비활성화' 기능을 사용하여 비활성화할 수 있습니다.");
                }
                else
                {
                    result.AppendLine("✓ VBS가 비활성화되어 있어 가상화 성능에 영향을 주지 않습니다.");
                }

                result.AppendLine();
            }
            catch (Exception ex)
            {
                result.AppendLine($"✗ VBS 상태 확인 실패: {ex.Message}");
                result.AppendLine();
            }
        }

        private async void ExecuteDisableButton_Click(object sender, RoutedEventArgs e)
        {
            // 가상화 설정 점검 상태 확인
            if (!isVirtualizationCheckCompleted || lastVirtualizationCheckResult == null)
            {
                var checkResult = MessageBox.Show(
                    "가상화 설정 점검을 진행하지 않은 상태로 조치를 진행할 경우 모든 점검 항목이 수정됩니다.\n\n" +
                    "그래도 진행하시겠습니까?",
                    "가상화 설정 점검 필요",
                    MessageBoxButton.YesNoCancel,
                    MessageBoxImage.Question);

                if (checkResult == MessageBoxResult.No)
                {
                    MessageBox.Show(
                        "가상화 설정 점검을 먼저 실행해 주세요.\n\n" +
                        "'가상화 설정' 탭에서 점검을 진행한 후 다시 시도하세요.",
                        "점검 필요",
                        MessageBoxButton.OK,
                        MessageBoxImage.Information);
                    return;
                }
                else if (checkResult == MessageBoxResult.Cancel)
                {
                    return;
                }
                // Yes인 경우 계속 진행
            }

            var result = MessageBox.Show(
                "조치 완료 후 PC 재부팅이 필요할 수 있습니다.\n" +
                "PC 재부팅 필요 시 진행 여부를 확인할 수 있는 팝업을 제공합니다.\n\n" +
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
                await Task.Run(() => DisableVbsAndHyperVWithIntegratedProgress());
            }
            catch (OperationCanceledException)
            {
                StatusTextBlock.Text = "작업이 취소되었습니다";
                AppendToResultTextBox("\n작업이 사용자에 의해 취소되었습니다.");
            }
            catch (Exception ex)
            {
                StatusTextBlock.Text = "작업 중 오류 발생";
                AppendToResultTextBox($"\n오류 발생: {ex.Message}");
            }
            finally
            {
                ExecuteDisableButton.IsEnabled = true;
                StatusTextBlock.Text = "준비됨";
            }
        }

        private void DisableVbsAndHyperV()
        {
            var result = new StringBuilder();
            result.AppendLine("=== VBS 및 Hyper-V 비활성화 작업 ===");

            // 가상화 설정 점검 결과가 있으면 필요한 항목만 조치
            if (isVirtualizationCheckCompleted && lastVirtualizationCheckResult != null)
            {
                result.AppendLine("가상화 설정 점검 결과를 기반으로 필요한 항목만 조치합니다.");
                DisableVbsAndHyperVSelective(result);
            }
            else
            {
                result.AppendLine("모든 항목에 대해 조치를 진행합니다.");
                DisableVbsAndHyperVComplete(result);
            }

            Dispatcher.Invoke(() =>
            {
                DisableResultTextBox.Text = result.ToString();
                ShowRebootDialog();
            });
        }

        private void AppendToResultTextBox(string text)
        {
            Dispatcher.Invoke(() =>
            {
                DisableResultTextBox.Text += text + Environment.NewLine;
                DisableResultTextBox.ScrollToEnd();
            });
        }

        private void UpdateCurrentTaskStatus(string status)
        {
            Dispatcher.Invoke(() =>
            {
                StatusTextBlock.Text = status;
            });
        }

        private void DisableVbsAndHyperVWithIntegratedProgress()
        {
            try
            {
                AppendToResultTextBox("=== VBS 및 Hyper-V 비활성화 작업 ===");
                Thread.Sleep(500); // UI 업데이트를 위한 짧은 대기

                // 가상화 설정 점검 결과가 있으면 필요한 항목만 조치
                if (isVirtualizationCheckCompleted && lastVirtualizationCheckResult != null)
                {
                    AppendToResultTextBox("가상화 설정 점검 결과를 기반으로 필요한 항목만 조치합니다.");
                    DisableVbsAndHyperVSelectiveWithIntegratedProgress();
                }
                else
                {
                    AppendToResultTextBox("모든 항목에 대해 조치를 진행합니다.");
                    DisableVbsAndHyperVCompleteWithIntegratedProgress();
                }

                AppendToResultTextBox("");
                AppendToResultTextBox("✅ 모든 비활성화 작업이 완료되었습니다.");
                AppendToResultTextBox("🔄 변경사항 적용을 위해 시스템 재부팅이 필요합니다.");

                Dispatcher.Invoke(() =>
                {
                    ShowRebootDialog();
                });
            }
            catch (Exception ex)
            {
                AppendToResultTextBox($"❌ 작업 중 오류가 발생했습니다: {ex.Message}");
                throw;
            }
            finally
            {
                UpdateCurrentTaskStatus("준비됨");
            }
        }

        private void DisableVbsAndHyperVCompleteWithIntegratedProgress()
        {
            var totalSteps = 4;
            var currentStep = 0;

            try
            {
                AppendToResultTextBox("");

                // 1. Hyper-V 기능 비활성화
                currentStep++;
                UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) Hyper-V 기능 비활성화 중...");
                AppendToResultTextBox($"📋 {currentStep}. Hyper-V 기능 비활성화 중...");

                var hyperVResult = DisableHyperVFeatures();
                AppendToResultTextBox(hyperVResult);
                AppendToResultTextBox("");

                Thread.Sleep(500); // UI 업데이트를 위한 짧은 대기

                // 2. WSL2 비활성화
                currentStep++;
                UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) WSL2 비활성화 중...");
                AppendToResultTextBox($"📋 {currentStep}. WSL2 비활성화 중...");

                var wslResult = DisableWSL2();
                AppendToResultTextBox(wslResult);
                AppendToResultTextBox("");

                Thread.Sleep(500);

                // 3. VBS 비활성화
                currentStep++;
                UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) VBS (가상화 기반 보안) 비활성화 중...");
                AppendToResultTextBox($"📋 {currentStep}. VBS (가상화 기반 보안) 비활성화 중...");

                var vbsResult = DisableVBS();
                AppendToResultTextBox(vbsResult);
                AppendToResultTextBox("");

                Thread.Sleep(500);

                // 4. 코어 격리 비활성화
                currentStep++;
                UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) 코어 격리 비활성화 중...");
                AppendToResultTextBox($"📋 {currentStep}. 코어 격리 비활성화 중...");

                var coreIsolationResult = DisableCoreIsolation();
                AppendToResultTextBox(coreIsolationResult);
                AppendToResultTextBox("");

                UpdateCurrentTaskStatus("모든 작업 완료");
            }
            catch (Exception ex)
            {
                AppendToResultTextBox($"❌ 단계 {currentStep}에서 오류 발생: {ex.Message}");
                throw;
            }
        }

        private void DisableVbsAndHyperVSelectiveWithIntegratedProgress()
        {
            try
            {
                AppendToResultTextBox("");

                // 점검 결과 분석
                var needsHyperVDisable = CheckIfHyperVNeedsDisabling();
                var needsWSLDisable = CheckIfWSLNeedsDisabling();
                var needsVBSDisable = CheckIfVBSNeedsDisabling();
                var needsCoreIsolationDisable = CheckIfCoreIsolationNeedsDisabling();

                var totalSteps = new List<bool> { needsHyperVDisable, needsWSLDisable, needsVBSDisable, needsCoreIsolationDisable }.Count(x => x);

                if (totalSteps == 0)
                {
                    UpdateCurrentTaskStatus("모든 항목이 이미 최적화되어 있습니다");
                    AppendToResultTextBox("✅ 모든 항목이 이미 최적화되어 있습니다. 추가 조치가 필요하지 않습니다.");
                    return;
                }

                var currentStep = 0;

                // 1. Hyper-V 기능 확인 및 비활성화
                if (needsHyperVDisable)
                {
                    currentStep++;
                    UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) Hyper-V 기능 비활성화 중...");
                    AppendToResultTextBox($"📋 {currentStep}. Hyper-V 기능 비활성화 중...");

                    var hyperVResult = DisableHyperVFeatures();
                    AppendToResultTextBox(hyperVResult);
                    AppendToResultTextBox("");
                    Thread.Sleep(500);
                }
                else
                {
                    AppendToResultTextBox("✅ Hyper-V 기능: 이미 비활성화되어 있음");
                    AppendToResultTextBox("");
                }

                // 2. WSL2 확인 및 비활성화
                if (needsWSLDisable)
                {
                    currentStep++;
                    UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) WSL2 비활성화 중...");
                    AppendToResultTextBox($"📋 {currentStep}. WSL2 비활성화 중...");

                    var wslResult = DisableWSL2();
                    AppendToResultTextBox(wslResult);
                    AppendToResultTextBox("");
                    Thread.Sleep(500);
                }
                else
                {
                    AppendToResultTextBox("✅ WSL2: 이미 비활성화되어 있음");
                    AppendToResultTextBox("");
                }

                // 3. VBS 확인 및 비활성화
                if (needsVBSDisable)
                {
                    currentStep++;
                    UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) VBS (가상화 기반 보안) 비활성화 중...");
                    AppendToResultTextBox($"📋 {currentStep}. VBS (가상화 기반 보안) 비활성화 중...");

                    var vbsResult = DisableVBSSelective();
                    AppendToResultTextBox(vbsResult);
                    AppendToResultTextBox("");
                    Thread.Sleep(500);
                }
                else
                {
                    AppendToResultTextBox("✅ VBS (가상화 기반 보안): 이미 비활성화되어 있음");
                    AppendToResultTextBox("");
                }

                // 4. 코어 격리 확인 및 비활성화
                if (needsCoreIsolationDisable)
                {
                    currentStep++;
                    UpdateCurrentTaskStatus($"({currentStep}/{totalSteps}) 코어 격리 비활성화 중...");
                    AppendToResultTextBox($"📋 {currentStep}. 코어 격리 비활성화 중...");

                    var coreIsolationResult = DisableCoreIsolation();
                    AppendToResultTextBox(coreIsolationResult);
                    AppendToResultTextBox("");
                }
                else
                {
                    AppendToResultTextBox("✅ 코어 격리: 이미 비활성화되어 있음");
                    AppendToResultTextBox("");
                }

                if (currentStep > 0)
                {
                    AppendToResultTextBox($"📊 총 {currentStep}개 항목의 비활성화 작업이 완료되었습니다.");
                }

                UpdateCurrentTaskStatus("선택적 작업 완료");
            }
            catch (Exception ex)
            {
                AppendToResultTextBox($"❌ 선택적 비활성화 중 오류 발생: {ex.Message}");
                throw;
            }
        }




        private void DisableVbsAndHyperVComplete(StringBuilder result)
        {
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
        }

        private void DisableVbsAndHyperVSelective(StringBuilder result)
        {
            result.AppendLine();

            var actionCount = 0;

            // 점검 결과 분석
            var needsHyperVDisable = CheckIfHyperVNeedsDisabling();
            var needsWSLDisable = CheckIfWSLNeedsDisabling();
            var needsVBSDisable = CheckIfVBSNeedsDisabling();
            var needsCoreIsolationDisable = CheckIfCoreIsolationNeedsDisabling();

            // 1. Hyper-V 기능 확인 및 비활성화
            if (needsHyperVDisable)
            {
                actionCount++;
                result.AppendLine($"{actionCount}. Hyper-V 기능 비활성화 중...");
                var hyperVResult = DisableHyperVFeatures();
                result.AppendLine(hyperVResult);
                result.AppendLine();
            }
            else
            {
                result.AppendLine("✓ Hyper-V 기능: 이미 비활성화되어 있음");
                result.AppendLine();
            }

            // 2. WSL2 확인 및 비활성화
            if (needsWSLDisable)
            {
                actionCount++;
                result.AppendLine($"{actionCount}. WSL2 비활성화 중...");
                var wslResult = DisableWSL2();
                result.AppendLine(wslResult);
                result.AppendLine();
            }
            else
            {
                result.AppendLine("✓ WSL2: 이미 비활성화되어 있음");
                result.AppendLine();
            }

            // 3. VBS 확인 및 비활성화
            if (needsVBSDisable)
            {
                actionCount++;
                result.AppendLine($"{actionCount}. VBS (가상화 기반 보안) 비활성화 중...");
                var vbsResult = DisableVBSSelective();
                result.AppendLine(vbsResult);
                result.AppendLine();
            }
            else
            {
                result.AppendLine("✓ VBS (가상화 기반 보안): 이미 비활성화되어 있음");
                result.AppendLine();
            }

            // 4. 코어 격리 확인 및 비활성화
            if (needsCoreIsolationDisable)
            {
                actionCount++;
                result.AppendLine($"{actionCount}. 코어 격리 비활성화 중...");
                var coreIsolationResult = DisableCoreIsolation();
                result.AppendLine(coreIsolationResult);
                result.AppendLine();
            }
            else
            {
                result.AppendLine("✓ 코어 격리: 이미 비활성화되어 있음");
                result.AppendLine();
            }

            if (actionCount > 0)
            {
                result.AppendLine($"총 {actionCount}개 항목의 비활성화 작업이 완료되었습니다.");
                result.AppendLine("변경사항을 적용하려면 시스템을 재부팅해야 합니다.");
            }
            else
            {
                result.AppendLine("모든 항목이 이미 최적화되어 있습니다. 추가 조치가 필요하지 않습니다.");
            }
        }

        private bool CheckIfHyperVNeedsDisabling()
        {
            if (lastVirtualizationCheckResult == null) return true;

            foreach (var item in lastVirtualizationCheckResult)
            {
                if (item.Category.Contains("Hyper-V") && item.Status.Contains("활성화"))
                {
                    return true;
                }
            }
            return false;
        }

        private bool CheckIfWSLNeedsDisabling()
        {
            if (lastVirtualizationCheckResult == null) return true;

            foreach (var item in lastVirtualizationCheckResult)
            {
                if ((item.Category.Contains("WSL") || item.Category.Contains("Virtual Machine Platform")) &&
                    item.Status.Contains("활성화"))
                {
                    return true;
                }
            }
            return false;
        }

        private bool CheckIfVBSNeedsDisabling()
        {
            if (lastVirtualizationCheckResult == null) return true;

            foreach (var item in lastVirtualizationCheckResult)
            {
                if ((item.Category.Contains("VBS") || item.Category.Contains("HVCI") ||
                     item.Category.Contains("Credential Guard") || item.Category.Contains("Key Guard") ||
                     item.Category.Contains("시스템 가드") || item.Category.Contains("보안 생체") ||
                     item.Category.Contains("LSA")) && item.Status.Contains("활성화"))
                {
                    return true;
                }
            }
            return false;
        }

        private bool CheckIfCoreIsolationNeedsDisabling()
        {
            if (lastVirtualizationCheckResult == null) return true;

            foreach (var item in lastVirtualizationCheckResult)
            {
                if ((item.Category.Contains("정책") || item.Category.Contains("MAT") ||
                     item.Category.Contains("취약한 드라이버")) && item.Status.Contains("활성화"))
                {
                    return true;
                }
            }
            return false;
        }

        private string DisableVBSSelective()
        {
            try
            {
                var result = new StringBuilder();

                // 점검 결과에서 활성화된 VBS 관련 항목만 비활성화
                var vbsKeysToDisable = new List<(string keyPath, string valueName, string description)>();

                if (lastVirtualizationCheckResult != null)
                {
                    foreach (var item in lastVirtualizationCheckResult)
                    {
                        if (item.Status.Contains("활성화") && item.Details.Contains("레지스트리"))
                        {
                            // 레지스트리 경로에서 키 정보 추출
                            var details = item.Details;
                            if (details.Contains("HKLM\\"))
                            {
                                var regPath = details.Substring(details.IndexOf("HKLM\\") + 5);
                                if (regPath.Contains("\\"))
                                {
                                    var pathParts = regPath.Split('\\');
                                    if (pathParts.Length >= 2)
                                    {
                                        var keyPath = string.Join("\\", pathParts.Take(pathParts.Length - 1));
                                        var valueName = pathParts.Last();
                                        vbsKeysToDisable.Add((keyPath, valueName, item.Category));
                                    }
                                }
                            }
                        }
                    }
                }

                // 감지된 항목들만 비활성화
                if (vbsKeysToDisable.Any())
                {
                    foreach (var (keyPath, valueName, description) in vbsKeysToDisable)
                    {
                        try
                        {
                            using (var key = Registry.LocalMachine.CreateSubKey(keyPath))
                            {
                                key?.SetValue(valueName, 0, RegistryValueKind.DWord);
                                result.AppendLine($"   ✓ {description}: {keyPath}\\{valueName} = 0 설정 완료");
                            }
                        }
                        catch
                        {
                            result.AppendLine($"   ✗ {description}: {keyPath}\\{valueName} 설정 실패");
                        }
                    }
                }
                else
                {
                    result.AppendLine("   점검 결과 비활성화가 필요한 VBS 항목이 없습니다.");
                }

                return result.ToString();
            }
            catch (Exception ex)
            {
                return $"   ✗ VBS 선택적 비활성화 중 오류 발생: {ex.Message}";
            }
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

                // bcdedit를 사용하여 하이퍼바이저 시작 유형 비활성화 (완전한 Hyper-V 비활성화)
                result.AppendLine();
                result.AppendLine("   하이퍼바이저 시작 유형 비활성화 중...");
                try
                {
                    var bcdProcess = new Process
                    {
                        StartInfo = new ProcessStartInfo
                        {
                            FileName = "bcdedit.exe",
                            Arguments = "/set hypervisorlaunchtype off",
                            UseShellExecute = false,
                            RedirectStandardOutput = true,
                            RedirectStandardError = true,
                            CreateNoWindow = true
                        }
                    };
                    
                    bcdProcess.Start();
                    var output = bcdProcess.StandardOutput.ReadToEnd();
                    var error = bcdProcess.StandardError.ReadToEnd();
                    bcdProcess.WaitForExit();
                    
                    if (bcdProcess.ExitCode == 0)
                    {
                        result.AppendLine("   ✓ 하이퍼바이저 시작 유형 비활성화 성공");
                        result.AppendLine("   → 부팅 시 하이퍼바이저가 로드되지 않습니다");
                    }
                    else
                    {
                        result.AppendLine($"   ✗ 하이퍼바이저 시작 유형 비활성화 실패 (Exit Code: {bcdProcess.ExitCode})");
                        if (!string.IsNullOrEmpty(error))
                        {
                            result.AppendLine($"   오류: {error.Trim()}");
                        }
                    }
                }
                catch (Exception bcdEx)
                {
                    result.AppendLine($"   ✗ bcdedit 명령 실행 실패: {bcdEx.Message}");
                    result.AppendLine("   → 관리자 권한이 필요할 수 있습니다");
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
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard", "Enabled", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard", "Enabled", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "EncryptionKeyAvailable", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "EncryptionKeyPersistent", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "IsSecureKernelRunning", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "KeyGuardEnabled", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "SecretsMode", 0),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SecureBiometrics", "Enabled", 0),
                    (@"SYSTEM\CurrentControlSet\Control\Lsa", "LsaCfgFlags", 0),
                    (@"SYSTEM\CurrentControlSet\Control\Lsa", "LsaCfgFlagsDefault", 0)
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
                            Arguments = "/r /t 5 /c \"VM Compatibility Tool에 의한 시스템 재부팅\"",
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

        private void SafeUpdateVirtualizationTableUI(ObservableCollection<VirtualizationInfoItem> items, string status)
        {
            try
            {
                if (Dispatcher.CheckAccess())
                {
                    VirtualizationDataGrid.ItemsSource = items;
                    StatusTextBlock.Text = status;

                    // 가상화 설정 점검 완료 상태 업데이트
                    if (status.Contains("완료") && items != null && items.Count > 0)
                    {
                        isVirtualizationCheckCompleted = true;
                        lastVirtualizationCheckResult = items;
                    }
                }
                else
                {
                    Dispatcher.BeginInvoke(() =>
                    {
                        try
                        {
                            VirtualizationDataGrid.ItemsSource = items;
                            StatusTextBlock.Text = status;

                            // 가상화 설정 점검 완료 상태 업데이트
                            if (status.Contains("완료") && items != null && items.Count > 0)
                            {
                                isVirtualizationCheckCompleted = true;
                                lastVirtualizationCheckResult = items;
                            }
                        }
                        catch (Exception uiEx)
                        {
                            System.Diagnostics.Debug.WriteLine($"가상화 테이블 UI 업데이트 실패: {uiEx.Message}");
                        }
                    });
                }
            }
            catch (Exception dispatcherEx)
            {
                System.Diagnostics.Debug.WriteLine($"가상화 테이블 Dispatcher 오류: {dispatcherEx.Message}");
            }
        }

        // 테이블 형식 시스템 정보 수집 메소드들
        private void CollectOSInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("운영체제 정보 수집 중...");
                var osInfo = GetWindowsVersionInfo();
                
                // OS 정보를 파싱하여 테이블 형식으로 변환
                var lines = osInfo.Split('\n', StringSplitOptions.RemoveEmptyEntries);
                foreach (var line in lines)
                {
                    var trimmedLine = line.Trim();
                    if (trimmedLine.Contains(":"))
                    {
                        var parts = trimmedLine.Split(':', 2);
                        if (parts.Length == 2)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "운영체제",
                                Item = parts[0].Trim(),
                                Value = parts[1].Trim()
                            });
                        }
                    }
                    else if (!string.IsNullOrEmpty(trimmedLine) && !trimmedLine.StartsWith("="))
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "운영체제",
                            Item = "정보",
                            Value = trimmedLine
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "운영체제",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectCPUInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("프로세서 정보 수집 중...");
                var cpuInfo = GetCpuInfo();
                
                var lines = cpuInfo.Split('\n', StringSplitOptions.RemoveEmptyEntries);
                foreach (var line in lines)
                {
                    var trimmedLine = line.Trim();
                    if (trimmedLine.Contains(":"))
                    {
                        var parts = trimmedLine.Split(':', 2);
                        if (parts.Length == 2)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "프로세서",
                                Item = parts[0].Trim(),
                                Value = parts[1].Trim()
                            });
                        }
                    }
                    else if (!string.IsNullOrEmpty(trimmedLine) && !trimmedLine.StartsWith("="))
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "프로세서",
                            Item = "정보",
                            Value = trimmedLine
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "프로세서",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectMemoryInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("메모리 정보 수집 중...");
                var memoryInfo = GetMemoryInfo();
                
                var lines = memoryInfo.Split('\n', StringSplitOptions.RemoveEmptyEntries);
                foreach (var line in lines)
                {
                    var trimmedLine = line.Trim();
                    if (trimmedLine.Contains(":"))
                    {
                        var parts = trimmedLine.Split(':', 2);
                        if (parts.Length == 2)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "메모리",
                                Item = parts[0].Trim(),
                                Value = parts[1].Trim()
                            });
                        }
                    }
                    else if (!string.IsNullOrEmpty(trimmedLine) && !trimmedLine.StartsWith("="))
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "메모리",
                            Item = "정보",
                            Value = trimmedLine
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "메모리",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectDiskInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("디스크 정보 수집 중...");
                var diskInfo = GetDiskInfo();
                
                var lines = diskInfo.Split('\n', StringSplitOptions.RemoveEmptyEntries);
                foreach (var line in lines)
                {
                    var trimmedLine = line.Trim();
                    if (trimmedLine.Contains(":"))
                    {
                        var parts = trimmedLine.Split(':', 2);
                        if (parts.Length == 2)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "디스크",
                                Item = parts[0].Trim(),
                                Value = parts[1].Trim()
                            });
                        }
                    }
                    else if (!string.IsNullOrEmpty(trimmedLine) && !trimmedLine.StartsWith("=") && !trimmedLine.StartsWith("드라이브"))
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "디스크",
                            Item = "정보",
                            Value = trimmedLine
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "디스크",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectBootInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("부팅 정보 수집 중...");
                
                // 부팅 시간 정보 수집
                try
                {
                    // Environment.TickCount를 사용하여 시스템 가동 시간 계산
                    var uptimeMs = Environment.TickCount;
                    var uptime = TimeSpan.FromMilliseconds(uptimeMs);
                    
                    items.Add(new SystemInfoItem
                    {
                        Category = "부팅 정보",
                        Item = "시스템 가동 시간",
                        Value = $"{uptime.Days}일 {uptime.Hours}시간 {uptime.Minutes}분"
                    });
                    
                    var bootTime = DateTime.Now - uptime;
                    items.Add(new SystemInfoItem
                    {
                        Category = "부팅 정보",
                        Item = "마지막 부팅 시간 (추정)",
                        Value = bootTime.ToString("yyyy-MM-dd HH:mm:ss")
                    });
                }
                catch (Exception bootEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "부팅 정보",
                        Item = "부팅 시간",
                        Value = $"확인 불가: {bootEx.Message}"
                    });
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "부팅 정보",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectMotherboardInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("메인보드 정보 수집 중...");

                // 메인보드 제조사 및 모델 정보
                try
                {
                    using (var searcher = new ManagementObjectSearcher("SELECT Manufacturer, Product FROM Win32_BaseBoard"))
                    {
                        foreach (ManagementObject obj in searcher.Get())
                        {
                            var manufacturer = obj["Manufacturer"]?.ToString() ?? "확인 불가";
                            var product = obj["Product"]?.ToString() ?? "확인 불가";

                            items.Add(new SystemInfoItem
                            {
                                Category = "메인보드",
                                Item = "제조사",
                                Value = manufacturer
                            });

                            items.Add(new SystemInfoItem
                            {
                                Category = "메인보드",
                                Item = "모델",
                                Value = product
                            });
                        }
                    }
                }
                catch (Exception mbEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "메인보드",
                        Item = "제조사/모델",
                        Value = $"확인 불가: {mbEx.Message}"
                    });
                }

                // 칩셋 정보
                try
                {
                    using (var searcher = new ManagementObjectSearcher("SELECT Name, Description FROM Win32_IDEController"))
                    {
                        bool chipsetFound = false;
                        foreach (ManagementObject obj in searcher.Get())
                        {
                            var name = obj["Name"]?.ToString() ?? "";
                            if (!string.IsNullOrWhiteSpace(name))
                            {
                                items.Add(new SystemInfoItem
                                {
                                    Category = "메인보드",
                                    Item = "칩셋 컨트롤러",
                                    Value = name
                                });
                                chipsetFound = true;
                                break; // 첫 번째 항목만 표시
                            }
                        }

                        if (!chipsetFound)
                        {
                            // PCI 버스로 칩셋 정보 확인
                            using (var pciSearcher = new ManagementObjectSearcher("SELECT Name FROM Win32_Bus WHERE DeviceID LIKE 'PCI%'"))
                            {
                                foreach (ManagementObject pciObj in pciSearcher.Get())
                                {
                                    var name = pciObj["Name"]?.ToString() ?? "PCI 버스";
                                    items.Add(new SystemInfoItem
                                    {
                                        Category = "메인보드",
                                        Item = "칩셋",
                                        Value = name
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
                catch (Exception chipsetEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "메인보드",
                        Item = "칩셋",
                        Value = $"확인 불가: {chipsetEx.Message}"
                    });
                }

                // BIOS 정보
                try
                {
                    using (var searcher = new ManagementObjectSearcher("SELECT Manufacturer, Name, SMBIOSBIOSVersion, ReleaseDate FROM Win32_BIOS"))
                    {
                        foreach (ManagementObject obj in searcher.Get())
                        {
                            var biosManufacturer = obj["Manufacturer"]?.ToString() ?? "확인 불가";
                            var biosName = obj["Name"]?.ToString() ?? "확인 불가";
                            var biosVersion = obj["SMBIOSBIOSVersion"]?.ToString() ?? "확인 불가";
                            var biosDate = obj["ReleaseDate"]?.ToString() ?? "";

                            items.Add(new SystemInfoItem
                            {
                                Category = "메인보드",
                                Item = "BIOS 제조사",
                                Value = biosManufacturer
                            });

                            items.Add(new SystemInfoItem
                            {
                                Category = "메인보드",
                                Item = "BIOS 종류",
                                Value = biosName
                            });

                            items.Add(new SystemInfoItem
                            {
                                Category = "메인보드",
                                Item = "BIOS 버전",
                                Value = biosVersion
                            });

                            if (!string.IsNullOrWhiteSpace(biosDate) && biosDate.Length >= 8)
                            {
                                // WMI 날짜 형식: yyyyMMddHHmmss.xxxxxx+xxx
                                string formattedDate = $"{biosDate.Substring(0, 4)}-{biosDate.Substring(4, 2)}-{biosDate.Substring(6, 2)}";
                                items.Add(new SystemInfoItem
                                {
                                    Category = "메인보드",
                                    Item = "BIOS 날짜",
                                    Value = formattedDate
                                });
                            }
                        }
                    }
                }
                catch (Exception biosEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "메인보드",
                        Item = "BIOS 정보",
                        Value = $"확인 불가: {biosEx.Message}"
                    });
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "메인보드",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectGraphicsCardInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("그래픽카드 정보 수집 중...");

                using (var searcher = new ManagementObjectSearcher("SELECT Name, AdapterRAM, DriverVersion, DriverDate FROM Win32_VideoController"))
                {
                    int gpuIndex = 0;
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        gpuIndex++;
                        string categoryName = gpuIndex == 1 ? "그래픽카드" : $"그래픽카드 #{gpuIndex}";

                        // GPU 모델명
                        var gpuName = obj["Name"]?.ToString() ?? "확인 불가";
                        items.Add(new SystemInfoItem
                        {
                            Category = categoryName,
                            Item = "모델",
                            Value = gpuName
                        });

                        // 비디오 메모리
                        try
                        {
                            var vramBytes = obj["AdapterRAM"];
                            if (vramBytes != null)
                            {
                                long vram = Convert.ToInt64(vramBytes);
                                if (vram > 0)
                                {
                                    double vramGB = vram / (1024.0 * 1024.0 * 1024.0);
                                    items.Add(new SystemInfoItem
                                    {
                                        Category = categoryName,
                                        Item = "비디오 메모리",
                                        Value = $"{vramGB:F2} GB"
                                    });
                                }
                            }
                        }
                        catch { }

                        // 드라이버 버전
                        var driverVersion = obj["DriverVersion"]?.ToString() ?? "확인 불가";
                        items.Add(new SystemInfoItem
                        {
                            Category = categoryName,
                            Item = "드라이버 버전",
                            Value = driverVersion
                        });

                        // 드라이버 날짜
                        try
                        {
                            var driverDate = obj["DriverDate"]?.ToString() ?? "";
                            if (!string.IsNullOrWhiteSpace(driverDate) && driverDate.Length >= 8)
                            {
                                // WMI 날짜 형식: yyyyMMddHHmmss.xxxxxx+xxx
                                string formattedDate = $"{driverDate.Substring(0, 4)}-{driverDate.Substring(4, 2)}-{driverDate.Substring(6, 2)}";
                                items.Add(new SystemInfoItem
                                {
                                    Category = categoryName,
                                    Item = "드라이버 날짜",
                                    Value = formattedDate
                                });
                            }
                            else
                            {
                                items.Add(new SystemInfoItem
                                {
                                    Category = categoryName,
                                    Item = "드라이버 날짜",
                                    Value = "확인 불가"
                                });
                            }
                        }
                        catch (Exception dateEx)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = categoryName,
                                Item = "드라이버 날짜",
                                Value = $"확인 불가: {dateEx.Message}"
                            });
                        }
                    }

                    if (gpuIndex == 0)
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "그래픽카드",
                            Item = "정보",
                            Value = "그래픽카드를 찾을 수 없습니다"
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "그래픽카드",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectPowerManagementInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("전원 관리 정보 수집 중...");

                // 현재 전원 구성표
                try
                {
                    var powerCfgProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/getactivescheme",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    powerCfgProcess.Start();
                    string output = powerCfgProcess.StandardOutput.ReadToEnd();
                    powerCfgProcess.WaitForExit();

                    // 출력 예: Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (균형 조정)
                    var match = System.Text.RegularExpressions.Regex.Match(output, @"\((.+)\)");
                    string schemeName = match.Success ? match.Groups[1].Value : "확인 불가";

                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "현재 전원 구성표",
                        Value = schemeName
                    });
                }
                catch (Exception powerEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "전원 구성표",
                        Value = $"확인 불가: {powerEx.Message}"
                    });
                }

                // 디스플레이 끄기 설정 (AC 및 DC)
                try
                {
                    // AC (전원 연결 시)
                    var displayAcProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/query SCHEME_CURRENT SUB_VIDEO VIDEOIDLE",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    displayAcProcess.Start();
                    string displayOutput = displayAcProcess.StandardOutput.ReadToEnd();
                    displayAcProcess.WaitForExit();

                    var acMatch = System.Text.RegularExpressions.Regex.Match(displayOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!acMatch.Success)
                    {
                        acMatch = System.Text.RegularExpressions.Regex.Match(displayOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    int displayAcSeconds = 0;
                    int displayDcSeconds = 0;

                    if (acMatch.Success)
                    {
                        displayAcSeconds = Convert.ToInt32(acMatch.Groups[1].Value, 16);
                        string timeStr = displayAcSeconds == 0 ? "해당 없음" : $"{displayAcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "디스플레이 끄기 (전원 연결)",
                            Value = timeStr
                        });
                    }

                    var dcMatch = System.Text.RegularExpressions.Regex.Match(displayOutput, @"현재 DC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!dcMatch.Success)
                    {
                        dcMatch = System.Text.RegularExpressions.Regex.Match(displayOutput, @"Current DC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (dcMatch.Success)
                    {
                        displayDcSeconds = Convert.ToInt32(dcMatch.Groups[1].Value, 16);
                        string timeStr = displayDcSeconds == 0 ? "해당 없음" : $"{displayDcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "디스플레이 끄기 (배터리)",
                            Value = timeStr
                        });
                    }

                    // 현재 PC의 실제 설정 (AC 또는 DC 중 활성)
                    bool isOnBattery = false;
                    try
                    {
                        using (var batterySearcher = new ManagementObjectSearcher("SELECT BatteryStatus FROM Win32_Battery"))
                        {
                            var batteryCollection = batterySearcher.Get();
                            isOnBattery = batteryCollection.Count > 0; // 배터리가 있으면 노트북으로 간주
                        }
                    }
                    catch { }

                    string currentSetting = isOnBattery ?
                        (displayDcSeconds == 0 ? "해당 없음" : $"{displayDcSeconds / 60}분") :
                        (displayAcSeconds == 0 ? "해당 없음" : $"{displayAcSeconds / 60}분");

                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "▶ 현재 디스플레이 끄기 설정",
                        Value = currentSetting
                    });
                }
                catch (Exception displayEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "디스플레이 설정",
                        Value = $"확인 불가: {displayEx.Message}"
                    });
                }

                // 절전 모드 설정
                try
                {
                    var sleepProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/query SCHEME_CURRENT SUB_SLEEP STANDBYIDLE",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    sleepProcess.Start();
                    string sleepOutput = sleepProcess.StandardOutput.ReadToEnd();
                    sleepProcess.WaitForExit();

                    var acMatch = System.Text.RegularExpressions.Regex.Match(sleepOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!acMatch.Success)
                    {
                        acMatch = System.Text.RegularExpressions.Regex.Match(sleepOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    int sleepAcSeconds = 0;
                    int sleepDcSeconds = 0;

                    if (acMatch.Success)
                    {
                        sleepAcSeconds = Convert.ToInt32(acMatch.Groups[1].Value, 16);
                        string timeStr = sleepAcSeconds == 0 ? "해당 없음" : $"{sleepAcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "절전 모드 (전원 연결)",
                            Value = timeStr
                        });
                    }

                    var dcMatch = System.Text.RegularExpressions.Regex.Match(sleepOutput, @"현재 DC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!dcMatch.Success)
                    {
                        dcMatch = System.Text.RegularExpressions.Regex.Match(sleepOutput, @"Current DC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (dcMatch.Success)
                    {
                        sleepDcSeconds = Convert.ToInt32(dcMatch.Groups[1].Value, 16);
                        string timeStr = sleepDcSeconds == 0 ? "해당 없음" : $"{sleepDcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "절전 모드 (배터리)",
                            Value = timeStr
                        });
                    }

                    // 현재 PC의 실제 설정 (AC 또는 DC 중 활성)
                    bool isOnBattery = false;
                    try
                    {
                        using (var batterySearcher = new ManagementObjectSearcher("SELECT BatteryStatus FROM Win32_Battery"))
                        {
                            var batteryCollection = batterySearcher.Get();
                            isOnBattery = batteryCollection.Count > 0; // 배터리가 있으면 노트북으로 간주
                        }
                    }
                    catch { }

                    string currentSleepSetting = isOnBattery ?
                        (sleepDcSeconds == 0 ? "해당 없음" : $"{sleepDcSeconds / 60}분") :
                        (sleepAcSeconds == 0 ? "해당 없음" : $"{sleepAcSeconds / 60}분");

                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "▶ 현재 절전 모드 설정",
                        Value = currentSleepSetting
                    });
                }
                catch (Exception sleepEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "절전 모드",
                        Value = $"확인 불가: {sleepEx.Message}"
                    });
                }

                // USB 선택적 절전 설정
                try
                {
                    var usbProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/query SCHEME_CURRENT SUB_USB USBSELECTIVESUSPEND",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    usbProcess.Start();
                    string usbOutput = usbProcess.StandardOutput.ReadToEnd();
                    usbProcess.WaitForExit();

                    var acMatch = System.Text.RegularExpressions.Regex.Match(usbOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!acMatch.Success)
                    {
                        acMatch = System.Text.RegularExpressions.Regex.Match(usbOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (acMatch.Success)
                    {
                        int value = Convert.ToInt32(acMatch.Groups[1].Value, 16);
                        string status = value == 0 ? "사용 안 함" : "사용";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "USB 선택적 절전 모드",
                            Value = status
                        });
                    }
                }
                catch (Exception usbEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "USB 절전",
                        Value = $"확인 불가: {usbEx.Message}"
                    });
                }

                // 하드 디스크 끄기 설정
                try
                {
                    var diskProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/query SCHEME_CURRENT SUB_DISK DISKIDLE",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    diskProcess.Start();
                    string diskOutput = diskProcess.StandardOutput.ReadToEnd();
                    diskProcess.WaitForExit();

                    var acMatch = System.Text.RegularExpressions.Regex.Match(diskOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!acMatch.Success)
                    {
                        acMatch = System.Text.RegularExpressions.Regex.Match(diskOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    int diskAcSeconds = 0;
                    int diskDcSeconds = 0;

                    if (acMatch.Success)
                    {
                        diskAcSeconds = Convert.ToInt32(acMatch.Groups[1].Value, 16);
                        string timeStr = diskAcSeconds == 0 ? "해당 없음" : $"{diskAcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "하드 디스크 끄기 (전원 연결)",
                            Value = timeStr
                        });
                    }

                    var dcMatch = System.Text.RegularExpressions.Regex.Match(diskOutput, @"현재 DC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!dcMatch.Success)
                    {
                        dcMatch = System.Text.RegularExpressions.Regex.Match(diskOutput, @"Current DC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (dcMatch.Success)
                    {
                        diskDcSeconds = Convert.ToInt32(dcMatch.Groups[1].Value, 16);
                        string timeStr = diskDcSeconds == 0 ? "해당 없음" : $"{diskDcSeconds / 60}분";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "하드 디스크 끄기 (배터리)",
                            Value = timeStr
                        });
                    }

                    // 현재 PC의 실제 설정
                    bool isOnBattery = false;
                    try
                    {
                        using (var batterySearcher = new ManagementObjectSearcher("SELECT BatteryStatus FROM Win32_Battery"))
                        {
                            var batteryCollection = batterySearcher.Get();
                            isOnBattery = batteryCollection.Count > 0; // 배터리가 있으면 노트북으로 간주
                        }
                    }
                    catch { }

                    string currentDiskSetting = isOnBattery ?
                        (diskDcSeconds == 0 ? "해당 없음" : $"{diskDcSeconds / 60}분") :
                        (diskAcSeconds == 0 ? "해당 없음" : $"{diskAcSeconds / 60}분");

                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "▶ 현재 하드 디스크 끄기 설정",
                        Value = currentDiskSetting
                    });
                }
                catch (Exception diskEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "하드 디스크 끄기",
                        Value = $"확인 불가: {diskEx.Message}"
                    });
                }

                // 하이브리드 절전 모드 설정
                try
                {
                    var hybridProcess = new System.Diagnostics.Process
                    {
                        StartInfo = new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = "powercfg",
                            Arguments = "/query SCHEME_CURRENT SUB_SLEEP HYBRIDSLEEP",
                            RedirectStandardOutput = true,
                            UseShellExecute = false,
                            CreateNoWindow = true
                        }
                    };
                    hybridProcess.Start();
                    string hybridOutput = hybridProcess.StandardOutput.ReadToEnd();
                    hybridProcess.WaitForExit();

                    var acMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!acMatch.Success)
                    {
                        acMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (acMatch.Success)
                    {
                        int value = Convert.ToInt32(acMatch.Groups[1].Value, 16);
                        string status = value == 0 ? "끔" : "켜짐";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "하이브리드 절전 모드 허용 (전원 연결)",
                            Value = status
                        });
                    }

                    var dcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"현재 DC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    if (!dcMatch.Success)
                    {
                        dcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"Current DC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    }

                    if (dcMatch.Success)
                    {
                        int value = Convert.ToInt32(dcMatch.Groups[1].Value, 16);
                        string status = value == 0 ? "끔" : "켜짐";
                        items.Add(new SystemInfoItem
                        {
                            Category = "전원 관리",
                            Item = "하이브리드 절전 모드 허용 (배터리)",
                            Value = status
                        });
                    }

                    // 현재 설정
                    bool isOnBattery = false;
                    try
                    {
                        using (var batterySearcher = new ManagementObjectSearcher("SELECT BatteryStatus FROM Win32_Battery"))
                        {
                            var batteryCollection = batterySearcher.Get();
                            isOnBattery = batteryCollection.Count > 0; // 배터리가 있으면 노트북으로 간주
                        }
                    }
                    catch { }

                    var currentAcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)");
                    var currentDcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"Current DC Power Setting Index:\s*0x([0-9a-fA-F]+)");

                    if (!currentAcMatch.Success)
                    {
                        currentAcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"현재 AC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    }
                    if (!currentDcMatch.Success)
                    {
                        currentDcMatch = System.Text.RegularExpressions.Regex.Match(hybridOutput, @"현재 DC 전원 설정 인덱스:\s*0x([0-9a-fA-F]+)");
                    }

                    string currentHybridStatus = "확인 불가";
                    if (isOnBattery && currentDcMatch.Success)
                    {
                        int val = Convert.ToInt32(currentDcMatch.Groups[1].Value, 16);
                        currentHybridStatus = val == 0 ? "끔" : "켜짐";
                    }
                    else if (!isOnBattery && currentAcMatch.Success)
                    {
                        int val = Convert.ToInt32(currentAcMatch.Groups[1].Value, 16);
                        currentHybridStatus = val == 0 ? "끔" : "켜짐";
                    }

                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "▶ 현재 하이브리드 절전 모드",
                        Value = currentHybridStatus
                    });
                }
                catch (Exception hybridEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "하이브리드 절전 모드",
                        Value = $"확인 불가: {hybridEx.Message}"
                    });
                }

                // 화면보호기 설정
                try
                {
                    using (var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(@"Control Panel\Desktop"))
                    {
                        if (key != null)
                        {
                            var screensaverActive = key.GetValue("ScreenSaveActive")?.ToString() ?? "0";
                            bool isActive = screensaverActive == "1";

                            items.Add(new SystemInfoItem
                            {
                                Category = "전원 관리",
                                Item = "화면보호기 사용",
                                Value = isActive ? "사용" : "사용 안 함"
                            });

                            if (isActive)
                            {
                                // 화면보호기 실행 파일
                                var screensaver = key.GetValue("SCRNSAVE.EXE")?.ToString() ?? "없음";
                                if (!string.IsNullOrEmpty(screensaver))
                                {
                                    string screensaverName = System.IO.Path.GetFileNameWithoutExtension(screensaver);
                                    items.Add(new SystemInfoItem
                                    {
                                        Category = "전원 관리",
                                        Item = "  └ 화면보호기 종류",
                                        Value = screensaverName
                                    });
                                }

                                // 화면보호기 대기 시간 (초)
                                var timeout = key.GetValue("ScreenSaveTimeOut")?.ToString() ?? "0";
                                if (int.TryParse(timeout, out int timeoutSeconds))
                                {
                                    int minutes = timeoutSeconds / 60;
                                    items.Add(new SystemInfoItem
                                    {
                                        Category = "전원 관리",
                                        Item = "  └ 대기 시간",
                                        Value = $"{minutes}분"
                                    });
                                }

                                // 화면보호기 해제 시 로그온 필요
                                var secureScreensaver = key.GetValue("ScreenSaverIsSecure")?.ToString() ?? "0";
                                bool isSecure = secureScreensaver == "1";
                                items.Add(new SystemInfoItem
                                {
                                    Category = "전원 관리",
                                    Item = "  └ 해제 후 로그온 필요",
                                    Value = isSecure ? "예" : "아니오"
                                });
                            }
                        }
                        else
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "전원 관리",
                                Item = "화면보호기",
                                Value = "레지스트리 접근 불가"
                            });
                        }
                    }
                }
                catch (Exception screensaverEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "화면보호기",
                        Value = $"확인 불가: {screensaverEx.Message}"
                    });
                }

                // 네트워크 어댑터 전원 관리
                try
                {
                    using (var searcher = new ManagementObjectSearcher("SELECT Name, NetEnabled FROM Win32_NetworkAdapter WHERE NetEnabled=TRUE"))
                    {
                        int adapterCount = 0;
                        foreach (ManagementObject obj in searcher.Get())
                        {
                            adapterCount++;
                            var adapterName = obj["Name"]?.ToString() ?? "알 수 없는 어댑터";

                            // 실제 전원 관리 설정은 디바이스 매니저를 통해서만 확인 가능
                            // 여기서는 어댑터가 활성화되어 있음을 표시
                            if (adapterCount <= 3) // 처음 3개만 표시
                            {
                                items.Add(new SystemInfoItem
                                {
                                    Category = "전원 관리",
                                    Item = $"네트워크 어댑터 #{adapterCount}",
                                    Value = $"{adapterName} (활성)"
                                });
                            }
                        }

                        if (adapterCount > 3)
                        {
                            items.Add(new SystemInfoItem
                            {
                                Category = "전원 관리",
                                Item = "추가 네트워크 어댑터",
                                Value = $"외 {adapterCount - 3}개"
                            });
                        }
                    }
                }
                catch (Exception netEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "전원 관리",
                        Item = "네트워크 어댑터",
                        Value = $"확인 불가: {netEx.Message}"
                    });
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "전원 관리",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        private void CollectEventLogInfoToTable(ObservableCollection<SystemInfoItem> items)
        {
            try
            {
                UpdateProgress("이벤트 로그 수집 중...");

                // 절전 관련 이벤트 (최근 7일)
                try
                {
                    var sevenDaysAgo = DateTime.Now.AddDays(-7);
                    var sleepEvents = new List<string>();

                    using (var eventLog = new System.Diagnostics.EventLog("System"))
                    {
                        var entries = eventLog.Entries.Cast<System.Diagnostics.EventLogEntry>()
                            .Where(e => e.TimeGenerated >= sevenDaysAgo &&
                                       (e.InstanceId == 1 || e.InstanceId == 42) && // Sleep/Wake events
                                       e.Source == "Power-Troubleshooter")
                            .OrderByDescending(e => e.TimeGenerated)
                            .Take(10)
                            .ToList();

                        if (entries.Any())
                        {
                            foreach (var entry in entries)
                            {
                                sleepEvents.Add($"[{entry.TimeGenerated:yyyy-MM-dd HH:mm:ss}] {entry.Message.Split('\n')[0]}");
                            }
                        }
                        else
                        {
                            // Kernel-Power 이벤트로도 확인
                            entries = eventLog.Entries.Cast<System.Diagnostics.EventLogEntry>()
                                .Where(e => e.TimeGenerated >= sevenDaysAgo &&
                                           e.Source == "Microsoft-Windows-Kernel-Power" &&
                                           (e.InstanceId == 42 || e.InstanceId == 107))
                                .OrderByDescending(e => e.TimeGenerated)
                                .Take(5)
                                .ToList();

                            foreach (var entry in entries)
                            {
                                string eventType = entry.InstanceId == 42 ? "절전 진입" : "절전 해제";
                                sleepEvents.Add($"[{entry.TimeGenerated:yyyy-MM-dd HH:mm:ss}] {eventType}");
                            }
                        }
                    }

                    if (sleepEvents.Any())
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "이벤트 로그",
                            Item = "절전 이벤트 (최근 7일)",
                            Value = $"{sleepEvents.Count}건 발견"
                        });

                        int eventIndex = 0;
                        foreach (var evt in sleepEvents.Take(5)) // 상위 5개만 표시
                        {
                            eventIndex++;
                            items.Add(new SystemInfoItem
                            {
                                Category = "이벤트 로그",
                                Item = $"  └ 이벤트 #{eventIndex}",
                                Value = evt
                            });
                        }
                    }
                    else
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "이벤트 로그",
                            Item = "절전 이벤트 (최근 7일)",
                            Value = "이벤트 없음"
                        });
                    }
                }
                catch (Exception sleepEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "이벤트 로그",
                        Item = "절전 이벤트",
                        Value = $"확인 불가: {sleepEx.Message}"
                    });
                }

                // 블루스크린 이벤트 (최근 30일)
                try
                {
                    var thirtyDaysAgo = DateTime.Now.AddDays(-30);
                    var bsodEvents = new List<string>();

                    using (var eventLog = new System.Diagnostics.EventLog("System"))
                    {
                        // BugCheck 이벤트 ID: 1001 (Source: BugCheck)
                        // 또는 Event ID: 41 (Source: Kernel-Power) - 시스템 비정상 재부팅
                        var entries = eventLog.Entries.Cast<System.Diagnostics.EventLogEntry>()
                            .Where(e => e.TimeGenerated >= thirtyDaysAgo &&
                                       ((e.InstanceId == 1001 && e.Source == "BugCheck") ||
                                        (e.InstanceId == 41 && e.Source == "Microsoft-Windows-Kernel-Power" && e.EntryType == System.Diagnostics.EventLogEntryType.Error)))
                            .OrderByDescending(e => e.TimeGenerated)
                            .Take(10)
                            .ToList();

                        foreach (var entry in entries)
                        {
                            string eventDesc = entry.Source == "BugCheck" ? "블루스크린" : "비정상 종료";
                            bsodEvents.Add($"[{entry.TimeGenerated:yyyy-MM-dd HH:mm:ss}] {eventDesc}");
                        }
                    }

                    if (bsodEvents.Any())
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "이벤트 로그",
                            Item = "⚠ 블루스크린 (최근 30일)",
                            Value = $"{bsodEvents.Count}건 발견"
                        });

                        int eventIndex = 0;
                        foreach (var evt in bsodEvents.Take(5)) // 상위 5개만 표시
                        {
                            eventIndex++;
                            items.Add(new SystemInfoItem
                            {
                                Category = "이벤트 로그",
                                Item = $"  └ 이벤트 #{eventIndex}",
                                Value = evt
                            });
                        }
                    }
                    else
                    {
                        items.Add(new SystemInfoItem
                        {
                            Category = "이벤트 로그",
                            Item = "블루스크린 (최근 30일)",
                            Value = "✓ 이벤트 없음"
                        });
                    }
                }
                catch (Exception bsodEx)
                {
                    items.Add(new SystemInfoItem
                    {
                        Category = "이벤트 로그",
                        Item = "블루스크린 이벤트",
                        Value = $"확인 불가: {bsodEx.Message}"
                    });
                }
            }
            catch (Exception ex)
            {
                items.Add(new SystemInfoItem
                {
                    Category = "이벤트 로그",
                    Item = "오류",
                    Value = $"정보 수집 실패: {ex.Message}"
                });
            }
        }

        // 테이블 형식 가상화 정보 수집 메소드들
        private void CheckHardwareVirtualizationToTable(ObservableCollection<VirtualizationInfoItem> items)
        {
            try
            {
                UpdateProgress("하드웨어 가상화 지원 확인 중...");
                
                using (var searcher = new ManagementObjectSearcher("SELECT VirtualizationFirmwareEnabled FROM Win32_Processor"))
                {
                    foreach (ManagementObject obj in searcher.Get())
                    {
                        var isEnabled = obj["VirtualizationFirmwareEnabled"];
                        if (isEnabled != null && (bool)isEnabled)
                        {
                            items.Add(new VirtualizationInfoItem
                            {
                                Category = "하드웨어 가상화",
                                Status = "✓ 활성화",
                                Details = "CPU에서 가상화 기술을 지원하며 BIOS/UEFI에서 활성화되어 있습니다",
                                Recommendation = "정상 상태"
                            });
                        }
                        else
                        {
                            items.Add(new VirtualizationInfoItem
                            {
                                Category = "하드웨어 가상화",
                                Status = "✗ 비활성화",
                                Details = "BIOS/UEFI에서 비활성화되어 있거나 지원되지 않습니다",
                                Recommendation = "BIOS/UEFI에서 Intel VT-x 또는 AMD-V 활성화"
                            });
                        }
                        break;
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new VirtualizationInfoItem
                {
                    Category = "하드웨어 가상화",
                    Status = "✗ 오류",
                    Details = $"확인 실패: {ex.Message}",
                    Recommendation = "관리자 권한으로 실행"
                });
            }
        }

        private void CheckWSLInstallationToTable(ObservableCollection<VirtualizationInfoItem> items)
        {
            try
            {
                UpdateProgress("WSL 설치 상태 확인 중...");

                var wslFeatures = new[]
                {
                    ("Microsoft-Windows-Subsystem-Linux", "WSL 1"),
                    ("VirtualMachinePlatform", "가상 머신 플랫폼 (WSL 2)")
                };

                foreach (var (featureName, displayName) in wslFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /get-featureinfo /featurename:{featureName}",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        var output = process.StandardOutput.ReadToEnd();
                        process.WaitForExit();
                        
                        string status, recommendation;
                        if (output.Contains("State : Enabled"))
                        {
                            status = "✓ 설치됨 (활성화)";
                            recommendation = "가상화 성능 영향 가능 - 비활성화 권장";
                        }
                        else if (output.Contains("State : Disabled"))
                        {
                            status = "○ 설치됨 (비활성화)";
                            recommendation = "필요시 활성화";
                        }
                        else
                        {
                            status = "○ 설치되지 않음";
                            recommendation = "필요시 설치";
                        }

                        items.Add(new VirtualizationInfoItem
                        {
                            Category = displayName,
                            Status = status,
                            Details = $"Windows 기능: {featureName}",
                            Recommendation = recommendation
                        });
                    }
                    catch
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = displayName,
                            Status = "? 확인 불가",
                            Details = "상태 확인 실패",
                            Recommendation = "관리자 권한으로 재시도"
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new VirtualizationInfoItem
                {
                    Category = "WSL 상태",
                    Status = "✗ 오류",
                    Details = $"확인 실패: {ex.Message}",
                    Recommendation = "관리자 권한으로 실행"
                });
            }
        }

        private void CheckHyperVInstallationDetailedToTable(ObservableCollection<VirtualizationInfoItem> items)
        {
            try
            {
                UpdateProgress("Hyper-V 설치 상태 확인 중...");

                var hyperVFeatures = new[]
                {
                    ("Microsoft-Hyper-V-All", "Hyper-V (전체)"),
                    ("Microsoft-Hyper-V", "Hyper-V 플랫폼"),
                    ("Microsoft-Hyper-V-Hypervisor", "Hyper-V 하이퍼바이저")
                };

                foreach (var (featureName, displayName) in hyperVFeatures)
                {
                    try
                    {
                        var process = new Process
                        {
                            StartInfo = new ProcessStartInfo
                            {
                                FileName = "dism.exe",
                                Arguments = $"/online /get-featureinfo /featurename:{featureName}",
                                UseShellExecute = false,
                                RedirectStandardOutput = true,
                                RedirectStandardError = true,
                                CreateNoWindow = true
                            }
                        };
                        
                        process.Start();
                        var output = process.StandardOutput.ReadToEnd();
                        process.WaitForExit();
                        
                        string status, recommendation;
                        if (output.Contains("State : Enabled"))
                        {
                            status = "✓ 설치됨 (활성화)";
                            recommendation = "다른 가상화 소프트웨어와 충돌 - 비활성화 권장";
                        }
                        else if (output.Contains("State : Disabled"))
                        {
                            status = "○ 설치됨 (비활성화)";
                            recommendation = "필요시 활성화";
                        }
                        else
                        {
                            status = "○ 설치되지 않음";
                            recommendation = "다른 가상화 소프트웨어 사용 가능";
                        }

                        items.Add(new VirtualizationInfoItem
                        {
                            Category = displayName,
                            Status = status,
                            Details = $"Windows 기능: {featureName}",
                            Recommendation = recommendation
                        });
                    }
                    catch
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = displayName,
                            Status = "? 확인 불가",
                            Details = "상태 확인 실패",
                            Recommendation = "관리자 권한으로 재시도"
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                items.Add(new VirtualizationInfoItem
                {
                    Category = "Hyper-V 상태",
                    Status = "✗ 오류",
                    Details = $"확인 실패: {ex.Message}",
                    Recommendation = "관리자 권한으로 실행"
                });
            }
        }

        private void CheckHypervisorLaunchTypeToTable(ObservableCollection<VirtualizationInfoItem> items)
        {
            try
            {
                UpdateProgress("하이퍼바이저 시작 유형 확인 중...");

                var process = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = "bcdedit.exe",
                        Arguments = "/enum {current}",
                        UseShellExecute = false,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                        CreateNoWindow = true
                    }
                };
                
                process.Start();
                var output = process.StandardOutput.ReadToEnd();
                var error = process.StandardError.ReadToEnd();
                process.WaitForExit();
                
                if (process.ExitCode == 0)
                {
                    if (output.Contains("hypervisorlaunchtype") && output.Contains("Off"))
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = "hypervisorlaunchtype",
                            Status = "✓ Off (비활성화)",
                            Details = "부팅 시 하이퍼바이저가 로드되지 않아 가상화 소프트웨어 사용 가능",
                            Recommendation = "조치 필요없음"
                        });
                    }
                    else if (output.Contains("hypervisorlaunchtype") && output.Contains("Auto"))
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = "hypervisorlaunchtype",
                            Status = "✗ Auto (활성화)",
                            Details = "부팅 시 하이퍼바이저가 자동으로 로드되어 가상화 성능에 영향",
                            Recommendation = "비활성화 필요: 'VBS 및 Hyper-V 비활성화' 기능 사용 권장"
                        });
                    }
                    else
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = "hypervisorlaunchtype",
                            Status = "? 값이 존재하지 않음",
                            Details = "Windows 기본 설정 (보통 Auto와 동일하게 작동)",
                            Recommendation = "값이 존재하지 않음"
                        });
                    }
                }
                else
                {
                    items.Add(new VirtualizationInfoItem
                    {
                        Category = "hypervisorlaunchtype",
                        Status = "✗ 확인 실패",
                        Details = $"bcdedit 명령 실행 실패 (Exit Code: {process.ExitCode})",
                        Recommendation = "관리자 권한이 필요할 수 있습니다"
                    });
                }
            }
            catch (Exception ex)
            {
                items.Add(new VirtualizationInfoItem
                {
                    Category = "hypervisorlaunchtype",
                    Status = "✗ 오류",
                    Details = $"확인 실패: {ex.Message}",
                    Recommendation = "관리자 권한으로 실행"
                });
            }
        }

        private void CheckVBSStatusDetailedToTable(ObservableCollection<VirtualizationInfoItem> items)
        {
            try
            {
                UpdateProgress("VBS 상태 및 레지스트리 확인 중...");

                var vbsRegistryKeys = new[]
                {
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "EnableVirtualizationBasedSecurity", "VBS 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard", "RequirePlatformSecurityFeatures", "플랫폼 보안 기능 요구"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled", "HVCI 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "WasEnabledBy", "HVCI 활성화 주체"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard", "Enabled", "시스템 가드 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard", "Enabled", "Credential Guard 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "EncryptionKeyAvailable", "암호화 키 사용 가능"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "EncryptionKeyPersistent", "암호화 키 지속성"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "IsSecureKernelRunning", "보안 커널 실행 상태"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "KeyGuardEnabled", "Key Guard 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\KeyGuard\Status", "SecretsMode", "비밀 모드"),
                    (@"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SecureBiometrics", "Enabled", "보안 생체 인식 활성화"),
                    (@"SYSTEM\CurrentControlSet\Control\Lsa", "LsaCfgFlags", "LSA 구성 플래그"),
                    (@"SYSTEM\CurrentControlSet\Control\Lsa", "LsaCfgFlagsDefault", "LSA 기본 구성 플래그"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "EnableVirtualizationBasedSecurity", "VBS 정책 활성화"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HypervisorEnforcedCodeIntegrity", "정책 HVCI 활성화"),
                    (@"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HVCIMATRequired", "HVCI MAT 요구"),
                    (@"SYSTEM\CurrentControlSet\Control\CI\Config", "VulnerableDriverBlocklistEnable", "취약한 드라이버 차단")
                };

                bool vbsEnabled = false;

                foreach (var (keyPath, valueName, description) in vbsRegistryKeys)
                {
                    try
                    {
                        using (var key = Registry.LocalMachine.OpenSubKey(keyPath))
                        {
                            if (key != null)
                            {
                                var value = key.GetValue(valueName);
                                if (value != null)
                                {
                                    string valueStr = value.ToString();
                                    string status = valueStr == "1" ? "✗ 활성화" : "✓ 비활성화";
                                    string recommendation = valueStr == "1" ? "'VBS 및 Hyper-V 비활성화' 기능 사용 권장" : "현재 설정이 올바름";
                                    
                                    if (valueStr == "1" && (valueName.Contains("Enable") || valueName == "Enabled"))
                                    {
                                        vbsEnabled = true;
                                    }
                                    
                                    items.Add(new VirtualizationInfoItem
                                    {
                                        Category = description,
                                        Status = status,
                                        Details = $"레지스트리 값: {valueStr} (경로: HKLM\\{keyPath})",
                                        Recommendation = recommendation
                                    });
                                }
                                else
                                {
                                    items.Add(new VirtualizationInfoItem
                                    {
                                        Category = description,
                                        Status = "○ 설정되지 않음",
                                        Details = $"레지스트리: HKLM\\{keyPath}\\{valueName}",
                                        Recommendation = "기본값 사용 중"
                                    });
                                }
                            }
                            else
                            {
                                items.Add(new VirtualizationInfoItem
                                {
                                    Category = description,
                                    Status = "○ 키 존재하지 않음",
                                    Details = $"레지스트리: HKLM\\{keyPath}",
                                    Recommendation = "기본값 사용 중"
                                });
                            }
                        }
                    }
                    catch (Exception regEx)
                    {
                        items.Add(new VirtualizationInfoItem
                        {
                            Category = description,
                            Status = "✗ 확인 실패",
                            Details = $"오류: {regEx.Message}",
                            Recommendation = "관리자 권한으로 재시도"
                        });
                    }
                }

                // 종합 상태
                items.Add(new VirtualizationInfoItem
                {
                    Category = "VBS 종합 상태",
                    Status = vbsEnabled ? "✗ VBS 활성화됨" : "✓ VBS 비활성화됨",
                    Details = vbsEnabled ? "가상화 성능에 영향을 줍니다" : "가상화 성능에 영향을 주지 않습니다",
                    Recommendation = vbsEnabled ? "'VBS 및 Hyper-V 비활성화' 기능 사용 권장" : "현재 설정이 올바름"
                });
            }
            catch (Exception ex)
            {
                items.Add(new VirtualizationInfoItem
                {
                    Category = "VBS 상태",
                    Status = "✗ 오류",
                    Details = $"확인 실패: {ex.Message}",
                    Recommendation = "관리자 권한으로 실행"
                });
            }
        }

        private void ExportSystemInfoButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                var saveFileDialog = new SaveFileDialog
                {
                    Filter = "CSV 파일 (*.csv)|*.csv|텍스트 파일 (*.txt)|*.txt",
                    DefaultExt = "csv",
                    FileName = $"시스템정보_{DateTime.Now:yyyy-MM-dd_HH-mm-ss}"
                };

                if (saveFileDialog.ShowDialog() == true)
                {
                    var systemInfoItems = SystemInfoDataGrid.ItemsSource as ObservableCollection<SystemInfoItem>;
                    if (systemInfoItems != null)
                    {
                        ExportToFile(systemInfoItems.ToList(), saveFileDialog.FileName, "시스템 상세 정보");
                        MessageBox.Show($"시스템 정보를 성공적으로 내보냈습니다.\n파일: {saveFileDialog.FileName}", 
                                      "내보내기 완료", MessageBoxButton.OK, MessageBoxImage.Information);
                    }
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"내보내기 중 오류가 발생했습니다: {ex.Message}", 
                              "오류", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void ExportVirtualizationInfoButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                var saveFileDialog = new SaveFileDialog
                {
                    Filter = "CSV 파일 (*.csv)|*.csv|텍스트 파일 (*.txt)|*.txt",
                    DefaultExt = "csv",
                    FileName = $"가상화설정_{DateTime.Now:yyyy-MM-dd_HH-mm-ss}"
                };

                if (saveFileDialog.ShowDialog() == true)
                {
                    var virtualizationInfoItems = VirtualizationDataGrid.ItemsSource as ObservableCollection<VirtualizationInfoItem>;
                    if (virtualizationInfoItems != null)
                    {
                        ExportToFile(virtualizationInfoItems.ToList(), saveFileDialog.FileName, "가상화 설정 점검");
                        MessageBox.Show($"가상화 설정 정보를 성공적으로 내보냈습니다.\n파일: {saveFileDialog.FileName}", 
                                      "내보내기 완료", MessageBoxButton.OK, MessageBoxImage.Information);
                    }
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"내보내기 중 오류가 발생했습니다: {ex.Message}", 
                              "오류", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void ExportToFile<T>(List<T> items, string filePath, string title)
        {
            var sb = new StringBuilder();
            var extension = Path.GetExtension(filePath).ToLower();
            
            // 헤더 정보
            sb.AppendLine($"# {title}");
            sb.AppendLine($"# 생성일시: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
            sb.AppendLine($"# VM Compatibility Tool v1.2.1");
            sb.AppendLine("");

            if (typeof(T) == typeof(SystemInfoItem))
            {
                // 시스템 정보 헤더
                sb.AppendLine("항목,세부 정보,값");
                
                foreach (var item in items.Cast<SystemInfoItem>())
                {
                    // CSV 형식으로 쉼표로 구분, 쉼표가 포함된 내용은 따옴표로 감싸기
                    var category = EscapeCsvField(item.Category ?? "");
                    var itemName = EscapeCsvField(item.Item ?? "");
                    var value = EscapeCsvField(item.Value ?? "");
                    
                    sb.AppendLine($"{category},{itemName},{value}");
                }
            }
            else if (typeof(T) == typeof(VirtualizationInfoItem))
            {
                // 가상화 정보 헤더
                sb.AppendLine("항목,상태,상세 정보,권장사항");
                
                foreach (var item in items.Cast<VirtualizationInfoItem>())
                {
                    var category = EscapeCsvField(item.Category ?? "");
                    var status = EscapeCsvField(item.Status ?? "");
                    var details = EscapeCsvField(item.Details ?? "");
                    var recommendation = EscapeCsvField(item.Recommendation ?? "");
                    
                    sb.AppendLine($"{category},{status},{details},{recommendation}");
                }
            }

            // UTF-8 BOM을 추가하여 한글 깨짐 방지
            var encoding = new UTF8Encoding(true);
            File.WriteAllText(filePath, sb.ToString(), encoding);
        }

        private string EscapeCsvField(string field)
        {
            if (string.IsNullOrEmpty(field))
                return "";

            // 쉼표, 줄바꿈, 따옴표가 포함된 경우 따옴표로 감싸고 내부 따옴표는 두 개로 변환
            if (field.Contains(",") || field.Contains("\n") || field.Contains("\"") || field.Contains("\r"))
            {
                field = field.Replace("\"", "\"\"");
                return $"\"{field}\"";
            }
            
            return field;
        }

    }
}