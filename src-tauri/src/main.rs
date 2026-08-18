// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // windows_subsystem = "windows" 빌드는 콘솔이 없어 panic 메시지가 그대로 사라짐 —
    // 다음 발생 시 원인 추적이 가능하도록 파일에 남긴다.
    std::panic::set_hook(Box::new(|info| {
        vm_compatibility_tool_lib::services::log_service::init();
        vm_compatibility_tool_lib::services::log_service::log_error("panic", &info.to_string());
    }));

    #[cfg(windows)]
    check_admin_or_exit();
    #[cfg(windows)]
    check_webview2_or_exit();

    vm_compatibility_tool_lib::run();
}

/// 관리자 권한 확인 — 없으면 안내 메시지 후 종료
#[cfg(windows)]
fn check_admin_or_exit() {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let is_admin = unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            false
        } else {
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                std::ptr::addr_of_mut!(elevation).cast(),
                size,
                &mut size,
            ) != 0;
            CloseHandle(token);
            ok && elevation.TokenIsElevated != 0
        }
    };

    if !is_admin {
        let title: Vec<u16> = "관리자 권한 필요\0".encode_utf16().collect();
        let msg: Vec<u16> = concat!(
            "이 프로그램은 관리자 권한이 필요합니다.\n\n",
            "프로그램을 우클릭하여 '관리자 권한으로 실행'을\n",
            "선택한 후 다시 실행해주세요.\0"
        )
        .encode_utf16()
        .collect();
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                msg.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}

/// WebView2 런타임 설치 여부 확인 — 없거나 손상된 상태에서 Tauri가
/// 창을 만들다 불명확하게 죽는 대신, 원인을 알 수 있는 안내 메시지를 띄우고 종료
#[cfg(windows)]
fn check_webview2_or_exit() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    use winreg::{
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
        RegKey,
    };

    // Evergreen WebView2 Runtime 클라이언트 GUID (Microsoft 공식 감지 키)
    const CLIENT_KEY: &str =
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const CLIENT_KEY_WOW64: &str =
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    fn has_pv(root: RegKey, path: &str) -> bool {
        root.open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("pv"))
            .map(|v| !v.is_empty() && v != "0.0.0.0")
            .unwrap_or(false)
    }

    let installed = has_pv(RegKey::predef(HKEY_LOCAL_MACHINE), CLIENT_KEY_WOW64)
        || has_pv(RegKey::predef(HKEY_LOCAL_MACHINE), CLIENT_KEY)
        || has_pv(RegKey::predef(HKEY_CURRENT_USER), CLIENT_KEY);

    if !installed {
        let title: Vec<u16> = "WebView2 런타임 필요\0".encode_utf16().collect();
        let msg: Vec<u16> = concat!(
            "이 프로그램을 실행하려면 Microsoft Edge WebView2 런타임이 필요합니다.\n\n",
            "Windows Update를 실행하거나 관리자에게 문의하여\n",
            "WebView2 런타임을 설치한 후 다시 실행해주세요.\0"
        )
        .encode_utf16()
        .collect();
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                msg.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}
