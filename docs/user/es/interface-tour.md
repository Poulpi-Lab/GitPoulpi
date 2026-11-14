[English](../interface-tour.md) · [Čeština](../cs/interface-tour.md) · [Deutsch](../de/interface-tour.md) · **Español** · [Français](../fr/interface-tour.md) · [Italiano](../it/interface-tour.md) · [日本語](../ja/interface-tour.md) · [한국어](../ko/interface-tour.md) · [Nederlands](../nl/interface-tour.md) · [Polski](../pl/interface-tour.md) · [Português (Brasil)](../pt-BR/interface-tour.md) · [Português (Portugal)](../pt-PT/interface-tour.md) · [Русский](../ru/interface-tour.md) · [Svenska](../sv/interface-tour.md) · [Türkçe](../tr/interface-tour.md) · [Українська](../uk/interface-tour.md) · [简体中文](../zh-CN/interface-tour.md)

# Recorrido por la interfaz

Esta página nombra cada zona de la ventana de GitPoulpi, de arriba abajo. Las
demás páginas usan los mismos nombres. Las imágenes usan un pequeño repositorio
de demostración con autores inventados.

![La ventana principal con sus zonas: barra de pestañas, barra de herramientas, barra lateral Refs, Log, panel de cambios de trabajo, formulario de commit y barra de estado](../images/hero.png)

## El marco

### Barra de título

El borde superior de la ventana dice
`🐙 GitPoulpi vX.Y.Z -- <ruta del repositorio>` (el nombre de la sesión está en la barra de estado, abajo por defecto).

- **🐙** abre la ventana **Acerca de**: versión, funciones de la compilación,
  licencia, enlaces, un breve aviso de que GitPoulpi es un proyecto independiente,
  los crates de terceros y las licencias y créditos. En un terminal que dibuja
  imágenes, el pulpo es el logo de píxeles, en el color de tu logo (en los
  demás se queda el emoji). Paleta: `Ayuda: Acerca de GitPoulpi`; teclas:
  `Ctrl+N` y luego `A`. En Acerca de, las pupilas del pulpo siguen al puntero del
  ratón. Haz clic en el logo (o pulsa `Space`) para hacerlo girar hasta el
  siguiente color. Haz clic derecho en el logo (o pulsa `c`) para ver la
  lista de colores. GitPoulpi guarda tu elección en `ui.logo_color`. Los enlaces
  se abren con un clic o con su tecla de acceso (`s` código fuente, `d`
  documentación, `i` licencia), y `l` lista los crates de terceros con sus
  versiones y licencias (una lista desplazable; `Esc` o `l` vuelve, y también un
  clic en su título). Cada crate de la lista lleva un enlace a su página (su
  repositorio o, si no tiene, su página en crates.io): haz clic en la fila, o
  elígela con `Up` / `Down` y pulsa `Enter`, para abrirla en el navegador. `t` (o el comando de la paleta `Ayuda: Licencias y créditos`) muestra los avisos de licencia y los créditos de todo lo que lleva el programa, en un texto desplazable (`Up` / `Down`, `PgUp` / `PgDn`, `Home` / `End`; `Esc` o `t` para volver). Desde un script, `gitpoulpi licenses` imprime el mismo texto.
- La **versión** (`v1.0.0`; `vX.Y.Z disponible` si hay una versión más reciente)
  está entre el nombre y la ruta: un clic abre el panel de actualización cuando
  existe una versión más reciente, y la pestaña Notas de la versión en caso contrario.
  Consulta [Actualizaciones](updates.md).
- **Abrir en ▾**, justo después de la ruta, abre la carpeta de la pestaña en un
  terminal, un editor, un multiplexor o el gestor de archivos (`Alt+O`). Consulta
  [Abrir en](open-in.md).
- **⌕ Búsqueda global** (una lupa y su etiqueta), justo a la izquierda del engranaje, abre la [búsqueda global](search.md) (`Ctrl+Alt+F`). Es lo primero que cede cuando la barra de título es demasiado estrecha: primero se va la etiqueta (la lupa se queda) y después el botón.
- El **engranaje** (`⚙`), justo a la izquierda de la **×**, abre los **Ajustes** (`Ctrl+,`).
- **×** en el extremo derecho sale, como `q`.

![La ventana Acerca de sobre la ventana principal: el pulpo pixelado, la versión, la licencia, las funciones de compilación, los enlaces y el número de crates](../images/about.png)

![La lista de crates de terceros de la ventana Acerca de: en cada fila el nombre, la versión, un enlace a la página del crate y la licencia](../images/about-crates.png)

### Barra de estado

La barra de estado es por defecto la última fila de la ventana (**Ajustes > Apariencia > Posición de la barra de estado** la lleva arriba, sobre las pestañas de repositorio). Muestra la rama actual, los commits
cargados (`500 / 48213 commits` mientras falta cargar historial más antiguo, con
`...` en lugar del total mientras se cuenta; solo `2500 commits` cuando todo el
historial está cargado), `sucio` o `limpio`, el nombre de la sesión, una etiqueta
`parcial` en un checkout parcial, y las teclas más útiles: `?` ayuda, `Ctrl+G`
acciones de git, `Ctrl+N` navegación, `Ctrl+P` comandos, `Shift+F10` menú, `q` salir.
En una ventana estrecha se omiten las teclas menos útiles. La barra de estado también
muestra mensajes de estado y progreso. Las teclas que muestra son siempre las que
están en vigor, incluidas las que hayas cambiado (consulta
[Ajustes](settings.md#teclado-y-ratón)). Cómo se cuenta el total es un ajuste
(**Contar el total de commits con**, véase [Ajustes](settings.md#historial-y-grafo)).

En su extremo derecho, el botón **Actividad** abre el Registro de actividad: una
pestaña Operaciones de git (las operaciones con lo que tardaron, mensajes y
errores), una pestaña Aplicación (el registro de la aplicación, con un cuadro de nivel y
un botón **Vaciar registro**) y un cuadro Alcance que limita ambas a este repositorio, a un espacio de
trabajo o a todos los repositorios. Un número en el botón cuenta los errores que aún no has
mirado. A su izquierda, el indicador de trabajos muestra un trabajo en segundo plano
con una `[x]` para cancelarlo. La pestaña **Inicio** tiene la misma barra de estado,
con el botón Actividad abajo a la derecha (se abre en Todos los repositorios). Cuando
una operación falla, la barra de estado lo dice en rojo; su entrada del Registro de
actividad explica por qué, ofrece un paso siguiente, los detalles y un Copiar para una IA. Ambos se
describen en [Cuando algo falla](troubleshooting.md#cuando-algo-falla).

### Superposiciones

La paleta de comandos, la ayuda, los menús contextuales, los diálogos, las
ventanas emergentes y los Ajustes se abren sobre la ventana. Cada uno lleva una `✕`
en la esquina superior derecha de su borde; `Esc` o un clic en la `✕` lo cierra, y un
clic fuera cierra un diálogo, una ventana emergente o un panel (véase
[Cerrar ventanas y diálogos](mouse-and-gestures.md#cerrar-ventanas-y-diálogos)).

## Fila de pestañas

La pestaña **Inicio** (el icono de la casa) está en el extremo izquierdo, seguida
del [chip del espacio de trabajo](workspaces.md) (cuando tienes un espacio de trabajo),
de la pestaña fija **Radar** (se cierra como cualquier pestaña; consulta
[el Radar](radar.md)) y de la pestaña **Espacios de trabajo** cuando la has abierto
(`Alt+J`; se cierra como cualquier pestaña). Luego viene una pestaña por cada repositorio
abierto. La `x` que cierra una pestaña solo se muestra en la pestaña activa y en la
que está bajo el puntero. La fila de pestañas solo lleva pestañas (el botón de la
[búsqueda global](search.md) está en la barra de título).

Cuando las pestañas de repositorio no caben, la fila se desplaza en lugar de encogerlas:
las pestañas quedan entre dos botones de flecha, `‹` a la izquierda y `›` a la derecha.
Una flecha se atenúa cuando no hay ninguna pestaña oculta de ese lado. Haz clic en una flecha
para mostrar una pestaña más de ese lado (mantenla pulsada para seguir desplazando), o usa la
rueda sobre la fila (arriba o a la izquierda muestra las pestañas anteriores). Cada vez que
cambia la pestaña actual (cambias de pestaña con el teclado, con el selector de pestañas o con
la paleta, abres o cierras una pestaña, o cambias el tamaño de la ventana), la fila se desplaza
lo justo para mostrarla entera. Si hay sitio, un botón `…` tras la flecha derecha abre el
selector de pestañas. Arrastrar una pestaña sobre una flecha desplaza la fila mientras la
mantienes ahí, así que puedes soltarla más allá de las pestañas visibles. `Ctrl+N` pone una
insignia en cada flecha activa.

**Ajustes > Apariencia > Posición de la barra de pestañas** mueve esta fila al final de la ventana (bajo la barra de estado) o la convierte en una columna en el borde izquierdo: una pestaña por fila, primero Inicio, luego el chip del espacio de trabajo, el Radar, la pestaña Espacios de trabajo, las pestañas (un nombre largo acaba en `…`) y el `+`. Cuando hay más pestañas que filas, una fila `…` indica cuántas quedan ocultas arriba (`▲`) y abajo (`▼`) y abre la lista de pestañas; la rueda desplaza la columna. Junto a una barra de herramientas en columna, las pestañas quedan en el exterior. Una ventana de menos de 80 columnas mantiene las pestañas arriba. Los clics, la cruz de cierre, el clic central, el menú del clic derecho, arrastrar para reordenar y el teclado funcionan igual en cada posición.

- El **+** justo después de la última pestaña abre un pequeño menú: **Nueva
  pestaña** (`Alt+T`, paleta **Pestaña: Nueva pestaña**) y, mientras sus pestañas
  están cerradas, **Radar** y **Espacios de trabajo** (`Alt+J`), que abren esas
  pestañas; **Pestaña: Nueva...** de la paleta abre el mismo menú. Una **Nueva
  pestaña** es la vista Inicio con el campo de búsqueda listo, de modo que al
  escribir se filtran tus repositorios al instante. Elige uno (o Examinar, Clonar, Init) y su pestaña aparece al final de
  la barra; un repositorio que ya está abierto simplemente se activa. `Esc` en la
  búsqueda vacía, `Ctrl+W`, su `x` o un clic central cierran la Nueva pestaña y
  vuelven a la pestaña en la que estabas. Nunca se guarda en tu sesión.

  ![Una Nueva pestaña: la vista Inicio con el campo de búsqueda listo, sobre las pestañas abiertas](../images/new-tab.png)
- Haz clic derecho en una pestaña, o pulsa `Ctrl+T`, para ver el menú de la
  pestaña: **Nueva pestaña**, **Cerrar pestaña**, **Cerrar las demás pestañas...**, **Cerrar las
  pestañas de la derecha...**, **Añadir a favoritos** (o **Quitar de favoritos**),
  **Añadir al espacio de trabajo** (o **Quitar del espacio de trabajo**, cuando tienes
  espacios de trabajo), **Copiar ruta**, **Abrir en ›**, **SSH y credenciales...** y
  **Checkout parcial...**.
- `Ctrl+PageUp` y `Ctrl+PageDown` cambian de pestaña. `Ctrl+W` cierra la pestaña
  actual. Un clic central en una pestaña también la cierra (con la misma
  pregunta).
- `t` (o `Ctrl+N t`) abre el selector de pestañas, una lista con búsqueda de las
  pestañas abiertas.
- **Reordenar pestañas:** pulsa una pestaña de repositorio y arrástrala hacia un lado
  (un fantasma de la pestaña sigue al puntero y una barra muestra dónde caerá; `Esc`
  cancela), o arrastra una fila del selector de pestañas, pulsa `Alt+Up` / `Alt+Down`
  sobre la fila resaltada del selector, o ejecuta **Pestaña: Mover a la izquierda** /
  **Pestaña: Mover a la derecha** desde la paleta. El orden se guarda con la sesión.
  Un clic simple sin movimiento sigue seleccionando la pestaña. La casa, el chip del
  espacio de trabajo y el `+` nunca se mueven.
- **Salir y cd aquí** aparece a la derecha de las pestañas cuando iniciaste
  GitPoulpi mediante el wrapper de shell `gp` y la pestaña no es la carpeta en la
  que empezaste. Consulta [Seguir la carpeta de la pestaña](following-the-tab-folder.md).
- El **chip del espacio de trabajo** justo después de la casa (un punto de color
  y un nombre, o "Sin espacio de trabajo") cambia de [espacio de trabajo](workspaces.md):
  haz clic en él o pulsa `Alt+W`. Aparece cuando tienes un espacio de trabajo.

![El selector de pestañas con la lista de los repositorios abiertos](../images/tab-picker.png)

Con **Recordar pestañas** activado (el valor por defecto), tus pestañas vuelven en
el siguiente arranque. Un worktree se abre en su propia pestaña, colocada justo después
de la de su repositorio (**Agrupar las pestañas de worktrees** en Ajustes > General;
activado por defecto). Esa pestaña se cierra cuando se elimina el worktree.

### Inicio

La pestaña Inicio es el gestor de repositorios. Tiene cuatro secciones:
**REPOSITORIOS ABIERTOS**, **FAVORITOS**, **REPOSITORIOS RECIENTES** y **TODOS LOS
REPOSITORIOS**. Cada fila muestra la rama, los recuentos de adelantados y
atrasados y el número de cambios. Cada fila tiene una estrella (favorito),
**Abrir** o **Ir a**, **Cerrar** o **Quitar**, y un menú `⋮`.

Arriba: **Examinar**, **Clonar**, **Init**, un cuadro de búsqueda, **Contraer
todo** y **Expandir todo**. Una carpeta que ya no existe sale sola de *Recientes*
y de *Todos*. Un favorito que falta se queda, con un botón **Quitar**. **Quitar**
nunca borra nada del disco.

Teclas: `Enter` abrir, `f` favorito, `x` cerrar o quitar, `.` menú, `/` buscar,
`Space` plegar una sección, `Esc` volver a tu pestaña.
`Ctrl+N` muestra una insignia en el campo de búsqueda, cada sección y cada botón: pulsa el carácter de una insignia para enfocarla
o pulsarla (consulta [Insignias](keybindings.md#insignias)).

Un único cuadro de búsqueda filtra todo lo que hay debajo, incluidos los favoritos: un repositorio se queda si su nombre, su rama o su ruta contiene el texto. Los espacios de trabajo tienen su propia pestaña y no son una página de Inicio: ábrela desde el menú del `+`, con `Alt+J` o desde la paleta, y ciérrala con su `x`: consulta [Espacios de trabajo](workspaces.md).

**Clonar** lista, por cada perfil conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit), los repositorios a
los que tienes acceso, una sección por propietario: primero los tuyos, luego una
por organización o grupo (en orden alfabético), cada una bajo un encabezado. Un
candado marca un repositorio privado. El cuadro de búsqueda filtra en todas las
secciones; una sección sin coincidencias desaparece. `Up` y `Down` saltan los
encabezados, y `Home`, `End`, `PgUp` y `PgDn` funcionan en la lista.

![La pestaña Inicio: repositorios abiertos, favoritos y recientes, y los botones Examinar, Clonar e Init](../images/home.png)

## Barra de herramientas

Por defecto, una fila bajo los paneles y sobre la barra de estado. **Ajustes > Apariencia > Posición de la barra de herramientas** la coloca arriba (bajo las pestañas), abajo, o en una columna a la izquierda o a la derecha de los paneles (una ventana de menos de 80 columnas dibuja la columna como una fila abajo). Los botones se colocan al inicio de la fila o de la columna; **Ajustes > Apariencia > Alineación de los botones de la barra de herramientas** los centra o los lleva al final. Cada botón tiene su propio color de icono, que fija el tema (consulta [Temas](themes.md)). Cuando no hay sitio para todos, los botones menos importantes pasan detrás de un `⋯`. En una columna cada botón ocupa una fila, con su `▾` al final de la fila, y su menú se abre junto a la columna; el menú de una barra de herramientas inferior se abre hacia arriba.

| Botón | Qué hace |
|---|---|
| **Deshacer**, **Rehacer** | Deshacen o restauran la última operación (`Ctrl+Z`, `Ctrl+Y`). Ocultos cuando Deshacer está desactivado. Consulta [Deshacer y rehacer](undo-redo.md). |
| **Pull** `▾` | Hace pull como indica **Ajustes > Git y codificación > Botón Pull**. La lista `▾` tiene **Fetch All**, **Pull (fast-forward si es posible)**, **Pull (solo fast-forward)** y **Pull (rebase)**. Un elemento de la lista se ejecuta una vez y no cambia el botón. |
| **Push** `▾` | Hace push de la rama con checkout. La lista `▾` hace push una vez al remoto que elijas. |
| **Rama** | Crea una rama en el commit con checkout. |
| **Tag** | Crea un tag en el commit con checkout. |
| **Stash**, **Pop** | Hace stash de todos los cambios. Hace pop del último stash. |
| **LFS** `▾` | Acciones de Git LFS: fetch, checkout, pull, push, prune (`Ctrl+G l`). Oculto cuando Git LFS está desactivado. Consulta [Git LFS](lfs.md). |
| **Flow** `▾` | El menú del flujo de trabajo de ramas (empezar, terminar, sincronizar, promover; `Ctrl+G Shift+W`). Solo en un repositorio que tiene un flujo de trabajo. Consulta [Flujos de trabajo de ramas](branching-workflows.md). |
| **Buscar** | Abre la barra de búsqueda de commits sobre el Log (`f`). |

En el extremo (la derecha de una fila, la parte inferior de una columna), cuando el repositorio está vinculado a un host, un **chip** muestra la pull request de la rama con checkout (por ejemplo `GH #12` o `GL !53`), o un tenue `+ PR`. **Ajustes > Apariencia > Fijar el botón de pull request al final** (activado por defecto) lo mantiene allí; desactivado, fluye con los demás botones y sigue su alineación. Consulta [Alojamiento y gestores de issues](hosting-integration.md).

Junto a él (en una columna, en su última fila), cuando tienes dos o más [perfiles](settings.md#perfiles),
el **combo de perfiles** elige la identidad de commit para este repositorio. La
primera fila, *Predeterminado*, sigue **Ajustes > Perfiles**. Después de elegir un
perfil, GitPoulpi pregunta si debe escribir también su nombre y correo en la
config de git del repositorio (`user.name`, `user.email`). **Escribir en la config
de git** los escribe. `Esc` conserva la elección solo en GitPoulpi. Paleta:
`Repositorio: Cambiar perfil...`.

Cada botón tiene un comando de la paleta y un atajo.

![La lista de Pull abierta bajo la barra de herramientas: Fetch All y las tres formas de hacer pull](../images/pull-push-tooltip.png)

## Avisos

Una franja sobre el Log te indica cuándo el repositorio está en un estado
especial. Tiene botones para seguir adelante. Cada botón es también un comando de
la paleta.

| Aviso | Botones |
|---|---|
| Un merge, cherry-pick o revert se detuvo por conflictos | **Resolver conflictos...**, **Commit** o **Continuar**, **Abortar** |
| Un rebase en curso | **Resolver conflictos...**, **Continuar**, **Omitir**, **Abortar** |
| **Aplicando parches (git am)** | **Resolver conflictos**, **Continuar**, **Omitir**, **Abortar** |
| Un `index.lock` residual bloquea todos los cambios | **Eliminar index.lock** (pregunta antes) |
| Un auto-stash no se restauró | **Pop now**, **Show stash**, **Empezar de nuevo**, **Drop** (las opciones dependen del estado) |

Consulta [Resolver conflictos de merge](merge-conflicts.md) y
[Solución de problemas](troubleshooting.md).

## Barra lateral Refs (izquierda)

Un árbol de todo lo que nombra un commit. Cada sección tiene un recuento en su
título, un chevron para plegarla y su propia posición de desplazamiento. Arrastra
el borde entre dos secciones para cambiar su tamaño.

- Cuadro de **Filtro** arriba (`/`): filtra por nombre todas las secciones.
- **LOCALES**: tus ramas, en carpetas divididas por `/` (`feature/`, `fix/`). Una
  marca indica la rama con checkout. Puedes desactivar las carpetas en los Ajustes.
- **REMOTAS**: cada remoto, con sus ramas debajo.
- **WORKTREES**: los worktrees vinculados, cuando los hay (o cuando mostraste la
  sección). El **+** de su encabezado crea uno; haz doble clic en un worktree
  para abrirlo en su propia pestaña, o clic derecho para Abrir en, Merge y
  Eliminar. Consulta [Worktrees](worktrees.md).
- **STASHES**: tus stashes. Haz clic derecho en uno para ver **Pop**, **Aplicar** y
  **Drop**.
- **TAGS**: tus tags, en carpetas separadas por `/` como las ramas (`releases/`, `backups/`). Una flecha indica que el tag también está en el remoto.
- **PULL REQUESTS** e **ISSUES**: después de conectar una cuenta de alojamiento y
  vincular el repositorio.
- **SUBMÓDULOS**: los submódulos y su estado, cuando los hay. Consulta
  [Submódulos](submodules.md).

**Mostrar u ocultar una sección:** clic derecho en el encabezado de una sección, o en la parte vacía del panel, para una lista de comprobación de las secciones (la última visible se queda). La elección se conserva por repositorio, entre reinicios; véase [Secciones del panel izquierdo](mouse-and-gestures.md#secciones-del-panel-izquierdo).

Pasa el puntero sobre una fila para mostrar sus botones: ocultar o mostrar la ref
en el Log, y **Solo** (mostrar solo los commits de esa ref). Haz doble clic en una
rama local o en un tag para hacer checkout. Haz clic derecho en una fila para ver
su menú. El `‹` del título de la barra lateral la contrae (`Ctrl+N h`).

Los dos paneles laterales (la barra lateral Refs, `Ctrl+N h`, y el panel derecho,
`Ctrl+N l`) pertenecen al Log: mostrar o contraer uno mientras hay un diff, una vista de
archivo o un editor abierto solo afecta a esa vista. Un diff o una vista de archivo también
contrae por sí solo la barra lateral Refs para dejar sitio al contenido. De vuelta en el
Log, ambos paneles quedan como los dejaste allí, y ese es el estado que se guarda para el
próximo inicio.

## El Log (centro)

![El Log: una fila de cabecera con las columnas, la fila de cambios de trabajo arriba, una fila de stash y los chips de ref a la izquierda del grafo](../images/log-graph.png)

Una fila por commit, el más reciente primero. Haz clic derecho en la cabecera para
mostrar u ocultar columnas. Arrastra el borde entre dos cabeceras para cambiar el
tamaño de una columna. Arrastra una cabecera para mover la columna.

El panel no tiene título: los encabezados de columna dicen lo que es. En un commit sin ref, la fila bajo el puntero muestra una píldora tenue con el nombre de la rama a la que pertenece (se desactiva con **Mostrar una rama fantasma en el commit bajo el puntero** en Ajustes > Log y grafo).

| Columna | Muestra |
|---|---|
| **Ref** | Chips de colores con las ramas y tags del commit: un icono de portátil para local, un globo para remota, un icono de tag para tags, `+N` cuando apuntan allí más refs. Los tags se dibujan donde están, nunca como fantasma. En un commit sin ref, la fila bajo el puntero muestra una píldora tenue con el nombre de la rama a la que pertenece; si esa rama se eliminó, una píldora tachada muestra el nombre leído del commit de merge que la fusionó (se desactiva con **Mostrar una rama fantasma en el commit bajo el puntero** en Ajustes > Log y grafo). |
| **Grafo** | Los carriles de las ramas, los merges y los puntos de ramificación. Se dibuja como imágenes en kitty, Ghostty, WezTerm, Rio e iTerm2 (y también Windows Terminal, foot y Tabby, mediante sixel), y como caracteres en los demás terminales. Los colores de los carriles vienen del tema o de una paleta que elijas (consulta [Temas](themes.md)). En su ancho mínimo (un punto de commit) la columna es compacta: todos los commits en un solo carril, sin líneas de rama ni de merge. |
| **Mensaje** | El resumen del commit. |
| **Autor** | El autor, con un avatar opcional. |
| **Fecha** | La fecha del autor, en el formato que elijas. |
| **Cambios** | Opcional: archivos cambiados y líneas `+añadidas -eliminadas`. |
| **SHA** | El hash corto. |

![El menú de columnas de la cabecera del Log con las casillas Ref, Grafo, Mensaje, Autor, Fecha, Cambios y SHA](../images/log-columns-menu.png)

Filas especiales:

- **La fila de cambios de trabajo** (fila WIP) arriba: un círculo discontinuo,
  `// WIP` o tu mensaje en borrador, y los recuentos de archivos modificados y
  añadidos. Haz clic en ella para hacer stage y commit en el panel derecho.
- **Filas de stash**: un cuadradito en un carril, junto al commit sobre el que se
  hizo el stash.
- Durante una búsqueda, las filas que no coinciden se atenúan.

Pasa el puntero sobre una celda para ver un tooltip con el texto completo. Haz
clic derecho en una fila para ver el [menú del commit](log-and-history.md#el-menú-del-commit).

![El menú de clic derecho de un commit: cherry-pick, revert, reset, editar mensaje, explicar, rebase, rama, tag, checkout, worktree, parche y acciones de copia](../images/context-menu.png)

## El panel derecho

El panel derecho sigue a la selección del Log.

**Cambios de trabajo** (la fila WIP está seleccionada):

- Un título como `4 archivos con cambios en main`. Durante un merge con conflictos añade
  el número de archivos en conflicto. Un `✗` rojo a la izquierda descarta todos
  los cambios (pregunta antes). Cuando la IA está activada, un `✦` a la derecha
  explica los cambios.
- **Ruta** o **Árbol** (lista plana o carpetas, `m`), **Contraer todo** y **Ver
  todos los archivos** (`a`).
- **Archivos unstaged** y **Archivos staged**, cada uno con un recuento y
  un botón **Stage todo** o **Unstage todo**. Pasa el puntero
  sobre una fila para ver su botón **Stage** o **Unstage**.
- El **formulario de commit** abajo: **Modificar el commit anterior**, **Componer
  commits con IA** (cuando la IA está activada), el **Resumen** con un contador de
  caracteres y el botón `✦`, la **Descripción** y el botón **Commit de cambios**.

**Detalles del commit** (hay un commit seleccionado), de arriba abajo: el hash
completo, el autor con avatar y fecha, el hash del padre y los recuentos de cambios,
y luego el mensaje (resumen y descripción; arrastra su borde inferior para cambiarle
el tamaño). El panel **Archivos cambiados** de debajo empieza con sus propios controles:
el interruptor de orden, **Ruta** / **Árbol**, **Ver todos los archivos** y un cuadro
**Filtrar archivos**, fijos sobre la lista, que es la única parte que se desplaza. Haz
clic en un archivo para abrir su diff. En una ventana baja cede primero el mensaje
(hasta una fila) y luego la lista. La vista de cambios de trabajo mantiene su fila
**Ruta** / **Árbol** / **Ver todos los archivos** arriba de las listas de archivos,
sobre las listas Unstaged y Staged.

**Enlaces en un mensaje.** Una dirección web escrita en el mensaje (`https://...`, o el destino de un
`[texto](https://...)` de markdown) aparece subrayada, muestra el puntero de mano y, al hacer clic,
pregunta **¿Abrir este enlace?** con la dirección completa antes de abrir nada: el texto de un mensaje
no es de fiar. **Abrir** (`A`) inicia el navegador; **Cancelar** (`C`, `Esc`, la cruz) no hace nada.
Solo se ofrecen direcciones `http` y `https`. El equivalente de teclado es el comando de la paleta
`Detalles del commit: Abrir enlace del mensaje...`, que pregunta por la primera dirección del mensaje
y recorre las demás con **Siguiente** (`S`). Algunos terminales dibujan su propio subrayado sobre una
dirección bajo el puntero y la abren ellos mismos con un clic con modificador (a menudo Shift+clic);
esa función es del terminal, no de GitPoulpi.

Cada lista de archivos tiene menús en archivos, carpetas y la parte vacía, y se pueden seleccionar varios archivos a la vez (**Ctrl+clic**, **Shift+clic**); consulta [Stage y commit](staging-and-committing.md).

## El panel central cuando hay un archivo abierto

Haz clic en un archivo cambiado y su vista sustituye al Log. `Esc` devuelve el
Log. Una barra de botones recorre la parte superior:

![Un archivo modificado abierto como diff, con la barra de vista encima: Ver archivo, Blame, el selector Archivo │ Todos, Editar, navegación por hunks, En línea, Dividida, Hunks, espacios en blanco, Ajuste y Mapa](../images/diff.png)

| Botón | Qué hace |
|---|---|
| **Ver archivo**, **Diff**, **Blame** | Eligen qué muestra el panel para el archivo. |
| **Archivo │ Todos** | Elige la lista de la [columna de commits](#la-columna-de-commits) junto a la vista y muestra la columna. |
| **Editar** | Abre el editor integrado sobre la copia de trabajo. |
| `↑` `↓` | Hunk anterior o siguiente. |
| **En línea**, **Dividida** | Una columna, o lado a lado. |
| **Hunks** | Solo los hunks cambiados, o el archivo entero. |
| **EB** | Mostrar u ocultar los cambios solo de espacios en blanco. |
| **Ajuste** | Ajustar las líneas largas. |
| **Mapa** | Un minimapa de todo el archivo junto a la barra de desplazamiento. |
| **Codificación** | La codificación con la que se lee el archivo (**Auto** primero). |

En un diff del árbol de trabajo, cada hunk tiene los botones **Stage hunk** y
**Descartar hunk**. Pulsa `v` para seleccionar líneas y luego haz stage o
descarta solo esas líneas.

![Blame de un archivo: el código con colores de sintaxis y, a la derecha, una columna de anotaciones por hunk con avatar, autor, antigüedad y resumen, más las franjas de autor y de antigüedad](../images/blame.png)

![Historial del archivo: los commits del archivo a la izquierda, en dos líneas, y el diff del commit seleccionado a la derecha](../images/file-history.png)

Ambas vistas se describen en [Blame](log-and-history.md#blame) e [Historial del archivo](log-and-history.md#historial-del-archivo).

El **editor** integrado tiene colores de sintaxis, números de línea, deshacer,
buscar y una barra de estado con la línea, la columna, el fin de línea, la
sangría y el lenguaje.

![El editor integrado mostrando un archivo Rust con resaltado de sintaxis y números de línea](../images/editor.png)

Un archivo de imagen se muestra como imagen: antes y después en el diff, o el
propio archivo en la vista de archivo. Consulta [Imágenes](image-view.md).

Un archivo binario que no es una imagen (un PDF, un documento, un archivo comprimido) muestra un botón **Abrir en** cuando el sistema conoce aplicaciones para su tipo: consulta [Abrir en](open-in.md).

## La columna de commits

Una columna puede aparecer a la izquierda del diff, la vista de archivo o el Blame de un archivo de un commit (nunca junto al editor integrado, que edita la copia de trabajo: la columna desaparece al abrir el editor y vuelve al cerrarlo sobre el diff). Tiene tres estados:

- **Archivo**: los commits de ese único archivo, de dos líneas cada uno (el [Historial del archivo](log-and-history.md#historial-del-archivo)). El lado derecho muestra la versión del archivo en el commit seleccionado.
- **Todos**: una lista compacta de los commits del Log con su grafo, descrita más abajo.
- **Oculta**: una franja fina con un `›` a la izquierda de la vista recupera la columna (su tooltip nombra la tecla).

La barra de botones de la vista ocupa siempre todo el ancho sobre la columna, así que ningún botón se mueve cuando la columna se abre, se cierra o cambia de lista. En ella, el selector **`Archivo │ Todos`** (dos segmentos de un mismo control, siempre del mismo ancho) elige la lista: un clic en un segmento muestra esa lista y abre la columna si estaba oculta; un clic en el segmento ya elegido, con la columna visible, la oculta. El segmento de la lista en pantalla está iluminado; mientras la columna está oculta, el de la lista con la que volverá está subrayado y la lista se conserva. Su tooltip nombra el clic, el segundo clic en el segmento elegido y `Alt+C`. El Historial del archivo también se abre desde los sitios de siempre (menús de archivo, `Historial del archivo: ...` en la paleta).

![La columna de commits en su estado Todos junto al diff de un archivo de un commit, con una cabecera de Ref, Grafo, Mensaje, Autor y Fecha y etiquetas de ramas y tags en la parte Ref](../images/commit-column-all.png)

- **Mostrar u ocultar**: `Alt+C` (paleta `Ver: Alternar la columna de commits`), un clic en el segmento elegido del selector, la ✕ de la columna o la franja `›`. En la paleta, `Ver: Columna de commits: ocultar`, `... : historial del archivo` y `... : todos los commits` eligen un estado. La elección, con la lista que tenía, se recuerda con los demás ajustes de la columna. Sin elección, el ancho del terminal decide para la lista Todos: aparece desde 140 columnas. La columna no se dibuja cuando la vista de al lado quedaría con menos de 30 columnas, ni el grafo de Todos para un diff del árbol de trabajo o una comparación (el historial del archivo sí: no necesita ningún commit).
- **Ancho**: arrastra su borde derecho (el borde iluminado sigue al puntero), o usa `Ver: Ensanchar la columna de commits` y `Ver: Estrechar la columna de commits`. Puede ocupar hasta el 70 % de la columna central (el resto sigue siendo de la vista, que nunca queda con menos de 30 columnas). Se recuerda con los demás tamaños.
- **Elegir un commit**: haz clic en él, o da el teclado a la columna (`Ctrl+N` y su insignia, o `Ver: Enfocar la columna de commits`) y usa `Arriba` / `Abajo`, `RePág` / `AvPág`, `Inicio` / `Fin`. `Intro` o `Derecha` pasa a la lista de archivos del commit, `Esc` devuelve el teclado a la vista. El panel derecho sigue la elección: detalles del commit y sus archivos modificados, para abrir otro de sus archivos.
- **Qué hace la vista**: el archivo abierto sigue abierto. Si el commit elegido lo cambió, la vista muestra su diff en ese commit. Si no, muestra el archivo tal como era entonces, con "sin cambios en este commit" en el título. Si el commit no tiene ese archivo, la vista lo dice y sigue siendo utilizable. Un Blame sigue siendo Blame, a fecha del commit elegido.

La lista Todos va bajo una cabecera (los nombres de columna del propio Log). Un commit por línea: los carriles del grafo, el título (el mensaje), el nombre del autor y una fecha corta y, si lo activas, las etiquetas de ramas y tags. Es la propia lista del Log, con los mismos commits en el mismo orden, y carga los más antiguos al desplazarse. Los carriles se dibujan como en la columna Grafo del Log: en un terminal que muestra imágenes (consulta [Elegir un terminal](terminals.md)), como carriles conectados con marcadores redondos y, con **Avatares de autor** en **grafo**, las fotos de los autores dentro de los círculos; en los demás, como caracteres. Por defecto los carriles ocupan el espacio que necesitan y el título conserva siempre unas palabras: en una columna estrecha ceden primero el autor y luego la fecha antes de que se recorten los carriles. Reducida a su ancho mínimo, un punto de commit, la parte Grafo es compacta, como en el Log: todos los commits en un solo carril, sin líneas de rama ni de merge. Una parte recortada por el panel (la fila desplazada hacia los lados, o una parte Grafo más ancha que el espacio) conserva sus imágenes y avatares, cortados en el borde. La rueda desplaza la lista bajo el puntero.

- **Tooltip y menú**: al dejar el puntero sobre una fila aparece el tooltip del Log para la parte que hay debajo (el mensaje completo sobre el título, el nombre y el correo del autor, la fecha completa). Un clic derecho en una fila (o la tecla de menú, `Shift+F10`, cuando la columna tiene el teclado; `Ver: Columna de commits: menú de la fila seleccionada` en la paleta) abre su menú: *Mostrar en el Log*, el diff del archivo en ese commit, el archivo tal como era en ese commit, el blame del archivo en ese commit, y luego el propio menú del Log para ese commit (copiar, cherry-pick, rama y, bajo una línea, las filas que reescriben cosas, como revert). Cada una de las cuatro primeras filas tiene también una entrada en la paleta (`Ver: Columna de commits: mostrar ...`). En la lista Archivo, las filas tienen el propio menú del Historial del archivo y los mismos tooltips.
- **Dimensionar las partes y desplazar hacia los lados**: arrastra un límite de la cabecera para dimensionar la parte que queda a su izquierda: los carriles, el mensaje, el autor, la fecha, las refs, los cambios o el SHA. La última parte también tiene un límite: la celda justo después de ella (el propio borde de la columna cuando la fila llena el espacio), iluminada del mismo modo, que dimensiona esa última parte. Como en el Log (`ui.column_resize`), el límite es por defecto la celda separadora entre dos partes, donde termina el tinte de fila del Grafo (el tinte la atraviesa, así que ninguna celda lisa queda entre ella y la parte siguiente): una marca descansa en ella en la cabecera y se dibuja una línea fina a lo largo de la columna mientras el puntero está sobre ella o la arrastra, y un arrastre desde cualquier punto de la línea redimensiona; con `header` es la primera letra del nombre siguiente, que se ilumina mientras el puntero está sobre ella (no se dibuja ninguna línea, la cabecera es el único sitio desde el que arrastrar). Mientras no dimensiones nada, el mensaje ocupa lo que dejan los demás; el primer arrastre fija cada parte al ancho que tiene, el mensaje incluido, así que dimensionar una parte mueve las partes siguientes y no cambia ningún otro ancho. Una parte que dimensionaste se conserva y nunca cede espacio: cuando la fila es entonces más ancha que la columna, se desplaza hacia los lados con Shift+rueda, una rueda horizontal o `Shift+Izquierda` / `Shift+Derecha` cuando la columna tiene el teclado, y una barra de desplazamiento en su borde inferior muestra dónde estás. Desde el teclado: `Tab` elige la parte (entre las visibles), `]` la ensancha, `[` la estrecha, `0` devuelve todas las partes al ancho automático (la paleta tiene `Ver: Columna de commits: dimensionar la parte siguiente`, `... ensanchar la parte`, `... estrechar la parte`, `... restablecer los anchos de las partes`, `... desplazar a la izquierda` y `... desplazar a la derecha`). Los anchos se recuerdan. Mientras un desplazamiento recorta los carriles, se dibujan como caracteres incluso en un terminal que muestra imágenes.
- **Elegir las partes**: un clic derecho en la cabecera abre un menú con una casilla por parte, todas las columnas que ofrece el Log: Ref, Grafo, Mensaje, Autor, Fecha, Cambios y SHA. Mensaje siempre se muestra. **Ref**, **Cambios** y **SHA** están desactivadas por defecto; si se muestran, Ref lleva las etiquetas de ramas y tags del commit, Cambios sus archivos y líneas `+añadidas -eliminadas` y SHA su hash corto, todo como las columnas del propio Log. Como Ref, Cambios y SHA nunca ceden espacio: la fila se desplaza hacia los lados. Desde el teclado, `Ver: Columna de commits: elegir las columnas` abre el mismo menú (las flechas e `Intro` eligen una fila) y `Ver: Columna de commits: alternar Refs`, `... alternar Grafo`, `... alternar Autor`, `... alternar Fecha`, `... alternar Cambios` y `... alternar SHA` activan o desactivan una parte al instante. Las partes visibles, su orden y sus anchos pertenecen solo a la columna: se guardan en `config.toml` (`ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths`), aparte de las columnas del propio Log, que nunca cambian con ellas, ni al revés.
- **Mover las partes**: arrastra un nombre de la cabecera sobre otro para mover esa parte allí, como en el Log (la fila sigue al puntero; con `ui.column_resize = "header"` la primera letra de un nombre es el límite de tamaño, así que pulsa en el resto del nombre). Cualquier parte puede ir a cualquier sitio, incluidos los carriles y las etiquetas Ref. Desde el teclado, `Tab` elige la parte y `<` / `>` la mueven un lugar a la izquierda o a la derecha entre las visibles (la paleta tiene `Ver: Columna de commits: mover la parte a la izquierda` y `... mover la parte a la derecha`). El orden se recuerda.

  ![El menú de clic derecho de la cabecera de la columna de commits: una casilla para Ref, Grafo, Mensaje (siempre visible), Autor, Fecha, Cambios y SHA](../images/commit-column-menu.png)

Los gestos del ratón también se enumeran en [Ratón y gestos](mouse-and-gestures.md#la-columna-de-commits-lista-todos).

## Los tamaños se recuerdan

GitPoulpi conserva cada tamaño que arrastras, entre cambios de pestaña y
reinicios. Los tamaños son los mismos para todos los repositorios. Esto abarca los
divisores entre la barra lateral Refs, el Log y el panel derecho, la altura de cada
sección de Refs, el bloque del mensaje del commit, los anchos de las columnas del
Log, los paneles del editor de merge y el divisor del selector de carpetas. Los
tamaños se guardan como fracciones de la ventana, de modo que un diseño hecho en
un terminal ancho también cabe en uno estrecho.

Para restablecer todos los tamaños, ejecuta el comando de la paleta
`Navegación: Restablecer el diseño`. Algunos tamaños también tienen su propio
restablecimiento: `Navegación: Restablecer los tamaños de sección de Refs`,
`Detalles del commit: Restablecer el tamaño del mensaje` y, en el editor de merge,
un doble clic en un divisor o `Alt+0`.

## Superposiciones y pantallas especiales

- **Paleta de comandos** (`Ctrl+P`): un cuadro de filtro sobre todos los comandos.
  Cada fila muestra su atajo. Un comando que no se puede ejecutar ahora se atenúa,
  y el motivo se muestra abajo. Los comandos tienen un prefijo de grupo como
  `Git:`, `Refs:`, `Navegación:`, `Diff:`, `Cambios de trabajo:`, `Commit:`,
  `Vista:`, `Repositorio:`, `Pestaña:`, `LFS:`, `Pull Requests:`, `Issues:` o
  `IA:`. Escribe un prefijo para ver su grupo.
- **Selectores**: algunos comandos de la paleta abren una segunda lista, por
  ejemplo `App: Cambiar tema...`, `App: Cambiar idioma...` y
  `App: Cambiar tema de color del grafo...`. La ventana muestra cada opción en
  directo mientras te mueves por la lista. `Enter` la conserva. `Esc` restaura la
  opción anterior. La lista de temas tiene los filtros **Todos**, **Oscuros**,
  **Claros** y **Con color**. Consulta [Temas](themes.md).
- **Ayuda** (`?`): pestañas. **Teclas** lista cada tecla y gesto; las demás
  pestañas (Inicio, Log, Cambios, Refs, Alojamiento, Ajustes, IA y CLI) son guías
  breves de las acciones, con cómo llegar a cada una. Cambia de pestaña con `Tab` /
  `Shift+Tab`, `Left` / `Right`, `Alt+1`..`Alt+8` o un clic. Escribe para filtrar la
  pestaña actual. `Ctrl+D`, o un clic en el enlace de abajo, abre la página de
  documentación de la pestaña en la que estás.

  ![La pestaña Alojamiento de la ayuda: una guía breve con cómo llegar a cada acción](../images/help-hosting.png)

  ![La superposición de ayuda: la sección Global con Ctrl+P, Ctrl+N, Ctrl+G, ? y las teclas de movimiento, y luego la sección Ratón: lo básico](../images/help.png)
- **Barra de búsqueda** (`f`): arriba a la derecha del Log. Muestra `n de N`
  coincidencias.
- **Menús contextuales**: clic derecho, o pulsa `.` sobre la fila seleccionada.
  Una fila con `›` abre un submenú.
- **Diálogos**: confirmaciones, el nombre de una rama o de un tag, Clonar, Init,
  Crear Pull Request, las opciones tras un push rechazado y otros. `Enter` acepta,
  `Esc` cancela.
- **Rebase interactivo** y el **compositor de commits** sustituyen al Log por una
  pantalla de revisión. Consulta
  [Trabajar con el Log](log-and-history.md#rebase-interactivo) y
  [Stage y commit](staging-and-committing.md#el-compositor-de-commits-divide-tus-cambios-en-commits).

  ![El compositor de commits revisando cuatro commits propuestos, cada uno con sus archivos, y los botones Restablecer, Cancelar y Crear 4 commits](../images/composer.png)

- **Ajustes** (`Ctrl+,` o el engranaje de la barra de título): una lista de páginas a la izquierda y las
  opciones de la página a la derecha. Consulta [Ajustes](settings.md).

  ![Los Ajustes: la lista de páginas a la izquierda, con un cuadro de búsqueda encima, y la página General a la derecha](../images/settings.png)

- **Ventanas emergentes** para respuestas largas, como la ventana **Explicar
  cambios** de la IA (`↑` `↓` desplazan, `c` copia, `Esc` cierra).

  ![La ventana emergente Explicar los cambios de trabajo con una explicación de la IA y una lista de cosas que conviene revisar](../images/ai-explain.png)

Todas las listas de las ventanas emergentes funcionan igual. Consulta
[Atajos de teclado](keybindings.md) y [Ratón y gestos](mouse-and-gestures.md).

## Palabras usadas en esta documentación

| Palabra | Significado |
|---|---|
| Panel | Una de las tres columnas: la barra lateral Refs, el Log (o el archivo abierto) y el panel derecho. |
| Chip de ref | Una marca de rama o de tag en la columna Ref. |
| Fila WIP | La fila de cambios de trabajo en la parte superior del Log. |
| Hunk | Un bloque de líneas cambiadas en un diff. |
| Compositor | El formulario de commit, y la revisión de IA que divide los cambios en commits. |
| Acorde (chord) | Una tecla seguida de otra, como `Ctrl+G` y luego `s`. |
| Prefijo | `Ctrl+G` (acciones de git) o `Ctrl+N` (navegación). |
| Insignia | La letra que `Ctrl+N` muestra junto a un control. Púlsala para enfocar ese control. |
