[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · **Español** · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Atajos de teclado

Esta página lista los atajos **por defecto**. Puedes reasignar la mayoría. La
aplicación siempre muestra las teclas en vigor: en la paleta de comandos, en la
ayuda de `?`, en las barras de estado, en los menús contextuales y en las
descripciones emergentes de los botones. Cuando cambias una tecla, esos lugares
muestran la nueva. Para el ratón, consulta [Ratón y gestos](mouse-and-gestures.md).

## Reasignar atajos

Abre **Ajustes > Teclado y ratón**. La página se describe en
[Ajustes](settings.md#teclado-y-ratón). Los cambios se aplican al instante y se
guardan en la tabla `[keybindings]` de `config.toml`. También puedes editar esa
tabla a mano:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

La sintaxis, las reglas de conflicto y los avisos están en la
[Referencia de configuración](config-reference.md) bajo `[keybindings]`. En
resumen:

- Un error se notifica en la barra de estado, y se conserva la tecla por
  defecto.
- Dos acciones en un mismo acorde dentro del mismo contexto se rechazan.
- Un prefijo necesita `Ctrl` o `Alt`.
- Un acorde de panel que oculta un acorde global solo genera un aviso.

La tabla siguiente lista todas las teclas que puedes reasignar. Algunas teclas
son fijas y no se pueden reasignar: el movimiento por listas (`j` `k` `h` `l`,
las flechas, `PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, los
dígitos (el prefijo de repetición), las teclas de edición de texto y las teclas
`Enter` / `Tab` / flechas de los popups. La paleta tiene
`App: Atajos de teclado...` y `Atajos: Restablecer todos...`.

## Atajos por defecto

Todas las teclas que puedes reasignar, con el id que debes usar en
`[keybindings.<contexto>]` y su valor por defecto. La tabla se genera a partir de
la aplicación (un test la mantiene al día). Las teclas posteriores a un prefijo
se muestran con el prefijo (`Ctrl+G s`).

<!-- keymap:begin -->
| Contexto | Id de acción | Qué hace | Teclas por defecto |
|---|---|---|---|
| `global` | `git_prefix` | Prefijo de acciones de git | `Ctrl+G` |
| `global` | `nav_prefix` | Prefijo de navegación | `Ctrl+N` |
| `global` | `palette` | Paleta de comandos | `Ctrl+P` |
| `global` | `settings` | Ajustes | `Ctrl+,` |
| `global` | `home` | Inicio (gestión de repositorios) | `Ctrl+O` |
| `global` | `clone` | Clonar repositorio | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nueva pestaña | `Alt+T` |
| `global` | `workspace_switch` | Cambiar de espacio de trabajo | `Alt+W` |
| `global` | `radar` | Abrir Radar | `Alt+R` |
| `global` | `workspaces` | Abrir espacios de trabajo | `Alt+J` |
| `global` | `tab_menu` | Menú de pestaña | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Abrir en (lista) | `Alt+O` |
| `global` | `tab_open_in_default` | Abrir en el predeterminado | `Ctrl+Alt+O` |
| `global` | `close_tab` | Cerrar pestaña | `Ctrl+W` |
| `global` | `next_tab` | Pestaña siguiente | `Ctrl+PageDown`, `Ctrl+Tab` |
| `global` | `prev_tab` | Pestaña anterior | `Ctrl+PageUp`, `Ctrl+Shift+Tab` |
| `global` | `help` | Ayuda | `?` |
| `global` | `quit` | Salir | `q` |
| `global` | `refresh` | Actualizar | `r` |
| `global` | `refresh_all` | Actualizar (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Buscar en el grafo de commits | `f` |
| `global` | `workspace_search` | Buscar en el espacio de trabajo | `Ctrl+Alt+F` |
| `global` | `select_tab` | Seleccionar pestaña | `t` |
| `global` | `undo` | Deshacer la última operación | `Ctrl+Z` |
| `global` | `redo` | Rehacer la operación deshecha | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Abrir el historial de deshacer | `Alt+Z` |
| `global` | `commit_column` | Mostrar u ocultar la columna de commits | `Alt+C` |
| `global` | `undo_offer` | Restaurar la rama a donde la dejó la acción bloqueada | `Alt+Shift+Z` |
| `global` | `context_menu` | Menú contextual | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Redactar mensaje de commit | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Hacer commit ya de los cambios en stage | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | IA: Dividir los cambios en commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | IA: Explicar los cambios | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash con mensaje | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | IA: Escribir mensaje de commit | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage de todos los cambios | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Descartar todos los cambios | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage del archivo seleccionado | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage del archivo seleccionado | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `pull_all_branches` | Pull de todas las ramas | `Ctrl+G Ctrl+P` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Abrir pull request vinculada | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push todos los tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch todos los remotos | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nuevo tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Rama nueva | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash de los cambios | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick del commit seleccionado | `Ctrl+G y` |
| `git_prefix` | `revert` | Revertir el commit seleccionado | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Editar mensaje del commit | `Ctrl+G e` |
| `git_prefix` | `amend` | Modificar el commit seleccionado | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Modificar el autor del commit seleccionado | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (mixto) al commit seleccionado | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase sobre la rama seleccionada | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop del stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Eliminar la ref seleccionada o descartar stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Abrir el menú LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Abrir el menú Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch de todos los objetos | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout de todos los archivos | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull de objetos | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push de todos los objetos | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Limpiar objetos locales | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Abrir el editor de reescritura del historial | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Contraer o restaurar el panel Refs | `Ctrl+N Shift+H` |
| `nav_prefix` | `collapse_status` | Contraer o restaurar el panel Estado | `Ctrl+N Shift+L` |
| `nav_prefix` | `settings` | Abrir ajustes | `Ctrl+N ,` |
| `nav_prefix` | `about` | Abrir Acerca de | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Abrir el Registro de actividad | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Abrir el registro de actividad global | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Mostrar las tareas en segundo plano | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Mostrar el panel de actualización | `Ctrl+N u` |
| `nav_prefix` | `help` | Alternar la ayuda | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Seleccionar pestaña | `Ctrl+N Tab` |
| `nav_prefix` | `change_theme` | Cambiar tema | `Ctrl+N t` |
| `nav_prefix` | `change_language` | Cambiar idioma | `Ctrl+N l` |
| `nav_prefix` | `change_graph_color_theme` | Cambiar tema de color del grafo | `Ctrl+N Shift+T` |
| `nav_prefix` | `toggle_graph_row_tints` | Alternar tinte de filas del grafo | `Ctrl+N Shift+Y` |
| `nav_prefix` | `grow_section` | Agrandar la sección de Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Reducir la sección de Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Agrandar el bloque de mensaje del commit | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Reducir el bloque de mensaje del commit | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Ensanchar el panel Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Estrechar el panel Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Ensanchar el panel Estado | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Estrechar el panel Estado | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agente: Mostrar la vista pendiente | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agente: Volver (o descartar la vista pendiente) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Organizar el diálogo abierto (mover, cambiar tamaño) | `Ctrl+N m` |
| `refs` | `filter` | Filtrar refs por nombre | `/` |
| `refs` | `solo` | Solo de la selección | `o` |
| `refs` | `clear_solo` | Quitar el solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Alternar la vista de jerarquía de ramas | `y` |
| `refs` | `show_all` | Mostrar todas las refs ocultas | `Shift+H` |
| `refs` | `reveal` | Mostrar en el grafo | `g` |
| `refs` | `mark_drop` | Marcar como origen de arrastre | `m` |
| `refs` | `drop_marked` | Soltar aquí la ref marcada | `Shift+M` |
| `refs` | `expand_all` | Expandir todas las carpetas | `=` |
| `refs` | `collapse_all` | Contraer todas las carpetas | `-` |
| `refs` | `expand_below` | Expandir la carpeta y sus subcarpetas | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Contraer la carpeta y sus subcarpetas | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Archivos: Ruta o árbol | `m` |
| `log` | `view_all_files` | Archivos: Ver todos los archivos | `a` |
| `log` | `compare_selection` | Comparar las filas seleccionadas | `v` |
| `log` | `compare_swap` | Intercambiar los extremos de la comparación | `x` |
| `log` | `compare_next` | Ir a la siguiente fila comparada | `n` |
| `log` | `pick_column` | Elegir una columna para moverla o ajustarla | `Shift+C` |
| `log` | `scroll_left` | Desplazar las columnas a la izquierda | `Shift+Left` |
| `log` | `scroll_right` | Desplazar las columnas a la derecha | `Shift+Right` |
| `log` | `copy_mermaid` | Copiar Commits como gitGraph de Mermaid | `Shift+Y` |
| `log_columns` | `next` | Elegir la columna siguiente | `Tab` |
| `log_columns` | `prev` | Elegir la columna anterior | `Shift+BackTab` |
| `log_columns` | `move_left` | Mover la columna a la izquierda | `<` |
| `log_columns` | `move_right` | Mover la columna a la derecha | `>` |
| `log_columns` | `wider` | Ensanchar la columna | `]` |
| `log_columns` | `narrower` | Estrechar la columna | `[` |
| `log_columns` | `reset_width` | Restablecer los anchos de las columnas | `0` |
| `log_columns` | `fit` | Ajustar la columna a su contenido | `=` |
| `log_columns` | `fit_all` | Ajustar todas las columnas a su contenido | `+` |
| `log_columns` | `layout` | Desplazar lateralmente o ajustar al panel | `s` |
| `log_files` | `collapse_all` | Contraer o expandir todas las carpetas | `z` |
| `log_files` | `filter` | Filtrar los archivos cambiados | `/` |
| `log_files` | `expand_all` | Expandir todas las carpetas | `=` |
| `log_files` | `collapse_all_folders` | Contraer todas las carpetas | `-` |
| `log_files` | `expand_below` | Expandir la carpeta y sus subcarpetas | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Contraer la carpeta y sus subcarpetas | `_`, `Shift+Left` |
| `status` | `compose` | Ir al mensaje del commit | `c` |
| `status` | `toggle_file_mode` | Archivos: Ruta o árbol | `m` |
| `status` | `view_all_files` | Archivos: Ver todos los archivos | `a` |
| `status` | `amend` | Alternar enmienda | `Shift+A` |
| `status` | `skip_hooks` | Alternar omitir hooks | `Shift+H` |
| `status` | `collapse_all` | Contraer o expandir todas las carpetas | `z` |
| `status` | `stage_toggle` | Stage o unstage de la fila seleccionada | `Ctrl+Space` |
| `status` | `open_diff` | Abrir el diff del archivo seleccionado | `d` |
| `status` | `expand_all` | Expandir todas las carpetas | `=` |
| `status` | `collapse_all_folders` | Contraer todas las carpetas | `-` |
| `status` | `expand_below` | Expandir la carpeta y sus subcarpetas | `*`, `Shift+Right` |
| `status` | `collapse_below` | Contraer la carpeta y sus subcarpetas | `_`, `Shift+Left` |
| `home` | `favorite` | Marcar o desmarcar el repositorio como favorito | `f`, `*` |
| `home` | `close_repo` | Cerrar, quitar o desmarcar la fila | `x`, `Delete` |
| `home` | `copy_path` | Copiar la ruta del repositorio | `y` |
| `home` | `open_in` | Abrir el repositorio en la herramienta predeterminada | `o` |
| `home` | `browse` | Examinar en busca de un repositorio | `b` |
| `home` | `clone` | Clonar un repositorio | `c` |
| `home` | `init` | Inicializar un repositorio | `i` |
| `home` | `collapse_all` | Contraer todas las secciones | `-` |
| `home` | `expand_all` | Expandir todas las secciones | `+`, `=` |
| `home` | `search` | Buscar en los repositorios | `/` |
| `home` | `workspace_new` | Nuevo espacio de trabajo | `n` |
| `home` | `workspace_edit` | Editar el espacio de trabajo | `e` |
| `home` | `workspace_from_tabs` | Nuevo espacio desde las pestañas abiertas | `a` |
| `home` | `workspace_fetch` | Fetch todo en el espacio de trabajo | `Shift+F` |
| `home` | `workspace_pull` | Pull todo en el espacio de trabajo | `Shift+P` |
| `home` | `workspace_clone_missing` | Clonar los repositorios ausentes | `m` |
| `home` | `workspace_locate` | Localizar el repositorio ausente | `l` |
| `refs` | `hosting_filter` | Pull requests / Issues: elegir el filtro | `v` |
| `refs` | `hosting_edit` | Pull request: editar en el host | `e` |
| `refs` | `new_pull_request` | Pull requests: crear una | `n` |
| `refs` | `add_remote` | Remotos: Añadir un remoto | `+` |
| `refs` | `remotes` | Rama: Editar sus remotos | `u` |
| `diff` | `close` | Cerrar el diff | `q` |
| `diff` | `edit` | Editar en el directorio de trabajo | `e` |
| `diff` | `lfs_raw` | LFS: Tarjeta o puntero en bruto | `r` |
| `diff` | `copy` | Copiar la selección o la línea | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage o unstage del hunk | `Space` |
| `diff` | `stage_lines` | Stage o unstage de las líneas seleccionadas | `Enter` |
| `diff` | `discard_hunk` | Descartar el hunk | `x` |
| `diff` | `discard_lines` | Descartar las líneas seleccionadas | `Shift+X` |
| `diff` | `select_lines` | Iniciar o soltar una selección de líneas | `v` |
| `diff` | `next_hunk` | Hunk siguiente | `n` |
| `diff` | `prev_hunk` | Hunk anterior | `p` |
| `diff` | `toggle_layout` | Disposición: Unificada o lado a lado | `m` |
| `diff` | `toggle_hunks_only` | Solo hunks o archivo entero | `Shift+H` |
| `diff` | `toggle_minimap` | Alternar el minimapa | `Shift+M` |
| `diff` | `choose_encoding` | Elegir la codificación | `Shift+C` |
| `diff` | `toggle_wrap` | Alternar el ajuste de línea | `w` |
| `diff` | `toggle_whitespace` | Mostrar u ocultar cambios solo de espacios en blanco | `s` |
| `diff` | `merge_editor` | Abrir el editor de merge (archivo en conflicto) | `Shift+E` |
| `file_insight` | `close` | Cerrar la vista de archivo | `q` |
| `file_insight` | `edit` | Editar en el directorio de trabajo | `e` |
| `file_insight` | `copy` | Copiar la selección o la fila | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Ir a la última fila | `Shift+G` |
| `file_insight` | `open_commit` | Ir al commit de la fila | `Enter` |
| `history` | `show_in_log` | Mostrar en el Log | `Enter` |
| `history` | `copy_sha` | Copiar el SHA del commit | `y` |
| `history` | `copy_summary` | Copiar el resumen | `Shift+Y` |
| `history` | `show_diff` | Mostrar el diff de este commit | `d`, `Alt+1` |
| `history` | `open_file` | Abrir el archivo en este commit | `o`, `Alt+2` |
| `history` | `blame` | Blame en este commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Comparar con el árbol de trabajo | `c` |
| `history` | `restore` | Restaurar esta versión | `Shift+R` |
| `history` | `range_up` | Ampliar la selección hacia arriba | `Shift+Up` |
| `history` | `range_down` | Ampliar la selección hacia abajo | `Shift+Down` |
| `history` | `switch_focus` | Alternar entre la lista y el diff | `Tab` |
| `history` | `widen_list` | Ensanchar la lista | `Alt+Right` |
| `history` | `narrow_list` | Estrechar la lista | `Alt+Left` |
| `history` | `reset_split` | Restablecer el ancho de la lista | `Alt+0` |
| `commit_column` | `scroll_left` | Desplazar la columna a la izquierda | `Shift+Left` |
| `commit_column` | `scroll_right` | Desplazar la columna a la derecha | `Shift+Right` |
| `commit_column` | `next_part` | Dimensionar la parte siguiente | `Tab` |
| `commit_column` | `move_left` | Mover la parte a la izquierda | `<` |
| `commit_column` | `move_right` | Mover la parte a la derecha | `>` |
| `commit_column` | `wider_part` | Ensanchar la parte | `]` |
| `commit_column` | `narrower_part` | Estrechar la parte | `[` |
| `commit_column` | `reset_widths` | Restablecer los anchos de las partes | `0` |
| `commit_column` | `fit` | Ajustar la columna a su contenido | `=` |
| `commit_column` | `fit_all` | Ajustar todas las columnas a su contenido | `+` |
| `file_insight` | `blame_card` | Blame: Mostrar u ocultar la tarjeta de detalle | `i` |
| `file_insight` | `blame_next` | Blame: Hunk siguiente | `n` |
| `file_insight` | `blame_prev` | Blame: Hunk anterior | `p` |
| `file_insight` | `blame_next_same` | Blame: Hunk siguiente del mismo commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Hunk anterior del mismo commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Ir a la línea | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Copiar el SHA del commit | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Diff del commit para este archivo | `d` |
| `file_insight` | `blame_parent` | Blame: Blame en el padre del commit | `b` |
| `file_insight` | `blame_history` | Blame: Abrir el historial del archivo | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Ensanchar la columna de anotaciones | `>` |
| `file_insight` | `blame_narrower` | Blame: Estrechar la columna de anotaciones | `<` |
| `image` | `zoom_in` | Acercar la imagen | `+`, `=` |
| `image` | `zoom_out` | Alejar la imagen | `-` |
| `image` | `fit` | Ajustar la imagen a la vista | `0` |
| `image` | `actual_size` | Mostrar la imagen a tamaño real | `1` |
| `image` | `flip_side` | Mostrar el otro lado (antes / después) | `Tab` |
| `image` | `widen_before` | Ensanchar el panel Antes | `Alt+Right` |
| `image` | `widen_after` | Ensanchar el panel Después | `Alt+Left` |
| `image` | `reset_split` | Restablecer la división antes / después | `Alt+0` |
| `image` | `edit_in` | Abrir con... | `Shift+O` |
| `image` | `edit_default` | Abrir en la aplicación predeterminada | `o` |
| `editor` | `save` | Guardar | `Ctrl+S` |
| `editor` | `undo` | Deshacer | `Ctrl+Z` |
| `editor` | `redo` | Rehacer | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Buscar | `Ctrl+F` |
| `editor` | `find_next` | Buscar siguiente (Shift: anterior) | `F3` |
| `editor` | `paste` | Pegar | `Ctrl+V` |
| `editor` | `toggle_minimap` | Alternar el minimapa | `Alt+M` |
| `editor_prompt` | `save` | Cambios sin guardar: Guardar y cerrar | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Cambios sin guardar: Descartar y cerrar | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Cambios sin guardar: Seguir editando | `c`, `Shift+C` |
| `merge` | `next_conflict` | Conflicto siguiente | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Conflicto anterior | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Archivo en conflicto siguiente | `Alt+PageDown` |
| `merge` | `prev_file` | Archivo en conflicto anterior | `Alt+PageUp` |
| `merge` | `next_pane` | Ir al panel siguiente | `F6` |
| `merge` | `prev_pane` | Ir al panel anterior | `Shift+F6` |
| `merge` | `toggle_panes` | Mostrar u ocultar los paneles laterales (Refs y Estado) | `F9` |
| `merge` | `toggle_base` | Mostrar u ocultar el panel Base | `Alt+D` |
| `merge` | `toggle_full_file` | Alternar el diff del archivo entero | `Alt+F` |
| `merge` | `undo` | Deshacer (elecciones y escritura) | `Ctrl+Z` |
| `merge` | `redo` | Rehacer (elecciones y escritura) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Ensanchar el panel Actual (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Ensanchar el panel Entrante (B) | `Alt+Left` |
| `merge` | `reset_panes` | Restablecer los tamaños de panel | `Alt+0` |
| `merge` | `save` | Guardar sin marcar como resuelto | `Alt+S` |
| `merge` | `save_resolve` | Guardar y marcar como resuelto | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Sugerir una resolución (IA) | `Alt+I` |
| `merge` | `unpick` | Deshacer la elección de este conflicto (sin resolver) | `Alt+U` |
| `merge` | `reset_all` | Devolver todo el archivo a sus conflictos originales | `Alt+R` |
| `merge` | `focus_a` | Ir al panel Actual (A) | `Alt+1` |
| `merge` | `focus_b` | Ir al panel Entrante (B) | `Alt+2` |
| `merge` | `focus_output` | Ir al panel Salida | `Alt+3` |
| `merge` | `focus_base` | Ir al panel Base | `Alt+4` |
| `merge_sides` | `toggle_line` | Elegir o soltar la línea | `Space` |
| `merge_sides` | `toggle_chunk` | Elegir o soltar el bloque entero | `Enter` |
| `merge_sides` | `take_a` | Tomar solo Actual (A) | `a` |
| `merge_sides` | `take_b` | Tomar solo Entrante (B) | `b` |
| `merge_sides` | `take_both` | Tomar ambos, A y luego B | `c` |
| `merge_sides` | `take_both_reversed` | Tomar ambos, B y luego A | `Shift+C` |
| `merge_sides` | `clear` | No tomar nada en este conflicto | `x` |
| `merge_sides` | `unpick` | Deshacer la elección de este conflicto (sin resolver) | `u` |
| `merge_sides` | `reset_all` | Devolver todo el archivo a sus conflictos originales | `Shift+U` |
| `merge_sides` | `swap_order` | Intercambiar el orden de los dos lados | `s` |
| `merge_sides` | `next_conflict` | Conflicto siguiente | `n` |
| `merge_sides` | `prev_conflict` | Conflicto anterior | `Shift+N` |
| `merge_sides` | `take_all_a` | Tomar todo Actual (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Tomar todo Entrante (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Mostrar u ocultar el panel Base | `d` |
| `merge_sides` | `scroll_left` | Desplazar los paneles a la izquierda | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Desplazar los paneles a la derecha | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Alternar el diff del archivo entero | `f` |
| `merge_sides` | `cycle_height` | Recorrer las alturas del resultado | `z` |
| `merge_card` | `take_a` | Quedarse con A (Actual) tal cual | `a` |
| `merge_card` | `take_b` | Tomar B (Entrante) tal cual | `b` |
| `merge_card` | `take_base` | Tomar la Base (ancestro común) | `o` |
| `merge_card` | `keep` | Conservar el archivo que queda | `k` |
| `merge_card` | `delete` | Aceptar la eliminación | `Shift+D` |
| `merge_card` | `external_tool` | Abrir en la herramienta de merge externa | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (parar para modificar) | `e` |
| `rebase` | `autosquash` | Alternar autosquash | `a` |
| `rebase` | `start` | Iniciar el plan | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Restablecer el plan | `Ctrl+R` |
| `rebase` | `split` | Dividir el commit de la fila | `t` |
| `rebase` | `move_up` | Subir la fila | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Bajar la fila | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Crear los commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Alternar la selección por líneas | `v` |
| `split` | `move_up` | Subir el commit | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Bajar el commit | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Restablecer la división | `Shift+R` |
| `rewrite` | `preview` | Generar la vista previa de la reescritura | `F5` |
| `rewrite` | `apply` | Aplicar la reescritura | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Cancelar la vista previa | `Ctrl+X` |
| `rewrite` | `add_rule` | Añadir una regla | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Activar o desactivar la regla | `Space` |
| `rewrite` | `move_up` | Subir la regla | `Alt+Up` |
| `rewrite` | `move_down` | Bajar la regla | `Alt+Down` |
| `rewrite` | `delete_rule` | Eliminar la regla | `d`, `Delete` |
| `rewrite` | `restore_rule` | Restaurar la regla eliminada | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplicar la regla | `Ctrl+D` |
| `rewrite` | `next_tab` | Pestaña siguiente de la vista previa | `Alt+Right` |
| `rewrite` | `prev_tab` | Pestaña anterior de la vista previa | `Alt+Left` |
| `rewrite` | `save_shared` | Guardar la receta en el repositorio | `Alt+S` |
| `rewrite` | `save_mine` | Guardar la receta solo para mí | `Alt+M` |
| `rewrite` | `load` | Cargar una receta | `Alt+L` |
| `rewrite` | `name` | Renombrar la receta | `F2` |
| `rewrite` | `protected_all` | Incluir todas las ramas protegidas | `Alt+A` |
| `rewrite` | `protected_none` | Excluir todas las ramas protegidas | `Alt+U` |
| `rewrite` | `check_tools` | Volver a comprobar las herramientas | `F6` |
| `rewrite` | `analyze` | Analizar los tamaños del historial | `F7` |
| `rewrite` | `host_howto` | Explicar la protección de ramas del host | `Alt+H` |
| `rewrite` | `ai_rules` | Crear reglas de reescritura con IA | `Alt+I` |
| `reword` | `apply` | Aplicar el mensaje nuevo | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modificar el historial más a fondo | `Alt+H` |
| `composer` | `commit` | Commit desde el compositor | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Insertar una referencia a un issue | `Alt+R` |
| `md_editor` | `preview` | Editor de Markdown: Alternar Escribir y Previa | `Alt+P` |
| `md_editor` | `split` | Editor de Markdown: Alternar la vista dividida | `Alt+L` |
| `md_editor` | `submit` | Editor de Markdown: Enviar el texto | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Editor de Markdown: Escribir con IA | `Alt+E` |
| `dialog` | `browse` | Examinar en busca de una carpeta | `Ctrl+B` |
| `dialog` | `sources` | Ir a los orígenes de clonación | `Alt+S` |
| `dialog` | `submit` | Enviar el formulario de pull request | `Ctrl+S` |
| `dialog` | `web` | Abrir el formulario web del proveedor | `Alt+W` |
| `folder_picker` | `choose` | Elegir la carpeta | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Carpeta nueva | `Alt+N` |
| `folder_picker` | `filter` | Filtrar las entradas de la carpeta | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Mostrar u ocultar carpetas hermanas | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Mostrar u ocultar carpetas ocultas | `Alt+H` |
| `preferences` | `search` | Buscar en los ajustes | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Añadir un elemento a la lista | `Insert` |
| `preferences` | `list_duplicate` | Duplicar el elemento seleccionado de la lista | `Ctrl+D` |
| `preferences` | `list_remove` | Quitar el elemento seleccionado de la lista | `Delete` |
| `preferences` | `list_move_up` | Subir el elemento seleccionado de la lista | `Alt+Up` |
| `preferences` | `list_move_down` | Bajar el elemento seleccionado de la lista | `Alt+Down` |
| `preferences` | `prev_tab` | Pestaña anterior de la página de ajustes | `Alt+Left` |
| `preferences` | `next_tab` | Pestaña siguiente de la página de ajustes | `Alt+Right` |
| `ai_popup` | `copy` | Copiar el resultado | `c` |
| `activity_log` | `scroll_left` | Desplazar el registro de actividad a la izquierda | `Left` |
| `activity_log` | `scroll_right` | Desplazar el registro de actividad a la derecha | `Right` |
| `integrations` | `add` | Añadir un perfil | `a` |
| `integrations` | `edit` | Editar el perfil seleccionado | `e` |
| `integrations` | `open_token` | Abrir la página del token | `o` |
| `integrations` | `remove` | Quitar el perfil seleccionado | `d`, `Delete` |
| `prompt` | `ai_stash_message` | IA: Generar mensaje de stash | `Ctrl+G` |
<!-- keymap:end -->

### Teclas que el terminal no puede enviar, y el protocolo de teclado de kitty

Un terminal antiguo no puede enviar algunos acordes: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` y `Ctrl+<dígito>`. No envía nada, o envía otra tecla: por ejemplo,
`Ctrl+Shift+N` llega como `Ctrl+N`, `Ctrl+I` como `Tab` y `Ctrl+M` como
`Enter`. La página de Ajustes avisa de una asignación así («este terminal no
envía ...»).

Los terminales con el **protocolo de teclado de kitty** pueden enviar estos
acordes. Ejemplos: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 y posteriores,
y zellij (tmux no responde: consulta [Multiplexores](multiplexers.md#tmux)). GitPoulpi usa el protocolo por
defecto:

- Al arrancar, GitPoulpi pregunta a el terminal y espera como mucho unos 100 ms
  la respuesta. Si el terminal no responde, se conservan las teclas clásicas.
- Si el terminal responde, GitPoulpi activa el protocolo. Lo desactiva de nuevo
  al salir, al fallar o al bloquearse.
- Los avisos de la página de Ajustes siguen al protocolo: cuando está activado,
  los acordes anteriores se pueden enviar.
- **Ajustes > Teclado y ratón > Protocolo de teclado de Kitty**
  (`ui.kitty_keyboard`) en `off` nunca cambia el modo de teclado de el terminal.
  Úsalo si un terminal o un multiplexor se comporta mal. Requiere reiniciar.

Las consolas de Windows informan por sí mismas de los modificadores y no
necesitan el protocolo.

## Paleta de comandos (`Ctrl+P`)

La paleta lista **todos** los comandos de la aplicación, así que da acceso a
todo sin necesidad de atajo. La lista completa está
[más abajo](#todos-los-comandos-de-la-paleta).

| Tecla | Acción |
|---|---|
| `Ctrl+P` | Abrir la paleta |
| *(escribir)* | Filtrar la lista. Primero va una coincidencia exacta tras el prefijo `Categoría: `, luego una coincidencia en cualquier punto de la etiqueta y después una coincidencia aproximada |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Mover el resaltado (consulta [Listas emergentes filtrables](#listas-emergentes-filtrables)) |
| `Enter`, o un doble clic | Ejecutar el comando resaltado |
| `Esc` | Cerrar |

`j` y `k` se escriben en el filtro. No mueven el resaltado.

Cada fila muestra su atajo a la derecha. Un comando que no se aplica ahora (por
ejemplo, sin ninguna ref seleccionada) sigue en la lista, en gris, y el motivo
aparece debajo de la lista. Las etiquetas empiezan por una categoría, como
`Navegación:`, `Refs:`, `Git:`, `Ver:`, `Diff:` o `App:`. Escribe una categoría
para ver todo su grupo.

### Una tecla para cualquier comando

Cada fila de la paleta termina con un botón de tecla (`[k]`), y `Alt+K` hace lo mismo en la fila resaltada: abre **Ajustes > Teclado y ratón** filtrado por el atajo de ese comando, listo para reasignarlo. Un comando sin atajo propio recibe uno aquí: aparece en **Comandos de la paleta**, sin tecla por defecto, y la tecla que le des ejecuta el comando igual que al elegirlo en la paleta (un comando que ahora no puede ejecutarse dice por qué). La fila muestra entonces tu tecla.

En `config.toml` es la tabla `[keybindings.palette]`, con el nombre del comando (`toggle_minimap = "Ctrl+Alt+M"`). Una tecla que choca con otra acción se muestra en rojo, y `c` en su fila lista todas las acciones con las que choca.

La banda de ayuda bajo la lista indica esta tecla (`Alt+K` por defecto). Es la acción `palette.configure_shortcut`, así que se reasigna como cualquier otra.

La paleta y los selectores (tema, idioma, pestaña, búsqueda del espacio de trabajo) se mueven y redimensionan como un diálogo: arrastra el borde superior, pulsa `Ctrl+N` y luego `m` (como sobre cualquier diálogo), o pulsa `Alt+G`. `Alt+G` es la acción `palette.arrange` («Diálogo: Mover o cambiar de tamaño el diálogo en primer plano»): una vía más, válida sobre cualquier diálogo y popup, para iniciar el modo de organizar el que está en primer plano (las filas `Diálogo: Mover o cambiar de tamaño` y `Diálogo: Restablecer posición y tamaño` de la paleta actúan sobre la propia paleta mientras está abierta). Las flechas lo mueven, `Shift`+flechas cambian su tamaño, `0` lo restablece, `Enter` o `Esc` salen del modo y el popup sigue abierto. `ui.dialog_geometry` decide si se conserva el lugar.

`Ctrl+N` es aquí también el prefijo de navegación: muestra una insignia en el cuadro de filtro, la primera fila de la lista y la cruz de cierre, y la tecla que le sigue pulsa ese control o `m` organiza el popup. Ya no baja el resaltado, y `Ctrl+P` no lo sube: solo las flechas, `PageUp` / `PageDown` y `Home` / `End` mueven una lista con campo de filtro.

### Todos los comandos de la paleta

Generada a partir de la aplicación (un test la mantiene al día). Los comandos de
IA solo aparecen mientras la IA está activada; los demás aparecen en gris, con el
motivo, cuando no se aplican. Las teclas son las de por defecto.

<!-- palette:begin -->
| Comando | Tecla por defecto |
|---|---|
| Navegación: Ir al panel Refs | `Ctrl+N <badge>` |
| Navegación: Ir al panel Log | `Ctrl+N <badge>` |
| Navegación: Ir al panel Estado | `Ctrl+N <badge>` |
| Navegación: Ir al filtro de Refs | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Locales | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Remotas | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Worktrees | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Submódulos | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Stashes | `Ctrl+N <badge>` |
| Navegación: Ir a Refs > Tags | `Ctrl+N <badge>` |
| Panel izquierdo: Mostrar u ocultar secciones... |  |
| Panel izquierdo: Restablecer las secciones |  |
| Panel izquierdo: Usar estas secciones en todos los repositorios |  |
| Panel izquierdo: Mostrar u ocultar ramas locales |  |
| Panel izquierdo: Mostrar u ocultar remotos |  |
| Panel izquierdo: Mostrar u ocultar worktrees |  |
| Panel izquierdo: Mostrar la sección Worktrees |  |
| Panel izquierdo: Mostrar u ocultar stashes |  |
| Panel izquierdo: Mostrar u ocultar tags |  |
| Panel izquierdo: Mostrar u ocultar pull requests |  |
| Panel izquierdo: Mostrar u ocultar issues |  |
| Panel izquierdo: Mostrar u ocultar submódulos |  |
| Navegación: Ir a Estado > Unstaged | `Ctrl+N <badge>` |
| Navegación: Ir a Estado > Staged | `Ctrl+N <badge>` |
| Navegación: Pasar el foco al panel siguiente | `Tab` |
| Navegación: Pasar el foco al panel anterior | `Shift+Tab` |
| Navegación: Buscar commits... | `f` |
| Navegación: Contraer o expandir el panel Refs | `Ctrl+N H` |
| Navegación: Ensanchar el panel Refs | `Ctrl+N Right` |
| Navegación: Estrechar el panel Refs | `Ctrl+N Left` |
| Navegación: Ensanchar el panel Estado | `Ctrl+N Shift+Left` |
| Navegación: Estrechar el panel Estado | `Ctrl+N Shift+Right` |
| Navegación: Contraer o expandir el panel Estado | `Ctrl+N L` |
| Navegación: Agrandar la sección de Refs | `Ctrl+N +` |
| Navegación: Reducir la sección de Refs | `Ctrl+N -` |
| Navegación: Restablecer los tamaños de sección de Refs |  |
| Navegación: Restablecer el diseño |  |
| Detalles del commit: Agrandar el mensaje | `Ctrl+N ]` |
| Detalles del commit: Reducir el mensaje | `Ctrl+N [` |
| Detalles del commit: Restablecer el tamaño del mensaje |  |
| Detalles del commit: Desplazar el mensaje hacia abajo |  |
| Detalles del commit: Desplazar el mensaje hacia arriba |  |
| Detalles del commit: Abrir enlace del mensaje... |  |
| Archivos: Alternar el orden de archivos del commit (A-Z / Z-A) |  |
| Refs: Checkout de la ref seleccionada | `doble clic` |
| Refs: Copiar nombre de la ref |  |
| Refs: Copiar ref completa (refs/heads/...) |  |
| Refs: Copiar SHA de destino de la ref |  |
| Refs: Merge de la rama seleccionada en la actual |  |
| Refs: Rebase de la rama actual sobre la seleccionada | `Ctrl+G r` |
| Refs: Soltar la ref seleccionada sobre la rama activa... |  |
| Refs: Marcar como origen de arrastre | `m` |
| Refs: Soltar aquí la ref marcada... | `M` |
| Refs: Renombrar la rama seleccionada... |  |
| Refs: Fast-forward de la rama seleccionada a la actual |  |
| Refs: Fast-forward de la rama actual a la seleccionada |  |
| Refs: Crear worktree desde la rama o el tag seleccionados... |  |
| Refs: Reflog de la rama seleccionada... |  |
| Refs: Iniciar pull request desde la rama seleccionada... |  |
| Refs: Abrir worktree en una pestaña | `Enter / doble clic` |
| Refs: Abrir worktree en... |  |
| Refs: Merge de la rama del worktree en... |  |
| Refs: Eliminar worktree... |  |
| Refs: Crear worktree... | `+` |
| Refs: Crear worktree desde la rama actual |  |
| Refs: Crear worktree eligiendo una rama |  |
| Refs: Actualizar submódulo (seleccionado) |  |
| Refs: Inicializar submódulo (seleccionado) |  |
| Refs: Abrir submódulo en una pestaña | `Enter / doble clic` |
| Git: Actualizar todos los submódulos |  |
| Git: Crear worktree en el commit seleccionado... |  |
| Git: Crear parche desde el commit seleccionado... |  |
| Git: Crear parche desde el commit seleccionado hasta HEAD... |  |
| Git: Crear parche desde los cambios del árbol de trabajo... |  |
| Reescritura: Purgar el historial antiguo... |  |
| Git: Aplicar parche... |  |
| Reescritura: Editor del historial | `Ctrl+G H` |
| Reescritura: Reemplazar un autor... |  |
| Reescritura: Anonimizar autores... |  |
| Reescritura: Aplicar .mailmap al historial... |  |
| Reescritura: Merge de identidades (.mailmap)... |  |
| Reescritura: Desplazar las fechas de los commits... |  |
| Reescritura: Cambiar la zona horaria de los commits... |  |
| Reescritura: Limitar los commits a un horario... |  |
| Reescritura: Igualar la fecha del committer a la del autor |  |
| Reescritura: Buscar y reemplazar en mensajes... |  |
| Reescritura: Eliminar líneas de mensajes... |  |
| Reescritura: Añadir o quitar trailers... |  |
| Reescritura: Quitar firmas... |  |
| Reescritura: Corregir la codificación de mensajes... |  |
| Reescritura: Buscar y reemplazar en archivos... |  |
| Reescritura: Quitar archivos del historial... |  |
| Reescritura: Quitar archivos grandes del historial... |  |
| Reescritura: Mover o renombrar rutas... |  |
| Reescritura: Migrar archivos a LFS... |  |
| Reescritura: Recuperar archivos de LFS... |  |
| Reescritura: Convertir fines de línea... |  |
| Reescritura: Quitar el bit de ejecución... |  |
| Reescritura: Convertir la codificación de archivos... |  |
| Reescritura: Ejecutar un script... |  |
| ✦ Reescritura: Escribir un script con IA... |  |
| ✦ Reescritura: Crear reglas con IA... |  |
| Reescritura: Desde el commit seleccionado hasta HEAD... |  |
| Reescritura: Commits seleccionados... |  |
| LFS: Inicializar en este repositorio |  |
| LFS: Seguir patrón... |  |
| LFS: Dejar de seguir patrón... |  |
| LFS: Fetch de objetos |  |
| LFS: Abrir menú | `Ctrl+G l` |
| LFS: Fetch de todos los objetos | `Ctrl+G F` |
| LFS: Checkout de todos los archivos | `Ctrl+G K` |
| LFS: Push de todos los objetos | `Ctrl+G J` |
| LFS: Pull de objetos | `Ctrl+G L` |
| LFS: Push de objetos |  |
| LFS: Podar objetos locales | `Ctrl+G V` |
| LFS: Mostrar bloqueos |  |
| LFS: Abrir página de instalación |  |
| LFS: Alternar puntero en bruto en el diff | `r` |
| LFS: Seguir el archivo seleccionado |  |
| LFS: Seguir la extensión del archivo seleccionado |  |
| LFS: Bloquear el archivo seleccionado |  |
| LFS: Desbloquear el archivo seleccionado |  |
| LFS: Forzar el desbloqueo del archivo seleccionado |  |
| Refs: Eliminar la rama o el tag seleccionados... | `Ctrl+G d` |
| Refs: Eliminar la rama seleccionada de su remoto... |  |
| Refs: Eliminar la rama seleccionada en local y en su remoto... |  |
| Refs: Copiar enlace de la rama seleccionada en el remoto |  |
| Git: Copiar enlace del commit seleccionado en el remoto |  |
| Git: Tag anotado en el commit seleccionado... |  |
| Refs: Push del tag seleccionado al remoto por defecto |  |
| Refs: Eliminar el tag seleccionado de su remoto... |  |
| Git: Fetch del Tag seleccionado |  |
| Refs: Anotar el tag seleccionado... |  |
| Refs: Move Selected Tag to Selected Commit... |  |
| Refs: Alternar ocultar en el grafo |  |
| Refs: Mostrar en el grafo | `g` |
| Refs: Alternar solo en la ref seleccionada | `o` |
| Refs: Quitar solo (todas las refs) | `O` |
| Refs: Mostrar todas las refs ocultas | `H` |
| Refs: Alternar ocultar en la carpeta/remoto seleccionados |  |
| Refs: Alternar solo en la carpeta/remoto seleccionados | `o` |
| Refs: Filtrar por nombre... | `/` |
| Refs: Quitar el filtro de nombre | `Esc` |
| Refs: Expandir todas las carpetas | `=` |
| Refs: Contraer todas las carpetas | `-` |
| Refs: Expandir la carpeta seleccionada y sus subcarpetas | `*` |
| Refs: Contraer la carpeta seleccionada y sus subcarpetas | `_` |
| Refs: Sección siguiente | `l` |
| Refs: Sección anterior | `h` |
| Refs: Expandir o contraer la sección activa | `Enter` |
| Refs: Alternar la vista de jerarquía de ramas | `y` |
| Git: Stage del archivo seleccionado | `Ctrl+G s` |
| Git: Unstage del archivo seleccionado | `Ctrl+G u` |
| Git: Stage de todos los cambios | `Ctrl+G a` |
| Cambios de trabajo: Descartar el archivo seleccionado... |  |
| Cambios de trabajo: Ignorar el archivo seleccionado |  |
| Cambios de trabajo: Ignorar la extensión del archivo seleccionado |  |
| Cambios de trabajo: Tratar el archivo seleccionado como binario... |  |
| Cambios de trabajo: Tratar el archivo seleccionado como texto... |  |
| Cambios de trabajo: Tratar la extensión del archivo seleccionado como binaria... |  |
| Cambios de trabajo: Tratar la extensión del archivo seleccionado como texto... |  |
| Cambios de trabajo: Stash del archivo seleccionado |  |
| Cambios de trabajo: Historial del archivo seleccionado |  |
| Cambios de trabajo: Blame del archivo seleccionado |  |
| Cambios de trabajo: Abrir el archivo seleccionado en la herramienta de diff externa |  |
| Cambios de trabajo: Abrir el archivo seleccionado en el editor externo |  |
| Cambios de trabajo: Abrir el archivo seleccionado en el programa predeterminado |  |
| Cambios de trabajo: Mostrar el archivo seleccionado en su carpeta |  |
| Cambios de trabajo: Copiar la ruta del archivo seleccionado |  |
| Cambios de trabajo: Crear parche desde el archivo seleccionado... |  |
| Cambios de trabajo: Editar el archivo seleccionado |  |
| Cambios de trabajo: Eliminar el archivo seleccionado... |  |
| Cambios de trabajo: Abrir el menú de la carpeta seleccionada |  |
| Cambios de trabajo: Crear archivo |  |
| Cambios de trabajo: Descartar todos los cambios... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Hacer commit ya de los cambios en stage | `Ctrl+G Enter` |
| Git: Editar el mensaje del commit WIP (en el Log) | `c en el Log` |
| Git: Fetch todos los remotos | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward si es posible) |  |
| Git: Pull (solo fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Pull de todas las ramas | `Ctrl+G Ctrl+P` |
| Git: Pull de todas las ramas (fast-forward si es posible) |  |
| Git: Pull de todas las ramas (solo fast-forward) |  |
| Git: Pull de todas las ramas (rebase) |  |
| Git: Opciones de pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push a un remoto... |  |
| Git: Push a remotos seleccionados... |  |
| Git: Push a todos los destinos |  |
| Git: Pull de todas las fuentes |  |
| Git: Pull de un remoto... |  |
| Git: Mostrar push agrupado |  |
| Git: Continuar push agrupado |  |
| Git: Abortar push agrupado |  |
| Git: Reintentar remotos fallidos |  |
| Git: Force push (con lease)... |  |
| Git: Remotos de la rama... |  |
| Remoto: Añadir remoto... |  |
| Remoto: Quitar remoto... |  |
| Remoto: Editar remoto... |  |
| Remoto: Definir remoto por defecto... |  |
| Remoto: Copiar nombre del remoto |  |
| Remoto: Copiar enlace del remoto |  |
| Remoto: Fetch remoto... |  |
| Remoto: Acciones del remoto seleccionado... | `.` |
| Repositorio: Checkout parcial... |  |
| Repositorio: SSH y credenciales... |  |
| Repositorio: Probar conexión |  |
| Repositorio: Usar credenciales predeterminadas |  |
| Flow: Abrir menú | `Ctrl+G W` |
| Flow: Empezar rama... |  |
| Flow: Nuevo issue... |  |
| Flow: Recoger rama del host... |  |
| Flow: Terminar la rama actual... |  |
| Flow: Sincronizar con el padre... |  |
| Flow: Promover... |  |
| Flow: Establecer padre... |  |
| Flow: Continuar o abortar la acción detenida... |  |
| Flow: Alternar ramas agrupadas |  |
| Flow: Ajustes del flujo de trabajo... |  |
| Remoto: Ocultar remoto en el grafo... |  |
| Remoto: Mostrar remoto en el grafo... |  |
| Git: Rama nueva en HEAD... | `Ctrl+G b` |
| Git: Tag nuevo en HEAD... | `Ctrl+G t` |
| Git: Push todos los tags sin enviar | `Ctrl+G U` |
| Git: Stash de todos los cambios | `Ctrl+G T` |
| Git: Stash de todos los cambios con mensaje... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: Cherry-pick del commit seleccionado sobre HEAD | `Ctrl+G y` |
| Git: Checkout del commit seleccionado (HEAD desacoplado) |  |
| Git: Crear rama en el commit seleccionado... |  |
| Git: Crear tag en el commit seleccionado... |  |
| Commit: Copiar mensaje |  |
| Commit: Copiar nombre del autor |  |
| Commit: Copiar correo del autor |  |
| Commit: Copiar fecha |  |
| Commit: Copiar SHA corto |  |
| Commit: Copiar SHA completo |  |
| Git: Revertir el commit seleccionado... | `Ctrl+G v` |
| Git: Editar mensaje del commit... | `Ctrl+G e` |
| Commit: Modificar... | `Ctrl+G E` |
| Commit: Modificar autor... | `Ctrl+G A` |
| Commit: Modificar resumen... |  |
| Commit: Modificar descripción... |  |
| Git: Dividir el commit seleccionado... |  |
| Git: Rebase interactivo desde el commit seleccionado... |  |
| Rebase: Continuar |  |
| Rebase: Omitir |  |
| Rebase: Abortar |  |
| git am: Continuar |  |
| git am: Omitir este parche |  |
| git am: Abortar |  |
| Repositorio: Eliminar index.lock obsoleto |  |
| Aviso: Pop del auto-stash pendiente |  |
| Aviso: Terminar el auto-stash pendiente (conservar el resultado) |  |
| Aviso: Mostrar el auto-stash pendiente |  |
| Conflictos: Mostrar conflictos previstos |  |
| Aviso: Descartar el auto-stash pendiente... |  |
| Aviso: Empezar de nuevo... |  |
| Git: Reset de la rama al commit seleccionado (soft - conserva índice y árbol de trabajo) |  |
| Git: Reset de la rama al commit seleccionado (mixed - conserva el árbol de trabajo) | `Ctrl+G R` |
| Git: Reset de la rama al commit seleccionado (hard - descarta todo)... |  |
| Git: Resolver el conflicto con Ours (esta rama) |  |
| Git: Resolver el conflicto con Theirs (entrante) |  |
| Git: Resolver el conflicto con Base (ancestro común) |  |
| Conflictos: Abrir el editor de merge | `Enter sobre un archivo en conflicto` |
| Merge: Conflicto siguiente | `F7` |
| Merge: Conflicto anterior | `Shift+F7` |
| Merge: Archivo en conflicto siguiente | `Alt+PageDown` |
| Merge: Archivo en conflicto anterior | `Alt+PageUp` |
| Merge: Tomar bloque de A |  |
| Merge: Tomar bloque de B |  |
| Merge: Tomar Actual en este conflicto | `a` |
| Merge: Tomar Entrante en este conflicto | `b` |
| Merge: Tomar ambos, A y luego B | `c` |
| Merge: Tomar ambos, B y luego A | `C` |
| Merge: Tomar Base en este conflicto |  |
| Merge: Vaciar este conflicto | `x` |
| Merge: Intercambiar el orden | `s` |
| Merge: Tomar todo Actual | `A` |
| Merge: Tomar todo Entrante | `B` |
| Merge: Editar este conflicto en el resultado |  |
| Merge: Mostrar u ocultar los paneles laterales | `F9` |
| Merge: Desplazar los paneles a la izquierda | `Shift+Left` |
| Merge: Desplazar los paneles a la derecha | `Shift+Right` |
| Merge: Alternar el panel Base | `Alt+D` |
| Merge: Alternar el diff del archivo entero | `Alt+F` |
| Merge: Deshacer | `Ctrl+Z` |
| Merge: Rehacer | `Ctrl+Y` |
| Merge: Guardar sin marcar como resuelto | `Alt+S` |
| Merge: Guardar y marcar como resuelto | `Ctrl+S` |
| Merge: Restablecer el resultado al del merge | `Alt+R` |
| Merge: Recorrer las alturas del resultado | `z` |
| Merge: Ensanchar el panel Actual | `Alt+Right` |
| Merge: Ensanchar el panel Entrante | `Alt+Left` |
| Merge: Restablecer los tamaños de panel | `Alt+0` |
| Merge: Deshacer la elección de este conflicto | `Alt+U` |
| Merge: Ir al panel Actual (A) | `Alt+1` |
| Merge: Ir al panel Entrante (B) | `Alt+2` |
| Merge: Ir al panel Salida | `Alt+3` |
| Merge: Ir al panel Base | `Alt+4` |
| Merge: Panel de la izquierda (mismo conflicto) |  |
| Merge: Panel de la derecha (mismo conflicto) |  |
| Merge: Ir al panel siguiente | `F6` |
| Merge: Ir al panel anterior | `Shift+F6` |
| Merge: Continuar desde el archivo en disco |  |
| Merge: Empezar de nuevo desde el merge |  |
| Merge: Anular resolución y reabrir |  |
| Merge: Recargar el archivo |  |
| Merge: Usar el archivo Actual tal cual | `a` |
| Merge: Usar el archivo Entrante tal cual | `b` |
| Merge: Usar el archivo Base tal cual | `o` |
| Merge: Conservar el archivo que queda | `k` |
| Merge: Aceptar la eliminación | `D` |
| Merge: Abrir en la herramienta de merge externa | `m` |
| ✦ IA: Sugerir resolución de conflicto de merge | `Alt+I` |
| Merge: Cerrar el editor de merge |  |
| Git: Abortar merge, cherry-pick o revert |  |
| Git: Omitir cherry-pick o revert |  |
| Repositorio: Abrir o cambiar (gestión de repositorios)... | `Ctrl+O` |
| Repositorio: Examinar en busca de un repositorio... | `b (pestaña de gestión)` |
| Repositorio: Crear uno nuevo (Init)... | `i (pestaña de gestión)` |
| Repositorios: Abrir el seleccionado | `Enter` |
| Repositorios: Añadir o quitar el seleccionado de favoritos | `f` |
| Repositorios: Cerrar o quitar el seleccionado | `x` |
| Repositorios: Menú del repositorio seleccionado... | `.` |
| Repositorios: Copiar la ruta del seleccionado | `y` |
| Repositorios: Abrir el seleccionado en la herramienta predeterminada | `o` |
| Repositorios: Buscar | `/ / Ctrl+F` |
| Repositorios: Contraer todas las secciones | `-` |
| Repositorios: Expandir todas las secciones | `+` |
| Repositorios: Cerrar todas las pestañas... |  |
| Repositorios: Quitar el seleccionado de la lista |  |
| Repositorios: Quitar todos los recientes |  |
| Espacio de trabajo: Nuevo... | `n` |
| Espacio de trabajo: Nuevo desde pestañas abiertas... | `a` |
| Espacio de trabajo: Cambiar... | `Alt+W` |
| Espacio de trabajo: Editar... | `e` |
| Espacio de trabajo: Cerrar (sin espacio de trabajo) |  |
| Espacio de trabajo: Gestionar... | `Alt+J` |
| Espacio de trabajo: Cerrar pestaña |  |
| Espacio de trabajo: Localizar el repositorio ausente... | `l` |
| Espacio de trabajo: Eliminar... |  |
| Espacio de trabajo: Fetch todo | `F` |
| Espacio de trabajo: Pull todo | `P` |
| Espacio de trabajo: Clonar los ausentes | `m` |
| Espacio de trabajo: Añadir la pestaña al espacio de trabajo |  |
| Espacio de trabajo: Quitar la pestaña del espacio de trabajo |  |
| Radar: Abrir | `Alt+R` |
| Radar: Actualizar | `F5` |
| Radar: Buscar | `/` |
| Radar: Abrir en el navegador | `o` |
| Radar: Abrir repositorio | `Enter` |
| Radar: Cambiar a rama | `c` |
| Radar: Crear worktree... | `t` |
| Radar: Mostrar en repositorio | `s` |
| Radar: Empezar a trabajar en issue... | `n` |
| Radar: Fijar o dejar de fijar | `p` |
| Radar: Posponer... | `z` |
| Radar: Revisar solicitud... | `e` |
| Radar: Abrir la conversación del issue | `i` |
| Radar: Revisar mediante la API |  |
| Radar: Clonar y revisar... |  |
| Radar: Editar asignados... |  |
| Radar: Editar revisores... |  |
| Radar: Editar etiquetas... |  |
| Radar: Establecer hito... |  |
| Radar: Elegir ámbito | `w` |
| Radar: Elegir vista | `v` |
| Radar: Elegir filtro | `f` |
| Radar: Mostrar solicitudes no clonadas | `a` |
| Radar: Mostrar borradores | `d` |
| Radar: Activar las solicitudes de revisión de toda la cuenta... | `T` |
| Radar: Menú de la fila... | `.` |
| Radar: Elegir columna | `C` |
| Radar: Ensanchar columna | `]` |
| Radar: Estrechar columna | `[` |
| Radar: Restablecer anchos de columna | `0` |
| Radar: Ajustar la columna a su contenido | `=` |
| Radar: Ajustar todas las columnas a su contenido | `+` |
| Radar: Volver al inicio | `Esc` |
| Radar: Cerrar pestaña |  |
| Pestaña: Nueva pestaña | `Alt+T` |
| Pestaña: Nueva... |  |
| Pestaña: Menú de acciones... | `Ctrl+T` |
| Pestaña: Cerrar las demás pestañas... |  |
| Pestaña: Cerrar las pestañas de la derecha... |  |
| Pestaña: Añadir a favoritos o quitar de favoritos |  |
| Pestaña: Copiar ruta del repositorio |  |
| Pestaña: Abrir en el gestor de archivos |  |
| Pestaña: Abrir en el terminal |  |
| Pestaña: Abrir en... | `Alt+O` |
| Pestaña: Abrir en el predeterminado | `Ctrl+Alt+O` |
| Repositorio: Cambiar perfil... |  |
| Ayuda: Acerca de GitPoulpi | `Ctrl+N a` |
| Ayuda: Licencias y créditos |  |
| Ver: Registro de actividad | `Ctrl+N A` |
| Ver: Registro de actividad global | `Ctrl+N G` |
| Ayuda: Notas de la versión |  |
| Actualizaciones: Mostrar actualización | `Ctrl+N u` |
| Actualizaciones: Abrir aviso de seguridad |  |
| Actualizaciones: Copiar comando de actualización |  |
| Actualizaciones: Ejecutar comando de actualización en un terminal |  |
| Actualizaciones: Buscar actualizaciones ahora |  |
| Actualizaciones: Descargar |  |
| Actualizaciones: Actualizar ahora |  |
| Actualizaciones: Reiniciar para actualizar |  |
| Actualizaciones: Volver a la versión anterior |  |
| Actualizaciones: Omitir esta versión |  |
| Notas de la versión: Cerrar pestaña |  |
| Tareas: mostrar las tareas en segundo plano | `Ctrl+N J` |
| Tareas: cancelar todo lo que se pueda |  |
| Ver: Copiar diagnóstico |  |
| Registro de actividad: Mostrar u ocultar el nivel Error |  |
| Registro de actividad: Mostrar u ocultar el nivel Aviso |  |
| Registro de actividad: Mostrar u ocultar el nivel Info |  |
| Registro de actividad: Mostrar u ocultar el nivel Depuración |  |
| Registro de actividad: Mostrar u ocultar el nivel Traza |  |
| Registro de actividad: Mostrar todos los niveles |  |
| Registro de actividad: No mostrar ningún nivel |  |
| Registro de actividad: Vaciar el registro de la aplicación |  |
| Registro de actividad: Elegir el alcance |  |
| Registro de actividad: Operaciones de git |  |
| Registro de actividad: Registro de la aplicación |  |
| Registro de actividad: Mostrar la operación de esta línea |  |
| Registro de actividad: Siguiente operación fallida |  |
| Registro de actividad: Operación fallida anterior |  |
| Registro de actividad: Desplazar a la izquierda | `Left` |
| Registro de actividad: Desplazar a la derecha | `Right` |
| Registro de actividad: Copiar el último error |  |
| Registro de actividad: Ejecutar la acción sugerida |  |
| Registro de actividad: Abrir el último error |  |
| Pestaña: Salir y cd aquí |  |
| Repositorio: Clonar... | `Ctrl+Shift+N` |
| Repositorio: Pestaña siguiente | `Ctrl+PageDown / Ctrl+Tab` |
| Repositorio: Pestaña anterior | `Ctrl+PageUp / Ctrl+Shift+Tab` |
| Pestaña: Mover a la izquierda |  |
| Pestaña: Mover a la derecha |  |
| Repositorio: Cerrar pestaña | `Ctrl+W` |
| Repositorio: Seleccionar pestaña... | `t` |
| Buscar: Espacio de trabajo... | `Ctrl+Alt+F` |
| Edición: Deshacer | `Ctrl+Z` |
| Edición: Rehacer | `Ctrl+Y` |
| Edición: Olvidar la entrada de deshacer bloqueada |  |
| Edición: Historial de deshacer... | `Alt+Z` |
| Edición: Reflog... |  |
| Edición: Recuperar commits perdidos... |  |
| Edición: Restaurar la rama a donde la dejó la acción bloqueada | `Alt+Shift+Z` |
| Revisión: Mostrar resumen |  |
| Revisión: Mostrar archivos |  |
| Revisión: Mostrar Commits |  |
| Revisión: Comentar la línea... | `c` |
| Revisión: Comentar en la conversación... | `C` |
| Revisión: Responder al hilo... | `r` |
| Revisión: Resolver o reabrir el hilo | `x` |
| Revisión: Volver al comentario en el resumen | `Esc` |
| Revisión: Plegar o desplegar el hilo | `z` |
| Revisión: Editar el comentario pendiente... | `e` |
| Revisión: Eliminar el comentario pendiente | `d` |
| Revisión: Marcar el archivo como visto | `v` |
| Revisión: Archivo siguiente | `]` |
| Revisión: Archivo anterior | `[` |
| Revisión: Terminar la revisión... | `f` |
| Revisión: Descartar la revisión pendiente... |  |
| Revisión: Actualizar | `F5` |
| Revisión: Abrir en el navegador | `o` |
| Revisión: Cambiar a la rama | `b` |
| Revisión: Cerrar | `Esc` |
| Revisión: Editar asignados... | `a` |
| Revisión: Editar revisores... | `p` |
| Revisión: Editar etiquetas... | `l` |
| Revisión: Establecer hito... | `m` |
| Revisión: Elegir patch set... | `P` |
| Revisión: Comparar patch sets... | `D` |
| Revisión: Integrar el cambio... | `U` |
| Revisión: Abandonar el cambio... | `Q` |
| Revisión: Alternar vista en línea / dividida | `V` |
| Revisión: Alternar archivo entero / solo los hunks | `H` |
| Revisión: Alternar cambios solo de espacios en blanco | `S` |
| Revisión: Alternar ajuste de línea | `W` |
| Revisión: Hunk siguiente | `J` |
| Revisión: Hunk anterior | `K` |
| Revisión: Elegir codificación... | `E` |
| Revisión: Alternar minimapa | `M` |
| Revisión: Desplazar el diff a la izquierda | `Left` |
| Revisión: Desplazar el diff a la derecha | `Right` |
| Revisión: Copiar el texto seleccionado o la línea | `Ctrl+C` |
| Tablero: Mover tarjeta a... | `m` |
| Tablero: Mover tarjeta a la izquierda |  |
| Tablero: Mover tarjeta a la derecha |  |
| Tablero: Abrir tarjeta | `Enter` |
| Tablero: Editar asignados... |  |
| Tablero: Editar etiquetas... |  |
| Tablero: Establecer hito... |  |
| Tablero: Nuevo issue en esta columna | `n` |
| Tablero: Filtrar por asignado... | `a` |
| Tablero: Filtrar por etiqueta... | `l` |
| Tablero: Filtrar por hito... | `M` |
| Tablero: Filtrar por texto | `/` |
| Tablero: Quitar filtros | `x` |
| Tablero: Definir etiquetas de columna... | `c` |
| Tablero: Actualizar | `r` |
| Tablero: Cerrar | `Esc` |
| Conversación: Volver |  |
| Conversación: Actualizar | `F5` |
| Conversación: Cargar entradas anteriores | `m` |
| Conversación: Entrada anterior | `Up` |
| Conversación: Entrada siguiente | `Down` |
| Conversación: Siguiente entrada sin leer | `n` |
| Conversación: Responder a la entrada | `r` |
| Conversación: Citar la entrada | `Q` |
| Conversación: Comentario nuevo | `c` |
| Conversación: Enlaces de la entrada... | `l` |
| Conversación: Abrir el diagrama de la entrada | `g` |
| Conversación: Abrir el código del hilo | `G` |
| Conversación: Copiar el enlace de la entrada | `y` |
| Conversación: Abrir en el navegador | `o` |
| Conversación: Editar comentario | `e` |
| Conversación: Eliminar comentario | `d` |
| Conversación: Mostrar u ocultar eventos | `v` |
| Conversación: Alternar Escribir y Previa |  |
| Conversación: Alternar la vista dividida |  |
| Conversación: Enviar el comentario |  |
| Conversación: Alternar nota interna |  |
| Conversación: Cerrar | `Esc` |
| Editor de Markdown: Escribir con IA |  |
| Editor de Markdown: Actualizar el contexto de la IA |  |
| Alojamiento: Abrir la conversación de un issue... |  |
| Revisión: ✦ Redactar revisión... | `s` |
| Revisión: Aceptar sugerencia | `y` |
| Revisión: Editar y aceptar sugerencia... | `w` |
| Revisión: Descartar sugerencia | `n` |
| Revisión: Aceptar todas las sugerencias | `A` |
| Revisión: Descartar todas las sugerencias | `N` |
| Revisión: Usar el resumen de la IA en el texto de la revisión | `i` |
| Revisión: ✦ Usar el borrador... | `u` |
| Revisión: Editar el texto del borrador ✦... | `t` |
| Revisión: Descartar el borrador ✦ | `N` |
| Historial de deshacer: Deshacer hasta aquí... | `u` |
| Historial de deshacer: Rehacer hasta aquí... | `y` |
| Historial de deshacer: Restaurar solo esta... | `r` |
| Historial de deshacer: Restaurar la rama a donde estaba... | `o` |
| Historial de deshacer: Actualizar |  |
| Historial de deshacer: Cerrar | `Esc` |
| Reflog: Mostrar en el Log | `l` |
| Reflog: Crear rama aquí... | `b` |
| Reflog: Crear tag aquí... | `t` |
| Reflog: Aplicar cambios al árbol de trabajo | `w` |
| Reflog: Guardar cambios como stash | `s` |
| Reflog: Restaurar a esta posición... | `r` |
| Reflog: Cargar más | `m` |
| Reflog: Actualizar |  |
| Trabajo perdido: Crear rama aquí... | `b` |
| Trabajo perdido: Crear tag aquí... | `t` |
| Trabajo perdido: Cherry-pick | `c` |
| Trabajo perdido: Aplicar cambios al árbol de trabajo | `w` |
| Trabajo perdido: Guardar cambios como stash | `s` |
| Trabajo perdido: Buscar de nuevo |  |
| Reflog / Trabajo perdido: Panel siguiente | `Tab` |
| Reflog / Trabajo perdido: Lista de archivos como ruta o árbol | `v` |
| Reflog / Trabajo perdido: Restablecer tamaños de los paneles |  |
| Diálogo: Mover o cambiar de tamaño | `Ctrl+N m` |
| Diálogo: Restablecer posición y tamaño |  |
| Ver: Actualizar | `r` |
| Ver: Actualizar (fetch, pull requests, issues) | `F5` |
| Ver: Alternar la ayuda superpuesta | `?` |
| Ver: Alternar la columna Ref |  |
| Ver: Alternar la columna Grafo |  |
| Ver: Alternar la columna Mensaje |  |
| Ver: Alternar la columna Autor |  |
| Ver: Alternar la columna Fecha |  |
| Ver: Alternar la columna Cambios |  |
| Ver: Alternar la columna SHA |  |
| Ver: Columna del Log: elegir una columna para moverla o ajustarla | `C` |
| Ver: Columna del Log: elegir la columna siguiente | `Tab` |
| Ver: Columna del Log: elegir la columna anterior | `Shift+BackTab` |
| Ver: Columna del Log: mover la columna a la izquierda | `<` |
| Ver: Columna del Log: mover la columna a la derecha | `>` |
| Ver: Columna del Log: ensanchar la columna | `]` |
| Ver: Columna del Log: estrechar la columna | `[` |
| Ver: Columna del Log: restablecer los anchos de las columnas | `0` |
| Ver: Columna del Log: ajustar la columna a su contenido | `=` |
| Ver: Columna del Log: ajustar todas las columnas a su contenido | `+` |
| Ver: Log: desplazar las columnas a la izquierda | `Shift+Left` |
| Ver: Log: desplazar las columnas a la derecha | `Shift+Right` |
| Ver: Log: alternar el desplazamiento lateral de las columnas | `s` |
| Ver: Log: elegir las columnas que se muestran |  |
| Diff: Abrir el diff del archivo seleccionado | `Enter` |
| Diff: Cerrar la vista de diff | `Esc` |
| Diff: Hunk siguiente | `n` |
| Diff: Hunk anterior | `p` |
| Diff: Stage del hunk bajo el cursor | `Space` |
| Diff: Unstage del hunk bajo el cursor | `Space` |
| Diff: Seleccionar líneas (o quitar la selección) | `v` |
| Diff: Stage de las líneas seleccionadas | `Enter` |
| Diff: Unstage de las líneas seleccionadas | `Enter` |
| Diff: Descartar el hunk bajo el cursor... | `x` |
| Diff: Descartar las líneas seleccionadas... | `X` |
| Copiar: Texto seleccionado (arrastra antes sobre el texto) | `Ctrl+C` |
| Copiar: Línea bajo el cursor (o las líneas seleccionadas) | `y` |
| Diff: Copiar el hunk bajo el cursor |  |
| Diff: Disposición en línea (+/- en una columna) | `m` |
| Diff: Disposición dividida (antes y después) | `m` |
| Diff: Alternar solo hunks (archivo entero / solo los cambios) | `H` |
| Archivos: Alternar ruta/árbol | `m` |
| Archivos: Alternar ver todos los archivos | `a` |
| Commit: Alternar omitir hooks | `H` |
| Archivos: Abrir el menú de la fila o de la selección | `.` |
| Archivos: Expandir todas las carpetas | `=` |
| Archivos: Contraer todas las carpetas | `-` |
| Archivos: Expandir la carpeta seleccionada y sus subcarpetas | `*` |
| Archivos: Contraer la carpeta seleccionada y sus subcarpetas | `_` |
| Archivos: Seleccionar todo |  |
| Archivos: Borrar la selección |  |
| Archivos: Alternar la selección de la fila actual |  |
| Log: Copiar como gitGraph de Mermaid... | `Y` |
| Log: Comparar la selección | `v` |
| Log: Intercambiar los extremos de la comparación | `x` |
| Log: Ir a la siguiente fila comparada | `n` |
| Diff: Alternar el ajuste de línea | `w` |
| Diff: Elegir codificación... | `C` |
| Ver: Alternar el minimapa | `M` |
| Ver: Alternar la columna de commits | `Alt+C` |
| Ver: Columna de commits: ocultar |  |
| Ver: Columna de commits: historial del archivo |  |
| Ver: Columna de commits: todos los commits |  |
| Ver: Enfocar la columna de commits |  |
| Ver: Ensanchar la columna de commits |  |
| Ver: Estrechar la columna de commits |  |
| Ver: Columna de commits: desplazar a la izquierda | `Shift+Left` |
| Ver: Columna de commits: desplazar a la derecha | `Shift+Right` |
| Ver: Columna de commits: dimensionar la parte siguiente | `Tab` |
| Ver: Columna de commits: mover la parte a la izquierda | `<` |
| Ver: Columna de commits: mover la parte a la derecha | `>` |
| Ver: Columna de commits: ensanchar la parte | `]` |
| Ver: Columna de commits: estrechar la parte | `[` |
| Ver: Columna de commits: restablecer los anchos de las partes | `0` |
| Ver: Columna de commits: Ajustar la columna a su contenido | `=` |
| Ver: Columna de commits: Ajustar todas las columnas a su contenido | `+` |
| Ver: Columna de commits: menú de la fila seleccionada | `.` |
| Ver: Columna de commits: elegir las columnas |  |
| Ver: Columna de commits: alternar Refs |  |
| Ver: Columna de commits: alternar Grafo |  |
| Ver: Columna de commits: alternar Autor |  |
| Ver: Columna de commits: alternar Fecha |  |
| Ver: Columna de commits: alternar Cambios |  |
| Ver: Columna de commits: alternar SHA |  |
| Ver: Columna de commits: mostrar la fila seleccionada en el Log |  |
| Ver: Columna de commits: mostrar los cambios del archivo en la fila seleccionada |  |
| Ver: Columna de commits: mostrar el archivo como era en la fila seleccionada |  |
| Ver: Columna de commits: blame del archivo en la fila seleccionada |  |
| Editor: Alternar seguir la selección |  |
| Imagen: Acercar | `+` |
| Imagen: Alejar | `-` |
| Imagen: Ajustar a la vista | `0` |
| Imagen: Tamaño real (1:1) | `1` |
| Imagen: Alternar antes / después | `Tab` |
| Imagen: Ensanchar panel Antes | `Alt+Right` |
| Imagen: Ensanchar panel Después | `Alt+Left` |
| Imagen: Restablecer tamaños de paneles | `Alt+0` |
| Imagen: Editar en... | `O` |
| Imagen: Editar en el editor predeterminado | `o` |
| Historial del archivo: Mostrar en el Log | `Enter` |
| Historial del archivo: Copiar el SHA del commit | `y` |
| Historial del archivo: Copiar el resumen | `Y` |
| Historial del archivo: Mostrar el diff de este commit | `d` |
| Historial del archivo: Abrir el archivo en este commit | `o` |
| Historial del archivo: Blame en este commit | `b` |
| Historial del archivo: Comparar con el árbol de trabajo | `c` |
| Historial del archivo: Restaurar esta versión | `R` |
| Historial del archivo: Ampliar la selección hacia arriba | `Shift+Up` |
| Historial del archivo: Ampliar la selección hacia abajo | `Shift+Down` |
| Historial del archivo: Alternar entre la lista y el diff | `Tab` |
| Historial del archivo: Ensanchar la lista | `Alt+Right` |
| Historial del archivo: Estrechar la lista | `Alt+Left` |
| Historial del archivo: Restablecer el ancho de la lista | `Alt+0` |
| Archivo: Abrir con... | `O` |
| Archivo: Abrir en la aplicación predeterminada | `o` |
| Diff: Alternar los cambios solo de espacios en blanco | `s` |
| ✦ IA: Generar mensaje de commit | `Ctrl+G i` |
| ✦ IA: Generar mensaje de stash |  |
| ✦ IA: Insertar variable de prompt |  |
| ✦ IA: Editar perfil... |  |
| ✦ IA: Abrir el agente... |  |
| ✦ IA: Explicar el commit seleccionado | `Ctrl+G w` |
| ✦ IA: Dividir el commit seleccionado... |  |
| ✦ IA: Explicar los cambios de trabajo | `Ctrl+G w` |
| ✦ IA: Componer commits a partir de los cambios de trabajo... | `Ctrl+G C` |
| Archivos sensibles: Mostrar en el archivo | `S` |
| Compositor: Mover archivo a un commit... | `m` |
| Diff: Blame |  |
| Blame: Mostrar el commit en el Log | `Enter` |
| Blame: Copiar el SHA del commit | `Y` |
| Blame: Diff del commit para este archivo | `d` |
| Blame: Blame en el padre del commit | `b` |
| Blame: Abrir el historial del archivo | `H` |
| Blame: Mostrar u ocultar la tarjeta de detalle | `i` |
| Blame: Hunk siguiente | `n` |
| Blame: Hunk anterior | `p` |
| Blame: Hunk siguiente del mismo commit | `N` |
| Blame: Hunk anterior del mismo commit | `P` |
| Blame: Ir a la línea | `g` |
| Blame: Ensanchar la columna de anotaciones | `>` |
| Blame: Estrechar la columna de anotaciones | `<` |
| Diff: Historial del archivo |  |
| Diff: Vista de archivo |  |
| Archivo: Editar en el directorio de trabajo | `e` |
| Editor: Cambiar codificación |  |
| Editor: Reabrir con codificación |  |
| Editor: Convertir a codificación |  |
| Editor: Convertir saltos de línea |  |
| Editor: Cambiar sangría |  |
| App: Ajustes... | `Ctrl+, / Ctrl+N ,` |
| App: Buscar en los ajustes... | `/ en Ajustes` |
| App: Atajos de teclado... |  |
| Agente: Mostrar la vista pendiente | `Ctrl+N Enter` |
| Agente: Descartar la vista pendiente |  |
| Agente: Volver | `Ctrl+N Backspace` |
| Atajos: Restablecer todos... |  |
| Integraciones: Abrir pull request vinculada | `Ctrl+G g` |
| Integraciones: Editar pull request vinculada en el host | `e (sobre una fila)` |
| Integraciones: Copiar URL de la pull request vinculada |  |
| Integraciones: Actualizar pull requests e issues |  |
| Integraciones: Checkout de pull request... |  |
| Integraciones: Abrir referencia del commit (#issue / !solicitud)... |  |
| Integraciones: Abrir issue... |  |
| Alojamiento: Revisar solicitud... |  |
| Alojamiento: Push para revisión... |  |
| Alojamiento: Editar asignados... |  |
| Alojamiento: Editar revisores... |  |
| Alojamiento: Editar etiquetas... |  |
| Alojamiento: Establecer hito... |  |
| Issues locales: Cerrar issue... |  |
| Issues locales: Reabrir issue... |  |
| Issues locales: Abrir en Obsidian... |  |
| Issues: Abrir tablero |  |
| Issues locales: Crear base de Obsidian |  |
| Pull requests: Crear pull request... | `n` |
| Alojamiento: Nuevo issue... | `n` |
| Pull requests: Buscar | `Enter sobre la fila de búsqueda` |
| Pull requests: Elegir filtro (abiertas / merged / todas)... | `v` |
| Pull requests: Elegir proveedor... | `Enter sobre la fila del proveedor` |
| Pull requests: Elegir repositorio... | `Enter sobre la fila del repositorio` |
| Pull requests: Desvincular origen (ninguno) |  |
| Issues: Elegir proveedor... | `Enter sobre la fila del proveedor` |
| Issues: Desvincular origen (ninguno) |  |
| Integraciones: Desvincular perfil (automático) |  |
| Issues: Elegir repositorio... | `Enter sobre la fila del repositorio` |
| Issues: Buscar | `Enter sobre la fila de búsqueda` |
| Issues: Elegir filtro... | `v` |
| Issues: Editar consulta / listas... |  |
| Commit: Insertar referencia a un issue... |  |
| Integraciones: Usar perfil para este repositorio... |  |
| Integraciones: Gestionar el perfil de este repositorio |  |
| Integraciones: Conectar integración... |  |
| Ajustes: Opciones del Radar de un perfil... |  |
| Integraciones: Revisar comprobación de seguridad |  |
| App: Salir | `q` |
| App: Tema... |  |
| Conflictos: Editar listas de ramas base... |  |
| Conflictos: Ramas base de este repositorio... |  |
| App: Cambiar tema... | `Ctrl+N t` |
| App: Cambiar idioma... | `Ctrl+N l` |
| Lanzador: Crear acceso directo |  |
| Lanzador: Crear acceso directo del escritorio |  |
| Lanzador: Quitar accesos directos |  |
| Lanzador: Crear alias del comando |  |
| Lanzador: Quitar alias del comando |  |
| MCP: Sugerir un puerto libre |  |
| Lanzador: Añadir al PATH |  |
| Lanzador: Quitar del PATH |  |
| Lanzador: Instalar menú Abrir en GitPoulpi |  |
| Lanzador: Quitar menú Abrir en GitPoulpi |  |
| Lanzador: Convertir en la instalación principal |  |
| Lanzador: Quitar lanzadores heredados |  |
| Lanzador: Olvidar instalaciones desaparecidas |  |
| App: Cambiar tema de color del grafo... | `Ctrl+N T` |
| App: Alternar tinte de filas del grafo | `Ctrl+N Y` |
<!-- palette:end -->

## Moverse

| Tecla | Acción |
|---|---|
| `Tab` / `Shift+Tab` | Dar el foco al panel siguiente / anterior |
| `Ctrl+N`, y después una insignia | Dar el foco a esa parte de la ventana, o pulsar ese control del diálogo o popup en primer plano (consulta el [Prefijo de navegación](#prefijo-de-navegación-ctrln)) |
| `j` / `k`, `Up` / `Down` | Mover la selección, en todos los paneles |
| `PageUp` / `PageDown`, `Home` / `End` | Mover una página; ir a la primera / última fila |
| un número, y después `j` / `k` / `Up` / `Down` | Prefijo de repetición: `4j` mueve cuatro filas. Con `PageUp` / `PageDown`, la cuenta es un número de páginas. La barra de estado muestra la cuenta pendiente (`4…`). `Esc` o cualquier otra tecla la borra. Un `0` inicial nunca empieza una cuenta |
| `h` / `l`, `Left` / `Right` | Panel Refs: cambiar de sección. Log: salir (`h`) o entrar (`l`) en la lista de archivos modificados de un commit |
| `?` | Abrir o cerrar la ayuda (consulta la [Ayuda integrada](#ayuda-integrada-)) |
| `r` | Refrescar |
| `F5` | Hacer fetch de todos los remotos, recargar las pull requests y los issues, y refrescar |
| `q` | Salir |
| `Ctrl+Z` | **Deshacer** la última operación. Consulta [Deshacer y rehacer](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Rehacer**. `Ctrl+Shift+Z` necesita el protocolo de teclado de kitty; `Ctrl+Y` funciona siempre |
| `.`, `Menu`, `Shift+F10` | Abrir el menú contextual de la fila seleccionada |

### Ctrl+Z / Ctrl+Y en un campo de texto

Mientras un campo de texto tiene el teclado, `Ctrl+Z` y `Ctrl+Y` (`Ctrl+Shift+Z`
también) actúan sobre el texto de ese campo, nunca sobre el repositorio: un campo que
tiene el teclado siempre gana, así que `Ctrl+Z` en el mensaje de commit no puede
deshacer un commit. En cualquier otro sitio conservan su significado global (el
diario de deshacer / rehacer de arriba). Esto vale para todos los campos de texto
y editores multilínea: el mensaje de commit, los campos y editores de texto de
Ajustes, los formularios de pull request y de issue, los diálogos de IA y todos
los cuadros de filtro o de búsqueda.

- **Escribir** se deshace palabra a palabra (una tanda de letras, una de espacios,
  una de signos de puntuación), y escribir con una pausa de más de un segundo
  empieza un paso nuevo. Varios retrocesos seguidos son un solo paso. Pegar, cortar,
  `Ctrl+U` / `Ctrl+K`, insertar un fragmento o aceptar una compleción es un paso cada uno.
- Deshacer devuelve el texto, el cursor y la selección. Una edición nueva
  tras deshacer descarta lo que se podía rehacer.
- Cada campo guarda un historial acotado (los últimos 200 pasos, unos 1 MB de
  texto), y lo olvida cuando el campo se reemplaza desde fuera (una carpeta elegida
  en el selector, un formulario recargado) o cuando otro campo toma el teclado.
- Si no queda nada que deshacer, `Ctrl+Z` no hace nada: nunca pasa al deshacer del
  repositorio mientras un campo tiene el teclado.
- El **editor de instrucciones** de los ajustes de IA tiene además los botones
  **Deshacer** y **Rehacer**, junto a Guardar y Cancelar, desactivados mientras no hay
  nada a lo que volver.
- El editor de código y el editor de merge conservan su propio deshacer
  (`Edición: Deshacer`, `Merge: Deshacer`), que también cubre sus elecciones.

## Listas emergentes filtrables

Todas las listas emergentes filtrables usan las mismas teclas: la paleta de
comandos y sus selectores (tema, idioma, colores del grafo), el selector de
pestañas y todos los desplegables con línea de búsqueda (opciones de los
Ajustes, el formulario de pull request, el selector de referencias de issues, la
lista **Abrir en**). Escribe para filtrar la lista. Estas teclas mueven el
resaltado:

| Tecla | Acción |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Mover una fila arriba / abajo |
| `Home` / `End` | Primera / última fila |
| `PageUp` / `PageDown` | Mover una página visible |
| `Enter` | Elegir la fila resaltada |
| `Esc` | Cerrar |

En estos popups, `Home` y `End` mueven la lista, no el cursor del campo de
filtro, y `Ctrl+K` / `Ctrl+J` también mueven la lista, de modo que `Ctrl+K` ya no
borra hasta el final del filtro (`Ctrl+U` sigue vaciándolo; en un campo de texto
normal fuera de una lista `Ctrl+K` sigue borrando hasta el final de la línea). `Ctrl+A` y
`Ctrl+E` siguen moviendo el cursor al principio y al final. `Ctrl+J` / `Ctrl+K` mueven
también todas las demás listas, con o sin campo de texto (`Ctrl+N` es el prefijo de
navegación y `Ctrl+P` la paleta de comandos, así que no son teclas de lista). `j` y
`k` sin más se escriben en el filtro; en las listas sin campo de texto, `j` / `k` son
una vía más para bajar / subir.

Con el ratón:

- La rueda **desplaza** la lista. No mueve el resaltado.
- Un clic **selecciona** una fila. Un doble clic la **elige**, como `Enter`.
- Una lista más alta que su popup muestra una barra de desplazamiento en el
  borde derecho. Puedes arrastrarla.
- Un desplegable corto sin línea de búsqueda (dos o tres opciones) elige al
  primer clic.

**Vista previa en vivo.** Los selectores de tema, de tema de colores del grafo y
de idioma aplican al instante la fila resaltada, detrás del popup. `Enter` la
conserva. `Esc`, o un clic fuera del popup, restaura el valor anterior.

**Filtro de temas.** El selector de temas tiene una fila bajo el campo de filtro:
**Todos**, **Oscuros**, **Claros** y **Con color**. `Tab` / `Shift+Tab` van al
siguiente / anterior. `Alt+1` a `Alt+4` eligen uno directamente. También puedes
hacer clic en ellos.

## Ayuda integrada (`?`)

`?`, `Ctrl+N ?` o el comando de la paleta `Ver: Alternar la ayuda superpuesta`
abre la ayuda. Lista las teclas y los gestos del ratón de cada parte de la
aplicación. Se abre sobre cualquier cosa, incluida la pantalla de Ajustes.

| Tecla | Acción |
|---|---|
| `?` | Cerrar la ayuda (y borrar el filtro) |
| *(escribir)* | Filtrar: conserva solo las secciones que contienen el texto. `j`, `k` y `q` también se escriben |
| `Backspace`, `Ctrl+U` | Borrar el último carácter / borrar el filtro |
| `Esc` | Borrar el filtro; si ya está vacío, cerrar la ayuda |
| `Up` / `Down` | Desplazar una línea |
| `PageUp` / `PageDown`, `Home` / `End` | Desplazar una página; ir al principio / al final |
| `Ctrl+D`, o un clic en el enlace | Abrir esta documentación en GitHub en el navegador |

La rueda desplaza la ayuda, y puedes arrastrar su barra de desplazamiento. Un
clic fuera de la ayuda la cierra.

## Prefijo de navegación (`Ctrl+N`)

`Ctrl+N` espera una tecla más. La barra de estado muestra las teclas que acepta.
La tecla siguiente ejecuta una acción o cancela.

| Tecla | Acción |
|---|---|
| `Ctrl+N` y después una insignia | Dar el foco a la parte de la ventana que muestra esa insignia |
| `Ctrl+N Shift+H` / `Ctrl+N Shift+L` | Contraer o restaurar el panel Refs / el panel derecho |
| `Ctrl+N Left` / `Ctrl+N Right` | Mover un paso a la izquierda / derecha el divisor entre el panel Refs y el Log (un panel Refs contraído vuelve primero) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Mover un paso a la izquierda / derecha el divisor entre el Log y el panel derecho |
| `Ctrl+N +` / `Ctrl+N -` | Agrandar / reducir en una fila la sección activa de Refs |
| `Ctrl+N ]` / `Ctrl+N [` | Agrandar / reducir en una fila el bloque del mensaje de commit del panel derecho |
| `Ctrl+N Tab` | Elegir una pestaña (el selector de pestañas; `t` solo hace lo mismo) |
| `Ctrl+N t` / `Ctrl+N l` / `Ctrl+N Shift+T` | Cambiar el tema / el idioma / el tema de colores del grafo |
| `Ctrl+N Shift+Y` | Activar o desactivar los tintes de fila del grafo |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Mostrar la vista que preparó un agente / volver (consulta [MCP y agentes](mcp-and-agents.md)) |
| `Ctrl+N ,` | Abrir los Ajustes |
| `Ctrl+N ?` | Abrir o cerrar la ayuda |
| `Esc`, o cualquier otra tecla | Cancelar |

### Insignias

Mientras el prefijo espera, cada parte de la ventana que puede recibir el foco
muestra una insignia de un carácter en su borde superior izquierdo. Pulsa ese
carácter para darle el foco. Las insignias se reparten de nuevo cada vez, en
orden de pantalla: primero de `1` a `9`, luego `a`, `b`, `c`... Por eso conviene
leer la insignia en pantalla: una parte no siempre recibe la misma insignia.

Partes con insignia: el filtro de Refs y cada sección visible de Refs, el Log
(o la vista Diff), la Vista de archivo cuando está abierta, las listas Unstaged
y Staged, el campo de resumen del compositor de commits y los controles de la
lista de archivos del commit. En la pestaña Inicio, las insignias son el campo
de búsqueda, las cuatro secciones y los botones de la barra de herramientas (la insignia de un botón lo pulsa). En la pestaña Radar
son el campo de filtro, las listas desplegables Ámbito, Vista y Mostrar (la
insignia de una lista desplegable la abre), los botones Actualizar y Volver, y
cada encabezado de sección en pantalla. La insignia de una sección lleva la
selección a su encabezado; después `Enter` o `Space` la pliega.

En el selector de carpetas de Examinar y en los diálogos Clonar, Init, Crear worktree, Espacio de trabajo, Checkout parcial, Flow, Nuevo issue y Crear pull request, las insignias son los campos, las casillas y los botones (la insignia de un botón lo pulsa, la de un campo pone el cursor en él; en una lista, una insignia está en su primera fila; el selector de carpetas de Examinar tiene una insignia para la lista de Acceso rápido y otra para la lista de Carpetas, junto a la primera fila de cada una, y pulsar una pone el teclado en esa lista sin elegir ninguna fila). En el Registro de actividad son las dos pestañas, los cuadros Alcance y Nivel, el filtro, las casillas de nivel, los botones y la primera entrada de la lista. Una insignia hace lo mismo que un clic en ese control. En el selector de carpetas, `Ctrl+N` inicia las insignias, así que su tecla de Carpeta nueva es `Alt+N`.

En el diálogo Modificar (y Editar mensaje del commit), las insignias son el resumen, la descripción, el nombre, el correo y la fecha del autor, **Modificar el historial más a fondo...**, **Cancelar**, **Aplicar** y la cruz de cierre; en el banco de trabajo de reescritura del historial son el cuadro de alcance, sus casillas y el botón de refs, el nombre de la receta, cada regla, su casilla y sus botones, el botón **+ Añadir regla**, los campos de la regla seleccionada, las pestañas de la vista previa, el cuadro de filtro, las filas de ramas protegidas y su Marcar todo / Desmarcar todo, y los botones Vista previa, Aplicar, Cancelar, Guardar, Cargar y Crear reglas con IA (las páginas Secretos, Identidades, Tamaño y resultado añaden sus propios botones y filas; las filas de la pestaña Commits, al ser una lista, no tienen). Una insignia hace lo que un clic. No se marca nada mientras haya abierto una lista desplegable, una pregunta, un campo en el que se escribe, el selector de carpetas, la pantalla de purga, el editor de scripts, los diálogos de IA, el diff del blob o la guía del host, ni mientras el diálogo Modificar pregunta «¿descartar?»; `Esc` quita las insignias sin cerrar nada.

En Ajustes, las insignias son el cuadro de búsqueda, la lista de páginas (una sola insignia, en la página actual), las subpestañas, la lista y sus botones, cada campo de la página que está en pantalla y la cruz de cierre. Una insignia hace lo mismo que un clic en ese control: un campo de texto se abre para editarlo, una casilla se marca, una lista desplegable se abre, un botón se ejecuta. `Ctrl+N` conserva su significado propio mientras se escribe en un campo o hay una lista desplegable, un editor o la búsqueda abiertos.

La paleta tiene las mismas acciones, por ejemplo
`Navegación: Ir a Refs > Locales` o `Navegación: Ir a Estado > Staged`.

## Repositorios y pestañas

| Tecla | Acción |
|---|---|
| `Ctrl+O`, o un clic en `⌂` | Ir a la pestaña Inicio (en la pestaña Inicio: dar el foco al campo de búsqueda; con el Radar o las notas de la versión abiertos: ciérralos primero) |
| `Ctrl+Shift+N` | Abrir la pestaña Inicio con el diálogo Clonar |
| `Ctrl+PageDown` / `Ctrl+PageUp`, `Ctrl+Tab` / `Ctrl+Shift+Tab` | Pestaña siguiente / anterior (el par `Ctrl+Tab` solo donde el terminal lo envía) |
| `t` | Elegir una pestaña en una lista filtrable |
| `Ctrl+W` | Cerrar la pestaña actual (pregunta antes; el repositorio no cambia) |
| `Ctrl+T`, o un clic derecho en una pestaña | El menú de la pestaña (sus filas se enumeran en [Fila de pestañas](interface-tour.md#fila-de-pestañas)) |
| `Alt+O` / `Ctrl+Alt+O` | Abrir la carpeta de la pestaña: elegir en la lista **Abrir en** / usar la entrada predeterminada. Consulta [Abrir en](open-in.md) |

### La pestaña Inicio

La pestaña `⌂` gestiona tus repositorios. Tiene cuatro secciones:
**Repositorios abiertos**, **Favoritos**, **Repositorios recientes** y **Todos
los repositorios**. Consulta el [Recorrido por la interfaz](interface-tour.md#inicio).

| Tecla | Acción |
|---|---|
| `j` / `k`, flechas, `PageUp` / `PageDown`, `Home` / `End` | Mover |
| `Enter`, o un doble clic | Abrir el repositorio (o cambiar a su pestaña). En el encabezado de una sección: plegarla o desplegarla |
| `f` o `*`, o un clic en la estrella | Añadir a favoritos / quitar de favoritos |
| `x`, `Delete`, o el botón de la fila | Cerrar su pestaña (Abiertos), quitarlo de la lista (Recientes), quitar la estrella (Favoritos), olvidarlo (Todos). No se borra nada en el disco |
| `.`, un clic derecho, o el `⋮` de la fila | El menú de la fila |
| `/` o `Ctrl+F` | Buscar en todas las secciones por nombre, rama o ruta. `Enter` abre la primera coincidencia; `Esc` borra la búsqueda y después sale del campo |
| `Space`, `Left` / `Right` | Plegar / desplegar la sección actual |
| `-` / `+` | Contraer todo / Expandir todo |
| `b` / `c` / `i` | Examinar (abrir una carpeta) / Clonar / Init |
| `Esc` | Borrar la búsqueda, o volver a la pestaña del repositorio activo |
| `q` | Salir |

`Ctrl+P`, `Ctrl+,`, `?` y `Ctrl+N` funcionan en la pestaña Inicio. `Ctrl+G`,
`Ctrl+W` y `Ctrl+T` necesitan una pestaña de repositorio: en la pestaña Inicio
solo muestran un mensaje. Los comandos de la paleta que necesitan un
repositorio aparecen en gris.

### El selector de carpetas

**Examinar** (y el botón **Examinar...** de los diálogos Init, Clonar y Crear
worktree) abre el selector de carpetas. Tiene un campo de ruta arriba, los
lugares y el árbol de carpetas a la izquierda, y el contenido de la carpeta
actual a la derecha.

| Tecla | Acción |
|---|---|
| `Tab` / `Shift+Tab` | Campo de ruta, luego la columna izquierda, luego el contenido |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Moverse por la columna con el foco. En el campo de ruta: moverse por las sugerencias de autocompletado |
| una letra (contenido) | Ir a la siguiente carpeta que empiece por esa letra |
| `Enter` / `Right` | Abrir la carpeta seleccionada. En el campo de ruta: ir a la carpeta escrita; si ya es la carpeta mostrada, elegirla |
| `Backspace` / `Left` / `Alt+Up` | Carpeta superior (en el campo de ruta, `Backspace` borra un carácter) |
| `Tab` (campo de ruta) | Aceptar la única sugerencia, o la resaltada |
| `Alt+N` | Carpeta nueva (`Ctrl+N` inicia las insignias, consulta [Insignias](#insignias)) |
| `Alt+H` | Mostrar u ocultar las carpetas ocultas |
| `Ctrl+F`, o `/` fuera del campo de ruta | Filtrar las entradas de la carpeta mientras escribes (sin distinguir mayúsculas; `..` se queda; la cabecera cuenta «x de n»; `Enter` salta a la primera coincidencia, `Esc` lo borra; se vacía al cambiar de carpeta). También se llega con `Tab` o con un clic |
| `Ctrl+S`, `Ctrl+Enter`, o el botón de confirmar | Elegir la subcarpeta seleccionada o, si no hay, la carpeta mostrada |
| `Esc` | Cerrar las sugerencias, o cancelar |

Cuando el campo de ruta muestra una ruta nueva, toda la ruta queda seleccionada:
la primera tecla que escribas la reemplaza.

### Diálogos Init y Clonar

| Tecla | Acción |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Campo o botón siguiente / anterior |
| `Enter` | Pulsar el botón con el foco, o crear / clonar |
| `Space` | Marcar o desmarcar la casilla con el foco, o pulsar el botón con el foco |
| `Ctrl+B` | Examinar para elegir la carpeta |
| `j` / `k`, `g` / `G` | En la lista de fuentes y en la lista de repositorios del diálogo Clonar: bajar / subir, primero / último (no en el campo de búsqueda) |
| `Esc` | Cerrar. Durante un clon, el clon continúa en segundo plano |

En el diálogo Clonar, la columna izquierda lista **Clonar con URL** y una fuente
por cada perfil de alojamiento conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Elige una fuente con `Up` / `Down` o con un clic. Después escribe en
el campo de búsqueda **Repositorio**, muévete por la lista con las flechas,
`PageUp` / `PageDown` o la rueda, y elige una fila con `Enter` o con un clic.
Consulta [Primeros pasos](getting-started.md) para ver los campos.

## Panel Refs

| Tecla | Acción |
|---|---|
| `Enter` / `Space` | Expandir o contraer la sección activa, o la carpeta seleccionada |
| `h` / `l`, `Left` / `Right` | Cambiar de sección |
| `/` | Filtrar por nombre (`Enter` aplica, `Esc` cancela) |
| `y` | Activar o desactivar la vista de jerarquía de ramas: carpetas a partir de las `/` de los nombres de rama (LOCALES y REMOTAS) |
| `o` | Hacer **Solo** de la ref seleccionada, o de todas las refs de la carpeta o del remoto seleccionados. Haz solo de varias refs para ver su unión |
| `Shift+O` | **Dejar de usar solo**: borra todos los solo |
| `Shift+H` | **Mostrar todo**: vuelve a mostrar todas las refs ocultas |
| `g` | **Mostrar en el grafo**: desplaza el Log hasta el commit de la ref seleccionada (o del stash) y hace parpadear su fila; la selección no se mueve |
| `m` | **Marcar** la ref seleccionada como origen de arrastre (`Esc`, o `m` sobre ella otra vez, quita la marca) |
| `Shift+M` | **Soltar la ref marcada** sobre la ref seleccionada: abre el menú que abre un arrastre (Merge, Rebase, Push, Fast-forward...) |
| `+` (sección REMOTAS) | Añadir un remoto |
| `.`, `Menu`, `Shift+F10`, o un clic derecho | El menú de la fila: una rama, un tag, el nombre de un remoto o una carpeta |
| `Enter` en una fila de WORKTREES | Abrir ese worktree en su propia pestaña |
| `Ctrl+G d` | Eliminar la rama o el tag seleccionados (pregunta antes). En una fila de REMOTAS: eliminar esa rama remota |

Para saber qué hacen los menús, consulta [Trabajar con el Log](log-and-history.md).

### Pull requests e issues

Con una integración adecuada, el panel Refs tiene una sección de pull requests
(en GitLab: merge requests). Con un perfil de alojamiento conectado, también
tiene una sección ISSUES. Consulta
[Pull requests e issues](pull-requests-and-issues.md#pull-requests-e-issues-en-la-barra-lateral).

| Tecla | Acción |
|---|---|
| `Enter` | Abrir la fila seleccionada: un issue (y una request de Azure DevOps o Bitbucket) como [conversación](conversations.md), una pull request en la [revisión](review.md) en los hosts que la tienen, cualquier otra cosa en el navegador. En una fila `Reconectar`: abrir los Ajustes en ese perfil |
| `o` | Abrir la fila seleccionada en el navegador |
| `Space` | Expandir o contraer la sección |
| `v` | Abrir el filtro de la sección. Pull requests: Abiertas, Merged / cerradas, Todas. Issues: Abiertos, Asignados a mí, Cerrados, Todos |
| `n` | Sección de pull requests: crear una pull request. Sección ISSUES: issue nuevo. (El botón `+` del encabezado) |
| `e` | Sección de pull requests: editar la solicitud seleccionada en el host (el lápiz de la fila) |
| `Enter` en la fila de búsqueda | Escribir en el campo de búsqueda. `Enter`, `Tab` o `Esc` salen de él; `Ctrl+U` lo borra |
| `Enter` en el encabezado de un grupo | Plegar o desplegar el grupo (*Mis pull requests*, *Asignadas a mí*, *Pendientes de mi revisión*, *Todas las pull requests*) |
| `Enter` en la fila del filtro, del proveedor o del repositorio | Abrir ese desplegable. La lista de proveedores termina con **Ninguno (desvincular este repositorio)** |
| `.`, o un clic derecho | El menú de la fila: abrir la conversación, abrir en el navegador, copiar la URL o el número, hacer checkout de la rama de origen (pull requests); crear una rama desde el issue (issues) |

### El tablero de issues

**Tablero** en el encabezado ISSUES (paleta: **Issues: Abrir tablero**) abre el tablero Kanban del origen de issues. Ver
[el tablero](pull-requests-and-issues.md#el-tablero-de-issues) y [los issues locales](local-issues.md#el-tablero-kanban). Cada tecla
de abajo es también un comando de la paleta (**Tablero: ...**).

| Tecla | Acción |
|---|---|
| Flechas | Seleccionar una tarjeta (`Up`, `Down`) o la columna vecina (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` saltan dentro de la columna |
| `Shift+Left`, `Shift+Right` | Mover la tarjeta seleccionada a la columna anterior o siguiente |
| `m` | **Mover tarjeta a...**: un desplegable con las columnas |
| `Enter` | Abrir la [conversación](conversations.md) del issue (la página del host donde el host no tiene ninguna que leer) |
| `e`, `.` | Editar asignados, etiquetas o hito; el menú de la tarjeta |
| `n` | Issue nuevo en la columna seleccionada |
| `a`, `l`, `M`, `/` | Filtrar por asignado, etiqueta, hito, texto. `x` quita los filtros |
| `c` | Las etiquetas que son las columnas de un tablero de GitHub / GitLab |
| `r` | Leer los issues de nuevo |
| `Esc` | Cerrar el tablero |

Ratón: arrastra una tarjeta a otra columna para moverla, haz clic en una tarjeta para seleccionarla, doble clic para abrirla, clic
derecho para el menú de la tarjeta, rueda para desplazar una columna, `Shift`+rueda (o la barra bajo el tablero) para desplazar las columnas.

### El editor de Markdown y la vista de conversación

Todo cuadro cuyo texto va a un host (un comentario, una respuesta, un resumen de revisión, la descripción de un issue o de
una pull request nuevos) es el mismo editor. Sus teclas están en la tabla `md_editor` de arriba y se pueden reasignar.

| Tecla | Acción |
|---|---|
| `Alt+P` | Alterna **Escribir** y **Previa** (desde **Dividida** pasa a Previa) |
| `Alt+L` | Activa **Dividida**, o vuelve a Escribir (necesita una ventana de 100 columnas) |
| `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` | Envía el texto (el botón propio del formulario hace lo mismo) |
| `Alt+E` | **Escribir con IA**: abre o cierra el panel bajo la barra de modos (solo con la IA activada) |
| `Alt+R` | Solo en el compositor de commits: inserta una referencia a un issue |
| `Alt+W` | En los formularios de issue y pull request nuevos: crear en la propia página web del proveedor |
| `Esc` | En el panel de **Escribir con IA**: detiene un borrador en curso y, si no, cierra el panel |

En la [vista de conversación](conversations.md) de un issue o de una request (filas de paleta
*Conversación: ...*; las filas con tecla están en la lista de la paleta de arriba):

| Tecla | Acción |
|---|---|
| `r` | Responder a la entrada seleccionada (en su hilo, o con su texto citado en un host sin hilos) |
| `Q` | Citar la entrada seleccionada en un comentario nuevo |
| `c` | Comentario nuevo |
| `e`, `d` | Editar, eliminar uno de tus propios comentarios |
| `Alt+I` | En la caja de comentario: **Nota interna** (GitLab, Redmine) |
| `l`, `y`, `o` | Enlaces de la entrada, copiar su dirección, abrirla en el navegador |
| `n` | Siguiente entrada sin leer |
| `v`, `m`, `F5` | Mostrar u ocultar eventos, cargar entradas anteriores, actualizar |
| `g` | Abrir el diagrama de la entrada (un bloque `mermaid`) |
| `Alt+Left`, `Backspace` | Volver por la pila de elementos abiertos desde referencias |
| `Tab` | Línea de tiempo, caja de comentario, botones de la cabecera |
| `q`, `Esc` | Cerrar |

En el Radar, `i` abre la conversación de una fila de issue.

En una fila de pull request o de issue de las secciones de la barra lateral, `Enter` abre la conversación y `o` abre la fila en el navegador.

### Flow (workflows de ramas)

Con un flujo de trabajo configurado para el repositorio (consulta [Flujos de trabajo de ramas](branching-workflows.md)), la barra de herramientas
tiene un botón **Flow** y la barra lateral agrupa LOCAL y REMOTE por tipo de rama. Cada fila del menú es un comando de la paleta
(**Flow: ...**; **Flow: Empezar feature...** y **Flow: Nuevo issue de tipo feature...** se generan para cada tipo de rama del
flujo de trabajo). Las filas dirigidas por el host (*Nuevo issue de tipo \<tipo\>*, *Recoger una rama creada por el host*) y sus diálogos, la tarjeta de espera y la
tarjeta «ha aparecido una rama» se describen en [Flujos de trabajo de ramas](branching-workflows.md#flujos-dirigidos-por-el-host-en-la-aplicación);
`Enter` ejecuta el botón con el foco (*Crear issue*, *Recoger*), `Esc` deja de esperar o cierra.

| Tecla | Acción |
|---|---|
| `Ctrl+G Shift+W` | Abrir el menú Flow (el botón de la barra de herramientas) |
| `Enter`, `Tab`, `Shift+Tab` | En el diálogo Empezar / Terminar / Sincronizar / Promover / Establecer padre: vista previa o ejecutar, campo siguiente, campo anterior |
| `Space` | Marcar una casilla (*Publicar*, *Ejecutar los hooks*, *Actualizar también las ramas apiladas sobre ella*) |
| `Esc` | Cerrar una lista desplegable, volver de una vista previa al formulario, cerrar el diálogo (no mientras hay una ejecución en curso) |
| Pasar el ratón sobre una rama | Su tipo, su padre y cuánto va por detrás del padre |

## Panel Log

| Tecla | Acción |
|---|---|
| `Enter` / `l` | Entrar en la lista de archivos modificados del commit seleccionado |
| `Enter` / `l` en una fila de archivo | Abrir el diff del archivo en el centro |
| `Esc` / `h` | Volver a los commits |
| `Right` | (commit con más de una ref) Abrir la lista de sus otras refs: `Up` / `Down` mueven, `Enter` hace checkout, `Esc` cierra |
| `f` | Abrir la barra de búsqueda (consulta [Búsqueda](#búsqueda)); nunca fue `/`, así que la búsqueda del Log no cambia |
| `/` | Dar el foco al campo de filtro de Refs (la misma tecla lo hace cuando el panel Refs tiene el foco); se escribe como carácter cuando un campo de texto tiene el foco |
| `c` | (con cambios sin commit) Seleccionar la fila WIP y poner el cursor en su campo de mensaje |
| `Enter` en la fila WIP | Poner el cursor en el campo de mensaje de la fila WIP |
| `.`, `Menu`, `Shift+F10` | El menú del commit. `Up` / `Down` mueven, `Right` / `l` / `Enter` abren un submenú, `Left` / `h` / `Esc` vuelven |
| `m` / `a` | Archivos del panel derecho: `Ruta` o `Árbol` / `Ver todos los archivos` |
| `Shift+C` | Elegir una columna del Log para moverla o ajustarla (véase más abajo) |

Con una columna elegida, `Tab` / `Shift+Tab` eligen otra, `<` / `>` la mueven, `]` / `[` la ensanchan o la estrechan, `0` devuelve todos los anchos al modo automático y `Esc` o `Enter` salen (contexto `log_columns` en la tabla generada de arriba).

El campo de mensaje de la fila WIP es el resumen del compositor de commits.
Mientras tiene el cursor, se escribe cada tecla, aunque los acordes con `Ctrl`
siguen funcionando. `Enter` pasa a la descripción del compositor. `Esc` sale del
campo y conserva el texto. Consulta [Stage y commit](staging-and-committing.md).

### Seleccionar varios commits

| Tecla o gesto | Acción |
|---|---|
| `Space` | Añadir el commit actual a la selección, o quitarlo |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Ampliar la selección (Alt para los terminales que se reservan Shift) |
| `Ctrl+clic` / `Shift+clic` o `Alt+clic` | Alternar un commit / seleccionar un rango |
| `Esc` | Borrar la selección |
| `Shift+Y` | [Copiar la selección como gitGraph de Mermaid](log-and-history.md#copiar-commits-como-gitgraph-de-mermaid) (un popup con niveles de detalle y una vista previa en vivo; `Enter` copia) |

Con varios commits seleccionados, haz clic derecho en uno de ellos y elige
**Cherry-pick de N commits sobre `<rama>`...**. Abre la misma vista que un rebase
interactivo.

### Poner nombre a una rama o tag

`Ctrl+G b`, `Ctrl+G t`, los botones **Rama** y **Tag** de la barra de
herramientas y las opciones **Crear rama aquí...** / **Crear tag aquí...** de los
menús piden el nombre en un cuadro de texto en la fila del commit. `Enter` crea
la ref. `Esc` cancela. Un nombre no válido o ya existente se muestra en rojo, y el
cuadro sigue abierto.

Cuando esa fila no está en pantalla (hay un diff abierto, el Log está aislado o la
columna Ref está oculta), el nombre se escribe en su lugar en un pequeño cuadro con
marco, con una cruz de cierre y los botones **Cancelar** y **Crear**. Renombrar una
rama se hace directamente sobre su fila en el panel Refs, o en la celda Ref de su
fila del Log; crear un archivo se hace en la lista de Cambios de trabajo, bajo la
carpeta. Las mismas teclas valen en todas partes: `Enter` confirma, `Esc` cancela.

Nunca se escribe nada en la línea de estado. Los avisos que necesitan más espacio
son diálogos con botones de verdad: el mensaje del stash, el mensaje de un tag
anotado (`Ctrl+Enter` o `Ctrl+S` aplica, `Enter` añade una línea), Definir el
upstream (una lista con búsqueda de ramas remotas, más un botón Quitar el upstream)
y los patrones de Git LFS que se siguen o se dejan de seguir. Un ajuste numérico o
de texto elegido en la paleta abre Ajustes en esa fila, listo para editar, y el
filtro de Refs (`/`) se escribe en el cuadro de la parte superior del panel Refs.

### Editar el mensaje de un commit

**Editar mensaje del commit...** en el menú del commit, `Ctrl+G e`, o el comando
de la paleta `Git: Editar mensaje del commit...` abre el popup **Reformular
mensaje del commit**. Consulta [Trabajar con el Log](log-and-history.md) para ver
qué cambia.

| Tecla | Acción |
|---|---|
| `Enter` en el resumen | Ir a la descripción |
| `Enter` en la descripción | Línea nueva. `Up` en la primera línea vuelve al resumen |
| `Ctrl+Enter`, `Ctrl+S`, o **Actualizar mensaje** | Guardar el mensaje nuevo |
| `Tab` / `Shift+Tab` | Moverse entre el resumen, la descripción y los botones |
| `Esc`, o **Cancelar** | Cerrar sin cambios |

### Modificar un commit

`Ctrl+G Shift+E` (todos los campos) y `Ctrl+G Shift+A` (el autor) abren el diálogo Modificar, igual que el menú del commit, las filas `Commit: Modificar...` de la paleta y el panel de detalles del commit; todas las formas de abrirlo se describen en [Modificar un commit](log-and-history.md#modificar-un-commit-mensaje-autor-fechas). Ambos acordes se pueden reasignar: `git_prefix.amend` y `git_prefix.edit_author`. Teclas dentro del diálogo:

| Tecla | Acción |
|---|---|
| `Tab` / `Shift+Tab` | Moverse entre los campos y los botones |
| `Enter` en un campo de una línea | Campo siguiente; tras el último, **Aplicar** |
| `Up` / `Down` | Campo anterior / siguiente (en la descripción, desde su primera / última línea) |
| `Ctrl+Enter`, `Ctrl+S`, o **Aplicar** | Aplicar los campos cambiados |
| `Esc`, o **Cancelar** | Cerrar sin cambios |
| `Alt+H`, o **Modificar el historial más a fondo...** | Cerrar el diálogo y abrir el editor de reescritura del historial desde este commit (solo en compilaciones con reescritura del historial); pregunta antes si editaste un campo (`Esc` confirma). Se puede reasignar: `reword.deeper` |

### Crear worktree

**Crear worktree desde `<ref>`...** (menús de rama y de tag) y **Crear worktree
aquí...** (menú del commit) abren un diálogo: la carpeta, una rama nueva
opcional y **Abrir el nuevo worktree en una pestaña**. Teclas: `Tab` / `Shift+Tab`
mueven, `Enter` crea, `Space` marca o desmarca la casilla, `Ctrl+B` examina,
`Esc` cancela.

### Rebase interactivo

**Rebase interactivo desde aquí...** en el menú del commit, o
`Git: Rebase interactivo desde el commit seleccionado...`, abre la vista de
rebase. Un cherry-pick de varios commits usa la misma vista. Consulta
[Trabajar con el Log](log-and-history.md).

| Tecla | Acción |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Fijar la acción de la fila: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup y Edit solo en un rebase) |
| `Space` | Abrir la lista de acciones de la fila (`Up` / `Down` eligen, `Enter` aplica, `Esc` cierra la lista) |
| `Up` / `Down` | Mover la selección |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Mover la fila seleccionada hacia arriba / abajo |
| `a` | Activar o desactivar la vista previa de autosquash |
| `Enter` en una fila Reword | Editar el mensaje nuevo de ese commit |
| `Ctrl+S`, `Ctrl+Enter`, o **Iniciar rebase** / **Iniciar cherry-pick** | Ejecutar el plan |
| `Ctrl+R`, o **Restablecer** | Restaurar el plan tal como estaba al abrir la vista |
| `Esc`, o **Cancelar** | Cerrar sin cambios |

Cuando un rebase se detiene (por un conflicto o un paso Edit), un banner sobre el
Log ofrece **Continuar**, **Omitir** y **Abortar**. La paleta tiene
`Rebase: Continuar`, `Rebase: Omitir` y `Rebase: Abortar`.

## Búsqueda

`f`, el botón **Buscar** de la barra de herramientas o
`Navegación: Buscar commits...` abre la barra de búsqueda sobre el Log. Busca en
el mensaje completo del commit, el nombre y el correo del autor, el SHA y los
nombres de las refs. No oculta filas: las filas que no coinciden se atenúan.

| Tecla | Acción |
|---|---|
| *(escribir)* | Editar el texto de búsqueda (se escribe cada tecla) |
| `Enter` / `Down`, o el `↑` de la barra | Coincidencia siguiente |
| `Shift+Enter` / `Up`, o el `↓` de la barra | Coincidencia anterior |
| `Esc`, o el `✕` de la barra | Cerrar la barra de búsqueda |

## Vista Diff

Un archivo modificado se abre en el centro, en lugar del Log. Consulta
[Stage y commit](staging-and-committing.md).

| Tecla | Acción |
|---|---|
| `Enter` / `l`, o un doble clic (en una fila de archivo) | Abrir el diff del archivo |
| `Esc`, `q`, o el `✕` del título | Cerrar el diff |
| `j` / `k`, `Up` / `Down` | Desplazar |
| `PageUp` / `PageDown`, `Home` / `End` | Desplazar una página; ir al principio / al final |
| `h` / `l`, `Left` / `Right` | Desplazar lateralmente (no con ajuste de línea) |
| `n` / `p` | Hunk siguiente / anterior |
| `m` | Disposición en línea o dividida |
| `Shift+H` | Solo hunks o archivo completo |
| `w` | Activar o desactivar el ajuste de línea |
| `s` | Mostrar u ocultar los cambios que solo afectan a espacios en blanco |
| `Shift+M` | Activar o desactivar el minimapa |
| `Shift+C` | Elegir la codificación con la que se lee el archivo (una lista: primero Detección automática) |
| `e` | Editar el archivo en el editor integrado (consulta [Editor](#editor)) |
| `r` | (archivo de Git LFS) La tarjeta LFS o el puntero en bruto. Consulta [Git LFS](lfs.md) |
| `Shift+E` | (archivo en conflicto) Abrir el editor de merge. Consulta [Resolver conflictos de merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copiar el texto seleccionado. Sin selección: la línea bajo el cursor, o las líneas seleccionadas |
| `.`, o un clic derecho en un hunk | El menú del hunk |

### Stage y descarte desde el diff

Estas teclas funcionan solo en un diff de los cambios de trabajo (abierto desde
la lista Unstaged o Staged). El descarte funciona solo desde la lista Unstaged.

| Tecla | Acción |
|---|---|
| `Space` | Stage (Unstaged) / unstage (Staged) del hunk bajo el cursor |
| `v` | Empezar o abandonar una selección de líneas; `j` / `k` la amplían |
| `Enter` | Stage / unstage de las líneas seleccionadas, o de la línea bajo el cursor |
| `x` | Descartar el hunk bajo el cursor (pregunta antes: `y` / `Enter` confirman, `n` / `Esc` cancelan) |
| `Shift+X` | Descartar las líneas seleccionadas, o la línea bajo el cursor (pregunta antes) |

Solo cuentan las líneas modificadas: las líneas de contexto de una selección se
ignoran. Una selección que llega hasta un segundo hunk actúa solo sobre el
primer hunk.

## Vista de imagen

Una imagen en la vista Diff o en la Vista de archivo. Las teclas están en el
contexto `image` de la tabla anterior. Consulta [Imágenes](image-view.md).

## Vista de archivo, Blame e Historial de archivo

| Tecla | Acción |
|---|---|
| `Esc` / `q` | Cerrar (con una selección de texto, `Esc` la borra primero) |
| `j` / `k`, `Up` / `Down` | Mover la selección |
| `PageUp` / `PageDown` | Desplazar una página |
| `Home` / `End`, `Shift+G` | Primera / última línea |
| `h` / `l`, `Left` / `Right` | Desplazar el texto lateralmente |
| `Enter` | (Blame, Historial de archivo) Ir al commit de la fila en el Log |
| `e` | Editar el archivo en el editor integrado |
| `y` / `Ctrl+C` | Copiar el texto seleccionado. Sin selección: la fila seleccionada (Blame: solo el código; Historial del archivo: el resumen) |

Blame y el historial del archivo tienen sus propias teclas, listadas en la tabla generada de arriba (contextos `file_insight` y `history`) y como `Blame: ...` / `Historial del archivo: ...` en la paleta; todas se pueden reasignar. Detalles en [Blame](log-and-history.md#blame) e [Historial del archivo](log-and-history.md#historial-del-archivo). En la lista del historial, las letras sueltas (`d` `o` `b` `c` `y`) actúan sobre la fila seleccionada, pero un acorde con Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) conserva siempre su significado global.

## Columna de commits

La columna a la izquierda de la vista de un archivo (consulta [La columna de commits](interface-tour.md#la-columna-de-commits)). `Alt+C` la muestra u oculta desde cualquier sitio; las demás teclas funcionan cuando tiene el teclado (`Ctrl+N` y su insignia, o `Ver: Enfocar la columna de commits`).

| Tecla | Acción |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Elegir un commit |
| `Enter` / `Right` | Pasar a la lista de archivos del commit elegido |
| `Esc` | Devolver el teclado a la vista |
| `.`, `Menu`, `Shift+F10` | El menú de la fila elegida |

Sus otras teclas (`Shift+Left` / `Shift+Right` para desplazar hacia los lados, `Tab`, `]`, `[` y `0` para dimensionar las partes) están en la tabla generada de arriba, contexto `commit_column`.

## Editor

**Editar**, o `e` en un diff o en una Vista de archivo, abre el editor integrado
sobre la copia del archivo en el directorio de trabajo. Consulta
[Stage y commit](staging-and-committing.md).

| Tecla | Acción |
|---|---|
| *(escribir)* | Insertar en el cursor (reemplaza la selección) |
| `Enter` | Línea nueva, con la sangría de la línea actual |
| `Tab` / `Shift+Tab` | Aumentar / reducir la sangría (de las líneas seleccionadas, si son varias) |
| flechas, `Home` / `End` | Mover. `Home` va al primer carácter que no es un espacio, y luego a la columna 0. `Shift` amplía la selección |
| `Ctrl+Left` / `Ctrl+Right` | Mover una palabra |
| `Ctrl+Home` / `Ctrl+End` | Principio / final del archivo |
| `PageUp` / `PageDown` | Una página hacia arriba / abajo |
| `Ctrl+A` | Seleccionar todo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copiar / cortar / pegar |
| `Ctrl+U` / `Ctrl+K` | Borrar hasta el principio / el final de la línea |
| `Ctrl+Z` | Deshacer |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Rehacer |
| `Ctrl+S` | Guardar |
| `Ctrl+F` / `F3` | Buscar. En el campo de búsqueda: `Enter` / `Down` / `F3` siguiente, `Shift+Enter` / `Up` anterior, `Esc` cierra |
| `Alt+M` | Activar o desactivar el minimapa (**Map**) |
| `Esc` | Cerrar. Con cambios sin guardar pregunta: **Guardar** (`S` / `Enter`), **Descartar** (`D`), **Cancelar** (`C` / `Esc`) |

`Tab` aplica sangría en el editor, así que no mueve el foco. Los demás acordes
con `Ctrl` (`Ctrl+P`, `Ctrl+N`) siguen funcionando.

La **codificación**, los **finales de línea** y la **sangría** de la barra de estado son
botones: cada uno abre un menú desplegable (flechas o rueda, escribe para filtrar, `Enter`
para elegir, `Esc` para cerrar). La paleta (`Ctrl+P`) abre los mismos menús: *Editor: Cambiar codificación*, *Editor: Reabrir con codificación*, *Editor: Convertir a codificación*, *Editor: Convertir saltos de línea*, *Editor: Cambiar sangría*.
Detalles en [Stage y commit](staging-and-committing.md).

## Acciones de Git (`Ctrl+G`)

`Ctrl+G` espera una tecla más:

| Tecla | Acción |
|---|---|
| `s` / `u` | Stage / unstage del archivo seleccionado |
| `a` | Stage de todos los cambios |
| `x` | Descartar todos los cambios (pregunta antes) |
| `c` | Abrir el compositor de commits |
| `Enter` | Hacer commit ya de los cambios en stage, con el mensaje del formulario, desde cualquier sitio (consulta [Stage y commit](staging-and-committing.md#escribir-el-commit)) |
| `m` | Stash de todos los cambios con un mensaje, escrito en un pequeño diálogo (se admite vacío). Con la IA activada, su botón de IA (`Ctrl+G` en el diálogo) escribe el mensaje |
| `Shift+T` | Stash de todos los cambios (incluidos los archivos sin seguimiento), sin mensaje |
| `o` | Pop de un stash: la fila de stash seleccionada en el Log o, si no, el último stash |
| `p` | Pull, según está configurado el botón Pull (`git.pull_mode`) |
| `Shift+P` | Push al remoto predeterminado (fija el upstream en el primer push), o pregunta por el remoto |
| `Shift+U` | Push de todos los tags |
| `f` | Fetch de todos los remotos, con prune |
| `g` | Abrir en el navegador la pull request de la rama con checkout |
| `b` / `t` | Rama nueva / tag nuevo en HEAD (consulta [Poner nombre a una rama o tag](#poner-nombre-a-una-rama-o-tag)) |
| `d` | Eliminar la rama o el tag seleccionados (pregunta antes) |
| `e` | Editar el mensaje del commit seleccionado |
| `Shift+E` / `Shift+A` | Modificar el commit seleccionado (todos los campos) / su autor (consulta [Modificar un commit](#modificar-un-commit)) |
| `y` | Cherry-pick del commit seleccionado |
| `v` | Revert del commit seleccionado |
| `Shift+R` | Reset (mixed) al commit seleccionado |
| `r` | Rebase sobre la rama seleccionada |
| `Shift+W` | Abrir el menú Flow (en un repositorio con un flujo de trabajo de ramas) |
| `Shift+H` | Abrir el editor de reescritura del historial (solo en compilaciones con reescritura del historial) |
| `l` | Abrir el menú LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch de todo / checkout de todo / pull / push de todo / prune. Consulta [Git LFS](lfs.md) |
| `i` | ✦ IA: escribir el mensaje de commit de los cambios en stage |
| `w` | ✦ IA: explicar el commit seleccionado, o los cambios de trabajo |
| `Shift+C` | ✦ IA: proponer cómo dividir los cambios de trabajo en commits |

Las teclas de IA solo funcionan con la IA activada (consulta
[Asistencia de IA](ai.md)). `Esc` cancela una petición de IA en curso.

## Panel derecho (cambios de trabajo y detalles del commit)

Con la fila WIP seleccionada, el panel derecho muestra las listas **Unstaged** y
**Staged** y el compositor de commits. Con un commit seleccionado, muestra los
detalles del commit y sus archivos.

| Tecla | Acción |
|---|---|
| `j` / `k`, `Up` / `Down` | Mover. Al final de una lista, la selección continúa en la otra |
| `h` / `l`, `Left` / `Right` | Ir a la lista Unstaged / Staged |
| `Enter` | Contraer o expandir la lista activa |
| `Space` / `Insert` | Añadir el archivo actual a la selección, o quitarlo |
| `Ctrl+A` | Seleccionar todos los archivos de la lista |
| `Ctrl+Space` | Stage / unstage del archivo seleccionado, o plegar una fila de carpeta |
| `d` | Abrir el diff del archivo seleccionado |
| `m` | `Ruta` (lista plana) o `Árbol` (carpetas) |
| `a` | `Ver todos los archivos`: todo el árbol, no solo los archivos modificados |
| `z` | Contraer todas las carpetas, o expandirlas de nuevo |
| `/` | Filtrar por nombre los archivos modificados de un commit |
| `.`, `Menu`, `Shift+F10` | El menú del archivo |
| `c` | Dar el foco al campo de resumen del compositor de commits |
| `Shift+A` | Activar o desactivar **Modificar el commit anterior** |
| `Ctrl+N ]` / `Ctrl+N [` | Agrandar / reducir el bloque del mensaje de commit |

En el compositor de commits:

| Tecla | Acción |
|---|---|
| `Enter` | Resumen: ir a la descripción. Descripción: línea nueva |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` lo hace desde cualquier sitio) |
| `Alt+R` | Insertar una referencia a un issue |
| `Esc` | Salir del compositor |

El menú del archivo contiene: Stage o Unstage, Descartar cambios, Ignorar, Stash
del archivo, Historial del archivo, Blame del archivo, Abrir en herramienta de
diff externa, Abrir en editor externo, Abrir archivo en el programa
predeterminado, Mostrar en la carpeta, Copiar ruta del archivo, Crear parche con
los cambios del archivo, Editar archivo y Eliminar archivo. En un archivo en
conflicto empieza con **Abrir en el editor de merge** y las tres filas
**Resolver con**. Con Git LFS activado, tiene filas de LFS. **Descartar cambios**
y **Eliminar archivo** preguntan antes.

## Botones de la ventana

Cada botón tiene su equivalente de teclado:

| Botón | Tecla |
|---|---|
| <img src="../../../assets/logo/gitpoulpi.png" alt="Icono de GitPoulpi" height="16"> a la izquierda de la barra de título (Acerca de) | `Ayuda: Acerca de GitPoulpi` en la paleta |
| **Abrir en** en la barra de título | `Alt+O`, `Ctrl+Alt+O` |
| `×` a la derecha de la barra de título (salir) | `q` |
| La versión en la barra de título (Notas de la versión) | `Ayuda: Notas de la versión` en la paleta |
| `⌕ Búsqueda global` en la barra de título, a la izquierda del `⚙` | `Ctrl+Alt+F` |
| `⚙` en la barra de título (Ajustes) | `Ctrl+,` |
| El chip del espacio de trabajo en la fila de pestañas | `Alt+W` |
| `+` tras la última pestaña | `Alt+T` |
| La pestaña **Radar** | `Alt+R` |
| La pestaña **Espacios de trabajo** | `Alt+J` |
| **Actividad** en la barra de estado | `Ctrl+N Shift+A` |
| Barra de herramientas: **Deshacer**, **Rehacer** | `Ctrl+Z`, `Ctrl+Y` |
| Barra de herramientas: **Pull** / su `▾` | `Ctrl+G p` / `Git: Opciones de pull...` |
| **Pull de todas las ramas** (fila del `▾` de Pull) | `Ctrl+G Ctrl+P` |
| Barra de herramientas: **Push** / su `▾` | `Ctrl+G Shift+P` / `Git: Push a un remoto...` |
| Barra de herramientas: **Rama**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Barra de herramientas: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Barra de herramientas: **LFS** (cuando LFS está activado) | `Ctrl+G l` |
| Barra de herramientas: **Flow** (cuando el repositorio tiene un flujo de trabajo) | `Ctrl+G Shift+W` |
| Barra de herramientas: **Buscar** | `f` |
| Barra de herramientas: la pull request vinculada (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Barra de herramientas: la lista de perfiles (con dos o más perfiles) | `Repositorio: Cambiar de perfil...` |

La barra de herramientas se describe en el
[Recorrido por la interfaz](interface-tour.md#barra-de-herramientas).

## Ajustes

`Ctrl+,`, `Ctrl+N ,`, el engranaje `⚙` o `App: Ajustes (Preferencias)...` abre la
pantalla de Ajustes.

| Tecla | Acción |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Moverse entre la lista de páginas y las opciones |
| `j` / `k`, `Up` / `Down` | Mover la selección |
| `Enter` / `Space` | Abrir una página, marcar o desmarcar una casilla, editar un campo, abrir un desplegable o pulsar un botón |
| `/`, `Ctrl+F` | Buscar en todas las opciones |
| `Alt+Left` / `Alt+Right` | Cambiar de subpestaña (General / la lista) |
| `Esc` / `q` | Cerrar los Ajustes. Mientras se edita un campo, `Esc` cancela solo la edición |
| `?` | Abrir la ayuda sobre la pantalla de Ajustes |

Consulta [Ajustes](settings.md).

## Campos de texto

Todos los campos de texto comparten las mismas teclas: filtros, campos de
búsqueda, la paleta de comandos, prompts, el compositor de commits, el campo de
la fila WIP, los campos de los diálogos y los campos de los Ajustes. Mientras un
campo tiene el teclado, estas teclas prevalecen sobre los atajos de los paneles.

| Tecla | Acción |
|---|---|
| un carácter | Escribirlo en el cursor (reemplaza la selección) |
| `Left` / `Right` | Mover un carácter (con selección: ir a su principio / final) |
| `Ctrl+Left` / `Ctrl+Right` | Mover una palabra |
| `Home` / `End` | Principio / final de la línea |
| `Shift` + un movimiento anterior | Ampliar o reducir la selección |
| `Up` / `Down` (solo en la descripción) | Línea anterior / siguiente |
| `Ctrl+Home` / `Ctrl+End` (solo en la descripción) | Principio / final del texto |
| `Ctrl+A` | Seleccionar todo |
| `Backspace` / `Delete` | Borrar el carácter anterior / posterior al cursor, o la selección |
| `Ctrl+U` | Borrar desde el cursor hasta el principio de la línea (o la selección) |
| `Ctrl+K` | Borrar desde el cursor hasta el final de la línea (o la selección) |
| `Ctrl+C` / `Ctrl+X` | Copiar / cortar la selección (no en un campo de contraseña) |
| pegar (el pegado propio de el terminal) | Insertar en el cursor, reemplazando la selección |

En la descripción del commit, `Enter` escribe una línea nueva y `Ctrl+Enter`
hace el commit. No todas los terminales envían `Ctrl+Enter` de la misma manera,
así que `Alt+Enter` y `Ctrl+J` también hacen el commit.

Algunas consolas envían `Ctrl+letra` como un carácter de control (`Ctrl+A` como
`0x01`). GitPoulpi entiende las dos formas. Los caracteres de `AltGr` (`@`, `#`,
`€` en teclados suizos o franceses) se escriben.

## Descripciones emergentes

Aparece una descripción emergente cuando el puntero permanece unos 0,6 s sobre un
elemento (una celda del Log, un botón de la barra de herramientas, la pull
request vinculada). Una pulsación de tecla, un clic, un desplazamiento, un
cambio de tamaño, un popup o menú abierto, o que el puntero abandone el elemento,
la oculta. Después, solo vuelve a aparecer cuando el puntero se mueve de nuevo.
