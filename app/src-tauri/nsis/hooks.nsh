; Inno wrote the key under the plain app id, with no braces: `AppId={{APP_ID}}` in that template
; is a placeholder the packaging tool filled, not Inno's own `{{` escape. An earlier 1.x spelled
; it differently again, so the name is something to look for rather than to guess.
Function FindLegacyUninstaller
  Push $1
  Push $2
  Push $3
  Push $4
  StrCpy $4 ""
  StrCpy $1 0
  lu_next_key:
    EnumRegKey $2 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall" $1
    ${If} $2 == ""
      Goto lu_no_more
    ${EndIf}
    IntOp $1 $1 + 1
    ; The app id 1.x was built with, however its installer decided to write it down.
    StrCpy $3 $2 8
    ${If} $3 != "7B2F4A1E"
      Goto lu_next_key
    ${EndIf}
    ReadRegStr $4 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\$2" "UninstallString"
    ${If} $4 != ""
      Goto lu_no_more
    ${EndIf}
    Goto lu_next_key
  lu_no_more:
  StrCpy $0 $4
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Push $0
FunctionEnd

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

  ; 1.x is not a version this one can sit beside. Its installer wrote the browser keys under
  ; HKLM, and its uninstaller deletes the HKCU ones the app itself wrote — which are the very
  ; keys 2.0 writes, so removing 1.4 after this point would take 2.0's registration with it.
  ;
  ; Offered rather than done: somebody trying 2.0 may want 1.4 back. Skipped where nothing can be
  ; asked — an update, or a passive install — and there 2.0 simply takes the keys.
  ${If} $UpdateMode <> 1
  ${AndIf} $PassiveMode <> 1
    Call FindLegacyUninstaller
    Pop $0
    ${If} $0 != ""
      ; Named labels, not relative jumps: the LogicLib blocks around this compile to jumps of
      ; their own, so counting instructions from here is counting something that moves.
      MessageBox MB_YESNO|MB_ICONQUESTION "LinkUnbound 1.4 is installed. Both versions register as the same browser, so they cannot both work: the one installed last takes the links, and uninstalling either leaves the other without them.$\r$\n$\r$\nRemove 1.4 now? Your rules and browsers are kept." /SD IDNO IDYES lu_drop_legacy
      Goto lu_legacy_done
      lu_drop_legacy:
        DetailPrint "Removing LinkUnbound 1.x"
        ; ExecShellWait, not ExecWait: that uninstaller was installed for all users and asks for
        ; elevation, which CreateProcess cannot raise — it would fail with no prompt and no word.
        ; And it has to finish before anything below writes a key, because what it deletes on its
        ; way out is what this install is about to put there.
        ClearErrors
        ExecShellWait "open" "$0" "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART"
        ${If} ${Errors}
          DetailPrint "LinkUnbound 1.x was left in place"
        ${EndIf}
      lu_legacy_done:
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Nothing registers the handler on its own: the resident never writes the registry, and the
  ; settings window only reconciles when someone opens it. Without this, a fresh install receives
  ; no links until the user happens to visit the settings.
  ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --register' $0
  ${If} $0 <> 0
    DetailPrint "The browser registration did not complete (code $0)"
  ${EndIf}

  ; And put the resident back. PREINSTALL stopped it to replace the file, the template only ever
  ; relaunches the main binary, and nothing else in the program starts it — so without this every
  ; update ends with no tray icon and no shortcut until the next link happens to arrive.
  ;
  ; RunAsUser, never Exec: run from an installer somebody elevated, the resident would inherit
  ; that token, and its single-instance mutex and its pipe would sit at an integrity level the
  ; processes that actually open links cannot reach. Every click would be dropped in silence.
  ; --hushed because this is not somebody asking for a window.
  nsis_tauri_utils::RunAsUser "$INSTDIR\linkunbound-shell.exe" "--hushed"
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
    StrCpy $0 1
    ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --unregister' $0
    ${EndIf}

    ; Whatever happened above, these do not get to survive. A ProgId left behind still answers
    ; for http and https: the person's UserChoice goes on naming it, its command points at a
    ; binary that is gone, and every link in the system opens nothing at all, with nothing to
    ; say why. Nobody but its owner can put UserChoice back.
    ${If} $0 <> 0
      DetailPrint "Handing back the browser registration the short way"
      DeleteRegKey HKCU "Software\Classes\LinkUnboundURL"
      DeleteRegKey HKCU "Software\Clients\StartMenuInternet\LinkUnbound"
      DeleteRegKey HKCU "Software\LinkUnbound"
      DeleteRegValue HKCU "Software\RegisteredApplications" "LinkUnbound"
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Rules and browsers are the person's: wanting the program gone is not wanting the choices
  ; gone. Only what the program keeps for itself is swept.
  ${If} $UpdateMode <> 1
    Delete "$LOCALAPPDATA\${PRODUCTNAME}\update.json"
    Delete "$LOCALAPPDATA\${PRODUCTNAME}\updating.json"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\icons"

    ; The template's own checkbox deletes ${BUNDLEID}, which holds the webview cache and nothing
    ; a person would recognise. What they were answering about is this.
    ${If} $DeleteAppDataCheckboxState = 1
      RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}"
    ${EndIf}
  ${EndIf}
!macroend
