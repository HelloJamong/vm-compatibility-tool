fn main() {
    let mut attrs = tauri_build::Attributes::new();

    #[cfg(target_os = "windows")]
    {
        attrs = attrs.windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(ADMIN_MANIFEST),
        );
    }

    tauri_build::try_build(attrs).expect("failed to run tauri-build");

    // bundle.active=false 시 tauri-build가 아이콘을 임베딩하지 않으므로 직접 처리
    #[cfg(target_os = "windows")]
    {
        winresource::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .compile()
            .expect("아이콘 리소스 임베딩 실패");
    }
}

/// requireAdministrator manifest
///
/// - Microsoft.Windows.Common-Controls v6 포함 필수:
///   Tauri가 TaskDialogIndirect를 사용하므로 활성화 컨텍스트가 없으면
///   "프로시저 시작 지점을 찾을 수 없습니다" 오류 발생
const ADMIN_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10/11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>"#;
