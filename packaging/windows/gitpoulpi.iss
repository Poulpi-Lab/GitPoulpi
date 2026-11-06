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
; .gitpoulpi-dist marker) and, after the copy, rewrites the marker with the
; install channel: inno-user or inno-machine by the privileges the user chose,
; or whatever /CHANNEL= says (Chocolatey, winget). /FEED=<url> adds the feed
; line an internalised Chocolatey package needs. The channel, not the marker's
; presence, decides how GitPoulpi updates (the install-channels design). Every
; other choice is made by the installed `gitpoulpi.exe`
; (`desktop shortcut install`, `path add`, `alias create`, `uninstall`), the
; same code as Settings > General > Launcher, so the installer and Settings
; never disagree. Per-user by default (no admin); the first dialog lets the
; user switch to an all-users install in Program Files, where an update asks
; for administrator approval. The command line picks the scope with /CURRENTUSER
; or /ALLUSERS, and a later upgrade reuses the earlier choice
; (UsePreviousPrivileges).

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
PrivilegesRequiredOverridesAllowed=commandline dialog
UsePreviousPrivileges=yes
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

[Messages]
PrivilegesRequiredOverrideText1=Install GitPoulpi for all users (in Program Files, updates ask for administrator approval) or only for you.
PrivilegesRequiredOverrideText2=Install GitPoulpi only for you (no administrator rights, updates in place) or for all users.
PrivilegesRequiredOverrideAllUsers=For all users, in Program Files (updates ask for administrator approval)
PrivilegesRequiredOverrideAllUsersRecommended=For all users, in Program Files (recommended; updates ask for administrator approval)
PrivilegesRequiredOverrideCurrentUser=Only for me (no administrator rights; updates in place)
PrivilegesRequiredOverrideCurrentUserRecommended=Only for me (recommended; no administrator rights; updates in place)

[Tasks]
Name: "startmenu"; Description: "Create a &Start menu shortcut"
Name: "desktopicon"; Description: "Create a &Desktop shortcut"; Flags: unchecked
Name: "addtopath"; Description: "Add GitPoulpi to the &PATH (for new terminals)"

[Files]
Source: "{#SourceDir}\gitpoulpi.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-tui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-mcp.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-launch.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\gitpoulpi-updater.exe"; DestDir: "{app}"; Flags: ignoreversion
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
  AliasPage.Values[0] := ExpandConstant('{param:ALIAS|gp}');
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

{ The channel this install records in the marker: /CHANNEL= when it is a plain
  lower-case name (Chocolatey and winget pass theirs), else by the privileges
  the user chose. }
function GetChannel: String;
var
  I: Integer;
begin
  Result := Trim(ExpandConstant('{param:CHANNEL|}'));
  for I := 1 to Length(Result) do
    if not (((Result[I] >= 'a') and (Result[I] <= 'z')) or (Result[I] = '-')) then
    begin
      Result := '';
      Break;
    end;
  if Result = '' then
  begin
    if IsAdminInstallMode then
      Result := 'inno-machine'
    else
      Result := 'inno-user';
  end;
end;

{ The archive's marker says channel=archive; the installed one names the real
  channel, and the feed an internalised Chocolatey package was built for. }
procedure WriteMarker;
var
  Text, Feed: String;
begin
  Text := 'version={#AppVersion}' + #10 + 'target=x86_64-pc-windows-msvc' + #10 +
    'channel=' + GetChannel + #10;
  Feed := Trim(ExpandConstant('{param:FEED|}'));
  if (Feed <> '') and (Pos(#10, Feed) = 0) and (Pos(#13, Feed) = 0) then
    Text := Text + 'feed=' + Feed + #10;
  if not SaveStringToFile(ExpandConstant('{app}\.gitpoulpi-dist'), Text, False) then
    Log('Could not write the install marker');
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    WriteMarker;
end;
