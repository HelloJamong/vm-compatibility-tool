fn main() {
    // Windows 관리자 권한 manifest 임베드
    #[cfg(target_os = "windows")]
    embed_manifest();

    tauri_build::build()
}

#[cfg(target_os = "windows")]
fn embed_manifest() {
    // winresource로 매니페스트 임베드
    // (winresource crate 추가 시 활성화)
    // let mut res = winresource::WindowsResource::new();
    // res.set_manifest(ADMIN_MANIFEST);
    // res.compile().unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

#[allow(dead_code)]
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
</assembly>"#;
