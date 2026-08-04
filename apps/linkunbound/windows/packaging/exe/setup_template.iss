[Setup]
AppId={{APP_ID}}
AppVersion={{APP_VERSION}}
AppName={{DISPLAY_NAME}}
UninstallDisplayName={{DISPLAY_NAME}}
AppPublisher={{PUBLISHER_NAME}}
AppPublisherURL={{PUBLISHER_URL}}
AppSupportURL={{PUBLISHER_URL}}
DefaultDirName={{INSTALL_DIR_NAME}}
DisableProgramGroupPage=yes
OutputBaseFilename={{OUTPUT_BASE_FILENAME}}
SetupIconFile={{SETUP_ICON_FILE}}
UninstallDisplayIcon={app}\{{EXECUTABLE_NAME}},0
Compression=lzma
SolidCompression=yes
PrivilegesRequired={{PRIVILEGES_REQUIRED}}
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
{% for locale in LOCALES %}
{% if locale == 'en' %}Name: "english"; MessagesFile: "compiler:Default.isl"{% endif %}
{% if locale == 'es' %}Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"{% endif %}
{% endfor %}

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{{SOURCE_DIR}}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{{DISPLAY_NAME}}"; Filename: "{app}\{{EXECUTABLE_NAME}}"
Name: "{autodesktop}\{{DISPLAY_NAME}}"; Filename: "{app}\{{EXECUTABLE_NAME}}"; Tasks: desktopicon

[Registry]
; ProgID — URL handler
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL"; ValueType: string; ValueData: "LinkUnbound URL"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL"; ValueName: "EditFlags"; ValueType: dword; ValueData: "2"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL"; ValueName: "FriendlyTypeName"; ValueType: string; ValueData: "LinkUnbound URL"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL\Application"; ValueName: "ApplicationName"; ValueType: string; ValueData: "LinkUnbound"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL\Application"; ValueName: "ApplicationDescription"; ValueType: string; ValueData: "Browser picker for Windows"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL\Application"; ValueName: "ApplicationIcon"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"",0"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL\DefaultIcon"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"",0"
Root: HKLM; Subkey: "SOFTWARE\Classes\LinkUnboundURL\shell\open\command"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"" ""%1"""

; StartMenuInternet — browser declaration
Root: HKLM; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound"; ValueType: string; ValueData: "LinkUnbound"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound\DefaultIcon"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"",0"
Root: HKLM; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound\shell\open\command"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"""
Root: HKLM; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound\InstallInfo"; ValueName: "ReinstallCommand"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"""
Root: HKLM; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound\InstallInfo"; ValueName: "IconsVisible"; ValueType: dword; ValueData: "1"

; Capabilities
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities"; ValueName: "ApplicationName"; ValueType: string; ValueData: "LinkUnbound"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities"; ValueName: "ApplicationDescription"; ValueType: string; ValueData: "Browser picker for Windows"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities"; ValueName: "ApplicationIcon"; ValueType: string; ValueData: """{app}\{{EXECUTABLE_NAME}}"",0"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\Startmenu"; ValueName: "StartMenuInternet"; ValueType: string; ValueData: "LinkUnbound"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\URLAssociations"; ValueName: "http"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\URLAssociations"; ValueName: "https"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".htm"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".html"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".xhtml"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".xht"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".pdf"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".svg"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".mhtml"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".mht"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".shtml"; ValueType: string; ValueData: "LinkUnboundURL"
Root: HKLM; Subkey: "SOFTWARE\LinkUnbound\Capabilities\FileAssociations"; ValueName: ".webp"; ValueType: string; ValueData: "LinkUnboundURL"

; Advertise our ProgID as a candidate handler for these file types so they
; appear in the per-app default association page in Windows Settings.
Root: HKLM; Subkey: "SOFTWARE\Classes\.htm\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.html\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.xhtml\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.xht\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.pdf\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.svg\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.mhtml\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.mht\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.shtml\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""
Root: HKLM; Subkey: "SOFTWARE\Classes\.webp\OpenWithProgIds"; ValueName: "LinkUnboundURL"; ValueType: string; ValueData: ""

; RegisteredApplications (only delete value, not the shared key)
Root: HKLM; Subkey: "SOFTWARE\RegisteredApplications"; ValueName: "LinkUnbound"; ValueType: string; ValueData: "SOFTWARE\LinkUnbound\Capabilities"; Flags: uninsdeletevalue

; HKCU cleanup — the app writes these at runtime; remove on uninstall
Root: HKCU; Subkey: "SOFTWARE\Classes\LinkUnboundURL"; Flags: uninsdeletekey dontcreatekey
Root: HKCU; Subkey: "SOFTWARE\Clients\StartMenuInternet\LinkUnbound"; Flags: uninsdeletekey dontcreatekey
Root: HKCU; Subkey: "SOFTWARE\LinkUnbound"; Flags: uninsdeletekey dontcreatekey
Root: HKCU; Subkey: "SOFTWARE\RegisteredApplications"; ValueName: "LinkUnbound"; Flags: uninsdeletevalue dontcreatekey

[Run]
; runasoriginaluser is mandatory here: the installer requires admin, and without
; it the app inherits the elevated token. Its single-instance mutex and IPC pipe
; would then live at high integrity, unreachable from the medium-integrity
; processes that actually open links (Slack, Teams, Explorer) — every click
; would be silently dropped. It also keeps per-user registry writes in the
; signed-in user's hive rather than the admin's.
Filename: "{app}\{{EXECUTABLE_NAME}}"; Description: "{cm:LaunchProgram,{{DISPLAY_NAME}}}"; Flags: nowait postinstall skipifsilent runasoriginaluser

[Code]
const
  SHCNE_ASSOCCHANGED = $08000000;
  SHCNF_IDLIST = $0000;

procedure SHChangeNotify(wEventId, uFlags: Cardinal; dwItem1, dwItem2: Cardinal);
  external 'SHChangeNotify@shell32.dll stdcall';

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, 0, 0);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, 0, 0);
end;
