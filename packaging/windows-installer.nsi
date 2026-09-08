Unicode true

!ifndef PROJECT_ROOT
  !error "PROJECT_ROOT is required"
!endif
!ifndef VERSION
  !error "VERSION is required"
!endif
!ifndef OUTPUT_FILE
  !error "OUTPUT_FILE is required"
!endif

Name "CodexLight"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\CodexLight"
InstallDirRegKey HKCU "Software\CodexLight" "InstallDir"
RequestExecutionLevel user

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "CodexLight"
VIAddVersionKey "FileDescription" "CodexLight Windows x64 installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "CodexLight" SEC_MAIN
  SetOutPath "$INSTDIR"
  File /oname=codex-light.exe "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/codex-light.exe"

  WriteRegStr HKCU "Software\CodexLight" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight" "DisplayName" "CodexLight"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight" "Publisher" "CodexLight"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight" "UninstallString" '"$INSTDIR\Uninstall.exe"'

  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\CodexLight"
  CreateShortcut "$SMPROGRAMS\CodexLight\CodexLight.lnk" "$INSTDIR\codex-light.exe"
  CreateShortcut "$SMPROGRAMS\CodexLight\Uninstall CodexLight.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\CodexLight\CodexLight.lnk"
  Delete "$SMPROGRAMS\CodexLight\Uninstall CodexLight.lnk"
  RMDir "$SMPROGRAMS\CodexLight"
  Delete "$INSTDIR\codex-light.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexLight"
  DeleteRegKey HKCU "Software\CodexLight"
SectionEnd

