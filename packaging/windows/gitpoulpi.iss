; GitPoulpi installer for Windows (Inno Setup 6).
;
; Built by .github/workflows/release.yml from the release zip, extracted into
; SourceDir:
;
;   ISCC.exe /DAppVersion=0.2.0 /DSourceDir=C:\path\to\gitpoulpi-0.2.0-x86_64-pc-windows-msvc /DOutputDir=C:\out gitpoulpi.iss
;
; It writes gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe.
;
; Design: the installer only copies the files (the exes and the
; .gitpoulpi-dist marker, which keeps the in-app self-updater working: it
; replaces the exes in place, keeping .old copies that the uninstaller also
; removes). Every other choice is made by the installed `gitpoulpi.exe`
; (`desktop shortcut install`, `path add`, `alias create`, `uninstall`), the
; same code as Settings > General > Launcher, so the installer and Settings
; never disagree. Per-user by default (no admin); the first dialog lets the
; user switch to an all-users install in Program Files, where the updater
; cannot replace files without elevation.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #define SourceDir "dist-files"
#endif
#ifndef OutputDir
  #define OutputDir "Output"
#endif

[Setup]
AppId={{8C1D3E5A-6B7F-4A29-9E0C-47A1B2C3D4E5}
AppName=GitPoulpi
AppVersion={#AppVersion}
AppPublisher=GitPoulpi
AppPublisherURL=https://github.com/Poulpi-Lab/GitPoulpi
DefaultDirName={autopf}\GitPoulpi
DefaultGroupName=GitPoulpi
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
CloseApplications=yes
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName=GitPoulpi
UninstallDisplayIcon={app}\gitpoulpi-tui.exe
OutputDir={#OutputDir}
OutputBaseFilename=gitpoulpi-{#AppVersion}-x86_64-pc-windows-msvc-setup

[Tasks]
Name: "startmenu"; Description: "Create a &Start menu shortcut"
Name: "desktopicon"; Description: "Create a &Desktop shortcut"; Flags: unchecked
Name: "addtopath"; Description: "Add GitPoulpi to the &PATH (for new terminals)"

[Files]
Source: "{#SourceDir}\gitpoulpi.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-tui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-mcp.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-launch.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\.gitpoulpi-dist"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\INSTALL.txt"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Run]
Filename: "{app}\gitpoulpi.exe"; Parameters: "desktop shortcut install --where startmenu"; Tasks: startmenu; Flags: runhidden waituntilterminated; StatusMsg: "Creating the Start menu shortcut..."
Filename: "{app}\gitpoulpi.exe"; Parameters: "desktop shortcut install --where desktop"; Tasks: desktopicon; Flags: runhidden waituntilterminated; StatusMsg: "Creating the Desktop shortcut..."
Filename: "{app}\gitpoulpi.exe"; Parameters: "path add"; Tasks: addtopath; Flags: runhidden waituntilterminated; StatusMsg: "Adding GitPoulpi to the PATH..."
Filename: "{app}\gitpoulpi.exe"; Parameters: "alias create {code:GetAliasName}"; Check: HasAlias; Flags: runhidden waituntilterminated; StatusMsg: "Creating the command alias..."

[UninstallRun]
; Removes the alias, the PATH entries and the shortcuts the same way Settings
; does. The files are left to the uninstaller (a running exe cannot delete
; itself), which also deletes the updater's leftovers below.
Filename: "{app}\gitpoulpi.exe"; Parameters: "uninstall --keep-files"; RunOnceId: "GitPoulpiUninstall"; Flags: runhidden waituntilterminated skipifdoesntexist

[UninstallDelete]
Type: files; Name: "{app}\*.old"
Type: files; Name: "{app}\*.new"
Type: files; Name: "{app}\*.swap"
Type: files; Name: "{app}\.gitpoulpi-dist.old"
Type: files; Name: "{app}\.gitpoulpi-dist.new"
Type: dirifempty; Name: "{app}\bin"
Type: dirifempty; Name: "{app}"

[Code]
var
  AliasPage: TInputQueryWizardPage;

procedure InitializeWizard;
begin
  AliasPage := CreateInputQueryPage(wpSelectTasks,
    'Command alias',
    'Which command should start GitPoulpi?',
    'Type a short name to type in a terminal, for example gp. Leave it empty for none. ' +
    'Windows PowerShell already uses gp for Get-ItemProperty: pick another name there, ' +
    'or use gitpoulpi.');
  AliasPage.Add('Command alias:', False);
  AliasPage.Values[0] := 'gp';
end;

function GetAliasName(Param: String): String;
begin
  Result := Trim(AliasPage.Values[0]);
end;

function HasAlias: Boolean;
begin
  Result := GetAliasName('') <> '';
end;

function AliasNameIsValid(Name: String): Boolean;
var
  I: Integer;
  C: Char;
begin
  Result := True;
  if Name = '' then
    Exit;
  if (Name[1] = '-') or (Name[1] = '.') then
  begin
    Result := False;
    Exit;
  end;
  for I := 1 to Length(Name) do
  begin
    C := Name[I];
    if not (((C >= 'a') and (C <= 'z')) or ((C >= 'A') and (C <= 'Z')) or
            ((C >= '0') and (C <= '9')) or (C = '-') or (C = '_') or (C = '.')) then
    begin
      Result := False;
      Exit;
    end;
  end;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if (CurPageID = AliasPage.ID) and not AliasNameIsValid(GetAliasName('')) then
  begin
    MsgBox('Use only letters, digits, - _ and . in the command alias, and do not start it with - or .',
      mbError, MB_OK);
    Result := False;
  end;
end;
