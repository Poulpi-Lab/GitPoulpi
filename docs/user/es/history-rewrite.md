[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · **Español** · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Reescritura del historial

Reescribir el historial cambia commits que ya existen: quién los escribió y cuándo,
sus mensajes, los archivos que contienen, sus firmas. GitPoulpi lo hace con una
**receta** (un alcance y una lista ordenada de reglas) y tres pasos, de los que
nunca te saltas el del medio:

1. **Vista previa**: calcula el historial nuevo sin tocar nada. Da los recuentos, lo
   que hizo cada regla, las ramas y los tags que se moverían y una fila por cada
   commit modificado.
2. **Aplicar**: mueve a él tus ramas y tags locales. Es una sola entrada del
   [historial de deshacer](undo-redo.md): **Deshacer** devuelve cada ref movida (y los
   archivos activos) a como estaban.
3. **Push** (un force push con lease) y **Purga** son pasos aparte y deliberados,
   porque no se pueden deshacer: véanse [Enviar el resultado](#enviar-el-resultado)
   y [Eliminar el historial antiguo](#eliminar-el-historial-antiguo-purga).

El editor, la paleta, los comandos `gitpoulpi rewrite` y las herramientas MCP usan
el mismo motor, así que una vista previa hecha en uno es la que aplica otro.
[Desde la línea de comandos](#desde-la-línea-de-comandos) lista los flags;
[Crear reglas con IA](#crear-reglas-con-ia) rellena el editor a partir de una descripción;
[Propuestas de agentes de IA](#propuestas-de-agentes-de-ia) dice qué puede hacer
aquí un agente y qué no.

**Las reglas, según lo que cambian**

| Grupo | Reglas | Sección |
|---|---|---|
| Autores | Reemplazar un autor, Anonimizar autores, Aplicar .mailmap | [Autores y el .mailmap](#autores-y-el-mailmap) |
| Fechas | Desplazar fechas, Cambiar la zona horaria, Limitar a un horario, Fecha del committer = fecha del autor | [Fechas](#fechas) |
| Mensajes | Buscar y reemplazar, Eliminar líneas, Añadir o quitar un trailer, Corregir la codificación de mensajes | [Mensajes](#mensajes) |
| Contenido de archivos y rutas | Buscar y reemplazar en archivos, Quitar archivos, Quitar archivos grandes, Mover o renombrar rutas, Convertir fines de línea, Quitar el bit de ejecución, Convertir la codificación de archivos | [Archivos y rutas](#archivos-y-rutas) |
| LFS | Migrar archivos a LFS, Recuperar archivos de LFS | [Mover archivos a Git LFS o traerlos de vuelta](#mover-archivos-a-git-lfs-o-traerlos-de-vuelta) |
| Firmas | Quitar firmas (y volver a firmar al aplicar) | [Firmas](#firmas) |
| Script | Lua, shell, Python | [La regla Script](#la-regla-script) |

**Antes de empezar**

- Una vista previa no cambia nada. Aplicar cambia solo tus refs locales y nunca
  hace push.
- Los commits reescritos reciben ids nuevos. Hay que avisar a todos los que tengan el
  historial antiguo: [Enviar el resultado](#enviar-el-resultado) escribe la nota.
- Un secreto que ya se publicó se considera filtrado. **Rótalo primero**, y luego
  reescribe, haz un force push y purga.
- Los commits que quedan como están conservan sus ids y sus firmas.

## El editor de reescritura

`Ctrl+G Shift+H` (o **Reescritura: Editor del historial** en la paleta) abre el
editor de reescritura sobre el Log. No tiene botón en la barra de herramientas:
las acciones avanzadas viven en la paleta (cada fila `Reescritura: ...` lo abre
con esa regla ya añadida), en el submenú **Reescribir el historial** del menú del
commit (*Desde este commit hasta HEAD...*, *Commits seleccionados...* cuando hay
varios seleccionados, *Abrir el editor de reescritura...*) y tras el acorde.

`Ctrl+N` muestra una insignia en cada control del banco de trabajo (alcance, reglas, campos, pestañas, botones); pulsar una insignia hace lo que un clic en ese control, y `Esc` quita las insignias (consulta [Insignias](keybindings.md#insignias)).

![El banco de trabajo de reescritura del historial: la barra de alcance, una lista de dos reglas Reemplazar un autor con los campos de la regla seleccionada y el resumen de la vista previa con cuántos commits cambian](../images/history-rewrite.png)

- **Barra de alcance:** qué refs se mueven (la rama actual, todas las ramas
  locales, las ramas y tags elegidos, los commits seleccionados), la casilla
  *Tags* y, tras *Desde este commit hasta HEAD...*, una casilla para empezar
  después del padre de ese commit.
- **Reglas:** una lista numerada que se aplica por orden. La casilla activa o
  desactiva una regla, las flechas la mueven y la cruz la elimina (*Restaurar la
  regla eliminada* o `Ctrl+Z` la recupera). *Añadir regla* lista los tipos por
  grupos. Una regla que necesita un programa que no tienes aparece en gris con el
  motivo.
- **Ajustes de la regla:** un clic en un campo de texto lo edita; un clic en un
  campo con varios valores abre su lista; las casillas se alternan.
- **Vista previa** (`F5`) calcula el historial nuevo sin tocar nada y muestra una
  barra de progreso. Las pestañas: **Resumen** (recuentos, lo que hizo cada regla,
  avisos), **Commits** (una fila por commit modificado, paginada desde el daemon;
  el filtro lista los modificados, los eliminados o los ya publicados), **Ramas y
  tags** (qué se mueve y las ramas protegidas ya publicadas, cada una con su
  casilla), **Impacto** (qué supone reescribir historial ya publicado: un force
  push con lease, colaboradores, firmas), **Secretos**, **Identidades** y
  **Tamaño** (véanse más abajo).
- **Archivos de un commit:** `Enter` (o doble clic) en una fila de Commits abre
  los archivos que las reglas de contenido cambiaron en él y, a su lado, el diff
  del seleccionado. La cruz, `Esc` o un clic fuera lo cierran. Las líneas
  eliminadas por una regla de secretos se ocultan.
- **Aplicar** (`Ctrl+S` o `Ctrl+Enter`) está disponible cuando la vista previa
  coincide con las reglas. Las preguntas sin respuesta por defecto se hacen cada
  vez: qué pasa con las firmas de los commits firmados y cómo se extrae una
  carpeta. El panel de resultado ofrece **Deshacer**, que devuelve cada ref movida
  a como estaba.
- **Guardar / Cargar** conservan una receta en el repositorio (compartida) o solo
  para ti; los valores secretos nunca se guardan.

Cada botón tiene una tecla: la lista está en Ajustes > Teclado y ratón (contexto
*Editor de reescritura del historial*) y en la guía `keybindings.md`.

### Autores y el .mailmap

- **Reemplazar un autor** busca un autor o committer por nombre y/o correo (un
  texto literal o una expresión regular) y le pone un nombre y/o correo nuevos.
  *Aplicar a* elige el autor, el committer o ambos; *También los autores de tags*
  hace que los tags anotados que se recrean los sigan. El committer original se
  conserva salvo que la ejecución diga *yo, ahora*.
- **Anonimizar autores** reemplaza cada identidad por un marcador numerado
  (`Contributor 1`, `contributor-1@anonymous.invalid`; ambos patrones los puedes
  cambiar, con `{n}` para el número). En una misma ejecución, una persona recibe
  siempre el mismo número; las identidades que indiques en *Dejar sin cambios* se
  quedan como están.
- **La pestaña Identidades** (también **Reescritura: Merge de identidades
  (.mailmap)...** en la paleta) sirve para arreglar a una persona que hizo
  commits con varios nombres y correos **sin reescribir nada**. Lista todos los
  nombres y correos del alcance de la barra de alcance, con cuántos commits
  escribió y confirmó cada uno, y cuáles asigna ya el `.mailmap` del repositorio
  (`asignada a Nombre <correo>`). Marca las identidades que son la misma persona
  (`Space`, o un clic; *Marcar todas* / *Desmarcar todas*) y luego **Merge
  en...** (`m`): un diálogo pregunta cuál se queda (la que tiene más commits va
  primero) y **añade una línea por cada otra identidad al `.mailmap`** (`Nombre
  <correo> Nombre Antiguo <antiguo@correo>`; una línea que ya esté no se repite,
  y el archivo se crea si falta). No se hace commit de nada: haz tú el commit del
  `.mailmap` para compartirlo. El Log, el blame y el historial del archivo
  muestran al instante el nombre canónico (ajuste `log.use_mailmap`, activado
  por defecto; los commits guardados nunca cambian, así que no hay nada que
  enviar con un force push). La barra de estado dice qué se escribió y ofrece
  **Deshacer** (`u`), que devuelve el `.mailmap` anterior (es una entrada del
  historial de deshacer, como cualquier otra acción). `r` vuelve a leer la lista.
- **Un `.mailmap` en lugar de una reescritura.** Cuando una regla de autor
  cambia commits que ya están en un remoto, el Resumen dice *Un .mailmap corrige
  cómo se muestran sin reescribir el historial. ¿Usar .mailmap en su lugar?* con
  un botón **Usar .mailmap** (`m` en el Resumen). Abre la pestaña Identidades con
  las identidades que coinciden con la regla marcadas, y la identidad que la
  regla pondría ofrecida como la que se conserva aunque todavía ningún commit la
  lleve.
- **La regla `.mailmap`** (*Reescritura: Aplicar .mailmap al historial...*, o
  `rewrite preview --mailmap`) sí reescribe: los autores y committers (a tu
  elección) pasan a ser lo que el `.mailmap` del worktree les asigna, el
  resultado que muestra `git log --use-mailmap`. Úsala cuando el propio
  historial deba llevar los nombres canónicos, por ejemplo antes de publicarlo;
  es una reescritura como cualquier otra (vista previa, Aplicar, force push).
- **Reglas de identidad y trailers.** Una regla de identidad también puede cambiar
  a las personas que aparecen en los trailers: marca **Aplicar también a trailers**
  (`--map-trailers`). Todo trailer cuyo valor sea exactamente `Nombre <correo>` y
  coincida con *El correo es* / *El nombre es* de la regla pasa a tener el nombre y el
  correo nuevos, haga lo que haga la regla con el autor y el committer. Está
  desactivado por defecto: un `Signed-off-by` registra quién firmó, así que cambiarlo
  es una decisión aparte.

### Fechas

Las cuatro reglas de fechas actúan sobre la fecha del autor, la del committer o
ambas (*Fechas* en el panel de la regla).

- **Desplazar fechas** mueve todas las fechas una cantidad (`+2h`, `-1d`, `90m`,
  `1d 2h`). Se conserva la zona horaria en la que se escribió cada commit.
- **Cambiar la zona horaria** escribe las fechas con otro desfase UTC. *Conservar*
  dice qué se queda igual: el **instante** (el mismo momento, el reloj marca otra
  hora) o la **hora del reloj** (la misma hora local, cambia el instante).
- **Limitar a un horario** mueve cada commit que cae fuera de las horas permitidas
  (por ejemplo `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) a la hora permitida más
  cercana, o solo hacia atrás, o solo hacia delante (*Mover hacia*). *Nunca antes
  que su padre* conserva el orden del historial; la *Semilla* hace repetible el
  pequeño reparto aleatorio: la misma semilla sobre el mismo historial da las
  mismas fechas.
- **Fecha del committer = fecha del autor** pone en el committer (nombre, correo y
  fecha) los del autor, justo lo que deshacen un `git commit --amend` o un rebase.

Las fechas son reglas como las demás: vista previa, revisa la pestaña **Commits**
(cada fila muestra la fecha antigua y la nueva) y aplica. El id de un commit depende
de sus fechas, así que una regla de fechas cambia todos los commits posteriores al
primero que toca.

### Mensajes

- **Buscar y reemplazar en mensajes** y **Eliminar líneas de mensajes** aceptan un
  texto literal o una expresión regular (*Expresión regular*, *Distinguir
  mayúsculas*); un reemplazo vacío elimina. Un mensaje que no es UTF-8 válido nunca
  lo tocan: ejecuta antes **Corregir la codificación de mensajes** si quieres
  editarlo.
- **Trailers.** *Añadir o quitar un trailer* (*Reescritura: Añadir o quitar
  trailers...* en la paleta, o `--trailer-add`, `--trailer-remove`,
  `--trailer-map`) edita las líneas `Clave: valor` del final de un mensaje, las que
  conoce `git interpret-trailers` (`Signed-off-by`, `Co-authored-by`,
  `Reviewed-by`...). El desplegable **Acción** elige *Añadir* (la clave en
  **Trailer**, su **Valor** y **Si ya existe**: *Añadir (salvo si es idéntico)*,
  *Reemplazar los existentes* o *Dejar el commit como está*), *Quitar* (todos los
  trailers de esa clave, o solo aquellos cuyo valor contiene **El valor contiene**)
  o *Cambiar el valor* (**Buscar en el valor** pasa a **Reemplazar por**, por
  ejemplo un correo antiguo por uno nuevo en `Signed-off-by`). Las claves se
  comparan sin distinguir mayúsculas. Solo cambian las líneas de trailer; el resto
  del mensaje queda exactamente como estaba, y un mensaje que aún no tiene trailers
  recibe una línea en blanco y un bloque propio. La pestaña Commits muestra cada
  cambio como `+Signed-off-by: ...` / `-Co-authored-by: ...`. El último párrafo de
  un mensaje es su bloque de trailers (nunca el título), así que una línea como
  `Nota: véase el ticket` en el cuerpo no es un trailer, salvo que esté sola en el
  último párrafo junto con otros trailers.
- **Ids de commit en los mensajes.** **Actualizar ids de commit**
  (`--update-message-hashes`, desactivado por defecto; al final de los paneles de
  las reglas que editan mensajes) hace que los ids escritos en los mensajes sigan
  a la reescritura: un id abreviado de 7 o más caracteres hexadecimales
  (`This reverts commit 1a2b3c4`, `véase 1a2b3c4d5e`) que nombra exactamente un
  commit del historial reescrito se reemplaza por el id nuevo de ese commit,
  recortado a la misma longitud. Uno que coincide con varios commits, con ninguno o
  con un commit que la reescritura crea más tarde se deja como está; el Resumen dice
  cuántos se reemplazaron y cuántos se dejaron.
- **Corregir la codificación de mensajes** (*Reescritura: Corregir la codificación
  de mensajes...*, o `--fix-message-encoding [--fix-name-encoding]`) recodifica los
  mensajes a UTF-8, en dos casos:
  - Un commit con una cabecera `encoding` (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, cualquiera que conozcan git y las tablas de codificación) se
    decodifica y se escribe como UTF-8, y la cabecera se quita: el texto se lee
    igual en todas partes, sin conversión. Los nombres de autor y committer de ese
    commit estaban en la misma codificación y se decodifican con ella, así que un
    commit nunca queda convertido a medias.
  - El **mojibake**, UTF-8 que antes se leyó como Latin-1 o Windows-1252
    (`RÃ©Ã©criture` en lugar de `Réécriture`), se repara **solo cuando la
    conversión de ida y vuelta es exacta**: el texto debe volver a codificarse en
    bytes que sean UTF-8 válido, y estropear el resultado del mismo modo debe dar el
    original. Un mensaje que solo contiene `Ã©` junto a letras normales, o caracteres
    fuera de los alfabetos antiguos, no lo cumple y se queda como está.

  *Corregir también los nombres* (`--fix-name-encoding`) repara de la misma manera
  los nombres de autor y committer estropeados. Lo que no se puede decidir (un
  nombre de codificación desconocido, bytes que no son válidos en la codificación
  declarada, un mensaje que no es UTF-8 y no declara nada, texto que parece estropeado
  pero no se puede demostrar) **se deja como está y se cuenta**: el Resumen dice
  *n mensajes se dejaron como están: no se puede decidir su codificación*, y
  `counts.encoding_left` tiene el número. La pestaña Commits muestra cada corrección
  con el mensaje antiguo y el nuevo y una fila *codificación* (`ISO-8859-1` pasa a
  `UTF-8`).

### Archivos y rutas

- **Reglas de contenido de archivos** (*Buscar y reemplazar en archivos*, *Quitar
  archivos del historial*, *Quitar archivos grandes*): el panel comprueba cada
  valor mientras escribes y muestra el motivo bajo el campo (un patrón que no
  compila, un patrón de ruta incorrecto, un tamaño inferior a un byte). Los
  tamaños admiten unidades (`500 KiB`, `5 MiB`, `1 GiB`). Un patrón de ruta sin
  barra coincide a cualquier profundidad. Los *Valores secretos* se escriben en
  un campo enmascarado, se usan solo en esa ejecución y nunca se guardan; la vista
  previa oculta las líneas que los contenían. *Commits vacíos* indica si un commit
  que las reglas dejan vacío se quita (por defecto) o se conserva. Los archivos
  binarios y los que superan el límite de tamaño se dejan como están y se cuentan
  en el Resumen.
- **Mover rutas.** *Mover o renombrar rutas* (*Reescritura: Mover o renombrar
  rutas...* en la paleta) tiene tres variantes, que se eligen en el desplegable
  **Mover**:
  - *Renombrar una ruta* (**Desde**, **Hacia**; `--move-path FROM=TO`): los mismos
    archivos y carpetas aparecen en la ruta nueva en cada commit que tenía la
    antigua; nada más cambia, y cada archivo conserva su id. Un commit donde la ruta
    nueva ya existe nunca se sobrescribe: la vista previa se rechaza y lista los
    commits y las rutas que chocan (elige un nombre libre, o mueve antes la ruta
    existente). Solo rutas exactas, sin comodines.
  - *Mover todo a una carpeta* (**Carpeta**; `--into-subdir DIR`): todas las rutas
    de todos los commits reciben el prefijo. Útil para combinar repositorios sin
    choques de nombres.
  - *Extraer una carpeta como raíz* (**Carpeta**; `--extract DIR`): la carpeta pasa a
    ser la raíz, los commits que no la tocan se descartan (un commit que la crea no
    tiene padre), los merges que ya no unen nada se simplifican, y las ramas y los
    tags que no tienen nada de la carpeta se eliminan. Se te pregunta **cada vez**
    cómo termina:
    - *En un clon nuevo, abierto en una pestaña nueva* (`--extract-mode clone`): se
      hace una copia de este repositorio en la carpeta que elijas (el repositorio
      nuevo recibe dentro una carpeta con el nombre de la extraída), se reescribe
      allí y se limpia para que contenga solo el historial nuevo, con todas las
      ramas y los tags locales que tienen la carpeta. Este repositorio no se toca.
      El repositorio nuevo no tiene remoto (el original no es su upstream).
    - *En el sitio: se reescribe el historial de este repositorio* (`--extract-mode
      in-place`): la reescritura de siempre, con vista previa, Aplicar y deshacer.
      Las ramas sin la carpeta se eliminan (deshacer las recupera); la rama activa
      debe tenerla.
    - *Como submódulo de este repositorio* (`--extract-mode submodule`): el
      repositorio nuevo se crea como en el caso del clon y luego **un commit nuevo**
      en la rama actual reemplaza la carpeta por un submódulo suyo (`.gitmodules` y
      el enlace); la carpeta queda como un submódulo activo. El historial de este
      repositorio **no** se reescribe: sus commits antiguos conservan los archivos de
      la carpeta, así que nada que apunte a ellos (tags, otros clones) se rompe. La
      URL de `.gitmodules` es la ruta local del repositorio nuevo: cámbiala
      (`git config -f .gitmodules submodule.<carpeta>.url`) cuando viva en un host.
      Archivos editados o sin seguimiento en la carpeta lo rechazan. Deshacer
      devuelve la carpeta normal.

    El Resumen dice que la vista previa muestra el historial del repositorio nuevo y
    a qué carpeta va. Las reglas que vienen después de una extracción (un renombrado,
    una eliminación) se aplican al árbol extraído; para un clon o un submódulo, la
    extracción debe ser la primera regla de rutas. Las reglas de rutas se ejecutan
    después de las de contenido, así que un glob de eliminación nombra las rutas tal
    como eran.

#### Fines de línea, bit de ejecución y codificación de archivos

Tres reglas del grupo **Contenido** del menú Añadir (o las filas de la paleta
*Reescritura: Convertir fines de línea...*, *Quitar el bit de ejecución...* y
*Reescritura: Convertir la codificación de archivos...*). Las tres aceptan
**Archivos** (globs; un patrón sin barra coincide a cualquier profundidad; vacío
significa todos los archivos) y **Salvo estos archivos**. Cambian solo lo que nombran
los globs, en cada commit del alcance. Los archivos binarios (un byte NUL en los
primeros 8 KiB) nunca se tocan: el Resumen los cuenta como *dejados como están*.

- **Convertir fines de línea** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Convertir a*: LF (Linux, macOS), CRLF (Windows) o CR
  (Mac clásico). Todos los fines de línea de un archivo de texto que coincide pasan a
  ser ese, así que un archivo con fines mezclados queda uniforme; los archivos que ya
  están bien conservan su versión intacta. *Seguir .gitattributes* (desactivado por
  defecto, `--eol-follow-attributes`) lee el `.gitattributes` de la raíz de cada
  commit: los archivos marcados `-text` o `binary` se omiten y los marcados
  `eol=lf` o `eol=crlf` conservan ese fin de línea en lugar del que elegiste.
- **Quitar el bit de ejecución** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Los archivos que coinciden pasan del modo 100755 al 100644; su
  contenido no se toca. Hace falta al menos un patrón de archivos: quitárselo a todos
  los ejecutables rompería tus scripts.
- **Convertir la codificación de archivos** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Convertir a*: UTF-8 (por defecto), UTF-8 con BOM, UTF-16 LE o BE
  (escritos con marca de orden de bytes), Windows-1252 o ISO-8859-1. *Los archivos
  están en*: por defecto se detecta la codificación de cada archivo (una marca de
  orden de bytes, si no UTF-8 válido, si no la *Alternativa*, Windows-1252 salvo que
  la cambies); nombra la codificación para forzarla, pero una elección equivocada da
  un texto equivocado (UTF-8 leído como Latin-1 duplica cada acento). Un archivo que
  la codificación de origen no puede leer, o que contiene un carácter que el destino
  no puede guardar (un signo de euro en ISO-8859-1), se queda exactamente como está y
  se cuenta en el Resumen.

Las reglas se ejecutan en el orden de la lista, sobre el mismo archivo: convertir un
archivo a UTF-16 y pedir después fines de línea lo deja como está (ya no es texto
plano). Aplicar es una entrada de la lista de deshacer, como cualquier reescritura.

### Buscar secretos

- **Buscar secretos:** `gitpoulpi rewrite scan-secrets` (y la herramienta MCP
  `rewrite_scan_secrets`) lee el historial una vez y lista prefijos de token
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), claves
  privadas, JWT, contraseñas en URL y líneas `password = value` de aspecto
  aleatorio, cada una con su archivo, el primer y el último commit y una vista
  censurada como `ghp_****abcd`. No es exhaustivo, y el valor nunca se muestra,
  se registra ni se guarda. `--remove-finding ID` convierte un hallazgo en una
  regla que lo quita de todos los commits. Un secreto que ya se publicó se
  considera filtrado: rótalo primero y luego reescribe, haz un force push y
  purga.
- **La pestaña Secretos** hace lo mismo desde el editor, para el alcance de la
  barra de alcance: **Buscar secretos** (`s`) lanza el análisis como una tarea
  (la barra de progreso dice *Analizando*; **Cancelar** o `Esc` lo detiene). Cada
  hallazgo es una fila con su tipo, la vista censurada, el archivo y cuántos más
  lo contienen; bajo la lista: el primer y el último commit en que se vio y
  cuántos commits y versiones de archivo lo contienen. La pestaña indica cuántos
  archivos binarios y demasiado grandes no se leyeron, y que el análisis no es
  exhaustivo. Todos los hallazgos empiezan marcados; **Quitar del historial**
  (`r`) convierte los marcados en una sola regla que los nombra por id (nunca por
  valor), que previsualizas y aplicas como cualquier otra. Los hallazgos quedan
  obsoletos tras una reescritura o un cambio de alcance: la pestaña lo dice,
  analiza de nuevo. El editor nunca tiene el valor de un secreto, así que no
  puede mostrarlo.

### Encontrar lo que pesa

- **Encontrar lo que pesa:** `gitpoulpi rewrite analyze [--top N]` (y la
  herramienta MCP `rewrite_analyze`) es el análisis de tamaño, como
  `filter-repo --analyze`: los archivos más grandes con su ruta, tamaño, primer y
  último commit y si siguen en HEAD, las carpetas más grandes, los archivos
  eliminados de HEAD que el historial aún guarda, y los totales. Solo lee
  tamaños y rutas, nunca el contenido de los archivos, y no cambia nada.
- **La pestaña Tamaño** hace lo mismo desde el editor (`F7`, o *Analizar los
  tamaños del historial* en la paleta, la abre y lo ejecuta): **Analizar
  tamaños** (`s`) es una tarea (*Analizando*; **Cancelar** o `Esc` la detiene).
  **Mostrar** elige la lista (`v`): *Archivos* (las versiones de archivo únicas
  más grandes, cada una marcada *en HEAD*, *versión antigua* de un archivo que
  sigue ahí, o *eliminado*), *Carpetas* (todo lo que se ha guardado alguna vez
  bajo cada una) o *Eliminados* (rutas que ya no están en HEAD pero que el
  historial conserva). **Ordenar** (`o`) es por tamaño, por ruta o eliminados
  primero. Marca filas (`Space`, `Alt+A` / `Alt+U` para todas / ninguna): la
  pestaña dice cuánto contienen, con sinceridad como «hasta unos», porque el
  espacio solo se recupera tras la reescritura, una purga de los objetos
  antiguos y un `gc`. **Quitar del historial** (`r`) convierte las rutas marcadas
  en una regla de rutas (con los caracteres especiales escapados; una ruta que
  ya está en una regla no se añade dos veces); **Quitar todo archivo de más de
  ...** (`b`, lista de Archivos) escribe una regla de tamaño justo por debajo del
  archivo marcado más pequeño. Genera la vista previa y aplícalas como cualquier
  otra regla. Las listas muestran las 200 filas más grandes; la CLI dice cuántas
  más hay y `--top` las lista. Las filas quedan obsoletas tras una reescritura o
  un cambio de alcance: analiza de nuevo.

### Firmas

- **Firmas:** un commit reescrito no puede conservar su firma (firmaba los bytes
  antiguos), así que, cuando se reescriben commits firmados, se te pregunta cada
  vez: *Quitar las firmas* (antes se muestra el recuento), *Volver a firmar solo
  los commits que estaban firmados* o *Volver a firmar todos los commits
  reescritos*. No hay opción por defecto. Los commits que quedan como están
  conservan la suya. Volver a firmar usa tu propia configuración, igual que un
  commit firmado: `gpg.format` (`openpgp`, `ssh` o `x509`), `user.signingkey` y
  `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`. La cuarta fila de la
  pregunta, **Comprobar la firma** (`4`), firma una vez un contenido de prueba y lo
  verifica: te dice cuánto tarda una firma y estima la ejecución para el número de
  commits de la pregunta, o por qué no funciona (falta la clave, el agente está
  bloqueado, el programa no está instalado, con cómo instalarlo). La vista previa
  hace la misma prueba antes de leer el historial y se rechaza con ese motivo, así
  que una configuración rota nunca deja una reescritura firmada a medias. Los
  commits se firman **durante la vista previa** (sus ids contienen la firma), así
  que una vista previa que vuelve a firmar tarda lo que tarde la firma, el Resumen
  dice *Commits firmados de nuevo: n, en 12.5 s*, y entonces puede abrirse una
  petición de contraseña: desbloquea antes tu agente (Comprobar la firma hace
  justo eso). **Cancelar** se detiene entre dos firmas y no deja nada a medias. Solo
  se vuelven a crear las firmas de commit; un tag firmado sigue perdiendo la suya.
- **La regla *Quitar firmas*** (menú Añadir, grupo *Firmas*; *Reescritura: Quitar
  firmas...* en la paleta; `--remove-signatures`) quita la firma de todos los
  commits del alcance, aunque no cambie nada más, y no pregunta nada sobre volver a
  firmar. Desde la CLI: `rewrite check-signing` y `--signatures
  strip|resign-signed|resign-all`.

### Deshacer una reescritura

**Deshacer** (el botón del panel de resultado, *Deshacer* de la barra de herramientas
con `Ctrl+Z` en el Log, o `gitpoulpi undo`) devuelve cada rama, tag y nota movidos a
su sitio, y con ellos los archivos activos, a partir de los commits antiguos que la
reescritura mantiene anclados. Pregunta antes cuando una rama reescrita ya está en su
upstream (deshacer en local no deshace un push) y cuando tienes cambios sin
confirmar: se conservan, y un archivo que cambiaste y que el historial antiguo
sobrescribiría detiene el deshacer (los archivos se mueven como los mueve
`reset --keep`). Deshacer deja de ser posible para una reescritura cuando
[purgas](#eliminar-el-historial-antiguo-purga) su historial antiguo; la lista del
historial la marca entonces como *purgada*.

## Mover archivos a Git LFS o traerlos de vuelta

Dos reglas del grupo *LFS* del menú Añadir reescriben el historial para que los
archivos grandes vivan en [Git LFS](lfs.md) desde el principio (o dejen de hacerlo).
Necesitan el programa `git-lfs`: sin él, las reglas aparecen en gris con la pista de
instalación y la vista previa se rechaza. Son nuestro propio motor, no `git lfs
migrate`, así que tienes la vista previa, el alcance, el deshacer y la protección del
push de cualquier otra regla.

- **Migrar archivos a LFS** (*Reescritura: Migrar archivos a LFS...*, o
  `--lfs-import PATTERN` con `--lfs-min-size BYTES`). Todo archivo que coincide con
  los patrones (`*.psd`, `assets/**`; un patrón sin barra coincide a cualquier
  profundidad) y es al menos tan grande como *Solo archivos de al menos* (vacío:
  todos los que coincidan) pasa a ser un puntero LFS en cada commit que lo contiene.
  Su contenido va al almacén LFS local (`.git/lfs/objects`, una vez por cada
  contenido distinto) al Aplicar, y el `.gitattributes` de la raíz recibe
  `<patrón> filter=lfs diff=lfs merge=lfs -text` en cada commit reescrito que
  contiene un puntero (se crea si no hay ninguno, las demás líneas no se tocan y
  nunca se duplica). Un archivo que ya es un puntero se deja como está. El Resumen
  dice *3 archivos (12 MiB) pasan a LFS. Objetos LFS por escribir: 3.* y la pestaña
  Commits lista *mueve big.psd a LFS* por commit.
- **Recuperar archivos de LFS** (*Reescritura: Recuperar archivos de LFS...*, o
  `--lfs-export PATTERN`). Los punteros que coinciden vuelven a ser los archivos
  reales y se quitan las líneas de `.gitattributes` de esos patrones. Los objetos
  deben estar en el almacén LFS local: cuando faltan algunos, la vista previa se
  rechaza con la lista y un botón **Descargar objetos LFS** que los trae de
  `origin`; después, genera la vista previa de nuevo. Una importación seguida de una
  exportación de los mismos patrones devuelve exactamente los mismos commits.

Tras Aplicar, los archivos activos son el contenido real (no punteros) y
**Deshacer** devuelve el historial antiguo. El panel de resultado tiene **Enviar
objetos LFS** (`l`): lo mismo que `LFS: Push All Objects`, para poder hacer push del
historial nuevo sin que un push posterior se quede atascado por objetos que faltan
(un push normal de las ramas reescritas también los sube mediante el hook de
pre-push que instala Aplicar). La nota para colaboradores dice que necesitan git-lfs
y `git lfs pull`.

Conviene saber: Aplicar instala los filtros LFS del repositorio (`git lfs install
--local`) cuando faltan, porque git los necesita para comparar los archivos reales con
los punteros. Solo se escribe el `.gitattributes` de la raíz, nunca uno anidado. Un
archivo *por debajo* del tamaño mínimo que coincide con un patrón sigue siendo un
archivo normal, pero git querrá convertirlo en puntero la próxima vez que se añada,
así que `git status` lo muestra como modificado hasta que le hagas commit. La
importación procesa los archivos grandes en flujo; la exportación tiene en memoria un
archivo cada vez.

## Enviar el resultado

Aplicar una reescritura nunca hace push. Las ramas y los tags reescritos que
estaban en un remoto necesitan un force push, y ese push nunca fuerza a ciegas:
cada ref se envía con un lease sobre el id que tenía el remoto cuando hiciste la
vista previa (no lo que diga ahora tu rama de seguimiento), de modo que el push de
un compañero entre medias se rechaza para esa ref en lugar de sobrescribirse, y las
refs van en un único push de todo o nada (`--atomic`) cuando el servidor lo
permite. Desde la CLI:

```
gitpoulpi rewrite push --dry-run        # pregunta al remoto, no envía nada
gitpoulpi rewrite push --yes            # lo envía
gitpoulpi rewrite note                  # lo que deben ejecutar los demás
gitpoulpi rewrite maps --out ./maps     # commit-map y ref-map
```

Las ramas protegidas (ajuste `rewrite.protected_branches`, por defecto `main`,
`master`, `develop`, `release/*`) se dejan en paz si ya están publicadas, tanto en
la reescritura como en el push, salvo que las nombres (`--include-protected main`)
o las incluyas todas. La protección de ramas del propio host sigue decidiendo si
acepta un force push; su rechazo se muestra por ref.

En el editor, el panel de resultado tras Aplicar hace todo esto:

- **Enviar las refs reescritas al remoto** lista las refs que movió la
  reescritura y que tiene un remoto, cada una con una casilla (`Space`). Las
  ramas protegidas empiezan sin marcar, con *protegida* al lado: márcalas una a
  una, o usa *Marcar todas* / *Desmarcar todas* (`Alt+A` / `Alt+U`). **Push de
  las ramas reescritas** (`p`) envía las marcadas; la respuesta es una fila por
  ref (*enviada*, *ya está al día*, *rechazada: el remoto ha cambiado desde la
  vista previa*, *rechazada: protegida por el host*, *omitida: rama protegida*,
  *no enviada*). Una fila rechazada tiene **Fetch y revisar** (`f`): el editor se
  cierra y se hace fetch de los remotos, para que el Log muestre lo que
  enviaron los demás. No se sobrescribe nada: el lease protege su trabajo, y
  generas la vista previa de nuevo antes de volver a intentarlo.
- **Copiar la nota** (`c`) pone en el portapapeles la nota para colaboradores,
  en el idioma de la interfaz salvo los comandos de git, que no se traducen.
- **Abrir el mapa de commits** (`m`) muestra dónde están el mapa de commits y el
  mapa de refs; un clic en una ruta (o `1` / `2`) la copia.

Un push no se puede deshacer: si haces `undo` de la reescritura después, solo
cambia tu repositorio, y el deshacer lo dice antes de preguntar. Envía la nota a
todos los que tengan el historial antiguo: lista la punta antigua y la nueva de
cada rama y tag y los comandos (`git fetch`, luego `git reset --hard
origin/<branch>` o `git rebase --onto`). Los dos mapas se conservan 24 horas junto
con la ejecución; cópialos si la limpieza de repositorio de GitLab o un script los
necesita.

## Cuando el host protege una rama

Tu host puede proteger una rama por su cuenta (la protección de ramas y los rulesets
de GitHub, las ramas protegidas de GitLab, las políticas de rama de Azure DevOps, las
restricciones de rama de Bitbucket) y rechazar el force push que necesita una
reescritura. Cuando el repositorio está **vinculado** a su host (la sección de pull
requests de la barra lateral, véase [Integraciones de alojamiento](hosting-integration.md)),
el editor le pregunta al host qué ramas protege, con tu propio token:

- Esas ramas **se dejan fuera de la reescritura por defecto**, igual que las ramas
  protegidas de `rewrite.protected_branches` y del flujo de trabajo: la pestaña Ramas
  y tags las lista con una casilla cada una y los botones *Marcar todas* /
  *Desmarcar todas*, y dice por qué: *protegida en GitHub*.
- Si el token no puede leer la protección (le falta el permiso, o no eres
  administrador), el editor dice *Protección del host desconocida* y no da por
  protegido en el host nada que él no confirme. La reescritura nunca se bloquea por eso.
  A un repositorio que no está vinculado nunca se le pregunta.

Cuando una rama protegida estorba (la vista previa se rechazó porque todas las ramas
están protegidas, o el host rechazó el push: *rechazada: protegida por el host*), el
botón **Cómo sortearla** (`Alt+H`) explica las tres salidas, con la página de ajustes
exacta de tu proveedor y de tu repositorio (elige un enlace y cópialo con `c`, o ábrelo
con `o`). GitPoulpi nunca hace ninguna de ellas por ti:

1. **Quitar la protección temporalmente** en los ajustes del host, hacer el push y
   volver a ponerla.
2. **Hacer push de la reescritura con otro nombre de rama**, convertir esa rama en la
   rama por defecto (o renombrarla) y eliminar la antigua cuando nadie la use.
3. **Como último recurso, eliminar y volver a crear el repositorio remoto** y hacer push
   de todo. Esto pierde las pull / merge requests y sus revisiones, los issues, el
   vínculo de los forks con el historial antiguo, las estrellas y los seguidores, los
   ajustes, secretos y variables de CI, y los webhooks y las claves de despliegue.

Desde la CLI, el mismo texto vuelve como `protected_by_host` en el JSON de
`rewrite preview` / `rewrite run` (cuando el host protege una rama del alcance), de un
`rewrite push` rechazado y de `rewrite note`: `provider`, `branches`, `ways` (cada una
con sus `steps` y `links`), `recreate_loses` y la explicación entera como `text`.

## Eliminar el historial antiguo (purga)

Tras una reescritura, los commits antiguos siguen en tu repositorio: el deshacer
de la reescritura los conserva (eso es lo que lo hace posible), y también los
reflogs. Si reescribiste para librarte de un secreto, ese secreto sigue en la
carpeta `.git`. La purga elimina el historial antiguo para siempre:

```
gitpoulpi rewrite purge                 # pregunta: lista lo que se elimina y se pierde (código de salida 3)
gitpoulpi rewrite purge --yes           # lo hace
```

Suelta las anclas de deshacer de la reescritura, caduca las entradas del reflog
que nombran los commits antiguos, ejecuta `git gc --prune=now` y comprueba que
todo commit y toda versión de archivo antiguos han desaparecido de verdad. La
confirmación dice antes lo que pierdes: **ya no se podrá deshacer** la
reescritura (ni cualquier operación anterior cuyo deshacer también necesitara los
commits antiguos); siguen en la lista del historial, marcadas como purgadas. Una
purga no se puede detener una vez iniciada ni se puede deshacer. No toca ningún
remoto.

Todo lo que aún retenga el historial antiguo bloquea la purga, y entonces no se
cambia nada. La lista nombra cada cosa con lo que hay que hacer: una rama o un tag
que no formaba parte de la reescritura, un stash sobre el historial antiguo
(elimínalo, o consérvalo y acepta que el historial antiguo se quede), otro
worktree cuyo HEAD está sobre él, y las ramas de seguimiento remoto que aún
apuntan al historial antiguo (haz push de las ramas reescritas y luego fetch). Tu
lista de stash y los reflogs de otras ramas no se tocan nunca.

Un secreto que ya se publicó se considera filtrado: **rótalo primero**. Tras la
purga, la respuesta lista lo que aún necesita el host, por remoto: GitHub
mantiene accesibles los commits antiguos en vistas en caché y en las refs de las
pull requests (contacta con GitHub Support, o elimina y recrea el repositorio),
GitLab tiene una limpieza de repositorio que toma el `commit-map` (`gitpoulpi
rewrite maps --out DIR`) más el mantenimiento, Azure DevOps y Bitbucket necesitan
una solicitud de soporte o un repositorio nuevo, y todo el que tenga un clon o un
fork conserva el historial antiguo.

### La pantalla de purga

En el editor, el panel de resultado de una reescritura tiene un botón **Purgar el
historial antiguo...** (tecla `e`), y la paleta tiene **Reescritura: Purgar el
historial antiguo...** (actúa sobre la reescritura más reciente que aún no esté
purgada, también después de cerrar el editor). La pantalla lista lo que elimina la
purga y lo que pierdes (las operaciones cuyo deshacer desaparece con ella; las
marcadas *también pierde el deshacer* no son reescrituras pero necesitaban los
commits antiguos), y luego lo que aún retiene el historial antiguo, una fila para
cada cosa:

- una rama de seguimiento remoto: **Fetch** las actualiza desde los remotos;
- un stash: **Eliminar** pregunta antes, porque un stash es trabajo que nunca
  confirmaste;
- una rama, un tag, un worktree u otra ref: **Copiar nombre**, y resuélvelo donde
  vive (reescribe o elimina la rama, activa una rama reescrita en el worktree).

La comprobación se repite tras cada arreglo, y **Comprobar de nuevo** (`r`) la
ejecuta a demanda. **Purgar** (`p`) solo está disponible cuando no queda nada.
Hace una última pregunta, numerada y sin opción por defecto (`1` purga para
siempre, `2` lo deja todo como estaba), y luego se ejecuta; la pantalla muestra el
paso en que va y dice que no se puede detener. El resultado indica que el historial
antiguo ya no está en este clon, o cuántos objetos antiguos quedan. Debajo, los
pasos que hay que dar en cada remoto (se puede copiar un enlace). La cruz, `Esc` y
un clic fuera cierran la pantalla, salvo mientras se ejecuta la purga.

Por MCP, la herramienta de purga no existe salvo que pongas
`allow_history_purge = true` en la tabla `[mcp]` de `config.toml`.

## La regla Script

*Añadir regla > Script* (o **Reescritura: Ejecutar un script...** en la paleta)
añade una regla de script Lua y abre su editor sobre el editor de reescritura;
`Enter` en la línea *Script* de la regla lo vuelve a abrir. El editor parte de una
plantilla pequeña.

- **Lenguaje** (una fila del panel de la regla): *Lua (integrado)*, *Script de
  shell*, *Python* o *Lua (instalado)*. Todo salvo el Lua integrado es un [script
  externo](#scripts-externos-shell-python-lua): sin aislar, y un lenguaje
  desactivado en Ajustes o cuyo intérprete falta aparece atenuado en la lista con
  el motivo al lado (al elegirlo dice qué hacer). Cambiar el lenguaje de un
  script que aún no has escrito cambia la plantilla.
- **Script** es un editor de texto con números de línea. Lo que escribes es la
  regla al instante; no hay nada que confirmar.
- Debajo, la línea de **comprobación** dice `Correcto: se encontró
  rewrite_commit`, o lista los problemas con su línea (`script:3: ...`; el número
  aparece en rojo en el margen). Se ejecuta un momento después de que dejes de
  escribir, nunca mientras mantienes una tecla pulsada.
- **Parámetros** tiene una fila por cada línea `--! param` de la cabecera del
  script, con su etiqueta junto al campo: texto, un número o una casilla para
  verdadero/falso. Un valor que borras vuelve al valor por defecto de la
  cabecera; un parámetro de texto sin valor por defecto debe rellenarse antes de
  la vista previa.
- **Archivo** (una ruta dentro del repositorio, para empezar
  `.gitpoulpi/rewrite/script.lua`) con **Cargar desde archivo...** (reemplaza el
  texto) y **Guardar en archivo...**.
- **Probar con la selección** ejecuta el script sobre los commits que
  seleccionaste en el Log, o sobre los 20 más recientes si no hay ninguno
  seleccionado, y no escribe nada. La tabla lista cada commit (id corto), cada
  campo que cambiaría como `antiguo -> nuevo`, y las líneas `log(...)` del
  script. **Cancelar** deja de esperar. Si el script falla, se muestran el commit
  y el motivo.
- La cruz, un clic fuera del editor y `Esc` lo cierran; `Tab` y `Shift+Tab`
  mueven entre el editor, los parámetros, el archivo, los botones y los
  resultados; `F5` prueba.

La vista previa rechaza un script que no compila: el panel de la vista previa
dice *un script falló* con la línea (o, si falla en un commit, su id corto y el
motivo). En una versión sin scripts, el tipo sigue en el menú Añadir, atenuado.

## Escribir un script con IA

Con la IA activada ([asistencia de IA](ai.md)), el editor de scripts tiene un botón
`✦ Escribir con IA...`, y la paleta tiene `✦ Reescritura: Escribir un script con
IA...`, que abre el editor de reescritura sobre una regla Script nueva con el
diálogo abierto. No se envía nada antes de pulsar **Enviar**: el diálogo muestra,
mientras escribes, el texto exacto que saldrá.

- **¿Qué debe hacer el script?** Tus propias palabras: *pasar a la mañana
  siguiente todos los commits hechos de noche*, *sustituir mi correo antiguo por
  el nuevo, solo antes de 2024*.
- **Enviar una muestra de N commits** (desactivado): si lo marcas, se añaden el
  autor, la fecha y el asunto de los commits que usaría «Probar con la
  selección» (tu selección del Log, o si no los 20 más recientes) para que el
  modelo vea cómo son tus datos. Nunca el contenido de los archivos ni los
  cuerpos de los commits. Con la casilla desactivada, ningún dato de commits sale
  de la máquina.
- **Qué se envía** lista las instrucciones (el contrato fijo del script y luego
  tus instrucciones de IA) y tu petición, y qué perfil las recibe. `Tab` pasa a
  esa zona, las flechas la desplazan.
- **Enviar** (`Tab` hasta él, `Enter`). Mientras el modelo trabaja el diálogo
  espera, y `Esc` cancela la petición. Si falla, el diálogo se queda con el
  motivo para que puedas intentarlo de nuevo.

La respuesta se comprueba con la misma comprobación del daemon que un script
escrito a mano; un error de compilación se devuelve una vez al modelo. El script
**reemplaza entonces el texto del editor** (el diálogo lo avisa cuando el editor
tiene algo distinto de la plantilla) y el título del editor dice *✦ escrito por
IA: léelo antes de la vista previa* hasta que cambies el texto. Nunca se prueba,
previsualiza ni aplica por ti: léelo, pulsa **Probar con la selección** y luego
genera la vista previa como con cualquier script. El script se ejecuta en el mismo
entorno aislado que uno escrito por ti (sin archivos, sin reloj, sin red).

Desde la línea de comandos: `gitpoulpi ai rewrite-script "<instrucción>" [--sample N]`
imprime el script y el resultado de `rewrite script check` sobre él como JSON y no
ejecuta nada (código de salida 2 si aún no compila). Se usa el perfil *Script de
reescritura del historial* de Ajustes > IA.

## Crear reglas con IA

Con la IA activada ([asistencia de IA](ai.md)), el editor tiene un botón `✦ Crear reglas con IA...`
junto a Guardar y Cargar, la tecla `Alt+I` (*Crear reglas de reescritura con IA* en Ajustes > Teclado y
ratón, contexto *Rewrite*), y la paleta tiene `✦ Reescritura: Crear reglas con IA...`. Con la IA
desactivada, o en una compilación sin IA, el botón y la fila de la paleta no están. Describes con tus
palabras cómo debe reescribirse el historial (*sustituir mi antiguo correo a@b.org por c@d.org*,
*quitar secrets.txt de todo el historial*, *desplazar mis fechas dos horas*) y el modelo propone las
reglas y los ajustes, que **rellenan el editor**. No se envía nada antes de pulsar **Enviar**: el
diálogo muestra, mientras escribes, el texto exacto que saldrá.

- **¿Cómo debe reescribirse el historial?** Un campo de varias líneas: `Enter` añade una línea,
  `Ctrl+Enter` o `Ctrl+S` envía, `Esc` y la cruz cancelan.
- **Enviar el alcance, los nombres de ramas y tags y los autores del historial** (desactivado): si
  lo marcas, se añaden el alcance de la barra de alcance, los nombres de ramas y tags y los autores
  distintos (nombre y correo) de los commits más recientes que muestra el Log (20 commits como
  máximo), para que el modelo pueda nombrar tu correo con exactitud (el `ai rewrite-rules --context` de la
  línea de comandos lee en cambio los 200 commits más recientes). Nunca el contenido de los
  archivos ni los mensajes de los commits. Con la casilla desactivada, solo tu descripción y las
  instrucciones fijas salen de la máquina.
- **Reemplazar las reglas actuales** (solo cuando el editor ya tiene reglas; desactivado): por
  defecto las reglas nuevas se añaden después de las que tienes; si lo marcas, las reemplazan (el
  alcance y los ajustes que indica la respuesta se aplican en ambos casos).
- **Qué se envía** lista las instrucciones (el vocabulario de las reglas y luego tus instrucciones
  de IA), tu petición y el perfil que las recibe. `Tab` pasa a esa zona, las flechas la desplazan.

El modelo puede usar todos los tipos de regla de esta página salvo **Script** (es código: *Escribir
un script con IA* muestra el script para que lo revises) y la forma de valores secretos de *Buscar
y reemplazar en archivos* (los valores secretos nunca se entregan a un modelo: usa la pestaña
Secretos). Puede fijar el alcance en la rama actual, en todas las ramas locales y tags, o en ramas
y tags concretos que existan, y las opciones *Tags*, *committer*, *commits vacíos* e *IDs
abreviados en los mensajes*. Lo que el editor no puede expresar (un límite de fecha como *desde
marzo*, un rango de commits) no se adivina: el modelo lo dice en una observación. Qué pasa con
las firmas nunca lo decide el modelo: se te pregunta al Aplicar.

**La IA solo prepara reglas y ajustes.** Nunca previsualiza, aplica ni ejecuta nada: la respuesta
se copia en el editor como una receta que hubieras cargado, la vista previa en pantalla queda
desactualizada, y Vista previa y Aplicar (con sus preguntas y confirmaciones) son cosa tuya, como
siempre. Cada regla se analiza y se comprueba por separado con la misma comprobación que usan el
editor y el motor; una regla que falla se deja fuera. Cuando algo se dejó fuera, o el modelo hizo
una observación, una lista lo muestra después de añadir las reglas (*Descartadas (se conservaron
las reglas válidas)*, cada una con su posición, su nombre y el motivo) y `Enter` o `Esc` la
cierra. Lee las reglas, edítalas y luego genera la vista previa.

Desde la línea de comandos: `gitpoulpi ai rewrite-rules "<descripción>" [--context]` imprime la
propuesta como JSON (`recipe`, que lee `rewrite preview --recipe-json`, `problems`, `notes` y
`applied: false`) y no ejecuta nada (código de salida 2 cuando se dejó algo fuera). La herramienta
MCP es `ai_rewrite_rules`. Se usa el perfil *Reglas de reescritura del historial* de Ajustes > IA.

## Scripts Lua

Una regla de script ejecuta tu función en cada commit. Es Lua 5.4 normal dentro
de un sandbox: sin archivos, sin reloj, sin procesos, sin módulos y sin `pcall`;
el mismo script sobre el mismo historial da siempre el mismo resultado, y una
llamada que supera los 10 millones de instrucciones, los 64 MiB o los 5 segundos
se detiene con un error que nombra el commit.

```lua
--! name: Reemplazar un correo
--! param old_email: text "Correo a reemplazar"
--! param new_email: text "Correo nuevo"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` contiene `id`, `parents`, `tree`, `author` y `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` y `signed`. `ctx` contiene `index`, `total`,
`params`, `state` (devuelve `state = {...}` para pasar datos al commit siguiente)
y `window.before` / `window.after` (define `--! window: N`). Devuelve solo lo que
cambia: `author`, `committer`, `message`. Funciones auxiliares: `regex_replace`,
`regex_match`, `stable_hash`, `rand`, `weekday`, `hour`, `log` y, para los mensajes,
`trailers(message)` (una lista de `{key, value}`), `set_trailer(message, key, value)`
(reemplaza los trailers de esa clave; un cuarto argumento `"add"` o `"skip"` lo
cambia), `remove_trailer(message, key)` y `at("2024-03-05 18:30", offset_minutes)` (una
fecha en un desfase dado, como tiempo Unix).

Desde la línea de comandos: `gitpoulpi rewrite script check my.lua` lo compila,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` muestra
lo que cambiaría sin escribir nada, y
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` planifica
toda la reescritura.

## Scripts externos (shell, Python, Lua)

El mismo contrato que un script Lua, ejecutado por un programa de tu máquina:
*Script de shell* (bash o sh; PowerShell en Windows, o cualquier `.ps1`),
*Python* y *Lua (instalado)* (un `lua` tuyo, no el integrado; no tiene biblioteca
JSON, así que la plantilla lee la línea con un patrón). **No están aislados: un
script externo se ejecuta con tus permisos** y puede leer o modificar todo lo que
tú puedes. El panel de la regla y el título del editor lo dicen.

Cómo se comunica: GitPoulpi arranca el script una vez por ejecución y le envía una
petición JSON por línea en su entrada estándar (los campos de un commit, `params`,
el `state` devuelto por la última respuesta); él responde con una línea JSON en la
salida estándar, con los mismos campos que devuelve un script Lua (`author`,
`committer`, `message`, `state`, `log`), o `{"error": "texto"}` para rendirse.
Imprime los diagnósticos en la salida de error estándar: aparecen como las líneas
`log` del commit. Las líneas de cabecera empiezan por `#!` en shell y Python
(`#! name: ...`, `#! param old: text "Antiguo"`; una línea shebang sigue siendo un
shebang) y por `--!` en Lua.

```sh
#!/bin/sh
#! name: Autores en mayúsculas
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Qué recibe.** Un entorno vacío salvo `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` y `LANG`; una carpeta temporal nueva como carpeta de
trabajo (con su propia copia del script); no la ruta del repositorio. **Límites de
tiempo:** `rewrite.script_timeout_ms` (5000) para cada respuesta y
`rewrite.script_total_timeout_s` (3600) para toda la pasada. Cuando se superan,
falla, imprime algo que no es JSON o se cancela la tarea, GitPoulpi mata el script
y todo lo que arrancó, y la reescritura se detiene con el commit y el motivo; no
se ha movido nada.

**Qué lenguajes existen.** `rewrite.external_scripts` en `config.toml` los lista:
`["shell"]` por defecto; añade tú `"python"` y `"lua"` en `config.toml`
(`[rewrite]`, `external_scripts = ["shell", "python"]`; una lista no se puede
fijar con `gitpoulpi config set`). Un lenguaje que no figura aparece atenuado en
el editor y la CLI y MCP lo rechazan (`scripts_disabled`); uno que figura pero
cuyo intérprete no se encuentra aparece atenuado con la pista de instalación
(`tool_missing`).

**Confianza.** Aunque un lenguaje esté activado, un script solo se ejecuta después
de que le des confianza a su texto exacto. Un script que escribes en el editor es
tuyo y se ejecuta al instante; uno que viene con una receta compartida o un
archivo del repositorio no: la vista previa y *Probar con la selección* se
detienen y preguntan *¿Confiar en este script?* con dos respuestas numeradas y
ninguna preseleccionada (`1` confiar y continuar, `2` no confiar, `Esc` lo
deja). Dar confianza recuerda el SHA-256 del texto para este repositorio, así que
un script modificado vuelve a preguntar, y un repositorio clonado no puede
ejecutar código en tu máquina por sí solo. Desde la línea de comandos, lee el
archivo y luego `gitpoulpi rewrite script trust FILE` (o `--trust` en `preview`,
`run`, `script check` y `script test`, que da confianza y continúa). Un agente por
MCP no puede dar confianza a nada: solo puede ejecutar un script en el que tú ya
confiaste.

**Comprobación.** La línea de comprobación bajo el editor dice *Aún no es de
confianza* para un script sin confianza (no se arrancó nada); para uno de
confianza, GitPoulpi lo arranca una vez con un commit inventado (`"dry_run":
true`) y comprueba que la respuesta sigue el contrato. `gitpoulpi rewrite script
check FILE` hace lo mismo e informa `trust`, `hash` e `interpreter`.

## Desde la línea de comandos

`gitpoulpi rewrite` (y las herramientas MCP del mismo nombre) usa el mismo motor,
para scripts y agentes. Una **receta** es un archivo TOML o JSON, el que guarda el
editor (`--recipe FILE`, o `-` para la entrada estándar), y los flags añaden reglas
después de las de la receta.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plan, no cambia nada
gitpoulpi rewrite apply --run ID --digest D           # ejecuta el plan que revisaste
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # lo deja todo como estaba
```

- Reglas como flags: `--set-name` / `--set-email` con `--match-email` (identidad),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. El alcance es la rama actual, `--all-local` o `--ref R`
  (repetible), desde `--since REV`.
- **Las preguntas sin respuesta por defecto** son flags: `--signatures
  strip|resign-signed|resign-all` cuando cambian commits firmados, y
  `--extract-mode` para una extracción. Sin ellos la respuesta es un rechazo o un
  `needs_choice` (código de salida 3), nunca una suposición.
- **Códigos de salida:** 0 hecho; 1 un error (receta ilegible, flags incorrectos);
  2 rechazado, con el motivo (`stale`, `signature_choice_needed`,
  `dirty_paths_touched`, `empty_scope`, ...); 3 una pregunta que responder, o refs
  ya publicadas que necesitan `--yes`.
- `preview` guarda el plan en el repositorio, así que `apply` funciona desde un
  comando posterior, tras reiniciar el daemon o desde otro frontend; un plan que ya
  no coincide con su digest es `stale`.

La [referencia de la CLI](cli-reference.md) tiene todos los flags, y
[MCP y agentes](mcp-and-agents.md) dice qué herramientas recibe un agente (la
herramienta de purga no existe salvo que la actives, y un agente no puede dar
confianza a un script).

## Propuestas de agentes de IA

Un agente puede **sugerir** una reescritura, nunca ejecutarla. `gitpoulpi ui propose
rewrite --recipe FILE|-` (la herramienta MCP `ui_propose_rewrite`, desactivada hasta
que pongas `[mcp] allow_ui_propose = true`) abre el editor en tu ventana en marcha
con las reglas de la receta ya en la lista y un aviso morado, *✦ Propuesto por
<agente>: una reescritura del historial (3 reglas) · Ctrl+Enter la acepta · Esc la
rechaza*.

- **No se ejecuta nada.** Ni vista previa, ni aplicar, ni push, ni purga: la propuesta
  solo rellena las reglas. Las lees, las editas si quieres, generas la **Vista
  previa** y aplicas con tu propia tecla. Cerrar el editor rechaza la propuesta.
- **Lo que el agente no puede incluir:** un script (se ejecutaría al previsualizar),
  más de 50 reglas, valores secretos (se quitan de la receta antes de enviarla). Una
  propuesta así se rechaza con el motivo.
- El agente puede leer el resultado en `ui state` (`accepted` cuando aplicaste las
  reglas tal como se propusieron, `accepted_edited` cuando las cambiaste antes,
  `rejected` cuando lo cerraste). Una propuesta a la vez; se rechaza mientras haya
  otro diálogo tuyo abierto.

## Herramientas

La mayor parte de una reescritura del historial no necesita nada más que Git:
cambiar autores y committers, aplicar un `.mailmap`, anonimizar, desplazar o
limitar fechas a un horario, reescribir mensajes y trailers, reemplazar
contenido, quitar rutas o archivos grandes y mover rutas se ejecuta dentro de
GitPoulpi.

**Los scripts Lua vienen integrados.** El intérprete forma parte de GitPoulpi,
así que un script Lua no necesita ninguna herramienta ni configuración. Los
scripts de shell, Python y Lua instalado necesitan su intérprete (véase la tabla
de abajo) y se describen en [Scripts
externos](#scripts-externos-shell-python-lua).

Algunas funciones usan un programa externo. GitPoulpi lo comprueba (el programa
se busca en tu `PATH`, en Windows también mediante `PATHEXT` y el propio
`usr/bin` de Git para Windows, y debe responder a `--version` en menos de 2
segundos); la respuesta se guarda en caché un minuto y "Comprobar de nuevo"
vuelve a preguntar. Una regla que necesita un programa ausente se queda en la
lista, en gris, con el motivo y cómo instalarlo, y la vista previa se niega antes
de ejecutar nada.

| Función | Herramienta | Notas |
|---|---|---|
| Volver a firmar commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) o `gpgsm` (`gpg.format = x509`) | La firma SSH y X.509 también necesita `user.signingkey`. Se respetan `gpg.program`, `gpg.ssh.program` y `gpg.x509.program`. |
| Migración a LFS (las dos reglas de LFS) | `git-lfs` 3.0 o posterior | La misma comprobación que el panel de LFS. |
| Scripts de Python | `python` 3.8 o posterior (`py -3` en Windows) | Desactivados por defecto: añade `"python"` a `rewrite.external_scripts`. |
| Scripts de Lua (instalado) | `lua5.4`, `lua54`, `lua` o `luajit` | Desactivados por defecto: añade `"lua"` a `rewrite.external_scripts`. |
| Scripts de shell | `bash`, `pwsh` (PowerShell) o `sh` | Activados por defecto (`"shell"`). Un script de PowerShell (un archivo `.ps1`, un shebang `pwsh`, o cualquier script de shell en Windows) necesita `pwsh` o `powershell`; uno POSIX necesita `bash` o `sh` (en Windows los aporta Git para Windows). |

Consejos de instalación, primero Windows:

- Windows: Git para Windows incluye `gpg`, `ssh-keygen`, `bash`, `sh` y Git LFS.
  Si no, `winget install GnuPG.GnuPG`, Configuración > Características
  opcionales > Cliente de OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: los paquetes de la distribución (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` y `sh`
  vienen integrados.
