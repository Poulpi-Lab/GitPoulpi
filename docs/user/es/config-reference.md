[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · **Español** · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Referencia de configuración

Todas las claves de `config.toml`, su valor por defecto y lo que hacen. Rara vez
necesitarás editar el archivo a mano: la [pantalla de Ajustes](settings.md) (`Ctrl+,`) lo escribe
a medida que cambias cosas, y `gitpoulpi config set` también.

## Dónde está y cómo se elige un valor

| | |
|---|---|
| Archivo | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` lo muestra. |
| Otra carpeta | `GITPOULPI_CONFIG_DIR=<dir>` mueve toda la carpeta de configuración (el archivo, `sessions/`, los archivos de puerto del daemon): una instalación portátil, una prueba, o un agente mantenido lejos de tus propios ajustes. |
| Precedencia | valor por defecto integrado, luego `config.toml`, luego la variable `GITPOULPI_*`, luego una opción de línea de comandos. Gana la última de esa lista. Un valor incorrecto (una opción mal escrita en el archivo, una variable errónea) es un error, nunca se ignora en silencio. |
| Variables | Pon la clave en mayúsculas y cambia los puntos por guiones bajos: `general.auto_prune` es `GITPOULPI_GENERAL_AUTO_PRUNE`. Solo para las claves que GitPoulpi realmente lee (no las marcadas *no se usa* más abajo, ni listas como `ui.log_columns_hidden`). |
| Opciones | `--backend git2\|system` anula `general.default_backend` para un daemon que inicie ese comando. |
| Tokens y claves | Nunca en este archivo. Los tokens de alojamiento y las claves de API de IA van al gestor de credenciales de tu sistema (Git Credential Manager, libsecret / `pass`, Keychain); el archivo solo guarda una referencia (`token_ref`). Sin gestor, guardar un token falla con un mensaje y el token dura solo la sesión. |

```
gitpoulpi config path                          # el archivo, y si ya existe
gitpoulpi config get general.auto_stash        # el valor, la capa de la que viene (default/file/env/flag), y si algo lo lee
gitpoulpi --json config get ui                 # una sección entera
gitpoulpi config set general.auto_stash always # se comprueba contra el esquema y luego se escribe
```

`config set` mantiene el archivo escueto (solo las claves que alguien ha fijado) y
conserva las claves por las que no se le ha preguntado, pero elimina los comentarios.
Rechaza listas y tablas enteras, salvo `keybindings.*` (varios atajos separados por
espacios; vacío los desasigna). Edita `[[profiles]]`, `[[integrations]]` y
`[[ai.profiles]]` en Ajustes o a mano.

En las tablas, **Reinicio** significa que el valor se lee al arrancar; todo lo demás
se aplica al instante (o en la siguiente actualización, fetch o al cerrar Ajustes).

## `[general]`

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, recurriendo al programa `git` para staging, merge, rebase, stash y red) o `system` (solo el `git` instalado). Un daemon en marcha conserva su backend: **reinícialo** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Cuando los cambios locales bloquean un checkout, pull, merge o rebase: `ask` (un diálogo en la TUI; la CLI y MCP fallan y nombran `--autostash`), `always` (stash, ejecutar, pop) o `never` (rechazar). Ver [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | El auto-stash también incluye los archivos sin seguimiento. |
| `general.initial_commits_in_graph` | `5000` | Commits que el Log carga primero (mínimo 500). Un valor mayor cuesta memoria y tiempo de dibujo en repositorios enormes. |
| `general.lazy_load_commits_in_graph` | `true` | Carga los commits más antiguos al desplazarte cerca del final. Desactivado: el Log se detiene en el número inicial. |
| `general.commits_per_lazy_page` | `1000` | Commits por página en segundo plano (mínimo 50). |
| `general.max_commits_in_graph` | `100000` | Dónde se detiene la carga diferida (unos 300 bytes por commit); `0` = sin límite. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All en segundo plano cada N minutos, de 0 a 60; `0` = desactivado. **Reinicio.** |
| `general.auto_fetch_interval_unit` | `"auto"` | La unidad en la que Ajustes muestra el intervalo: `minutes`, `hours` o `auto`. Solo de visualización: el intervalo son siempre `auto_fetch_interval_minutes` minutos. `auto` (y una configuración sin la clave) elige la unidad más grande que divide los minutos de forma exacta (60 se lee «1 hora», 45 «45 minutos»); una unidad que no encaja se ignora. Ajustes escribe las dos claves; un GitPoulpi más antiguo ignora la unidad y sigue leyendo los minutos. |
| `general.fetch_on_tab_open` | `true` | Fetch All al abrir una pestaña (se omite si GitPoulpi hizo fetch de ese repositorio hace menos de 60 s). |
| `general.auto_prune` | `false` | Los fetch también podan las ramas de seguimiento remoto que ya no existen. |
| `general.auto_prune_tags` | `false` | Un fetch elimina también los tags locales que el remoto ha eliminado (solo los que estaban en él; un tag que nunca enviaste se conserva). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Actualización de las listas de pull requests / issues y del chip de la barra de herramientas, de 0 a 120; `0` = solo a petición y por eventos. |
| `general.hosting_refresh_unit` | `"auto"` | La unidad en la que Ajustes muestra la actualización: `minutes`, `hours` o `auto`, leída como `general.auto_fetch_interval_unit`. Solo de visualización. |
| `general.keep_submodules_up_to_date` | `false` | Tras un checkout o pull, ejecuta `git submodule update --init --recursive` cuando un submódulo está desfasado. Ver [Submódulos](submodules.md). |
| `general.default_branch_name` | `"main"` | Primera rama de un repositorio creado con Init; vacío deja decidir a git. No renombra nada. |
| `general.delete_orig_files_after_merge` | `false` | Tras un merge que se completa, elimina los archivos de copia `*.orig`. |
| `general.remember_tabs` | `true` | Reabre las pestañas de la última sesión. Desactivado, un arranque dentro de un repositorio abre solo ese repositorio, y un arranque fuera de uno abre solo Inicio. **Reinicio.** |
| `general.radar_enabled` | `true` | El [Radar](radar.md): su pestaña, sus comandos de la paleta, `Alt+R` y su consulta en segundo plano. Desactivado lo oculta todo y no se consulta nada para él. Se aplica al instante; no se muestra en una compilación sin la función `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | La pestaña en la que se abre un arranque cuando la carpeta desde la que se inicia está dentro de un repositorio: `"current-repo"` (la pestaña de ese repositorio, abierta si hace falta), `"last"` (la pestaña activa al cerrar por última vez), `"home"` o `"radar"`. Un repositorio indicado con `--repo` abre siempre su propia pestaña. `"last"` es Inicio con `remember_tabs` desactivado o si no hay nada recordado; `"radar"` es Inicio si el Radar está desactivado o no está en la compilación. Se lee una vez al iniciar; requiere reiniciar. |
| `general.startup_tab_outside_repo` | `"home"` | La pestaña en la que se abre un arranque cuando la carpeta desde la que se inicia no es un repositorio (una carpeta de usuario, por ejemplo): `"home"`, `"last"` o `"radar"`. `"last"` es Inicio con `remember_tabs` desactivado o si no hay nada recordado; `"radar"` es Inicio si el Radar está desactivado o no está en la compilación. Se lee una vez al iniciar; requiere reiniciar. |
| `general.startup_tab_launcher` | `"last"` | La pestaña en la que se abre un arranque cuando GitPoulpi se inicia desde el acceso directo del escritorio o del menú Inicio que ha creado (pasan `--launcher`), sea cual sea la carpeta: `"home"`, `"last"` o `"radar"`. Tiene prioridad sobre los dos ajustes anteriores. `"last"` es Inicio con `remember_tabs` desactivado o si no hay nada recordado; `"radar"` es Inicio si el Radar está desactivado o no está en la compilación. Se lee una vez al iniciar; requiere reiniciar. |
| `general.group_worktree_tabs` | `true` | Mantiene las pestañas de los worktrees vinculados de un repositorio justo después de la pestaña de ese repositorio: cuando se abre un worktree, cuando una pestaña de worktree se arrastra a otro sitio (vuelve a colocarse junto a su grupo; la pestaña del repositorio encabeza el grupo y sus worktrees la siguen) y cuando se activa el ajuste. Desactivado deja las pestañas donde las pongas. |
| `general.undo_depth` | `100` | Cuántas operaciones puede retroceder Deshacer; `0` = ilimitado. |
| `general.conflict_detection` | `true` | Detección proactiva de conflictos: una insignia en el Log cuando hacer merge de la rama actual en la rama de la que partió daría conflictos, calculada en segundo plano (un merge en memoria: no cambia nada en tus archivos, el índice ni las refs). Un repositorio puede desactivarla para sí mismo (`[preferences] conflict_detection`). Consulta [Resolver conflictos de merge](merge-conflicts.md#prevención-de-conflictos). |
| `general.dev_mode` | ausente (automático) | Para quien compila GitPoulpi desde un checkout; nunca se muestra en Ajustes. Ausente: activado cuando el ejecutable en marcha está en `<checkout>/target/<profile>/` (`Cargo.toml` y `crates/gitpoulpi-tui` junto a `target`). `true` / `false` lo fuerza. El modo de desarrollo solo desbloquea `ui.launcher_build`. |
| `general.journal_outside` | `true` | Registra lo que un shell o un agente de IA hace en ramas, tags, stashes y HEAD como entradas del historial de deshacer, marcadas como externas, para que `Ctrl+Z` pueda revertirlo. Las ediciones de archivos nunca se capturan. Requiere `undo_enabled`. Ver [Deshacer y rehacer](undo-redo.md#cambios-hechos-fuera-de-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: sin ningún registro de deshacer (lo más rápido), se borra el historial de deshacer y desaparecen los botones Deshacer/Rehacer. Ver [Deshacer y rehacer](undo-redo.md). |
| `general.active_profile` | `""` | Id del `[[profiles]]` (identidad) con el que se hacen los commits. Vacío, o un perfil con nombre o correo en blanco: `user.name` / `user.email` de git. |
| `general.external_editor` | `""` | Comando para *Abrir en <editor>* de un archivo, p. ej. `code -r {file}`. Vacío: VS Code si está instalado, si no el predeterminado del sistema. |
| `general.external_diff_tool` | `""` | Comando para *Abrir en herramienta de diff externa*: `{local}` (versión confirmada / HEAD), `{remote}` (versión del árbol de trabajo / índice). Vacío: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Comando para *Abrir en herramienta de merge externa* en una tarjeta de conflicto (un archivo binario, LFS o demasiado grande): `{base}`, `{local}` (A, actual), `{remote}` (B, entrante) y `{merged}` (el archivo donde la herramienta escribe el resultado; GitPoulpi le hace stage si la herramienta termina con éxito). Vacío: `git mergetool --no-prompt`. |
| `general.file_explorer` | según el SO | Comando para *Mostrar en la carpeta*: `{path}` (el archivo), `{dir}` (su carpeta). Por defecto: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Las tres plantillas de comando se dividen primero en argumentos (`"..."` agrupa una
palabra) y después se rellenan los marcadores, de modo que las rutas con espacios
siguen siendo un solo argumento. No interviene ningún shell.

```toml
[[profiles]]                # identidades de commit (Ajustes > Perfiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log y grafo**

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (una imagen donde el terminal pueda dibujarla, caracteres en los demás), `image`, `text`. **Reinicio.** |
| `ui.image_protocol` | `"auto"` | Cómo llegan las imágenes (grafo, avatares) al terminal: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protocolo Kitty en kitty, Ghostty y un herdr con `kitty_graphics = true`; iTerm2 en WezTerm, iTerm2, Rio, mintty; sixel (solo avatares) en Windows Terminal, foot, Konsole, mlterm; ninguno en tmux, zellij, screen y terminales desconocidos. **Reinicio.** |
| `ui.theme` | `"system"` | El tema de color: un slug integrado (`gitpoulpi theme list` los lista todos), el slug de un archivo de `<config>/themes/`, o `system` (sigue el fondo claro u oscuro del terminal, detectado al arrancar; el antiguo valor `default` significa lo mismo). `config set` rechaza los nombres desconocidos. Ver [Temas](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Paleta de carriles: `theme` (sigue el tema) o cualquier nombre de paleta de carriles: `gitpoulpi`, `light`, `material`, `serie`, y las paletas de los demás temas. |
| `ui.graph_lane_background` | `"off"` | `off` o `row_tint` (una banda tenue del color del carril tras el grafo, más una barra en su borde). **App: Alternar tinte de filas del grafo** (`Ctrl+N Shift+T`) lo alterna; **App: Cambiar tema de color del grafo...** (`Ctrl+N Shift+Y`) elige `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (solo con `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` y `right` dibujan una columna; una ventana de menos de 80 columnas vuelve a `bottom`. |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: dónde se colocan los botones a lo largo de la barra (izquierda, centro, derecha de una fila; arriba, centro, abajo de una columna). |
| `ui.toolbar_pr_pinned` | `true` | El botón de pull request queda fijado en el extremo de la barra de herramientas; `false` lo deja fluir con los demás botones. |
| `ui.statusbar_position` | `"bottom"` | `top` (encima de las pestañas de repositorio) o `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (la última fila de la ventana, bajo la barra de estado) o `left` (una columna en el borde izquierdo; con menos de 80 columnas se queda arriba). |
| `ui.graph_show_stashes` | `true` | Una fila por stash en el Log. |
| `ui.ref_connector` | `true` | Una línea fina desde la etiqueta de una ref hasta su commit. |
| `ui.column_resize` | `"line"` | Cómo se agarra el límite de una columna en el Log, el Radar y la columna de commits de las vistas de diff, de archivo y de blame. `line`: cada límite es un hueco de una celda con una pequeña marca en la cabecera; mientras el puntero está sobre el límite, en la cabecera o en cualquier punto de la tabla, se dibuja una línea fina de acento sobre toda la altura y un arrastre desde cualquier punto de ella redimensiona. `header`: sin hueco ni marca; se ilumina el primer carácter de la celda de cabecera que sigue al límite y un arrastre desde ahí redimensiona (solo en la cabecera). Las teclas de dimensionado y las filas de la paleta son las mismas en ambos modos. |
| `ui.ref_tree` | `true` | Agrupa las ramas y los tags por `/` en carpetas en la barra lateral Refs (`y` lo alterna). |
| `ui.graph_ghost_ref` | `true` | La columna Ref del Log muestra una píldora tenue con el nombre de la rama (si no, la rama remota) a la que pertenece un commit bajo el puntero, cuando el commit no tiene ref propia y la columna tiene sitio; nunca un tag. Una rama eliminada se deduce del asunto del commit de merge que la fusionó y se dibuja tachada. Al pasar el puntero por la píldora se ve su tipo. |
| `ui.reveal_on_ref_click` | `true` | Un solo clic en una ref (rama, tag, rama remota, stash) del panel Refs desplaza el Log hasta su commit y hace parpadear la fila; la selección de commits no se mueve. |
| `ui.log_wip_counts_in_message` | `true` | Recuentos de cambios del WIP en el mensaje del WIP mientras la columna Cambios está oculta. |
| `ui.log_columns_layout` | `"fit"` | Cómo dispone el Log sus columnas. `fit`: cada columna visible cabe exactamente en el panel y solo `message` se adapta; cuando el panel es estrecho, ceden columnas enteras. `scroll`: las columnas conservan sus anchos (`message` también), ninguna cede, y la tabla se desplaza lateralmente cuando es más ancha que el panel. |
| `ui.log_column_widths_scroll` | vacío | Los anchos de columna de la disposición `scroll`, nombre -> celdas (se admite `message`, a diferencia de `ui.log_column_widths`, que sigue siendo el de la disposición `fit` y nunca contiene `message`). Se escribe al arrastrar, dimensionar o ajustar una columna mientras la disposición es `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | vacío | Las columnas del Log (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), que se escriben al ocultar, arrastrar o redimensionar una columna. `message` no se puede ocultar. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | vacío | Las partes de la [columna de commits](interface-tour.md#la-columna-de-commits) junto a un diff, propias de la columna y aparte de las del Log: las partes visibles por nombre (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; vacío son carriles, mensaje, autor y fecha), su orden de izquierda a derecha con los mismos nombres (vacío es el orden que acaba de listarse; un nombre omitido sigue en ese orden) y los anchos arrastrados en celdas por parte (una parte omitida es automática), que se escriben al elegir, mover o dimensionar las partes. `message` no se puede ocultar. Un archivo de sesión escrito antes de que existieran estas claves se lee una vez en ellas. |
| `ui.sidebar_sections_hidden` | vacío | Las secciones del panel izquierdo que ocultaste (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), que se escriben desde el menú de clic derecho del panel o desde la paleta. Siempre queda visible al menos una sección existente. Es el valor predeterminado: cada repositorio puede ocultar sus propias secciones (se guardan en su archivo de estado) y esta clave se aplica a un repositorio sin elección propia; *Usar en todos los repositorios* la escribe. |
| `ui.sidebar_sections_scope` | `"repository"` | De quién es la elección de secciones del panel izquierdo: `repository` (cada repositorio conserva la suya, y `ui.sidebar_sections_hidden` es el valor predeterminado) o `global` (todos los repositorios muestran `ui.sidebar_sections_hidden`, y un cambio se escribe ahí). Cambiar de una a otra no hace perder ninguna: las elecciones por repositorio siguen en sus archivos de estado mientras `global` está activo. En Ajustes: *Secciones del panel izquierdo*. |
| `ui.language` | `"auto"` | El idioma de la interfaz: `auto` (el idioma del sistema, si no inglés), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Al instante.** `GITPOULPI_LANG` lo anula. Un idioma que no esté en esta compilación muestra inglés. **App: Cambiar idioma...** (`Ctrl+N g`) también lo fija, con vista previa en vivo. La salida de la CLI y de MCP sigue en inglés. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... siguen en inglés en todos los idiomas. `true`: el vocabulario de git del idioma elegido (en francés «remiser», «étiquette», «valider»...). **Al instante.** |
| `ui.git_terms_english` | `false` | Solo ja, ko y zh-CN: muestra las palabras de git en inglés (stage, unstage, staged, unstaged...) en lugar de su transcripción. Sin efecto mientras `ui.translate_git_terms` sea `true`. **Al instante.** |
| `ui.mouse_enabled` | `true` | `false` deja el ratón (y la selección de texto) al terminal. **Reinicio.** |
| `ui.mouse_scroll_lines` | `3` | Filas del Log por muesca de la rueda, de 1 a 20. **Reinicio.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` pide al terminal el protocolo de teclado de kitty (para que lleguen `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`); `off` nunca lo pide. **Reinicio.** |
| `ui.logo_color` | `""` | Combinación de colores del logo en la ventana Acerca de, un slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); vacío o desconocido significa el valor por defecto de la compilación (`lavender`). Se escribe al hacer clic en el logo o elegir un color en Acerca de. |

**Avatares** (desactivados por defecto; activarlos envía un hash SHA-256 de los
correos de los autores a Gravatar, y consulta a tu cuenta conectada de GitHub/GitLab)

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (en el círculo del commit del grafo de imagen), `author` (en la columna Autor), `off`. El antiguo `ui.show_avatars` (`true`) se sigue leyendo como `author` mientras `ui.avatars` no exista. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Orden de búsqueda: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. A un host de GitLab solo se le pregunta mediante un perfil conectado. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (generado localmente) o `initials`. En un terminal sin protocolo de imágenes se usa siempre la insignia con iniciales. |

**Diff y editor**

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` o `split` (`m` en un diff). |
| `ui.diff_hunks_only` | `true` | `false` muestra el archivo entero con los cambios en su sitio (`H`). |
| `ui.diff_show_whitespace` | `true` | Muestra los cambios que son solo de espacios en blanco (`s`). |
| `ui.diff_word_wrap` | `false` | Ajusta las líneas largas del diff (`w`; Ajustes > Diff y editor). |
| `ui.minimap` | `false` | Un minimapa junto a la barra de desplazamiento del diff y del editor (`M`). |
| `ui.mermaid_detail` | `refs` | Detalle del Copiar como gitGraph de Mermaid del Log: `structure`, `refs`, `subjects`, `full` (se recuerda desde el popup; el valor por defecto de `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Dirección de ese grafo: `lr` (de izquierda a derecha) o `tb` (de arriba abajo). |
| `ui.mermaid_fence` | `false` | Envuelve el script de Mermaid copiado en un bloque de código `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Los asuntos de los commits se recortan a este número de caracteres en los ids de Mermaid (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` o `16`: con cuántos colores dibujar. `auto` es truecolor salvo en Apple Terminal (256), la consola de Linux (16) y los terminales `*-256color` sin `COLORTERM` (256). Al instante. Ver [Temas](themes.md). |
| `ui.input_color` | `"theme"` | Fondo de los campos de texto: `theme` (el propio del tema), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | El color `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | El compositor de commits con IA abre los archivos de cada propuesta `expanded` (expandidos) o `collapsed` (contraídos). |
| `ui.blame_side` | `"right"` | Qué lado de la vista Blame lleva la columna de anotaciones (avatar, autor, antigüedad, resumen): `right` (por defecto) o `left`. |
| `ui.blame_width` | `46` | Ancho de la columna de anotaciones de Blame en celdas (24 a 90). Arrastra su borde o pulsa `<` / `>` en la vista; el nuevo ancho se guarda aquí. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request en GitHub, merge request en GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Oculta el chip de la barra de herramientas con la última pull request o merge request de la rama con checkout cuando ya está cerrada o fusionada; en su lugar aparece el chip atenuado «+ PR» (crear una nueva), como si la rama no tuviera ninguna. `false`: el chip muestra siempre la última solicitud, sea cual sea su estado. |

Los interruptores del diff que cambias dentro de un diff se guardan aquí al
instante y los usan todos los diffs que abras después.

**Fechas**: `ui.date_format` (`"YYYY-MM-DD"`, la columna de fecha de Blame / Historial del archivo),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, la columna Fecha del Log) y
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, los detalles del commit, el tooltip
de la fecha y *Copiar > Fecha*). Tokens: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (día de la semana, completo / abreviado) `HH` `mm` `ss`; cualquier otra
cosa se copia tal cual. Los nombres de meses y días de la semana siguen a `ui.language`,
y un mes escrito tras un número de día va en genitivo cuando el idioma lo tiene (ruso,
ucraniano, polaco, checo). Una secuencia `D MMMM YYYY` se escribe con el año primero
en japonés, chino y coreano (`2024年3月5日`). `ui.date_locale` (`"system"`, el valor
por defecto, o `"english"`) mantiene los nombres en inglés sea cual sea el idioma.

**Agentes, terminales y la carpeta de la pestaña**

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | La ventana acepta en absoluto `ui show` / `ui propose` de los agentes (Ajustes > Agentes y MCP > *Permitir que los agentes de IA controlen la vista*). Desactivado, ningún agente la alcanza. Ver [MCP y agentes](mcp-and-agents.md#mostrar-cosas-al-usuario). |
| `ui.launch` | `"off"` | Si un agente puede abrir una ventana de GitPoulpi cuando no hay ninguna abierta: `off`, un preset (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) o `custom`. |
| `ui.launch_command` | `""` | La plantilla `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Se divide en argumentos, sin shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (la fila de pestañas ofrece *Salir y cd aquí*) o `auto` (cada salida cambia la carpeta del shell, y una pestaña nueva ejecuta la entrada *Abrir en* por defecto). Ver [Seguir la carpeta de la pestaña](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Heredada, ver abajo y [Abrir en](open-in.md). Cómo abría un terminal el antiguo *Terminal aquí*: `auto` (el preset de `ui.launch`, si no `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), un preset, o `custom`. |
| `ui.terminal_launch_command` | `""` | La plantilla `custom`; `{repo}` es la carpeta. |
| `ui.open_in` | *(ausente)* | La lista de [Abrir en](open-in.md), un array de tablas `[[ui.open_in]]` con `label` y `command`. Ausente significa «aún sin configurar»: la lista se calcula a partir de las claves heredadas `ui.terminal_launch*` (un valor distinto de `auto` pasa a ser la primera entrada), el explorador de archivos del SO, los multiplexores y terminales detectados en el entorno (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) y los editores encontrados en el `PATH`. El primer guardado desde Ajustes escribe la lista. `open_in = []` es una lista vacía. No se puede fijar con `config set` (edita el archivo). |
| `ui.open_in_default` | `""` | Etiqueta de la entrada que ejecuta «Abrir en el predeterminado»; vacío o desconocido significa la primera entrada. |
| `ui.launch_terminal` | `"auto"` | El terminal en el que los accesos directos del escritorio (Ajustes > General > Lanzador) y el relanzamiento en Windows inician GitPoulpi. `auto` elige entre lo instalado (Windows: Windows Terminal, si no WezTerm, Alacritty, la consola simple; Linux: `xdg-terminal-exec`, si no el primer terminal encontrado; macOS: iTerm2, Ghostty, WezTerm o kitty si están instalados, si no Terminal), `default` es el del sistema, o bien un id de terminal como `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Solo en modo de desarrollo (véase `general.dev_mode`; solo entonces se muestra en Ajustes > Lanzador, y si no se ignora). `"current"`: los accesos directos y el alias del comando inician la compilación que los creó. `"newest"`: inician la compilación más reciente entre `target/dist` y `target/release` del checkout, decidida en cada clic, e iniciar cualquiera de las dos nunca los reescribe. |
| `ui.command_alias` | `""` | El nombre del comando que inicia GitPoulpi desde un terminal, por ejemplo `gp` (Ajustes > General > Lanzador, `gitpoulpi alias create`). Vacío: ninguno. Solo letras, dígitos, `-`, `_` y `.`. Es únicamente el nombre que usa el botón **Crear: alias del comando**; no existe nada hasta que lo creas. |
| `ui.command_alias_add` | `""` | El nombre del segundo comando, por ejemplo `gpa`, que ejecuta `gitpoulpi --add`: abre la carpeta actual en una pestaña nueva del GitPoulpi que ya está en marcha (Ajustes > General > Lanzador, `gitpoulpi alias create gpa --add`). Vacío: ninguno. Mismas reglas de nombre; el botón **Crear** lo crea junto con `ui.command_alias`. |
| `ui.relaunch_in_terminal` | `true` | Solo Windows: cuando el exe se inicia con un doble clic en una consola propia, GitPoulpi se reabre en `ui.launch_terminal` y sale. Desactivado: se queda en esa consola. |
| `ui.reuse_window` | `true` | Un clic en un acceso directo de GitPoulpi (`gitpoulpi launch`) muestra la ventana de un GitPoulpi que ya está abierto, en lugar de iniciar otro. Desactivado: cada clic inicia una ventana nueva. |

**Los comandos de `ui.open_in`** se dividen primero en argumentos (espacios en blanco, `'...'`, `"..."`) y después se sustituyen las variables dentro de cada argumento, de modo que una carpeta con espacios sigue siendo un solo argumento; no interviene ningún shell salvo que el comando inicie uno (`sh -c '... "$GITPOULPI_CWD"'`). Variables: `${cwd}` (la carpeta de la pestaña), `${repo}` (raíz del repositorio), `${name}` (nombre base de la carpeta), `${branch}` (vacío con HEAD desacoplado), `${web_url}` (la página web del remoto `origin` cuando es un proyecto de GitHub o GitLab, si no vacío; una entrada que la use se niega a ejecutarse si está vacía), `${env:NAME}` (vacío si no está definida). El antiguo `{repo}` sigue significando `${cwd}`. Cualquier otro `${...}` es un error que lo nombra. Ejemplo:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Heredadas: `ui.terminal_launch` y `ui.terminal_launch_command` se siguen leyendo (inicializan la lista mientras `ui.open_in` no exista: `custom` pasa a ser «Terminal (custom)» con `{repo}` reescrito como `${cwd}`, un id de preset pasa a ser ese preset), pero las sustituye `ui.open_in`.

### Imágenes

| Clave | Por defecto | Significado |
|---|---|---|
| `ui.image_view` | `true` | Muestra como imagen un archivo binario que es una imagen (PNG, JPEG, GIF, WebP, BMP, ...) en la vista Diff y en la Vista de archivo. Desactivado: el aviso binario habitual. |
| `ui.image_view_max_mib` | `32` | Las imágenes mayores que esto (MiB) no se decodifican; en su lugar se muestra una tarjeta informativa. |
| `ui.image_view_max_unit` | `"auto"` | La unidad en la que Ajustes muestra el límite: `mib`, `gib` (1024 MiB) o `auto`. Solo de visualización: el límite son siempre `image_view_max_mib` MiB, de 1 a 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | En un terminal sin protocolo de imagen: `halfblock` (caracteres de medio bloque coloreados) o `card` (solo tarjeta informativa). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom a partir del cual ampliar muestra píxeles cuadrados en lugar de suavizar: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Los paneles antes y después comparten un mismo zoom y posición de desplazamiento. |
| `ui.image_editors` | *(ausente)* | Los editores de imagen de «Editar en», un array de tablas `[[ui.image_editors]]` con `label` y `command`. Ausente significa «aún sin configurar»: la lista son los editores detectados en esta máquina (programas en el `PATH`, instalaciones Flatpak, carpetas de instalación conocidas en Windows, `/Applications` en macOS), luego la aplicación predeterminada del SO. El primer guardado desde Ajustes escribe la lista. `image_editors = []` es una lista vacía. No se puede fijar con `config set` (edita el archivo). |
| `ui.image_editor_default` | `""` | Etiqueta del editor que ejecuta «Editar en el predeterminado»; vacío o desconocido significa la primera entrada. |

**Los comandos de `ui.image_editors`** se dividen primero en argumentos (espacios en blanco, `'...'`, `"..."`) y después se sustituyen las variables dentro de cada argumento, de modo que una ruta con espacios sigue siendo un solo argumento; no interviene ningún shell. Variables: `${file}` (la ruta absoluta de la imagen), `${dir}` (su carpeta), `${name}` (su nombre de archivo), `${repo}` (la raíz del repositorio, vacío para una copia temporal), `${env:NAME}` (vacío si no está definida). Cualquier otro `${...}` es un error que lo nombra. Una imagen que no es el archivo del árbol de trabajo (la versión de un commit, el índice, un archivo eliminado) se abre como una **copia temporal de solo lectura**, de modo que un editor no puede tocar el repositorio. Los editores sin línea de comandos (Affinity Photo de la Microsoft Store, algunas instalaciones de Photoshop) no están en los presets: escribe una entrada personalizada. Ejemplo:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Aún no se usan** (se aceptan y se guardan, pero nada las lee):
`ui.graph_colorization` (eliminada de Ajustes;
se sigue aceptando al cargar), `ui.date_word_format` (eliminada de Ajustes; se sigue
aceptando al cargar). El antiguo `ui.diff_view_mode`
se lee una vez como `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `editor.tab_size` | `4` | Columnas por tabulación, de 1 a 8 (editor y diffs). |
| `editor.eol` | `"auto"` | Fin de línea que escribe el editor integrado: `auto` (conserva el del archivo), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Colores del lenguaje en el editor. |
| `editor.line_numbers` | `true` | Números de línea en el editor y en los diffs. |
| `editor.follow_selection` | `true` | Mientras el editor está abierto, seleccionar otro archivo en el panel derecho lo abre también en el editor, tras preguntar si se guardan los cambios pendientes. Un archivo que no se puede editar se abre en su vista habitual; un archivo de un commit sale del editor hacia su diff. |

## `[merge]`

El editor de conflictos de merge ([Resolver conflictos de merge](merge-conflicts.md)). Lo muestra Ajustes > Diff y editor.

| Clave | Por defecto | Significado |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | El ancho, en columnas, del área del editor a partir del cual A y B se muestran lado a lado (las líneas largas se desplazan lateralmente: Shift+rueda, Shift+flechas). Más estrecho, muestra un lado cada vez tras unas pestañas. `40` es el valor más bajo, `0` significa siempre lado a lado (el aviso por debajo de 40 columnas se mantiene). El editor oculta los paneles Refs y Estado mientras está abierto, así que su área suele ser toda la ventana. |

## `[git]`, `[credentials]`, `[ssh]`

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `git.pull_mode` | `"merge"` | Lo que hace Pull (botón, `Ctrl+G p`, `Git: Pull`): `merge` (fast-forward cuando es posible), `ff_only`, `rebase`. Siempre se pasa a git de forma explícita. El antiguo `fetch_all` se lee como `merge`. |
| `git.push_behavior` | `"direct"` | `direct` hace push al remoto por defecto; `ask` muestra antes el menú de remotos. |
| `git.default_encoding` | `"utf-8"` | Codificación para el texto que **no** es UTF-8 válido (contenido de archivos, blame, diffs, mensajes de commit sin cabecera `encoding`): cualquier etiqueta WHATWG (`windows-1252`, `shift_jis`, ...). Solo afecta a la visualización; el staging nunca recodifica. Un repositorio puede anularla. |
| `git.lfs_enabled` | `true` | Muestra y maneja Git LFS. Un repositorio puede anularla. Ver [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Antes de un pull o push de un repositorio que usa LFS, instala sus filtros LFS / hook pre-push (`git lfs install --local`, solo este repositorio) si faltan. Ver [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *No se usan*: `core.autocrlf` / `core.eol` de un repositorio son ajustes propios de git, que se editan en Ajustes > Git y codificación (o con `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Autenticación HTTPS para fetch, pull, push, clone: `credential_helper` (el de git), `prompt` (ignora los gestores; git pregunta en un terminal, así que falla en el daemon en segundo plano), `none` (solo remotos públicos). Los antiguos valores `encrypted_file` se leen como `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Tipo que crea *Generar par de claves*: `ed25519`, `rsa` (4096 bits) o `ecdsa` (NIST P-256); el archivo es `gitpoulpi_<tipo>`. |
| `ssh.private_key_path`, `ssh.public_key_path` | sin definir | Los registra *Generar par de claves*. Cuando `ssh.private_key_path` está definida, fetch, pull, push y clone ejecutan git con `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` hace lo mismo (ssh elige la clave del agente que coincide) cuando la ruta privada está vacía. Un `GIT_SSH_COMMAND`/`GIT_SSH` en el entorno o un `core.sshCommand` en la configuración de git (el anclaje por repositorio, `gitpoulpi ssh-command set --key`) tiene prioridad. Se leen antes de cada transferencia, así que los cambios se aplican a la siguiente. |

## `[[integrations]]` (alojamiento y gestores de issues)

Lo escribe Ajustes > Integraciones o `gitpoulpi hosting integrations add`. Todos los
tipos tienen cliente (solo tokens personales, sin OAuth).

```toml
[[integrations]]
id = "corp"                         # lo que toma --id; el token se guarda bajo integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # de aquí se deriva la raíz de la API
token_ref = "integration:corp"      # una referencia, nunca el token
name = "Corp GitLab"                # nombre para mostrar, opcional
account = "me@example.com"          # opcional, consulta más abajo
enabled = true                      # false: se ignora al buscar el host
```

Un servidor autoalojado puede usar http sin cifrar: `url = "http://localhost:3000"` se conserva
tal cual (el esquema y el puerto llegan a cada petición, y un remoto en ese host
coincide con el perfil). Un token enviado por http sin cifrar viaja en claro, así que usa
http solo en una red local o de confianza.

`kind` también acepta `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` y `gerrit`
(Gitea y Forgejo comparten un cliente; un perfil de Forgejo usa por defecto `https://codeberg.org`). Jira,
Trello, Linear, YouTrack y Redmine son solo gestores de issues; Gerrit es solo de revisión (sus cambios son las
pull requests, no tiene issues). Bitbucket Cloud no tiene gestor de issues.

`account` es la identidad no secreta que acompaña al token, vacía y omitida en la
mayoría de los tipos: el correo de la cuenta Atlassian para `jira_cloud` y `bitbucket`
(una cuenta de Bitbucket vacía significa un token de acceso de workspace o de
repositorio, enviado como Bearer), la clave de API para `trello`, el nombre de usuario para `gerrit`
(el token es su contraseña HTTP, enviada como HTTP basic). El token en sí
sigue siendo el único secreto. Para `azure_devops` la `url` es
`https://dev.azure.com/<organization>`; una URL
`https://<organization>.visualstudio.com` se lee como la misma organización.
Para `azure_devops_server` es la raíz de la colección, como
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (la cuenta conectada) y `ssh` (la clave elegida para el perfil) se escriben
por ti. El token sale de `GITPOULPI_TOKEN_<ID>` (id en mayúsculas, caracteres no
alfanuméricos como `_`), si no del token guardado, si no de tu gestor de credenciales
para ese host cuando solo lo usa un perfil (solo perfiles de GitHub y GitLab: los
demás tipos nunca leen el gestor de credenciales). Detalles: [Alojamiento y
gestores de issues](hosting-integration.md). La antigua tabla
`[[hosting]]` *no se usa*.

Para `local_markdown` (una carpeta de notas Markdown como origen de issues) `url` es la
carpeta, relativa a cada repositorio o absoluta, y no hay token. Su tabla opcional
`[integrations.local]` fija `statuses` (el orden de las columnas del tablero, por
defecto `todo`, `doing`, `done`), `done_status` (por defecto `done`),
`reference_prefix` (`LOC` muestra `LOC-12`) y `me` (qué significa "asignado a mí").
Ver [Issues locales y Obsidian](local-issues.md).

### `[integrations.radar]` (listas de toda la cuenta)

Tabla opcional de un perfil de host de código (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; no Bitbucket Cloud). Si falta,
todas las claves están desactivadas. La escribe `gitpoulpi hosting integrations radar`; la leen
`gitpoulpi hosting radar` y el Radar, y se edita en Ajustes > Integraciones (el bloque Radar de un perfil conectado).

```toml
[integrations.radar]
review_requests = true    # solicitudes que esperan mi revisión, todos los repositorios
open_requests = false     # todas las solicitudes abiertas de mis espacios de nombres
include_drafts = false    # mantiene las solicitudes en borrador en estas listas
assigned_issues = true    # issues abiertos asignados a mí, todos los repositorios
namespaces = ["acme"]     # organizaciones / grupos donde buscar solicitudes abiertas
review_mode = "ask"       # ask | clone | api
```

| Clave | Por defecto | Significado |
|---|---|---|
| `review_requests` | `false` | Solicitudes abiertas que piden tu revisión, esté o no clonado el repositorio |
| `open_requests` | `false` | Todas las solicitudes abiertas de tu propio espacio de nombres y de cada organización o grupo al que perteneces (o de `namespaces`), nunca «todo lo que ve el token» |
| `include_drafts` | `false` | Las solicitudes en borrador quedan fuera de estas listas salvo que esto esté activado |
| `assigned_issues` | `false` | Issues abiertos asignados a ti, todos los repositorios |
| `namespaces` | `[]` | Organizaciones / grupos donde buscar `open_requests`; vacío significa tu propio espacio de nombres más todas tus pertenencias (GitHub: se buscan 12 como máximo, GitLab: 10 grupos y 10 proyectos propios, Gitea, Forgejo y Gerrit: 10; Azure DevOps toma nombres de proyecto, Bitbucket Data Center claves de proyecto, Gerrit prefijos de ruta de proyecto) |
| `review_mode` | `"ask"` | Qué hace revisar una solicitud cuyo repositorio no está clonado: `ask`, `clone` (clonarlo y luego revisar) o `api` (revisar mediante la API del host) |

Las respuestas se guardan en caché por perfil en `radar-account.json`, junto a
`config.toml`, durante cinco minutos; el archivo es una caché y se puede borrar sin problema.

## `[mcp]` (lo que pueden hacer los agentes de IA)

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `mcp.allow_mutations` | `false` | Desactivado: se rechazan las herramientas MCP que cambian algo; las lecturas, `ui_*` y las propuestas siguen funcionando. `true` permite a los agentes modificar el repositorio. |
| `mcp.mutation_allowlist` | `[]` | No vacía: solo estos nombres de herramienta pueden cambiar cosas (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: rechaza las herramientas `ui_*` que mueven tu ventana (no `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (la ventana muestra un chip que confirmas), `auto` (se aplica salvo que estés ocupado), `off`. `--ui-control` en el comando del servidor lo anula por agente. También se aplica a `gitpoulpi ui show`. |
| `mcp.allow_history_purge` | `false` | Solo en `config.toml` (sin fila en Ajustes). `true` muestra a los agentes la herramienta `rewrite_purge` (sigue necesitando `allow_mutations`). Desactivado, el agente no ve la herramienta: una purga elimina para siempre el historial antiguo de una reescritura, su deshacer y sus reflogs. La CLI y la TUI no dependen de este ajuste. |
| `mcp.allow_ui_propose` | `false` | Permite `ui_propose_*` / `gitpoulpi ui propose`: diálogos rellenados de antemano que confirmas tú mismo. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` y `gitpoulpi-mcp` se niegan a arrancar, con un mensaje que nombra esta clave. Ajustes > Agentes y MCP > *Servidor MCP*. Se lee cuando un agente arranca el servidor. |
| `mcp.bind` | `"127.0.0.1:8808"` | La dirección en la que escucha `--http` cuando no das ninguna (`--http ADDR` / `--bind` tienen prioridad). Solo loopback salvo con `--allow-remote`. Vacío o el antiguo `"stdio"` significan el valor por defecto. Ajustes > Agentes y MCP > *Puerto del servidor MCP* edita el puerto y conserva el host. |

`allow_mutations`, `mutation_allowlist` y `allow_ui` controlan solo el servidor MCP;
la CLI es tu propio shell y no la limitan. Consulta [MCP y
agentes](mcp-and-agents.md#permisos).

## `[ai]` (IA integrada, desactivada por defecto)

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `ai.enabled` | `false` | El interruptor. Desactivado, no se muestra nada de IA en ningún sitio y no se envía nada. |
| `ai.default_profile` | `"default"` | Id del perfil que usan las acciones cuando no nombran ninguno. |
| `ai.global_instructions` | `""` | Texto añadido al prompt de cada acción. Puede usar `${repo}`, `${uilang}`, ... (ver [IA > Variables de los prompts](ai.md#variables-del-prompt)); igual para las instrucciones por acción y las del repositorio. |
| `ai.actions.<action>.profile` | `""` | Perfil de una acción; vacío = el predeterminado. Acciones: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Un espacio de trabajo (`workspaces.toml`, `[workspace.ai_instructions]` y `[workspace.ai_overrides.<action>]`) y un repositorio pueden añadir a estas instrucciones o reemplazarlas: ver [IA](ai.md#instrucciones). |
| `ai.actions.<action>.instructions` | `""` | Texto añadido solo para esa acción. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` o `no`: si el agente local de un borrador de revisión de código trabaja dentro del repositorio en esa ejecución, sustituyendo el `workdir` del perfil de IA. Un repositorio puede fijar el suyo (`gitpoulpi ai review-context --repo`). Ver [IA > Contexto de la revisión](ai.md#contexto-de-la-revisión). |
| `ai.review.include_claude_md` | `true` | Da a un borrador de revisión de código el `CLAUDE.md` del repositorio (raíz, leído en la rama a la que apunta la solicitud; cortado a 8 KB). |
| `ai.review.include_agents_md` | `true` | Lo mismo para `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Un archivo más para un borrador de revisión de código: una ruta relativa a la raíz del repositorio, una ruta absoluta o un enlace a un archivo de un host de código (se lee con el perfil de integración que coincida, si no de forma anónima). Todos los archivos juntos se cortan a 16 KB. |

La tabla propia de un repositorio `[preferences.ai_review]` (en su `repo-state.toml`, o la entrada de un repositorio que no
está clonado en `remote-reviews.toml`) admite las mismas claves, todas opcionales, más `no_extra_file = true`: ningún
archivo adicional para ese repositorio, diga lo que diga `ai.review.extra_file`. Una clave sin fijar, un `extra_file` sin
fijar y un `no_extra_file` ausente heredan el valor global; fijar `extra_file` a un archivo borra el interruptor y viceversa.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # vacío = el valor por defecto del kind
base_url = ""           # vacío = el valor por defecto del kind (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # dónde se guarda la clave de API; nunca la clave
command = ""            # solo local_agent: el programa exacto ("claude", un alias, un script envoltorio, una ruta)
harness = ""            # solo local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; vacío = se infiere de `command`
args = []               # solo local_agent, p. ej. ["-p", "--model", "haiku"]
run_mode = "headless"   # solo local_agent: headless (predeterminado) | terminal (la pantalla interactiva del agente en un terminal virtual)
terminal_args = []      # solo local_agent: los argumentos del modo terminal (args conserva el conjunto sin interfaz)
workdir = "default"     # solo local_agent: default | dedicated | repository (default = carpeta dedicada en modo terminal, la carpeta de GitPoulpi en modo sin interfaz)
timeout_secs = 0        # solo local_agent: segundos antes de detener el agente; 0 = 120 sin interfaz, 180 terminal

[ai.profiles.env]       # solo local_agent, opcional: variables definidas solo para ese proceso
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = una de las propias variables de GitPoulpi
```

`harness` elige las sugerencias que ofrece Ajustes > IA (argumentos sin interfaz,
opciones, opción y lista de modelos, variables de entorno) con independencia de
`command`; ambas claves se omiten del archivo cuando están vacías. Los valores de
entorno nunca se registran y se ocultan en Ajustes cuando parecen secretos.

Ajustes > IA ofrece presets para `harness`/`command`/`args`/`env` (las CLI de agentes habituales, con
sus opciones sin interfaz, opción de modelo y variables de entorno útiles) y para `kind`/`base_url`/`model` (los
proveedores HTTP alojados y locales); consulta [IA > Configurar la IA](ai.md#configurar-la-ia).

Se crea un perfil llamado *Default* cuando no hay ninguno. Los antiguos archivos con
`[[ai.presets]]` / `[ai.defaults]` se migran al cargar y se conservan en
`[[ai.legacy_presets]]`. Todo lo demás: [Asistencia de IA](ai.md).

## `[repos]`, `[session]`

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `repos.clone_root` | `""` | Dónde empieza el diálogo Clonar; vacío = `%USERPROFILE%\source\repos` o `~/src`. |
| `repos.browse_root` | `""` | Dónde se abre Examinar; vacío = la última carpeta elegida, luego la carpeta personal. |
| `repos.show_hidden` | `false` | El selector de carpetas muestra las carpetas ocultas (`Alt+H` lo alterna y lo guarda). |
| `repos.show_siblings` | `false` | El árbol izquierdo del selector de carpetas también lista las demás carpetas de cada nivel abierto, como una jerarquía real (`Alt+S` lo alterna y lo guarda). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Dónde guarda GitPoulpi lo que recuerda de un repositorio (filtros de refs, estado de plegado, el flujo de trabajo personal que sustituye al compartido, la caché de alojamiento): `in_git_dir` = `<git dir>/gitpoulpi` (compartido por todos los worktrees, nunca aparece en `git status`), `in_worktree` = `<repo>/.gitpoulpi` (el único sitio donde un equipo puede hacer commit de `workflow.toml`), `user_data` = `<carpeta de datos del SO>/repos/<nombre>-<id>` (`%APPDATA%\GitPoulpi\data` en Windows, `$XDG_DATA_HOME/gitpoulpi` en Linux, `~/Library/Application Support/GitPoulpi` en macOS). El valor antiguo `".gitpoulpi"` sigue significando `in_worktree`. Cambiarlo no devuelve nada a su sitio: los datos que se encuentran en un `<repo>/.gitpoulpi` heredado se trasladan al sitio elegido la primera vez que hacen falta. |

## `[log]` (el registro de la aplicación)

Lo que muestra la pestaña «Aplicación» del Registro de actividad y lo que cita «Copiar diagnóstico». Detalles: [Cuando algo falla](troubleshooting.md#cuando-algo-falla).

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `log.level` | `"info"` | La línea más detallada que se registra: `error`, `warn`, `info`, `debug` o `trace`. El cuadro de nivel del Registro de actividad lo cambia (y lo guarda aquí). |
| `log.retention_days` | `7` | Días de archivos de registro que se conservan, el de hoy incluido: de 1 a 3650, diez años (un valor mayor o menor se ajusta al límite). Siempre es un número de días, sea cual sea la unidad en la que Ajustes lo muestre. Los archivos más antiguos se borran al iniciar GitPoulpi. El archivo de un día también está limitado por `log.max_file_mb`, así que el peor caso es de unos `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | La unidad en la que Ajustes muestra `log.retention_days`: `days`, `weeks`, `months`, `years` o `auto`. Solo de visualización: nunca cambia cuánto tiempo se conservan los archivos. Una semana son 7 días, un mes son 30 días (no un mes de calendario, para que sean los mismos días en cualquier momento) y un año son 365 días. `auto`, y una configuración escrita antes de que existiera esta clave, elige la unidad más grande que divide los días de forma exacta (14 días se leen «2 semanas», 45 días «45 días»). Una unidad que no divide los días (`retention_days = 10` con `weeks`) se ignora y se muestran los días. Ajustes escribe las dos claves; un GitPoulpi más antiguo ignora la unidad y sigue leyendo los días. |
| `log.max_file_mb` | `5` | El archivo de un día se rota a una copia `.old` cuando alcanza este tamaño, así que un día nunca ocupa más del doble. |
| `log.max_file_unit` | `"auto"` | La unidad en la que Ajustes muestra el tamaño: `mib`, `gib` (1024 MiB) o `auto`. Solo de visualización: el tamaño son siempre `max_file_mb` MiB, de 1 a 1024. |
| `log.timings` | `true` | La pestaña Operaciones de git del Registro de actividad muestra cuánto tardó cada operación (`Push - 123 ms`: abrir un repositorio, actualizar, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). La ventana mide las duraciones, desde la petición hasta la respuesta del daemon, así que incluyen el viaje de ida y vuelta por IPC. Desactivado, oculta las duraciones y las entradas de las operaciones correctas. Los fallos siempre se listan. |
| `log.extended` | `false` | «Registro ampliado»: la pestaña Operaciones de git también lista las operaciones en segundo plano (la actualización periódica que consulta el estado, el auto-fetch) con sus duraciones. Siempre se escriben en el registro de la aplicación con nivel `debug`. |
| `log.commit_count` | `"system-git"` | Cómo averigua la barra de estado el total de `cargados / total`. `"system-git"` ejecuta `git rev-list --count` (un proceso, lo más rápido en un equipo normal: git lee el archivo commit-graph). `"libgit2"` cuenta en el propio proceso con un revwalk de libgit2 y no lanza nada: elígelo cuando iniciar cualquier proceso de línea de comandos es lento, normalmente porque un antivirus con análisis en tiempo real inspecciona cada uno; con 50 000 commits tarda unos 200 ms en segundo plano en lugar de 12 ms, pero no se inicia ningún proceso. `"off"` nunca cuenta: la barra muestra solo los commits cargados. Se aplica al instante. Un recuento filtrado por ruta siempre usa el git del sistema. |
| `log.use_mailmap` | `true` | Muestra cada autor con el nombre que le dan el `.mailmap` del repositorio (y `mailmap.file`) en el Log, en el blame y en el historial de archivos: el daemon pide a git `%aN`/`%aE` (el backend de sistema) o aplica él mismo los mismos archivos (el backend libgit2), así que la respuesta es idéntica. Los commits guardados no cambian y no se reescribe nada. Buscar en el Log por autor coincide con los nombres tal como se muestran. Se aplica en vivo; el Log lo relee en su sondeo de 2 s. |

Los archivos son `gitpoulpi-YYYY-MM-DD.log` en la carpeta de estado del SO (`~/.local/state/gitpoulpi/logs` en Linux; `<cache>\logs` en Windows y macOS; `<config>/cache/logs` con `GITPOULPI_CONFIG_DIR`). Los tokens, las contraseñas y las cabeceras `Authorization` se ocultan antes de escribir nada.

## `[rewrite]` (reescritura del historial)

Lo que protegen `gitpoulpi rewrite` y el editor de reescritura del historial. Detalles: [Reescribir el historial](history-rewrite.md).

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Ramas (nombres, o patrones donde `*` coincide con cualquier texto) que una reescritura deja en paz cuando ya están publicadas, salvo que la ejecución las indique (`--include-protected RAMA`; `--include-all-protected` para el push; la casilla de cada rama en el editor). Se suman a las bases que el `workflow.toml` de un repositorio marca como `protected`. El push de una reescritura las omite igual. Es la protección propia de GitPoulpi: la protección de ramas del host sigue decidiendo si el push forzado se acepta. Se lee en vivo: la siguiente vista previa respeta el cambio. |
| `rewrite.external_scripts` | `["shell"]` | Qué lenguajes de script externo puede usar la regla Script: cualquiera de `"shell"` (bash o sh; pwsh o powershell para scripts de PowerShell), `"python"`, `"lua"` (un Lua instalado; el Lua integrado siempre está activo y no necesita entrada). **Un script externo no está aislado: se ejecuta con tus permisos.** Un lenguaje que no figura aparece atenuado en el editor y la CLI, MCP y el daemon lo rechazan (`scripts_disabled`). Aunque figure, un script externo que viene del repositorio o de una receta compartida solo se ejecuta después de que le des confianza (`gitpoulpi rewrite script trust FILE`, o la pregunta del editor); la confianza se asocia al hash del contenido del script, así que un script modificado vuelve a preguntar. Activa Python editando `config.toml` (una lista no se puede fijar con `config set`): `[rewrite]` y luego `external_scripts = ["shell", "python"]`. Se lee en vivo. |
| `rewrite.script_timeout_ms` | `5000` | Cuánto tiempo tiene un script externo para responder a un commit. Si se agota, se mata todo el árbol de procesos y la reescritura falla (`script_failed`, con el nombre del commit); no se ha movido nada. `0` se interpreta como el valor por defecto. Ajustes acepta de 1 a 600000 (diez minutos). Se lee en vivo. |
| `rewrite.script_timeout_unit` | `"auto"` | La unidad en la que Ajustes muestra el límite de tiempo: `milliseconds`, `seconds` o `auto`. Solo de visualización: el límite son siempre `script_timeout_ms` milisegundos. Se lee en vivo. |
| `rewrite.script_total_timeout_s` | `3600` | Cuánto puede tardar en total una pasada de un script externo. `0` se interpreta como el valor por defecto. Ajustes acepta de 1 a 86400 (un día). Se lee en vivo. |
| `rewrite.script_total_timeout_unit` | `"auto"` | La unidad en la que Ajustes muestra el límite de tiempo: `seconds`, `minutes`, `hours` o `auto`. Solo de visualización: el límite son siempre `script_total_timeout_s` segundos. Se lee en vivo. |

## `[search]` (la búsqueda en el espacio de trabajo)

Qué recorre `gitpoulpi search` (y su herramienta MCP). Detalles: [Espacios de trabajo y búsqueda en la CLI](cli-reference.md#espacios-de-trabajo-y-búsqueda).

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (los más recientes primero, sobre todas las ramas, ramas remotas y tags) cuyos mensajes conserva en memoria el índice de búsqueda por repositorio, unos 100 bytes más el mensaje cada uno. Un historial mayor se corta ahí y la respuesta dice que es `partial`. `0` desactiva la búsqueda de commits (las ramas, tags, stashes y worktrees se siguen buscando). Se lee en vivo: la siguiente búsqueda respeta un cambio. |

## `[updates]` (buscar una versión más reciente)

Lo que hace el chip de versión de la barra de título con las versiones más recientes. Detalles: [Actualizaciones](installation.md#actualizaciones).

| Clave | Por defecto | Valores y efecto |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` pregunta a GitHub una vez al día, en segundo plano, si hay una versión más reciente, y convierte el chip de versión en `vX.Y.Z disponible`. `"auto"` además descarga y prepara la actualización por sí solo cuando esta compilación procede de un paquete de versión (aun así pregunta antes de reiniciar); en cualquier otra compilación se comporta como `"notify"`. `"off"` nunca hace ninguna petición. Se aplica al instante. |
| `updates.channel` | `"stable"` | `"stable"` solo considera versiones finales; `"prerelease"` también ofrece versiones candidatas (`-rc.1`). Se aplica al instante. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | El `owner/name` de github.com cuyas versiones se comprueban. Mientras ese repositorio sea privado, GitHub responde 404 salvo que haya un token guardado para github.com (Ajustes > Integraciones); sin él, la comprobación falla en silencio y escribe una línea en el registro de la aplicación. |

## `[keybindings]`

Reasigna los atajos de la TUI; Ajustes > Teclado y ratón lo edita.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # por defecto Ctrl+G
nav_prefix = "Alt+n"             # por defecto Ctrl+N
settings   = ["Ctrl+,", "F2"]    # una lista = alternativas
tab_menu   = []                  # sin asignar

[keybindings.git_prefix]         # la tecla que se pulsa tras el prefijo de git
stage_file = "z"
```

`[keybindings.<contexto>] <acción> = "<atajo>"`; un valor sustituye los valores por
defecto de esa acción, y quitar la línea los restaura. Atajos: `Ctrl+Shift+N`, `Alt+n`,
`F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Una entrada incorrecta nunca bloquea
nada: la TUI la ignora, conserva el valor por defecto y lo indica en la barra de estado.
Contextos, ids de acción, valores por defecto y reglas de conflicto: [Atajos de teclado](keybindings.md).

## Ajustes por repositorio: `.gitpoulpi/repo-state.toml`

Se guarda dentro del repositorio; la carpeta se ignora a sí misma (un `.gitignore` con `*`), así que
nunca aparece como un cambio. Lo escriben la TUI y la CLI; rara vez lo editarás.

| Clave | Qué es | Se fija con |
|---|---|---|
| `[preferences] encoding` | Anula aquí `git.default_encoding` | Ajustes > Git y codificación |
| `[preferences] lfs_enabled` | Anula aquí `git.lfs_enabled` | Ajustes > Git y codificación |
| `[preferences] profile` | El id de `[[profiles]]` que firma aquí los commits; ausente significa `general.active_profile` | el combobox de perfil de la barra de herramientas, **Repositorio: Cambiar perfil...** |
| `[preferences] autocrlf`, `eol` | *Aún no se usan* | Ajustes > Git y codificación |
| `[preferences] push_after_commit` | Hacer push de la rama a su upstream después de cada commit (desactivado si falta) | Ajustes > Ajustes del repositorio > Commit |
| `[preferences] skip_hooks_by_default` | La casilla Omitir hooks del formulario de commit empieza marcada (desactivado si falta) | Ajustes > Ajustes del repositorio > Commit |
| `[preferences] squash_merge` | El merge de una rama local hace squash: deja los cambios en stage, sin commit de merge (desactivado si falta) | Ajustes > Ajustes del repositorio > Opciones de merge |
| `[preferences] template_apply`, `template_strip_comments` | Rellenar con la plantilla un formulario de commit vacío; quitar las líneas de comentario `#` al hacer el commit (ambos desactivados si faltan) | Ajustes > Ajustes del repositorio > Plantilla de mensaje |
| `[preferences] conflict_detection` | Detección proactiva de conflictos de este repositorio; si falta, sigue `general.conflict_detection` | Ajustes > Ajustes del repositorio > Prevención de conflictos |
| `[preferences] conflict_base_branches` | Los patrones de Ramas base, uno por línea (`**` si falta) | Ajustes > Ajustes del repositorio > Prevención de conflictos |
| `commit-template.txt` (un archivo junto a `repo-state.toml`) | La plantilla de commit; el `commit.template` de git apunta a él | Ajustes > Ajustes del repositorio > Plantilla de mensaje |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: las instrucciones de IA de este repositorio | Ajustes > Ajustes del repositorio > IA |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` para una acción aquí | Ajustes > Ajustes del repositorio > IA |
| `default_remote` | El remoto que usan Push/Pull/Fetch de un clic | menú de remotos, `gitpoulpi remote default <name>` |
| `hosting_integration` | El perfil de `[[integrations]]` fijado aquí | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | El proyecto del proveedor que leen las listas de PR / issues, como `{ integration, project, filter }`. `filter` es opcional: una cláusula extra nativa del proveedor (un fragmento JQL de Jira, una condición WIQL de Azure, nombres de listas de Trello) unida con AND a la consulta de la lista; GitHub y GitLab lo ignoran. | secciones de la barra lateral, `gitpoulpi hosting link` |
| `ref_visibility` | Refs ocultas / en solo | barra lateral Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotos ocultos del grafo enteros (todas sus ramas, también las que se obtengan después) | el ojo y el menú del remoto, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Secciones de la barra lateral y carpetas de ramas y de tags contraídas. `[folds]` también guarda `status_folders` (carpetas contraídas de los árboles de archivos Unstaged / Staged) y `commit_folders` (carpetas contraídas del árbol de archivos cambiados de un commit, igual para todos los commits), cada uno limitado a 500 rutas, descartando primero las más antiguas | la barra lateral, los árboles de archivos |
| `pending_auto_stash` | Un stash que GitPoulpi hizo y aún no ha devuelto | lo escribe el daemon; ver [Auto-stash](log-and-history.md#auto-stash) |

## Otras variables de entorno

| Variable | Efecto |
|---|---|
| `GITPOULPI_CONFIG_DIR` | La carpeta de configuración (ver arriba). |
| `GITPOULPI_TOKEN_<ID>` | Un token de alojamiento para el perfil `<ID>`, con prioridad sobre el guardado (CI, uso puntual). |
| `GITPOULPI_GIT` | El ejecutable `git` a usar (por defecto: `git` del `PATH`; en Windows el verdadero `mingw64\bin\git.exe` tras el shim `cmd\git.exe` de Git for Windows, que ahorra un proceso por llamada). |
| `GITPOULPI_LANG=fr` | El idioma de la interfaz, con prioridad sobre `ui.language` (cualquier valor de `ui.language`; `qps` es un pseudo-idioma para traductores). Un valor desconocido o vacío se ignora. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Ninguna imagen, diga lo que diga la configuración. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Indicaciones de resolución de imagen (tamaño de celda en píxeles; sobremuestreo, 2 por defecto, `1` ahorra CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nunca lanza un navegador web (la URL se muestra igualmente). |
| `GITPOULPI_TIMING=1` | Escribe los tiempos de las peticiones en `gitpoulpi-timing.log` en la carpeta temporal (informes de rendimiento). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | No espera a Enter tras salir la TUI por un error. |
