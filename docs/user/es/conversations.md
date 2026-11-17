[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · **Español** · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Conversaciones

La vista de conversación de un issue, un elemento de trabajo, una tarjeta o una pull / merge request. Necesita una cuenta y un repositorio vinculado: ver [Integraciones de alojamiento y de gestores de issues](hosting-integration.md); las listas que la abren están en [Pull requests e issues](pull-requests-and-issues.md); revisar código está en [Revisar una pull request](review.md).

Una **conversación** es todo lo que se dice sobre un issue, un elemento de trabajo, una tarjeta, un cambio o una pull /
merge request: la descripción, los comentarios, las revisiones, los hilos sobre el código y los eventos (etiquetas,
asignaciones, cambios de estado, pushes). GitPoulpi la lee del host, la muestra con formato y te deja responder sin salir
de la ventana.

### Abrir una

| Desde | Cómo |
|---|---|
| Una fila de ISSUES | `Enter` o un doble clic; el **Abrir conversación** del menú de la fila; la paleta *Alojamiento: Abrir la conversación de un issue...* (una lista de los issues en pantalla). El **Abrir en el navegador** del menú de la fila conserva el navegador. |
| Una fila de PULL REQUESTS en Azure DevOps o Bitbucket | `Enter` o **Abrir conversación**. En GitHub, GitLab, Gitea, Forgejo y Gerrit la fila abre la [revisión](review.md), cuya página Resumen es esta misma conversación (la línea de tiempo, las teclas y la caja de comentario de más abajo). |
| Una tarjeta del [tablero de issues](pull-requests-and-issues.md#el-tablero-de-issues) | `Enter` o un doble clic. |
| El [Radar](radar.md) | `i` en una fila de issue o de request, el **Abrir conversación** del menú de la fila, o *Radar: Abrir la conversación del issue*. |
| Una referencia en otra conversación | Haz clic en ella (o `l`, y elígela): `#7`, `!12`, `ABC-12` de una fuente vinculada se abren en la app y se apilan en una pila de Volver. |
| Un agente | `gitpoulpi ui show issue <n> --view conversation` (también `ui show pr`), ver [la referencia de la CLI](cli-reference.md). |

**Repositorios que no están clonados.** Una fila del Radar de un repositorio del que no tienes clon también abre su
conversación: la vista solo necesita el perfil de integración y el proyecto, así que el Radar la aloja sobre su propia
ventana, y `Esc` devuelve el Radar. Lo que necesita un clon se deja fuera: una referencia a un commit se abre en el host
en lugar de en el Log, y *Dejar que la IA lea el repositorio* queda en gris con su motivo (ver [el capítulo de
IA](ai.md#ayuda-para-escribir-en-un-host)). Una request de un repositorio clonado tiene en su lugar su revisión (`e`), y
el **Abrir conversación** del Radar dice que no hay conversación que leer aquí para ella; un issue de un repositorio
clonado la abre en la pestaña de ese repositorio.

Un issue de un host que no tiene ninguno que leer (Bitbucket y Gerrit no tienen issues) abre el navegador como antes, y
una compilación sin la función `conversation` hace lo mismo en todas partes ([compilaciones
personalizadas](custom-builds.md)).

### La vista

Una vista a ventana completa: la cabecera (estado, quién la abrió y cuándo, las etiquetas, el número de comentarios,
**Mostrar eventos**, **Volver**, **Actualizar**, **Abrir en el navegador**), la línea **Publicando como**, la línea de
tiempo y, debajo, la caja de comentario.

![La conversación del issue #7: la descripción con un enlace a #9, dos comentarios, un evento de etiqueta mostrado por Mostrar eventos y la caja Nuevo comentario vacía con Escribir, Previa y Dividida](../images/issue-conversation.png)

![La página Resumen de la merge request !58: la descripción, un hilo de código sin resolver y otro resuelto con sus enlaces de archivo y línea, los comentarios, Responder y Citar en el hilo seleccionado y la caja de comentario](../images/pr-conversation.png)

- La línea de tiempo se lee del más antiguo al más reciente y empieza por la ventana más reciente (como máximo 300
  entradas). **Cargar anteriores** en su parte superior (`m`, la paleta *Conversación: Cargar entradas anteriores*) lee
  la ventana previa, con el número que falta cuando el host lo dice.
- Un comentario muestra su autor, la fecha, un chip `editado`, un chip `bot` para los bots, `interno` para las notas
  privadas, y los botones **Responder**, **Citar**, **Editar**, **Eliminar** y **Enlaces** que permite la entrada
  seleccionada. Un hilo sobre código muestra `ruta:línea`, `resuelto` o `sin resolver`, `obsoleto` y sus comentarios. Una
  revisión muestra su veredicto.
- Los eventos son una línea atenuada cada uno (*carol añadió la etiqueta lfs*). **Mostrar eventos** (`v`) los oculta; la
  elección se conserva por repositorio. Los hosts sin historial de eventos (YouTrack, los elementos de trabajo de Azure
  DevOps, las notas locales) no tienen esa casilla.
- El texto se muestra como Markdown renderizado: encabezados, listas anidadas y de tareas, citas, avisos al estilo de
  GitHub, código resaltado, tablas, enlaces, imágenes como `[image: alt]`. El texto que llegó en el markup propio del
  host se convierte antes (ver [Conversión de markup](#conversión-de-markup)). Un bloque de código `mermaid` tiene un
  botón **Abrir diagrama** que muestra el código fuente del diagrama en un visor desplazable (`Enter` lo copia); la
  paleta tiene *Conversación: Abrir el diagrama de la entrada*. Necesita una compilación con `conversation` y `mermaid`.
- En el Resumen de la revisión, el `ruta:línea` de un hilo de código es un enlace al código (`G`, *Conversación: Abrir el código del hilo*;
  véase [Revisión](review.md)); en una conversación abierta por sí sola (un issue, una fila del Radar) no hay página de código a la que ir.
- La vista vuelve a leer cada 60 segundos y con `F5` (**Actualizar**).

**Teclas.** Cada una tiene un botón, una fila de menú o una fila de paleta (*Conversación: ...*), y `Ctrl+N` pone
insignias en los botones.

| Tecla | Acción |
|---|---|
| `Arriba`, `Abajo`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Entrada anterior / siguiente |
| `PageUp`, `PageDown`, `Home`, `End`, la rueda, la barra de desplazamiento | Desplazar |
| `Tab`, `Shift+Tab` | Línea de tiempo, caja de comentario y luego los botones de la cabecera |
| `r` | Responder: en el hilo de la entrada seleccionada, o con su texto citado donde el host no tiene hilos |
| `Q` | Citar la entrada seleccionada en un comentario nuevo |
| `c` | Comentario nuevo |
| `e`, `d` | Editar, eliminar uno de tus propios comentarios (antes de eliminar se pregunta) |
| `l`, `Enter` | Los enlaces de la entrada: `Enter` sigue el único, o abre el menú de ellos |
| `y`, `o` | Copiar la dirección de la entrada, abrirla en el navegador |
| `n` | Siguiente entrada sin leer |
| `v`, `m`, `F5` | Mostrar u ocultar eventos, Cargar anteriores, Actualizar |
| `Alt+Left`, `Backspace` | Volver por la pila de Volver |
| `q`, `Esc` | Cerrar (pregunta antes cuando la caja contiene texto) |

### Escribir

**Comentario nuevo** (`c`, o `Tab` hasta la caja), **Responder** (`r`), **Citar** (`Q`) y **Editar** (`e`) usan el mismo
editor de Markdown: **Escribir**, **Previa** y **Dividida** (`Alt+P` alterna Escribir y Previa, `Alt+L` activa Dividida y
necesita una ventana de 100 columnas). `Ctrl+Enter` (también `Ctrl+S` y `Alt+Enter`) o **Enviar** publica, **Cancelar**
descarta la caja, y `Alt+E` abre **Escribir con IA** donde la IA está activada ([el capítulo de
IA](ai.md#ayuda-para-escribir-en-un-host)). La vista previa se maqueta con el mismo renderizador que la línea de
tiempo, con las mismas referencias. Cuando el host no admite Markdown, el pie nombra el markup en el que se envía el
texto (*Enviado como Jira wiki markup*, *Enviado como Textile*).

- El comentario se añade a la línea de tiempo al instante y ocupa su lugar cuando responde el host. Si el host lo
  rechaza, la entrada desaparece y la caja se abre de nuevo con tu texto y el motivo.
- Un texto que no has enviado se conserva por elemento (`remote-conversations.toml` en la carpeta de configuración) y
  está ahí la próxima vez que abras esa caja; enviarlo o descartarlo lo elimina. Cerrar una caja que contiene texto
  pregunta antes.
- **Responder** en un hilo (un hilo de revisión, una discusión de GitLab, un comentario de Linear, un hilo de Bitbucket
  o de Azure DevOps) publica dentro de ese hilo. Donde el host solo tiene una lista plana, **Responder** inicia un
  comentario nuevo con la entrada seleccionada citada (`> **autor**`), igual que **Citar**.
- **Nota interna** (casilla, `Alt+I`): en GitLab y Redmine el comentario es visible solo para los miembros del
  proyecto. Solo se muestra en los hosts que tienen esas notas, y solo en un comentario nuevo.
- Publicar es una acción hacia el exterior: no se registra nada en el diario y **nada se puede deshacer desde
  GitPoulpi**. Sin conexión la caja se conserva, **Enviar** queda en gris con el motivo, y la cabecera dice que se
  muestra la copia guardada.

**Quién puede editar o eliminar.** Solo tus propios comentarios llevan **Editar** y **Eliminar**. Manda primero la
palabra del propio host cuando da una (`viewerCanUpdate` y `viewerCanDelete` de GitHub, `can_edit` en otros), y luego tu
login frente al autor del comentario, comparados sin distinguir mayúsculas; los comentarios de un bot de App deben venir
además de una cuenta de bot. Mientras no se conoce el login de un bot de GitHub App, solo cuentan los comentarios que
publicó esta instalación (el repositorio guarda sus ids): no se adivina nada a partir de un nombre. En un hilo,
**Editar** y **Eliminar** actúan sobre el último comentario del hilo que es tuyo. Un host que no puede editar ni
eliminar (ver [la tabla](#qué-no-puede-hacer-cada-host)) muestra los botones en gris con el motivo.

### Publicar como

La línea bajo la cabecera dice con qué nombre se publican tus mensajes, y la misma línea está en cada caja:

| Línea | Significado |
|---|---|
| *Publicando como ann (token personal)* | El token del perfil: tu propio login (`GET /user`). |
| *Publicando como ci-bot (comando de token del repositorio)* | Se usa el [comando de token](hosting-integration.md#8-actuar-como-bot-de-una-github-app) del repositorio para la API del host. |
| *Publicando como my-app[bot] (bot de GitHub App)* | Un token de instalación. El nombre viene de **Publicar como**, si no del host. |
| *Publicando como el bot de la GitHub App (aún no se sabe su nombre: define Publicar como en Ajustes)* | No hay nombre configurado y aún no se ha aprendido ninguno. |
| *Publicando como ... (bot del proyecto)* | Un token de acceso de proyecto o de grupo de GitLab. |
| *Escribiendo como kai (nota local)* | Una carpeta de notas Markdown: el `local.me` del perfil, si no el usuario conectado. |
| *Aún no se sabe con qué nombre publicarías. Define Publicar como en Ajustes.* | No se pudo aprender nada. |

**Publicar como** es un nombre: para una GitHub App es `my-app[bot]`. Defínelo para todos los repositorios en **Ajustes >
Integraciones > Publicar como** (`general.hosting_post_as`), o para un repositorio en **Ajustes > Ajustes del repositorio > SSH y
credenciales > Alojamiento**, que lo sustituye. Si se deja vacío, GitPoulpi pregunta a GitHub (GraphQL `viewer { login }`,
que un token de instalación puede responder o no) y, si no, aprende el nombre del autor del primer comentario que
publica y lo conserva con el repositorio. Un token personal nunca lo necesita. Ver [Actuar como bot de una GitHub
App](hosting-integration.md#8-actuar-como-bot-de-una-github-app).

### Referencias cruzadas

Las referencias en una conversación aparecen subrayadas y se pueden pulsar. Primero se comparan con las fuentes de
ISSUES y de PULL REQUESTS vinculadas del repositorio, así que `ABC-12` de un proyecto de Jira vinculado se abre en la
app incluso dentro de un comentario de GitHub; lo que añade la gramática propia del host abre su página tras la
confirmación del enlace. Un id de commit se abre en el Log cuando el repositorio está clonado y el id tiene al menos 7
caracteres, y en el host en caso contrario. Nada dentro de un fragmento de código, un bloque delimitado, un enlace
Markdown, una dirección de correo o una URL es una referencia, y tampoco lo son `C#1`, `UTF-8`, `SHA-256` ni `ISO-8601`.

| Host | Se reconoce |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, ids de commit |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (un proyecto hermano), `@user`, `~label` y `%milestone` como chips, ids de commit |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, ids de commit |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, menciones `{uuid}`, ids de commit |
| Gerrit | valores `Change-Id` (`I` y 40 dígitos hexadecimales), ids de commit |
| Redmine | `#7`, revisiones `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, solo para un proyecto o equipo de una fuente vinculada |
| Trello | direcciones de tarjeta (`trello.com/c/...`) |
| Notas locales | `[[note]]`, `[[note\|alias]]` y enlaces `.md` relativos a otra nota de la carpeta |

`gitpoulpi hosting markdown --body T --id I` imprime lo que la app hace con un texto, referencias incluidas.

### Conversión de markup

Los hosts que no hablan Markdown se convierten en ambos sentidos, así que lees y escribes Markdown en todas partes. Lo que
un conversor no entiende queda como texto legible.

- **Jira** (Cloud y Data Center) guarda los comentarios como wiki markup a través de la API v2. Lo que lees se convierte a
  Markdown; lo que escribes se convierte a wiki markup, y el pie dice *Enviado como Jira wiki markup*.
- **Redmine** da formato al texto como Textile o Markdown, algo que la API no dice: la fila **Formato de texto del
  servidor** del perfil (Ajustes > Integraciones, `markup`, Textile por defecto) se lo indica a GitPoulpi. Textile se
  convierte a Markdown para leer y de vuelta a Textile para escribir.
- **Azure DevOps Server** devuelve los comentarios de los elementos de trabajo como HTML y los acepta como HTML: se
  muestran como el Markdown convertido a partir de él y se escriben como HTML. Azure DevOps Services y los hilos de las
  pull requests aceptan Markdown.
- **Gerrit** es texto plano: no se renderiza como Markdown y lo que escribes se envía tal como lo tecleas.
- Todo importador elimina primero las secuencias de control y de escape, así que un comentario no puede mover el
  terminal.

### La copia guardada, Cargar anteriores, sin leer

- Cada conversación que abres se guarda (`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`, solo de
  usuario, sin ningún token dentro, como máximo 2 MB por archivo, se conservan los 50 archivos más recientes). Se pinta al
  instante cuando se abre el elemento y se actualiza por detrás. Cuando no se puede contactar con el host, la copia
  guardada permanece en pantalla bajo *Sin conexión: se muestra la copia guardada hace 3 min (motivo)* y Enviar queda en
  gris.
- **Sin leer:** lo último que viste se recuerda por conversación (se conservan las últimas 200). Las entradas de otra
  persona que son más nuevas reciben una barra de acento y un chip `nuevo`, y `n` salta a la siguiente. Una conversación
  que abres por primera vez no tiene nada nuevo.

### Qué no puede hacer cada host

Cada `no` de abajo es un límite real: el control se oculta o queda en gris con el motivo, y la CLI responde con un error
que nombra el host antes de enviar nada.

| Host | Elementos | Responder en un hilo | Editar y eliminar los propios | Resolver un hilo | Eventos | Notas internas | Se escribe como |
|---|---|---|---|---|---|---|---|
| GitHub | issues, pull requests | pull requests | ambos | pull requests | sí | no | GitHub Markdown |
| GitLab | issues, merge requests | sí (discusiones) | ambos | merge requests | sí | sí | GitLab Markdown |
| Gitea | issues, pull requests | pull requests | ambos | pull requests | sí | no | Markdown |
| Forgejo | issues, pull requests | pull requests | ambos | no (el host no tiene endpoint) | sí | no | Markdown |
| Azure DevOps | elementos de trabajo, pull requests | pull requests | ambos | no (el estado se muestra, no se cambia) | solo pull requests | no | Markdown (HTML en los elementos de trabajo de Server) |
| Bitbucket (Cloud, Data Center) | solo pull requests | sí | ambos | no (se muestra, no se cambia) | sí | no | Markdown |
| Gerrit | solo cambios | sí | no (no se puede editar ni eliminar ninguno) | sí | sí | no | texto plano |
| Jira | issues | no (lista plana) | ambos | no | sí (changelog) | no (necesita un rol de proyecto) | wiki markup |
| Linear | issues | sí (respuestas a un comentario) | ambos | no | sí | no | Markdown |
| YouTrack | issues | no | ambos | no | no | no | Markdown |
| Redmine | issues | no | solo editar (Redmine 5 y posteriores), nunca eliminar | no | sí (journals) | sí (notas privadas) | Markdown o Textile |
| Trello | tarjetas | no | ambos | no | sí (movimientos) | no | Markdown |
| Notas locales | notas | no | los propios del autor local | no | no | no | Markdown |

Además: ningún host te deja iniciar un hilo nuevo sobre una línea desde la conversación (hazlo en la revisión), y las
reacciones solo se muestran en GitHub. Se rechaza un texto más largo de lo que admite el host (GitHub 65.536
caracteres, GitLab 1.000.000, Azure DevOps 150.000, Jira 32.767, Trello 16.384): la CLI lo comprueba antes, y en la app
se muestra el mensaje del propio host y se conserva tu texto. Una nota local guarda sus comentarios en una sección final
`## Comments` del propio archivo de la nota, con una entrada `### autor - hora` cada uno, así que Obsidian y una shell
también los ven.

En la línea de comandos las mismas conversaciones son `gitpoulpi hosting issue comments`, `issue comment`,
`comment-edit` y `comment-delete`, y `hosting review conversation` (ver [la referencia de la
CLI](cli-reference.md)); para los agentes son herramientas MCP con los mismos nombres (ver [MCP y
agentes](mcp-and-agents.md)).
