[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · **Español** · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage y commit

Cómo ver lo que cambiaste, elegir exactamente qué entra en un commit, escribir el commit y
mirar los archivos con más detalle (blame, historial, el editor integrado). Términos como
*fila WIP* y *hunk* se explican en el [recorrido por la interfaz](interface-tour.md).

## Los cambios de trabajo

El trabajo sin commit es la **primera fila del Log** (círculo discontinuo, `// WIP`, con
contadores `✎ n` de modificados y `+ n` de añadidos). Selecciónala y el panel derecho muestra
los cambios de trabajo:

![Los cambios de trabajo: archivos Unstaged y Staged a la derecha, con el formulario de commit debajo](../images/hero.png)

- **Archivos unstaged**: archivos editados, añadidos (sin seguimiento) y eliminados que aún no
  están en el próximo commit.
- **Archivos staged**: lo que contendrá el próximo commit.
- **Los archivos renombrados o movidos** son una sola fila, `nuevo <- antiguo`, en ambas listas (un archivo borrado en el
  disco y uno nuevo con al menos un 50% del mismo contenido cuentan como un movimiento incluso antes de hacer stage). Stage,
  Unstage y Descartar actúan sobre ambos nombres: hacer stage añade el borrado y el archivo nuevo, de modo que git ve un solo
  renombrado. Abrir la fila muestra el diff del contenido contra el nombre antiguo, con el encabezado
  `antiguo -> nuevo (N% de similitud)`; un movimiento sin ediciones dice **Sin cambios de contenido**. Lo mismo vale para
  los archivos de un commit, y `gitpoulpi status` / `gitpoulpi diff` informan `renamed_from`.
- **Ruta** o **Árbol** (`m`) lista los archivos en plano o por carpetas. **Contraer todo**
  (`z`) pliega las carpetas. **Ver todos los archivos** (`a`) muestra el árbol completo en
  lugar de solo los archivos modificados. Haz clic derecho en la parte vacía de la lista para ver
  **Expandir todo** / **Contraer todo**, y en una fila de carpeta para ver **Expandir todas las subcarpetas** /
  **Contraer todas las subcarpetas**; teclas `=`, `-`, `*`, `_` (véase
  [Ratón y gestos](mouse-and-gestures.md)).
- El título dice cuántos archivos cambiaron en qué rama.

### Stage, unstage, descartar

| Para | Haz |
|---|---|
| Hacer stage de un archivo | Pasa el ratón por su fila y haz clic en **Stage**, o selecciónalo y pulsa `Ctrl+Space`, o `Ctrl+G s` |
| Hacer stage de una carpeta | **Stage carpeta** en la fila de la carpeta |
| Hacer stage de todo | **Stage todo** en el título de la sección, o `Ctrl+G a` |
| Unstage | Los mismos botones y teclas en la sección Staged (`Ctrl+G u`) |
| Descartar los cambios de un archivo | Clic derecho en el archivo > **Descartar cambios...** (pregunta antes; `Ctrl+Z` recupera los cambios) |

### Archivos sensibles

Hacer stage de un archivo, una carpeta o todo comprueba antes lo que añadiría. Un archivo
cuyo nombre coincide con `[privacy] sensitive_patterns` (`.env` y `.env.*` salvo los
`.example`, `.sample` y `.template`, `*.pem`, `*.key`, claves privadas SSH,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), o
cuyo texto contiene un secreto que reconoce el escáner, no pasa a stage. La pregunta
**Archivos sensibles** lista cada archivo con su motivo (un patrón, o el tipo de secreto
con una vista previa enmascarada, nunca el valor) y ofrece **1. Hacer stage de todos modos** o **2. Cancelar,
no hacer stage de nada**. Ninguna respuesta aparece resaltada hasta que pulsas una flecha o un dígito,
así que un Enter accidental no responde nada; `Esc`, la cruz de cierre y un clic fuera cancelan.
Un secreto hallado en un archivo nombra su línea (`config/app.env:12`). Para mirar antes de decidir, haz clic en un hallazgo
(o muévete entre ellos con `Tab` / `Shift+Tab`, `Ctrl+J` / `Ctrl+K`) y pulsa el botón **Mostrar en el archivo**, `S`, o
**Archivos sensibles: Mostrar en el archivo** de la paleta: el archivo se abre en la Vista de archivo desplazado a esa línea
y resaltándola, y la pregunta vuelve intacta al cerrar la Vista de archivo (`Esc`). Mirar no hace stage de nada.
Es la misma pregunta desde el teclado, un botón, un menú, la paleta o un
arrastrar y soltar, porque todos envían la misma petición. Cancelar no hace stage de nada,
ni siquiera de los archivos normales del mismo stage.

Se leen archivos de texto de hasta 1 MiB cada uno y 32 MiB en total; los archivos más grandes y
los binarios solo se juzgan por el nombre. `[privacy] staging_guard = false` desactiva la comprobación.
Hacer stage de un hunk o de algunas líneas no se comprueba: ya estás trabajando en ese archivo.

Haz clic derecho en un archivo para ver su menú: stage o unstage, descartar, copiar la ruta,
historial del archivo y blame, abrir en tu editor externo, herramienta de diff o gestor de
archivos (se configuran en [Ajustes > Diff y editor](settings.md#diff-y-editor)), y más.

## Ignorar archivos y tratarlos como binarios o como texto

Clic derecho en un archivo > **Ignorar** para ignorarlo o ignorar su extensión, o en una fila de carpeta > **Ignorar todos los archivos de la carpeta**. GitPoulpi pregunta primero en qué archivo va la regla: el `.gitignore` de la raíz del repositorio (por defecto), el `.gitignore` de cada carpeta hasta la del archivo, o `.git/info/exclude` (solo este clon, nunca confirmado). Los archivos que aún no existen se marcan como *nuevo* y se crean al confirmar. La línea se escribe relativa al archivo elegido: `src/a/b.txt` pasa a ser `/a/b.txt` en `src/.gitignore`. Muévete con las flechas, escribe para filtrar la lista, `Enter` escribe, `Esc` cancela. `Ctrl+Z` deshace una escritura en un `.gitignore`; `.git/info/exclude` está fuera del árbol de trabajo y no se puede deshacer.

![La lista que pregunta en qué archivo va una regla de ignorado: el .gitignore del repositorio, el .gitignore de la carpeta del archivo o .git/info/exclude, cada uno con la línea que escribiría](../images/ignore-picker.png)

Clic derecho en un archivo > **Tratar como binario** (o **Tratar como texto** si git ya lo trata como binario) escribe una regla de `.gitattributes`: `binary` o `text diff`. Elige el archivo o todos los archivos con su extensión, y luego el `.gitattributes` donde escribir, con la misma lista de arriba (y `.git/info/attributes` solo para este clon). Una regla ya escrita para el mismo patrón se reemplaza, nunca se contradice. El diff del archivo se actualiza al instante y la barra de estado dice qué se escribió y dónde (y avisa si otra regla sigue prevaleciendo). Una fila de carpeta tiene **Tratar los archivos de la carpeta como** > **Binario** o **Texto**. Lo mismo está en la paleta de comandos (**Cambios de trabajo: Tratar el archivo seleccionado como binario...**) y en la línea de comandos (`gitpoulpi treat binary|text <rutas>`). La vista de archivo también sigue la regla: un archivo tratado como texto muestra sus líneas aunque contenga un byte NUL, y un archivo que git trata como binario en el diff muestra el aviso de binario.

## Menús, selección múltiple y conversión en las listas de archivos

Las listas de archivos del panel derecho (Unstaged, Staged, los archivos modificados de un commit y **Ver todos los archivos**) comparten los mismos menús y gestos. Haz clic derecho en un archivo, en una fila de carpeta o en la parte vacía de una lista, o pulsa `.` (`Shift+F10`, la tecla Menú) en la fila con el foco, o ejecuta **Archivos: Abrir el menú de la fila o de la selección** desde la paleta.

- **Un archivo de un commit** (y un archivo de **Ver todos los archivos**): **Abrir diff**, **Abrir archivo**, **Historial del archivo**, **Blame del archivo**, abrir en tu editor, en el programa predeterminado o en **Abrir con...**, **Mostrar en la carpeta**, **Editar archivo**, **Copiar ruta del archivo**, **Copiar la ruta completa del archivo**, **Ignorar**, **Tratar como binario** o **Tratar como texto**, **Convertir**, **Restaurar el archivo desde este commit** (pregunta antes; como `git checkout <commit> -- <archivo>`, también hace stage del archivo, y `Ctrl+Z` devuelve el archivo anterior) y **Eliminar archivo...**. Una fila sobre la copia de trabajo aparece en gris, con su motivo, cuando el archivo no está en el directorio de trabajo. **Diff con el directorio de trabajo** abre el diff de la versión del commit contra el archivo en disco. **Crear parche con los cambios del archivo...** escribe un parche del cambio de ese archivo en el commit (una selección y una fila de carpeta también lo tienen; en **Ver todos los archivos** sobre los cambios de trabajo, crea el parche de los cambios del archivo en el árbol de trabajo).
- **Una fila de carpeta**: **Abrir carpeta**, **Copiar ruta del archivo**, **Ignorar todos los archivos de la carpeta**, **Tratar los archivos de la carpeta como**, **Convertir**, **Crear un archivo en esta carpeta**.
- **La parte vacía de una lista**: **Crear archivo** y **Abrir la carpeta del repositorio**. La lista Unstaged añade **Stage todo** y **Descartar todos los cambios...**, la lista Staged **Unstage todo**.

![El menú contextual de un archivo de un commit: abrir el diff, historial y blame del archivo, abrir en un editor o en el programa predeterminado, copiar rutas, ignorar, convertir, restaurar desde el commit y eliminar](../images/commit-file-menu.png)

### Seleccionar varios archivos

Los gestos son los del Log. **Ctrl+clic** añade una fila a la selección o la quita. **Shift+clic** (**Alt+clic** en los terminales que se quedan con Shift+clic) toma el rango desde el ancla. Un clic simple empieza de nuevo y abre el diff como antes. `Esc` borra la selección. Con el teclado, `Space` o `Insert` alterna la fila actual en todas las listas (en Unstaged y Staged `Ctrl+Space` hace stage de la selección, o de la fila actual) y `Shift+Up` / `Shift+Down` amplían el rango. Una fila de carpeta selecciona todos los archivos que contiene, plegada o no. Las filas seleccionadas se resaltan y la barra de estado las cuenta. `Ctrl+A` selecciona todas las filas de la lista activa.

Haz clic derecho en una fila seleccionada (o pulsa `.`) para el menú de toda la selección, titulado **N archivos**: stage o unstage, descartar, ignorar, tratar como binario o texto, convertir, copiar las rutas, restaurar desde el commit, eliminar. Stage, unstage y restaurar son una sola acción cada uno; descartar y eliminar preguntan antes e indican cuántos archivos. Un clic derecho en una fila fuera de la selección empieza la selección de nuevo con esa fila.

![Cuatro archivos seleccionados en la lista de cambios sin preparar, con el menú titulado 4 archivos: preparar, descartar, ignorar, tratar como binario o texto, convertir, copiar las rutas y eliminar](../images/file-pane-multiselect.png)

### Convertir la codificación, los finales de línea y la sangría

**Convertir** está en el menú de un archivo, una carpeta o una selección. `gitpoulpi convert` hace lo mismo desde la línea de comandos.

- **Codificación**: UTF-8, UTF-8 BOM, UTF-16 LE y BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Fines de línea**: LF, CRLF o CR.
- **Sangría**: espacios o tabulaciones al principio de la línea, con el ancho de una tabulación.

Los archivos del árbol de trabajo se reescriben en segundo plano con las mismas funciones que la barra de estado del editor. Un archivo que no es UTF-8 válido y no tiene marca de orden de bytes se lee como Windows-1252, como en el editor. Los archivos binarios, ausentes o que la codificación de destino no puede representar se omiten y se informa de ello (*12 archivos convertidos, 3 omitido(s) (3 binario(s))*), nunca se dañan; un archivo que no se puede escribir provoca una notificación de error. Todo el lote es una sola entrada del diario de deshacer: un `Ctrl+Z` devuelve todos los archivos. Un archivo abierto en el editor bloquea la conversión hasta que se cierre.

## Leer un diff

Haz clic en el nombre de un archivo (o pulsa `d` sobre él) y su diff sustituye al Log en el
centro. `Esc` o `q` devuelve el Log.

![Un diff con dos hunks, cada uno con los botones Descartar hunk y Stage hunk](../images/diff.png)

La barra de encima cambia la vista:

- **En línea** muestra el diff unificado; **Dividida** pone el antes y el después en paralelo
  (`m` recorre las disposiciones).
- **Hunks** (`Shift+H`) muestra solo los hunks modificados con una línea de contexto.
  Desactívalo para ver el archivo entero.
- **EB** (`s`) oculta los cambios que solo difieren en espacios en blanco.
- **Ajuste** (`w`) parte las líneas largas en lugar de truncarlas.
- **Mapa** (`Shift+M`) muestra un minimapa de todo el archivo junto a la barra de
  desplazamiento. Haz clic en él para saltar. Las demás apariciones de una palabra
  seleccionada se tiñen y se marcan en la barra de desplazamiento.
- `n` y `p` (o las flechas de la barra) saltan entre hunks.
- **Auto** (el último botón, `Shift+C`) es la codificación con la que se lee el archivo. Haz clic en él (o pulsa
  `Shift+C`, o ejecuta *Diff: Elegir codificación...* desde la paleta) y elige una de la lista:
  primero **Detección automática**, después UTF-8, UTF-8 BOM, UTF-16 LE y BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 y EUC-KR (escribe para
  buscar en la lista). Los dos lados del archivo se leen de nuevo, como bytes exactos, en esa codificación, de modo que un
  archivo Latin-1 muestra sus tildes y un archivo UTF-16 (que git considera binario) muestra sus líneas. La
  elección se recuerda para la ruta de ese archivo hasta que salgas, en esta vista y en la
  [revisión](review.md). Mientras hay una codificación elegida, los hunks son los propios de esta vista, así que preparar
  (stage) y descartar están desactivados (elige **Detección automática** para volver a hacer stage). Una lectura lenta dice *leyendo como ...* en el
  título pasado un momento.

![El mismo diff en la disposición doble, el antes y el después en paralelo](../images/diff-split.png)

Puedes seleccionar texto con el ratón (arrastrando, o con doble clic en una palabra) y copiarlo
con `y` o `Ctrl+C`; solo se copia el texto del archivo, nunca los números de línea ni los
marcadores `+` y `-`.

### Hacer stage de un hunk o de una sola línea

En un diff del árbol de trabajo, la cabecera de cada hunk tiene **Stage hunk** y **Descartar
hunk** (en el lado Staged, **Unstage hunk**).

- `Space` hace stage del hunk bajo el cursor.
- Para hacer stage de unas pocas líneas, pulsa `v` para iniciar una selección, muévete con
  `j`/`k` y pulsa `Enter`. Solo cuentan las líneas *modificadas*.
- `x` descarta el hunk y `Shift+X` las líneas seleccionadas. Ambos preguntan antes, y `Ctrl+Z`
  recupera los cambios.
- Clic derecho (o `.`) en un hunk para ver su menú: stage, unstage, descartar, historial del
  archivo, blame, copiar.

El diff de un commit es de solo lectura. Una imagen modificada se muestra como imagen; consulta
[Imágenes](image-view.md).

## Escribir el commit

El formulario de commit está al pie del panel derecho, y su **Resumen** es también el campo de
la fila de cambios de trabajo del Log: lo que escribas en uno aparece en el otro.

![El formulario de commit rellenado con un mensaje escrito por la IA: resumen con contador de caracteres, descripción y el botón verde de commit](../images/ai-commit-message.png)

- **Resumen** (el contador indica cuántos de 72 caracteres quedan) y **Descripción**. `Enter`
  en el resumen pasa a la descripción; `Enter` en ella añade una línea.
- **Commit de cambios en N archivos** (el botón verde) o `Ctrl+Enter` hace commit de los
  cambios en stage. Si no hay nada en stage, el botón lo dice y no hace nada.
- **Modificar el commit anterior** (`Shift+A`) incorpora los cambios en stage y el mensaje
  nuevo al último commit. Un commit que ya se envió necesitará entonces un force push (con
  lease).
- **Omitir hooks** (`Shift+H`, o **Commit: Alternar omitir hooks** en la paleta) hace commit
  sin ejecutar los hooks `pre-commit` y `commit-msg` del repositorio. Vale para un solo
  commit; **Ajustes > Ajustes del repositorio > Commit** hace que empiece marcada. Se muestra
  cuando la fila tiene sitio.
- `c` en el panel Estado lleva el cursor al formulario. `Esc` sale de él.
- `Ctrl+G c` y `Ctrl+G Enter` (**Git: Commit...** y **Git: Hacer commit ya de los cambios en stage**
  en la paleta) son un solo comando, el botón de commit desde cualquier sitio. Con un resumen
  en el formulario y algo en stage, hace commit enseguida, respetando **Modificar el commit
  anterior** y **Omitir hooks**, sin mover el foco. Si no, no hace commit de nada: lleva el
  cursor al resumen del formulario (mostrando el panel si estaba oculto o plegado) y la línea
  de estado dice qué atajo hace commit (`Ctrl+Enter` por defecto). Si no hay nada en stage, lo
  dice. No se escribe nada en la línea de estado. Mientras la IA aún está escribiendo un
  mensaje, no hace commit de nada y lo dice (`Esc` cancela la petición). `Ctrl+Z` deshace el
  commit como cualquier otro.
- Por repositorio, los [Ajustes](settings.md#ajustes-del-repositorio-commit) pueden **hacer push
  tras cada commit**, empezar el formulario con una **plantilla de commit** (y quitar sus
  comentarios `#` del mensaje) y hacer que un merge sea **squash**: el formulario se abre
  entonces con el mensaje de squash de git.
- Con una integración de alojamiento vinculada, `Alt+R` en el formulario inserta una referencia a
  un issue; consulta [Alojamiento y gestores de issues](hosting-integration.md).

Con la [IA](ai.md) activada, el botón **`✦`** a la derecha del resumen escribe un mensaje a
partir de los cambios en stage (`Ctrl+G i`), y **Componer commits con IA** divide tu trabajo
en varios commits, como se explica más abajo. El texto se pone en el formulario para que lo
edites; no se hace commit de nada hasta que pulses el botón de commit.

**Componer commits con IA** está en el extremo derecho de la fila de Modificar y se encoge con el panel derecho en lugar de
desaparecer: la etiqueta completa, después **Componer con IA**, después solo el **`✦`**. Conserva su clic, su tooltip y su
tecla (`Ctrl+G Shift+C`) a cualquier ancho; las etiquetas de Modificar y de los hooks de Omitir ceden ante él, nunca al revés.

## El compositor de commits: divide tus cambios en commits

*(Requiere la IA activada: consulta [Asistencia de IA](ai.md).)* Pulsa **Componer commits con
IA** (o `Ctrl+G C`, o haz clic derecho en la fila de cambios de trabajo). El modelo propone
varios commits, cada uno con sus archivos; cada archivo modificado está en exactamente un
commit, y un plan no válido se devuelve al modelo una vez. La propuesta se abre como una
revisión en la que aún no se ha hecho commit de nada:

![El compositor de commits: cuatro commits propuestos, cada uno desplegable hasta sus archivos, con controles de subir, bajar, editar y descartar y los botones Restablecer, Cancelar y Crear 4 commits](../images/composer.png)

| Para | Haz |
|---|---|
| Expandir o contraer un commit | `Left` / `Right`, o clic en el chevron; `*` alterna todos |
| Subir o bajar un commit | Los `▲` `▼` de su fila, o `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Mover un archivo a otro commit | Arrástralo sobre el commit (desde la lista, o desde la lista de cambios de trabajo de la derecha); o `m` sobre el archivo y elige el commit en el desplegable; o `Alt+Up` / `Alt+Down` sobre el archivo |
| Poner un archivo en un commit propio | `n` sobre el archivo, o el botón `+ commit propio` |
| Reformular un commit | El lápiz, `e`, `Enter` o un doble clic: edita el mensaje, `Enter` lo conserva, `Esc` lo descarta |
| Dejar un commit fuera | `drop` (`d` o `Delete`); sus archivos quedan sin commit. `restaurar` lo recupera |
| Empezar de nuevo | **Restablecer** (`r`) vuelve al plan del modelo |
| Crearlos | **Crear N commits** (`Ctrl+S`, `Ctrl+Enter` o `a`) |
| Salir | **Cancelar** o `Esc`: no se hace nada |

Crear los commits pone todo en unstage y después, para cada commit, hace stage de sus archivos
y commit, de modo que tu árbol de trabajo nunca se toca. Si un paso falla, se detiene y dice
cuántos commits se hicieron. Se hace stage de archivos enteros: un archivo que estaba en
stage solo en parte se confirma entero. Elige qué perfil e instrucciones usa el compositor en
*Ajustes > IA*; consulta [Asistencia de IA](ai.md#el-compositor-de-commits).

## Vista de archivo, Blame e Historial del archivo

La barra de vistas tiene **Ver archivo** (el contenido del archivo), **Diff**, **Blame** y el
selector `Archivo │ Todos` de la [columna de commits](interface-tour.md#la-columna-de-commits)
(su lista `Archivo` es el Historial del archivo); las mismas están en el menú de clic derecho de un archivo. Muestran el archivo tal
como estaba en el commit del que vienes, o sin commit (`@ WIP`) cuando lo abriste desde los
cambios de trabajo.

- **Blame** y el **Historial del archivo** son vistas completas por derecho propio: consulta [Blame](log-and-history.md#blame) e [Historial del archivo](log-and-history.md#historial-del-archivo) en *Trabajar con el Log*. `Enter` sobre un hunk del blame o una fila del historial va a ese commit en el Log.
- `y` o `Ctrl+C` copia el texto seleccionado (en Blame, solo el código).
- `Shift+G` salta a la última línea. `q` cierra la vista.

## El editor integrado

**Editar** (`e`) abre la *copia del directorio de trabajo* del archivo en un pequeño editor
sobre el panel central: colores de sintaxis, números de línea, deshacer y rehacer, búsqueda y
una barra de estado (posición, final de línea, sangría, lenguaje). Siempre edita la copia de
trabajo, incluso cuando lo abriste desde el diff de un commit antiguo.

![El editor mostrando un archivo Rust con resaltado de sintaxis y números de línea; la barra de herramientas tiene Guardar, Deshacer, Rehacer y Buscar](../images/editor.png)

| Tecla | Acción |
|---|---|
| `Ctrl+S` | Guardar (el archivo se escribe en un archivo temporal y se renombra, así que nunca queda a medio escribir; se conservan los finales de línea, el BOM y el salto de línea final) |
| `Ctrl+Z`, `Ctrl+Y` | Deshacer, rehacer |
| `Ctrl+F`, `F3` | Buscar, buscar siguiente |
| `Tab`, `Shift+Tab` | Aumentar la sangría, reducirla (según el estilo del propio archivo) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Seleccionar todo, copiar, cortar, pegar |
| `Esc` | Cerrar. Con cambios sin guardar pregunta **Guardar** (`s`), **Descartar** (`d`) o **Cancelar** (`c`) |

Los archivos binarios se rechazan. El tamaño de tabulación, los
finales de línea, el resaltado de sintaxis, los números de línea y el ajuste de línea están en
*Ajustes > Diff y editor*; consulta [Ajustes](settings.md#diff-y-editor).

Con **Seguir la selección** marcado (la casilla de la barra del editor, también *Editor: Alternar seguir la selección* en la paleta y *Ajustes > Diff y editor*), seleccionar otro archivo en el panel derecho con el editor abierto abre la copia de trabajo de ese archivo en el editor, en lugar de su diff o su vista de archivo. Si hay cambios sin guardar, antes pregunta **Guardar**, **Descartar** o **Cancelar**; Cancelar te deja en el archivo actual. Un archivo que no se puede editar (binario, demasiado grande, eliminado) se abre en su vista habitual y la barra de estado dice por qué. Al cerrar el editor llegas a la vista del archivo seleccionado. Está marcado por defecto. Esto vale para los cambios de trabajo: elegir un archivo de un commit o de una comparación con el editor abierto sale del editor (antes pregunta si se guardan los cambios pendientes: **Guardar**, **Descartar** o **Cancelar**) y muestra el diff de ese archivo en ese commit, y la columna de commits, que el editor no tiene, vuelve con él.

La barra de estado, a la derecha, muestra la **codificación**, los **finales de línea** y la
**sangría** del archivo, y después la posición del cursor y la sintaxis. Los tres primeros son
botones: cada una abre un menú desplegable (flechas o rueda, escribe para filtrar, `Enter`
para elegir, `Esc` para cerrar). La paleta (`Ctrl+P`) abre los mismos menús: *Editor: Cambiar codificación*, *Editor: Reabrir con codificación*, *Editor: Convertir a codificación*, *Editor: Convertir saltos de línea*, *Editor: Cambiar sangría*.

![El menú de codificación del editor abierto bajo el chip UTF-8 de la barra: reabrir el archivo con otra codificación o convertirlo](../images/editor-encoding-menu.png)

![El menú de fin de línea del editor abierto bajo el chip LF: convertir el archivo a LF, CRLF o CR](../images/editor-eol-menu.png)

- **Codificación.** Un archivo se abre en la codificación que anuncia (un BOM: UTF-8 o
  UTF-16), si no en UTF-8, si no en Windows-1252, y la barra dice cuál es. **Reabrir con
  codificación** vuelve a leer los bytes del archivo con otra codificación (pregunta
  antes si hay cambios sin guardar): úsalo cuando la detección se equivocó. **Convertir a
  codificación** conserva el texto y cambia la codificación en la que se guarda el archivo
  (UTF-8, UTF-8 BOM, UTF-16 LE y BE, Windows-1252, ISO-8859-1 e ISO-8859-15, Windows-1250,
  Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). No se escribe nada hasta
  que guardas, y Deshacer lo revierte. Si el texto tiene caracteres que el destino no
  admite, la barra de estado dice cuántos y dónde está el primero, y no se convierte
  nada; al guardar se rechaza igual, nunca escribe `?`.
- **Fines de línea.** Muestra `LF`, `CRLF`, `CR` o `Mixto` cuando el archivo mezcla
  varios. El menú convierte todos los saltos de línea del archivo a LF, CRLF o CR en un
  solo paso de deshacer. Las líneas nuevas que escribes usan el final del propio archivo.
  Tu elección prevalece sobre el ajuste *Fin de línea*.
- **Sangría.** Muestra `Espacios: 4` o `Tabulaciones: 4`. El menú elige qué insertan `Tab` y la
  sangría automática (espacios o tabulaciones), el tamaño (de 1 a 8: el paso de sangría y
  el ancho de una tabulación en este archivo), y convierte el espacio en blanco inicial de
  cada línea a espacios o a tabulaciones, en un solo paso de deshacer.

## Stash, parches y Git LFS

- **Stash** y **Pop** están en la barra de herramientas; consulta
  [Trabajar con el Log](log-and-history.md#ramas-tags-y-la-barra-lateral-refs).
- **Crear parche** y **Aplicar parche** están en los menús de commit y de refs; la CLI tiene
  `gitpoulpi patch`.
- Los archivos grandes guardados con Git LFS se muestran como pequeños archivos de puntero
  hasta que traes su contenido; consulta [Git LFS](lfs.md).

## Desde la línea de comandos

Todo lo de esta página se puede automatizar con scripts:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # tras el rechazo de abajo
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` rechaza un archivo sensible de la misma manera: no se hace stage de nada, el
código de salida es 2 y `--json` imprime `{error: "sensitive_files", files: [{path, reason}]}`
(una regla y una vista previa enmascarada, nunca un valor). `--allow-sensitive` es el reintento. A un
agente MCP se le rechaza con la misma respuesta y no puede pasar `allow_sensitive` salvo que
`[mcp] allow_sensitive_staging = true`: se le dice que te pregunte.

Consulta la [referencia de la CLI](cli-reference.md).
