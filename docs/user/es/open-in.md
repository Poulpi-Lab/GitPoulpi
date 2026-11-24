[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · **Español** · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Abrir en

El botón **Abrir en ▾** de la barra de título, junto a la ruta del repositorio, abre la carpeta
de la pestaña activa en otro sitio: tu terminal, una pestaña o panel nuevo de un multiplexor,
un editor, el gestor de archivos. Tú decides qué hay en la lista. No hay nada fijo: cada
entrada es una etiqueta y un comando.

## Cómo se usa

- **Clic en Abrir en ▾**: se despliega una lista. Haz clic en una entrada para ejecutarla. La
  entrada predeterminada está marcada con `•`; la última fila, **Configurar…**, abre
  [Ajustes](settings.md#abrir-en) en la página Abrir en.

  ![La lista Abrir en: Terminal, dos editores, una herramienta Git y el gestor de archivos, con la predeterminada marcada con un punto y Configurar al final](../images/open-in-menu.png)

- **Teclado**: `Alt+O` abre la misma lista (`Up`/`Down`, `Enter`, `Esc`; con más de siete
  entradas, escribe para buscar). `Ctrl+Alt+O` ejecuta de inmediato la entrada
  predeterminada. Paleta (`Ctrl+P`): `Pestaña: Abrir en…` y `Pestaña: Abrir en el predeterminado`.
  Los comandos de la paleta funcionan incluso cuando la barra de título no tiene sitio para el botón.
  Ambas teclas se pueden reasignar; consulta [Atajos de teclado](keybindings.md).
- **Por etiqueta**: un paso `tab_open_in` de una [secuencia](config-reference.md#sequences) ejecuta la entrada
  cuya **etiqueta** nombras (`label = "VS Code"`), así que reordenar la lista nunca cambia lo que
  ejecuta. Una etiqueta desconocida detiene el paso con «ninguna entrada de Abrir en se llama ...».
  `Pestaña: Terminal aquí` (paleta) abre tu terminal en la carpeta, el antiguo botón
  «Terminal aquí» de la barra de pestañas.
- **Un agente de IA en la carpeta**: `Pestaña: Abrir agente de IA aquí` abre tu terminal en la
  carpeta de la pestaña ejecutando un agente de IA local; consulta [IA](ai.md#abrir-el-agente-en-la-carpeta-de-una-pestaña).
- **Clic derecho en una pestaña de repositorio**: el menú de la pestaña tiene un submenú
  **Abrir en** con las mismas entradas.
- La lista empieza en la última entrada que usaste en esta sesión.
- La barra de estado muestra al instante «abriendo en ...». Si el programa falla en menos de
  1,5 segundos (por ejemplo tmux sin servidor, o un parámetro incorrecto), la primera línea de
  su salida de error aparece en la barra de estado. Si el programa no está instalado, se
  indica su nombre de inmediato.

Mientras no guardes tu propia lista, la entrada predeterminada se elige según lo que se detecte: dentro de herdr, «Herdr: new tab»; en Omarchy, la ventana nueva de kitty (kitty es su terminal); en cualquier otro caso, el gestor de archivos.

El botón aparece en todas las pestañas de repositorio, incluida la carpeta en la que se inició
GitPoulpi, y en el modo `ui.tab_dir_follow = "auto"`. En el modo `auto`, la entrada
predeterminada también se ejecuta sola al abrir una pestaña nueva (no para las pestañas
restauradas al arrancar). Consulta [Seguir la carpeta de la pestaña activa](following-the-tab-folder.md).

## Cómo es una entrada

En `config.toml`, una tabla `[[ui.open_in]]` por entrada:

```toml
[ui]
open_in_default = "Herdr: new tab"   # una etiqueta; vacío significa la primera entrada

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Rara vez tendrás que escribirlo a mano: la página de Ajustes lo edita. Todas las claves están
en la [referencia de configuración](config-reference.md).

### Comandos y variables

El comando se divide primero en argumentos (los espacios, `'...'` y `"..."` agrupan palabras)
y después se sustituyen las variables dentro de cada argumento. Así, una carpeta con un
espacio en el nombre sigue siendo un solo argumento. **No interviene ningún shell**, salvo que
tu comando inicie uno.

| Variable | Se sustituye por |
|---|---|
| `${cwd}` | La carpeta de la pestaña. |
| `${repo}` | La raíz del repositorio (difiere de `${cwd}` en una pestaña de worktree). |
| `${name}` | El nombre base de la carpeta. |
| `${branch}` | La rama activa; vacía con HEAD desacoplado. |
| `${web_url}` | La página web del remoto origin si es GitHub o GitLab; vacía en otro caso. |
| `${env:NAME}` | La variable de entorno `NAME`; vacía si no está definida. |

La antigua `{repo}` sigue funcionando y significa `${cwd}`. Cualquier otro `${...}` es un
error que nombra la variable.

El programa se inicia con la carpeta de la pestaña como directorio de trabajo y recibe tres
variables de entorno, útiles para un fragmento de shell:

| Variable | Valor |
|---|---|
| `GITPOULPI_CWD` | La carpeta de la pestaña. |
| `GITPOULPI_REPO` | La raíz del repositorio. |
| `GITPOULPI_BRANCH` | La rama, o vacío. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

En Windows el programa se busca mediante `PATH` y `PATHEXT`, así que `code` y `cursor`
(archivos `.cmd`) funcionan y no aparece ninguna ventana de consola.

## Ajustes predefinidos

Ajustes > Abrir en > **Añadir ajuste predefinido** lista entradas ya preparadas solo para tu
sistema operativo (los de Windows nunca aparecen en Linux, y así sucesivamente), por
categoría: la fila empieza con la categoría, `Explorer: ...`, `Multiplexer: ...`,
`Terminal: ...`, `Editor: ...`, `Git tool: ...`, `AI agent: ...`, `Web: ...`. Arriba,
**Detectado** contiene las entradas de la lista predeterminada calculada. Dentro de cada
categoría van primero los ajustes que puedes usar de inmediato (estás dentro de ese
multiplexor o terminal y el programa está en el `PATH`), después los instalados y por último
el resto. Escribe en el cuadro combinado para buscar, por ejemplo `lazygit` o `Git tool:`.
**Probar en la pestaña actual** prueba una entrada antes de que te fíes de ella.

![La lista + Añadir de la página Abrir en: primero las entradas detectadas, luego Explorador, Multiplexor y las demás categorías, cada ajuste con su categoría como prefijo](../images/settings-open-in-presets.png)

Los ajustes de «ejecutar una herramienta en un panel» (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) solo se ofrecen cuando están
instalados el programa anfitrión (herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows
Terminal) y la herramienta. `$EDITOR in ...` ejecuta el editor indicado por tu variable
`EDITOR` (con `vi` como alternativa en herdr) en herdr o en tmux.

Los ajustes de herramientas de herdr crean la pestaña o el split y después escriben el
comando en su shell, de modo que la herramienta arranca en la carpeta y el panel sigue
abierto cuando termina. Los de `AI agent` usan `herdr agent start`, así que herdr hace
seguimiento del estado del agente.

| Categoría | Ajustes (`V` = verificado ejecutándolo o con la ayuda instalada) |
|---|---|
| Explorador | `explorer.exe` de Windows (V), Finder de macOS (V), `xdg-open` de Linux (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger en un panel/pestaña/terminal |
| Multiplexor | herdr pestaña, workspace, split a la derecha/abajo (V); tmux ventana, split a la derecha/abajo, sesión desacoplada (V); zellij panel, pestaña, flotante; WezTerm pestaña, split, ventana nueva; kitty pestaña, ventana en split, ventana del SO (V: parámetros según la ayuda; requiere `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (ventana, pestaña, split y una pestaña para cada uno de PowerShell 7, Windows PowerShell, Símbolo del sistema, WSL), ventanas de PowerShell 7 / Windows PowerShell / Símbolo del sistema / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` en un panel/pestaña/terminal |
| Herramienta Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui en un panel/pestaña/terminal (los de herdr, tmux, kitty y `xdg-terminal-exec` son V) |
| Agente de IA | Claude Code, Codex, Gemini CLI, opencode en una pestaña/split de herdr (V, con Claude), tmux, zellij, WezTerm, kitty, terminal nuevo |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL en Windows: `wsl.exe --cd ${cwd}` acepta la ruta de Windows tal cual, y la pestaña
`wsl.exe` de Windows Terminal arranca en la carpeta, que WSL mapea bajo `/mnt/`.

El explorador, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio y Zed
(Zed es solo para Unix) aparecen únicamente donde existen; tmux, zellij, cmux, screen, el
control remoto de kitty y los ajustes de herdr basados en `sh` son solo para Unix; herdr
(ajustes simples), WezTerm, Alacritty, VS Code, Cursor y los IDE de JetBrains se ofrecen en
todas partes.

### Variable `${web_url}`

La página web del remoto `origin` del repositorio, cuando es un proyecto de GitHub o GitLab
(github.com, gitlab.com o un host autogestionado que tenga una entrada en `[[integrations]]`);
vacía en otro caso, y entonces una entrada que la use se detiene con un mensaje en lugar de
abrir una dirección vacía. Es opcional: ningún ajuste la necesita salvo
`Repository page on the web`.

## La página de Ajustes

Ajustes > **Abrir en** edita la lista. Las entradas forman una
[lista](settings.md#listas) a la izquierda (la entrada predeterminada lleva la marca `★`) y los
campos de la entrada seleccionada están a la derecha; `+ Añadir`, `Duplicar`, `Quitar`, `↑` / `↓` y
arrastrar una fila gestionan la lista.

| Fila | Qué hace |
|---|---|
| **Entrada predeterminada** | La entrada que ejecuta **Pestaña: Abrir en el predeterminado**. Está marcada con `•` en la lista Abrir en. |
| **Etiqueta** | El nombre que se muestra en la lista. Único, no vacío. |
| **Comando** | El comando, con los chips de variables (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) para insertar. |
| **Probar en la pestaña actual** | Ejecuta la entrada ahora en la carpeta de la pestaña activa. |
| **+ Añadir ▾** | Añade una entrada ya preparada (las detectadas primero), o **Personalizada**, una entrada vacía cuyo Comando se edita a continuación. |
| **Restaurar valores detectados** | Sustituye la lista por lo que se detecta en este equipo y borra la entrada predeterminada. |

Los cambios se guardan a medida que los haces. La primera edición escribe en `config.toml`
todo el array `[[ui.open_in]]`; hasta entonces la lista se calcula (consulta [Claves heredadas](#claves-heredadas)) y el archivo
no tiene la clave `open_in`. La búsqueda de Ajustes también encuentra la página.

## Archivos binarios: Abrir en

Un archivo binario (un PDF, un documento de Word, una hoja de cálculo, un archivo comprimido...) no tiene texto que mostrar en la vista Diff ni en la vista de archivo. Cuando el sistema operativo conoce una aplicación para su tipo, la vista muestra un botón **Abrir en** junto al aviso habitual. Su lista contiene la aplicación con la que el sistema abre ese tipo, la primera y marcada con una viñeta, y después todas las demás aplicaciones que el sistema registra para él (la lista *Abrir con* del propio sistema). En Linux es el tipo MIME del nombre del archivo, `mimeapps.list` para la predeterminada y los archivos `.desktop` para las demás; en Windows el registro (la elección del usuario y los programas registrados para la extensión); en macOS solo la aplicación predeterminada.

`o` ejecuta la aplicación predeterminada y `Shift+O` abre la lista. Ambas están en la paleta de comandos (*Archivo: Abrir en la aplicación predeterminada*, *Archivo: Abrir con...*), y **Abrir con...** está en el menú contextual de un archivo del panel de estado y de la lista de archivos de un commit. La búsqueda se hace en segundo plano, una vez por extensión: la vista muestra «buscando aplicaciones...» mientras tanto. Un tipo para el que el sistema no conoce ninguna aplicación, y el texto plano, conservan el aviso simple y no tienen botón.

![Un PDF en la vista de archivo con su lista Abrir en: primero la aplicación predeterminada, marcada con un punto, luego las demás aplicaciones que el sistema registra para el tipo](../images/open-with.png)

El archivo del árbol de trabajo se entrega a la aplicación tal cual. La versión de un commit (o del índice) se escribe en una **copia temporal de solo lectura**, con un aviso en la barra de estado, para que una aplicación no pueda cambiar el repositorio. Ninguna de las dos cambia el repositorio: sin entrada en el diario, Deshacer no se ve afectado.

La lista del menú se abre junto al menú, a la altura de la fila **Abrir con...**, como un submenú (a la izquierda cuando no hay sitio a la derecha), tanto con el ratón como con el teclado. La lista de editores de imagen (**Editar en...**) abierta desde un menú de archivo hace lo mismo.

### Archivos Git LFS

La tarjeta de objeto Git LFS (consulta [Git LFS](lfs.md)) lleva bajo su texto las mismas dos acciones: **Abrir en predeterminado** y **Abrir en**, con las mismas teclas (`o`, `Shift+O`) y filas de la paleta. Abren el archivo real, nunca el puntero: el archivo del árbol de trabajo tal cual, o una copia temporal de solo lectura del objeto del almacén LFS local para la versión de un commit o del índice. Un objeto que aún no está descargado no tiene botones en la tarjeta, y la línea de estado dice que el contenido aún no está descargado. La misma negativa se aplica a un archivo del árbol de trabajo que todavía contiene solo el puntero: **Abrir en predeterminado**, **Abrir en** y **Editar en**, desde la tarjeta o desde cualquier menú de archivo, dicen que el contenido no está descargado en lugar de abrir el texto del puntero (descárgalo con las filas de pull o checkout de la barra LFS y vuelve a intentarlo). Abrir es de solo lectura: sin entrada en el diario, nada que deshacer.

### Navegadores en Windows

Chrome y Edge registran `--single-argument %1`, lo que les hace leer todo lo que sigue al parámetro como una sola dirección, comillas incluidas. GitPoulpi pasa la ruta a un programa así tal cual y sin comillas, como hace Windows, de modo que un archivo con espacios en su ruta se abre en lugar de fallar con `http://"c/...`. Cualquier otro programa recibe la ruta como un argumento normal.

## Claves heredadas

Las claves antiguas `ui.terminal_launch` y `ui.terminal_launch_command` solo se leen mientras
`ui.open_in` no exista. Entonces la lista calculada es:

1. tu antigua elección de terminal, si no es `auto` (`custom` pasa a ser
   «Terminal (custom)» con `{repo}` reescrito a `${cwd}`; un id de ajuste pasa a ser ese
   ajuste). Un valor `auto` pasa a ser la primera entrada, «Default terminal».
2. el explorador de archivos del SO,
3. los multiplexores y terminales detectados a partir del entorno,
4. los editores encontrados en el `PATH`.

Cuando guardas la lista desde Ajustes, `ui.open_in` toma el relevo y las claves antiguas se
ignoran. Los comandos de la paleta **Pestaña: Abrir en el terminal** y **Pestaña: Abrir en el gestor
de archivos** no usan esta lista.

## Solución de problemas

- *«no se encontró el programa X»*: el programa no está en el `PATH` que ve GitPoulpi. Inicia
  GitPoulpi desde un shell que lo tenga, o escribe la ruta completa.
- *Una entrada de tmux/zellij/herdr falla al instante*: estos hablan con un servidor en
  ejecución, así que solo funcionan cuando GitPoulpi se ejecuta dentro de él.
- *Falla la entrada de pestaña de kitty*: necesita `allow_remote_control` en `kitty.conf`.
- *Explorador de Windows*: `explorer.exe` termina con código 1 incluso cuando tiene éxito;
  GitPoulpi lo trata como un éxito.
