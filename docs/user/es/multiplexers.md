[English](../multiplexers.md) · **Español**

# Multiplexores y paneles

Un multiplexor se interpone entre GitPoulpi y tu terminal: redibuja las celdas por sí mismo y
solo reenvía las secuencias de escape que entiende. Así, lo que funciona en el terminal solo
([Elegir un terminal](terminals.md)) puede no funcionar dentro de tmux, zellij, screen o herdr.
Esta página dice qué pasa y qué configuración hay que añadir. Los resultados salen de la misma
pasada de pruebas de noviembre de 2026 que las tablas de terminales: tmux 3.8 (MSYS2: 3.7c),
zellij 0.45.1 y 0.46.0, GNU screen 5.0 (macOS trae 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi detecta un multiplexor a partir de `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` o `tmux*`
y `HERDR_*`. Dentro de uno, las imágenes pasan por defecto a caracteres, salvo donde la tabla
de abajo dice que GitPoulpi detecta una ruta que funciona. `GITPOULPI_TERMINAL_IMAGE=off`
desactiva todas las imágenes, y `ui.image_protocol` fuerza un protocolo.

## De un vistazo

| | Imágenes | Forma del puntero (OSC 22) | Portapapeles (OSC 52) | Teclas |
|---|---|---|---|---|
| **tmux** | sixel cuando el terminal de detrás lo admite (detectado); imágenes de kitty con `allow-passthrough on` y `ui.image_protocol = "kitty"` | con `allow-passthrough on`, en kitty y foot; no en Ghostty | con `set-clipboard on` | sin teclado de kitty; `Ctrl+Enter` y `Ctrl+,` nunca llegan |
| **zellij** | caracteres | no | sí | el teclado de kitty funciona |
| **GNU screen** | caracteres | no | no | teclas clásicas |
| **herdr** | imágenes de kitty (detectadas con `kitty_graphics = true`) | no | sí | no medido |
| **psmux** (Windows) | caracteres (a propósito) | no (Windows Terminal no tiene) | el portapapeles del sistema funciona | no medido |
| **tmux de MSYS2** (Windows) | caracteres (a propósito) | no | el portapapeles del sistema funciona | no medido; la pantalla sale confusa |
| **Paneles de WezTerm**, **paneles de Windows Terminal**, **divisiones de kitty y Ghostty**, **paneles divididos de iTerm2** | como el propio terminal | como el terminal | como el terminal | como el terminal |
| **`tmux -CC` de iTerm2** | caracteres | no | no (iTerm2 lo deniega por defecto) | no medido |

## tmux

**Imágenes.**

- **Sixel** lo dibuja el propio tmux, recodificado para un cliente que lo admita. GitPoulpi
  pregunta una vez a tmux al arrancar (`tmux display-message -p '#{client_termfeatures}'`,
  limitado a 400 ms) y usa sixel cuando el cliente lo lista. Ocurre detrás de foot, WezTerm,
  Rio y Tabby; no ocurre detrás de kitty y Ghostty (tmux imprime entonces el texto "SIXEL
  IMAGE"). Necesita un tmux compilado con sixel (los de Arch y Homebrew lo están; compruébalo
  con `tmux -V` y una prueba de sixel). Sin ajustes. La propia respuesta de tmux a una
  consulta de atributos de dispositivo declara sixel sea cual sea el terminal que tiene
  detrás, por eso GitPoulpi pregunta en su lugar por las funciones del cliente.
- **iTerm2 detrás de tmux** anuncia sixel pero pinta opacos los píxeles sin definir, así que
  el grafo, casi transparente, se volvería un rectángulo sólido. GitPoulpi reconoce iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, el tipo de cliente) y dibuja caracteres.
- Las **imágenes de kitty** pasan por el paso directo DCS de tmux (kitty, Ghostty; también
  vistas en iTerm2 y Rio en una sonda en bruto). Necesitan `set -g allow-passthrough on`
  (tmux 3.3a o posterior) **y** `ui.image_protocol = "kitty"` en GitPoulpi, que entonces
  envuelve las subidas y los borrados. Sin la opción de tmux se descarta el envoltorio y el
  grafo queda en blanco.
- Las imágenes en línea de iTerm2 pasan por tmux, pero tmux no puede colocarlas: GitPoulpi
  nunca las usa ahí.

**Forma del puntero.** GitPoulpi envuelve `OSC 22` para el paso directo. Llega a la ventana
en kitty y foot con `allow-passthrough on`. Ghostty no la muestra cuando va envuelta (la
flecha se queda). WezTerm, Rio, Alacritty, Tabby e iTerm2 no tienen `OSC 22` que mostrar.

**Portapapeles.** `set -g set-clipboard on`. GitPoulpi envía también la forma envuelta para
el paso directo (funciona con `allow-passthrough on`). Detrás de kitty, Ghostty, Alacritty y
Tabby llega; iTerm2 lo deniega hasta que se activa su permiso de portapapeles.

**Teclas.** tmux no responde a la consulta del teclado de kitty, y `extended-keys` no hace
llegar `Ctrl+Enter` ni `Ctrl+,`: medido con tmux 3.8 dentro de kitty, ambos llegan como un
simple `Enter` y `,` con `extended-keys` en off, on o always. Los acordes con letra como
`Ctrl+Shift+Z` solo llegan con `extended-keys` en on o always **y**
`extended-keys-format csi-u` (tmux 3.5 o posterior) **y** una aplicación que pida las teclas
extendidas; GitPoulpi nunca lo hace, así que el ajuste no cambia nada aquí. Usa las variantes
con `Alt` dentro de tmux
([Atajos de teclado](keybindings.md#teclas-que-el-terminal-no-puede-enviar-y-el-protocolo-de-teclado-de-kitty)).

**Configuración** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a o posterior: formas del puntero, imágenes de kitty forzadas
```

**tmux dentro de WSL.** No medido. Es la compilación de Linux, así que espera lo mismo que
arriba con Windows Terminal como terminal exterior.

## zellij

Imágenes: zellij acepta el protocolo de kitty pero no sus marcadores de posición Unicode, y
no transporta imágenes sixel ni de iTerm2, así que GitPoulpi dibuja caracteres (un veto
deliberado). Forma del puntero: se la traga, así que GitPoulpi no la envía. Portapapeles:
`OSC 52` se reenvía (funciona detrás de kitty, Ghostty, Alacritty, Tabby; iTerm2 y
Terminal.app lo deniegan o lo ignoran). Teclas: zellij responde a la consulta del teclado de
kitty, así que el teclado completo funciona.

zellij 0.46 abre un popup "Welcome to Zellij" en un arranque limpio: pulsa `Esc`. Una sesión
que terminó conserva su nombre y el siguiente arranque con el mismo nombre pregunta qué
hacer. No hace falta ninguna configuración.

## GNU screen

No pasa nada: las imágenes y `OSC 22` se imprimen como caracteres o se descartan, `OSC 52` se
ignora y el teclado es el clásico. GitPoulpi dibuja caracteres. screen 4.00.03 (la versión que
trae macOS) además confunde los colores y los glifos anchos de la interfaz; screen 5 se probó
solo en Linux. Usa tmux o zellij para una sesión utilizable.

## herdr

herdr incrusta un terminal que reemite **imágenes de kitty**. GitPoulpi las usa cuando el
`config.toml` de herdr dice `kitty_graphics = true` (bajo `[terminal]`, o `[experimental]` en
versiones más antiguas) y el terminal que rodea a herdr las dibuja (kitty, Ghostty, WezTerm):
detectado, sin ajustes. Si la configuración está en otro sitio (por ejemplo bajo
`HERDR_HOME`, que GitPoulpi no lee) pon `ui.image_protocol = "kitty"`. Por SSH
(`herdr --remote`) GitPoulpi no puede ver el cliente local, así que dibuja caracteres;
consulta
[Solución de problemas](troubleshooting.md#el-grafo-se-dibuja-con-caracteres-no-como-imagen-herdr-tmux-otros-terminales).
herdr 0.9.5 no reenvía `OSC 22`, así que GitPoulpi no lo envía. `OSC 52` llega detrás de
kitty, Ghostty, Alacritty y Tabby. En la sonda en bruto herdr dibujó imágenes de kitty en
kitty, Ghostty e iTerm2, también con la configuración por defecto. Teclas: no medido.

## psmux y tmux de MSYS2 (Windows)

- **psmux** (Rust, ConPTY, lenguaje de comandos de tmux; `winget install psmux`) define
  `TMUX`, así que GitPoulpi dibuja caracteres ahí a propósito. Dentro de Windows Terminal se
  ejecuta con normalidad y la copia funciona mediante el portapapeles del sistema; el puntero
  no cambia (Windows Terminal no tiene `OSC 22`). No se midieron otros terminales anfitriones.
- **tmux de MSYS2 o Cygwin** también define `TMUX` (caracteres a propósito), y en la prueba la
  pantalla salió confusa: fotogramas obsoletos dibujados unos sobre otros en mintty y en
  Windows Terminal (con y sin `-u`; sin él, los caracteres de cajas se imprimían como
  `qqqq`). tmux no redibuja bien un programa nativo de Windows dentro de un pseudoterminal de
  MSYS2. GitPoulpi no puede arreglarlo: usa psmux, los paneles de Windows Terminal o los de
  WezTerm.
- zellij y screen no tienen versión para Windows. herdr 0.9.5 se instaló pero no se ejecutó.

## Paneles de terminal

Los paneles que pertenecen al propio terminal no son multiplexores para GitPoulpi:

- **Paneles de WezTerm** (`wezterm cli split-pane`, `wezterm connect`, dominios SSH): WezTerm
  lo dibuja todo, así que las imágenes funcionan (grafo y avatares en el panel) y el puntero
  se comporta como en el propio WezTerm (las formas funcionan en una compilación con el PR
  #8092 de upstream de WezTerm; el divisor del panel da el puntero de redimensionar). No
  medido en Linux ni en macOS.
- **Paneles de Windows Terminal** (`wt ; split-pane`): el grafo y los avatares con sixel
  funcionan en el panel, la copia funciona, sin formas de puntero.
- **Divisiones de kitty, Ghostty e iTerm2**: nativas, como el terminal. No medido en macOS
  para kitty y Ghostty (necesitan una GPU).

## `tmux -CC` de iTerm2 (modo de control)

iTerm2 abre una ventana nativa por cada ventana de tmux y mantiene una ventana pasarela
"[tmux bash]". `TMUX` está definido, así que GitPoulpi dibuja caracteres en la ventana nativa.
Las imágenes sí llegan a iTerm2 por la conexión de control (apareció el diálogo "Allow
Terminal-Initiated Display?" con una prueba de `OSC 1337`), pero GitPoulpi no las usa ahí. La
carga de kitty de un `ui.image_protocol = "kitty"` forzado se imprime en la ventana pasarela,
así que déjalo en `auto`. `OSC 22` y `OSC 52` no llegan.

## Ajustes que conviene conocer

| Ajuste | Uso |
|---|---|
| `ui.image_protocol` | `kitty` fuerza las imágenes de kitty a través de un multiplexor que las reenvía (tmux con `allow-passthrough on`, herdr con una configuración que GitPoulpi no lee); `none` para caracteres. |
| `ui.graph_renderer` | `text` siempre es correcto. |
| `ui.kitty_keyboard` | `off` si un multiplexor se porta mal con la consulta del teclado. |

Consulta [Elegir un terminal](terminals.md) para el terminal que hay detrás del multiplexor, y
[Solución de problemas](troubleshooting.md) cuando el grafo se dibuja con caracteres.
