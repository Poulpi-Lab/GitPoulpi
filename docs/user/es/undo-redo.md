[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · **Español** · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Deshacer y rehacer

**Siempre puedes volver atrás.** Cada cambio que GitPoulpi hace en tu repositorio se
registra (hasta 100 acciones por defecto, `general.undo_depth`), y también cada posición
que ha tenido una rama, incluidos los movimientos hechos por otros programas. Si te
equivocas, una de estas vías recupera el trabajo:

| Quieres | Usa |
|---|---|
| revertir lo último | `Ctrl+Z` (rehacer: `Ctrl+Y` o `Ctrl+Shift+Z`), o los botones **Deshacer** y **Rehacer** de la barra de herramientas |
| revertir varias cosas a la vez, o una de hace mucho | el **Historial de deshacer** (`Alt+Z`, la flecha junto a Deshacer, `Edición: Historial de deshacer...`) |
| recuperar una rama, un tag, un stash, un archivo o un hunk que quitaste hace varias acciones, **sin deshacer lo que hiciste después** | **Restaurar solo esta** en el Historial de deshacer |
| devolver una rama a donde estaba, aunque otro programa la haya movido | el **Reflog** de la rama (menú de la ref, `Reflog...`) |
| encontrar commits que ya no alcanza ninguna rama | **Trabajo perdido** (`Edición: Recuperar commits perdidos...`) |
| revertir lo que un shell o un **agente de IA** hizo en ramas, tags, stashes o HEAD | el mismo `Ctrl+Z`: esos cambios también están en el historial, marcados como **externos** (ver [Cambios hechos fuera de GitPoulpi](#cambios-hechos-fuera-de-gitpoulpi)) |

Después de cada acción que destruye algo y se puede deshacer (descartar, eliminar un
archivo, eliminar una rama o un tag, hard reset, descartar un stash, quitar un remoto), la
barra de estado lo dice: «descartar hecho - Ctrl+Z para deshacer», con la tecla tal como la
tengas asignada. El tooltip de Deshacer indica cuántas acciones puedes retroceder.

`Ctrl+Z` revierte lo último que GitPoulpi hizo en tu repositorio. `Ctrl+Y` (o
`Ctrl+Shift+Z`) lo vuelve a poner. La barra de herramientas tiene los botones **Deshacer** y
**Rehacer**, la paleta tiene `Edición: Deshacer` y `Edición: Rehacer`, y el tooltip de cada
botón dice qué revertiría («Deshacer commit “fix typo” (a81b9e4)»).

![La barra de herramientas tras Stage todo: Deshacer está activo y su tooltip nombra la operación que revertiría](../images/undo-toolbar.png)

Rehacer nunca vuelve a ejecutar la operación. Restaura el estado que se registró, así que un
commit rehecho es exactamente el mismo commit, con el mismo id y la misma marca de tiempo.

`Ctrl+Z` no hace nada mientras un campo de texto tiene el teclado (el mensaje del commit, el
cuadro de filtro, el editor, que lleva su propio deshacer de texto).

## El Historial de deshacer

`Alt+Z` (reasignable), la flecha junto al botón Deshacer o `Edición: Historial de deshacer...`
abre un panel con todas las acciones registradas, de la más reciente a la más antigua: un
icono según el tipo de acción (commit, rama, tag, stash, reset, merge y similares, descarte,
stage, configuración, cambio externo), qué hizo en tu idioma, hace cuánto y su estado
(**restaurable**, **bloqueada** con el motivo, o **deshecha** y lista para rehacer). Cuando las
entradas abarcan varios días, un encabezado fino (Hoy, Ayer y luego la fecha) abre cada día. El
encabezado las cuenta («Se pueden deshacer 12 acciones (hasta 100)»).

Las acciones que has deshecho quedan **encima** de la más reciente, bajo un divisor
(«deshechas - Rehacer las recupera»), atenuadas, y la que Rehacer recupera primero queda junto
al divisor. La selección empieza en la acción más reciente, no en ellas. Las flechas mueven la
selección (los encabezados y el divisor se saltan), `1` / `2` / `3` (o `Ctrl+Tab` /
`Ctrl+Shift+Tab`) alternan entre las tres vistas y `Esc` cierra. `Enter` nunca ejecuta una
acción: cada botón tiene su propia letra, que se muestra en la línea de ayuda bajo los
botones. Para la fila seleccionada:

- **Deshacer hasta aquí** (`d`): deshace esa acción y todas las más recientes, de la más
  reciente a la más antigua, tras una sola confirmación que las lista todas. Un paso que el
  daemon considera arriesgado vuelve a preguntar antes de ejecutarse.
- **Restaurar solo esta** (`s`): recupera lo que quitó esa única acción y conserva todo lo que
  se hizo después. Una rama o un tag eliminados vuelven en el commit que tenían (si su nombre
  ya está en uso se te pide otro, con uno libre ya escrito); un stash descartado se devuelve
  a lo alto de la lista de stashes; un archivo descartado o eliminado se escribe de nuevo en
  su sitio si no ha cambiado desde entonces, y si no, a su lado como `nombre.restored` (nunca
  se sobrescribe nada); un remoto, un upstream, una línea de ignorados o una línea de
  atributos quitados se añaden de nuevo. Un reset, amend, rebase, merge o commit recibe una
  rama `restore/<rama>-<sha>` en la punta antigua: tu rama actual y tus archivos nunca se
  mueven. La confirmación muestra exactamente lo que hará, y la restauración es una acción
  nueva: `Ctrl+Z` la revierte.
- **Rehacer hasta aquí** (`r`) en una fila deshecha.
- **Restaurar main a donde la dejó Commit** (`t`, o `Alt+Shift+Z` /
  `Edición: Restaurar la rama a donde la dejó la acción bloqueada` desde fuera del panel):
  cuando otro programa movió la rama y bloqueó Deshacer, el reflog todavía sabe dónde la dejó
  la acción.

Haz clic derecho en una fila (o pulsa `.`) para ver las mismas acciones como menú.

![El Historial de deshacer tras eliminar una rama y hacer dos commits: la fila de la rama eliminada está seleccionada y se puede restaurar por sí sola](../images/undo-history.png)

## El Reflog

El menú de la ref de una rama local tiene **Reflog...**; `Edición: Reflog...` muestra el de
HEAD, y el Historial de deshacer tiene una pestaña para él. Lista las posiciones que ha tenido
la rama, de la más reciente a la más antigua: SHA corto, el motivo registrado, hace cuánto y
quién, incluidos los movimientos hechos en un shell. El mensaje y los archivos cambiados de la
posición seleccionada se muestran junto a la lista.

El reflog se lee de 200 posiciones en 200. Uno más largo sigue solo: mover el cursor (o
desplazarte) cerca del final de la lista trae la página siguiente, y la última fila dice
**... Cargar más** mientras queda más. **Cargar más** (`m`, o `Reflog: Cargar más` en la
paleta) la trae de inmediato y dice «posiciones cargadas: N» en la barra de estado. Los repositorios que
usan el formato reftable de git se leen como cualquier otro.

![El Reflog de main: todas las posiciones que ha tenido, de la más reciente a la más antigua, con la seleccionada descrita a la derecha](../images/reflog.png)

**Mostrar en el Log** (`l`), **Crear rama
aquí** (`b`), **Crear tag aquí** (`t`), **Aplicar al árbol de trabajo** (`a`), **Guardar como
stash** (`s`) (las tres últimas funcionan como en Trabajo perdido, más abajo) y
**Restaurar `<rama>` a esta posición** (`r`). La restauración se puede deshacer.
Cuando la rama tiene checkout y hay cambios locales que no puede mover sin sobrescribirlos, un
desplegable pregunta qué hacer con ellos: conservar mis cambios, soft, mixed o hard. Hard
descarta los cambios de tus archivos, pero antes los guarda, así que `Ctrl+Z` todavía los
recupera (lo dice en rojo).

## Trabajo perdido

`Edición: Recuperar commits perdidos...` (también en el menú de commit del Log) encuentra los
commits que no alcanza ninguna rama, tag ni stash: los que un reflog todavía nombra (los
commits de una rama eliminada), los que solo conserva un pin de deshacer y los huérfanos. La
lista aparece al instante y el análisis más lento de los commits huérfanos se completa
mientras se muestra un indicador de progreso; nunca bloquea la interfaz.

![Trabajo perdido: los dos commits de una rama eliminada (encontrados a través del reflog) y commits huérfanos](../images/lost-work.png)

**Crear rama aquí**
(`b`) recupera un commit con el nombre que elijas, **Crear tag aquí** (`t`) lo fija con un tag
(nombre por defecto `restored-<id corto>`) y **Cherry-pick** (`c`) lo aplica a la rama actual.
Un commit perdido no está en el Log, así que esta vista no tiene «Mostrar en el Log» (la vista
Reflog lo conserva para los commits que aún son alcanzables).

**Restaurar cambios** devuelve lo que cambió el commit sin hacer commit de ello: **Aplicar al
árbol de trabajo** (`a`) reproduce los cambios del commit sobre los archivos, todo o nada (el
índice y HEAD se quedan como están; un commit de merge se rechaza). Cuando no se puede aplicar
limpiamente, no se toca nada y el diálogo ofrece **Guardar como stash** (`s`), que conserva esos
mismos cambios como un stash nuevo; esa acción también está siempre disponible por sí sola. Las
dos quedan en el diario: `Ctrl+Z` devuelve los archivos a su estado anterior, o vuelve a
eliminar el stash. Las mismas acciones son `gitpoulpi apply-commit <rev>` y `gitpoulpi stash
from-commit <rev> [-m M]`, y la vista Reflog también las tiene.

### Paneles, hunks y la ventana

Reflog y Trabajo perdido muestran la lista de commits a la izquierda y, a la derecha, la
cabecera del commit seleccionado, **todos los archivos que cambió** y los hunks del archivo
seleccionado (solo los hunks, ajustados al panel). La lista de archivos es la del panel derecho
de la ventana principal: `Ruta` o `Árbol` (`v`, o haz clic en el selector), una carpeta se
pliega con `Enter`, `Izquierda` o `Derecha`, un clic o las flechas eligen un archivo y sus hunks
aparecen debajo (se le pide al daemon fuera del hilo de la interfaz; se conservan los últimos
archivos). La lista de commits, la de archivos y los hunks tienen cada uno una barra de
desplazamiento que puedes pulsar y arrastrar.

- `Tab` / `Shift+Tab` recorren los paneles (lista, archivos, hunks); las flechas, `RePág`,
  `AvPág`, `Inicio` y `Fin` actúan sobre el panel que tiene el teclado (su borde está
  iluminado). `Enter` sobre un archivo pasa a sus hunks.
- **Divisores**: arrastra el que hay entre la lista y el panel derecho, o el que hay entre los
  archivos y los hunks; haz doble clic en un divisor para restablecerlo. Con el teclado:
  `Alt+Izquierda` / `Alt+Derecha`, `Alt+Arriba` / `Alt+Abajo`, `Alt+0` restablece ambos. La
  paleta tiene `Reflog / Trabajo perdido: Restablecer tamaños de los paneles`, `Panel siguiente`
  y `Lista de archivos como ruta o árbol`.
- `Ctrl+N` muestra sus marcas sobre las tres pestañas de vista y los tres paneles, como en la
  ventana principal; la tecla de una marca salta allí.
- **Mover y redimensionar el diálogo**: arrastra la barra de título para moverlo, arrastra los
  bordes izquierdo, derecho e inferior o una esquina inferior (el agarre `◢`) para cambiar su
  tamaño (nunca por debajo de 60 x 16, siempre dentro de la pantalla). La cruz de cierre
  conserva su clic y soltar fuera del diálogo nunca lo cierra. El equivalente con el teclado es
  `Ctrl+N` y luego `m`: las flechas lo mueven, `Shift`+flechas cambian su tamaño, `0` lo
  restablece, `Enter` o `Esc` salen (el borde lo indica mientras está activo). La paleta tiene
  `Diálogo: Mover o cambiar de tamaño` y `Diálogo: Restablecer posición y tamaño`. Si el sitio
  se conserva lo decide `ui.dialog_geometry` (Ajustes > Apariencia > Disposición): `session`
  (por defecto), `saved` (se restaura en el próximo arranque) o `never`.

## Desde la paleta de comandos

Cada acción de fila de las tres vistas es también una fila de la paleta de comandos, así
que el panel no necesita ratón ni una letra que recordar. Pulsa `Ctrl+P` con el panel
abierto: la paleta se abre encima y sus filas **Historial de deshacer:**, **Reflog:** y
**Trabajo perdido:** actúan sobre la fila seleccionada de la vista que está en pantalla.

| Vista | Filas de la paleta |
|---|---|
| Historial de deshacer | `Historial de deshacer: Deshacer hasta aquí...`, `Rehacer hasta aquí...`, `Restaurar solo esta...`, `Restaurar la rama a donde estaba...`, `Actualizar`, `Cerrar` |
| Reflog | `Reflog: Mostrar en el Log`, `Crear rama aquí...`, `Crear tag aquí...`, `Aplicar cambios al árbol de trabajo`, `Guardar cambios como stash`, `Restaurar a esta posición...`, `Cargar más`, `Actualizar` |
| Trabajo perdido | `Trabajo perdido: Crear rama aquí...`, `Crear tag aquí...`, `Cherry-pick`, `Aplicar cambios al árbol de trabajo`, `Guardar cambios como stash`, `Buscar de nuevo` |

Las filas aparecen atenuadas (con el motivo) mientras su vista no está abierta; los tres
abridores, `Edición: Historial de deshacer...`, `Edición: Reflog...` y `Edición:
Recuperar commits perdidos...`, también cambian un panel abierto a su vista. Una fila que
la selección no puede ejecutar dice por qué en la barra de estado en lugar de no hacer
nada: «esta acción ya está deshecha: usa Rehacer hasta aquí», el motivo por el que una
entrada no se puede restaurar por sí sola, «solo se puede devolver una rama o HEAD a una
posición anterior», etcétera. Las filas con `...` abren la misma confirmación o petición
de nombre que el botón.

## Qué se puede deshacer

| Área | Operaciones |
|---|---|
| Commits | commit, amend, reword, el compositor de commits con IA (todos sus commits como un solo paso) |
| Ramas y tags | crear, eliminar, renombrar, establecer o quitar el upstream, crear una rama de seguimiento, crear / eliminar / anotar un tag, fast-forward, hacer checkout de una pull request como rama |
| Moverse | checkout de una rama o de un commit (incluido un HEAD desacoplado) |
| Historial | merge (fast-forward o commit de merge), rebase (simple e interactivo, una vez terminado), cherry-pick, revert, reset (soft, mixed y hard), la parte local de un pull |
| Cambios de trabajo | stage y unstage (archivo, carpeta, hunk, líneas, todo), descartar (archivo, carpeta, hunk, líneas, un checkout de archivo), aplicar un parche o los cambios de un commit (`apply-commit`), **eliminar un archivo del disco**, **ignorar** (archivo, extensión o carpeta: la línea del `.gitignore` que elijas), **tratar como binario o texto** (la línea de `.gitattributes`) |
| Stashes | push, apply, pop, drop, guardar los cambios de un commit como stash (`stash from-commit`) |
| Remotos y configuración | añadir, quitar, renombrar, cambiar la URL o la URL de push de un remoto, track y untrack de Git LFS |
| En un servidor | **eliminar una rama o un tag en un remoto** (desde el menú de la ref, la paleta, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, o el cierre de un flujo que elimina la rama remota): véase abajo |

Mientras un merge, cherry-pick o revert está detenido por conflictos, **Guardar y marcar
resuelto**, el stage y el unstage también se pueden deshacer (cada uno se revierte antes que el
anterior), y **Abortar merge** (o cherry-pick, revert) se puede deshacer: al deshacerlo
vuelve toda la operación detenida, con sus conflictos (las tres versiones de cada archivo sin
resolver), los archivos ya resueltos y en stage, los marcadores de conflicto en el árbol de
trabajo y el mensaje del merge; rehacer vuelve a abortar. El stage y las resoluciones dejan
de poder deshacerse cuando la operación termina (con commit o abortada). *Unstage* y *Unstage
todo* nunca tocan un archivo que sigue en conflicto: no tiene ningún cambio en stage que
revertir.

Todo lo que descarta un discard o un hard reset se guarda antes, así que deshacerlo devuelve
los bytes exactos, incluidos los archivos sin seguimiento, los bits de ejecución y los enlaces
simbólicos. Deshacer un stash eliminado devuelve el mismo stash, con el mismo id de commit
(queda en lo alto de la lista de stashes, como `stash@{0}`, no en su posición anterior).
Deshacer la eliminación de un remoto devuelve sus ramas de seguimiento remoto, su alias
`origin/HEAD` y los ajustes de upstream de las ramas que lo seguían.

### Eliminar una rama o un tag en un remoto

Antes de la eliminación, GitPoulpi pregunta al servidor qué contiene ahora la rama o el
tag y lo registra: el remoto, el nombre y el commit exacto (en un tag anotado, el propio
objeto del tag, para conservar su anotación). La eliminación solo se hace si el servidor
sigue teniendo ese valor.

**Deshacer la vuelve a enviar** con el mismo nombre, en el mismo commit, sin forzar nunca.
Como es un cambio en un servidor que otras personas ven, Deshacer **siempre pregunta
primero** y lo dice («Deshacer esto vuelve a crear la rama topic en origin en
3f2a1c0...»); los pasos de la entrada también lo dicen (`gitpoulpi undo --dry-run`). Usa
las mismas credenciales que un push. La rama de seguimiento remoto vuelve con ella.
**Rehacer** la elimina otra vez, con la misma seguridad.

Deshacer se niega, sin cambiar nada, cuando:

- la rama o el tag se **creó de nuevo en el remoto** desde entonces, o se movió: está en
  otro commit distinto del registrado, y enviarlo sobrescribiría el trabajo de otra persona
  (la negativa nombra ambos commits);
- el commit (o el objeto del tag) **ya no está en tu repositorio** (lo eliminó la
  recolección de basura): el deshacer no se puede hacer.

«Eliminar en local y en el remoto» es **una sola entrada**: Deshacer restaura juntas la rama
o el tag local y el remoto, Rehacer vuelve a eliminar ambos. Eliminar solo la rama remota
deja intacta la local; deshacerlo hace lo mismo. El cierre de un flujo que también elimina
la rama remota se puede deshacer en conjunto.

## Qué no se puede deshacer, y por qué

- **La mayoría de las cosas en un servidor**: push, enviar un tag, push de LFS, bloquear y
  desbloquear, abrir una pull request. Ese estado se comparte con otras personas; deshacer
  un push sería un force-push o una eliminación en su lado, y GitPoulpi no lo hará por ti.
  (Eliminar una rama o un tag remotos es la excepción: el deshacer solo devuelve lo que el
  servidor tenía, y se niega si algo cambió allí.) Además, deshacer nunca se bloquea por una
  operación así.
- **Fetch**: solo mueve refs de seguimiento remoto, que reflejan el servidor. El siguiente
  fetch volvería a hacerlo.
- **Abortar un rebase**: no se ofrece deshacerlo (un rebase mueve ramas y guarda su propio
  estado; el propio abort es la vía de vuelta). Abortar un **merge, cherry-pick o revert** *sí*
  se registra, consulta más arriba.
- **Crear un worktree**, **crear un archivo de parche**, clone, init, acciones de alojamiento
  (pull requests, issues) y llamadas a la IA: no forman parte del historial del repositorio.
- **Crear un archivo vacío nuevo** desde el menú del archivo, la eliminación automática de
  archivos `*.orig` tras un merge y guardar un archivo en el editor integrado no se registran.
  Tampoco cambiar el **comando SSH** (`core.sshCommand`), ni cambiar el
  **checkout parcial** (no se pierde nada; *Desactivar* recupera todos los archivos).
- **Escribir en `.git/info/exclude` o `.git/info/attributes`** (la opción «solo este clon» de las listas de ignorar y de tratar como binario/texto): esos archivos están fuera del árbol de trabajo, así que no se registran.
- **Las ediciones de archivos del árbol de trabajo hechas fuera de GitPoulpi** (un editor, un
  script, un agente de IA que escribe código) nunca se capturan: no existe ninguna instantánea
  de ellas, así que un `git reset --hard` o un `git checkout -- archivo` hecho en un shell
  pierde para siempre las ediciones sin confirmar. Lo que *sí* se captura del trabajo de un
  shell es lo que vive en las refs: ver la sección siguiente.

## Cambios hechos fuera de GitPoulpi

Cuando trabajas con un agente de IA (o con un shell junto a GitPoulpi), la mayor parte del
trabajo con git ocurre fuera de la ventana: commits, ramas y worktrees nuevos, ramas
eliminadas, resets, rebases, stashes. GitPoulpi también ve todo esto. El daemon guarda las
últimas refs que vio (ramas locales, tags, HEAD, la lista de stashes, la lista de worktrees) y,
cuando el repositorio cambió y no fue obra de GitPoulpi, registra **una entrada por cada cambio
coherente** en el Historial de deshacer, marcada como **externa**, con el motivo que git
anotó (`commit: fix login`, `rebase (finish)`, `reset: moving to HEAD~1`, `checkout: moving
from a to b`) y el nombre que git registró para quien lo hizo. GitPoulpi no distingue un shell
de un agente de IA y nunca lo adivina; la entrada solo dice que se hizo fuera.

- **Una ráfaga es una entrada.** Veinte commits hechos entre dos comprobaciones son una sola
  entrada, del primer valor al último; un rebase es una entrada, registrada cuando termina (no
  se registra nada mientras un rebase o un merge está detenido).
- **Las refs eliminadas se fijan.** Una rama, un tag o un stash que desapareció se mantiene
  vivo bajo `refs/gitpoulpi/undo/` en su commit antiguo antes de que nada pueda podarlo, así
  que **Restaurar solo esta** (o `Ctrl+Z`) recupera la rama que el agente eliminó, incluso
  después del reflog y de una recolección de basura.
- **Mientras GitPoulpi estaba cerrado.** La instantánea se guarda junto al diario
  (`.git/gitpoulpi/refs-snapshot.json`), así que lo que ocurrió sin ningún GitPoulpi en
  marcha se encuentra en el siguiente arranque y se marca «mientras GitPoulpi estaba cerrado».
- **`Ctrl+Z` funciona con ellas como con cualquier entrada**: las refs vuelven atrás, una
  rama eliminada se recrea, un stash descartado se guarda de nuevo, un checkout externo vuelve
  a hacer checkout de la rama anterior. Una rama con checkout vuelve atrás como `git reset
  --keep`: los archivos la siguen, y cuando un cambio local tuyo estorba **se niega y no cambia
  nada**. Rehacer la aplica de nuevo. **Restaurar solo esta** también funciona: una rama
  eliminada vuelve y las acciones posteriores se quedan; una rama movida vuelve como
  `restore/<rama>-<sha>`.
- **La barra de estado lo dice una vez**: «cambio externo registrado - Ctrl+Z para
  deshacerlo» cuando llega una entrada externa nueva y la ventana está al frente.
- Como el cambio externo pasa a ser la entrada más reciente, la antigua negativa «bloqueado
  por un cambio externo» casi desaparece: primero deshaces el cambio externo.

Lo que no se registra: los movimientos de solo fetch de las ramas de seguimiento remoto (reflejan
el servidor), las refs ocultas de GitPoulpi, un worktree añadido o quitado sin ningún cambio de
rama (la entrada de la rama que contiene dice «árbol de trabajo añadido <ruta>»; deshacer solo
mueve refs, no quita el worktree) y cualquier cosa dentro de los archivos.
`general.journal_outside = false` (Ajustes > General, «Registrar los cambios hechos fuera de
GitPoulpi») lo desactiva: no se registra nada externo y un cambio hecho fuera bloquea la
entrada más reciente, como antes.

## Cuándo se niega a deshacer

Antes de actuar, GitPoulpi comprueba que todo lo que tocó la operación sigue exactamente como
la operación lo dejó. Si no, **no cambia nada** y dice qué se movió:

- `main` está en 3f2a1c0, no en a81b9e4 donde lo dejó Commit (un commit hecho en un shell,
  por ejemplo);
- se ha cambiado desde entonces un archivo, el índice, un stash o una sección de configuración;
- hay un merge, rebase o cherry-pick en curso;
- la operación nunca se completó porque GitPoulpi se detuvo mientras se ejecutaba (la entrada
  se marca como *interrumpida*, nunca se deshace automáticamente, y `gitpoulpi undo --list`
  muestra qué iba a cambiar para que puedas arreglarlo a mano).

Deshacer es estrictamente último en entrar, primero en salir: una entrada más reciente
bloqueada bloquea también las más antiguas, porque deshacer una operación antigua por debajo
de una más nueva que sigue en vigor sería adivinar. Si lo dejas todo como estaba por tu
cuenta, la entrada vuelve a ser válida. Los cambios que no tocan nada de lo que tocó una
entrada (un commit desde un shell en otra rama, un fetch) no bloquean nada.

### Una entrada bloqueada que no puedes despejar: Olvidar

Cuando la entrada más reciente está bloqueada para siempre (el repositorio cambió fuera de
GitPoulpi, o la entrada nunca terminó), tampoco se puede deshacer nada de lo que hay debajo.
El tooltip de Deshacer y la negativa lo dicen y nombran la salida:
**Edición: Olvidar la entrada de deshacer bloqueada** en la paleta (pregunta antes), o
`gitpoulpi undo-forget` en la línea de comandos. Olvidar solo quita esa entrada del diario; el
repositorio nunca se toca, y la entrada ya no se puede deshacer ni rehacer. La entrada de
debajo pasa entonces a ser la más reciente.

Dos casos se resuelven solos:

- un merge o rebase con conflictos que terminas con git a secas: la entrada se cierra la
  próxima vez que GitPoulpi lee el estado del repositorio;
- los archivos de un stash pop o apply con conflictos que resuelves con **Guardar y marcar
  resuelto**: Deshacer vuelve a poner el conflicto mientras el archivo resuelto sigue en
  stage y sin commit.

## Confirmaciones

Deshacer solo pregunta cuando es arriesgado:

- el commit (o merge, rebase, cherry-pick, revert) que vas a revertir ya está en el upstream
  de la rama: deshacerlo reescribe historial que publicaste, y tendrías que hacer force-push
  del resultado;
- tienes cambios sin commit y el deshacer mueve el árbol de trabajo: tus cambios se
  conservan, pero el deshacer se detiene, sin cambiar nada, si uno de ellos está en un
  archivo que tiene que reescribir;
- la entrada eliminó una rama o un tag en un remoto: deshacerla **la vuelve a enviar al
  servidor** (véase arriba), así que siempre pregunta, y lo mismo el rehacer que la
  elimina allí otra vez.

Rehacer, y todo deshacer de un simple movimiento de refs, nunca pregunta: la otra dirección
está a una tecla de distancia.

## Hasta dónde se puede retroceder

`general.undo_depth` (Ajustes > General) es el número de operaciones que se conservan. El valor
predeterminado es **100**; `0` significa sin límite. Bajarlo olvida de inmediato las entradas
más antiguas. Empezar una operación nueva tras un deshacer descarta lo que podías rehacer,
como en cualquier editor.

El historial se guarda por worktree (un worktree enlazado tiene el suyo), en el directorio de
git (`.git/gitpoulpi/journal.json`), así que sobrevive a cerrar el programa y nunca forma parte
de `git status` ni de un commit.

## Desactivarlo

`general.undo_enabled = false` (Ajustes > General, «Deshacer activado») desactiva toda la
función (y con ella el registro de los cambios externos). No se registra ni se conserva nada, así que el stage, el descarte y el commit son un
poco más rápidos. Los botones Deshacer y Rehacer desaparecen y `Ctrl+Z` dice «Deshacer está
desactivado».

**Desactivarlo borra el historial**: el archivo del diario y las refs ocultas descritas más
abajo se eliminan, la primera vez que cualquier petición llega al daemon mientras está
desactivado. Volver a activarlo empieza con un historial vacío.

## Qué se guarda

Para garantizar que un deshacer nunca pueda perder datos, cada commit, objeto de tag, stash o
instantánea que una entrada pueda necesitar se mantiene vivo mediante una ref oculta bajo
`refs/gitpoulpi/undo/` mientras la entrada esté en el historial. Se excluyen de los listados
propios de GitPoulpi, pero un `git log --all` o `git for-each-ref` desde un shell las muestra,
y `git push --mirror` las enviaría (usa `git push --all` en su lugar). Se eliminan cuando su
entrada sale del historial o cuando se desactiva el deshacer. Nunca se elimina nada más.

## Desde la línea de comandos y el MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` describe los pasos y no cambia nada.
- `--list` imprime el diario, con lo más reciente al final (`--all` añade las entradas que ya
  no se pueden rehacer pero aún se conservan), con el id y el estado de cada entrada.
- `--expect <id>` hace que la llamada se rechace por *obsoleta* salvo que esa entrada siga
  siendo la más reciente: un script (o una segunda ventana) nunca deshace algo distinto de lo
  que miró.
- `undo-forget` quita del diario la entrada bloqueada más reciente (`--redo`: la entrada de
  rehacer más reciente). Se niega cuando esa entrada no está bloqueada, y termina con **3** sin
  `--yes`. Para el MCP es una mutación, pero no destructiva: el repositorio no se toca.
- Código de salida **0** hecho, **2** rechazado (el JSON nombra el motivo, por ejemplo
  `RemoteRefMoved` cuando una rama remota eliminada se creó de nuevo), **3** arriesgado y
  necesita `--yes` (deshacer la eliminación de una rama o un tag remotos, que hace un push al
  servidor).

Volver atrás sin deshacer todo lo posterior:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (también: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (también: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` muestra, por cada entrada, qué podría recuperar **restaurar solo esta**
  (`restore.items`) o por qué no puede recuperar nada (`restore.not_restorable`), y quién la
  hizo: `"source": "gitpoulpi"`, o `"source": "outside"` con `outside.message` (el motivo de
  git) y `outside.committer` para un cambio hecho por un shell o un agente de IA. `undo`, `redo`
  y `undo-restore` tratan las entradas externas como cualquier otra.
- `undo-restore <id>` recupera lo que quitó esa entrada (una rama o un tag eliminados, un
  stash descartado, un archivo descartado o eliminado, un remoto quitado con sus ramas de
  seguimiento y upstreams, una línea de ignorados) y conserva todas las operaciones
  posteriores. Nunca sobrescribe: un archivo que cambió desde entonces recibe la versión
  guardada a su lado como `<name>.restored`; un nombre de rama en uso es un rechazo (salida
  2) que sugiere uno libre, que se indica con `--name`; tras un reset, amend o rebase crea
  una rama nueva en la punta antigua y nunca mueve tu rama. La restauración es una entrada
  del diario, así que `gitpoulpi undo` la revierte.
- `reflog` lista dónde estuvieron `HEAD` o una rama, con el motivo de git, incluido lo que
  hizo un shell. `reflog-restore` mueve una rama (o `HEAD`) a una de esas posiciones; una
  rama con checkout y cambios sin commit necesita `--mode`. También se registra en el
  diario.
- `lost-commits` lista los commits que no alcanza ninguna rama, tag, remoto ni stash. Para
  recuperar el trabajo de uno sin hacer commit de él: `apply-commit <rev>` (al árbol de
  trabajo, todo o nada) o `stash from-commit <rev>` (como stash); ambos quedan en el diario.
- Cuando un movimiento hecho en un shell bloquea la entrada más reciente, el rechazo lleva la
  posición del reflog que contiene lo que la entrada espera (`reason.RefMoved.restore`), para
  pasarla a `reflog-restore`.

Existen dos comandos para que las acciones de arriba se puedan deshacer desde todas las
superficies: `gitpoulpi rm-file <paths>` (eliminar archivos del árbol de trabajo) y
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (añadir una línea a un `.gitignore`;
no ocurre nada si la línea ya está; deshacer devuelve el archivo a como estaba, o lo elimina
si lo creó el comando, y se niega si el archivo se editó desde entonces).

El servidor MCP tiene una herramienta por comando, así que `undo`, `redo`, `rm-file` e
`ignore` también son herramientas. `undo`, `redo` y `rm-file` se clasifican como destructivas
(con el mismo control que `reset`, consulta [MCP y agentes](mcp-and-agents.md)); `ignore` es una
mutación normal. El diario es un archivo del repositorio, así que `gitpoulpi undo` en un shell
revierte lo que hizo la ventana, y la ventana puede deshacer lo que hizo un script.
