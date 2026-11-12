[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · **Español** · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Elegir un terminal

GitPoulpi funciona en cualquier terminal con soporte de ratón, pero se ve y se siente
mejor en uno que dibuje imágenes, muestre color verdadero y envíe el teclado completo.
Esta página dice qué ofrece cada terminal en cada sistema operativo y qué ajuste conviene
cambiar.

**Cómo leerla.** Las tablas salen de una pasada de pruebas de **noviembre de 2026**: cada
terminal se manejó con secuencias de escape en bruto y, donde pudo ejecutarlo, con el propio
GitPoulpi, en Linux, Windows 11 y macOS 15. Las versiones están
[más abajo](#versiones-probadas). Cada celda es una de estas:

- **funciona**;
- **parcial** (y qué falta);
- **no funciona** (y por qué: el *terminal*, la *capa del SO* o *GitPoulpi*);
- **n/a** (el terminal no existe en ese SO);
- **no medido** (y por qué).

Los multiplexores (tmux, zellij, screen, herdr, psmux, paneles) tienen su propia página:
[Multiplexores](multiplexers.md).

## Mejores opciones

| SO | Mejores opciones, por orden | Por qué |
|---|---|---|
| **Windows** | **Windows Terminal**, luego **WezTerm** y **Rio** | Windows Terminal dibuja el grafo y los avatares como imágenes sixel, bien ancladas, y es el predeterminado del lanzador. WezTerm y Rio los dibujan como imágenes de iTerm2, ancladas correctamente desde la corrección de ConPTY de noviembre de 2026 (consulta [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty y Ghostty ofrecen imágenes, las formas del puntero y el teclado completo. foot dibuja imágenes sixel y tiene el teclado completo y las formas del puntero. |
| **macOS** | **iTerm2**, luego **kitty** y **Ghostty** | iTerm2 3.7.7 dibuja imágenes con el protocolo de kitty. Terminal.app funciona, pero no tiene imágenes ni color verdadero. kitty y Ghostty se comportan como en Linux (no medido en macOS). |

En cualquier SO, **screen** y **zellij** ocultan a GitPoulpi el protocolo de imágenes del
terminal, así que dibuja caracteres; tmux deja pasar sixel. Consulta
[Multiplexores](multiplexers.md).

## Qué usa GitPoulpi

- **Imágenes** (columna del grafo, avatares de los autores, diff de imágenes y Vista de
  archivo): el **protocolo gráfico de kitty** con marcadores de posición Unicode, las
  **imágenes en línea de iTerm2** (`OSC 1337`) o **sixel**. La detección es de denegación
  por defecto, a partir de `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` y las
  variables de los multiplexores. Cuando el entorno no nombra a nadie (un xterm iniciado
  con `-ti vt340`, un terminal de VS Code con imágenes activadas, cualquier terminal al que
  se llega por SSH, donde `TERM_PROGRAM` no se reenvía), deciden las respuestas del propio
  terminal: GitPoulpi pregunta una vez al arrancar (consulta "La consulta de arranque" más
  abajo) y usa el nombre XTVERSION del terminal (kitty, Ghostty, WezTerm, iTerm2, mintty,
  foot) o sixel en sus atributos de dispositivo. Dentro de un multiplexor las respuestas se
  ignoran, porque tmux declara sixel sea cual sea el terminal que tiene detrás.
- **Qué protocolo y dónde.** kitty, Ghostty e iTerm2 3.7.7 o posterior: el protocolo de
  kitty. WezTerm y Rio: el protocolo de iTerm2, mezclando la transparencia del PNG. mintty
  e iTerm2 anterior a 3.7.7: el protocolo de iTerm2 con un fondo opaco. Windows Terminal,
  foot, Tabby, Konsole y mlterm: **sixel**. Todo lo demás: caracteres, y el avatar pasa a
  ser la insignia con las iniciales. `ui.image_protocol` fuerza uno.
- **iTerm2 3.7.7.** Cada imagen en línea `OSC 1337` abre un diálogo modal "Allow
  Terminal-Initiated Display?" que bloquea la ventana hasta que marcas "Remember my
  choice". Las mismas imágenes enviadas con el protocolo de kitty, marcadores de posición
  Unicode incluidos, se dibujan sin preguntar, así que GitPoulpi usa ahí el protocolo de
  kitty. `ui.image_protocol = "iterm2"` sigue dando las imágenes en línea.
- **Sixel** se dibuja para la **columna del grafo** (carriles, nodos y, con
  `ui.avatars = "graph"`, los avatares dentro de los círculos de los commits) y, en el resto,
  para los **avatares** (Log, detalle del commit, Historial del archivo, Blame, la vista de
  imágenes). Un sixel no tiene alfa, ni ids, ni escalado, así que la imagen mide exactamente
  `columnas x ancho de celda` por `filas x alto de celda` píxeles, se dibuja solo donde hay
  algo, sobre celdas en blanco que conservan su fondo, y se vuelve a codificar solo cuando
  cambian los carriles, la posición de desplazamiento o el zoom. Un sixel no se puede
  borrar: GitPoulpi escribe texto sobre sus celdas para quitarlo. Sixel necesita el tamaño
  real de la celda en píxeles: Windows Terminal, Tabby y mintty responden a `CSI 16 t` una
  vez al arrancar y tras un cambio de zoom (sin bloquear nunca si callan); en los demás
  sale del tamaño de ventana del terminal, o de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un
  terminal que dibuja sixel sin decirlo necesita `ui.image_protocol = "sixel"`. Windows
  Terminal anterior a 1.22 no dibuja sixel; pon `ui.image_protocol = "none"` ahí.
- **Color verdadero**: los temas son RGB. `ui.color_depth = "auto"` baja a 256 colores en
  un `TERM` `*-256color` sin `COLORTERM=truecolor` (y en Terminal.app, que no tiene color
  verdadero), y a 16 colores en la consola de Linux.
- **Ratón**: informe de ratón SGR con botones, arrastre y movimiento (resaltado al pasar,
  arrastrar y soltar, clic derecho, doble clic).
- **La consulta de arranque.** Salen cuatro preguntas en una sola escritura: las banderas
  del teclado de kitty (`CSI ? u`), el color de fondo (`OSC 11`, solo se usa con
  `ui.theme = system`), el nombre del terminal (XTVERSION, `CSI > 0 q`) y sus atributos de
  dispositivo (`CSI c`, la última: todos los terminales la responden, así que cierra la
  lectura). La lectura es un solo bucle de `poll` con un plazo de 100 ms, terminado antes de
  que arranque el lector de teclado. Un terminal que no responde nada cuesta esos 100 ms una
  vez y todo sigue como dice el entorno. Solo Unix: en Windows la consola informa de las
  teclas por sí misma y decide el entorno. Las respuestas están en el informe de entorno del
  Registro de actividad (línea `start-up query:`).
- **Teclado**: el **protocolo de teclado de kitty** (solo la bandera de desambiguación) se
  pide al arrancar. Hace que lleguen `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` y
  `Ctrl+Shift+Z`. Sin él se conservan las teclas clásicas (`Ctrl+Y` rehace, las variantes
  con `Alt` sustituyen a algunos acordes). Consulta
  [Atajos de teclado](keybindings.md#teclas-que-el-terminal-no-puede-enviar-y-el-protocolo-de-teclado-de-kitty).
- **Portapapeles**: toda acción de copia sigue una sola política. En local, el **portapapeles
  del sistema** de la máquina en la que se ejecuta GitPoulpi, más `OSC 52` al terminal cuando
  se sabe que lo respeta (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` es además la alternativa cuando falla el portapapeles del
  sistema. **Por SSH, o sin pantalla** (una consola de Linux, un contenedor), solo se envía
  `OSC 52`, y dentro de tmux se envía envuelto para el paso directo además de sin envolver.
  La línea de estado dice "copiado" solo cuando el portapapeles del sistema tomó el texto;
  lo que fue al terminal dice "enviado ... al portapapeles del terminal (OSC 52, sin
  confirmar)", porque nada confirma que el terminal obedeciera. El informe de entorno del
  Registro de actividad nombra la ruta en su línea `clipboard:`.
- **Forma del puntero del ratón** (puntero de redimensionar sobre los divisores, mano sobre
  los elementos pulsables, cursor de texto sobre los campos de texto): `OSC 22`, un extra donde
  el terminal lo implementa. GitPoulpi siempre lo envía (salvo en la consola de Linux y
  detrás de screen, zellij y herdr, que lo tragan o lo imprimen; detrás de tmux se envuelve
  para el paso directo). Con `TERM_PROGRAM=mintty` envía los nombres de cursor propios de
  mintty (`hand`, `ibeam`, `sizewe`, ...), los únicos que mintty lee. **Rio no implementa
  `OSC 22` en ningún SO**: su código fuente analiza la secuencia y la descarta, así que
  ningún ajuste de un lado ni del otro lo cambia.
- **Cursor de texto**: con `ui.text_cursor = "terminal"` (el valor por defecto) el cursor
  del propio terminal se coloca en el punto de inserción del campo de texto con foco, como una
  barra parpadeante (`DECSCUSR`), y se oculta en el resto; al salir vuelve tu propio estilo de
  cursor. `drawn` dibuja en su lugar el cursor propio de GitPoulpi.
- Los **eventos de foco** están activados (los tooltips se cierran cuando la ventana pierde
  el foco). El **pegado entre corchetes** se pide al arrancar y se libera en todas las
  salidas (Unix), así que un pegado de varias líneas llega como un solo pegado: sus saltos de
  línea nunca envían un campo y sus letras nunca disparan atajos. En Windows la consola no
  tiene corchetes de pegado; consulta [Windows](#windows) para ver qué hace GitPoulpi en su
  lugar. GitPoulpi no pide hipervínculos OSC 8 ni subrayado ondulado. Solo en la ruta de
  imágenes, cada fotograma se envuelve en una actualización sincronizada (modo 2026,
  ignorado por los terminales que no lo conocen) para que una imagen nunca parpadee sobre un
  popup.
- **Emoji y símbolos**: GitPoulpi solo usa glifos con presentación emoji por defecto o formas
  BMP de ancho simple, porque los terminales y las fuentes no se ponen de acuerdo con los
  selectores de variación.

## La matriz

Las filas son terminales, las columnas son el SO. "GitPoulpi" en una celda significa que
GitPoulpi se ejecutó ahí; "sonda" significa que solo se enviaron secuencias de escape en
bruto. Celdas **n/a**: kitty no tiene compilación nativa para Windows (las versiones traen
Linux y macOS); Ghostty no tiene compilación para Windows (solo su biblioteca libghostty es
multiplataforma); foot es un terminal de Wayland (solo Linux); xterm es un cliente de X11
(Linux, y macOS mediante XQuartz); iTerm2 y Terminal.app son solo de macOS; Windows
Terminal, conhost y mintty son solo de Windows (mintty también bajo Cygwin y MSYS2).

**No medido en macOS: kitty, Ghostty, WezTerm, Rio y Warp.** Necesitan una GPU (Metal u
OpenGL 3.3). Sus resultados en Linux son la referencia de lo que hace el mismo programa;
vuelve a probarlos en un Mac con GPU antes de fiarte de una celda de macOS.

### Imágenes: qué protocolo elige GitPoulpi y si se ve

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona**: protocolo de kitty (GitPoulpi) | n/a | no medido (necesita una GPU) |
| **Ghostty** | **funciona**: protocolo de kitty (GitPoulpi) | n/a | no medido (necesita una GPU) |
| **WezTerm** | **funciona**: protocolo de iTerm2 con transparencia (GitPoulpi). Los marcadores de posición se dibujan aplastados, así que no se usa el protocolo de kitty | **funciona**: protocolo de iTerm2 con transparencia, grafo y avatares en sus filas (la versión oficial 20240203 y la compilación del PR #8092 de upstream). La 20240203 oficial ponía las imágenes en filas equivocadas antes de que GitPoulpi mostrara el cursor a su alrededor, consulta "El límite de ConPTY". | no medido (necesita una GPU) |
| **foot** | **funciona**: sixel con transparencia (GitPoulpi) | n/a | n/a |
| **Rio** | **funciona**: protocolo de iTerm2 con transparencia (GitPoulpi) | **funciona**: protocolo de iTerm2 con transparencia, exacto a la celda en todos los tamaños de fuente probados (0.5.28; GitPoulpi). Sixel y kitty nunca llegan (ConPTY los descarta), así que no hay nada que elegir | no medido (necesita una GPU) |
| **Alacritty** | texto: **funciona** (el terminal no admite imágenes) | texto: **parcial** (sin imágenes; el diff de imágenes como bloques) | texto: **funciona** |
| **Tabby** | **funciona**: sixel con transparencia (sonda; GitPoulpi no se ejecutó en Tabby en Linux) | **funciona**: sixel, alineado (GitPoulpi) | **funciona**: sixel, transparencia conservada (GitPoulpi) |
| **Warp** | no medido (necesita XWayland y descodifica mal las pulsaciones sintéticas, así que no se pudo escribir nada) | **no funciona** (terminal: sin imágenes sixel, iTerm2 ni kitty); GitPoulpi no medido (no se pudo arrancar desde el cuadro de entrada de Warp) | no medido (necesita una GPU) |
| **xterm** | sonda: sixel **funciona** con `-ti vt340`; GitPoulpi lo elige a partir de los atributos de dispositivo (la consulta de arranque llegó después de la prueba), o pon `ui.image_protocol = "sixel"` | n/a | texto: **funciona**; sixel se dibuja (sonda) y necesita `ui.image_protocol = "sixel"` |
| **iTerm2** | n/a | n/a | **funciona**: protocolo de kitty en 3.7.7 y posteriores, sin diálogo (GitPoulpi). Las versiones anteriores reciben las imágenes en línea con un fondo opaco. Sixel se dibuja pero pinta opacos los píxeles sin definir, así que `ui.image_protocol = "sixel"` da un rectángulo sólido |
| **Terminal.app** | n/a | n/a | texto: **funciona** (sin imágenes) |
| **Windows Terminal** | n/a | **funciona**: sixel (GitPoulpi, 1.25) | n/a |
| **conhost** | n/a | texto: **parcial** (grafo de caracteres, diff de imágenes con medios bloques) | n/a |
| **mintty** | n/a | **no funciona** (terminal detrás de ConPTY: no se dibuja ninguna imagen, ni siquiera una `OSC 1337` sencilla colocada con el cursor visible; la columna del grafo queda en blanco; el texto y la paleta están bien) | n/a |

### Forma del puntero del ratón (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/a | no medido (necesita una GPU) |
| **Ghostty** | **funciona** | n/a | no medido (necesita una GPU) |
| **WezTerm** | **no funciona** (terminal: no implementado en la compilación probada) | oficial: **no funciona** (terminal). Compilación con el PR #8092 de upstream de WezTerm, que añade formas de puntero: **funciona** (mano, cursor de texto, flechas de redimensionar) | no medido (necesita una GPU) |
| **foot** | **funciona** | n/a | n/a |
| **Rio** | **no funciona** (terminal: se analiza y se descarta) | **no funciona** (terminal, mismo motivo; no es ConPTY) | **no funciona** (terminal, mismo código; probado solo en Linux y Windows) |
| **Alacritty** | **no funciona** (terminal) | **no funciona** (terminal) | **no funciona** (terminal) |
| **Tabby** | **no funciona** (terminal: xterm.js) | **no funciona** (terminal) | **no funciona** (terminal) |
| **Warp** | no medido | **no funciona** (terminal) | no medido |
| **xterm** | no medido | n/a | no medido |
| **iTerm2** | n/a | n/a | **no funciona** (terminal) |
| **Terminal.app** | n/a | n/a | **no funciona** (terminal) |
| **Windows Terminal** | n/a | **no funciona** (terminal) | n/a |
| **conhost** | n/a | **no funciona** (terminal) | n/a |
| **mintty** | n/a | **funciona** con los nombres propios de mintty; GitPoulpi los envía | n/a |

Donde la celda dice "no funciona", el divisor o el botón bajo el ratón se ilumina igualmente.

### Protocolo de teclado

GitPoulpi pide el protocolo de teclado de kitty en Linux y macOS. En Windows no lo pide: la
consola informa de los modificadores por sí misma, y los acordes de abajo están limitados por
lo que el terminal se reserva para sí.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/a | no medido (necesita una GPU) |
| **Ghostty** | **funciona** | n/a | no medido (necesita una GPU) |
| **WezTerm** | **parcial**: no responde hasta que `enable_kitty_keyboard = true` en `wezterm.lua` (por defecto `false`) | **parcial**: WezTerm se queda con `Ctrl+Shift+N` (abre su ventana); `Ctrl+,` no se entrega (compilación oficial). La compilación del PR #8092 no se queda con ninguno de los dos acordes, pero `Ctrl+,` sigue sin abrir Ajustes (no hay respuesta del teclado de kitty a través de ConPTY) | no medido (necesita una GPU) |
| **foot** | **funciona** | n/a | n/a |
| **Rio** | **funciona** (activado por defecto) | **parcial**: Rio se queda con `Ctrl+Shift+N` (abre su ventana) | no medido (necesita una GPU) |
| **Alacritty** | **funciona** (0.13 y posteriores) | **parcial**: `Ctrl+Shift+N` llega como `Ctrl+N`, `Ctrl+,` no se entrega | **funciona** |
| **Tabby** | **no funciona** (terminal: no responde; se queda con `Ctrl+Shift+N` y `Ctrl+,`) | **parcial**: Tabby se queda con `Ctrl+Shift+N` y `Ctrl+,` | **no funciona** (terminal: no responde); algunos acordes no llegaron |
| **Warp** | no medido | no medido | no medido (necesita una GPU) |
| **xterm** | **no funciona** (terminal) | n/a | **no funciona** (terminal) |
| **iTerm2** | n/a | n/a | **funciona** (3.7.7; si no, revisa *Profiles > Keys*) |
| **Terminal.app** | n/a | n/a | **no funciona** (terminal: no responde; teclas clásicas) |
| **Windows Terminal** | n/a | **parcial**: WT se queda con `Ctrl+Shift+N` (ventana nueva) y `Ctrl+,` (sus ajustes) | n/a |
| **conhost** | n/a | no medido | n/a |
| **mintty** | n/a | **parcial**: ninguno de los dos acordes tuvo un efecto visible | n/a |

Donde un acorde no llega, usa las variantes con `Alt`
([Atajos de teclado](keybindings.md#teclas-que-el-terminal-no-puede-enviar-y-el-protocolo-de-teclado-de-kitty)),
o reasigna el acorde en *Ajustes > Teclado y ratón*.

### Portapapeles (acciones de copia)

Una copia usa siempre el portapapeles del sistema cuando hay pantalla local, así que estas
celdas describen la ruta `OSC 52` del terminal, que importa por SSH, en contenedores y como
alternativa. Todas las pruebas de copia de GitPoulpi que usaron el portapapeles del sistema
funcionaron.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/a | no medido (necesita una GPU) |
| **Ghostty** | **funciona** (visto detrás de tmux, zellij, herdr); a secas: no medido (una escritura en el portapapeles necesita el foco del teclado, que la prueba no le dio) | n/a | no medido (necesita una GPU) |
| **WezTerm** | **funciona** | **funciona** | no medido (necesita una GPU) |
| **foot** | no medido (una escritura necesita el foco del teclado, que la prueba no le dio) | n/a | n/a |
| **Rio** | no medido (mismo motivo) | **funciona** | no medido (necesita una GPU) |
| **Alacritty** | no medido (mismo motivo) | **no funciona** (el portapapeles no cambia con la configuración por defecto; sin investigar) | **funciona** |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | no medido | **no funciona** (el portapapeles no cambia) | no medido (necesita una GPU) |
| **xterm** | no medido | n/a | **no funciona** (el portapapeles no cambia) |
| **iTerm2** | n/a | n/a | **no funciona** hasta que se activa *Settings > General > Selection > Applications in terminal may access clipboard* |
| **Terminal.app** | n/a | n/a | **no funciona** (terminal) |
| **Windows Terminal** | n/a | **funciona** (con foco) | n/a |
| **conhost** | n/a | no medido (no se ejecutó ninguna sonda) | n/a |
| **mintty** | n/a | **no funciona** (el portapapeles no cambia con sus ajustes por defecto) | n/a |

### Pegado

En Linux y macOS GitPoulpi pide el pegado entre corchetes, así que un pegado de varias
líneas es un solo pegado. No se probó terminal por terminal (**no medido** por terminal). En
Windows la consola no tiene corchetes de pegado, así que un terminal que no los pide antes
entrega el texto como pulsaciones y cada salto de línea sería un `Enter`. Por eso GitPoulpi
trata las pulsaciones que se suceden en menos de 30 ms (un `Enter` interior, u ocho o más
caracteres) como un solo pegado. Las teclas mantenidas y "abc, `Enter`" escrito a mano no son
pegados.

| Terminal | Windows (pegado de dos líneas en el prompt de rama nueva) |
|---|---|
| **Windows Terminal** | **funciona**: WT pregunta antes "¿pegar varias líneas?" (su propio aviso), y después no se crea ninguna rama |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funciona** (no se crea ninguna rama) |
| **Warp** | no medido (no se pudo arrancar GitPoulpi dentro de él) |
| **conhost** | no medido (`Ctrl+V` y `Shift+Insert` enviados por la prueba no pegaron en la consola clásica) |

### Ancho de emoji e iconos

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funciona** (celdas anchas alineadas; kitty dibuja el engranaje U+2699 en blanco, un asunto conocido de la fuente) | WezTerm, Rio, Tabby, Windows Terminal: **funciona**. Alacritty: **parcial** (emoji como contornos monocromos) | Alacritty, Tabby: **funciona**. Los demás: no medido (necesita una GPU) |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | no medido | **funciona** | no medido |
| **xterm** | **parcial** (cuadros: la fuente) | n/a | **parcial** (cuadros: la fuente) |
| **iTerm2**, **Terminal.app** | n/a | n/a | **funciona** |
| **conhost** | n/a | **parcial**: emoji de color dibujados, celdas anchas alineadas | n/a |
| **mintty** | n/a | **funciona** | n/a |

### Versiones probadas

kitty 0.49.2 (la 0.50.0 de macOS no arrancó), Ghostty 1.4.0, WezTerm 20260716 en Linux y
20240203 (la última versión etiquetada) en Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows)
y 0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) y 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Los terminales cambian deprisa: una versión más nueva que estas puede diferir.

## Terminal por terminal

### Windows

- **Windows Terminal.** El predeterminado del lanzador y el mejor soporte de imágenes en
  Windows: grafo y avatares con sixel, OSC 52, buenos emoji, y responde a la consulta del
  tamaño de celda. Elige una Nerd Font en *Settings > Profiles > Defaults > Appearance >
  Font face*. Mantén `Shift` para seleccionar texto mientras el ratón está capturado. Se
  queda con `Ctrl+Shift+N` y `Ctrl+,`: usa las variantes con `Alt` o reasigna. La forma del
  puntero no cambia.
- **WezTerm.** En `wezterm.lua`, pon `config.enable_kitty_keyboard = true` (compilaciones de
  Linux y macOS) y una fuente con emoji de respaldo, por ejemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Las imágenes usan el protocolo de iTerm2 y no hay que configurar nada: la última versión
  etiquetada, 20240203-110809 (febrero de 2024), las ancla en las filas correctas
  (comprobado de nuevo en noviembre de 2026 con el GitPoulpi actual; antes de la corrección
  de ConPTY descrita más abajo no lo hacía). El PR #8092 de upstream de WezTerm añade formas
  de puntero.
- **Rio** (MSI, pide UAC). Las imágenes (grafo, avatares, diff de imágenes) usan el
  protocolo de iTerm2 con transparencia y caen exactamente en sus celdas: medido en 0.5.28
  con la fuente por defecto y a 18 y 22,5 px (celdas de 8x16, 11x21 y 13x26 píxeles, siempre
  píxeles enteros), una imagen de 20x10 celdas y otra de 4x27 salieron exactas a la celda. No
  hay que configurar nada. La pseudoconsola se traga la consulta del tamaño de celda, así
  que GitPoulpi supone que las celdas de Rio son el doble de altas que anchas (cierto en los
  tres tamaños); con una fuente de otra proporción pon `GITPOULPI_TERMINAL_CELL_PX=WxH` (por
  ejemplo `10x24`) para tener marcadores de commit redondos. Rio no tiene formas de puntero
  en ningún SO.
- **Tabby.** Grafo y avatares con sixel, la copia y el pegado funcionan. Tabby se queda con
  `Ctrl+Shift+N` y `Ctrl+,`.
- **Alacritty.** Necesita OpenGL 3.3. Grafo de caracteres; la copia mediante `OSC 52` no
  estaba activada con la configuración por defecto.
- **mintty** (Git Bash, MSYS2, Cygwin). Las formas de puntero funcionan (los nombres propios
  de mintty). Las imágenes no se ven (la columna del grafo queda en blanco; mintty 3.8.3 no
  dibuja ninguna imagen `OSC 1337` detrás de ConPTY ni siquiera colocada por una sonda
  sencilla) y no responde a ninguna consulta de tamaño de celda ahí: pon
  `ui.graph_renderer = "text"`.
- **Warp.** No medido con GitPoulpi: no se pudo arrancar desde el cuadro de entrada de Warp.
  El terminal no tiene imágenes sixel, iTerm2 ni kitty.
- **conhost, cmd, una ventana de PowerShell.** Grafo de caracteres, diff de imágenes con
  medios bloques. Por defecto, GitPoulpi arrancado desde una ventana así se reabre en el
  terminal que elegiste; consulta "La regla de relanzamiento".

**El límite de ConPTY.** Windows ejecuta el programa de cada terminal detrás de una
pseudoconsola (ConPTY). Las compilaciones de Windows Terminal y de WezTerm que incluyen una
reciente dejan pasar las secuencias de imagen; la que va dentro de Windows (la usan Rio y
mintty) y la que incluye WezTerm 20240203 no. Lo que se pierde: las secuencias DCS (sixel) y
APC (gráficos de kitty) se descartan, así que solo llegan las imágenes de iTerm2
(`OSC 1337`), por eso Rio y WezTerm las usan; `CSI 16 t` (tamaño de celda) solo se responde
en Windows Terminal y Tabby. Lo que se corrigió: una `OSC 1337` cruza ConPTY al instante y
cae en el cursor que ConPTY pintó por última vez, y ConPTY pinta el cursor solo mientras es
visible. GitPoulpi mantiene el cursor oculto, así que sus imágenes caían en filas obsoletas
(el grafo sobre el pie, una pantalla rota tras desplazar). Ahora muestra el cursor alrededor
de cada imagen que escribe en Windows, dentro de la misma escritura, y lo vuelve a ocultar;
Rio y WezTerm 20240203 anclan cada imagen en su celda (`pty_conpty_picture_anchor` lee de
vuelta el movimiento del cursor detrás de ConPTY). Nada cambia fuera de Windows ni para
sixel.

**Pegado en Windows.** Consulta la tabla "Pegado" de arriba: GitPoulpi reúne una ráfaga de
pulsaciones en un solo pegado, así que un pegado de varias líneas nunca envía un prompt en
los terminales probados.

**La regla de relanzamiento.** Un doble clic en el exe lo arranca en una ventana de consola
clásica (conhost) propia. Cuando `ui.launch_terminal` nombra otro terminal (por defecto:
Windows Terminal), GitPoulpi se reabre ahí y la primera ventana se cierra
(`ui.relaunch_in_terminal`). Lo hace **solo** cuando está solo en una ventana de consola
clásica. Un terminal que ejecuta el exe directamente (`wt.exe gitpoulpi.exe`, WezTerm, Rio,
Alacritty `-e`, Tabby `run`) es donde quieres estar, y no se relanza nada; desde un shell no
está solo, así que tampoco pasa nada. Desactívalo con `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Todo funciona sin ajustes. `Shift+clic` es de kitty, así que usa
  `Alt+clic` para ampliar una selección ([Ratón y gestos](mouse-and-gestures.md)).
- **Ghostty.** Todo funciona sin ajustes. Si los iconos salen como huecos, añade una fuente
  emoji o Nerd Font a `font-family` (repite la clave para añadir alternativas).
- **WezTerm.** Imágenes con el protocolo de iTerm2, transparencia mezclada. Pon
  `enable_kitty_keyboard = true` para tener los acordes completos. Sin formas de puntero.
- **foot.** Grafo y avatares con sixel, formas de puntero, teclado completo, rápido. Pon
  `font=JetBrainsMono Nerd Font:size=11` en `foot.ini`. La copia mediante `OSC 52` la rige
  `security.osc52`.
- **Rio.** Imágenes con el protocolo de iTerm2 con transparencia, teclado completo. Sin
  formas de puntero (Rio descarta `OSC 22`).
- **Alacritty.** Grafo de caracteres, teclado de kitty desde la 0.13. Sin soporte de
  imágenes.
- **Tabby.** Imágenes sixel; sin teclado de kitty, sin formas de puntero (Electron,
  xterm.js). Se queda con `Ctrl+Shift+N` y `Ctrl+,`.
- **xterm.** Arráncalo como `xterm -u8 -ti vt340` para tener sixel (hace falta UTF-8 para
  los caracteres de dibujo de cajas). Cuadros en lugar de emoji: la fuente.
- **Warp.** No medido: necesita XWayland y, en la prueba, descodificó mal las pulsaciones
  sintéticas.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** No forman parte de la pasada de noviembre de
  2026. Funcionan con caracteres y teclas clásicas: usa las variantes con `Alt` y `Ctrl+Y`
  para rehacer. VTE ignora `OSC 52`, así que la copia por SSH no llega al portapapeles. Un
  método de entrada (IBus, Fcitx) puede quedarse con el atajo `Ctrl+Space` de stage; reasígnalo
  en *Ajustes > Teclado y ratón*. Konsole se reconoce para sixel por su propia
  documentación.
- **Wayland, Hyprland, Omarchy.** La pista del puntero (`OSC 22`) funciona en kitty, Ghostty
  y foot. La entrada del lanzador de Omarchy pasa por `xdg-terminal-exec`.

### macOS

- **iTerm2.** La 3.7.7 y las posteriores dibujan imágenes con el protocolo de kitty, sin
  preguntar. Ajustes que ayudan:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    necesario para copiar mediante `OSC 52` (si no, iTerm2 muestra "the terminal attempted
    to access the clipboard but it was denied").
  - *Profiles > Keys > Left Option key: Esc+*: necesario para los acordes con `Alt`.
  - Si pones `ui.image_protocol = "iterm2"` (o usas una versión anterior a 3.7.7), marca
    "Remember my choice" en el diálogo "Allow Terminal-Initiated Display?". Las imágenes
    tienen entonces un fondo opaco (un rectángulo visible si tu fondo difiere del del tema;
    `ui.graph_renderer = "text"` lo elimina).
  - El protocolo de teclado de kitty está activado por defecto (3.6.9 y posteriores).
- **Terminal.app.** Solo 256 colores: GitPoulpi elige `ui.color_depth = 256` por sí mismo y
  los temas se ven más planos. Sin imágenes, sin teclado de kitty, sin portapapeles mediante
  `OSC 52`. Marca *Use Option as Meta key* para los acordes con `Alt`.
- **Alacritty, Tabby.** Alacritty: grafo de caracteres, teclado de kitty, copia con `OSC 52`;
  pon `option_as_alt = "Both"` para los acordes con `Alt`. Tabby: grafo y avatares con sixel.
- **Option como Meta.** Option compone caracteres (Option+B escribe un signo de integral) a
  menos que el terminal la convierta en Meta: iTerm2 *Left Option key: Esc+*, Terminal.app
  *Use Option as Meta key*, Alacritty `option_as_alt`. Los acordes con `Alt` de GitPoulpi lo
  necesitan, y en ninguno está activado por defecto.
- **kitty, Ghostty, WezTerm, Rio, Warp**: no medido en macOS (necesitan una GPU). Espera el
  comportamiento de Linux.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel necesita `ui.image_protocol = "sixel"`.

### VS Code y Warp

El terminal de VS Code (`TERM_PROGRAM=vscode`) no forma parte de la pasada. Se reconoce por
sus atributos de dispositivo cuando `terminal.integrated.enableImages` está activado; si no,
GitPoulpi dibuja caracteres; `ui.image_protocol = "sixel"` fuerza las imágenes. Warp:
consulta las notas por SO de arriba.

## Multiplexores

tmux, zellij, screen, herdr, psmux y los paneles propios de los terminales se interponen
entre GitPoulpi y el terminal y cambian lo que pasa. La versión corta: tmux deja pasar sixel
y, con `allow-passthrough on`, las formas del puntero y las imágenes de kitty; zellij y
screen no dejan pasar ni imágenes ni formas del puntero; herdr deja pasar las imágenes de
kitty. Todos los detalles y la configuración que hay que añadir están en
[Multiplexores](multiplexers.md).

## Qué se pierde, y cómo se compensa

| Falta | Lo que obtienes en su lugar |
|---|---|
| Protocolo de imágenes | Grafo de caracteres. El avatar pasa a ser la insignia de color con las iniciales (una letra) diga lo que diga `ui.avatar_fallback`: un identicon apretado en celdas de texto no se reconoce. El diff de imágenes y la Vista de archivo usan medios bloques de color, o una tarjeta de información. |
| Truecolor | Los colores se ajustan a la paleta de 256 o 16 colores (`ui.color_depth`). |
| Protocolo de teclado de kitty | Teclas clásicas; algunos acordes llegan como otra tecla. Ajustes avisa, y reasignas o usas las variantes con `Alt`. |
| OSC 52 | En local no cambia nada (se usa el portapapeles del sistema). Por SSH o sin pantalla, las acciones de copia no ponen nada en el portapapeles: selecciona con el ratón (mantén `Shift`) y usa la copia del propio terminal. |
| OSC 22 | El puntero no cambia; el divisor o el botón bajo el ratón se ilumina igualmente. |
| Fuente emoji | Cuadros o huecos en lugar de iconos; el grafo y el texto siguen funcionando. Instala una Nerd Font o una fuente emoji y ponla como alternativa. |

## Ajustes que conviene conocer

| Ajuste | Uso |
|---|---|
| `ui.image_protocol` | `auto` (por defecto), `kitty`, `iterm2`, `sixel`, `none`. Pon `none` si ves basura donde deberían estar las imágenes. Reinicio. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` siempre es correcto (úsalo donde se aplica el límite de ConPTY). Reinicio. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` o `off`, si un terminal o un multiplexor se porta mal. Reinicio. |
| `ui.text_cursor` | `terminal` (por defecto) o `drawn`. |
| `ui.mouse_enabled` | Desactivado deja el ratón al terminal. Reinicio. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | El terminal que abren los accesos directos del SO, y si un doble clic reabre GitPoulpi ahí ([Ajustes](settings.md)). |

Consulta también [Multiplexores](multiplexers.md), [Imágenes](image-view.md),
[Atajos de teclado](keybindings.md) y [Solución de problemas](troubleshooting.md).

## Fuentes

Documentación comprobada en octubre de 2026, comportamiento probado en noviembre de 2026. Fuentes primarias salvo que se indique.


- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado de kitty de la [matriz de terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundaria)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundaria), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (sin fila de `OSC 22`), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 de un [resumen del aviso de OSC 52 de Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundaria)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sin truecolor): [debate de Apple](https://discussions.apple.com/thread/255315820) (secundaria)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sitios de comparación (secundarios, usados solo para contrastar): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
