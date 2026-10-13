; installer/souls_engine_setup.iss — Script de Compilação do Instalador Inno Setup para Souls Engine v7
; Gera o executável SoulsEngine_Setup_v7.exe para instalação no Windows 11 64-bit

#define MyAppName "Souls Engine"
#define MyAppVersion "7.2.0"
#define MyAppPublisher "Souls Engine Bare-Metal Team"
#define MyAppURL "http://127.0.0.1:9123/dashboard"
#define MyAppExeName "souls_engine.exe"

[Setup]
AppId={{5D9A95C1-306B-4D2A-B962-972E5E8A9123}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\SoulsEngine
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Não requer elevação de Administrador (instalação limpa em %LOCALAPPDATA%)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Z:\souls_engine\installer_output
OutputBaseFilename=SoulsEngine_Setup_v7
SetupIconFile=Z:\souls_engine\assets\branding\souls_icon_cyan.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
DisableDirPage=no
DisableProgramGroupPage=yes

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Binários Nativos Release
Source: "Z:\souls_engine\target\release\souls_engine.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "Z:\souls_engine\target\release\souls_server.exe"; DestDir: "{app}"; Flags: ignoreversion
; Scripts e Launchers Operacionais
Source: "Z:\souls_engine\tools\souls_launcher.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "Z:\souls_engine\assets\branding\*"; DestDir: "{app}\assets\branding"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; Atalho no Menu Iniciar
Name: "{userprograms}\{#MyAppName}"; Filename: "pwsh.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\souls_launcher.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\assets\branding\souls_icon_cyan.ico"; Comment: "Souls Engine v7 Bare-Metal Companion & Daemon"
; Atalho na Área de Trabalho
Name: "{userdesktop}\{#MyAppName}"; Filename: "pwsh.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\souls_launcher.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\assets\branding\souls_icon_cyan.ico"; Comment: "Souls Engine v7 Bare-Metal Companion & Daemon"; Tasks: desktopicon

[Run]
; Opcional: Rodar o companion ao término da instalação
Filename: "pwsh.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\souls_launcher.ps1"""; Description: "Iniciar o Souls Engine agora"; Flags: nowait postinstall skipifsilent

[Code]
// Auto-montagem do VHDX Dev Drive ReFS caso Z: esteja desmontado
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then
  begin
    // Monta VHDX se Z:\ não estiver disponível
    if not DirExists('Z:\') then
    begin
      if FileExists('C:\dev_projects\Souls_FORGE.vhdx') then
      begin
        Exec('powershell.exe', '-NoProfile -Command "Mount-DiskImage -ImagePath ''C:\dev_projects\Souls_FORGE.vhdx'' -NoDriveLetter:$false"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
      end;
    end;

    // Executa a injeção no Hermes Agent e linkagem de plugins
    if FileExists(ExpandConstant('{app}\..\..\hermes\config.yaml')) then
    begin
      Exec('powershell.exe', ExpandConstant('-NoProfile -ExecutionPolicy Bypass -File "Z:\souls_engine\tools\install_souls.ps1" -SkipBuild'), '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;
