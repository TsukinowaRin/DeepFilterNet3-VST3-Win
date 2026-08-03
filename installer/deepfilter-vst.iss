#ifndef MyAppVersion
  #define MyAppVersion "dev"
#endif
#ifndef SourceDir
  #define SourceDir "..\target\bundled"
#endif
#ifndef RepoRoot
  #define RepoRoot ".."
#endif
#ifndef OutputDir
  #define OutputDir "..\dist"
#endif
#ifndef OutputBaseFilename
  #define OutputBaseFilename "DeepFilterNet3-VST3-Win-dev-windows-x86_64-setup"
#endif

[Setup]
AppId={{6ACC1E2B-2F42-40A4-8330-3930DC8B0FB8}
AppName=DeepFilterNet3 VST3
AppVersion={#MyAppVersion}
AppVerName=DeepFilterNet3 VST3 {#MyAppVersion}
AppPublisher=TsukinowaRin
AppPublisherURL=https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win
AppSupportURL=https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win/issues
AppUpdatesURL=https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win/releases
DefaultDirName={autopf}\DeepFilterNet3 VST3
DefaultGroupName=DeepFilterNet3 VST3
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
SetupLogging=yes
Uninstallable=yes
UninstallDisplayName=DeepFilterNet3 VST3
UninstallDisplayIcon={uninstallexe}
; Do not restore previous install's task choices; wizard must always default both formats ON.
UsePreviousTasks=no

[Tasks]
; Independent format checkboxes; default checked (no checkedonce/unchecked) so both are always initially selected.
Name: "installvst3"; Description: "Install VST3 plug-in"; GroupDescription: "Plug-in formats:"
Name: "installclap"; Description: "Install CLAP plug-in"; GroupDescription: "Plug-in formats:"

[Files]
Source: "{#SourceDir}\deepfilter-vst.vst3\*"; DestDir: "{commoncf64}\VST3\deepfilter-vst.vst3"; Flags: ignoreversion recursesubdirs createallsubdirs; Tasks: installvst3
Source: "{#SourceDir}\deepfilter-vst.clap"; DestDir: "{commoncf64}\CLAP"; Flags: ignoreversion; Tasks: installclap
Source: "{#RepoRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\README_ja.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\DeepFilterNet3 VST3\Uninstall DeepFilterNet3 VST3"; Filename: "{uninstallexe}"

[Code]
function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = wpSelectTasks then
  begin
    if (not WizardIsTaskSelected('installvst3')) and (not WizardIsTaskSelected('installclap')) then
    begin
      MsgBox(
        'Select at least one plug-in format.'#13#10
        'Check Install VST3 plug-in and/or Install CLAP plug-in.',
        mbError,
        MB_OK);
      Result := False;
    end;
  end;
end;
