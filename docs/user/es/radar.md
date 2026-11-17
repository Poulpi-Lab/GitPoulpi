[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · **Español** · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# El Radar

El Radar es una pestaña propia, justo después de Inicio: una única lista de lo que te espera en **varios repositorios**:
las pull (o merge) requests que escribiste, las que piden tu revisión, los issues
asignados a ti o creados por ti, y los repositorios donde dejaste trabajo en
curso. Es la ventana de Inicio con la vista Radar encima, para que nada de debajo
pueda robar un clic ni una tecla.

Necesita un perfil de host de código (consulta
[Alojamiento e issue trackers](hosting-integration.md); las listas de un repositorio están en [Pull requests e issues](pull-requests-and-issues.md), y la discusión de una fila se abre como [conversación](conversations.md)) y repositorios vinculados
a él, o un perfil con sus listas de toda la cuenta activadas (consulta "Más allá del ámbito" más abajo).
Un repositorio que no está vinculado nunca se consulta: muestra una línea
**Vincular...** (consulta "Repositorios con algo que decir" más abajo).

![El Radar sobre dos repositorios: una solicitud fijada, luego Pendientes de mi revisión, Mis pull requests, Asignados a mí, issues y trabajo en curso](../images/radar.png)

**Revisar...** en una fila de pull o merge request (o `e`) abre el repositorio con la revisión de la solicitud dentro de la
aplicación ([Revisar una pull request](review.md)).

## Abrirlo

- **`Alt+R`** (reasignable: `global.radar`), desde cualquier sitio.
- La paleta: `Radar: Abrir`.
- La pestaña **Radar**, justo después de Inicio en la barra de pestañas.
- La fila **Radar** del menú **`+`**, mientras la pestaña está cerrada.

Desde la pestaña de un repositorio te lleva a la ventana Inicio con el Radar
abierto; `Esc` o **Volver** regresa a la lista de repositorios, y `⌂` hace lo
mismo.

## Cerrar la pestaña o desactivar el Radar

La pestaña se cierra como la de un repositorio: con su **x**, un clic central, `Ctrl+W` mientras se muestra o la
entrada `Radar: Cerrar pestaña` de la paleta. Inicio es la única pestaña que no se puede cerrar. Una pestaña
cerrada sigue cerrada en esa sesión, también tras reiniciar, hasta que vuelvas a abrir el Radar (`Alt+R`,
`Radar: Abrir` o el menú `+`); entonces reaparece justo después de Inicio. Cerrarla no cambia ningún ajuste.

Si nunca lo usas, desactívalo en **Ajustes > General > Radar activado** (`general.radar_enabled`, consulta
la [referencia de configuración](config-reference.md)). Desactivado, la pestaña, sus comandos de la paleta y
`Alt+R` desaparecen y GitPoulpi no consulta nada para él. Se aplica al instante, sin reiniciar.

## Qué cubre: el ámbito

El combobox **Ámbito** elige qué repositorios se consultan:

- un **espacio de trabajo**, por nombre (sus repositorios, incluso los que no
  tienen pestaña abierta);
- **Pestañas abiertas**: los repositorios abiertos como pestañas de esta sesión;
- **Carpeta de clonado por defecto** y **Carpeta de exploración por defecto**: todos los repositorios que se
  encuentran en la carpeta definida en los [Ajustes](config-reference.md) (`repos.clone_root`,
  `repos.browse_root`), tengan o no una pestaña.

Se abre en el último ámbito que elegiste aquí (guardado en `radar.toml`), mientras siga existiendo; antes de
tu primera elección, o cuando ese espacio de trabajo se eliminó, en el espacio de trabajo activo de la sesión y, si no, en las
pestañas abiertas. La elección vale solo para esta vista: no cambia el espacio de trabajo activo de la ventana. Cuando dos
repositorios tienen el mismo nombre de carpeta, se añade la carpeta superior al
chip para distinguirlos.

### Las dos carpetas de ámbito

Un ámbito de carpeta busca repositorios **a dos niveles de profundidad**: las carpetas que están directamente
en ella y las carpetas dentro de las que no son repositorios (`~/src/work/app`). Se omiten las carpetas ocultas,
nunca se entra en un repositorio y se listan como mucho 200. La búsqueda se hace en segundo plano, nunca en el
hilo que dibuja, y su resultado se conserva hasta que actualizas: el botón Actualizar, `r`, `F5` (la tecla de actualizar de la ventana, reasignable) o
`Radar: Actualizar` vuelven a buscar en la carpeta (un repositorio clonado desde entonces aparece).
El host sigue siendo opcional por repositorio: a un repositorio de la carpeta que **no está vinculado** a un host
nunca se le pregunta. Solo aporta su fila de trabajo en curso, tiene una línea `sin vincular` con **Vincular...**
en Repositorios (abre la sección de pull requests de ese repositorio, donde eliges el proveedor y el proyecto),
y la barra de estado los cuenta (`3 sin vincular (solo trabajo en curso)`).
Una carpeta sin definir aparece atenuada en la lista (`Carpeta de exploración por defecto (se configura en Ajustes)`);
al elegirla se abre su fila de Ajustes (Ajustes > General) y el ámbito queda como estaba. La carpeta de clonado
cuenta como definida por su valor por defecto de cada sistema operativo (la carpeta con la que empieza el diálogo
Clonar) mientras esa carpeta exista; la carpeta de exploración no tiene valor por defecto. La lista de ámbitos se
lee de los ajustes cada vez que se abre, así que un cambio hecho allí se aplica al instante.

## La lista

| Grupo | Qué contiene |
|---|---|
| Fijados | Todo lo que fijaste, sea del tipo que sea, en el orden en que lo fijaste |
| Pendientes de mi revisión | Solicitudes que piden tu revisión (y que no escribiste tú) |
| Mis solicitudes | Las solicitudes que escribiste; el chip dice `borrador` para un borrador |
| Asignados a mí | Solicitudes asignadas a ti |
| Me mencionan | Solicitudes e issues abiertos que te mencionan y no están ya en un grupo anterior (un issue que no tienes asignado ni creaste tú aparece aquí solo cuando te menciona) |
| Otras solicitudes | Las demás solicitudes abiertas |
| Issues asignados a mí | Issues abiertos asignados a ti |
| Issues creados por mí | Issues abiertos que creaste |
| Trabajo en curso | Repositorios del ámbito con cambios sin commit o commits sin enviar |
| Pospuestos | Elementos que ocultaste por un tiempo; plegado hasta que lo abras |
| Repositorios | Repositorios con algo que decir (sin conexión, token por renovar, sin vincular) |

Las palabras "pull request" y "merge request" siguen el ajuste de terminología
habitual y el host.

Cada fila muestra el tipo de elemento (`⇄` solicitud, `●` issue, `✎` trabajo en
curso), el chip del repositorio (en el color del espacio de trabajo), el número,
el título, el autor, la antigüedad y, donde hay sitio, las etiquetas. Una fila de
trabajo en curso muestra la rama y lo que espera en ella: `✎` archivos
modificados, `+` archivos nuevos, `↑` commits por enviar, `↓` commits por traer.

Los recuentos están en cada encabezado. Dentro de un grupo, el elemento
actualizado más recientemente va primero.

La barra de estado bajo el filtro muestra un indicador giratorio cuando los
hosts tardan más de un momento en responder. Las listas en caché (la misma caché
que usa la barra lateral) se muestran al instante y se sustituyen según llegan
las respuestas.

### Columnas

El chip del repositorio, el `#` (número y marcas de checks), el autor y la antigüedad son **columnas ajustables**. En automático, la columna del repositorio es tan ancha
como el nombre más largo en pantalla (hasta 40 celdas, y como máximo un tercio de la ventana), así que `owner/name` y el
`group/sub/project` de una lista de cuenta no se cortan en un terminal normal. El título ocupa lo que queda. Un nombre
que no cabe termina en puntos suspensivos (`group/sub...`), y:

- **Deja el puntero quieto** sobre un repositorio, título, autor o lista de etiquetas cortados y un tooltip muestra el
  texto completo. Selecciona la fila y la línea de estado bajo el filtro también nombra el proyecto y el título
  completos, así que el teclado ve lo mismo.
- **Arrastra el separador** (la celda justo después del nombre del repositorio, después del `#`, o justo antes del autor o de la edad) para ajustar la
  columna; haz **doble clic** en él para volver a ajustarla automáticamente. La lista tiene una **fila de cabecera** (repositorio, `#`,
  título, autor, edad) y los separadores siguen `ui.column_resize` como los del Log: con `line` (el
  valor por defecto) un `│` atenuado descansa en el separador en la fila de cabecera, y mientras el puntero está sobre el separador
  (en la cabecera o en cualquier fila de la lista), la columna está elegida con las teclas o arrastras, se dibuja una línea fina de acento
  a lo largo de toda la lista; un arrastre desde cualquier punto de ella redimensiona. Al soltar, la línea desaparece salvo que el
  puntero siga sobre el separador (entonces se va con el puntero): un arrastre con el ratón nunca deja una columna elegida. Con `header` no hay marca ni línea: la primera
  celda tras el separador, en la fila de cabecera, se ilumina y es el único sitio desde el que arrastrar. El puntero se convierte
  en un puntero de redimensionar sobre un separador en los terminales que lo admiten. El chip de estado tiene un
  ancho fijo, y también la edad (la edad más ancha mostrada, hasta que la dimensiones; una edad cortada por una columna estrecha muestra su
  fecha completa en un tooltip), así que el autor empieza en la misma celda en todas las filas.
- O usa las teclas: **`C`** elige una columna (otra vez, o `Tab`, para la siguiente), **`]`** la ensancha dos celdas,
  **`[`** la estrecha, **`0`** devuelve todas las columnas a automático, `Esc` sale. La paleta tiene lo mismo:
  `Radar: Elegir columna`, `Radar: Ensanchar columna`, `Radar: Estrechar columna`, `Radar: Restablecer anchos de columna`.

Los anchos se guardan en `radar.toml` (`[columns]`, `repo`, `number`, `author` y `age`, en celdas) y sobreviven a un reinicio. Son los
mismos gestos de ajuste que las columnas del Log.

### Checks y conflictos

Una fila de solicitud lleva dos marcas pequeñas tras su número cuando el host ha dicho algo:

| Marca | Significado |
|---|---|
| `✓` | los checks del commit head pasaron |
| `✗` | falló un check |
| `●` | hay checks en cola o en ejecución |
| `!` (junto a ella) | la solicitud tiene conflictos con su rama de destino |

Deja el puntero sobre las marcas y un tooltip lo dice con palabras («Comprobaciones fallidas», «En conflicto con main»);
selecciona la fila y la barra de estado bajo el filtro dice lo mismo, de modo que el teclado también lo ve. Una solicitud
sin marca es una de la que el host no dijo nada (sin CI configurado, o no se preguntó).

- **GitLab** envía el estado del pipeline y la marca de conflicto con la lista de merge requests: cada fila
  tiene sus marcas.
- **GitHub** no: la lista no trae ninguna de las dos. GitPoulpi las pide por solicitud (la propia solicitud para
  su estado de merge, el estado combinado del commit head y sus check runs: tres llamadas), como máximo para
  **15 solicitudes abiertas por repositorio**, las tuyas primero. Si GitHub rechaza una de esas llamadas (un límite de tasa),
  el resto se queda sin marcar hasta la siguiente actualización. Las respuestas se guardan con la caché de alojamiento.

- **Gitea y Forgejo** se leen como GitHub: la lista trae la marca de conflicto (`mergeable`), el estado de los commits
  se pide por solicitud (como máximo 15 por repositorio, las tuyas primero) y los issues son los mismos grupos.
  Las menciones usan la búsqueda del servidor (`mentioned_by`).

### Menciones

**Me mencionan** y el filtro **Mostrar: Me mencionan** usan lo que el host sabe de las menciones. A GitHub se le
pregunta mediante su búsqueda (`mentions:`), a GitLab mediante tu lista de tareas pendientes: solo cuentan las menciones aún **pendientes**
allí, así que una tarea que marcaste como hecha sale del grupo.

Las solicitudes cerradas y fusionadas y los issues cerrados no se listan.

## Vistas y filtros

Tres comboboxes y un campo de texto acotan la lista. Cada elección es un
desplegable: haz clic en él, o usa su tecla.

- **Vista** (`v`): Todo, solicitudes, Issues, WIP (trabajo en curso), Pospuestos.
- **Mostrar** (`f`): Todos, Míos (solicitudes que escribiste, issues que
  creaste), Asignados a mí, Revisión solicitada, Me mencionan.
- **Ámbito** (`w`): ver arriba.
- **Texto de filtro** (`/` o `Ctrl+F`): coincidencia aproximada con el
  repositorio, el número, el título, el autor, la rama y las etiquetas. `Enter`
  devuelve las teclas a la lista y conserva el texto, `Esc` lo borra.

`Ctrl+N` muestra una insignia en el filtro, cada combobox, los botones
Actualizar y Volver y cada encabezado de sección en pantalla. Pulsa el carácter
de una insignia: el filtro recibe el teclado, un combobox se abre, un botón se
ejecuta y un encabezado recibe la selección (`Enter` o `Space` lo pliega
después). `Esc` cancela. Consulta [Insignias](keybindings.md#insignias).

## Acciones

Selecciona una fila y usa su menú (clic derecho, el `⋮` al final de la fila, `.`
o `Menu`), una tecla, o la paleta (`Radar: ...`). Una acción del ratón
siempre tiene una tecla y un equivalente en la paleta.

| Acción | Tecla | Paleta |
|---|---|---|
| Abrir repositorio (su pestaña se abre, o se cambia a ella) | `Enter` / doble clic | `Radar: Abrir repositorio` |
| Revisar... (una solicitud; una solicitud no clonada pregunta cómo) | `e` | `Radar: Revisar solicitud...` |
| Revisar una solicitud mediante la API del host (sin clon) | `a` en la pregunta | `Radar: Revisar mediante la API` |
| Clonar el repositorio y luego revisar la solicitud | `c` en la pregunta | `Radar: Clonar y revisar...` |
| Abrir en el navegador | `o` | `Radar: Abrir en el navegador` |
| Cambiar a rama (una solicitud) | `c` | `Radar: Cambiar a rama` |
| Crear worktree... (una solicitud) | `t` | `Radar: Crear worktree...` |
| Mostrar en repositorio (su lista) | `s` | `Radar: Mostrar en repositorio` |
| Empezar a trabajar en issue... (un issue) | `n` | `Radar: Empezar a trabajar en issue...` |
| Fijar arriba, o Dejar de fijar | `p` / clic en la `★` | `Radar: Fijar o dejar de fijar` |
| Editar asignados... / revisores... / etiquetas..., Establecer hito... (GitHub, GitLab) | menú de la fila | `Radar: Editar asignados...`, `Radar: Editar revisores...`, `Radar: Editar etiquetas...`, `Radar: Establecer hito...` |
| Posponer... | `z` | `Radar: Posponer...` |
| Dejar de posponer | `u` | |
| Mostrar u ocultar las solicitudes de repositorios no clonados | `a` | `Radar: Mostrar solicitudes no clonadas` |
| Mostrar u ocultar los borradores de las filas de toda la cuenta | `d` | `Radar: Mostrar borradores` |
| Activar las solicitudes de revisión de toda la cuenta de un host | `T` / `Enter` en su línea de aviso / **Activar** | `Radar: Activar las solicitudes de revisión de toda la cuenta...` |
| Actualizar | `r`, `F5` | `Radar: Actualizar` |
| El menú de la fila | `.` | `Radar: Menú de la fila...` |
| Volver al inicio | `Esc` | `Radar: Volver al inicio` |

Las acciones que necesitan la ventana propia del repositorio (cambiar de rama,
worktree, mostrar, empezar a trabajar, vincular, los selectores de metadatos) cambian primero a su pestaña y
se ejecutan allí cuando ha cargado:

- **Cambiar a rama** usa el checkout propio de la barra lateral: la rama
  cuando se conoce localmente, y si no, la cabeza de la solicitud traída a una
  rama local (la de un fork, por ejemplo).
- **Crear worktree...** abre el diálogo Crear worktree sobre la rama de la
  solicitud.
- **Mostrar en repositorio** selecciona el elemento en la lista PULL REQUESTS
  o ISSUES.
- **Empezar a trabajar en issue...** abre el prompt Nueva rama con un nombre
  hecho a partir del issue (`12-fix-the-thing`).

### Fijar y posponer

Una **fijación** mantiene un elemento arriba. Una **posposición** lo oculta
durante **1 hora**, **4 horas**, **Hasta mañana** (las 08:00 de la mañana siguiente, en tu hora local, aunque ahora todavía no sean las 08:00) o **Hasta que
cambie** (el host informa de una actualización del elemento, o, en una fila de
trabajo en curso, sus contadores se mueven). Ambos se guardan en `radar.toml`
en tu carpeta de configuración, con clave host, repositorio, tipo y número, y los
comparten todas las sesiones de la máquina.

### Repositorios con algo que decir

Un repositorio con un problema recibe su propia línea al final de la lista, y
sus elementos en caché siguen en pantalla:

- **sin conexión, hace 5m**: no se pudo contactar con el host;
- **Reconectar**: el token falta o fue rechazado; el botón abre Ajustes > Integraciones;
- **en pausa en esta ronda**: el host pidió ir más despacio (o rechazó el token),
  así que GitPoulpi no volvió a consultarlo para los demás repositorios de esta
  ronda;
- **sin vincular, se sugiere GitHub**: el remoto del repositorio coincide con un
  perfil, pero el alojamiento es opcional por repositorio, así que no se consulta
  nada hasta que lo vincules. **Vincular...** te lleva a la sección PULL REQUESTS
  del repositorio, donde eliges el proveedor y el proyecto.

Se consultan como máximo cuatro repositorios a la vez, y nunca en el hilo que
dibuja la pantalla.

## Más allá del ámbito: filas de toda la cuenta

El ámbito abarca los repositorios que clonaste. Una solicitud que espera tu revisión en un repositorio que
nunca clonaste es invisible ahí. Las **filas de toda la cuenta** consultan al propio host, una vez por perfil, y
se unen a la lista sea cual sea el ámbito: tratan de ti, no de una carpeta.

Están **desactivadas por defecto** (la búsqueda de un host cuesta cuota de API). Mientras un host de código conectado no
tiene ninguna de sus listas activada, la parte superior de la lista muestra una línea para él:

`GitHub: listar las solicitudes que esperan tu revisión, también en repositorios que no has clonado  [Activar]`

Haz clic en **Activar**, pulsa `Enter` en esa línea, pulsa `T`, o ejecuta `Radar: Activar las solicitudes de revisión
de toda la cuenta...`: escribe `integrations.radar.review_requests = true` para ese perfil en `config.toml` y
consulta al host de inmediato. Las demás listas (todas las solicitudes abiertas de tus espacios de nombres, issues asignados a ti,
borradores, espacios de nombres) se fijan con `gitpoulpi hosting integrations radar` o en los ajustes del perfil; ver
la [referencia de configuración](config-reference.md) y la [referencia de la CLI](cli-reference.md). Responden GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center y Gerrit; Bitbucket Cloud no tiene un endpoint de toda la cuenta,
así que sus solicitudes solo aparecen para los repositorios vinculados del ámbito.

Los **Espacios de nombres** del perfil limitan todas sus listas a ciertos repositorios (Ajustes > Integraciones > Radar: un área de texto con una entrada por línea y una casilla **Expresiones regulares**). Una línea coincide con la ruta completa de un repositorio cuando es igual a ella o es un prefijo de ruta que termina en `/`: `ptm` conserva `ptm/produits/x` y descarta `ptmx/y` y `formation/y`. Con la casilla activada, cada línea es una expresión regular que se aplica a la ruta completa tal como la escribes (`^ptm/(produits|infra)/`); una línea no válida se muestra bajo el campo y no se guarda. Vacío significa sin límite.

Cómo se colocan las filas:

- Una fila que el ámbito ya lista (mismo host, proyecto, tipo y número) aparece una sola vez: **gana la fila
  del ámbito**.
- Una fila cuyo proyecto has clonado en algún sitio (repositorios recientes, espacios de trabajo, las carpetas que
  encontraron los ámbitos de carpeta; se compara por la URL del remoto) lleva ese clon: **Enter**, cambiar a rama, worktree y
  las demás acciones lo abren como cualquier fila.
- Cualquier otra fila está **sin clonar**: la etiqueta del repositorio se atenúa y muestra el proyecto (`acme/app`), y
  la línea de estado dice `No clonados` cuando está seleccionada. **Abrir en el navegador** funciona, **Revisar** pregunta cómo
  revisarla (más abajo), y cambiar a rama, worktree, mostrar en repositorio y empezar a trabajar
  dicen `clona primero el repositorio`.

### Revisar una solicitud

**Revisar** (`e`) en una solicitud de los repositorios propios del ámbito abre la pestaña de ese repositorio con la revisión
dentro. En una solicitud de una fila de toda la cuenta, el camino depende de si se conoce un clon y del ajuste
**Solicitud de un repositorio no clonado** del perfil (Ajustes > Integraciones, `review_mode`
en [la referencia de configuración](config-reference.md)):

| Ajuste | Se conoce un clon | No se conoce ningún clon |
|---|---|---|
| Preguntar (por defecto) | pregunta: **Abrir el clon local** (`l`) o **Revisar mediante la API** (`a`) | pregunta: **Clonar y luego revisar** (`c`) o **Revisar mediante la API** (`a`) |
| Clonarlo | abre el clon | abre el diálogo Clonar |
| Revisar mediante la API | revisa mediante la API | revisa mediante la API |

La pregunta es un diálogo pequeño: las letras eligen, **Enter** elige la fila resaltada, `Esc`, **Cancelar**,
la `✕` y un clic fuera lo cierran, y **Recordar para GitHub** (`r`) escribe el ajuste del perfil para que
no se vuelva a preguntar (la fila de Ajustes lo cambia de nuevo). `Ctrl+N` marca sus controles con insignias.

- **Revisar mediante la API** abre la revisión en la ventana de Inicio, sobre el Radar, con el código tomado
  del host (ver [Revisar sin un clon](review.md#revisar-sin-un-clon)). `Esc` o la `✕` devuelve
  el Radar; las filas *Revisión: ...* de la paleta funcionan sobre ella.
- **Clonar y luego revisar** abre el diálogo Clonar en la cuenta conectada de la que vino la fila (la misma lista de
  repositorios, agrupada por propietario), con el repositorio de la solicitud seleccionado y la carpeta de clonado por
  defecto rellenada; cambia cualquiera de las dos y clona, con el mismo inicio de sesión que cualquier clon de esa
  cuenta. Si el repositorio no está en la lista de la cuenta (no eres miembro), la lista sigue abierta con la búsqueda
  en su nombre y una nota lo dice; para un host que no puede listar repositorios, el diálogo se abre en **Clonar con URL**
  con la dirección rellenada. Cuando el clon ha terminado, el nuevo repositorio se vincula al host
  (su fuente de PULL REQUESTS, y su fuente de ISSUES cuando el host tiene issues y no hay ninguna fijada), se abre su pestaña
  y la revisión de esa solicitud queda abierta en ella. Cancelar el diálogo, o un clon que falla, no vincula nada.
- Las solicitudes de Azure DevOps y Bitbucket no tienen revisión en la aplicación: **Revisar** abre la página de la solicitud en el
  navegador y lo dice en la línea de estado.

Las filas de la paleta *Radar: Revisar mediante la API* y *Radar: Clonar y revisar...* se saltan la pregunta.

Dos casillas junto al filtro (aparecen cuando se está consultando una lista de toda la cuenta), guardadas en
`radar.toml`:

| Casilla | Tecla | Por defecto | Efecto |
|---|---|---|---|
| `[x] No clonados` | `a` | activada | Lista las filas de repositorios sin clon conocido. Desactivada, solo quedan las filas de clones conocidos. |
| `[ ] Borradores` | `d` | desactivada | Lista las solicitudes en borrador de las filas de toda la cuenta. Los borradores de los repositorios propios del ámbito siempre se listan. |

Ambas están en las insignias de `Ctrl+N` y en la paleta (`Radar: Mostrar solicitudes no clonadas`, `Radar: Mostrar borradores`).

La respuesta de un perfil se guarda en `radar-account.json` durante **5 minutos**: si vuelves a abrir el Radar en ese
tiempo, muestra las filas sin consultar al host. **Actualizar** (`r` o `F5`) siempre consulta: cada repositorio y cada lista de toda la cuenta, sea cual sea la antigüedad de la última respuesta. Un host que dijo que fueras más despacio
o rechazó el token no se vuelve a consultar en la misma ronda. Se conservan como máximo 300 filas por perfil y tipo,
y una línea bajo **Repositorios** dice cuántas más había, o por qué no se pudo leer un perfil
(**Reconectar** abre Ajustes > Integraciones).

## Límites

- Las filas de toda la cuenta vienen de perfiles de GitHub, GitLab (incluido el autoalojado), Gitea, Forgejo, Azure DevOps,
  Bitbucket Data Center y Gerrit (no Bitbucket Cloud). Los issues asignados a ti vienen de los cinco primeros;
  Bitbucket y Gerrit no tienen issues.
- Las listas por repositorio usan perfiles de GitHub, GitLab (incluido el autoalojado), Gitea, Forgejo, Azure DevOps,
  Bitbucket y Gerrit; los checks, los conflictos y las menciones solo se leen para los cuatro primeros.
- Las menciones de GitLab solo vienen de las tareas pendientes; los checks de GitHub, Gitea y Forgejo se piden para 15 solicitudes por repositorio.
