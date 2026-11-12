[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · **Español** · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS («Large File Storage») mantiene los archivos grandes (archivos de diseño, vídeos, conjuntos de datos) fuera del historial de un repositorio: git guarda en su lugar un *puntero* de texto de tres líneas, y el contenido real vive en el servidor LFS y en una caché local bajo `.git/lfs`.

GitPoulpi no necesita ninguna cuenta para nada de esto. LFS usa las credenciales que ya usa tu remoto (tu credential helper de git, o el ssh-agent).

## Qué necesita el programa `git-lfs` y qué no

GitPoulpi ejecuta el programa estándar `git-lfs` para todo lo que convierte o mueve archivos (install, track, fetch, pull, push, bloqueos).

Leer el *estado* de LFS no necesita ningún programa: si un repositorio usa LFS, qué archivos son punteros, cuánto ocupan y si el objeto real ya está almacenado en local.

| | necesita `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (informa de que falta el programa en lugar de fallar) | no |
| `gitpoulpi lfs files` | no |
| install / track / untrack / prune | sí |
| fetch / pull / push | sí |
| bloqueos | sí |

## Instalar `git-lfs`

- **Windows**: viene incluido con Git for Windows; o `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Cualquier otro sistema: https://git-lfs.com

Se requiere la versión 3.0 o posterior. Tras instalarlo, GitPoulpi lo detecta en menos de un minuto (o al instante con `lfs status`).

## Cuando falta `git-lfs`

GitPoulpi nunca se cierra ni se bloquea por ello:

- `gitpoulpi lfs status` termina con éxito y dice `git-lfs: not installed`, con el enlace de instalación.
- Una operación que necesita el programa falla con un mensaje claro que termina con el enlace de instalación (`https://git-lfs.com`) y un código de salida distinto de cero; por el MCP, la herramienta devuelve ese mensaje.
- Un repositorio que usa LFS se abre igualmente; sus archivos grandes se muestran simplemente como punteros hasta que se instale el programa.

## Ver los archivos LFS

```
gitpoulpi lfs status            # programa, patrones, filtros, hook
gitpoulpi lfs files             # archivos puntero del índice, con tamaño y local / no descargado
gitpoulpi lfs files --at v1.0   # ... en una revisión concreta
gitpoulpi show big.bin --at HEAD  # informa «Git LFS object, 12.3 MB, sha256:...» en lugar del texto del puntero
```

## Preparar un repositorio: install, track, untrack

```
gitpoulpi lfs install                 # filtros + hooks de ESTE repositorio (git lfs install --local)
gitpoulpi lfs install --user          # en tu configuración global de git (nunca se hace en silencio)
gitpoulpi lfs track '*.psd'           # pasa los *.psd por LFS
gitpoulpi lfs track '*.psd' --lockable   # ... y los deja de solo lectura hasta que se bloqueen
gitpoulpi lfs track '*.bin' --renormalize  # ... y vuelve a poner en stage como punteros los *.bin ya confirmados
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # elimina del almacén local los objetos antiguos ya enviados
```

- `install` escribe por defecto en el propio `.git/config` y en los hooks del repositorio, así que tu configuración global de git nunca se toca salvo que pases `--user`.
- `track` y `untrack` editan `.gitattributes` y lo dejan **sin pasar a stage**: aparece como un cambio normal del árbol de trabajo, del que haces commit tú mismo. Los archivos añadidos *después* de empezar el seguimiento se guardan como punteros en su siguiente commit.
- Los archivos que **ya están en un commit** como contenido normal siguen siendo normales en el historial. `--renormalize` los vuelve a poner en stage para que tu siguiente commit los sustituya por punteros. Para moverlos también en todos los commits pasados, usa la regla de reescritura *Migrar archivos a LFS* (véase [Reescritura del historial](history-rewrite.md#mover-archivos-a-git-lfs-o-traerlos-de-vuelta)); es propia de GitPoulpi, no `git lfs migrate`.
- Entrecomilla el patrón para que tu shell no lo expanda.
- Por el MCP son mutaciones normales (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), sujetas al control `[mcp]` de `config.toml`.

## Mover objetos: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # descarga objetos al almacén local; el árbol de trabajo no se toca
gitpoulpi lfs pull  [--remote origin]           # fetch y después sustituye los punteros por el contenido real
gitpoulpi lfs push  [--remote origin] [--all]   # sube los objetos de la rama actual (o de todas las refs con --all)
```

- El remoto es por defecto el remoto predeterminado del repositorio, igual que `fetch` y `push`.
- Las credenciales son las habituales de tu remoto (credential helper, ssh-agent). GitPoulpi nunca pide una cuenta propia.
- `pull` instala los filtros LFS de este repositorio si faltan: sin ellos, git-lfs descarga los objetos pero no sustituye los punteros, y un checkout posterior volvería a poner punteros.
- El MCP trata `lfs push` como `push`: una herramienta destructiva (cambia el remoto).
- La ventana muestra el progreso de una transferencia (archivos y bytes) en la barra de estado. La CLI espera el resultado.

### Escribir el contenido: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # sin red: escribe el contenido real sobre los punteros, desde el almacén local
```

Nunca descarga (eso es `lfs pull`); los objetos que faltan en el almacén local siguen siendo punteros. Como `pull`, instala antes los filtros LFS del repositorio cuando no hay ninguno configurado. Como toda operación LFS, no se puede deshacer; consulta [Deshacer y rehacer](undo-redo.md). Por el MCP es `lfs_checkout` (una escritura, no destructiva).

### El botón LFS de la barra de herramientas, y pull / push

La barra de herramientas tiene un botón **`LFS ▾`** después de **Pop** (oculto cuando `git.lfs_enabled` está desactivado, y en un repositorio donde LFS no está inicializado ni lo usa su `.gitattributes`: consulta [Ajustes > LFS](settings.md#ajustes-del-repositorio-lfs)). Un clic abre un desplegable: **Fetch de todos los archivos LFS** (`lfs fetch --all`), **Checkout de todos los archivos LFS** (`lfs checkout`), **Pull de todos los archivos LFS** (`lfs pull`), **Push de todos los archivos LFS** (`lfs push --all`, los objetos de todas las refs) y **Limpiar el almacén LFS local** (`lfs prune`). Cada fila es también un comando de la paleta (`LFS: Fetch de todos los objetos`, `LFS: Checkout de todos los archivos`, `LFS: Pull de objetos`, `LFS: Push de todos los objetos`, `LFS: Podar objetos locales`). `LFS: Abrir menú` abre el desplegable. Cada uno tiene un acorde `Ctrl+G`: `l` para el menú, y después `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` y `Shift+V` para las filas. Puedes cambiarlos en *Ajustes > Teclado y ratón*. Si falta `git-lfs`, todas las filas llevan al diálogo de instalación.

![El desplegable LFS de la barra de herramientas: fetch, checkout, pull y push de todos los archivos LFS, y limpieza del almacén local](../images/lfs-menu.png)

**Pull y push gestionan LFS por sí solos.** En un repositorio que usa LFS, con `git-lfs` instalado, un `pull` normal comprueba primero que los filtros LFS del repositorio están instalados, y un `push` normal comprueba su hook `pre-push`. Cuando falta alguno, GitPoulpi ejecuta `git lfs install --local` (solo este repositorio) antes de la transferencia, y la barra de estado dice «Filtros LFS instalados». Sin el hook, un push no subiría ningún objeto LFS. Desactívalo con `git.lfs_autofix_filters = false` (*Ajustes > Git y codificación > Instalar los filtros LFS antes de pull y push*). Un `push` normal sigue subiendo solo los objetos de la rama enviada, nunca los de todas las refs; el prune nunca es implícito.

## Bloqueos de archivos

Algunos archivos no se pueden fusionar (un archivo de Photoshop, un nivel, un recurso binario). Los *bloqueos* de Git LFS permiten que una persona diga «estoy editando esto»: nadie más puede bloquearlo ni enviar cambios sobre él hasta que se desbloquee.

```
gitpoulpi lfs locks  [--remote origin]              # quién tiene bloqueado cada archivo (los tuyos van marcados)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- El bloqueo es una función del **servidor LFS** (GitHub, GitLab y Gitea la tienen). Se accede a él con las credenciales del propio remoto; GitPoulpi no tiene cuenta propia.
- Un servidor sin bloqueos hace que `lock` falle con un mensaje que lo dice (el tipo `LfsLockingUnsupported`). `locks` en ese servidor no lista nada, que es lo que informa el propio git-lfs.
- Seguir un patrón con `--lockable` (`lfs track '*.psd' --lockable`) hace que esos archivos sean de solo lectura hasta que los bloquees.
- `unlock` del bloqueo de otra persona se rechaza; `--force` lo rompe, y por eso el MCP trata `lfs_unlock` como una herramienta destructiva. `lfs_locks` es de solo lectura.

## Clonar un repositorio LFS

`gitpoulpi clone <url>` (y el diálogo Clone de la TUI) hace checkout del repositorio *sin* ejecutar los filtros LFS y, cuando el repositorio usa LFS y `git-lfs` está instalado, descarga los objetos con `git lfs pull` (instalando los filtros de ese repositorio si no hay ninguno configurado) y lo muestra como una fase propia, *Descargando objetos LFS*. `--no-lfs` omite ese paso y deja punteros.

Un clon **nunca falla por culpa de LFS**: si falta `git-lfs`, o una descarga falla, el clon termina igualmente con éxito y los archivos grandes se quedan como punteros; el estado LFS del repositorio explica por qué, y `gitpoulpi lfs pull` termina el trabajo una vez resuelto el problema.

## En la interfaz de terminal

- **La marca `LFS`.** Un archivo guardado con Git LFS lleva una pequeña marca `LFS` tras su nombre en las listas Unstaged y Staged del panel Estado y en la lista de archivos modificados de un commit.
- **La tarjeta del diff.** Al abrir el diff de un archivo LFS aparece una tarjeta en lugar del texto del puntero: `Git LFS object`, `1.2 MB -> 1.4 MB`, los oids cortos, el antiguo y el nuevo, y si el objeto nuevo está almacenado en local o `not downloaded`. Pulsa `r` para ver el diff del puntero en bruto, y `r` otra vez para volver. A un archivo LFS solo se le aplica el stage a nivel de archivo (no hay hunks ni líneas que poner en stage). Bajo la tarjeta, **Abrir en predeterminado** y **Abrir en** (`o`, `Shift+O` o la paleta) abren el archivo real, nunca el puntero: consulta [Abrir en](open-in.md#archivos-git-lfs).
- **La Vista de archivo** de un archivo LFS dice `Git LFS object, 12.3 MB, sha256:ab12cd34...` y dónde conseguir el contenido.
- **El banner.** Si un repositorio usa LFS mientras `git-lfs` falta o es anterior a la 3.0, un banner del panel Estado lo indica. Haz clic en él (o usa la entrada de la paleta *LFS: Abrir página de instalación*) para abrir https://git-lfs.com. No cambia nada más: la ventana se abre, los diffs funcionan y los archivos grandes se muestran simplemente como punteros.
- **El menú del archivo y la paleta.** Haz clic derecho en un archivo modificado del panel Estado: *Seguir con LFS: este archivo* / *esta extensión* para un archivo normal; *Bloquear archivo (LFS)*, *Desbloquear archivo (LFS)* o *Forzar desbloqueo (LFS, bloqueo de otra persona)* para un archivo LFS, según quién tenga su bloqueo. La fila de un archivo bloqueado dice `bloqueado por ti` / `bloqueado por <nombre>`. Todo está también como comando de la paleta (`Ctrl+P`, escribe `LFS`): inicializar, seguir o dejar de seguir un patrón, fetch, pull o push de objetos, prune, mostrar bloqueos, abrir la página de instalación.
- **Una acción fallida se explica sola.** Si una acción LFS falla porque falta `git-lfs`, un diálogo muestra el mensaje y un botón *Abrir página de instalación*.
- **Ajustes.** *Ajustes > LFS* (por repositorio) indica si LFS está inicializado, lista los patrones con seguimiento y tiene **Configurar LFS aquí**. *Git y codificación* (por repositorio) tiene el interruptor **Git LFS**, una fila de solo lectura **Programa git-lfs** y un botón **Inicializar LFS en este repositorio**.
- Con **Git LFS** desactivado (`git.lfs_enabled = false`, consulta la [referencia de configuración](config-reference.md)) todo esto se oculta. Los filtros LFS del propio git siguen ejecutándose.
- El estado de LFS se lee poco después de abrir el repositorio, tras cada cambio, en una actualización manual, cuando cambia `.gitattributes` y cada diez segundos en un repositorio que usa LFS.

## Límites conocidos

- Un repositorio cuyos patrones LFS viven solo en un `.gitattributes` *anidado* (no en el de la raíz ni en `.git/info/attributes`) se detecta como que usa LFS una vez que tiene un almacén de objetos LFS (`.git/lfs`), no antes.
- `git lfs migrate` no se usa. Mover archivos a LFS o sacarlos de él a lo largo del historial es cosa de las reglas *Migrar archivos a LFS* / *Recuperar archivos de LFS* de la reescritura del historial: trabajan sobre el clon local, solo se escribe el `.gitattributes` de la raíz y los objetos aún hay que enviarlos después (*Enviar objetos LFS* en el panel del resultado, o `lfs push --all`).
