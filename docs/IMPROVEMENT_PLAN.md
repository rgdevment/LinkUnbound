# LinkUnbound — Improvement Plan (Windows + macOS)

> **Documento histórico (cerrado el 2026-08-04).** Las seis fases se
> implementaron. Se conserva porque cada ítem cita el archivo y la línea del
> hallazgo original, lo que sigue siendo útil para entender por qué el código
> quedó como quedó — pero **ya no es un plan vivo y no refleja el estado
> actual**. La referencia vigente sobre captura de enlaces es
> [`LINK_CAPTURE.md`](LINK_CAPTURE.md).
>
> | Fase | Contenido | Estado |
> |---|---|---|
> | 1 | macOS UX: sin botón Exit, titlebar nativa en Settings, activation policy, Cmd+W | ✅ |
> | 2 | Estabilidad Windows: TOCTOU, pipe busy-spin, buffering, migración | ✅ Probada en Windows real |
> | 3 | Latencia picker: cold start, round-trips, multi-monitor, IO en build | ✅ Probada en multi-monitor |
> | 4 | Ocultar tray + hotkey global + arranque silencioso por login item | ✅ |
> | 5 | Pulido: blur unificado, foco/Esc picker, tema claro/sistema, debounce de registro, log con IOSink, ProgId exacto, TTL de updates | ✅ P2.10 (notificaciones) diferido |
> | 6 | Icono/branding | ✅ |
>
> Lo hecho después de cerrar este plan no está recogido aquí: la reparación de
> la captura de enlaces en ambos sistemas, el endurecimiento del pipe y de la
> validación de URLs, y las funciones de modo privado, reglas por aplicación de
> origen, autodiagnóstico y captura del esquema `microsoft-edge:`. Todo ello
> está documentado en `LINK_CAPTURE.md` y en el historial de git.
>
> Las cifras de tests que aparecen más abajo en el cuerpo corresponden al
> momento de cada fase y hoy se quedan cortas: la suite está en 526 tests de app
> y 168 de core.

> Generado a partir de una auditoría completa del código (UI, performance, integración
> nativa Windows/macOS). Cada ítem referencia archivo y línea del hallazgo.
> Prioridades: **P0** = preocupaciones explícitas del owner / pérdida de datos,
> **P1** = estabilidad y latencia del picker (métrica central del producto),
> **P2** = pulido y paridad, **P3** = deuda menor.

---

## Resumen ejecutivo

| Área | Estado | Riesgo |
|---|---|---|
| macOS UX (botón Exit, Dock, ocultar/atajo) | 1 de 3 quejas ya casi resuelta (Dock via `LSUIElement`); las otras 2 requieren trabajo | Medio |
| Windows single-instance / pipe | Carrera TOCTOU puede **perder enlaces**; bucle de pipe puede consumir 100% CPU | **Alto** |
| Latencia del popup (cold start + show) | Red + IO síncrono + 6-7 round-trips de canal en serie en el camino crítico | **Alto** |
| Multi-monitor | El picker se posiciona contra el display primario siempre | Alto |
| Icono / branding | Pendiente — se aborda al final con la imagen nueva | — |

---

## P0 — Preocupaciones explícitas del owner (macOS UX)

### P0.1 Eliminar el botón "Salir" de la title bar en macOS
- **Hallazgo:** `apps/linkunbound/lib/ui/shared/widgets/title_bar.dart:82-104` renderiza un
  `TextButton.icon` "Exit" solo en macOS; viola la HIG y además **la etiqueta miente**: el
  handler (`lib/ui/settings/settings_view.dart:68-73`) solo oculta la ventana, no sale de la app.
- **Fix:**
  1. Eliminar el bloque `if (isMac)` del botón Exit en `title_bar.dart`.
  2. En `WindowChannel.applySettingsMode` (`macos/Runner/Channels/WindowChannel.swift:33-53`)
     mostrar los traffic lights (close + miniaturize); hoy se ocultan en ambos modos. En modo
     picker sí deben seguir ocultos.
  3. El botón rojo de cerrar debe **ocultar** (no terminar): ya garantizado por
     `setPreventClose(true)` + `applicationShouldTerminateAfterLastWindowClosed = false`.
  4. Wirear `MainMenu.xib`: Quit (Cmd+Q) → exit real con limpieza (`release()` de
     `macos_bindings.dart:127-130` antes de salir; hoy un Cmd+Q nativo se saltaría la limpieza
     del tray y el servidor inbound), Close (Cmd+W) → ocultar ventana.
  5. Windows conserva su `_CloseButton` actual sin cambios.

### P0.2 App de menu bar real (sin Dock) con foco correcto en Settings
- **Estado:** `Info.plist:27-28` ya declara `LSUIElement = true` — la app **ya no vive en el
  Dock**. Lo que falta es el juggling de activation policy: una app accessory no recibe
  foco de teclado ni menú de forma fiable cuando abre Settings.
- **Fix:**
  1. Añadir a `WindowChannel.swift` dos métodos: `setRegular` → `NSApp.setActivationPolicy(.regular)`
     y `setAccessory` → `NSApp.setActivationPolicy(.accessory)`.
  2. En `_applyAppMode` (`lib/bootstrap.dart:201-230`): al entrar a `AppMode.settings` llamar
     `setRegular` **antes** de `activate()`; al volver a `hidden`, `setAccessory`. El modo
     `picker` permanece accessory (popup estilo Spotlight, no debe tocar Dock ni menú).
  3. Nota esperada: el icono aparece en el Dock solo mientras Settings está abierto — correcto
     según HIG.
  4. Lanzamiento desde Spotlight/Aplicaciones ya funciona (`applicationShouldHandleReopen` →
     `enqueueShowSettings`, `AppDelegate.swift:37-44`). Mantener y cubrir con test.

### P0.3 Opción "ocultar por completo" + atajo global (ambos SO)
- **Estado:** no existe ningún paquete de hotkey ni toggle para ocultar el tray.
- **Fix:**
  1. **Toggle "ocultar icono de menu bar/tray"**: preferencia persistida (mismo patrón que
     `localeFile` en `macos_bindings.dart:65-66`). `MacOsTrayController.dispose()`
     (`lib/platform/macos/macos_tray_controller.dart:60-66`) ya destruye el `NSStatusItem`;
     falta exponer toggle en `general_page.dart` y re-init al desactivarlo. Paridad Windows:
     mismo toggle sobre `system_tray`.
  2. **Atajo global**: integrar `hotkey_manager` (usa Carbon `RegisterEventHotKey` en macOS —
     **no requiere permiso de Accesibilidad**, a diferencia de `CGEventTap`; en Windows usa
     `RegisterHotKey`). Atajo configurable que dispare `showSettings()`.
  3. **Salvaguarda de reentrada**: permitir ocultar el tray **solo si** hay un atajo global
     configurado; si no, la app quedaría alcanzable únicamente relanzando el .app (que también
     funciona, vía reopen). Avisar en la UI.
  4. macOS adicional: con arranque por login item la app hoy abre Settings en cada inicio de
     sesión (`macos_bindings.dart:115` fuerza `startsHidden = false` y `bootstrap.dart:179-183`
     muestra settings). Detectar arranque por login item / flag `--background` (como ya hace
     Windows, `pubspec.yaml:56`) y arrancar silenciosa en el menu bar.

### P0.4 Icono nuevo (al final)
- Regenerar la imagen (búho-escarabajo sosteniendo el rectángulo-escudo, simplificado para
  16x16), verificar transparencia y generar el set completo: PNG 16-1024, `app_icon.ico`
  multi-res, `AppIcon.appiconset`, iconos de tray. Reemplazar en `resources/assets/`,
  `apps/linkunbound/assets/`, `windows/runner/resources/`, `macos/Runner/Assets.xcassets/`.

---

## P0.5 — Links de Teams/Widgets abren Edge ignorando LinkUnbound (Windows)

- **Causa raíz (auditada):** Teams, Widgets, búsqueda de Inicio y Copilot abren links vía el
  protocolo `microsoft-edge:`, que tiene su propio UserChoice clavado a Edge. LinkUnbound solo
  registra `http`/`https` (`win_registration_service.dart`, `_writeCapabilities`). El código ya
  tiene `stripEdgeProtocol` en `packages/core/lib/src/url_utils.dart` esperando esas URLs.
- **Plan:**
  1. Toggle **opt-in** en Settings (desactivado por defecto): registra `microsoft-edge:` →
     nuevo ProgId `LinkUnboundEdgeProto` con `shell\open\command` = `"exe" "%1"`.
  2. Opt-in porque interceptarlo rompe funciones legítimas de Edge (sidebar Copilot, visor PDF).
  3. Antes de implementarlo, verificar config del usuario: Outlook/Teams tienen su propio ajuste
     "abrir enlaces en Edge" y políticas MDM que fuerzan Edge.
  4. Descartada la vía IFEO de MSEdgeRedirect (Defender la marca como secuestro de proceso).
  5. Caveat Win11: el UserChoice de `microsoft-edge` está endurecido; `ms-settings` no siempre
     ofrece este protocolo en su UI. Documentar la limitación.

## P0.6 — Single-instance en macOS (hallazgo de verificación)

- Durante la verificación en vivo convivieron sin queja dos instancias (la instalada en
  `/Applications` y un build debug). Windows tiene mutex+pipe; macOS no tiene guard equivalente.
  Normalmente Launch Services no relanza el mismo bundle, pero dos copias del .app (versiones
  distintas, debug vs instalada) sí coexisten y pueden pelearse los links.
- **Plan:** detección al arrancar (p. ej. `NSRunningApplication` con el mismo bundle id) y
  delegar al residente + salir, espejo del flujo de Windows.

## P1 — Estabilidad y latencia del picker

### P1.1 Carrera TOCTOU en single-instance (Windows) — puede perder enlaces
- **Hallazgo:** `lib/platform/windows/windows_bindings.dart:139-159` + `lib/bootstrap.dart:27-42`.
  Entre `tryDelegate()` (pipe aún no escucha) y `claim()` (mutex ya tomado) un segundo proceso
  hace `exit(0)` y **el enlace se descarta silenciosamente**.
- **Fix:** tras `acquire()`, confirmar que el pipe está escuchando antes de devolver `true`;
  en `tryDelegate` reintentar `CreateFileW` con backoff (3 × 50 ms, `WaitNamedPipe`); si
  `claim()` falla, reintentar `tryDelegate` una vez antes de `exit(0)`.

### P1.2 Bucle del pipe server con busy-spin y handles huérfanos (Windows)
- **Hallazgo:** `lib/platform/windows/win_pipe_server.dart:193-247`. `continue` sin pausa ante
  errores persistentes → 100% CPU en background; `stop()` solo conoce el primer handle.
- **Fix:** pausa de 50 ms antes de cada `continue` de error; reportar cada handle nuevo al
  `sendPort`; considerar `OVERLAPPED` + evento de cancelación.

### P1.3 Cold start bloquea el primer popup (macOS sobre todo)
- **Hallazgo:** `lib/bootstrap.dart:139,151,169,175`. Antes de suscribir `inboundEvents` se
  ejecutan en serie: load de browsers/rules, init de window manager, init de tray (con load de
  l10n) y `updateInfoProvider` → **HTTP a GitHub en el camino crítico del primer frame**.
- **Fix:** suscribir `inboundEvents` lo antes posible; mover tray init y update check a
  post-`runApp` (`addPostFrameCallback`); diferir el update check con `Timer` de varios segundos.

### P1.4 Transición a picker encadena 6-7 round-trips de canal en serie
- **Hallazgo:** `lib/bootstrap.dart:213-229`. `setPickerMode`, `cursorPosition`, `screenSize`,
  `setSize`, `setPosition`, `setSkipTaskbar`, `setAlwaysOnTop`, `show`, `activate` — todos con
  `await` secuencial. Es la latencia percibida del popup.
- **Fix:** paralelizar `cursorPosition()`/`screenSize()`; reposicionar **antes** de `show()`
  (también elimina el fantasma en la posición anterior — ver P2.4); evaluar una llamada nativa
  combinada (un solo round-trip) para la ruta caliente.

### P1.5 Multi-monitor roto: posicionamiento contra el display primario
- **Hallazgo:** `lib/platform/cursor_locator.dart:21-25` siempre usa `getPrimaryDisplay()`; el
  clamp (`bootstrap.dart:219-223`) no contempla monitores secundarios, offsets negativos, work
  area (taskbar) ni DPI por monitor.
- **Fix:** `getAllDisplays()` + hit-test del cursor; clamp contra `visiblePosition`/`visibleSize`
  y `scaleFactor` del display que contiene el cursor.

### P1.6 Eventos del pipe perdidos antes del primer listener (Windows cold start)
- **Hallazgo:** `lib/platform/windows/win_pipe_server.dart:148,161` — `StreamController.broadcast`
  sin buffer; el path macOS sí bufferiza (`mac_inbound_events.dart:36-42`), el de Windows no.
- **Fix:** bufferizar eventos hasta el primer `onListen`, igual que macOS.

### P1.7 Migración APPDATA → LOCALAPPDATA falla entre volúmenes
- **Hallazgo:** `lib/platform/windows/windows_bindings.dart:180-192` usa `renameSync`, que lanza
  si los perfiles están en volúmenes distintos (roaming corporativo) → **configuración perdida**.
  Además corre antes de `claim()`, con carrera en doble arranque.
- **Fix:** fallback a copia recursiva + borrado; ejecutar la migración después de `claim()`.

---

## P2 — Performance UI y pulido

### P2.1 IO síncrono en build del picker
- `lib/ui/picker/picker_view.dart:213,229`: `existsSync()` por fila y por rebuild (hover).
  Precomputar la existencia del icono al cargar browsers o usar `Image.file` + `errorBuilder`.
  Añadir `cacheWidth` (iconos se pintan a 28px).

### P2.2 Extracción de iconos bloquea el arranque y la UI
- `lib/bootstrap.dart:247-255` (primer arranque, secuencial, pre-`runApp`) y
  `general_page.dart:440-449` (refresh). En Windows la conversión GDI/PNG es síncrona en el
  hilo de UI (`win_icon_extractor.dart:121-190`). Mover a `Isolate.run`, paralelizar, y diferir
  el primer llenado a post-`runApp` con icono fallback.

### P2.3 Lógica de blur duplicada con timers mágicos distintos
- `lib/app.dart:50-78` (timer 400 ms) y `lib/ui/picker/picker_window.dart:44-62` (timer 200 ms):
  dos `WindowListener` que llaman `hide()`. Riesgo de picker pegado o cierre espurio.
  Unificar en un solo guard basado en el primer `onWindowFocus` real, no en timers.

### P2.4 Foco del picker no garantizado en Windows
- `_applyAppMode` no llama `windowManager.focus()` en el caso picker; `AllowSetForegroundWindow`
  se llama en el proceso entrante que muere (`windows_bindings.dart:143`). Forzar foreground
  desde el residente; añadir manejo de `Esc` para cerrar el picker (hoy solo blur o selección).
  En macOS, asegurar `canBecomeKey` en modo picker.

### P2.5 Lecturas de registro en el hilo de UI en cada foco (Windows)
- `lib/app.dart:44-47` invalida providers que disparan hasta 7+ `Registry.openPath` síncronos
  (`win_registration_service.dart:84-129`) al recibir foco. Cachear con debounce o mover a isolate.

### P2.6 Detección de "default browser" frágil (Windows)
- `progIdMatchesLinkUnbound` usa `contains('linkunbound')` (`win_registration_service.dart:18`) y
  no valida el `Hash` de `UserChoice`. Usar igualdad exacta del ProgId y, como fuente de verdad,
  `IApplicationAssociationRegistration::QueryCurrentDefault` (COM).

### P2.7 Tema fijo oscuro
- `lib/app.dart:82` fuerza `AppTheme.dark`. Añadir `AppTheme.light` + `themeMode: ThemeMode.system`
  y reaccionar a `onPlatformBrightnessChanged`. (El runner Windows ya lee `AppsUseLightTheme`.)

### P2.8 Reglas: semántica inconsistente y save sin manejo de error
- El lookup es jerárquico (`rule_service.dart:56-64`) pero el picker guarda `uri.host` exacto
  (`picker_view.dart:88-93`), y `ruleService.save()` se llama sin await ni catch → persistencia
  silenciosamente perdible. Alinear semántica y envolver con `unawaited(...catchError)`.

### P2.9 Atajos de teclado 7-9 lanzan navegadores fuera de la vista
- `picker_layout.dart:11` (`maxVisible = 6`) vs atajos 1-9 (`picker_view.dart:99-110`).
  Hacer scroll-into-view o limitar atajos a los visibles.

### P2.10 Notificaciones del sistema
- No existen. Añadir al menos "actualización disponible" (hoy solo visible si abres settings)
  y fallos de registro como navegador predeterminado.

---

## P3 — Deuda menor

| Ítem | Ubicación | Fix |
|---|---|---|
| Log con `writeAsStringSync` por record en main isolate | `packages/core/lib/src/services/log_service.dart:66` | `IOSink` con buffer o nivel WARNING en archivo |
| Regex de redacción corre en cada log | `log_service.dart:16-29` | corto-circuito si no contiene `://` |
| `kernel32.dll`/`shell32.dll` reabiertos por llamada | `win_pipe_server.dart:249-254`, `win_registration_service.dart:405-412` | `static final` |
| `updateInfoProvider` sin autoDispose ni TTL | `lib/providers.dart:198` | autoDispose + refresh periódico |
| Safari bundle id en minúsculas | `RegistrationChannel.swift:12` | `com.apple.Safari` |
| `List.unmodifiable` por acceso | `browser_service.dart:19` | cachear wrapper tras mutación |
| `_extractExePath` trunca en el primer `.exe` | `win_browser_detector.dart:106-107` | parse robusto |
| Estado de ventana inconsistente si una transición falla a medias | `bootstrap.dart:205-229` | try/finally por paso |
| Hardened runtime / notarización no documentados (sandbox off es necesario) | `Release.entitlements:5-6` | documentar flujo de firma |

---

## Orden de ejecución sugerido

1. **Fase 1 — macOS UX (P0.1, P0.2):** quitar botón Exit, traffic lights en settings,
   activation policy juggling, menú Cmd+Q/Cmd+W. Cambios pequeños, impacto visible inmediato.
2. **Fase 2 — Estabilidad Windows (P1.1, P1.2, P1.6, P1.7):** single-instance, pipe server,
   migración. Evita pérdida de enlaces y configuración. Bloqueante para release.
3. **Fase 3 — Latencia del picker (P1.3, P1.4, P1.5, P2.1, P2.2):** cold start, round-trips,
   multi-monitor, IO en build. La métrica central del producto.
4. **Fase 4 — Ocultar + hotkey (P0.3):** nueva capacidad, ambos SO, con salvaguarda de reentrada.
5. **Fase 5 — Pulido (P2.3-P2.10, P3):** blur unificado, foco, tema claro, notificaciones, deuda.
6. **Fase 6 — Branding (P0.4):** icono nuevo y set completo de assets.

Cada fase debe cerrar con: `melos run analyze` + tests (incluyendo nuevos tests de los fixes)
y verificación manual en el SO afectado.
