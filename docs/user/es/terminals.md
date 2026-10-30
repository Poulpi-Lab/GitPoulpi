[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · **Español** · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Elegir un terminal

GitPoulpi funciona en cualquier terminal con soporte de ratón, pero se ve y se siente
mejor en uno que dibuje imágenes, muestre color verdadero y envíe el teclado completo.
Esta página ordena los terminales por sistema operativo, explica qué ofrece cada uno y
qué ajuste conviene cambiar. Los datos se comprobaron en octubre de 2026 con la
documentación, el changelog o el gestor de incidencias de cada proyecto (consulta
[Fuentes](#fuentes)); nada de esto se probó en todos los terminales. Todo lo marcado como
**(sin verificar)** no pudo confirmarse en una fuente primaria.

## Mejores opciones

| SO | Mejores opciones, por orden | Por qué |
|---|---|---|
| **Windows** | **WezTerm**, luego **Windows Terminal** | WezTerm dibuja el grafo y los avatares como imágenes, con transparencia. Windows Terminal tiene la mejor integración con el SO (es el predeterminado del lanzador); dibuja el grafo y los avatares como imágenes sixel (sin transparencia). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, luego **foot** | kitty y Ghostty ofrecen imágenes y el teclado completo. WezTerm también ofrece imágenes. foot dibuja el grafo y los avatares como imágenes sixel, es rápido y tiene el teclado completo. En Omarchy, elige Ghostty o kitty para tener imágenes. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, luego **iTerm2** | El mismo razonamiento. Terminal.app funciona, pero no tiene imágenes ni color verdadero. |

En cualquier SO, **tmux, zellij y screen** ocultan a GitPoulpi el protocolo de imágenes
del terminal, así que dibuja caracteres (consulta [Multiplexores](#multiplexores)).

## Qué usa GitPoulpi

- **Imágenes** (columna del grafo, avatares de los autores, diff de imágenes y Vista de
  archivo): el **protocolo gráfico de kitty** con marcadores de posición Unicode, o las
  **imágenes en línea de iTerm2** (`OSC 1337`) o **sixel**, por ese orden de preferencia.
  La detección es de denegación por defecto, a partir de `TERM`, `TERM_PROGRAM`,
  `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`,
  `KONSOLE_VERSION` y las variables de los multiplexores, y nunca lee el teclado (así que
  tampoco pregunta al terminal por sus atributos de dispositivo). kitty y Ghostty usan el
  protocolo de kitty. WezTerm usa el protocolo de iTerm2 (a propósito) y mezcla la
  transparencia del PNG. iTerm2, Rio y mintty usan el protocolo de iTerm2 con un fondo
  opaco. Windows Terminal, foot, Konsole y mlterm usan **sixel**. Sin ninguno de ellos,
  los avatares son la insignia con las iniciales.
- **Sixel** se dibuja para la **columna del grafo** (carriles, nodos y, con `ui.avatars = "graph"`, los avatares dentro de los círculos de los commits) y, en el resto, para los **avatares** (Log, detalle del commit, Historial del archivo, Blame, vista de imagen). Un sixel no tiene alfa, ids ni escala, así que la imagen mide exactamente `columnas x ancho de celda` por `filas x alto de celda` píxeles, se dibuja solo donde hay algo, sobre celdas vacías que conservan su fondo; la imagen codificada se guarda en caché, se recodifica solo cuando cambian los carriles, el desplazamiento o el zoom, y se reenvía solo si se repintó una celda bajo ella. Un sixel no se puede borrar: GitPoulpi escribe texto sobre sus celdas para quitarlo. Sixel necesita el tamaño real de la celda en píxeles: a Windows Terminal se le pregunta una vez al arrancar y tras un zoom (`CSI 16 t`, responde en unos 10 ms, nunca bloquea si calla); en otros sitios sale del tamaño de la ventana o de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un terminal que dibuja sixel pero no se reconoce (un terminal de VS Code con `terminal.integrated.enableImages`, un xterm iniciado con `-ti vt340`) necesita `ui.image_protocol = "sixel"`. Windows Terminal anterior a 1.22 no dibuja sixel; pon allí `ui.image_protocol = "none"` (el grafo queda en caracteres).
- **Color verdadero**: los temas son RGB. `ui.color_depth = "auto"` baja a 256 colores en
  un `TERM` `*-256color` sin `COLORTERM=truecolor` (y en Terminal.app), y a 16 colores en
  la consola de Linux. kitty, WezTerm y Windows Terminal cuentan como truecolor.
- **Ratón**: informe de ratón SGR con botones, arrastre y movimiento (respuesta al pasar
  el puntero, arrastrar y soltar, clic derecho, doble clic).
- **Teclado**: el **protocolo de teclado de kitty** (solo la bandera de desambiguación) se
  solicita al arrancar con un tiempo límite de 100 ms. Hace que lleguen `Ctrl+,`,
  `Ctrl+Enter`, `Ctrl+Shift+N` y `Ctrl+Shift+Z`. Sin él se quedan las teclas heredadas
  (`Ctrl+Y` rehace, las variantes con `Alt` sustituyen a algunos acordes). Consulta
  [Atajos de teclado](keybindings.md#teclas-que-el-terminal-no-puede-enviar-y-el-protocolo-de-teclado-de-kitty).
- **Portapapeles**: las acciones de copiar escriben `OSC 52`.
- **Forma del puntero del ratón** (puntero de redimensionar sobre los divisores, mano
  sobre los elementos en los que se puede hacer clic): `OSC 22`, un extra allí donde está
  implementado.
- **Eventos de foco** activados (los tooltips se cierran cuando la ventana pierde el
  foco). GitPoulpi no solicita el pegado entre corchetes (bracketed paste), los
  hipervínculos OSC 8 ni el subrayado ondulado. Solo en la ruta de imágenes, cada cuadro
  va envuelto en una actualización sincronizada (modo 2026, ignorado por los terminales
  que no lo conocen) para que una imagen nunca parpadee sobre un popup.
- **Emoji y símbolos**: GitPoulpi solo usa glifos con presentación emoji por defecto o
  formas BMP de un solo ancho, porque los terminales y las fuentes no se ponen de acuerdo
  sobre los selectores de variación.

## Tabla de capacidades

«Imágenes» es lo que GitPoulpi dibuja hoy en ese terminal (protocolo **kitty**,
**iTerm2** o **sixel**, o **texto**), no lo que el terminal podría hacer.

| Terminal | SO | Imágenes (grafo, avatares, diff de imágenes) | Truecolor | Teclado kitty (acordes completos) | Copiar con OSC 52 | Puntero OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | sí | sí | sí | sí |
| **Ghostty** (1.3) | Linux, macOS (sin versión para Windows) | kitty | sí | sí | sí | sí, desde 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, con transparencia | sí | solo con `enable_kitty_keyboard = true` (por defecto `false`) | sí (solo escritura) | no (la pull request sigue abierta) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafo y avatares; sin gráficos kitty) | sí | sí, desde 1.10.3 | sí (`security.osc52`) | sí, desde 1.12 |
| **Alacritty** (0.17) | todos | texto (sin soporte de imágenes upstream) | sí | sí, desde 0.13 | sí (escritura; pegado desactivado por defecto) | no (no implementado) |
| **Konsole** | Linux | sixel (grafo y avatares) | sí | se está añadiendo en 2026 (merge request !1188); versión (sin verificar) | solo escritura, integrado a mediados de 2024 | (sin verificar) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | texto | sí | no (no está en ningún VTE publicado) | no (VTE lo ignora) | no |
| **iTerm2** (3.7) | macOS | iTerm2, fondo opaco | sí | sí, desde 3.6.9 (revisa Profiles > Keys) | desactivado por defecto: activa «Applications in terminal may access clipboard» | (sin verificar) |
| **Terminal.app** | macOS | texto | no (256 colores) | no | (sin verificar) | (sin verificar) |
| **Windows Terminal** (1.25) | Windows | sixel (grafo y avatares) | sí | sí, estable desde 1.25 (oct. de 2026); Preview desde 1.25 (mar. de 2026) | sí (escritura; solo con el foco) | no (no implementado) |
| **Rio** | Windows, Linux, macOS | iTerm2, fondo opaco (también habla gráficos kitty) | sí | sí (activado por defecto) | sí | no (no implementado) |
| **Warp** | macOS, Linux, Windows | texto (no se detecta) | sí | sí | sí (respeta la secuencia) | (sin verificar) |
| **Terminal de VS Code** (xterm.js) | todos | texto (las imágenes sixel y iTerm2 existen tras `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` usa el sixel) | sí | (sin verificar) | (sin verificar) | no |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, fondo opaco | sí | (sin verificar; no documentado) | (sin verificar; no documentado) | sí |
| **Consola de Windows** (conhost, cmd, ventana de PowerShell) | Windows | texto | solo Windows 10/11 recientes | no hace falta: informa de los modificadores por sí misma | no | no |
| **ConEmu** | Windows | texto | (sin verificar) | (sin verificar) | (sin verificar) | (sin verificar) |
| **tmux, zellij, screen** | todos | texto, siempre | necesita configuración | necesita configuración | necesita configuración | no |

Las celdas «no (no implementado)» de OSC 22 vienen de la ausencia en el changelog y en la
documentación de un proyecto, más informes de usuarios, no de una prueba; léelas como «no
documentado». La página no tabula el ancho de los emoji ni de los selectores de variación:
cambia según el terminal y la fuente, y por eso GitPoulpi evita los glifos VS16 (consulta
[Qué usa GitPoulpi](#qué-usa-gitpoulpi)).

## Terminal por terminal

### Windows

- **WezTerm.** El mejor soporte de imágenes en Windows. Deja `ui.image_protocol` en
  `auto`. En `wezterm.lua`, pon `config.enable_kitty_keyboard = true` para tener los
  acordes completos, y una fuente con emoji de respaldo, por ejemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  La última versión estable etiquetada de WezTerm es 20240203-110809-5046fc22 (febrero de
  2024); las correcciones más nuevas llegan en las versiones nightly. WezTerm todavía no
  tiene forma de puntero OSC 22.
- **Windows Terminal.** Truecolor, ratón, OSC 52 y buenos emoji. GitPoulpi dibuja los
  avatares como imágenes sixel (Windows Terminal 1.22 o posterior); el grafo es de
  caracteres. Elige una Nerd Font en *Settings > Profiles > Defaults > Appearance >
  Font face*. Mantén `Shift` para seleccionar texto mientras el ratón está capturado. La
  forma del puntero no cambia. En una versión anterior a 1.25 (sin protocolo de teclado
  de kitty), usa las variantes con `Alt`.
- **Rio** funciona con un fondo de imagen opaco; menos probado.
- **mintty** (Git Bash, MSYS2, Cygwin) dibuja imágenes con el protocolo de iTerm2 y un
  fondo opaco; menos probado.
- **Consola de Windows, ConEmu, cmd**: evítalos. Las imágenes se dibujan mal o no se
  dibujan. Al hacer doble clic en el exe, GitPoulpi se reabre en el terminal que hayas
  elegido (*Ajustes > General > Lanzador*).

### Linux

- **kitty.** Todo funciona sin ajustes. `Shift+clic` es de kitty, así que usa `Alt+clic`
  para ampliar una selección ([Ratón y gestos](mouse-and-gestures.md)).
- **Ghostty.** Todo funciona sin ajustes. Si los iconos salen como huecos, añade una
  fuente de emoji o una Nerd Font a `font-family` (repite la clave para añadir
  alternativas).
- **WezTerm.** Imágenes con el protocolo de iTerm2, con la transparencia mezclada. La
  misma nota sobre `enable_kitty_keyboard` que en Windows.
- **foot.** Grafo de caracteres, teclado completo, rápido. Los avatares son imágenes
  sixel y el grafo es de caracteres. Pon `font=JetBrainsMono Nerd Font:size=11` en `foot.ini`.
- **Alacritty.** Grafo de caracteres, teclado kitty desde 0.13. Alacritty no tiene
  soporte de imágenes.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Funcionan con caracteres y teclas heredadas:
  usa las variantes con `Alt` y `Ctrl+Y` para rehacer (un Konsole reciente puede añadir el
  protocolo de teclado de kitty). Las acciones de copiar de VTE (`OSC 52`) no ponen nada
  en el portapapeles. Un método de entrada (IBus, Fcitx) puede acaparar el
  `Ctrl+Space` que alterna el stage - **(sin verificar)**; reasígnalo en *Ajustes > Teclado
  y ratón*.
- **Wayland, Hyprland, Omarchy.** La pista del puntero del ratón (`OSC 22`) funciona en
  kitty, Ghostty y foot. La entrada del lanzador de Omarchy pasa por `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: igual que en Linux.
- **iTerm2.** Imágenes con un fondo opaco (un rectángulo visible si tu fondo difiere del
  del tema; `ui.graph_renderer = "text"` lo quita). Activa *Settings > General >
  Selection > Applications in terminal may access clipboard* para copiar, y *Profiles >
  Keys > Left Option key: Esc+* para los acordes con `Alt`.
- **Terminal.app.** Solo 256 colores: GitPoulpi elige él mismo `ui.color_depth = 256` y
  los temas se ven más planos. Sin imágenes. Marca *Use Option as Meta key* para los
  acordes con `Alt`.

### VS Code y Warp

El terminal de VS Code (`TERM_PROGRAM=vscode`) y Warp no se reconocen como terminales con
imágenes, así que GitPoulpi dibuja caracteres. Ambos son utilizables. En VS Code, activa
`terminal.integrated.enableImages` y pon `ui.image_protocol = "sixel"` para tener imágenes
de avatares. Warp documenta el
soporte del protocolo de teclado de kitty; el del terminal de VS Code está **(sin
verificar)**.

## Multiplexores

GitPoulpi detecta tmux, zellij, screen y herdr (`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*`
o `tmux*`, `HERDR_*`) y dibuja **caracteres**, porque un multiplexor reenvía celdas, no
secuencias de imagen. Para el resto de las funciones, en `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Para forzar las imágenes a través de un multiplexor que las reenvíe, pon
`ui.image_protocol` en `kitty` o `iterm2`. herdr con
`kitty_graphics = true` dentro de kitty, Ghostty o WezTerm se detecta automáticamente;
consulta
[Solución de problemas](troubleshooting.md#el-grafo-se-dibuja-con-caracteres-no-como-imagen-herdr-tmux-otros-terminales).
`GITPOULPI_TERMINAL_IMAGE=off` desactiva todas las imágenes.

tmux no reenvía por sí mismo el protocolo de teclado de kitty: `extended-keys` hace que
envíe las teclas con modificadores como secuencias `CSI u` o `xterm`. zellij 0.41 y
posteriores hablan el protocolo de teclado de kitty y reenvían OSC 52, según su
changelog.

## Qué se pierde, y cómo se compensa

| Falta | Qué obtienes en su lugar |
|---|---|
| Protocolo de imágenes | Grafo de caracteres. El avatar pasa a ser la insignia de color con iniciales (una letra) diga lo que diga `ui.avatar_fallback`: un identicon apretado en celdas de texto es irreconocible. El diff de imágenes y la Vista de archivo usan medios bloques de color, o una tarjeta de información. |
| Truecolor | Los colores se asignan a la paleta de 256 o de 16 colores (`ui.color_depth`). |
| Protocolo de teclado de kitty | Teclas heredadas; algunos acordes llegan como otra tecla. Ajustes avisa, y reasignas la tecla o usas las variantes con `Alt`. |
| OSC 52 | Las acciones de copiar no ponen nada en el portapapeles: selecciona con el ratón (mantén `Shift`) y usa la copia del propio terminal. |
| OSC 22 | El puntero no cambia; el divisor o el botón bajo el ratón sigue iluminándose. |
| Fuente de emoji | Cuadros o huecos en lugar de iconos; el grafo y el texto siguen funcionando. Instala una Nerd Font o una fuente de emoji y ponla como alternativa. |

## Ajustes que conviene conocer

| Ajuste | Para qué sirve |
|---|---|
| `ui.image_protocol` | `auto` (por defecto), `kitty`, `iterm2`, `sixel`, `none`. Pon `none` si ves basura donde deberían ir imágenes. Reinicio. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` siempre es correcto. Reinicio. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` u `off`, si un terminal o un multiplexor se comporta mal. Reinicio. |
| `ui.mouse_enabled` | Desactivado deja el ratón al terminal. Reinicio. |
| `ui.launch_terminal` | El terminal que abren los accesos directos del SO ([Ajustes](settings.md)). |

Consulta también [Imágenes](image-view.md), [Atajos de teclado](keybindings.md) y
[Solución de problemas](troubleshooting.md).

## Fuentes

Comprobadas en octubre de 2026. Fuentes primarias salvo que se indique.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado kitty de la [matriz de terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundaria)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundaria), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 de un [resumen del aviso de seguridad sobre OSC 52 en Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundaria)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sin truecolor): [debate de Apple](https://discussions.apple.com/thread/255315820) (secundaria)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sitios de comparación (secundarios, usados solo para contrastar): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Sigue sin verificar: sixel en Konsole y mlterm (tomado de las listas de funciones de los propios proyectos, sin contrastar con una versión y sin probarlo en una ventana real en ningún terminal); OSC 22 en Konsole, iTerm2, Terminal.app y Warp; OSC 52 en
Terminal.app, VS Code y mintty; el protocolo de teclado de kitty en VS Code, mintty y
ConEmu; todas las celdas de ConEmu; y el ancho de los emoji por terminal. No se confirmó
en qué versión incluye Konsole el protocolo de teclado de kitty.
