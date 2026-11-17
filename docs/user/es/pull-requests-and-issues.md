[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · **Español** · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests e issues

Leer, crear y gestionar pull / merge requests e issues desde la barra lateral, el tablero y la barra de herramientas. Antes hay que conectar una cuenta y vincular un repositorio: ver [Integraciones de alojamiento y de gestores de issues](hosting-integration.md). Todo lo que se dice sobre un issue o una request (comentarios, revisiones, hilos) está en [Conversaciones](conversations.md); leer y revisar una request está en [Revisar una pull request](review.md).

## Pull requests e issues en la barra lateral

### PULL REQUESTS

La sección tiene una cabecera con el recuento y un `+` (crear una pull request),
los desplegables de proveedor y de proyecto, un cuadro de búsqueda y un desplegable de estado (*Abiertas*,
*Merged / cerradas*, *Todas*). Después vienen cuatro grupos, cada uno con su recuento:

- **Mis pull requests**: eres el autor.
- **Asignadas a mí**.
- **Pendientes de mi revisión**: eres un revisor solicitado.
- **Todas las pull requests**: plegado al principio.

El cuadro de búsqueda filtra todos los grupos (título, número, autor, rama). Cada
cambio del desplegable de estado vuelve a preguntar al host.

Una fila empieza con un glifo de estado: `●` abierta (verde), `○` borrador, `◆` merged
(morado), `✕` cerrada (rojo). La request de la rama activa va en negrita.
Las filas se leen `#12  título` (GitHub, Bitbucket) o `!53  título` (GitLab,
Azure DevOps).

### ISSUES

El título de la sección nombra al proveedor: `ISSUES DE GITHUB`, `ISSUES DE JIRA`, y así
sucesivamente. Dice `ELEMENTOS DE TRABAJO DE ...` para Azure DevOps y `TARJETAS DE ...` para Trello.
Bajo los desplegables: un cuadro de búsqueda, el desplegable de estado (*Abiertos*,
*Asignados a mí*, *Cerrados*, *Todos*) y los issues. Una fila de Jira se lee
`ABC-12  título`, con el tipo y el estado atenuados cuando la barra lateral es lo bastante
ancha.

![La sección ISSUES de la barra lateral con el menú contextual del issue #7 abierto, Abrir conversación primero](../images/issues-sidebar.png)

### Acciones sobre una fila

- `Enter` o un doble clic abre la fila: una pull request en la [revisión](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit), un issue, y una request de Azure DevOps o de Bitbucket, como [conversación](conversations.md) en la ventana, y cualquier otra cosa en tu navegador. El **Abrir en el navegador** del menú de la fila abre siempre el navegador.
- Clic derecho, o `.`, abre el menú de la fila:
  - **Abrir conversación** (issues, y requests que no tienen vista de revisión), **Abrir en el navegador**, **Copiar URL**, **Copiar número**.
  - Para una pull request: **Editar en <proveedor>**, **Checkout de la rama de origen**,
    **Ver commits de la rama de origen**. Para una request desde un fork, el checkout dice
    **Checkout de pull request (fork)**, y **Ver** aparece en gris.
  - Para un issue: **Crear rama desde el issue**. Abre el aviso de nueva rama
    con un nombre como `12-fix-the-thing` (o `ABC-123-fix-the-thing` para
    Jira).
  - En GitHub, GitLab, Gitea y Forgejo, una fila de pull request tiene también **Revisar...**
    (y también **Enter** sobre la fila): la revisión dentro de la aplicación, ver
    [Revisar una pull request](review.md).
  - En GitHub, GitLab, Gitea y Forgejo, ambas filas tienen también **Editar asignados...**,
    **Editar etiquetas...** y **Establecer hito...** (y **Editar revisores...** en
    una pull request): ver
    [Asignados, revisores, etiquetas e hito](#asignados-revisores-etiquetas-e-hito).
- **Editar en <proveedor>** tiene otras tres vías de acceso: el lápiz `✎` en el extremo
  derecho de una fila de pull request con el ratón encima o seleccionada, `e` en la fila seleccionada, y
  el comando de la paleta *Integraciones: Editar pull request vinculada en el host*
  (para la request de la rama activa).
- Si la rama de origen todavía no está en el repositorio, GitPoulpi lo dice. Haz fetch
  primero (`Ctrl+G f`).

**Checkout de una pull request.** GitPoulpi hace fetch de la request a una rama local
`pr/<n>` (`mr/<n>` en GitLab) que la sigue, y cambia a ella. En
GitHub, GitLab, Gitea y Forgejo esto funciona también con forks (GitPoulpi hace fetch de la ref head
especial del host). Azure DevOps y Bitbucket hacen fetch de la rama de origen de la
request. Allí una request desde un fork se rechaza: añade el fork como remoto y haz
checkout de su rama tú mismo. Deshacer vuelve a la rama anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Revisar una pull request desde la línea de comandos (y un agente).** En GitHub
(incluido Enterprise), GitLab (incluido autoalojado), Gitea y Forgejo una request se puede
leer y revisar sin salir de GitPoulpi: `hosting review show <n>` (archivos,
estado del merge, checks), `hosting review threads <n>` (hilos de revisión y
conversación), `hosting review comment`, `hosting review reply`, `hosting review resolve`
y `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. El diff viene de tu repositorio, no del
host: `hosting review fetch <n>` hace fetch del head de la request a
`refs/gitpoulpi/pr/<n>` y de su rama de destino a `refs/gitpoulpi/pr/<n>-base`
(ninguno es una rama, no se hace checkout de nada). Los números de línea son los de ese
diff: `new` es la línea en el head, `old` la línea en la base del merge. GitLab
no tiene el estado de revisión «solicitar cambios»: allí se rechaza, sin enviar nada.
Gitea y Forgejo admiten un comentario sobre una sola línea (sin rangos); un hilo son los
comentarios sobre una línea de un archivo. Gitea puede resolver un hilo, Forgejo 11 no
(el host no tiene endpoint para ello: la llamada lo dice). Una respuesta en un servidor
sin el endpoint de respuestas se publica como una revisión de un comentario en la misma línea.
Gitea se niega a aprobar o rechazar tu propia request.
Sin clon: añade `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` y `suggest` para revisar un proyecto que no es el repositorio actual,
`hosting review diff <n> --project P` lee el diff (hunks por archivo) desde la API del host, y
`hosting file <path> --project P [--ref R]` lee un archivo en una revisión (GitHub, GitLab, Gitea, Forgejo y Gerrit; en Gerrit el diff de un
patch set se lee archivo por archivo y el archivo sale de la rama o, para un id de commit, del commit). Un archivo que el host no
muestra en el diff (demasiado grande) se marca como `omitted`.
Las herramientas MCP son los mismos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Asignados, revisores, etiquetas e hito

En GitHub (incluido Enterprise), GitLab (incluido autoalojado), Gitea y
Forgejo, puedes cambiar a quién está asignada una pull request o un issue, quién la revisa, sus etiquetas
y su hito sin salir de GitPoulpi. Los demás proveedores no muestran nada de esto.

- **Menú de la fila**: **Editar asignados...**, **Editar revisores...** (solo pull
  requests), **Editar etiquetas...**, **Establecer hito...**.
- **Paleta**: *Alojamiento: Editar asignados...*, *Alojamiento: Editar revisores...*,
  *Alojamiento: Editar etiquetas...*, *Alojamiento: Establecer hito...* listan primero las
  requests y los issues de la barra lateral entre los que elegir. En el
  [Radar](radar.md) las mismas cuatro actúan sobre la fila seleccionada.
- El selector se abre al instante y carga del host las personas o etiquetas del proyecto (y lo que
  tiene ahora el elemento). Una fila con indicador giratorio aparece pasados unos 150 ms.
  Escribe para filtrar (coincidencia aproximada), `Up` / `Down` mueven, `Space` marca o
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` llega a los botones. Un clic
  en una fila la marca, y la rueda desplaza. Las personas muestran una insignia con sus iniciales,
  las etiquetas su color.
- El hito es un desplegable (con **Sin hito** para quitarlo).
- Tras **Aplicar** se actualiza el host, y las listas y el Radar muestran el
  cambio tras su siguiente actualización, que empieza enseguida.

Los formularios de creación (una pull request, un issue) tienen las mismas opciones como una fila de
botones: `Asignados: ninguno`, `Revisores: ninguno`, `Etiquetas: ninguno`,
`Hito: ninguno`. En GitHub, GitLab, Gitea y Forgejo el botón Etiquetas sustituye al campo de
etiquetas escrito del formulario de issue. Si un host acepta la nueva pull request pero
rechaza algunas de sus etiquetas o personas, la barra de estado lo dice.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, y `--assignee`, `--reviewer`, `--label`,
`--milestone` en `hosting pr` y `hosting issue create`; ver la
[referencia de la CLI](cli-reference.md).

### Teclas en las secciones

| Tecla | Acción |
|---|---|
| `Enter` / doble clic | abre la conversación en GitPoulpi donde el host la tiene (una pull request de GitHub, GitLab, Gitea o Gerrit abre la revisión), si no el navegador |
| `o` | abrir la fila en el navegador |
| `.` / clic derecho | menú de la fila |
| `e` | editar la pull request seleccionada en el host |
| `n` | nueva pull request (PULL REQUESTS) o nuevo issue (ISSUES) |
| `v` | abrir el desplegable de estado |
| `r` | actualizar |
| `Enter` / `Space` en una cabecera | plegar o desplegar la sección |

### Reconectar y sin conexión

Cuando falta el token, se ha rechazado o le falta un ámbito, la primera fila de la
sección dice `Reconectar (token rechazado)`. Haz clic en ella, o pulsa `Enter`, para abrir
el perfil en Ajustes > Integraciones.

Cuando no se puede contactar con el host, las últimas listas se quedan en pantalla y la cabecera
muestra su antigüedad (`· de hace 5min`). La última respuesta también se guarda en caché en disco
(`.gitpoulpi/hosting-cache.json`), así que las listas aparecen antes de que la red
responda.

## El tablero de issues

El botón **Tablero** de la cabecera ISSUES (paleta: **Issues: Abrir tablero**) muestra los issues del origen vinculado como un tablero
Kanban sobre la ventana. Las columnas dependen del origen:

| Origen | Columnas | Mover una tarjeta |
|---|---|---|
| Notas Markdown locales | los estados (`statuses` del perfil, y después cualquier otro en uso) | reescribe el `status` de la nota |
| GitHub, GitLab, Gitea, Forgejo | **Abiertos**, después las etiquetas que elijas, después **Cerrados** | intercambia la etiqueta de estado; a **Cerrados** cierra el issue, a una columna de nuevo lo reabre |
| Trello | las listas abiertas del tablero, de izquierda a derecha | mueve la tarjeta a la lista |
| Redmine | los estados del issue | cambia el estado del issue (se aplican las reglas de flujo de Redmine) |
| Jira (Cloud y Data Center) | los estados del proyecto, los pendientes primero y los hechos al final | ejecuta la transición de flujo que lleva a la columna (`GET` y luego `POST .../transitions`); si no hay tal transición la tarjeta vuelve y la barra de estado dice `no transition from X to Y` |
| Azure DevOps (Services y Server) | los estados de los tipos de work item del proyecto, por categoría de estado | fija `System.State`; un work item cuyo tipo no tiene el estado de la columna vuelve, con el motivo (`a Bug has no state Done`) |

En un proyecto de GitHub o GitLab, **Columnas** (o `c`, o **Tablero: Definir etiquetas de columna...**) pide las etiquetas que son las
columnas, en orden, por ejemplo `status::todo, status::doing, status::review`. Propone las etiquetas del proyecto que
parecen estados. La elección se guarda para el repositorio. Sin etiquetas el tablero es Abiertos / Cerrados. Mover una tarjeta
solo toca las etiquetas de estado que listaste: sus demás etiquetas se quedan. Un tablero de host lista los issues abiertos y los 60 cerrados
más recientemente.

Una transición de Jira que pide campos obligatorios (una resolución, por ejemplo) la rechaza Jira y la tarjeta vuelve con
el propio mensaje de Jira: haz esa en el sitio web.

El mismo tablero está disponible para scripts y agentes: `gitpoulpi hosting board` imprime las columnas con sus tarjetas y
`gitpoulpi hosting board move <card> <column>` mueve una (consulta [la referencia de la CLI](cli-reference.md)).

Una tarjeta muestra el número, el título, los asignados como iniciales, las etiquetas y el hito. Arrástrala a otra columna,
o `Shift+Left` / `Shift+Right`, o **Tablero: Mover tarjeta a...** (`m`). Se mueve al instante; pasados unos 150 ms un indicador giratorio
dice que el host sigue trabajando, y si el host lo rechaza, la tarjeta vuelve y la barra de estado dice por qué. `Enter` abre
la [conversación](conversations.md) del issue en GitPoulpi, y el tablero vuelve al cerrarla; `e` edita asignados, etiquetas e hito; `n` inicia un issue nuevo en
la columna seleccionada; `a`, `l`, `M` y `/` filtran por asignado, etiqueta, hito y texto, `x` quita los filtros; `r`
vuelve a leer el host. El tablero vuelve a leer cuando se actualizan las listas de la barra lateral. El Radar no tiene tablero.

## Crear una pull request

Abre el formulario con el `+` de la cabecera PULL REQUESTS, `n` en la sección, o
el comando de la paleta *Pull requests: Crear pull request...*. Un menú de rama
también tiene **Iniciar una pull request desde <rama>...**.

El formulario tiene:

- Una pestaña por cada perfil conectado que tenga pull requests (`Left` / `Right` en
  la tira de pestañas).
- Comboboxes **Repo de origen / Rama** y **Repo de destino / Rama**. Las listas largas admiten
  búsqueda. Puedes escribir una rama o un repositorio que las listas no muestren.
- **Título**, rellenado con el asunto del commit de la punta.
- **Descripción**, en varias líneas (`Enter` añade una línea nueva, `Tab` sale del
  campo).
- Botones **Asignados**, **Revisores**, **Etiquetas** e **Hito** (GitHub,
  GitLab, Gitea y Forgejo), que abren el selector.
- **Enviar como borrador**.
- Con la [IA](ai.md) activada: **Generar título y descripción**.

El formulario se abre con el perfil, el repositorio y las ramas de la rama
activa (`feature -> main`). Un repositorio de origen distinto crea una request
desde un fork.

**Crear pull request** (o `Ctrl+S`) crea la request en el host. El
resultado se abre en tu navegador y la barra de estado la nombra.

## Crear un issue

Cuando la sección ISSUES tiene un proveedor **y** un proyecto, su cabecera tiene un
`+` (*+ Nuevo issue*). También: `n` en la sección, o el comando de la paleta
*Alojamiento: Nuevo issue...*.

El formulario tiene **Título** (obligatorio), **Descripción** (varias líneas) y
**Etiquetas** (separadas por comas, por ejemplo `bug, ui`; en GitHub, GitLab, Gitea y Forgejo los
botones Etiquetas, Asignados e Hito abren en su lugar el selector). Jira y Azure DevOps
añaden un desplegable **Tipo**. Trello añade un desplegable **Lista**. Linear y YouTrack
añaden un desplegable **Estado** (el estado del flujo, o el valor de `State`, en el que empieza
el issue). Redmine añade un desplegable **Tipo** con los trackers del proyecto (Bug, Feature,
Support...); no tiene etiquetas, así que no hay botón Etiquetas para él y una etiqueta
dada en la línea de comandos se rechaza. El formulario rellena estos
desplegables desde el host al abrirse.

**Crear issue** (o `Ctrl+S`) lo crea en el host. `Esc` o **Cancelar**
cierra el formulario. Si el host lo rechaza (token rechazado, ámbito que falta, proyecto
desconocido, etiqueta desconocida), el formulario muestra el motivo y conserva lo que escribiste.
Si todo va bien, la barra de estado nombra el nuevo issue y la sección ISSUES se
actualiza.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Crear en la propia página web del host

El formulario Nuevo issue y el formulario Crear pull request tienen un
botón **Crear en <proveedor>**, arriba a la derecha. La tecla es `Alt+W`. Abre el
formulario web del host en tu navegador, rellenado con lo que has escrito hasta ahora. El
diálogo sigue abierto. No necesita token y no hace ninguna llamada a la API, así que también funciona
tras un error de la API.

- GitHub no tiene parámetro de URL para *borrador*, así que la casilla de borrador no se
  traslada. GitLab recibe un prefijo `Draft: ` en el título.
- GitLab acepta las etiquetas solo por id, así que se añaden como una acción rápida
  `/label ~"x"` al final de la descripción.
- Una descripción muy larga se recorta para que la URL no pase de 6000 caracteres,
  con una nota `[...truncated: paste the rest]`.
- Bitbucket Data Center no tiene dirección de formulario web, así que no tiene
  botón **Crear en <proveedor>**.

CLI: `gitpoulpi hosting issue create ... --web` y `gitpoulpi hosting pr --web`
imprimen la misma URL.

## El chip de la barra de herramientas

En el extremo de la barra de herramientas (a la derecha por defecto), un chip muestra la pull request de la rama
activa: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) o
`BB #12` (Bitbucket). Su color indica el estado: verde abierta, gris borrador,
morado merged, rojo cerrada.

- Pasa el ratón por encima para ver el título, el estado y `origen → destino`.
- Haz clic en él, o pulsa `Ctrl+G g`, para abrir la request en el navegador.
- Haz clic derecho para **Abrir en el navegador**, **Copiar URL**, **Actualizar** y
  **Cambiar de perfil**.

Una rama con push hecho y sin request muestra un chip tenue `+ PR`. Haz clic en él para iniciar una
pull request. No hay chip cuando ningún perfil coincide, cuando la rama no tiene push,
o cuando no se pudo contactar con el host. En una ventana estrecha el chip se oculta para dejar sitio
a los botones de la barra de herramientas.

## Cuándo se actualizan las listas

Las listas se actualizan:

- cuando abres un repositorio;
- cuando cambia la rama activa;
- tras un push o un fetch;
- cada *Actualización de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, por defecto 5, `0` apaga el temporizador;
  Ajustes > Integraciones);
- cuando cierras Ajustes;
- cuando pulsas `r`, ejecutas *Integraciones: Actualizar pull requests e issues*, o
  cambias un filtro.

Las peticiones nunca bloquean la ventana.

Los hosts limitan cuántas llamadas puedes hacer. GitPoulpi devuelve el `ETag` de la
última respuesta, así que una lista sin cambios cuesta poco (nada en GitHub). Cuando un
host informa de que quedan pocas llamadas, la barra de estado lo dice. Cuando no queda
ninguna, GitPoulpi deja de preguntar a ese host hasta que se restablezca el límite, y las listas
se quedan como están.

## Terminología

GitHub, Azure DevOps y Bitbucket dicen «pull request». GitLab dice «merge
request». *Terminología de las solicitudes de revisión* (`ui.pr_terminology`, en
Ajustes > Integraciones) elige la palabra para cada etiqueta, menú, comando de
la paleta y mensaje:

- **Auto (según el host)**: sigue al proveedor vinculado del repositorio («Pull
  Request» mientras no haya nada vinculado).
- **Pull Request** o **Merge Request**: siempre esa palabra.

El cambio se aplica al instante. La CLI y el MCP siempre dicen «pull/merge request».
Esta página dice «pull request».

## Referencias en mensajes de commit y nombres de rama

Cuando los issues o las pull requests del repositorio están vinculados, las referencias en
el mensaje de commit de los detalles del commit aparecen subrayadas. Un clic abre la
página vinculada. Teclado: el comando de la paleta
*Integraciones: Abrir referencia del commit (#issue / !solicitud)...* (un menú cuando el
mensaje tiene varias referencias). Un repositorio sin nada vinculado no tiene
enlaces.

Lo que cuenta como referencia depende del proveedor de la sección ISSUES:

| ISSUES vinculados a | En un mensaje | En un nombre de rama |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` y `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, solo cuando `ABC` es la clave del proyecto vinculado. Un texto como `UTF-8`, `SHA-256` o `ISO-8601` nunca es un enlace. | `ABC-123` en cualquier parte del nombre (`feature/ABC-123-x`), en mayúsculas |
| Linear | `ENG-123`, solo cuando `ENG` es la clave del equipo vinculado (o de un equipo de la lista de equipos del perfil). `UTF-8` nunca es un enlace. | El propio `branchName` de Linear cuando creas la rama desde un issue; `ENG-123` en cualquier parte del nombre se lee de vuelta |
| YouTrack | `ABC-123`, solo cuando `ABC` es el nombre corto del proyecto vinculado (o de uno de la lista de proyectos del perfil) | `ABC-123` en cualquier parte del nombre |
| Redmine | `#123`, solo cuando Redmine es el origen de ISSUES vinculado (un origen de pull requests conserva su propio signo) | `123-fix-login` |
| Trello | una dirección `https://trello.com/c/<shortLink>`; `#12` solo cuando una tarjeta en caché tiene el número 12 | `12-fix-login` |

| PULL REQUESTS vinculadas a | En un mensaje |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Cuando ambas listas coinciden con `#N`, gana el issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**En las conversaciones.** El texto de un issue o de una request se lee con la gramática de su host, que es más amplia que la tabla de arriba: `owner/repo#7`, `!7`, `GH-7`, `@login`, ids de commit y las demás formas que se listan en [Referencias cruzadas](conversations.md#referencias-cruzadas). Primero se comparan con las fuentes vinculadas del repositorio, así que una clave de Jira en un comentario de GitHub sigue abriendo su issue en la app. `gitpoulpi hosting markdown --body TEXT --id I` muestra lo que la app encuentra en un texto.

### Insertar una referencia a un issue

En el compositor de commits, el botón `#▾` en el extremo derecho del Resumen abre
una lista filtrable de los issues vinculados. El issue nombrado por la rama actual
va primero, marcado *(de la rama)*. `Enter` o un clic inserta su referencia
(`ABC-123`, `#12`, `AB#12`) en el cursor. Teclado: `Alt+R` en el compositor.
Paleta: *Commit: Insertar referencia a un issue...*. Sin una lista de issues vinculada, el
botón aparece en gris y dice *vincula primero una lista de issues en la barra lateral*.
GitPoulpi nunca añade una referencia por sí solo.
