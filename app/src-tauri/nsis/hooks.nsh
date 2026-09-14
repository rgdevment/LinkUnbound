!macro NSIS_HOOK_PREINSTALL
  ; The program goes under Programs, never into the folder the data lives in: on Windows
  ; $LOCALAPPDATA\LinkUnbound and $LOCALAPPDATA\linkunbound are one folder, so the default would
  ; put the binaries where every process of this user can write beside them — and the resident is
  ; what the shell runs for each link. Only when nothing is installed yet: an existing install
  ; keeps its place, or the old copy would be orphaned where the data lives.
  ReadRegStr $0 SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
  ${If} $0 == ""
  ${AndIf} $INSTDIR == "$LOCALAPPDATA\${PRODUCTNAME}"
    StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    SetOutPath $INSTDIR
  ${EndIf}

  ; The template waits for the main binary, which here is only the settings window. The resident
  ; is what the shell runs for every link, it is mapped all day, and a locked
  ; linkunbound-shell.exe leaves the install half done with the registration pointing at a file
  ; that was never replaced.
  !insertmacro CheckIfAppIsRunning "linkunbound-shell.exe" "${PRODUCTNAME}"

  ; 1.x wrote the very same key this one does — HKCU\Software\Classes\LinkUnboundURL — so the
  ; two do not compete for the browser registration, they overwrite each other, and uninstalling
  ; either takes the registration away from whichever is left.
  ;
  ; Offered rather than done: somebody trying 2.0 may want 1.4 back, and taking it away without
  ; asking is not a decision an installer gets to make. Skipped when nothing can be asked — an
  ; update, or a passive install — where the install proceeds and 2.0 takes the key.
  ${If} $UpdateMode <> 1
  ${AndIf} $PassiveMode <> 1
    ReadRegStr $0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{7B2F4A1E-9C3D-4E5F-A6B8-1D2E3F4A5B6C}_is1" "UninstallString"
    ${If} $0 == ""
      ReadRegStr $0 HKLM "Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\{7B2F4A1E-9C3D-4E5F-A6B8-1D2E3F4A5B6C}_is1" "UninstallString"
    ${EndIf}
    ${If} $0 != ""
      ; Named labels, not relative jumps: the LogicLib blocks around this compile to jumps of
      ; their own, so counting instructions from here is counting something that moves.
      MessageBox MB_YESNO|MB_ICONQUESTION "LinkUnbound 1.4 is installed. Both versions register as the same browser, so they cannot both work: the one installed last takes the links, and uninstalling either leaves the other without them.$\r$\n$\r$\nRemove 1.4 now? Your rules and browsers are kept." /SD IDNO IDYES lu_drop_legacy
      Goto lu_legacy_done
      lu_drop_legacy:
        DetailPrint "Removing LinkUnbound 1.x"
        ; InnoSetup's own switches. It was installed for all users, so this raises a prompt of
        ; its own; the install goes on whatever comes of it.
        ExecWait '$0 /VERYSILENT /SUPPRESSMSGBOXES /NORESTART'
      lu_legacy_done:
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Nothing registers the handler on its own: the resident never writes the registry, and the
  ; settings window only reconciles when someone opens it. Without this, a fresh install receives
  ; no links until the user happens to visit the settings.
  ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --register'

  ; And put the resident back. PREINSTALL stopped it to replace the file, the template only ever
  ; relaunches the main binary, and nothing else in the program starts it — so without this every
  ; update ends with no tray icon and no shortcut until the next link happens to arrive.
  Exec '"$INSTDIR\linkunbound-shell.exe"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Both, and the settings window first: the template kills it only *after* this hook, and the
  ; single-instance guard makes a second copy hand its arguments to the live one and exit, so
  ; `--unregister` below would be answered by a window that has no idea what to do with it.
  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  !insertmacro CheckIfAppIsRunning "linkunbound-shell.exe" "${PRODUCTNAME}"

  ; Windows keeps offering an application whose keys are still there, so the registration is
  ; handed back before the binary that owns it is taken away.
  ;
  ; Not while updating: the installer is about to put the same keys back, and an update has no
  ; business dropping the user's default browser in between.
  ${If} $UpdateMode <> 1
  ${AndIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --unregister'
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Rules and browsers are the person's: wanting the program gone is not wanting the choices
  ; gone. Only what the program keeps for itself is swept.
  ${If} $UpdateMode <> 1
    Delete "$LOCALAPPDATA\${PRODUCTNAME}\update.json"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\icons"
    RMDir "$INSTDIR"
  ${EndIf}
!macroend
