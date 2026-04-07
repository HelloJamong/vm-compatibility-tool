// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    check_admin_or_exit();

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
