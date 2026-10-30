[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · **Español** · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Imágenes en la vista Diff y en la Vista de archivo

Un archivo binario que es una imagen se muestra como imagen, no como el aviso
«(sin cambios de texto - contenido binario, ...)». La vista Diff muestra el **antes** y el
**después**; la Vista de archivo muestra el archivo tal como es; el editor de merge muestra los
lados de una imagen en conflicto. Todo lo de aquí es de solo lectura: mirar una imagen nunca
cambia el repositorio, así que no hay nada que deshacer.

Desactívalo con **Ajustes > Imágenes > Previsualizar imágenes** (`ui.image_view`): vuelve el
aviso binario simple. Una compilación personalizada sin la feature `image-view` no tiene
ningún visor y la página Imágenes conserva solo la lista de editores (consulta
[Compilaciones personalizadas](custom-builds.md)).

## Qué archivos

PNG y JPEG siempre; GIF, WebP (su primer fotograma), BMP e ICO/CUR cuando la compilación tiene
`image-view`. TIFF, AVIF, HEIC, PSD y RAW se reconocen pero no se decodifican: una tarjeta de
información da el formato, el tamaño y las dimensiones. SVG es texto y conserva el diff de
texto. Una imagen de más de **Tamaño máx. de vista previa** (32 MiB por defecto) o de más de
64 megapíxeles muestra una tarjeta de información. Un archivo guardado con Git LFS se muestra
cuando su objeto está en el almacén local; si no, la tarjeta dice que hagas antes `LFS > Pull`.
La transparencia se dibuja sobre un tablero de ajedrez.

## Cómo se dibuja

En un terminal con un protocolo de imagen (Kitty y Ghostty, WezTerm e iTerm2, Rio, mintty) la
imagen es una imagen real. En cualquier otro sitio (Windows Terminal a secas, tmux, un
terminal que GitPoulpi no conoce) se dibuja con caracteres de medio bloque coloreados, con la
mitad de la resolución vertical pero legible en todas partes; **Sin protocolo de imagen** se
puede fijar en *Solo tarjeta de información*. **Protocolo de imagen** en la página
**Log y grafo** fuerza uno.

## La vista Diff

Abre el diff de un archivo de imagen (el de un commit, la mitad unstaged o staged del WIP). La
cabecera nombra ambos archivos (`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), y después van
los botones de zoom, **Ajustar**, **1:1** y **Editar en**.

- **Dividida | En línea** a la derecha de la cabecera, o `m`: en paralelo, o una imagen cada
  vez. **Tab**, o un clic en *Antes | Después*, muestra la otra en la vista en línea. Una vista
  dividida demasiado estrecha para dos paneles se dibuja en línea sin cambiar tu ajuste
  (`ui.diff_layout`).
- El **divisor** de una vista dividida se mueve arrastrándolo o con `Alt+Right` / `Alt+Left`
  (el panel del antes toma / cede espacio) y vuelve al centro con un doble clic o `Alt+0`. La
  proporción se recuerda durante la sesión.
- Un archivo nuevo muestra un solo panel titulado *Archivo nuevo*, uno eliminado *Archivo
  eliminado*; ambos ocultan el selector. Una imagen sin seguimiento seleccionada en los
  cambios de trabajo se previsualiza como archivo nuevo.
- El lado del antes es el commit padre (diff de commit), `HEAD` (staged) o el índice
  (unstaged); el lado del después es el commit, el índice o el archivo en disco. Si el archivo
  en disco cambia mientras el diff está abierto (lo guardaste desde un editor, una herramienta
  lo reescribió), la imagen lo sigue en unos 0,5 segundos.

## La Vista de archivo

Abre una imagen con **Vista de archivo** desde el menú de un archivo o desde la barra del diff:
un solo panel, con el archivo en la revisión (o en disco para los cambios de trabajo).

## Zoom y desplazamiento

| | |
|---|---|
| `+` / `-` (o `=`) | Acercar / alejar, alrededor del centro del panel |
| `0` | **Ajustar** (el valor por defecto): la imagen entera, que sigue ajustada cuando se redimensiona la ventana o el divisor |
| `1` | Tamaño real (1:1) |
| Flechas, `h j k l` | Desplazar una décima parte del panel; con `Shift`, un panel completo; `PageUp`/`PageDown`, `Home`/`End` |
| Rueda | Desplazar en vertical; `Shift`+rueda en horizontal; `Ctrl`+rueda hace zoom alrededor del puntero |
| Arrastrar | Agarra la imagen y muévela; las barras de desplazamiento de los lados funcionan como en todas partes |

Una vista con zoom conserva su escala y el punto de su centro cuando se redimensiona el
panel; Ajustar sigue siendo Ajustar. En la vista Diff, los dos paneles comparten un zoom y
una posición de desplazamiento (**Sincronizar zoom y desplazamiento antes/después**, activado
por defecto), de modo que se ve la misma parte de ambas imágenes; cuando difieren en tamaño,
el zoom se mantiene relativo a cada una. A partir de **Zoom pixelado desde** (2x por defecto) la
imagen se amplía en píxeles cuadrados en lugar de suavizada, lo que va bien para iconos y
pixel art.

## Editar en

El botón **Editar en** de la cabecera lista los editores de imágenes encontrados en este
equipo (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... y la aplicación
predeterminada del sistema); `o` ejecuta el predeterminado y `Shift+O` abre la lista.
También está en el menú de clic derecho de una imagen en las listas de archivos, y en la
paleta de comandos (*Imagen: Editar en...*). La lista y el predeterminado se configuran en
**Ajustes > Imágenes**, igual que funciona [Abrir en](open-in.md).

Editar el archivo en disco (el lado de los cambios de trabajo) es directo: el editor guarda
en su sitio y la imagen se actualiza. Cualquier otra versión (la de un commit, el índice, un
archivo eliminado) se escribe en una **copia temporal de solo lectura**, con un aviso en la
barra de estado, para que un editor no pueda cambiar el repositorio.

Las aplicaciones que el sistema registra para el tipo de la imagen (su lista *Abrir con*) siguen a tus editores en la lista, sin repetir un programa que ya está. Los demás archivos binarios tienen la misma lista en [Abrir en](open-in.md).

## En el editor de merge

Una imagen en conflicto muestra su tarjeta con las imágenes **A**, **B** y (cuando la hay)
**Base** en los recuadros, en paralelo, cada una ajustada a su recuadro y de solo lectura (ahí
no hay zoom; las imágenes solo están para ayudarte a elegir). Los botones y las teclas no
cambian: **Quedarse con A** (`a`), **Tomar B** (`b`), **Tomar base** (`o`), **Abrir en
herramienta de merge** (`m`). Las vistas previas necesitan una ventana lo bastante ancha
(unas 70 columnas) y algunas filas libres; si es más estrecha, la tarjeta es la de texto
simple. Consulta [Resolver conflictos de merge](merge-conflicts.md).

## Teclas y ajustes

Las teclas están en [Atajos de teclado](keybindings.md#vista-de-imagen); todas se pueden
reasignar y se alcanzan desde la paleta (`Ctrl+P`, escribe *Imagen*). Las opciones están en la
página **Imágenes** de [Ajustes](settings.md#imágenes) y en la
[referencia de configuración](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
