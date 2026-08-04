# Captura de enlaces: cómo funciona y cómo se rompe

Documento de referencia sobre el camino que recorre un enlace desde que se pulsa
hasta que aparece el picker, y sobre las condiciones que lo interrumpen. Escrito
tras la auditoría de agosto de 2026, que encontró la app sin capturar enlaces en
Windows y macOS simultáneamente por causas distintas en cada plataforma.

## El camino del enlace

### Windows

1. El usuario pulsa un enlace en Slack, Teams, Outlook o el Explorador.
2. El shell resuelve el handler de `https` leyendo
   `HKCU\…\UrlAssociations\https\UserChoice` → ProgId (`LinkUnboundURL`).
3. Resuelve el ProgId en `HKCU\Software\Classes\LinkUnboundURL\shell\open\command`
   y ejecuta `"<exe>" "<url>"`.
4. El proceso nuevo parsea `argv` (`windows_bindings.dart`, `_parseInitialEvent`).
5. Intenta delegar en la instancia residente por el named pipe `\\.\pipe\LinkUnbound`.
   Si lo consigue, sale de inmediato; si no, se convierte él en residente.
6. El residente aplica las reglas por dominio y, si no hay ninguna, muestra el picker.

### macOS

1. Launch Services resuelve el handler de `https` a partir de `CFBundleURLTypes`
   del bundle y de la elección del usuario.
2. Lanza (o reactiva) la app y entrega el evento a `application(_:open:)`.
3. `InboundEventsChannel` encola la URL hasta que Dart señala `ready`.
4. Igual que en Windows a partir del paso 6.

## Precedencia del registro en Windows (importante)

`HKEY_CLASSES_ROOT` es una vista combinada donde **`HKCU\Software\Classes`
tiene prioridad sobre `HKLM\SOFTWARE\Classes`**. Consecuencia práctica:

> Una entrada por usuario escrita por una compilación local **secuestra** la
> asociación de la instalación real, sea del instalador `.exe` (que escribe en
> HKLM) o de Microsoft Store (que la declara en el manifiesto MSIX).

Este fue el fallo observado en la máquina de desarrollo: la app estaba instalada
desde la Store, pero `HKCU\Software\Classes\LinkUnboundURL\shell\open\command`
apuntaba a un `linkunbound.exe` de un árbol de compilación que ya no existía.
Windows dejó de ofrecer la app como navegador y los enlaces no abrían nada.

### Reglas que aplica el código

Implementadas en `WinRegistrationService.ensureRegistered` y en
`isDevBuildPath` (`win_package_context.dart`):

| Contexto de ejecución | Comportamiento |
| --- | --- |
| Instalación normal (`.exe`) | Registra, y **re-registra** si la ruta grabada dejó de coincidir |
| MSIX / Microsoft Store | No escribe nada, y **borra** cualquier entrada HKCU que esté sombreando al paquete |
| Árbol de compilación (`\build\windows\`) | No registra nunca; si detecta una entrada propia de un build, la elimina |

`ensureRegistered` se ejecuta **en cada arranque**, antes del primer frame
(`bootstrap.dart`). Solo escribe cuando algo cambió, así que el coste habitual es
una lectura de registro.

Antes, `register()` se llamaba una única vez en el primer arranque, dentro de un
`addPostFrameCallback`. Eso significaba que la ruta quedaba congelada de por vida:
actualizar, reinstalar o mover la app dejaba el handler apuntando a un ejecutable
inexistente sin ninguna forma de repararlo desde la interfaz.

## Enlaces internos de aplicaciones Microsoft

Teams, Outlook, Widgets, Copilot y la búsqueda del menú Inicio **no abren
`https:`**: envuelven la URL en `microsoft-edge:`, un esquema asociado a Edge que
ignora por completo el navegador predeterminado. Por eso los enlaces de esas
aplicaciones seguían abriéndose en Edge aunque LinkUnbound fuera el predeterminado.

La app puede interceptar ese esquema registrando un handler propio para
`microsoft-edge` en `HKCU\Software\Classes` (`setEdgeProtocolCapture`).
`stripEdgeProtocol` desenvuelve la URL interna antes de procesarla.

Es **opt-in**, con un interruptor en Ajustes → General: le quita un protocolo a
Edge, y quien prefiera el comportamiento de Edge debe poder conservarlo. No está
disponible bajo MSIX, donde un paquete no puede reclamar un protocolo de otro.

## Reglas por aplicación de origen

Una regla puede fijarse al dominio (`github.com → Firefox`) o a la aplicación
desde la que se pulsó el enlace (`todo lo que venga de Slack → Brave`). Las
segundas se guardan con `domain: "*"` y `sourceApp: "<app>"`.

Cómo se determina el origen:

| Plataforma | Método | Fiabilidad |
| --- | --- | --- |
| Windows | Proceso padre del proceso que lanzó el shell (`win_source_app.dart`) | Alta |
| macOS | Aplicación en primer plano (`SourceAppChannel`) | Aproximada |

En Windows el dato solo es válido en el proceso que el shell acaba de lanzar: en
cuanto la URL se delega al residente por el pipe, el padre de ese residente no
tiene nada que ver. Por eso el origen se resuelve al parsear los argumentos y
viaja **dentro** del `OpenUrlEvent`.

macOS no expone quién originó un evento de apertura, así que se aproxima con la
app en primer plano. Acierta en el caso normal y falla si el usuario cambia de
ventana en ese mismo instante.

Se descartan `explorer`, `cmd` y `powershell` como origen: son el propio shell,
no una aplicación contra la que tenga sentido escribir una regla.

**Precedencia** (`RuleService.lookupRule`): una regla que nombra la app gana
siempre a una que solo nombra el dominio, aunque esta última sea más específica.
«Todo lo de Slack en Brave» es una afirmación deliberada sobre el origen y una
regla genérica de dominio no debe anularla en silencio. Dentro de cada ámbito, el
dominio más específico gana, y los subdominios heredan de su dominio padre.

## Modo privado

`Shift` + clic (o `Shift` + número) en el picker abre el enlace en una ventana
privada. Mientras `Shift` está pulsado, las filas muestran un icono; solo lo
hacen los navegadores que aceptan un modificador para ello.

El modificador depende de la familia (`private_mode.dart`), porque no hay
estándar y uno equivocado no se ignora: Chromium trata un `--flag` desconocido
como argumento y Firefox abre una página de error.

| Familia | Modificador |
| --- | --- |
| Chrome, Brave, Vivaldi, Chromium | `--incognito` |
| Edge | `-inprivate` |
| Firefox, LibreWolf, Waterfox, Zen | `-private-window` |
| Opera | `--private` |
| Safari | No admite; la opción no se ofrece |

En macOS el lanzamiento privado usa `open -na … --args <flag> <url>`. El `-n` es
obligatorio: si la aplicación ya está en ejecución, `open` descarta `--args` por
completo y el enlace se abriría en una ventana normal sin ningún aviso.

Un navegador puede fijar su propio modificador con `privateArgs` en
`browsers.json`; una lista vacía significa «este navegador no admite modo
privado».

## Autodiagnóstico

Ajustes → General muestra una tarjeta de reparación cuando `diagnose()` detecta
que el manejador registrado apunta a otra ubicación. El botón «Reparar» vuelve a
registrar la aplicación e invalida el estado en pantalla.

No se ofrece reparación cuando la app corre desde un árbol de compilación: ahí
registrar empeoraría las cosas en vez de arreglarlas, así que solo se explica el
motivo.

## Integridad y elevación (Windows)

El instalador pide privilegios de administrador. Si lanza la app al terminar sin
`runasoriginaluser`, la app hereda el token elevado y sus objetos kernel —el
mutex de instancia única y el named pipe— quedan a integridad **alta**. Slack,
Teams y el Explorador corren a integridad **media** y no pueden escribir en ellos:
cada clic se descartaba en silencio.

Dos medidas, ambas necesarias:

- `setup_template.iss` lanza la app con `runasoriginaluser`.
- El pipe se crea con un descriptor de seguridad explícito
  (`win_security.dart`): DACL restringida al usuario actual y a SYSTEM, más una
  etiqueta de integridad baja con `NO_WRITE_UP` para que un cliente de integridad
  media pueda entregar la URL aunque el servidor esté elevado.

## Diagnóstico en campo

`%LOCALAPPDATA%\LinkUnbound\navigate.log` (Windows) o
`~/Library/Application Support/LinkUnbound/navigate.log` (macOS).

Mensajes que identifican cada fallo:

| Mensaje | Significado |
| --- | --- |
| `Handler command drifted; re-registering` | La ruta grabada no coincidía; se ha reparado |
| `Removing stale per-user registration shadowing the MSIX package` | Había un residuo HKCU tapando la instalación de la Store |
| `Removing registration owned by a local build tree` | Un build local tenía secuestrada la asociación |
| `Refusing to register a local build tree as URL handler` | Se ejecutó desde `\build\windows\`; no se registra |
| `Ignoring unrecognised launch argument` | Llegó un argumento que no es una URL reconocible |
| `Pipe name already owned by another process` | Otro proceso ocupa el pipe; la delegación no funcionará |
| `Discarding initial event: no resident could be reached` | No se pudo delegar ni tomar el rol de residente |
| `Rejected URL with non-launchable scheme` | Esquema no permitido (ver más abajo) |

Comprobación rápida del registro en Windows:

```powershell
reg query "HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice"
reg query "HKCU\Software\Classes\LinkUnboundURL\shell\open\command"
```

El primero debe dar `ProgId = LinkUnboundURL` (o el ProgId `AppX…` del paquete
MSIX). El segundo debe apuntar a un ejecutable **que exista**.

## Esquemas aceptados

`isLaunchableUrl` (`packages/core/lib/src/url_utils.dart`) es la única puerta
por la que una URL llega a `Process.start`. Solo admite `http`, `https` y `file`,
y rechaza cualquier cadena que empiece por `-`, `/` o `\`.

El motivo es concreto: los navegadores interpretan como modificador cualquier
argumento que empiece por guion, y modificadores como `--gpu-launcher=` o
`--utility-cmd-prefix=` ejecutan binarios arbitrarios. Como los eventos entrantes
llegan por IPC desde cualquier proceso local, el esquema no es de fiar.

Las rutas locales (`file:`) pasan además por `resolveLocalWebFile`, que aplica
una lista blanca de extensiones y **rechaza rutas UNC**: comprobar la existencia
de `\\host\share\x.html` hace que Windows se autentique contra ese host y filtre
un hash NetNTLMv2.
