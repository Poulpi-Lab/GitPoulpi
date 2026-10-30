[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · **Español** · [Français](../fr/merge-conflicts.md) · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Resolver conflictos de merge

Cuando un merge, cherry-pick, revert o rebase se detiene por conflictos, el Log muestra un
banner (*Merge en curso · feature en main · 2 archivos en conflicto*) y los archivos en
conflicto aparecen en el panel Estado con una `!` roja. Abre uno en el **editor de merge**: A
(el lado actual) y B (el lado entrante) en paralelo, con el **Resultado**, el archivo que estás
escribiendo, debajo. Esta página es la versión corta; la lista completa de teclas está en
[Atajos de teclado](keybindings.md).

## Abrirlo

- `Enter`, o un doble clic, en una fila con `!` del panel Estado; su menú de clic derecho
  empieza por **Abrir en el editor de merge**.
- El botón **Resolver conflictos...** del banner (abre el primer archivo en conflicto).
- La vista Diff de un archivo en conflicto: el botón **Editor de merge** (`Shift+E`).
- La paleta: **Conflictos: Abrir el editor de merge**.

No se abre nada por sí solo: una operación detenida solo muestra el banner y las filas con `!`.

![El Log de un repositorio detenido en un merge con conflictos: el banner Merge en curso con Resolver conflictos y Abortar merge, y dos archivos en conflicto marcados con una ! roja](../images/conflict-banner.png)

El editor ocupa todo el ancho de la ventana: la barra lateral Refs y el panel
derecho se ocultan mientras está abierto y vuelven al cerrarlo (`F9` los muestra u oculta
entretanto). A y B van en paralelo siempre que el editor mida al menos
`merge.side_by_side_min_width` columnas (100 por defecto, Ajustes > Diff y editor; `0` significa
siempre); si es más estrecho, ves un lado cada vez detrás de pestañas. Las líneas largas se
desplazan en horizontal, A y B a la vez (`Shift+←` / `Shift+→`, Shift+rueda o una rueda
horizontal), y nunca se parten.

![El editor de merge: A (actual) y B (entrante) en paralelo sobre el Resultado, el conflicto 1 tomado de A y el conflicto 2 de B, el conflicto 3 aún por elegir](../images/merge-editor.png)

## Elegir

Cada conflicto es un bloque enmarcado en A y en B, con una casilla en su cabecera y un `+` /
`✓` delante de cada línea.

| Para... | Tecla | Ratón |
|---|---|---|
| tomar la línea bajo el cursor (lado con foco) | `Space` | clic en la línea |
| tomar todo el bloque del lado con foco | `Enter` | clic en el `[ ]` de su cabecera |
| tomar solo A / solo B para este conflicto | `a` / `b` | clic derecho en el bloque |
| tomar ambos, A y luego B / B y luego A | `c` / `Shift+C` | clic derecho en el bloque |
| no tomar nada | `x` | clic derecho en el bloque |
| intercambiar el orden de dos lados elegidos | `s` | clic en la insignia `1.º` / `2.º` |
| tomar todo A / todo B (todos los conflictos) | `Shift+A` / `Shift+B` | barra de herramientas |
| deshacer la elección de este conflicto (vuelve a sin resolver) | `u` en A / B, `Alt+U` en cualquier sitio | clic derecho en el bloque, **Deshacer la elección** |
| restablecer todo el archivo a sus conflictos originales (pregunta antes si escribiste en el Resultado; `Ctrl+Z` lo recupera) | `Shift+U` en A / B, `Alt+R` en cualquier sitio | **Restablecer** junto a Tomar todo |
| conflicto siguiente / anterior | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` en A / B) | `▼` / `▲`, o clic en una marca de una barra de desplazamiento |
| un número repite un movimiento | `3j`, `2k`, `2n`, `3N` (en A / B) | |
| archivo en conflicto siguiente / anterior | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` en la cabecera |
| mostrar u ocultar el panel Base (ancestro común) | `Alt+D` (`d` en A / B) | `Base` |
| mostrar el archivo entero en A y B (**Archivo entero**) | `Alt+F` (`f` en A / B) | `Archivo entero` |
| dar a A o B más ancho / volver a anchos iguales | `Alt+Right` / `Alt+Left`, `Alt+0` | arrastrar el divisor; doble clic en él |
| desplazar A / B / Base en horizontal | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+rueda, rueda horizontal |
| mostrar u ocultar los paneles Refs y Estado | `F9` | |
| dar foco a A, B, el Resultado | `F6` / `Shift+F6` | clic |
| saltar a un panel, en el conflicto en el que estás | `Alt+1` A, `Alt+2` B, `Alt+3` Resultado, `Alt+4` Base; o `Ctrl+N` y luego `1` / `2` / `3` | |
| pasar al panel vecino (A, Base, B, Resultado), mismo conflicto | `h` / `l` o `←` / `→` en A / B | |
| alternar entre las alturas del Resultado | `z` | arrastrar su título |
| deshacer / rehacer (elecciones y escritura, una sola pila) | `Ctrl+Z` / `Ctrl+Y` | |
| escribir el archivo, dejándolo en conflicto | `Alt+S` | **Guardar** |
| escribir el archivo y marcarlo como resuelto | `Ctrl+S` | **Guardar y marcar resuelto** |
| cerrar | `Esc` | `✕` |

Cada barra de desplazamiento (A / B / Base y el Resultado) lleva una marca por conflicto:
naranja mientras aún necesita una elección, verde una vez tratado, azul para el actual. Un
clic en una marca salta a ese conflicto (una marca bajo el tirador de la barra ya está en
pantalla, así que ahí el clic arrastra el tirador).

**Archivo entero** muestra el archivo completo en A y B con todas las diferencias coloreadas, no solo los conflictos:

![Modo Archivo entero: el archivo completo en A y B, con los cambios coloreados](../images/merge-editor-full-file.png)

El Resultado es un editor de verdad: haz clic en él y escribe, selecciona, corta y pega, busca
con `Ctrl+F`. Una elección sobre un conflicto que editaste a mano sustituye tu edición
(`Ctrl+Z` la recupera). Guardar con un conflicto sin nada elegido, o con una línea que aún
parece un marcador de conflicto, pregunta antes.

### ✦ Sugerir una resolución (IA)

Con la IA activada (Ajustes > IA; consulta [Asistencia de IA](ai.md)), la barra de herramientas
tiene un botón **✦ Sugerir**, el menú de clic derecho de un conflicto tiene *✦ Sugerir resolución (IA)*, y
`Alt+I` y la paleta (`IA: Sugerir resolución de conflicto de merge`) hacen lo mismo. Actúa solo sobre el
conflicto actual y se ejecuta solo cuando lo pides. La respuesta sustituye la parte de ese
conflicto en el Resultado como una sola edición, marcada como editada (`✎`), así que `Ctrl+Z`
devuelve el conflicto a como estaba. Nunca se guarda ni se pone en stage por ti. Mientras se
ejecuta, la fila de ayuda muestra el tiempo transcurrido; `Esc` cancela, y un fallo también
se muestra ahí. Con la IA desactivada, ninguno de estos controles existe.

Tras **Guardar y marcar resuelto**, el archivo queda en stage y vuelve la lista de archivos en
conflicto; abre tú el siguiente archivo. `Ctrl+Z` (o el botón Deshacer)
revierte el último «marcar como resuelto». Cuando no queda ningún archivo, el banner ofrece
**Commit merge** (o **Continuar** para un cherry-pick, un revert o un rebase) y **Abortar
merge**, que pregunta antes: devuelve HEAD, el índice y el árbol de trabajo a como estaban.

Un merge cuyo resultado es igual a HEAD (tomaste nuestro lado en todas partes) no tiene nada
en stage ni ningún archivo modificado, y aun así se puede concluir: **Commit merge** abre el
cuadro de commit con el mensaje del merge, y el commit tiene ambos padres. Del mismo modo, el
botón de commit del compositor está habilitado sin nada en stage mientras hay un merge detenido.

Un cherry-pick o revert vacío (el cambio ya está aplicado) no se puede confirmar: git lo
rechaza, y **Continuar** lo dice y apunta a **Omitir**. **Omitir** (botón del banner, entrada
de la paleta *Git: Omitir cherry-pick o revert*, CLI `conflicts skip`) pregunta antes y luego
termina la operación sin registrar nada; HEAD, el índice y el árbol de trabajo vuelven a HEAD.
Omitir no se puede deshacer (no se pierde nada que no se haya abandonado a propósito); un
merge no tiene nada que omitir.

## Conflictos que no son de texto

Un archivo binario, un puntero de Git LFS, un archivo de más de 8 MiB, un archivo eliminado en
un lado, un resto de renombrado, un enlace simbólico, un cambio de modo o un submódulo no
tienen líneas que elegir. El editor muestra una **tarjeta** con lo que contiene cada lado y un
botón por opción; al elegir, el archivo se resuelve y se pone en stage al instante.

| Tarjeta | Opciones |
|---|---|
| Archivo binario, puntero LFS, demasiado grande | **Quedarse con A** (`a`), **Tomar B** (`b`), **Tomar base** (`o`, cuando hay una), **Abrir en herramienta de merge** (`m`) |
| Eliminado en un lado, modificado en el otro | **Conservar el archivo** (`k`), **Aceptar la eliminación** (`Shift+D`) |
| Resto de renombrado (la ruta existe solo en un lado) | **Conservar el archivo** (`k`), **Eliminar el archivo** (`Shift+D`) |
| Enlace simbólico, submódulo, modo de archivo | **Quedarse con A**, **Tomar B**, **Tomar base** |

Cuando el archivo es una imagen (PNG, JPEG, GIF, WebP, BMP, ICO), los recuadros muestran
además las imágenes A, B y Base, ajustadas y de solo lectura, en una ventana lo bastante
ancha; consulta [Imágenes](image-view.md#en-el-editor-de-merge).

![La tarjeta de un conflicto binario: lo que contienen A, B y la base, y los botones Quedarse con A, Tomar B, Tomar base y Abrir en herramienta de merge](../images/merge-editor-card.png)

`Esc` cierra la tarjeta sin elegir. La herramienta de merge externa es
`general.external_merge_tool` en Ajustes > Diff y editor (`{base}`, `{local}` = A, `{remote}` = B,
`{merged}`; vacío significa `git mergetool`); el archivo se pone en stage cuando la herramienta
termina con éxito.

## Prevención de conflictos

GitPoulpi puede avisarte *antes* de hacer merge. Cuando la rama en la que estás daría conflictos con la rama de la que
partió (su **base**), aparece una franja en la fila del borde superior del Log: **Conflictos con origin/main en 2
archivos**, con un botón **Mostrar archivos**. Ocupa la siguiente fila libre del borde tras los banners de las
operaciones en curso, así que mientras se muestra alguno de ellos puede esperar a que haya sitio. No se hace merge
de nada: el daemon hace un merge en memoria de los dos commits (con libgit2, o con `git merge-tree --write-tree` si
git es 2.38 o posterior; un git más antiguo desactiva la función y deja una línea en el registro), así que tus
archivos, el índice y las refs no se tocan.

- **La base** es la rama de destino de la pull request de la rama cuando GitPoulpi ya la conoce (la caché que
  mantiene la sección de pull requests; para esto no se hace ninguna llamada de red); si no, la rama que cumple los
  patrones de **Ramas base** (Ajustes > Prevención de conflictos) y tiene menos commits desde la bifurcación. Un
  empate lo gana la rama cuya punta está más cerca de la bifurcación, luego una rama de seguimiento remoto sobre una
  local y, al final, el nombre. La propia rama y su copia remota nunca son la base.
- **La ventana emergente** (haz clic en la franja, o usa **Conflictos: Mostrar conflictos previstos** de la paleta,
  que funciona desde el teclado) lista los archivos y tiene **Abrir vista previa del merge** y **Rebase sobre la
  base**. Son las propias filas del menú de la rama base (*Merge <base> en <rama>*, *Rebase <rama> sobre <base>*)
  ejecutadas por ti: quedan en el historial de deshacer, así que `Ctrl+Z` las revierte, y un merge que sí da
  conflictos se detiene en el editor de merge como cualquier otro. Izquierda y Derecha pasan de un botón a otro,
  Enter pulsa el elegido y Esc cierra.
- **Cuándo se ejecuta**: en segundo plano, unos 2 segundos después de que el repositorio se quede en calma (un
  commit, un checkout, un fetch que movió una rama, un rebase), una vez por cada pareja `(HEAD, base)`. La franja se
  oculta en cuanto HEAD se mueve y vuelve, o no, con la respuesta nueva. Un HEAD desacoplado no se comprueba.
- **Desactivarla**: **Activar la detección proactiva de conflictos** en Ajustes > General, o, para un solo
  repositorio, en su página **Prevención de conflictos**. Desde un script, `gitpoulpi conflicts check` imprime la
  misma respuesta como JSON ([referencia de la CLI](cli-reference.md)).

## Agentes

Un agente puede mostrarte un conflicto (`gitpoulpi ui show conflict <path> [--conflict N]`: el
editor se abre sobre él, sin nada elegido) o **proponer** una resolución
(`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` o `--content-file F`, lo que requiere
*Dejar que los agentes de IA propongan cambios para que yo los confirme*). Una propuesta abre
el editor con las elecciones (o todo el texto) ya en el Resultado y un banner morado
*✦ Propuesto por claude-code*. Revísala, cambia lo que quieras y pulsa **Ctrl+S**: solo eso
escribe o pone en stage algo. Cerrar el editor rechaza la propuesta. Consulta
[MCP y agentes](mcp-and-agents.md).

Desde un script, las mismas operaciones son `gitpoulpi conflicts list | show | resolve |
unresolve | state | abort` ([Referencia de la CLI](cli-reference.md)); `conflicts check` los predice antes de un merge.
