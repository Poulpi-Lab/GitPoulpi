[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · **Español** · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Revisar una pull o merge request

Revisa una pull request (GitHub, Gitea, Forgejo), una merge request (GitLab) o un cambio de Gerrit (ver [Gerrit](#gerrit))
**sin salir de la ventana del repositorio**. La vista de revisión muestra la descripción, la conversación, los archivos
cambiados con los hilos de comentarios bajo sus líneas y los commits, y es donde escribes la revisión: comentarios,
aprobación y peticiones de cambios. Necesita un perfil de uno de esos hosts (ver
[Alojamiento y gestores de issues](hosting-integration.md)) y un repositorio vinculado a él; los demás proveedores no ofrecen
revisiones. Se respetan los límites de cada host: GitLab no tiene el veredicto *Solicitar cambios* y admite una línea por comentario.
Esta página llama a todas ellas una *solicitud*.

![La vista de revisión, página Resumen: la cabecera de la solicitud con sus botones de metadatos, la descripción, un hilo de código sin resolver y otro resuelto, los comentarios y la caja de comentario](../images/review-overview.png)

![La página Archivos: los archivos cambiados con sus casillas de visto, el diff con un hilo de comentarios bajo su línea, y un hilo resuelto fuera de las líneas mostradas](../images/review-files.png)

## Abrir una revisión

Todas las vías llevan a la misma vista:

- la lista **PULL REQUESTS** de la barra lateral: selecciona una fila y pulsa **Enter**, o haz clic derecho en ella y elige
  **Revisar...**;
- el **Radar**: el menú de la fila, **Revisar...**, o la tecla `e` (el repositorio se abre y la revisión con él;
  una solicitud de un repositorio que no has clonado pregunta si clonarlo o revisarla mediante la API,
  ver [Revisar sin un clon](#revisar-sin-un-clon));
- la **búsqueda del espacio de trabajo** (`Ctrl+Alt+F`): resalta una pull o merge request y pulsa **`Alt+R`**;
- la paleta: **Alojamiento: Revisar solicitud...** lista las solicitudes entre las que elegir.

La vista llena la ventana sobre el repositorio (es un modo de la pestaña del repositorio, no una ventana aparte).
**`Esc`** o la `✕` de su esquina la cierra. Mientras se abre, la solicitud se lee del host y su código se trae
con git en segundo plano; aparece un indicador giratorio si tarda más de un momento.

## La cabecera

Siempre visible: el número y el título, el estado (Abierta, Borrador, Fusionada, Cerrada), las ramas
(`main ← feature/moons`), el autor, los checks (`✓ comprobaciones superadas`, `✗ comprobaciones fallidas`, `● comprobaciones en curso`),
si se puede fusionar sin conflictos o tiene conflictos, el tamaño y **`3 comentarios pendientes`** cuando has escrito algunos.

Los cuatro botones de metadatos - **Asignados**, **Revisores**, **Etiquetas**, **Hito** - abren los mismos selectores que
la barra lateral (ver [Asignados, revisores, etiquetas e hito](pull-requests-and-issues.md#asignados-revisores-etiquetas-e-hito));
la cabecera muestra el resultado en cuanto el host lo ha aceptado. Los botones de acción: **Cambiar a la rama** (el checkout
habitual de la rama de la solicitud, que también cierra la vista), **Abrir en el navegador**, **Actualizar**
(**`F5`**) y **Terminar revisión**.

## Tres páginas

Un control segmentado en el borde superior cambia de página: **Resumen**, **Archivos**, **commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, o un clic).

### Resumen

El Resumen es **la conversación de la solicitud**, la misma línea de tiempo y la misma caja de comentario que tiene un
issue ([Conversaciones](conversations.md)): primero la descripción (*abrió esto*), y después los
comentarios, los hilos sobre código con su fragmento de diff, las revisiones enviadas con su veredicto (`aprobada`,
`con cambios solicitados`, `comentada`, `descartada`; en GitLab los veredictos son sus aprobaciones) y los eventos, de la más
antigua a la más reciente, en la ventana de lo más nuevo con **Cargar anteriores** arriba. La caja de comentario está
acoplada bajo la línea de tiempo (**Comentario nuevo**): **Escribir**, **Previa** y **Dividida**, **Cancelar** y **Enviar**.
**Un comentario publicado aquí va al host al instante** (nada queda pendiente: los comentarios pendientes son los que están
sobre código, que se envían con la revisión), aparece en la lista enseguida y ocupa su lugar cuando el host responde; si el
host lo rechaza, la caja se abre de nuevo con tu texto y el motivo.

Las teclas son las de la vista de conversación: **`Up`** / **`Down`** (y `Ctrl+J` / `Ctrl+K`) se mueven entre entradas,
**`r`** responde en el hilo de la entrada seleccionada, **`Q`** la cita en un comentario nuevo, **`c`** escribe un comentario
nuevo, **`e`** / **`d`** editan y eliminan tus propios comentarios (antes de eliminar se pregunta), **`l`** o `Enter` abre los
enlaces de una entrada, **`g`** su diagrama, **`y`** copia su dirección, **`o`** la abre en el navegador, **`n`** va a la
siguiente entrada sin leer, **`v`** muestra u oculta los eventos, **`m`** carga entradas anteriores, `Tab` va entre la línea
de tiempo y la caja, y **Nota interna** (`Alt+I`) está en los hosts que tienen esas notas. Los botones de la entrada
seleccionada (**Responder**, **Citar**, **Editar**, **Eliminar**, **Enlaces**) hacen lo mismo, y la paleta tiene las filas
*Conversación: ...* junto a las *Revisión: ...*. Antes de abrir un enlace se pregunta, como en toda la revisión.

Un hilo sobre código muestra su `ruta:línea` como un enlace (subrayado, un botón de verdad). Un clic en él, **`G`** sobre el hilo seleccionado o *Conversación: Abrir el código del hilo* de la paleta cambian a la página **Archivos**, abren ese archivo y ponen el cursor en la línea comentada (en el lado antiguo o el nuevo, como se escribió) con el hilo abierto debajo (uno plegado se despliega). **Esc**, el botón **Volver al resumen** a la izquierda de las pestañas de la página o *Revisión: Volver al comentario en el resumen* regresan a la misma entrada del Resumen. Un hilo cuya línea ya no está en el diff (obsoleto) abre el archivo en la línea más cercana y la línea de estado lo dice; un hilo sobre un archivo que la solicitud ya no cambia dice que el archivo no está en el diff. En una vista de issue o de Radar de la misma solicitud no hay código que abrir: `G` responde que la revisión lo tiene.

Donde una tecla de la conversación y una de la revisión coinciden, la letra en esta página es la de la conversación: `l`
(etiquetas), `m` (hito), `Q` (abandonar, de Gerrit), `e` / `d`, `r`. Las etiquetas, el hito y abandonar siguen en los botones
de la cabecera y en la paleta; `a` (asignados), `p` (revisores), `f` (**Terminar revisión**), `b` (**Checkout**), `P`, `D`,
`U` y `Alt+1` / `Alt+2` / `Alt+3` conservan aquí su significado. Mientras la caja tiene el teclado, todas las teclas son de la
caja, salvo `Alt+1` / `Alt+2` / `Alt+3`; `Esc` devuelve el teclado a la línea de tiempo y un segundo `Esc` cierra la revisión.

Un comentario escrito en otra ventana o con la CLI aparece aquí por sí solo: cada escritura de GitPoulpi (esta ventana
incluida) avisa a la sesión, y las ventanas que muestran la misma solicitud lo leen de nuevo.

La caja de comentario es el editor de Markdown compartido con [la vista de conversación](conversations.md#escribir)
(también la de una respuesta a un hilo, la de un comentario sobre una línea y la del resumen de **Terminar revisión**):
`Ctrl+Enter` (también `Ctrl+S`, `Alt+Enter`) envía, y donde la IA está activada `Alt+E` abre **Escribir con IA**
([el capítulo de IA](ai.md#ayuda-para-escribir-en-un-host)). La vista previa muestra el texto como lo mostrará el host,
referencias incluidas (`#7`, `!12`, `ABC-12`, ids de commit); un texto sin enviar se conserva por solicitud.

### Archivos

Los archivos cambiados a la izquierda (letra de estado, `+añadidas -eliminadas`, un recuento `●n` de hilos abiertos, un recuento `✎n` de
tus comentarios pendientes) y el diff del archivo seleccionado a la derecha. El diff viene de git: el head de la solicitud
se trae a `refs/gitpoulpi/pr/<n>` (nunca una rama, sin checkout de nada) y se compara con la
base del merge, así que los números de línea del margen son exactamente los que usan los comentarios.

- **`[ ]` antes de un archivo**: márcalo como *visto* (clic, o pulsa **`v`** / `Space` en la lista). El título los cuenta.
  Las marcas de visto se guardan por repositorio y sobreviven a un reinicio.
- Los **hilos** están justo bajo su línea: primer comentario, respuestas, y los botones **Responder** y
  **Resolver** / **Reabrir**. Un hilo resuelto se pliega en una fila; **`z`** (o `Enter` sobre él, o un clic en la
  `▾`) pliega y despliega cualquier hilo.
- Los **hilos obsoletos**, y los hilos sobre líneas que el diff no muestra, se listan tras los hunks bajo
  *Comentarios fuera de las líneas mostradas*, con una insignia `obsoleto` cuando el código cambió bajo ellos.
- **`c`** comenta la línea donde está el cursor (el pequeño `+` al comienzo de la línea hace lo mismo con el
  ratón). **`Shift+Up` / `Shift+Down`** (o un arrastre sobre las líneas) seleccionan antes varias líneas y el
  comentario abarca el rango. GitLab admite una línea por comentario, y la vista lo dice en lugar de seleccionar.
- **`r`** responde al hilo bajo el cursor, **`x`** lo resuelve o lo reabre. Ambas acciones se envían al host **al
  instante**.
- Teclas: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` mueven; `Tab`, `Right` y `Enter` pasan de la lista de archivos
  al diff, `Left` y `Tab` vuelven; `]` y `[` saltan al archivo siguiente y al anterior.

### Opciones de visualización del diff

El panel de diff de **Archivos** y **commits** es el de la [vista Diff](staging-and-committing.md), con la misma barra en su primera
fila y las mismas elecciones recordadas (son un único ajuste: lo que eliges aquí es con lo que se abre la vista Diff,
y al revés):

- **En línea / Dividida**: una columna, o antes y después lado a lado. En Dividida un hilo, un comentario pendiente o una
  sugerencia está bajo la mitad a la que pertenece: un comentario sobre una línea eliminada bajo la mitad izquierda, un comentario sobre una
  línea añadida o sin cambios bajo la mitad derecha. Haz clic en la mitad que quieres antes de pulsar **`c`** en una fila que
  muestra una eliminación y una adición; el teclado comenta en el lado nuevo.
- **Hunks**: solo los cambios con su contexto, o el archivo entero con los cambios en su sitio. En el archivo entero un
  hilo sobre una línea fuera de los cambios queda bajo su línea en lugar de en *Comentarios fuera de las líneas mostradas*.
  El texto del archivo se lee en segundo plano; los hunks siguen en pantalla con una línea que lo dice hasta que llega (en
  la página commits, se leen así hasta 40 archivos de un commit).
- **WS**: mostrar o ignorar los cambios que solo difieren en espacios en blanco.
- **Wrap**: ajustar las líneas largas.
- **Flechas arriba / abajo**: el hunk anterior y el siguiente.
- **Map** (**`M`**): el minimapa junto a la barra de desplazamiento, un único ajuste con el de la vista Diff. Haz clic o arrastra sobre él para
  desplazarte.
- **Auto** (**`E`**, o *Revisión: Elegir codificación...*): la codificación con la que se lee el archivo, como en la vista Diff:
  Detección automática o una de las codificaciones de la lista, elegida para el archivo que muestra el panel (en la página commits, el
  archivo bajo el cursor). Los dos lados del archivo se leen de nuevo con esa codificación, en segundo plano, y
  la elección se comparte con la vista Diff para la misma ruta. Un archivo que no se puede leer así dice por qué sobre
  su diff.
- **`⋯`**: cuando el panel es demasiado estrecho para todos los botones, los que no caben quedan tras un `⋯` al final de
  la barra; su lista ejecuta cada uno como si se hubiera hecho clic. Sus teclas y filas de la paleta funcionan como siempre.

La regla de la barra de desplazamiento muestra dónde está cada cosa: los cambios (solo archivo entero, como en la vista Diff), un punto `●`
por cada hilo, comentario pendiente y sugerencia, y el cursor.

Las líneas largas se cortan en el borde del panel (activa **Wrap** para verlas enteras) y se desplazan lateralmente: **`Right`**
y **`Left`** en el diff (`Left` en el borde izquierdo vuelve a la lista de archivos), la rueda lateral o `Shift`
con la rueda, la barra de desplazamiento a lo largo del borde inferior, o *Revisión: Desplazar el diff a la izquierda / derecha*.

### Seleccionar texto y líneas

Dónde empieza un arrastre decide lo que selecciona. Un arrastre que empieza **en el código** selecciona texto, como en la vista Diff
(el doble clic selecciona una palabra); **`Ctrl+C`** (o *Revisión: Copiar el texto seleccionado o la línea*) copia exactamente esos
caracteres, sin números de línea ni marcadores, y sin selección de texto copia la línea bajo el cursor. Un
arrastre que empieza **en el margen** (los números de línea, el marcador, el `+`) selecciona líneas, para comentarlas
como antes, y un doble clic en el margen sigue iniciando un comentario en esa línea.

Las teclas `n`, `p`, `m`, `w`, `s` de la vista Diff son las propias de la revisión, así que en la revisión los alternadores son letras
mayúsculas: **`V`** disposición, **`H`** hunks o archivo entero, **`S`** espacios en blanco, **`W`** ajuste de línea, **`J`** / **`K`** hunk siguiente y
anterior, **`E`** la codificación, **`M`** el minimapa. Cada una tiene una fila en la paleta (*Revisión: Alternar vista en línea / dividida*,
...).

### commits

Solo los commits de la solicitud (no el historial que hay detrás), del más reciente al más antiguo, y el diff del seleccionado
(sus cambios respecto a su padre), con todos sus archivos uno bajo otro y las mismas opciones de visualización.

## Escribir una revisión

Un comentario sobre código queda **pendiente**: se guarda en tu máquina (en el archivo de estado del repositorio, así que sobrevive a un
reinicio) y se cuenta en la cabecera. No se envía nada hasta que terminas la revisión. Un comentario pendiente se muestra
bajo su línea con la insignia `Pendiente - línea 12` y dos botones: **`e`** (Editar) y **`d`** (Eliminar).

**Terminar revisión** (**`f`**, el botón, o la paleta) abre un pequeño formulario: un resumen (opcional) y una lista
**Veredicto**: *Comentar*, *Aprobar*, *Solicitar cambios*. Solo se listan los que permite el host: GitLab no tiene
*Solicitar cambios*. **Enviar revisión** (`Ctrl+S`) envía el resumen, el veredicto y todos los comentarios pendientes de
una vez. **Descartar revisión...** tira los comentarios pendientes tras una confirmación.

Lo que se envía al instante, sin esperar a la revisión: las respuestas, resolver y reabrir, y los comentarios sobre la
conversación. La caja de comentario dice de qué tipo es (*Este comentario se publica al instante* o *Este comentario se añade a tu
revisión pendiente*). Si el host rechaza una respuesta, la caja se abre de nuevo con tus palabras.

Si la solicitud recibió commits nuevos después de que escribieras comentarios pendientes, la vista avisa de que sus líneas pueden haberse
movido (GitHub y GitLab colocan un comentario sobre el commit que leíste; una revisión de un head más antiguo se rechaza).

## Gerrit

Un servidor de Gerrit se conecta como los demás hosts (ver [Alojamiento y gestores de issues](hosting-integration.md): URL del
servidor, nombre de usuario, contraseña HTTP) y se vincula bajo **PULL REQUESTS** a un *proyecto*. Sus cambios se listan como pull
requests (`#7`, chip `GR`) y se revisan en la misma vista, con las diferencias que trae Gerrit:

- **Patch sets.** Un cambio tiene patch sets numerados (cada subida del commit es uno). La cabecera tiene un
  combobox **`Patch set N`** en el borde superior, junto a las pestañas de página (**`P`**, o la fila de la paleta *Revisión: Elegir
  patch set...*). Empieza en el último; al elegir otro se leen los archivos y el código de ese patch set, y los
  hilos escritos sobre otros patch sets se listan como *obsoletos*. La revisión que termines se envía sobre el patch set
  que se muestra.
- **Comparar con otro patch set.** Un segundo combobox a su izquierda, **`Comparar: base`** (**`D`**, o *Revisión: Comparar
  patch sets...*), lista la base y cada patch set anterior al mostrado. Elige el patch set 1 mientras se muestra el 3
  y la página Archivos lista y muestra el diff solo de lo que el patch set 3 cambió desde el patch set 1 (la diferencia
  del propio servidor, de modo que un rebase sobre un destino más nuevo no aparece como cambios); la página Commits
  muestra esa única diferencia. Ambos patch sets se descargan en `refs/gitpoulpi/pr/<n>` y `refs/gitpoulpi/pr/<n>-ps<k>`.
  Los hilos escritos sobre otros patch sets distintos del mostrado están *obsoletos*. Elegir para leer un patch set
  anterior al comparado restablece la comparación con la base. El equivalente en la CLI es `review show|fetch --compare-with K`.
- **Integrar y abandonar.** Cuando el servidor dice que el cambio se puede integrar (están los votos y las comprobaciones
  que necesita), un botón **Integrar** se une a los botones de la cabecera; **Abandonar** está mientras puedas abandonar el cambio.
  Ambos (teclas **`U`** y **`Q`**, paleta *Revisión: Integrar el cambio...* / *Revisión: Abandonar el cambio...*) preguntan antes, en
  un popup que dice que la acción ocurre en el servidor y que **GitPoulpi no puede deshacerla** (un cambio integrado
  se queda integrado; uno abandonado se puede restaurar en la interfaz web de Gerrit). Rechazar no envía nada; si sale bien,
  la vista vuelve a leer el cambio. Los equivalentes en la CLI son `gitpoulpi hosting review submit-change <n>` y
  `hosting review abandon <n> [--message M]`.
- **Votos en lugar de un veredicto.** **Terminar revisión** muestra un combobox por cada etiqueta en la que puedes votar - `Code-Review`
  (de -2 a +2) y `Verified` donde el proyecto lo tiene - que lista solo los valores que el servidor permite **para este
  cambio y para ti** (un desarrollador sin el derecho no puede elegir +2). Empieza en tu voto actual; solo se envía un voto que
  hayas cambiado. El resumen y los comentarios pendientes van en la misma llamada. Los borradores que guardaste en la interfaz web de
  Gerrit se publican con la revisión.
- Los **hilos** son las cadenas de comentarios de Gerrit. Un hilo está resuelto cuando su comentario más reciente dice `unresolved: false`;
  **resolver** publica una respuesta `Done` que lo hace, **reabrir** una respuesta `Reopened`. Las respuestas y el resolver van al servidor
  al instante, como en los demás hosts.
- **Push para revisión.** Gerrit crea un cambio cuando haces push a `refs/for/<branch>`. Con la sección PULL REQUESTS
  vinculada a un proyecto de Gerrit, el botón **`+`** de la sección y la fila de la paleta **Alojamiento: Push para
  revisión...** piden la rama de destino y hacen push de `HEAD` a `refs/for/<branch>` (en la línea de comandos se puede dar un topic). El
  push no se puede deshacer desde GitPoulpi (abandona el cambio en Gerrit); el mensaje lo dice. Su gemelo en la CLI es
  `gitpoulpi hosting review push`.
- Los botones de metadatos son de solo lectura en Gerrit (se muestran los revisores), no hay veredicto *Solicitar cambios*
  (un voto de -1 o -2 es lo mismo) y el chip de checks es la etiqueta `Verified`.

## Borrador de revisión con IA

![Dos sugerencias ✦ bajo sus líneas en la página Archivos, cada una con Aceptar, Editar y Descartar, junto a un hilo existente](../images/review-ai.png)

Con la IA activada (*Ajustes > IA*, ver [IA](ai.md)) la cabecera tiene un botón **`✦ Redactar revisión...`** (también **`s`**,
y la fila de la paleta *Revisión: ✦ Redactar revisión...*). Abre las [opciones de la revisión](#opciones-de-la-revisión) y
**`Enter`** allí consulta al modelo. Es lo único que consulta al modelo: abrir una revisión
nunca lo hace. El modelo lee el título y la descripción de la solicitud, los comentarios que ya existen (para que
no los repita) y el diff, y responde con un **borrador de revisión**: el texto que enviar, el veredicto que recomienda (Aprobar, Solicitar cambios o Comentar), un resumen general y una lista de comentarios sugeridos. Un
diff largo se recorta para el modelo - por archivo y en total - y el Resumen dice cuántos archivos se acortaron
o se dejaron fuera. Mientras trabaja, la barra de estado muestra un indicador giratorio pasado un momento y **`Esc`** (o el mismo botón)
lo cancela. Se aplican las instrucciones de *Revisión de código* de los [cuatro niveles](ai.md#instrucciones), de modo que un equipo o un
espacio de trabajo puede decir cómo es una buena revisión.

### Opciones de la revisión

`s` abre el diálogo **✦ Opciones de la revisión**, rellenado con los valores por defecto del repositorio (los globales
donde el repositorio no tiene ninguno). `Enter` (o el botón **Ejecutar**) inicia la ejecución con lo que muestra el diálogo; `Esc`, **Cancelar**
o la cruz lo cierran sin consultar nada.

| Control | Qué hace |
|---|---|
| **Ejecutar el agente en el repositorio** | *Ajuste del perfil* (decide la carpeta de trabajo del perfil de IA), *Sí* o *No*. Atenuado, con el motivo debajo, para un perfil HTTP (no tiene carpeta de trabajo) y cuando la solicitud se revisa mediante la API sin clon. |
| **CLAUDE.md**, **AGENTS.md** | Dan al modelo ese archivo del repositorio, leído en la rama a la que apunta la solicitud. Cuando el agente se ejecuta en el repositorio los lee por su cuenta, y el diálogo lo dice. |
| **Archivo adicional** | Un archivo más: una ruta del repositorio, una ruta absoluta o un enlace a un archivo de un host de código. **Examinar...** (`Ctrl+B`, o `Enter` sobre el botón) abre un selector de archivos que empieza en la raíz del repositorio (o junto al archivo ya indicado); un archivo dentro del repositorio se escribe como ruta relativa a la raíz, uno fuera como ruta absoluta. Escribir en el campo funciona como antes. |
| **Ningún archivo adicional** | Ningún archivo adicional en esta ejecución, aunque el ajuste global o el repositorio indique uno; escribir un archivo o elegirlo con **Examinar...** la desmarca. Con **Guardar como valores por defecto de este repositorio** pasa a ser el interruptor *Ningún archivo adicional* del repositorio. Aquí un campo vacío cuenta como ninguno, porque el diálogo parte de lo que heredaría la ejecución. |
| **Guardar como valores por defecto de este repositorio** | También conserva los valores para las próximas revisiones: en el archivo de estado del repositorio, o en `remote-reviews.toml` para un repositorio que no está clonado. Solo se conserva un valor que difiere del global, así que el resto sigue a *Ajustes > IA*. |

Todo se alcanza con `Tab`, `Espacio` marca una casilla y el campo de ruta es un campo de texto corriente. Los
mismos cuatro ajustes viven en *Ajustes > IA* (*Revisión de código*) y en *Ajustes del repositorio: IA*; ver
[Contexto de la revisión](ai.md#contexto-de-la-revisión). Lo que no se pudo dar al modelo (un archivo adicional que falta o
es ilegible) y cualquier archivo que se acortó para respetar los límites aparecen en la tarjeta del borrador.

Las sugerencias **no son comentarios**. Se muestran en dos sitios:

- **Bajo su línea en Archivos**, como comentarios fantasma marcados con `✦` con una gravedad (`información`, `aviso`, `problema`) y
  tres botones: **Aceptar**, **Editar** y **Descartar**. Una sugerencia sobre una línea que el diff no muestra se
  conserva, nunca se pierde: se lista tras los hunks bajo *Sugerencias de la IA sobre todo el archivo* y, una vez
  aceptada, cae en la primera línea cambiada del archivo con las palabras *(sobre la línea N)* delante.
- **En el Resumen**, en un bloque sobre la conversación (solo después de una ejecución): el resumen, y después la lista con los
  mismos botones y **Aceptar todas** / **Descartar todas**.

**Aceptar** (`y`) convierte una sugerencia en un **comentario pendiente** corriente (aún puedes editarlo o eliminarlo);
**Editar** (`w`) abre la caja de comentario con las palabras del modelo primero y acepta lo que dejes; **Descartar** (`n`)
la tira. **Aceptar todas** (`A`) solo rellena la lista de pendientes. En la página Archivos las teclas actúan sobre la
sugerencia bajo el cursor, en el Resumen sobre la primera de la lista (`y` y `n` vuelven a ser *Copiar enlace* y *Siguiente sin leer* de la conversación
cuando la lista queda vacía). **Usar resumen de la IA** (`i`, o el botón
del formulario Terminar revisión) pone el resumen del modelo en el texto de la revisión; nunca se inserta solo.
**No se envía nada al host hasta que envías la revisión.**

El Resumen muestra el borrador como una tarjeta al principio de ese bloque, sobre las sugerencias: la etiqueta del veredicto, el texto de la revisión y tres
botones.

- **✦ Usar borrador...** (`u`) acepta cada sugerencia abierta como comentario pendiente y abre *Terminar revisión* ya
  rellenado con el texto y el veredicto del borrador. Tú lo lees, lo cambias y lo envías; cancelar el
  formulario conserva el borrador. Se respetan los límites del host: GitLab no tiene *Solicitar cambios*, así que el formulario lo dice y
  recurre a *Comentar*; en Gerrit el veredicto pasa a ser un voto Code-Review de +1 o -1, solo cuando el servidor
  permite ese valor para ti (si no, no se fija ningún voto y el formulario lo dice).
- **Editar texto...** (`t`) cambia el texto de la revisión del borrador antes de usarlo.
- **Descartar borrador** (`N`) tira el texto, el veredicto y las sugerencias restantes.

El borrador se conserva por solicitud, así que sigue ahí cuando cierras y vuelves a abrir la revisión. Cuando el
head de la solicitud se movió desde que el modelo lo escribió, la tarjeta dice *La solicitud ha cambiado desde que se escribió este borrador.*
Enviar la revisión gasta el borrador.

## Todo tiene una tecla y una fila en la paleta

| Acción | Tecla | Paleta |
|---|---|---|
| Resumen / Archivos / commits | `Alt+1` / `Alt+2` / `Alt+3` | *Revisión: Mostrar resumen*, *Mostrar archivos*, *Mostrar commits* |
| Comentar la línea o el rango | `c` | *Revisión: Comentar la línea...* |
| Comentar en la conversación | `c` en el Resumen, `C` | *Revisión: Comentar en la conversación...* (abre la caja del Resumen), *Conversación: Comentario nuevo* |
| Abrir el código de un hilo (Resumen) / volver al comentario | `G` / `Esc` | *Conversación: Abrir el código del hilo*, *Revisión: Volver al comentario en el resumen* |
| Responder al hilo | `r` | *Revisión: Responder al hilo...* |
| Resolver o reabrir | `x` | *Revisión: Resolver o reabrir el hilo* |
| Plegar o desplegar un hilo | `z` | *Revisión: Plegar o desplegar el hilo* |
| Editar / eliminar un comentario pendiente | `e` / `d` | *Revisión: Editar el comentario pendiente...*, *Eliminar el comentario pendiente* |
| Marcar el archivo como visto | `v` | *Revisión: Marcar el archivo como visto* |
| En línea / Dividida | `V` | *Revisión: Alternar vista en línea / dividida* |
| Archivo entero / solo los hunks | `H` | *Revisión: Alternar archivo entero / solo los hunks* |
| Cambios solo de espacios en blanco | `S` | *Revisión: Alternar cambios solo de espacios en blanco* |
| Ajuste de línea | `W` | *Revisión: Alternar ajuste de línea* |
| Hunk siguiente / anterior | `J` / `K` | *Revisión: Hunk siguiente*, *Hunk anterior* |
| Archivo siguiente / anterior | `]` / `[` | *Revisión: Archivo siguiente*, *Archivo anterior* |
| Terminar / descartar la revisión | `f` | *Revisión: Terminar la revisión...*, *Descartar la revisión pendiente...* |
| Actualizar | `F5`, `Ctrl+R` | *Revisión: Actualizar* |
| Abrir en el navegador / cambiar a la rama | `o` / `b` | *Revisión: Abrir en el navegador*, *Cambiar a la rama* |
| Asignados / revisores / etiquetas / hito | `a` / `p` / `l` / `m` | *Revisión: Editar asignados...* y sus tres hermanas |
| Elegir el patch set (Gerrit) | `P` | *Revisión: Elegir patch set...* |
| Comparar con un patch set anterior (Gerrit) | `D` | *Revisión: Comparar patch sets...* |
| Integrar / abandonar el cambio (Gerrit) | `U` / `Q` | *Revisión: Integrar el cambio...*, *Revisión: Abandonar el cambio...* |
| ✦ Redactar revisión... (IA activada; abre las opciones de la revisión) | `s` | *Revisión: ✦ Redactar revisión...* |
| Aceptar / editar y aceptar / descartar una sugerencia | `y` / `w` / `n` | *Revisión: Aceptar sugerencia*, *Editar y aceptar sugerencia...*, *Descartar sugerencia* |
| Aceptar / descartar todas las sugerencias | `A` / `N` | *Revisión: Aceptar todas las sugerencias*, *Descartar todas las sugerencias* |
| Usar el resumen de la IA en el texto de la revisión | `i` | *Revisión: Usar el resumen de la IA en el texto de la revisión* |
| Usar el borrador ✦ (rellena Terminar revisión) | `u` | *Revisión: ✦ Usar el borrador...* |
| Editar el texto del borrador | `t` | *Revisión: Editar el texto del borrador ✦...* |
| Descartar el borrador ✦ | `N` | *Revisión: Descartar el borrador ✦* |
| Cerrar | `Esc` | *Revisión: Cerrar* |

Abre la paleta (`Ctrl+P`) con la vista abierta: sus filas actúan sobre ella, y dicen por qué una no hace nada ahora
mismo («pon antes el cursor en una línea de código»).

## Revisar sin un clon

Una solicitud de un repositorio que no está en tu equipo también se puede revisar. Desde el Radar, **Revisar**
(`e`) en su fila pregunta cómo (ver [El Radar](radar.md#revisar-una-solicitud)); **Revisar mediante la API** abre
esta misma vista en la ventana de Inicio, sobre el Radar. Todo lo de esta página funciona igual, con estas
diferencias:

- El código (los archivos cambiados, el diff de cada archivo, los commits de la solicitud, el diff de un commit, el archivo entero) se
  lee de la API del host, así que la primera mirada a una solicitud grande tarda un momento, y un archivo para el que el host no
  envía diff (un binario o uno muy grande) lo dice en lugar de mostrar uno.
- Tus comentarios pendientes se guardan en `remote-reviews.toml` en la carpeta de configuración, archivados bajo el host
  y el proyecto (`host/project`), en lugar de en el estado propio de un repositorio. Siguen ahí cuando
  cierras y vuelves a abrir la revisión, y enviar la revisión los gasta.
- No hay elección de codificación (*No disponible sin un clon*) ni **Cambiar a la rama** (dice que clones primero el
  repositorio).
- `Esc` o la `✕` de la esquina sale de la vista y devuelve el Radar. La paleta (`Ctrl+P`) alcanza las
  filas de la vista como en la pestaña de un repositorio.

Para conservar el clon, elige **Clonar y luego revisar** (o la fila de la paleta *Radar: Clonar y revisar...*):
el diálogo Clonar se abre en la cuenta conectada de la solicitud con su repositorio seleccionado, y la revisión
se abre en la pestaña del nuevo repositorio cuando el clon ha terminado.

## Desde la línea de comandos y para agentes

La misma revisión está disponible como `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(y las herramientas MCP derivadas de ellos): ver [referencia de la CLI](cli-reference.md). `threads` también lista los
veredictos ya enviados (`reviews`). `suggest <n>` consulta al modelo (IA activada) e imprime su `verdict`, `body`, resumen y sus
sugerencias como JSON sin publicar nada, de modo que un script o un agente puede elegir los comentarios que pasar a
`submit --comment`.

Nada de esto necesita un clon: `--project owner/name` (con `--id` cuando hay varias integraciones configuradas)
ejecuta `show`, `threads`, `comment`, `reply`, `resolve`, `submit` y `suggest` contra un proyecto del que el directorio
actual no sabe nada. `hosting review diff <n>` imprime el diff con sus hunks (`--api` lo lee del host incluso
dentro de un clon), y `hosting file <path> --project P [--ref R]` lee un archivo en una revisión, por
ejemplo el `CLAUDE.md` del proyecto. Ambos leen la API del host: GitHub, GitLab, Gitea, Forgejo y Gerrit.
