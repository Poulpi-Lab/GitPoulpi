[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · **Español** · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Ratón y gestos

Todo lo que puedes hacer con el ratón tiene también un equivalente de teclado.
Consulta [Atajos de teclado](keybindings.md). Los nombres de las zonas están en el
[recorrido por la interfaz](interface-tour.md).

## Reglas generales

- **Clic** en una fila para seleccionarla. Funciona en cada panel, lista, menú y
  diálogo.
- **Doble clic** en una fila para abrirla o ejecutarla. Un doble clic en una rama
  local, un tag o un chip de ref hace checkout. Un doble clic en una fila de
  repositorio abre el repositorio.
- **Clic derecho** en una fila para ver su menú. `.`, `Shift+F10` o la tecla Menú
  abren el mismo menú para la fila seleccionada.
- **La rueda** desplaza la zona bajo el puntero. En una lista, solo mueve la vista.
  La selección se queda donde está, incluso cuando sale de la vista. Solo un clic
  o una tecla mueve la selección.
- **Shift+rueda** desplaza en horizontal, en el diff, la vista de archivo y el
  editor cuando el ajuste de línea está desactivado.
- **Pasa el puntero** sobre un botón, una fila o el pulgar de una barra de
  desplazamiento para iluminarlo. Muchos terminales no pueden cambiar el puntero
  del ratón, así que esa luz suele ser la única señal de que algo se puede pulsar.
  Deja el puntero quieto sobre un botón o una celda del Log para ver un tooltip.
- **Clic fuera** de un campo de texto para salir de él. Lo que escribiste se
  conserva. El filtro de Refs aplica lo que escribiste, como hace `Enter`. La
  barra de búsqueda del Log se cierra, como hace `Esc`.

## Cerrar ventanas y diálogos

Toda ventana o diálogo con marco lleva una `✕` en la esquina superior derecha de su
borde, y las mismas reglas valen en todas partes:

- **Haz clic en la `✕`** para cerrarlo. `Esc` hace lo mismo, y cierra primero lo más
  interno (un desplegable abierto, un campo de texto, una confirmación) antes que la
  ventana.
- **Haz clic fuera** de un diálogo, de un panel de ayuda o de información o de una
  ventana emergente para cerrarlo. Un clic fuera nunca cierra una vista completa
  (Ajustes, la vista de diff, la vista de archivo, los editores, Review, Board):
  no tienen un fuera.
- Un clic **dentro** de un panel nunca lo cierra.
- Una pulsación del ratón (izquierda o derecha) mientras un acorde `Ctrl+G` o `Ctrl+N` espera su segunda tecla cancela el acorde como `Esc` y después actúa como siempre (un clic sigue seleccionando, un clic derecho sigue abriendo su menú).
- Un formulario que contiene texto que escribiste (un issue nuevo, una pull o merge
  request) ignora un clic fuera, para que un clic perdido no te haga perderlo. La
  `✕` y `Esc` preguntan antes **¿Descartar los cambios?**; **Seguir editando**
  vuelve atrás. Un formulario sin nada escrito se cierra al instante, y también un
  diálogo de una sola línea.
- La paleta de comandos, los selectores (pestaña, tema, idioma), los diálogos
  pequeños de revisión, tablero, fusión y editor de commits y el aviso de
  confirmación también llevan la `✕`. Los menús contextuales y los desplegables
  no la tienen: se cierran con `Esc` o con un clic fuera.

## Listas en Ajustes

Las listas de Ajustes (Perfiles, perfiles de IA, Flujo de trabajo, Abrir en,
Imágenes) responden al ratón así. **Haz clic en una subpestaña** (General o la lista) para pasar a ella; `Alt+Left` / `Alt+Right` hace lo mismo.

- **Clic** en una fila para seleccionarla, **doble clic** para pasar a sus campos,
  **rueda** para desplazar sin seleccionar.
- **Pulsa una fila y arrástrala** hacia arriba o abajo para reordenar: una barra
  muestra dónde quedará, al soltar se coloca y `Esc` cancela. El equivalente de
  teclado es `Alt+Up` / `Alt+Down`.
- **Haz clic en `+ Añadir`, `Duplicar`, `Quitar`, `↑`, `↓`** bajo la lista. Teclas:
  `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; véase
  [Ajustes](settings.md#listas).

## Listas en ventanas emergentes

Estas reglas se aplican a toda lista de una ventana emergente que tenga un campo de
filtro: la paleta de comandos, sus selectores (tema, idioma, colores del grafo), el
selector de pestañas, los desplegables con búsqueda, la lista de referencias de
issues y la ayuda.

- **Clic** en una fila para seleccionarla.
- **Doble clic** en una fila para ejecutarla o elegirla, como hace `Enter`.
- **La rueda** desplaza la lista. No mueve la selección.
- Cuando la lista es más larga que la ventana emergente, aparece una **barra de
  desplazamiento** en su borde derecho. Arrastra el pulgar o haz clic en el carril.

Un desplegable corto sin campo de filtro elige la fila con el primer clic. Un menú
contextual ejecuta la fila con el primer clic. Las teclas de estas listas están en
[Atajos de teclado](keybindings.md#listas-emergentes-filtrables).

## Barras de desplazamiento

Toda zona que se puede desplazar muestra una barra de desplazamiento fina en su
borde cuando su contenido no cabe: el Log, las secciones de Refs, las listas de
archivos, el diff, la vista de archivo, el editor y las listas de las ventanas
emergentes.

- **Arrastra el pulgar** para desplazar.
- **Haz clic en el carril** para traer el pulgar bajo el puntero: la vista salta
  allí en proporción, y si mantienes pulsado el botón sigues arrastrando.
- **Mantén pulsado el botón en una flecha de extremo** para seguir desplazando.
  Empieza tras un breve retraso. Aparta el puntero de la flecha para pausar.
- Una barra de desplazamiento solo mueve la vista. Nunca mueve la selección.

En el diff y en el editor, la barra de desplazamiento vertical muestra además todo
el archivo en pequeñas marcas: verde para las líneas añadidas, rojo para las
eliminadas, ámbar para las modificadas, `◆` para la línea del cursor y `▌` para
cada otro lugar donde aparece el texto seleccionado. El editor de merge marca cada
conflicto en sus barras de desplazamiento. Haz clic en una marca para ir allí.

## Fallos y Registro de actividad

- **Haz clic en el error rojo** de la barra de estado para abrir el Registro de
  actividad en su entrada; **haz clic en Copiar / el botón sugerido** (Reintentar,
  Pull primero, Stash y reintentar, Abrir Integraciones) de la entrada.
- **Haz clic en Actividad** en el extremo derecho de la barra de estado para abrir
  el Registro de actividad. Haz clic en una pestaña para cambiar, en la casilla
  de un nivel (o en **Todo** / **Nada**) para mostrar u ocultar esas líneas, en
  una entrada de Operaciones de git para abrir sus detalles, y en **Vaciar registro**
  en la pestaña Aplicación (pregunta antes). Desplaza las líneas largas
  hacia los lados con `Shift` + rueda o con la barra de desplazamiento
  horizontal. Consulta [El Registro de actividad](troubleshooting.md#el-registro-de-actividad).
- **Clic en `[x]`** del indicador de trabajos a la derecha de la barra de estado para cancelar el trabajo en curso, **clic en su etiqueta** para abrir el panel de trabajos. El botón **Actividad** de la esquina inferior derecha de la pestaña Inicio (o `Ctrl+N Shift+G`) abre el Registro de actividad en Todos los repositorios (su cuadro Alcance, un desplegable bajo las pestañas, elige un repositorio o un espacio de trabajo).

## Cambiar de tamaño

- **Arrastra un divisor** entre la barra lateral Refs, el Log y el panel derecho. El teclado los mueve un paso cada vez: `Ctrl+N Left` / `Ctrl+N Right` mueven el divisor Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` el divisor Log | panel derecho (el divisor va hacia donde apunta la flecha).
  Arrástralo más allá del ancho mínimo para contraer ese panel. Los equivalentes con el teclado son `Ctrl+N Left` / `Ctrl+N Right` (el divisor entre el panel Refs y el Log) y `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (el divisor entre el Log y el panel derecho), un paso cada uno, y el divisor va hacia donde apunta la flecha; la paleta los tiene como `Navegación: Ensanchar el panel Refs`, `Navegación: Estrechar el panel Refs`, `Navegación: Ensanchar el panel Estado` y `Navegación: Estrechar el panel Estado`, y `Navegación: Contraer o expandir el panel Refs` / `Navegación: Contraer o expandir el panel Estado`.
- **Arrastra el borde inferior** de una sección de Refs para cambiar su altura.
- **Arrastra el borde inferior** del bloque del mensaje del commit en el panel
  derecho.
- **Arrastra el borde entre dos cabeceras del Log** (por defecto el hueco, desde la cabecera o desde cualquier punto de la tabla; Ajustes > Log y grafo > Redimensionado de columnas) para cambiar el tamaño de una
  columna (con el teclado: `Shift+C` para elegir una columna y luego `]` / `[`).
- **Haz doble clic en el borde entre dos cabeceras del Log** para ajustar la columna de su izquierda a su contenido (con el teclado: `Shift+C` para elegir una columna y luego `=`; `+` ajusta todas las columnas). La última columna tiene un agarre en su propia última celda: arrástralo para cambiar el tamaño de esa columna, y la columna Mensaje absorbe la diferencia.
- **Desplaza el Log lateralmente** (la disposición de columnas `scroll`; Ajustes > Log y grafo > Disposición de columnas, o la casilla `Desplazar lateralmente` del selector): la rueda horizontal o `Shift`+rueda sobre la tabla, la barra del borde inferior (arrastra su pulgar, pulsa la pista vacía, haz clic en sus flechas de los extremos), o `Shift+Izquierda` / `Shift+Derecha`. En la disposición `fit` por defecto las columnas llenan el panel y no hay nada que desplazar.
- **Desplaza lateralmente un panel de lista** cuando una fila es más ancha que el panel: una sección de Refs (un nombre largo de rama, tag o stash), las listas Unstaged y Staged, los archivos modificados del commit (o de una comparación) y las líneas de información de los detalles del commit (id, autor, fecha de autoría, recuentos). La rueda horizontal o `Shift`+rueda sobre el panel lo desplaza; una barra aparece en el borde inferior del panel solo mientras algo desborda (arrastra su tirador, pulsa la pista vacía, haz clic en sus flechas de los extremos), así que nunca ocupa una fila. Con el teclado, `Alt+Left` / `Alt+Right` desplazan el panel que lo tiene (el Log desplaza los detalles del commit; dentro de los archivos modificados, los archivos); la paleta tiene `Ver: Desplazar el panel a la izquierda/derecha`. Se desplaza la fila entera (sangría, marca de estado, chevron y nombre juntos); el título del panel, su borde, las barras de desplazamiento y el botón Stage/Unstage de una fila de Estado bajo el puntero se quedan quietos. Un nombre ya no se corta con puntos suspensivos: su final queda a un desplazamiento (una rama con destinos de push adicionales conserva a la vista su insignia `⇉`, así que su nombre es el que cede). El desplazamiento es por panel, se detiene en la fila más ancha a la vista y se olvida al cambiar de repositorio. El bloque del mensaje del commit ajusta su texto, así que no tiene nada que desplazar. Mientras una sección está desplazada lateralmente, su borde inferior lleva la barra, así que cambia el tamaño de esa sección con sus comandos de teclado o desde una sección que quepa.
- **Arrastra una cabecera del Log** para mover la columna: un fantasma de su nombre sigue al puntero, la cabecera que dejó se atenúa y una barra de acento marca el hueco donde caería. Las columnas se mueven al soltar; `Esc` cancela, y el puntero muestra la mano abierta mientras la llevas (con el teclado: `Shift+C` para elegir una columna y luego `<` / `>`).

GitPoulpi recuerda cada tamaño. Consulta
[Los tamaños se recuerdan](interface-tour.md#los-tamaños-se-recuerdan).

## Barra de título, fila de pestañas y barra de herramientas

- **Haz clic en <img src="../../../assets/logo/gitpoulpi.png" alt="Icono de GitPoulpi" height="16">** en la barra de título para abrir **Acerca de**. En Acerca de,
  las pupilas del pulpo siguen al puntero; haz clic en el logo para hacerlo
  girar hasta el siguiente color, clic derecho para ver la lista de colores, y
  haz clic en un enlace para abrirlo.
- **Haz clic en Abrir en ▾** en la barra de título para ver la lista de programas.
  Consulta [Abrir en](open-in.md).
- **Haz clic en la versión** (`v1.0.0`, o `vX.Y.Z disponible`) en la barra de título para abrir la pestaña Notas de la versión.
- **Haz clic en la lupa y en `Búsqueda global`** en la barra de título, a la izquierda del engranaje, para
  la [búsqueda global](search.md).
- **Haz clic en el engranaje** de la barra de título, a la izquierda de la ×, para
  abrir los Ajustes.
- **Haz clic en ×** en el extremo derecho de la barra de título para salir.
- **Haz clic en una pestaña** para cambiar a ella. **Haz clic en su `x`** (visible en la
  pestaña activa y en la que está bajo el puntero) para cerrarla. **Arrastra una
  pestaña** hacia un lado para reordenarla. **Haz clic derecho en una pestaña** para
  ver el menú de la pestaña.
- **Haz clic central en una pestaña** para cerrarla, como su `x`.
- **Haz clic en `‹` o `›`** en los extremos de las pestañas (se muestran cuando no caben todas) para desplazar la fila una pestaña; mantén pulsado el botón para seguir desplazando, o usa la rueda sobre la fila. Arrastrar una pestaña sobre una flecha también desplaza la fila. La fila además se desplaza sola para mostrar la pestaña actual cada vez que cambia.
- **Haz clic en el `+`** después de la última pestaña para abrir un menú: Nueva pestaña (`Alt+T`) y, mientras sus pestañas están cerradas, Radar y Espacios de trabajo (`Alt+J`).
- **Haz clic en el icono de la casa** para ir a la pestaña Inicio.
- **Haz clic en la pestaña Radar o Espacios de trabajo** para abrirla; su `x`, un clic central o `Ctrl+W` la cierran. **Haz clic en el chip del espacio de trabajo** (`Alt+W`) para cambiar de espacio de trabajo.
- **Haz clic en Salir y cd aquí** (se muestra a la derecha de las pestañas cuando iniciaste GitPoulpi con el wrapper de shell `gp`) para salir y dejar tu shell en la carpeta de la pestaña activa. Consulta [Seguir la carpeta de la pestaña activa](following-the-tab-folder.md).
- **Haz clic en un botón de la barra de herramientas** para ejecutarlo. **Haz clic
  en `▾`** junto a Pull, Push o LFS para ver su lista. **Haz clic en `⋯`** para ver
  los botones que no cupieron.
- **Haz clic en el combo de perfiles** del extremo derecho de la barra de
  herramientas para elegir la identidad de commit de este repositorio.

## Pestaña Inicio

- **Clic** en una fila para seleccionarla. **Doble clic** para abrir el
  repositorio.
- **Haz clic en la estrella** para añadir o quitar un favorito.
- **Haz clic en Abrir, Ir a, Cerrar o Quitar** en una fila, o en **`⋮`** para ver
  el menú de la fila. **Clic derecho** en una fila para el mismo menú.
- **Haz clic en el título de una sección** para plegarla.
- **Haz clic en Examinar, Clonar, Init, Contraer todo o Expandir todo** arriba, y
  en **Cerrar todas** o **Quitar todos** en el título de una sección.

En el selector de carpetas **Examinar**: haz clic en un lugar o en una carpeta para
ir allí, doble clic en una carpeta para abrirla, haz clic en el campo de ruta para
escribir una ruta. Marca
**Mostrar carpetas hermanas** (`Alt+S`, se recuerda) y el árbol izquierdo también lista las
demás carpetas de cada nivel abierto: las carpetas de la ruta abierta muestran `▾`,
las demás `▸`, y un clic en una va allí. En **Clonar** e **Init**: haz clic en un campo para escribir en
él, y haz clic en un botón o en una casilla para usarlo.

## Barra lateral Refs

- **Clic** en una fila para seleccionarla. **Doble clic** en una rama local o en un
  tag para hacer checkout. **Doble clic** en un worktree o en un submódulo para
  abrirlo en una pestaña. El **+** del encabezado de WORKTREES crea un worktree;
  clic derecho en un worktree para Abrir en una pestaña nueva, Abrir en, Merge y
  Eliminar ([Worktrees](worktrees.md)).
- **Pasa el puntero** sobre una fila para mostrar el icono de su izquierda. **Haz
  clic** en él para ocultar la ref en el Log, o para mostrarla de nuevo. Mientras
  **Solo** está activado, ese mismo lugar pone la ref en solo. En una fila de
  carpeta, el icono actúa sobre todas las refs de la carpeta.
- **Haz clic en Dejar de usar solo** o **Mostrar todo** en el título de la barra
  lateral para limpiar todos los solo o todas las refs ocultas.
- **Clic derecho** en una fila para ver su menú: checkout, merge, fast-forward,
  rebase, crear un worktree, renombrar, eliminar, ocultar, solo y más. La fila de
  un remoto tiene **Fetch**, **Editar**, **Quitar** y otras acciones de remoto. Un
  stash tiene **Pop**, **Aplicar** y **Drop**. Una fila atenuada dice por qué no
  se puede ejecutar.
- **Haz clic en `+`** en el título REMOTAS para añadir un remoto.
- **Haz clic en una fila de carpeta** para plegarla o desplegarla.

### Pull requests e issues

| Gesto | Acción |
|---|---|
| Clic en una fila | La selecciona. |
| Doble clic en una fila | La abre en el navegador. |
| Clic derecho en una fila | Su menú: abrir, copiar la URL o el número, hacer checkout de la rama de origen, crear una rama a partir de un issue. |
| Clic en `+` del título de la sección | Abre el formulario **Crear Pull Request** o el formulario **Nuevo issue**. |
| Clic en la fila de filtro (`Abiertas ▾`) | Elige qué solicitudes o issues se listan. |
| Clic en el título de un grupo | Pliega o despliega el grupo. |
| Clic en la fila del proveedor o del repositorio (ISSUES) | Elige el origen de los issues. **Ninguno (desvincular este repositorio)** lo quita. |
| Clic en la fila de consulta (ISSUES) | Escribe una consulta JQL o WIQL (Jira, Azure DevOps), o elige listas de Trello. |
| Clic en el chip de la barra de herramientas (`GH #12`, `GL !53`, ...) | Abre la pull request en el navegador. Clic derecho para ver **Abrir en el navegador**, **Copiar URL**, **Actualizar** y **Cambiar de perfil**. |
| Clic en el chip tenue `+ PR` | Inicia una pull request para la rama enviada. |

Consulta [Alojamiento y gestores de issues](hosting-integration.md).

## El Log

- **Clic** en un commit para seleccionarlo. Sus detalles aparecen en el panel
  derecho.
- **Ctrl+clic** añade un commit a la selección o lo quita. **Shift+clic** selecciona
  un rango (**Alt+clic** en los terminales que se reservan Shift+clic para su
  propia selección de texto, como kitty y Windows Terminal). Haz clic derecho en
  un commit seleccionado para hacer cherry-pick de todos ellos. La selección puede
  incluir la fila WIP y los stashes; dos o más filas convierten el panel derecho
  en una comparación, consulta [Comparar commits](log-and-history.md#comparar-commits).
- **Haz clic en una tarjeta** del panel de comparación para llevar el cursor del
  Log a esa fila; **haz clic en Intercambiar** para cambiar los extremos Desde y
  Hasta. Un clic simple en una fila sale de la comparación.
- **Haz clic en la fila WIP** para ver los cambios de trabajo. **Haz clic en su
  texto `// WIP`** para escribir ahí el resumen del commit.
- **Pasa el puntero** sobre una celda para ver el texto completo: el mensaje, el
  autor y el correo, la fecha completa, el hash completo.
- **Pasa el puntero sobre el chip `+N`** para ver las demás refs del commit.
  Entra en la ventana emergente para hacer clic en una ref. Doble clic en una ref
  de la ventana emergente para hacer checkout.
- **Doble clic en un chip de ref** para hacer checkout.
- **Clic derecho en un commit** para ver el menú del commit. Tiene un submenú por
  cada ref del commit.
- **Clic derecho en un chip de ref** para ver un menú con las acciones de la
  ref y las acciones del commit.
- **Haz clic en el embudo de la cabecera Autor** (o `Shift+A` en el Log, o `Log: Filtrar por autor...` de la paleta) para elegir los commits de qué autores siguen iluminados; el resto se atenúa como hace una búsqueda. `Enter` o un clic en una fila alterna un autor, la fila **Quitar el filtro** (✕) lo quita, `Esc` cierra el popup y lo conserva. **Clic derecho en el embudo iluminado** para quitarlo (la paleta: `Log: Quitar el filtro de autor`).
- **Clic derecho en la cabecera** para mostrar u ocultar columnas: una lista de casillas (`[x]` / `[ ]`) con una ✕ en su título; haz clic en una fila o usa las flechas y `Espacio` / `Intro`, y ciérrala con la ✕, `Esc` o un clic fuera (la paleta: `Ver: Log: elegir las columnas que se muestran`). Bajo las casillas: `Ajustar <columna> a su contenido` (la columna sobre la que se hizo clic derecho), `Ajustar todas las columnas a su contenido` y `Restablecer los anchos de columna`. El equivalente con el teclado del clic derecho: `Shift+C` para elegir una columna y después `.` (o `Shift+F10`) abre el mismo menú sobre ella.
- **Las columnas del Radar** funcionan igual: arrastra el separador que sigue a la columna de repositorio, `#`, autor o antigüedad para ajustar su ancho, **haz doble clic en él** para ajustar la columna a su entrada más ancha (`C` elige una columna y luego `=`; `+` ajusta todas), y **haz clic derecho en el título de una columna** (o `C` y luego `.`) para `Ajustar <columna> a su contenido` y `Ajustar todas las columnas a su contenido`.
- **Haz clic en `↑`, `↓` o `✕`** en la barra de búsqueda para ir a la coincidencia
  anterior o siguiente, o para cerrar la barra.
- **Haz clic en `#12`, `ABC-12` o `AB#12`** en el mensaje de un commit, en el
  panel derecho, para abrir ese issue o pull request en el navegador.

### Secciones del panel izquierdo

Las **carpetas** (ramas y tags con nombres como `a/b/c`, los archivos del panel derecho en
vista de árbol) se pliegan y se despliegan de golpe: **haz clic derecho en la cabecera o en la parte
vacía de una sección** que tenga carpetas para ver **Expandir todo** / **Contraer todo**
(y **Mostrar u ocultar secciones...**, que lleva a la lista marcable de más abajo), y
**haz clic derecho en una fila de carpeta** para ver **Expandir todas las subcarpetas** / **Contraer todas las
subcarpetas** (la carpeta y todo lo que hay bajo ella). Teclas, igual en el panel Refs, en las listas
Unstaged y Staged y en los archivos cambiados de un commit: `=` expande todo, `-` contrae todo,
`*` (o `Shift+Right`) expande la carpeta bajo el cursor y todo lo que hay debajo, `_` (o `Shift+Left`)
la contrae. Paleta: *Refs: Expandir todas las carpetas*, *Refs: Contraer todas las carpetas*,
*Refs: Expandir la carpeta seleccionada y sus subcarpetas*, *Refs: Contraer la carpeta seleccionada y sus
subcarpetas* y las cuatro gemelas *Archivos:* para la lista de archivos con el foco. Mientras hay un filtro
de nombre activo (el filtro de Refs, *Filtrar archivos* de un commit), cada carpeta que contiene una
coincidencia se muestra abierta, sea cual sea lo que hubieras plegado, y tus propios pliegues ni se cambian
ni se guardan: quita el filtro y vuelven. Un pliegue que haces durante el filtro actúa sobre lo que se
muestra y se olvida con el filtro.

**Haz clic derecho en la cabecera de una sección** (una sección sin carpetas, o la parte vacía del panel izquierdo) para ver una lista
marcable de sus secciones: Locales, Remotas, Worktrees, Stashes, Tags, Pull requests, Issues y Submódulos. Un clic
marca o desmarca una sección; el menú sigue abierto. Una sección oculta no se dibuja y el teclado la salta. La última
sección visible no se puede desmarcar. La elección es por repositorio y se conserva al reiniciar; un repositorio sin elección propia sigue el valor predeterminado (el global `ui.sidebar_sections_hidden`). *Usar en todos los repositorios* convierte la elección actual en el valor predeterminado y *Restablecer* descarta la elección propia de este repositorio (paleta: *Panel izquierdo: Usar estas secciones en todos los repositorios*). Paleta: *Panel izquierdo: Mostrar u ocultar secciones...*, un comando *Mostrar u ocultar* por sección y
*Restablecer las secciones*.

### Arrastrar refs

Pulsa sobre una rama, una rama remota o un tag en la barra lateral Refs, o sobre un chip
de ref en el Log, y arrástrala sobre otra ref. La barra de estado dice lo que llevas. Un
destino que puede recibir lo soltado se ilumina. Suelta para obtener un menú. `Esc`
cancela el arrastre. El Log se queda donde está mientras llevas algo: pulsar una ref
solo desplaza el Log hasta su commit cuando resulta ser un clic (el salto ocurre al
soltar el botón), así que puedes desplazarte primero hasta el destino y luego
coger la ref.

| Arrastre | Menú |
|---|---|
| Rama local X sobre la rama local Y | **Merge X en Y**, **Rebase X sobre Y**, **Rebase interactivo de X sobre Y...**, **Fast-forward de Y a X** |
| Rama local sobre una rama remota | **Push X a Y** (la rama remota de destino) |
| Rama remota sobre la rama local Y | **Merge**, **Rebase Y sobre ella**, **Fast-forward de Y a ella**, y **Pull** cuando es el upstream de Y |
| Tag T sobre cualquier rama, rama remota o tag | **Mover el tag T aquí**; marcado *(no es un fast-forward: T dejaría de ser un ancestro)* cuando el destino no desciende del commit del tag (decides tú); atenuado *ya está ahí* cuando ya está en ese commit |
| Tag T sobre la rama local Y | además **Merge del tag T en Y** y, bajo una línea, el más raro **Merge de Y en el tag T y mover el tag** (un commit de merge del commit del tag e Y, detached, y luego el tag se mueve a él) |

Cuando la acción cambia una rama que no tiene checkout, GitPoulpi hace primero
checkout de ella. Si git rechaza el checkout, no cambia nada. Solo se puede soltar
un tag sobre un tag, y una rama no se puede soltar sobre uno.

Mover un tag conserva su tipo: un tag ligero sigue siendo ligero, un tag anotado se
vuelve a crear anotado con su mensaje y la línea de su autor trasladados sin cambios
(un tag firmado se rechaza, su firma ya no coincidiría). Solo se mueve tu tag local:
un remoto que tiene el tag conserva la posición anterior, el tag pasa a *differs* y
la barra de estado lo dice; haz tú el push cuando lo quieras. El movimiento y el
merge con movimiento son cada uno un paso de deshacer (deshacer devuelve el objeto
tag anterior). Un merge en un tag que da conflictos no cambia nada. Paleta:
`Refs: Mover el Tag seleccionado al Commit seleccionado...` mueve el tag del panel Refs
al commit seleccionado en el Log (pregunta antes cuando no es un
fast-forward); cualquier otro destino pasa por marcar y soltar, más abajo.

Teclado: «marcar y soltar» hace lo mismo que un arrastre, para cualquier par de
refs. En el panel Refs pulsa `m` sobre una rama (paleta: `Refs: Marcar como origen
de arrastre`). La ref muestra un `◆` y aparece subrayada, también en sus etiquetas
del Log, y la barra de estado dice adónde ir. Selecciona otra ref, en cualquier
sección, y pulsa `Shift+M` (paleta: `Refs: Soltar aquí la ref marcada...`): aparece
el mismo menú que abre un arrastre, y sus filas se ejecutan exactamente igual que
tras un arrastre. La marca se gasta en cuanto el menú está abierto; `Esc`, o `m`
sobre la ref marcada, la quita antes. El comando de la paleta `Refs: Soltar la ref
seleccionada sobre la rama activa...` abre el menú para la ref seleccionada y la
rama actual.

## El panel derecho

### Cambios de trabajo

- **Clic** en un archivo para abrir su diff. **Clic de nuevo en el mismo archivo**
  para volver al Log. Si el archivo está abierto en el editor con cambios sin
  guardar, GitPoulpi pregunta antes.
- **Pasa el puntero** sobre una fila para mostrar su botón **Stage** o
  **Unstage**. Una fila de carpeta tiene **Stage carpeta** o **Unstage carpeta**.
- **Haz clic en Stage todo** o **Unstage todo** en el título de una
  sección.
- **Haz clic en el título de una sección** (`Archivos unstaged`, `Archivos
  staged`) para plegarla.
- **Haz clic en Ruta o Árbol** para elegir una lista plana o carpetas. **Haz clic
  en Contraer todo** o **Expandir todo** para plegar todas las carpetas.
- **Haz clic en `✗`** a la izquierda del título para descartar todos los cambios
  (pregunta antes). **Haz clic en `✦`** a la derecha para explicar los cambios
  con IA.
- **Clic derecho en un archivo** para ver su menú: stage, descartar, ignorar, LFS,
  stash del archivo, historial, blame, abrir en un programa externo, copiar la
  ruta, crear un parche, editar, eliminar. Un archivo en conflicto empieza con las
  filas **Resolver con ...**.
- **Clic derecho en una fila de carpeta** para ver el menú de la carpeta: stage,
  descartar, ignorar, stash, crear un parche, abrir la carpeta, crear un archivo
  en ella.
- **Clic derecho en la parte vacía** de Archivos unstaged para ver **Crear
  archivo**.
- **Ignorar**, **Tratar como binario** y **Tratar como texto** (menús de archivo y carpeta) abren primero una lista de los archivos `.gitignore` o `.gitattributes` donde puede ir la regla. Haz clic en una fila y pulsa `Enter` (o usa las flechas); `Esc` o un clic fuera cancela sin escribir nada.
- **Ctrl+clic** en una fila de archivo o de carpeta la añade a la selección o la quita, **Shift+clic** (**Alt+clic**) toma un rango; igual en los archivos modificados de un commit y en Ver todos los archivos. Un clic simple empieza de nuevo. **Clic derecho en una fila seleccionada**: el menú de todos los archivos seleccionados.
- **Clic derecho en la parte vacía** de una lista ofrece también **Abrir la carpeta del repositorio**; Unstaged añade **Stage todo** y **Descartar todos los cambios...**, Staged añade **Unstage todo**.
- **Convertir** (menús de archivo, carpeta y selección) reescribe la codificación, los finales de línea o la sangría de los archivos en disco; un `Ctrl+Z` deshace el lote.

### El formulario de commit

- **Clic** en el Resumen o en la Descripción para escribir en ellos.
- **Haz clic en Modificar el commit anterior** para activar o desactivar amend.
- **Haz clic en el botón verde de commit** (**Commit de cambios en N archivos**)
  para hacer commit. Cuando el botón está atenuado, un clic dice por qué (por ejemplo *Stage de cambios para hacer commit*).
- **Haz clic en `✦`** a la derecha del Resumen para escribir el mensaje con IA.
- **Haz clic en `#▾`** al final del Resumen para ver la lista de issues. Haz clic en
  uno para insertar su referencia.
- **Haz clic en Componer commits con IA** para abrir el compositor de commits.

### Detalles del commit

- **Clic** en un archivo cambiado para abrir su diff. **Clic de nuevo** para volver
  al Log.
- **Arrastra** el borde inferior del mensaje para cambiarle el tamaño. La rueda
  sobre el mensaje desplaza una descripción larga.
- **Clic derecho** en el resumen, la descripción o el bloque del autor para
  *Modificar el resumen...*, *Modificar la descripción...* o *Modificar el autor...*;
  **doble clic** en uno de ellos (o en la fecha de autoría) para abrir el diálogo
  Modificar completo con el cursor en ese campo. Los equivalentes de teclado son
  `Ctrl+G Shift+E` (todos los campos) y `Ctrl+G Shift+A` (el autor); el resumen y la
  descripción están en la paleta.
- **Clic derecho** en un archivo modificado, una fila de carpeta, la parte vacía de la lista o una selección para su menú (abrir, historial, blame, abrir en un programa, copiar rutas, ignorar, tratar como binario o texto, convertir, restaurar desde el commit, eliminar). **Ctrl+clic** y **Shift+clic** seleccionan varios archivos.

Consulta [Stage y commit](staging-and-committing.md).

## El diff y la vista de archivo

- **Haz clic en un botón** de la barra sobre el diff: Ver archivo, Diff, Blame, el
  selector de columna `Archivo │ Todos`, Editar, `↑` `↓`, En línea, Dividida, Hunks,
  EB, Ajuste, Mapa, Codificación. Un botón activado aparece resaltado. Un clic en
  `Archivo` o `Todos` muestra esa lista en la columna de commits; un clic en el segmento
  ya elegido la oculta (consulta
  [La columna de commits](interface-tour.md#la-columna-de-commits)).
- **Haz clic en `✕`** en el título para cerrar el archivo.
- **Pasa el puntero sobre un hunk** para mostrar **Stage hunk** o **Unstage
  hunk**, y **Descartar hunk**.
- **Pasa el puntero sobre una línea cambiada** para mostrar sus propios botones
  **+ Stage**, **− Quitar** y **✗ Descartar** en el extremo derecho de la fila.
- **Clic derecho en una línea** para ver el menú del hunk: stage, unstage o
  descartar el hunk o las líneas seleccionadas, historial del archivo, blame,
  copiar.
- **Haz clic en el minimapa** para mover la vista a esa línea. Mantén pulsado el
  botón y muévete para desplazar.

Cada descarte pregunta antes. Cuando un descarte elimina archivos sin seguimiento, las carpetas que quedan completamente vacías se van con ellos (nunca una que aún contiene un archivo con seguimiento, ignorado u oculto, ni una que ya estaba vacía); Deshacer devuelve los archivos con sus carpetas.

### Blame e historial del archivo

- **Blame.** Deja el puntero sobre una anotación para ver la tarjeta del hunk; un clic selecciona un hunk y un clic derecho abre su menú. Arrastra el borde de la columna de anotaciones para cambiar su ancho (se guarda). Haz clic o arrastra en el minimapa para desplazarte.
- **Historial del archivo.** Un clic selecciona un commit; `Shift+clic` selecciona un rango para comparar; un doble clic muestra el commit en el Log; un clic derecho abre el menú de la fila; arrastra el borde entre la lista y el diff para cambiar su ancho; la ✕ de la lista la oculta (haz clic en la franja `›` para recuperarla); haz clic en `Ver archivo` / `Diff` / `Blame` de la barra para cambiar lo que muestra el lado derecho, y en el selector `Archivo │ Todos` para elegir la lista de la columna. La rueda desplaza la mitad bajo el puntero. Cada uno tiene su equivalente de teclado ([Blame](log-and-history.md#blame), [Historial del archivo](log-and-history.md#historial-del-archivo)).

### La columna de commits (lista Todos)

- **Pasa el puntero sobre una fila** para ver el tooltip del Log (el mensaje completo, el nombre y el correo del autor, la fecha completa, según la parte bajo el puntero).
- **Haz clic derecho en una fila** para ver su menú: mostrarla en el Log, el diff del archivo en ese commit, el archivo en ese commit, su blame, y luego el menú del Log para el commit. La tecla de menú hace lo mismo desde el teclado.
- **Arrastra un límite de la cabecera** (`ui.column_resize`: por defecto la celda separadora entre dos partes, marcada con un `│` atenuado en la cabecera y dibujada como una línea de acento a lo largo de la columna mientras el puntero está sobre ella, y un arrastre desde cualquier punto de ella redimensiona; con `header`, la primera letra del nombre siguiente, iluminada mientras el puntero está sobre ella) para dimensionar la parte que queda a su izquierda: carriles, mensaje, autor, fecha, refs, cambios o SHA. La celda justo después de la última parte es el límite de la última parte.
- **Haz doble clic en un límite de la cabecera** para ajustar la parte de su izquierda a su contenido, como hacen los límites del Log: el ancho que muestra entera su fila más ancha, nunca más ancho que la columna (con el teclado: haz clic en una fila o `Ctrl+N` para dar las teclas a la columna, `Tab` para elegir la parte y después `=`; `+` ajusta todas las partes; la paleta tiene `Ajustar la columna a su contenido` y `Ajustar todas las columnas a su contenido`). El mensaje conserva lo que dejan las demás partes.
- **Arrastra un nombre de la cabecera** sobre otro para mover la parte allí: el mismo fantasma, origen atenuado y barra de acento que la cabecera del Log, y la parte se mueve al soltar (`Esc` cancela; `<` / `>` del teclado, tras `Tab`, hacen lo mismo un lugar cada vez).
- **Clic derecho en la cabecera** para elegir qué partes se muestran (Ref, Grafo, Autor, Fecha, Cambios, SHA; Mensaje siempre se muestra); la lista de casillas tiene una ✕ en su título. Debajo: `Ajustar <parte> a su contenido` (la parte sobre la que se hizo clic derecho) y `Ajustar todas las columnas a su contenido`. La paleta tiene `Elegir las columnas` y una alternancia por parte.
- **Shift+rueda** o una rueda horizontal desplaza las filas hacia los lados cuando son más anchas que la columna.
- **Arrastra el borde derecho** de la columna para cambiar su ancho. **Haz clic en la franja `›`** (o en la ✕ de la columna, o en el **segmento elegido** del selector `Archivo │ Todos` de la barra) para recuperarla u ocultarla.

### Seleccionar y copiar texto

Puedes seleccionar y copiar el texto del diff, de la vista de archivo, del Blame y
del Historial del archivo.

- **Pulsa y arrastra** para seleccionar texto. La selección se queda en el mismo
  texto cuando te desplazas.
- **Doble clic** para seleccionar una palabra.
- **Un clic simple** no selecciona texto. Mueve el cursor o selecciona la fila, y
  borra la selección de texto.
- **`y` o `Ctrl+C`** copian la selección. Sin selección, copian la línea actual.
- **Clic derecho** en el diff para ver **Copiar selección**, **Copiar línea** y
  **Copiar hunk**.

GitPoulpi copia solo el texto del archivo. Los números de línea, las marcas `+` y
`-`, las cabeceras de hunk y las columnas del Blame no se copian. En la vista
Doble, una selección se queda en el lado donde empezaste el arrastre.

En el diff, un arrastre también selecciona las líneas que cruza, como hace `v`.
`Enter` hace entonces stage de esas líneas. `Esc` limpia primero la selección de
texto, luego la selección de líneas y después cierra el diff.

Seleccionar texto también tiñe todos los demás lugares donde aparece ese texto en
el diff y en el editor.

## Campos de texto

Esto se aplica a todos los campos de texto: filtros, la barra de búsqueda, la
paleta, el Resumen y la Descripción, los campos de los diálogos y los valores de
los Ajustes.

- **Clic** para poner el cursor ahí.
- **Pulsa y arrastra** para seleccionar texto.
- **Doble clic** para seleccionar una palabra.
- Escribe para reemplazar la selección. `Backspace` o `Delete` la elimina.

Las teclas de los campos de texto están en [Atajos de teclado](keybindings.md).

## El editor

- **Clic** para poner el cursor ahí. **Shift+clic** amplía la selección.
- **Arrastra** para seleccionar. Arrastra más allá del borde superior o inferior
  para desplazar.
- **Doble clic** para seleccionar una palabra.
- **La rueda** desplaza sin mover el cursor.
- **Haz clic en Guardar, Deshacer, Rehacer o Buscar** en la barra. En el modo de
  búsqueda, **`↑`** y **`↓`** van a la coincidencia anterior y siguiente.
- **Haz clic en `✕`** para cerrar. GitPoulpi pregunta antes cuando hay cambios
  sin guardar.

## Imágenes

Consulta [Imágenes](image-view.md) para el ratón en la vista de imagen.

## Mover y redimensionar diálogos

Los diálogos se pueden mover, y las ventanas más grandes redimensionar:

- **Arrastra la barra de título** para mover un diálogo (la cruz de cierre conserva su clic). La paleta de comandos y los selectores no tienen barra de título aparte: arrastra el borde superior.
- **Arrastra el borde izquierdo, derecho o inferior**, o una esquina inferior (el agarre `◢`), para cambiar el tamaño de una ventana. El diálogo se queda en pantalla y nunca baja de su mínimo (60 x 16 para el Historial de deshacer, menos para los demás). Una ventana redimensionada aprovecha el espacio: las áreas de texto y las listas crecen, el texto se ajusta al nuevo ancho.
- El arrastre continúa cuando el puntero sale del diálogo, y soltar fuera nunca lo cierra.
- **Arrastra los divisores** dentro de la vista Reflog / Trabajo perdido del Historial de deshacer; haz doble clic en uno para restablecerlo.
- Los equivalentes con el teclado son `Ctrl+N` y luego `m` (las flechas mueven, `Shift`+flechas cambian el tamaño, `0` restablece, `Enter` o `Esc` salen) y `Alt`+flechas para los divisores; la paleta tiene `Diálogo: Mover o cambiar de tamaño` y `Diálogo: Restablecer posición y tamaño`. Actúan sobre el diálogo que está encima. `Ctrl+N` funciona igual sobre cualquier superficie: muestra una insignia en los controles de la ventana, el diálogo o el popup en primer plano (campos, listas, pestañas, botones y la cruz de cierre), la tecla que le sigue pulsa ese control, `m` inicia el modo de organizar y `Esc` lo cancela; la línea de estado nombra `<insignia>`, `m=organizar` y `Esc`. Vale también para la paleta de comandos, los selectores, la búsqueda del espacio de trabajo y la superposición de ayuda, donde `Ctrl+N` ya no baja por la lista. `Alt+G` (`palette.arrange`) es una vía más, válida sobre cualquier diálogo y popup, para organizar el que está en primer plano sin el prefijo.
- **Un solo ajuste** (`ui.dialog_geometry`, Ajustes > Apariencia > Disposición) cubre todos los diálogos: cada uno recuerda su propio lugar y tamaño durante esta sesión, en disco para el próximo arranque, o nunca.

| Diálogo | Mover | Redimensionar |
|---|---|---|
| Historial de deshacer / Reflog / Trabajo perdido, Registro de actividad, panel de tareas, selector de carpetas | sí | sí |
| Paleta de comandos, selectores de pestaña / tema / idioma, búsqueda del espacio de trabajo | sí | sí (la lista crece con la altura, las filas con el ancho) |
| Clonar (la lista de repositorios), Pull request y Nuevo issue (la descripción), «Qué se envía» (el panel), Mermaid (la vista previa), opciones de revisión (los campos, solo el ancho) | sí | sí |
| Editores de texto largo de Ajustes (plantillas, hooks, prompts de IA), el cuadro de IA del formulario del espacio de trabajo | sí | sí (el área de texto) |
| Reescritura del historial: los dos diálogos auxiliares de IA | sí | sí (la descripción, el panel de lo enviado) |
| Acerca de, panel de actualización, comprobación de seguridad, conflictos previstos, «Por encima del límite de IA», borrado de tag, archivos sensibles | sí | no (el tamaño sigue al texto) |
| Init, Worktree, Sparse checkout, diálogo de Flow, diálogos del flujo de issues, formulario del mensaje de commit (reword), formulario del espacio de trabajo, selectores de etiqueta / hito | sí | no |
| Confirmaciones (cada «¿Seguro?») | sí | no |
| Diálogos de Revisión y Tablero: sus diálogos pequeños (comentario, terminar, elegir, columnas) | sí | no |
| Popups de la página de atajos (confirmación de restablecer, captura de tecla), el popup de respuesta de la IA | sí | no |
| Ajustes: los resultados y la confirmación de la autenticación del repositorio | sí | no |
| Reescritura del historial: diálogos de purga, identidad y preguntas | sí | no |
| Popup de mover un archivo del editor de commits, avisos de fusión | sí | no |

No se pueden mover: las pantallas de página completa (Ajustes, Ayuda, las páginas de Revisión y Tablero, que ya llenan la ventana), los menús contextuales y los desplegables (cuelgan de lo que los abrió), los tooltips y los popups pequeños dentro de una página (el formulario de remotos, el editor de división de commits).

El puntero muestra una forma de mover sobre la barra de título y una de redimensionar sobre los bordes y las esquinas (terminales con `OSC 22`).

## Forma del puntero del ratón

Algunos terminales dejan que GitPoulpi cambie el puntero del ratón: un puntero de
redimensionar sobre los divisores, sobre un límite de columna del Log, del Radar y de la
columna de commits (también mientras lo arrastras), una mano sobre los elementos
pulsables. Esto funciona en kitty, foot, Ghostty, mintty y una compilación de WezTerm con el
PR de upstream #8092 (la secuencia `OSC 22`). Solo se envía cuando la forma cambia, dentro
de tmux solo envuelta para el paso directo (`allow-passthrough on`; kitty y foot la
muestran, Ghostty no), y nunca en screen, zellij o herdr. El puntero por defecto se
restaura al salir de GitPoulpi. En otros terminales (Rio, Alacritty, Tabby, iTerm2,
Terminal.app, Windows Terminal, xterm), el puntero no cambia; consulta
[Elegir un terminal](terminals.md#forma-del-puntero-del-ratón-osc-22). El divisor o el botón bajo el puntero se
ilumina en todos los terminales.

El mismo tipo de componente da el mismo puntero en toda la aplicación: los botones de una revisión mostrada sobre el Radar (pestañas, la cruz de cierre, los botones de los popups) y los botones propios de la columna Acciones del Radar muestran la mano como cualquier otro botón.

- **Los botones son botones de verdad**, en Ajustes (Integraciones: Editar, Desconectar,
  Quitar, Examinar, Eliminar, los botones de fragmentos del editor de instrucciones...)
  igual que en los diálogos: etiquetas con relleno sobre un fondo sólido que se ilumina bajo el
  puntero, atenuadas cuando están desactivadas, nunca texto entre `[ ]`. Los únicos corchetes que
  quedan son las marcas de las casillas `[x]` / `[ ]`, que son casillas y reciben la mano
  como cualquier otro control.
- **La mano que señala** está en los controles: botones, casillas e interruptores,
  desplegables (en Ajustes, los paneles de issues y de pull requests, el Radar,
  los diálogos), pestañas y sus cruces, los botones `+`, enlaces y referencias
  (`#12`, `!45`), cabeceras de grupo y de carpeta con su flecha de despliegue,
  píldoras de ref, barras de desplazamiento, y los botones de Actividad, tareas
  y versión de la barra de estado. Una casilla o un botón dentro de una fila
  conserva la mano.
- **La flecha** está en las filas. Una fila se ilumina entera cuando el puntero está
  sobre ella (la misma luz en todas las listas) y no es un control por sí sola: las
  filas de los paneles Refs, Worktrees, Stashes y Submódulos, el Log y la columna de
  commits (solo una píldora de ref toma la mano), los márgenes de un diff, los archivos
  modificados, los elementos de pull requests, issues y Radar, y las filas de las
  listas de diálogos y paneles (clonar, selector de carpetas, Workspaces, las
  tarjetas del tablero, el panel de tareas, los selectores de asignado, etiqueta e hito,
  los pasos del rebase, los archivos y commits de la revisión, y las opciones de menús
  y desplegables).
- **El cursor de texto (I-beam)** está en los cuadros de entrada, sobre todo el
  cuadro: los buscadores (búsqueda global, Ajustes, la paleta de comandos, Home),
  los filtros (Refs, archivos, Radar, Log de actividad, el buscador del panel de
  issues), las filas de texto, número y área de Ajustes (también mientras las editas),
  los campos de los diálogos, los cuadros de comentarios de la revisión, el compositor
  de commits, el texto del editor de archivos y el resultado editable del editor de
  merge (sus paneles de solo lectura conservan la flecha, porque arrastrar allí no
  selecciona nada). Una regla cubre también el texto de solo lectura: dondequiera que
  un arrastre del ratón pueda seleccionar texto, el puntero es el I-beam. Es el caso del
  código de un diff (la vista Diff, En línea y Dividida, y el panel de diff de la revisión),
  la Vista de archivo y el código de Blame. Sus números de línea, marcas y cabeceras de
  hunk conservan la flecha, y un botón en una línea conserva la mano. Lo mismo vale para la
  conversación de un issue o de una solicitud (también el Resumen de la revisión y su
  bloque de borrador), el mensaje de los detalles del commit, las notas de versión, el Log
  de actividad (ambas pestañas), el popup de ayuda, la vista previa de Markdown del editor
  de comentarios y los comentarios, notas pendientes y sugerencias dentro del diff de la
  revisión: un arrastre selecciona el texto (un clic simple conserva su significado: un
  enlace se abre, un botón actúa, una entrada del Log de actividad o un hilo se abre o se
  pliega), `Ctrl+C` lo copia, `Esc` o un clic lo suelta. Una selección sigue a su texto
  cuando te desplazas. El mensaje del commit también es editable (el doble clic lo
  enmienda), y todo el bloque se ilumina ligeramente.
- **En los detalles del commit**, las líneas de información son controles: haz clic
  en `commit: <id>` para copiar el id corto, doble clic en la línea del autor o de la
  fecha para enmendarla (la mano y una marca ligera lo indican). La línea de
  estadísticas solo informa y conserva la flecha.

Los punteros de redimensionar permanecen sobre los divisores y los límites de columna,
y la mano abierta solo aparece mientras llevas una ref.

## Copiar al portapapeles

Las acciones de copia ponen el texto en el portapapeles del sistema de la máquina en la que
se ejecuta GitPoulpi y, donde se sabe que el terminal lo respeta, lo envían además al terminal
(OSC 52). Por SSH, o sin pantalla, solo se envía OSC 52: si tu terminal no lo admite
(iTerm2 hasta que activa su permiso de portapapeles, Terminal.app, GNU screen),
no se copia nada y la línea de estado dice que el texto solo se envió
al terminal. Consulta [Elegir un terminal](terminals.md#portapapeles-acciones-de-copia).
