[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · **Español** · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Solución de problemas y preguntas frecuentes

Empieza por las respuestas rápidas de abajo; las secciones más largas tratan de
Git LFS y del renderizado del grafo. Si nada de esto ayuda, abre un issue en el
repositorio de GitHub con tu SO, tu terminal, `gitpoulpi --version` y lo que
hiciste.

## Cuando algo falla

Una operación que falla no muestra ninguna ventana emergente ni tarjeta con marco: la **barra de estado** lo dice, en rojo, con el título y el motivo (*error: Falló el fetch: no se pudo contactar con el remoto.*), y se queda ahí hasta que otro mensaje la reemplaza. El botón Actividad, en su extremo derecho, cuenta los errores que todavía no has mirado. Haz clic en el mensaje rojo, o ejecuta `Registro de actividad: Abrir el último error` de la paleta, para abrir el Registro de actividad en esa entrada. Todo fallo que provocas funciona así, no solo las operaciones de git:

- **Operaciones de git**: un fetch, un push, un pull, un checkout, un merge, un rebase, un commit, un stash...
- **Un clonado** (y la creación de un repositorio nuevo): «Falló el clonado».
- **Una llamada a GitHub o GitLab**: las listas de repositorios de los selectores y del diálogo Clonar, la creación de una pull request o de un issue, la comprobación de un token o de una clave SSH en la página Integraciones: «Falló la solicitud al servicio de alojamiento».
- **Un programa externo**: el editor, la herramienta de diff o de merge, el explorador de archivos, el navegador, el terminal o el lanzador, un editor de imágenes, los botones *Probar* de los Ajustes: «Falló la herramienta externa».
- **Una solicitud de IA** (un mensaje de commit, una explicación, una sugerencia para un conflicto...): «Falló la solicitud de IA».
- **Un archivo que no se pudo escribir**: Guardar del editor integrado, el editor de merge, un ajuste que no se pudo guardar: «Falló el guardado».

El mensaje rojo está en la barra de estado sea cual sea la vista que tengas delante: una pestaña de repositorio, Inicio, el Radar, los Espacios de trabajo, los Ajustes (en la fila inferior de la pantalla), debajo del Registro de actividad o de un diálogo. Donde la barra de estado queda tapada (Inicio, el Radar, los Espacios de trabajo, los Ajustes) el mensaje se escribe sobre su fila inferior hasta que hayas mirado el error. Te sigue de una pestaña a otra. Una operación que todavía corre cuando cambias de pestaña no se sigue: abre el Registro de actividad para ver cómo terminó.

Las listas de las secciones PULL REQUESTS e ISSUES que no se pueden actualizar conservan en su lugar su propia línea en la sección (se actualizan solas; un mensaje cada pocos segundos sería solo ruido). Un diálogo que sigue abierto (Clonar, Crear pull request...) también conserva su propio mensaje de una línea, para que puedas corregir el campo y volver a intentarlo.

La entrada del Registro de actividad contiene todo lo que mostraba la ventana emergente:

- **Título y motivo.** «Falló el fetch», luego una frase que dice por qué con palabras claras, luego qué hacer. Se reconocen: el remoto tiene commits que no tienes (*haz pull primero*), credenciales rechazadas (*revisa Ajustes > Integraciones o tu clave SSH*), una parada por conflictos, cambios locales que se sobrescribirían (*haz commit, stash o descártalos*), el remoto no es accesible y otro proceso de git que mantiene un bloqueo sobre el repositorio. Cualquier otra cosa muestra la primera línea propia de git. Un clonado o una llamada al host que falla de la misma manera (credenciales rechazadas, host inaccesible) se redacta igual; los demás fallos reciben una frase sobre qué falló («No se pudo clonar el repositorio.», «No se pudo ejecutar el programa.»).
- **Los detalles**, visibles cuando la entrada está abierta (`Enter`, o el clic que la abrió): la operación, el comando de git que se ejecutó, su código de salida y las últimas líneas de lo que imprimió (para la biblioteca de git integrada, su clase y código de error). Las credenciales, los tokens y las cabeceras `Authorization` se ocultan antes de mostrar, registrar o copiar nada.
- **Copiar** (el botón, o `c`) pone en el portapapeles un bloque de texto hecho para pegarlo en un chat con alguien, o con una IA, que pueda explicarlo: la versión de GitPoulpi, el sistema, el terminal, el backend, la operación que falló con sus detalles y la versión de git.
- **Un botón para el paso siguiente**, cuando lo hay, junto a Copiar (o `r`): *Pull primero* (un push que el remoto rechazaría), *Stash y reintentar* (cambios locales que estorban a un checkout, merge, rebase o pull), *Reintentar* (no se pudo contactar con el remoto), *Abrir Integraciones* (credenciales). Ejecuta el paso y cierra la ventana. Para el fallo de un repositorio distinto de la pestaña activa solo se ofrece *Abrir Integraciones*: un reintento se ejecutaría en el repositorio equivocado.
- Sin ratón, la paleta de comandos tiene `Registro de actividad: Abrir el último error`, `Registro de actividad: Copiar el último error` y `Registro de actividad: Ejecutar la acción sugerida` (el botón de remedio de la entrada, para el error que muestra la barra de estado).

Un push que el remoto rechazó porque avanzó conserva su propio diálogo (pull primero, force push con lease, cancelar); los hechos siguen estando en el Registro de actividad.

### El Registro de actividad

El botón **Actividad** en el extremo derecho de la barra de estado (un número en él cuenta los errores que aún no has mirado), `Ctrl+N` y luego `Shift+A`, y `Ver: Registro de actividad` de la paleta abren la ventana. `Ctrl+N` muestra una insignia en cada pestaña, cuadro, campo, casilla y botón de la ventana (y en la primera entrada de la lista); pulsar el carácter hace lo mismo que un clic en él. Dos pestañas, **Operaciones de git** y **Aplicación**, que se cambian con `Tab`, con un clic, con `g` (Operaciones de git) o con `p` (Aplicación), y un cuadro **Alcance** bajo ellas que vale para ambas (véase más abajo):

- **Aplicación**: las líneas del registro del propio programa, las de la ventana y las del daemon en segundo plano, las más recientes primero. El cuadro **Nivel** (una lista desplegable: Error, Aviso, Info, Depuración, Traza; `l` la abre) dice cuánto mostrar; pedir más detalle del que se registra también sube el nivel de registro (y lo guarda como `log.level`). Debajo, **Mostrar** tiene una casilla por nivel (Error, Aviso, Info, Depuración, Traza, cada una con cuántas líneas tiene) para ocultar o mostrar las líneas de ese nivel, por ejemplo para ver solo los errores, y **Todo** / **Nada** marcan o desmarcan todas las casillas. Las teclas `1` a `5` alternan los niveles en ese orden, `a` es Todo, `n` es Nada; la paleta tiene `Registro de actividad: Mostrar u ocultar el nivel ...`, `Registro de actividad: Mostrar todos los niveles` y `Registro de actividad: No mostrar ningún nivel`. Los niveles por encima de lo elegido en el cuadro Nivel salen en gris (no se registra nada para ellos; pasa el puntero por uno para ver por qué). Las marcas se recuerdan hasta que sales, y solo la pestaña Aplicación las tiene. **Filtro** (`/`) conserva las líneas que contienen tu texto. **Copiar todo (filtrado)** copia lo que se ve (se aplican las marcas y el filtro); **Abrir la carpeta de registros** abre la carpeta en tu gestor de archivos. Una línea larga nunca se corta: véase «Líneas largas» más abajo.
![El Registro de actividad, pestaña Aplicación: el cuadro Nivel, las casillas Mostrar por nivel con Todo y Nada, el filtro y las líneas del registro](../images/activity-log.png)

**Vaciar registro** (`x`, o el comando de la paleta `Registro de actividad: Vaciar el registro de la aplicación`) está en la pestaña **Aplicación**: pregunta primero y luego vacía los archivos de registro de todos los días conservados y la lista en pantalla. No se puede deshacer y no figura en el historial de deshacer.

- **Operaciones de git** (la pestaña en la que se abre la ventana): las operaciones, las más recientes primero, cada una con la hora a la que se ejecutó y lo que tardó (`Push - 123 ms`): abrir el repositorio, cargar el grafo de commits, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Aquí también se listan los fallos, los mensajes de estado, un clon fallido y el Fetch all de un espacio de trabajo; `Enter` o un clic abre uno (su motivo, el paso siguiente, el comando, el código de salida y lo que imprimió), `c` o su botón Copiar lo copia, `r` o su botón ejecuta el paso sugerido. `Tab` (o `p`) pasa a la pestaña Aplicación; `Left`/`Right` desplazan lateralmente las líneas largas.
![El Registro de actividad, pestaña Operaciones de git: cada operación con su hora y lo que tardó, un fetch fallido en rojo](../images/activity-log-repository.png)

- **Qué acción produjo un error.** Todo fallo dice qué lo produjo, en la entrada, en el registro de la aplicación y en el informe copiado. La operación se nombra como lo haría una persona (`Quitar remoto`, `Fetch de todos los remotos`, `Petición al host: crear pull request`, `Comando de token`) con sus argumentos principales (`remote=cm6`, una rama, una ruta; nunca el cuerpo de un mensaje, y las URL y los tokens se ocultan como en todas partes), después el repositorio, quién la pidió (`TUI`, `CLI` o `MCP`) y el id de la operación, el número que el daemon dio a la petición (y su número de tarea en un fetch, push o pull). En la pestaña Operaciones de git, la cabecera de la entrada dice `Quitar remoto: error (remote=cm6) - ... [CLI #12]` y sus detalles empiezan por `Operación:`, `Repositorio:`, `Solicitada por:` y `Id de la operación:`; una operación sin título propio se titula con lo que era (`Quitar remoto: error`). En la pestaña Aplicación, la línea del fallo empieza con las mismas palabras y una etiqueta: `Quitar remoto (remote=cm6) [op #12, CLI, repo nombre]: backend error: no such remote: cm6`. Una línea así abre su operación: haz clic en ella, o selecciónala y pulsa `Enter` (el comando de la paleta `Registro de actividad: Mostrar la operación de esta línea`). `]` y `[` seleccionan la línea de fallo siguiente y la anterior (`Registro de actividad: Siguiente operación fallida` / `Registro de actividad: Operación fallida anterior`); con una línea seleccionada, las flechas mueven la selección en lugar de desplazar. La entrada que se abre es la propia de la ventana cuando fue la ventana quien ejecutó la operación; para un fallo de la CLI o de un agente contra la misma sesión, la ventana crea una entrada a partir de la línea. El informe de error copiado tiene líneas `operation:`, `arguments:`, `repository:`, `requested by:` y `operation id:`, así que se puede pegar por sí solo. Desde un script, el error `--json` de un comando fallido no cambia; la respuesta del daemon a una petición fallida lleva los mismos datos en un campo `context` añadido.

- **Líneas largas** (ambas pestañas): una cabecera o una línea de detalle más ancha que la ventana nunca se corta. Aparece una barra de desplazamiento horizontal bajo la lista; desplázate hacia los lados con `Left` / `Right` (reasignables como `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + la rueda o una rueda horizontal, los comandos de la paleta `Registro de actividad: Desplazar a la izquierda` / `Registro de actividad: Desplazar a la derecha`, o la barra (arrastra el control, o pulsa la pista para traer el control bajo el puntero). `Home` vuelve al principio, y al cambiar de pestaña la otra empieza en el borde izquierdo. El texto con saltos de línea (un resumen, una acción sugerida) sigue con sus saltos.

- **Alcance** (un desplegable bajo las pestañas, `s` lo abre, o el comando de la paleta `Registro de actividad: Elegir el alcance`): de qué tratan las dos pestañas. Las filas son **Este repositorio (nombre)** (el repositorio de la pestaña desde la que abriste el registro; no se ofrece desde Inicio, que no tiene ninguno), **Todos los repositorios**, cada **espacio de trabajo** y cada repositorio que conoce la ventana. Desde la pestaña de un repositorio el registro se abre en *Este repositorio*; desde Inicio (el botón Actividad, `Ctrl+N` y luego `Shift+A`), en *Todos los repositorios*; `Ctrl+N` y luego `Shift+G` y la entrada de la paleta `Ver: Registro de actividad global` se abren siempre en *Todos los repositorios*, y también un clic en el error rojo de un fallo cuyo repositorio no es la pestaña activa (un clon fallido, por ejemplo). Tu última elección se recuerda durante la sesión, por separado para las dos formas de abrirlo. En la pestaña Aplicación, una línea pertenece a un repositorio cuando el programa la escribió sobre uno (un push fallido, el tiempo de una operación); en *Todos los repositorios*, cada una de esas líneas dice de quién es (`[nombre]`). Una línea que no trata de ningún repositorio (el arranque, los ajustes, el propio daemon) y una entrada que no es de ninguno (un guardado fallido) son **de la aplicación**: se muestran en todos los alcances, marcadas con `[de la aplicación]` cuando el alcance es más estrecho que *Todos los repositorios*, para que un filtro nunca oculte un fallo. La paleta también tiene `Registro de actividad: Operaciones de git` y `Registro de actividad: Registro de la aplicación` para las pestañas.

- **Copiar diagnóstico** (un botón en ambas pestañas, `d`, siempre el registro entero, sea cual sea el alcance; y `Ver: Copiar diagnóstico` de la paleta) copia la versión, el sistema, el terminal y el backend, el último fallo con sus detalles y las últimas 200 líneas del registro en el nivel elegido: el bloque que dar a una IA cuando algo va mal.

Las duraciones las mide la ventana (de la petición a la respuesta del daemon, viaje de ida y vuelta incluido), así que lo que hacen la CLI o el MCP no se cronometra. `log.timings` (activado por defecto) desactiva las duraciones, `log.extended` (desactivado por defecto) lista también el trabajo en segundo plano, la actualización periódica y el auto-fetch; ambos están en Ajustes > General junto a los demás ajustes del registro, y cada operación se escribe de todos modos en el registro de la aplicación (`info` para lo que pediste, `debug` para el segundo plano).

Los archivos de registro son `gitpoulpi-YYYY-MM-DD.log`, se conservan una semana por defecto y como máximo diez años (`log.retention_days`, de 1 a 3650), se podan al iniciar GitPoulpi y se rotan en `log.max_file_mb` (5 MiB). Dónde están: [referencia de configuración](config-reference.md#log-el-registro-de-la-aplicación). Desde un script, el error `--json` de un comando fallido lleva los mismos datos en `detail`; consulta la [referencia de la CLI](cli-reference.md).

Un clic en el mensaje rojo abre el registro en *Todos los repositorios* con esa entrada abierta. La lista de Operaciones de git es la de la sesión; la pestaña Aplicación conserva el registro del programa entre reinicios.

**Trabajos en segundo plano.** Un fetch, push, pull, clon o transferencia LFS, el Fetch all / Pull all / Clone missing de un espacio de trabajo, una petición de IA y una pull request o incidencia que se está enviando son trabajos: el extremo derecho de la barra de estado muestra el que corre con un `[x]` que lo cancela, y un clic en su etiqueta (o `Ctrl+N` y `Shift+J`, la entrada `Jobs: Show background jobs`) abre el panel con un botón Cancelar por trabajo (también `c` o `Supr`). Un clon corre en segundo plano; una petición de IA y un recorrido de espacio de trabajo sobreviven a un cambio de pestaña. Un trabajo pasado el punto en que detenerlo ya no es seguro aparece en gris con el motivo; un trabajo cancelado deja el repositorio como estaba.

## Respuestas rápidas

**La barra de herramientas y la barra lateral muestran cuadros o signos de
interrogación en lugar de iconos.** Usa una Nerd Font o cualquier fuente con
cobertura de emojis en tu terminal. El grafo y el texto siguen funcionando sin
ellas.

**El grafo de commits está hecho de caracteres, no de imágenes.** Es el
comportamiento alternativo automático: las imágenes necesitan un terminal que hable
el protocolo gráfico de kitty (kitty, Ghostty, herdr con los gráficos de kitty
activados) o el protocolo de iTerm2 (WezTerm, Rio, mintty) o sixel (Windows Terminal, foot, Tabby, Konsole, y tmux cuando el terminal de detrás admite sixel). La consola de
Windows, zellij y screen reciben caracteres. Consulta la sección del grafo más abajo
para forzar una u otra cosa, [Elegir un terminal](terminals.md) y [Multiplexores](multiplexers.md).

**El ratón no hace nada, o seleccionar texto con el ratón no funciona.**
GitPoulpi captura el ratón para que los clics lleguen hasta él. Para seleccionar
texto con el propio terminal, mantén pulsada la tecla `Shift` mientras arrastras (en la
mayoría de los terminales). También puedes seleccionar dentro del diff (arrastra y
luego `y`). Para desactivar la captura del ratón, usa *Ajustes > Teclado y ratón >
Ratón activado* (requiere reiniciar). El teclado llega a todos los comandos.

**Un atajo no funciona.** Tu terminal, tmux o un script de AutoHotkey pueden coger
la tecla antes de que GitPoulpi la vea (`Ctrl+,` suele llegar como una simple `,`
en los terminales de Linux y macOS; `Ctrl+Shift+N` se ve como `Ctrl+N`). Usa las
alternativas (`Ctrl+N` y luego `,` abre los Ajustes; `Ctrl+P` llega a todos los
comandos), o reasigna la combinación: consulta [Ajustes](settings.md#teclado-y-ratón).

**`Ctrl+Tab` no cambia de pestaña.** kitty, WezTerm, Ghostty y Windows
Terminal se quedan con `Ctrl+Tab` y `Ctrl+Shift+Tab` para sus propias pestañas,
así que la tecla nunca llega a GitPoulpi. `Ctrl+PageDown` / `Ctrl+PageUp`
funcionan siempre. Para dejar el par a GitPoulpi, libéralo en la configuración
del terminal y reinicia el terminal (o recarga su configuración):

- kitty, en `kitty.conf` (un `map` sin acción quita la asignación por defecto;
  con `--when-focus-on` y un criterio de ventana como `title:...`, kitty puede
  quitarla solo mientras una ventana que coincida tiene el foco, consulta la
  documentación de asignaciones de kitty):

  ```
  map ctrl+tab
  map ctrl+shift+tab
  ```

- WezTerm, en `wezterm.lua`:

  ```lua
  config.keys = {
    { key = 'Tab', mods = 'CTRL', action = wezterm.action.DisableDefaultAssignment },
    { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.DisableDefaultAssignment },
  }
  ```

- Ghostty, en su archivo de configuración:

  ```
  keybind = ctrl+tab=unbind
  keybind = ctrl+shift+tab=unbind
  ```

- Windows Terminal, en `settings.json` (`"id": null` quita la asignación por
  defecto):

  ```json
  "keybindings": [
    { "id": null, "keys": "ctrl+tab" },
    { "id": null, "keys": "ctrl+shift+tab" }
  ]
  ```

  (Los archivos de configuración antiguos listan `nextTab` / `prevTab` en
  `actions` con estas teclas: quita esas teclas allí.)

Después, el terminal pierde su propio cambio de pestaña con `Ctrl+Tab`. Un
terminal que solo puede enviar la tecla mediante el protocolo de teclado de
kitty necesita tenerlo activado (lo está por defecto, consulta
[Atajos de teclado](keybindings.md#teclas-que-el-terminal-no-puede-enviar-y-el-protocolo-de-teclado-de-kitty)).

**No recuerdo un atajo.** Pulsa `Ctrl+P` y escribe lo que quieres; cada resultado
muestra su atajo. `?` lista todo.

**Una opción aparece atenuada en los Ajustes.** Depende de otra opción. La fila
dice cuál (por ejemplo, *Intensidad del tinte de fila* necesita que *Fondo de los
carriles del grafo* esté en *Tinte de fila*).

**«Auto-stash pendiente» sobre el Log.** GitPoulpi hizo stash de tus cambios para
ejecutar una operación y no pudo devolverlos. Usa **Pop now**, **Show stash**,
**Empezar de nuevo** (deshace la operación y devuelve tus cambios) o **Drop** del
aviso; consulta [Auto-stash](log-and-history.md#auto-stash).

**Un agente dice `no_viewer` o `deferred`.** `no_viewer`: no hay ninguna ventana de
GitPoulpi abierta para ese repositorio (o *Permitir que los agentes de IA controlen
la vista* está desactivado). `deferred`: la ventana muestra un chip; `Ctrl+N Enter`
lo acepta. Consulta [MCP y agentes](mcp-and-agents.md#mostrar-cosas-al-usuario).

**Abrí GitPoulpi y veo una ventana vacía o `log error: backend error`.**
Comprueba que lo iniciaste dentro de un repositorio Git (si no, acabas en la
pestaña Inicio: usa **Examinar**, **Clonar** o **Init**). Luego ejecuta `gitpoulpi
daemon status`; `gitpoulpi daemon restart` reinicia el proceso en segundo plano de
este repositorio. Pulsa `r` para actualizar.

**Dos ventanas sobre el mismo repositorio se comportan de forma extraña.**
Comparten un daemon por repositorio a propósito, de modo que un cambio en una se
ve en la otra. Si un daemon parece atascado, `gitpoulpi daemon stop --all` detiene
el daemon de todas las sesiones; la siguiente ventana inicia uno nuevo.

**Un commit, pull o push pide una contraseña, o falla al autenticarse.**
GitPoulpi usa el credential helper y el ssh-agent propios de tu `git` para la red.
Asegúrate de que `git fetch` funciona primero en un terminal normal. Los tokens de
GitHub/GitLab para las pull requests son aparte y opcionales: consulta
[Alojamiento y gestores de issues](hosting-integration.md).

**Un fetch, pull o push falla con «The configured credential helper failed», «The SSH host key of the remote is not known yet» o un inicio de sesión rechazado, y su entrada del Registro de actividad tiene un botón «SSH y credenciales...».**
El fallo viene de cómo inicia sesión este repositorio. El botón abre
Ajustes > SSH y credenciales, donde eliges otro origen para este
repositorio: los helpers propios de git con los del repositorio ignorados, una
cuenta de integración, un archivo de clave (con un archivo known_hosts) o un helper o
comando de token tuyo; **Probar conexión** comprueba después todos los remotos. Para un
helper que no se pudo ejecutar, `git config --show-origin --get-all
credential.helper` muestra dónde está definido.

**«GitPoulpi no envió el token de la cuenta a ...».** El remoto del repositorio
apunta a un servidor que no es la dirección a la que está vinculada la cuenta elegida
(a menudo una reescritura `url.<x>.insteadOf` en la config del propio repositorio), así que
el token se retuvo. Corrige el remoto o elige otro origen en la página.

**«Falló el comando de token de este repositorio», «... ha cambiado desde que lo confirmaste», «No se puede usar el inicio de sesión de este repositorio».**
La página no puede llevar a cabo lo elegido: el comando de token no imprimió ningún token
(se citan sus últimas líneas de error, sin secretos), un script que ejecuta
ha cambiado desde que lo confirmaste (vuelve a confirmar el comando en la página), o
la elección nombra un archivo de clave o una cuenta que ya no existe. Cada mensaje tiene el
botón **SSH y credenciales...**.

**OpenSSH dice «bad permissions» para un archivo de clave.** Otras cuentas pueden leer la
clave. En Ajustes > SSH y credenciales, la fila **Archivo de clave privada** muestra una
advertencia con **Restringir a mi cuenta**.

**«haciendo stage...» se queda en la barra de estado.** Sigue ejecutándose una actualización en
segundo plano; desaparece cuando se actualiza la lista de archivos. `r` fuerza una
actualización.

**Un repositorio muy grande va lento.** Baja *Commits iniciales en el grafo* en la
página Ajustes > Historial y grafo (el Log carga más al desplazarte cuando *Carga
diferida de commits en el grafo* está activada); oculta la columna *Cambios* del Log y desactiva los
avatares; desactiva *Deshacer activado* si no necesitas deshacer (así no hay
ninguna contabilidad en el staging ni en los commits).
`GITPOULPI_TIMING=1` escribe los tiempos de cada petición en
`gitpoulpi-timing.log` en la carpeta temporal, que conviene adjuntar a un informe
de rendimiento.

**Los botones de IA faltan o están en gris.** La IA está desactivada por defecto;
consulta [Asistencia de IA](ai.md#solución-de-problemas).

**GitPoulpi se cerró solo.** Cada salida queda registrada: busca
`gitpoulpi-tui-exit.log` (y `gitpoulpi-tui-panic.txt` tras un fallo) en el
directorio de configuración (`gitpoulpi config path` imprime el archivo de ajustes,
que está ahí) y adjúntalo a un issue.

**¿Dónde están mis ajustes?** `gitpoulpi config path` imprime `config.toml` (los
ajustes comunes) y `tui.toml` (los ajustes de interfaz de la interfaz de terminal:
tema, disposición, columnas, atajos...), que están en la misma carpeta.
`gitpoulpi config get <clave>` indica en cuál de los dos está una clave. El estado
por repositorio está en `.gitpoulpi/repo-state.toml` dentro del repositorio.
Consulta [Dos archivos](config-reference.md#dos-archivos-configtoml-y-tuitoml).

**Volví a un GitPoulpi más antiguo y mis ajustes de interfaz cambiaron (o el más
nuevo ignora lo que fijé en el antiguo).** GitPoulpi guarda los ajustes de interfaz
en `tui.toml`, creado como una copia de lo que contenía `config.toml` y nunca
retirado de `config.toml`. Un GitPoulpi más antiguo instalado junto al nuevo no
conoce `tui.toml`: sigue leyendo y escribiendo su propio `config.toml` completo.
Una vez que `tui.toml` existe, el nuevo lee los ajustes de interfaz solo de él, así
que un tema o una disposición que cambies después en la versión antigua no se
recoge, y una clave que falta en `tui.toml` toma su valor por defecto en lugar del
valor antiguo de `config.toml`. Para poner al día la versión nueva, cambia el ajuste
ahí (Ajustes, o `gitpoulpi config set`); para reiniciar los ajustes de interfaz
desde `config.toml`, cierra GitPoulpi y borra `tui.toml` (se crea de nuevo, como una
copia nueva, en el siguiente arranque).

**¿GitPoulpi envía algo a algún sitio?** Solo lo que pides: operaciones de git a
tus remotos, peticiones a los alojamientos después de conectar una cuenta,
peticiones de IA después de pulsar un botón de IA y, si activas los avatares, un
hash de los correos de los autores a Gravatar. No hay telemetría. Consulta
[Asistencia de IA](ai.md#privacidad).

## Todas las escrituras fallan: «Unable to create index.lock»

Un comando de git que murió (un fallo, un terminal cerrado a la fuerza) puede dejar
`.git/index.lock` atrás, y entonces falla cualquier cambio en el repositorio. Cuando
el archivo tiene más de 10 segundos, GitPoulpi muestra una franja sobre el Log:
**Eliminar index.lock** (paleta: *Repositorio: Eliminar index.lock obsoleto*; CLI:
`gitpoulpi index-lock remove`). Pregunta antes y elimina solo ese archivo.
GitPoulpi no puede saber con fiabilidad si otro programa git sigue en ejecución, así
que asegúrate de que no hay ninguno (un IDE, otro terminal) antes de confirmar. En
un worktree vinculado, el bloqueo es el propio del worktree, en
`.git/worktrees/<name>`.

## «git am» se detuvo y Abortar el rebase no hace nada

Un `am` (aplicar parches desde un buzón) no es un rebase. Un `git am` detenido tiene
su propia franja, **Aplicando parches (git am)**, con Resolver conflictos,
Continuar, Omitir y Abortar (paleta: *git am: Continuar / Omitir este parche /
Abortar*; CLI: `conflicts am-continue|am-skip|am-abort`). Abortar pregunta antes,
descarta los parches aplicados hasta ahora y devuelve la rama a donde estaba. Estos
botones no se pueden deshacer, igual que los del rebase.

## Deshacer está en gris y dice «Can't undo ...» (entrada bloqueada)

La entrada más reciente del diario está atascada: el repositorio avanzó fuera de
GitPoulpi, o la entrada se interrumpió. Pasa el puntero sobre Deshacer para ver el
motivo; si está atascada para siempre, el tooltip señala **Edición: Olvidar la
entrada de deshacer bloqueada** (paleta) o `gitpoulpi undo-forget`. Pregunta antes y
solo quita la entrada del diario; el repositorio no se toca. Consulta
[Deshacer y rehacer](undo-redo.md).

## Los archivos grandes se muestran como tres líneas de texto («los archivos LFS se muestran como punteros»)

Un archivo que debería ser una imagen, un modelo o un binario grande contiene `version https://git-lfs.github.com/spec/v1` y una línea `oid sha256:...`: es un **puntero** de Git LFS, y el contenido real no se ha descargado (o los filtros de LFS no están instalados).

- Mira el panel Estado: un repositorio que usa LFS mientras falta `git-lfs` muestra un aviso ámbar. Instala el programa (https://git-lfs.com; Windows: incluido con Git for Windows o `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) y reabre o pulsa `r`.
- `gitpoulpi lfs status` dice si el programa está instalado, si los filtros y el hook están configurados y qué patrones se rastrean; `gitpoulpi lfs files` lista los archivos puntero y cuáles son `local`.
- Con `git-lfs` instalado, ejecuta **`LFS: Pull de objetos`** en la paleta (o `gitpoulpi lfs pull`): descarga los objetos y sustituye los punteros. Instala antes los filtros de LFS para este repositorio si no hay ninguno.
- Si la descarga falla, el mensaje nombra el motivo (credenciales del remoto, un servidor que ya no tiene el objeto). GitPoulpi no necesita ninguna cuenta propia: LFS usa las credenciales habituales de tu remoto (credential helper o ssh-agent).
- Un clonado de un repositorio LFS nunca falla por culpa de LFS; si el paso de descarga no pudo ejecutarse, el clonado termina con punteros, y `LFS: Pull de objetos` lo termina más tarde. `gitpoulpi clone --no-lfs` omite la descarga a propósito.
- Puedes desactivar todo lo de LFS en Ajustes > Git y codificación > Git LFS (o `[git] lfs_enabled = false`).

## «git-lfs not found» / «Git LFS is not installed»

El mensaje `Git LFS is not installed. This repository stores large files with Git LFS; ...` (o un diálogo con un botón *Abrir página de instalación*, o el propio `git-lfs: command not found` / `'lfs' is not a git command` de git) significa que el repositorio está configurado para LFS - su `.gitattributes`, los filtros de `.git/config` o el hook `pre-push` llaman a `git-lfs` - y el programa no está en el `PATH` que ve GitPoulpi. Instálalo (enlace arriba). En Windows el programa viene con Git for Windows; si `git lfs version` funciona en un terminal pero GitPoulpi sigue diciendo que falta, inicia GitPoulpi desde un terminal que vea el mismo `PATH`. Para ocultar LFS a GitPoulpi sin alterar el comportamiento propio de git, desactiva `git.lfs_enabled`.

## El grafo se dibuja con caracteres, no como imagen (herdr, tmux, otros terminales)

GitPoulpi dibuja la columna Grafo y los avatares de los autores como píxeles solo donde está seguro de que el terminal los mostrará: imágenes en línea de iTerm2 en WezTerm a secas, Rio, mintty y iTerm2 anterior a 3.7.7; el protocolo gráfico de Kitty en kitty, Ghostty, iTerm2 3.7.7 o posterior y **herdr cuando la propia config de herdr establece `kitty_graphics = true`** (`[terminal]`, o `[experimental]` en herdr más antiguos) y herdr se ejecuta dentro de WezTerm, kitty o Ghostty. Las imágenes sixel se usan en Windows Terminal, foot, Tabby, Konsole y mlterm (y en tmux cuando el terminal de detrás admite sixel). En todos los demás casos dibuja caracteres, lo cual es siempre correcto.

- En herdr con los gráficos de Kitty activados pero con una config que GitPoulpi no encuentra (o dependiendo del valor por defecto de herdr), establece `ui.image_protocol = "kitty"` (Ajustes > Historial y grafo > Protocolo de imagen) y reinicia.
- En un terminal que muestra cuadros o basura donde debería estar el grafo, establece `ui.image_protocol = "none"` o `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` desactiva las imágenes sea cual sea la config.

**herdr remoto (`herdr --remote <host>`, o `ssh` y luego `herdr`): el grafo de texto es el valor por defecto.** Por SSH, GitPoulpi se ejecuta en el host remoto y solo puede leer el entorno del remoto y el `config.toml` de herdr. Que aparezca una imagen de Kitty lo decide el cliente herdr *local* (su propio ajuste `kitty_graphics` y su propio terminal), que GitPoulpi no puede ver: cuando el cliente no dibuja las imágenes, las celdas quedan en blanco. Por eso GitPoulpi dibuja el grafo de texto siempre que detecta herdr por SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Si tu herdr local tiene `kitty_graphics = true` y un terminal que dibuja imágenes de Kitty (kitty, Ghostty), actívalo con `ui.graph_renderer = "image"` (o `ui.image_protocol = "kitty"`) en el remoto. Las imágenes se envían en línea (`t=d`), lo que funciona sobre cualquier flujo de bytes. Si la columna queda vacía, establece `ui.graph_renderer = "text"`.

**Un rectángulo más claro detrás de los carriles.** La imagen es transparente (PNG RGBA con alfa directo) donde no se dibuja nada, de modo que el fondo del terminal y la banda de selección se transparentan. Los terminales con protocolo Kitty, **WezTerm** y **Rio** componen ese alfa; GitPoulpi entonces reserva celdas en blanco normales bajo la imagen y la envía después de ellas. Cualquier otro terminal con protocolo iTerm2 (iTerm2 anterior, mintty) no se sabe que mezcle el alfa del PNG, así que ahí la imagen lleva un fondo opaco del color del tema, que puede verse como un rectángulo en un terminal cuyo fondo difiere de él; `ui.graph_renderer = "text"` lo elimina. Con `ui.graph_lane_background = "off"` no se pinta nada detrás de los carriles en las rutas transparentes.

**Windows: la columna del grafo está en blanco, en las filas equivocadas, o la pantalla se rompe (WezTerm 20240203, Rio, mintty).** La pseudoconsola de esos terminales pierde o descoloca las secuencias de imagen ([Elegir un terminal](terminals.md#windows)). Establece `ui.graph_renderer = "text"`, o usa Windows Terminal.

Dentro de tmux, zellij, screen y herdr consulta [Multiplexores](multiplexers.md) para saber qué pasa y qué configuración añadir.

Consulta `ui.graph_renderer` y `ui.image_protocol` en la [referencia de configuración](config-reference.md).
