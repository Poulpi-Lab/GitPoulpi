[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · **Español** · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Compilaciones personalizadas

Para quien quiere un binario más pequeño: un script interactivo pregunta qué idiomas,
integraciones de alojamiento, temas, colores del logo y funciones incluir, y compila solo eso con
el perfil optimizado. La compilación normal y la versión oficial no cambian (todo activado).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd o PowerShell (scripts/build-custom.ps1)
```

El resultado queda en `target/custom/`, nunca sobre el `target/dist/` oficial. El script
imprime el tamaño de cada binario y la diferencia con `target/dist/` cuando esa compilación
existe. También escribe `BUILD-FEATURES.txt` junto a los binarios.

## Qué eliges

| Opción | Si la desactivas |
|---|---|
| Idiomas | Solo inglés (siempre incluido) más los que marques. El selector de Ajustes lista solo los idiomas compilados. |
| Temas | GitPoulpi Dark, GitPoulpi Light y Terminal siempre se compilan (`system` necesita Dark y Light). Cualquier otro tema se elige por familia (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` las imprime). Los temas no seleccionados no se compilan: faltan en el combo de Ajustes y en `gitpoulpi theme list`, y una configuración que nombre uno se trata como un tema desconocido, con el aviso habitual. |
| Colores del logo | El pulpo de píxeles de la ventana Acerca de viene en 20 combinaciones de color (`--list-logo-colors`). Elige las que compilar (al menos una), la predeterminada (Acerca de la muestra primero; también es a la que recurre un `ui.logo_color` guardado que no esté compilado; debe ser una de las compiladas; predeterminada oficial: lavender) y el color del icono del ejecutable de Windows (cualquier combinación, por defecto: la predeterminada del logo). Con una sola combinación, el logo sigue girando al hacer clic pero conserva ese color, y no se ofrece la lista de colores (`c`, clic derecho). |
| Integraciones (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Sin ninguna: solo clonar por URL; sin issues ni pull/merge requests. |
| IA | Sin botones ni comandos de IA, y sin las herramientas `ai` de la CLI y del MCP. |
| MCP | Sin binario `gitpoulpi-mcp` ni comando `gitpoulpi mcp`. Los comandos de agente `ui` se mantienen. |
| LFS, worktree, submódulos | Las pantallas de gestión y los comandos correspondientes desaparecen; se devuelve una respuesta tipada «not built into this binary». |
| Descarga de avatares | Solo insignias con iniciales, sin descarga por red. |
| Vista previa de imágenes | Los archivos de imagen muestran el aviso binario simple; **Editar en** sigue funcionando. |
| Radar | Sin pestaña Radar, ni `Alt+R` ni comando de la paleta; el Radar no se abre nunca. Necesita una integración (sin ninguna queda inerte de todos modos). |
| Mermaid | Sin Copiar como gitGraph de Mermaid (menú del Log, `Shift+Y`, paleta) y sin `log --format mermaid` en la CLI y el MCP. Tampoco hay botón **Abrir diagrama** bajo un bloque `mermaid` de una [conversación](hosting-integration.md#5c-conversaciones) (el bloque queda como código) ni la fila de paleta *Conversación: Abrir el diagrama de la entrada*. |
| Board | Sin vista de tablero Kanban (botón de la cabecera ISSUES, paleta) y sin el comando `hosting board`. Necesita una integración. |
| Review | Sin vista de revisión de código en la app (menú de fila, Enter en una fila de pull request, paleta, `Review` del Radar) y sin asistente de revisión con IA. Los comandos `hosting review` de la CLI y del MCP se mantienen (Gerrit y los agentes los usan). Necesita una integración. |
| Conversation | Sin vista de [conversación](hosting-integration.md#5c-conversaciones) en la app para un issue o una solicitud: Enter en una fila de issue, una tarjeta del tablero o una fila del Radar abre el navegador como antes, no hay fila *Abrir conversación* ni comandos de paleta *Conversación: ...*, y tampoco **Escribir con IA** en una conversación (los cuadros de texto de la revisión, del formulario de issue y del de pull request conservan el suyo). Los comandos de conversación de la CLI y el MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) y `ai hosting-reply` se mantienen. Requiere una integración. |
| Editor del historial (reescritura avanzada del historial: autores, fechas, mensajes, contenido) | Sin Editor de reescritura del historial: sin comandos de la paleta `Reescritura:`, sin atajo, teclas ni sección de ayuda del editor, sin filas «Reescribir el historial» en los menús del Log, sin el botón «Modificar el historial más a fondo...» en el diálogo Enmendar, sin «Crear reglas con IA» ni IA para scripts, sin límites de scripts en la página de ajustes Sync y sin bloques de IA de reescritura del historial en los ajustes de IA. Los comandos `rewrite` de la CLI y el MCP, `ai rewrite-script`, `ai rewrite-rules` y `ui propose rewrite` desaparecen del árbol de comandos. Las identidades de `.mailmap` (`mailmap`, `log.use_mailmap`) no forman parte de él y se mantienen. |
| Búsqueda global | Sin botón Búsqueda global en la barra de título, sin `Ctrl+Alt+F`, sin comando de la paleta `Buscar: Espacio de trabajo...` y sin comando `gitpoulpi search` de la CLI y el MCP. La barra de búsqueda del Log (`ui show search`) es otra función y se mantiene. |
| Espacios de trabajo | La ventana trabaja con la única lista de pestañas por defecto («Sin espacio de trabajo»): sin chip de espacio de trabajo, sin pestaña Espacios de trabajo, sin `Alt+W` / `Alt+J`, sin comandos de la paleta `Espacio de trabajo:`, sin filas «Añadir al espacio de trabajo» en las pestañas y sin comandos `gitpoulpi workspace` de la CLI y el MCP, ni opción `--workspace` en `commit` y `search`. Las pestañas y las sesiones funcionan como siempre. Un `workspaces.toml` escrito por otra compilación no se lee ni se modifica; el perfil por defecto y las instrucciones de IA de un espacio de trabajo no se aplican. El Radar y la Búsqueda global cubren las pestañas abiertas. |
| Scripts de reescritura (Lua integrado) | Las reglas Script de una reescritura del historial se rechazan con una respuesta tipada «no incluido en este binario», y no se enlaza ningún Lua. El Editor del historial en sí se mantiene (tiene su propia opción). No tiene sentido, y se avisa, cuando el Editor del historial está desactivado. |

Tus archivos de configuración se comparten entre compilaciones: un ajuste de algo que no está
compilado se conserva y se ignora.

## Interactivo

El script de Bash usa listas de casillas de `whiptail` o `dialog` (tres pantallas y después un
resumen), o preguntas numeradas cuando no hay ninguno de los dos. El script de PowerShell usa
`Out-GridView` cuando está disponible y, si no, preguntas. Tus elecciones anteriores vienen
marcadas.

## No interactivo

Cualquier argumento omite las preguntas y va directamente a
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # elecciones anteriores, solo imprime el comando de cargo
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Opción | Significado |
|---|---|
| `--langs fr,de` / `all` / `en` | Idiomas además del inglés. Los códigos son los nombres de los archivos de idioma (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Proveedores: GitHub, GitLab (incl. autogestionado), Azure DevOps (Services y Server), Bitbucket (Cloud y Data Center) (alojamiento), Gitea y Forgejo incl. Codeberg (`gitea`) (alojamiento), Jira, Trello, Linear, YouTrack (Cloud y Server), Redmine (gestores de issues), Gerrit Code Review (`gerrit`, solo revisión), issues locales de Markdown (`local-issues`, una carpeta o un vault de Obsidian). Cada uno de `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` es su propia feature de cargo; los tamaños están en [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Familias de temas a compilar (por defecto `all`). `--list-themes` imprime las familias. |
| `--logo-colors all` / `violet,teal` | Combinaciones de color del logo a compilar (por defecto `all`, en el orden de la tabla, que es el orden de giro). `--list-logo-colors` las imprime (`*` = predeterminada oficial). |
| `--logo-default teal` | La combinación predeterminada; debe estar entre las compiladas (por defecto: lavender, o si no la primera compilada). |
| `--icon-color sun` | Color del icono del ejecutable de Windows (por defecto: la predeterminada del logo). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Cada una está activada por defecto. |
| `--[no-]history-rewrite` | El Editor del historial (reescritura avanzada del historial: autores, fechas, mensajes, contenido). Activado por defecto. |
| `--[no-]global-search` | Búsqueda global (botón de la barra de título, `Ctrl+Alt+F`, `gitpoulpi search`). Activada por defecto. |
| `--[no-]workspaces` | Espacios de trabajo (grupos con nombre de repositorios). Activados por defecto; desactivados deja solo la lista de pestañas por defecto. |
| `--[no-]rewrite-scripts` | Lua integrado para las reglas Script de una reescritura. Activado por defecto; se ignora con una advertencia si el Editor del historial está desactivado. |
| `--save <name\|file>` | Guarda las elecciones resueltas como selección guardada (un nombre o una ruta de archivo). |
| `--preset <name\|file>` | Parte de una selección guardada (un nombre o una ruta de archivo); las demás opciones la sobrescriben, estén donde estén en la línea. |
| `--list-presets`, `--delete-preset <name>` | Muestra las selecciones guardadas y su carpeta / borra una por su nombre. |
| `--last` | Reutiliza `target/custom-build/last-choices.env`; las demás opciones lo sobrescriben. |
| `--full` | Sin ninguna selección (todo activado). |
| `--profile custom\|dist\|release` | Por defecto `custom` (los ajustes de `dist`, con un directorio de salida aparte). |
| `--dry-run` | Imprime el comando exacto de cargo y se detiene. |
| `-- <cargo args>` | Se pasan a cargo, por ejemplo `--target x86_64-unknown-linux-musl`. |

Nombres en PowerShell: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

La selección de temas llega a la compilación como la variable de entorno
`GITPOULPI_THEMES` (`all`, `none` o una lista de familias separadas por comas), que lee el
script de compilación de `gitpoulpi-config` y que elimina las demás paletas en tiempo de
compilación; defínela tú mismo para un `cargo build` normal si lo prefieres. `gitpoulpi
--version` imprime una línea `themes:` con el conjunto compilado.

Las elecciones del logo son las variables de entorno `GITPOULPI_LOGO_COLORS`
(`all` o una lista de slugs separados por comas), `GITPOULPI_LOGO_DEFAULT` y
`GITPOULPI_ICON_COLOR`, que leen los scripts de compilación de `gitpoulpi-tui` (filtra
`assets/logo/colourways.json` hasta la tabla que incrusta la ventana Acerca de) y de los tres
crates binarios (el icono). La fuente de los colores sigue siendo
`assets/logo/colourways.json`.

### Icono del ejecutable de Windows

Cuando el destino es Windows, los ejecutables `gitpoulpi`, `gitpoulpi-tui` y `gitpoulpi-mcp`
reciben el pulpo como icono, generado durante la compilación (sin necesidad de Python ni
ImageMagick) en el color de icono elegido, con tamaños de 16 a 128 px. Se incrusta con el
crate `winresource` y necesita el compilador de recursos de la plataforma (`rc.exe` del
Windows SDK con MSVC, `windres` con GNU). Si falta, la compilación solo imprime una
advertencia y el ejecutable conserva el icono predeterminado. Con la toolchain GNU solo
`gitpoulpi` lleva el icono (el recurso, si no, se enlazaría dos veces). Las compilaciones
para Linux y macOS ignoran todo esto salvo la tabla de colores.

## Selecciones guardadas

Para recompilar una versión nueva sin repetir toda la selección, guárdala con un nombre y parte de ella más adelante. Las opciones que des junto con `--preset` siguen sobrescribiendo los valores guardados.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Las selecciones guardadas son archivos `CLAVE=valor` sencillos (el formato de `last-choices.env`, legible desde un shell). Viven en la carpeta `custom-builds` del directorio de configuración de GitPoulpi, fuera de `target/`, así que `cargo clean` no las borra:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` por defecto)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` mueve esa carpeta junto con la configuración. `--preset` y `--save` también aceptan una ruta de archivo (un valor que contiene `/` o `\`, o que termina en `.env`), para guardar una selección en un repositorio de dotfiles.

Una selección escrita por otra versión de GitPoulpi se carga igualmente. Una clave desconocida, o un idioma, integración, tema o color que ya no existe, se ignora con una advertencia. Una clave que falta en el archivo (una función añadida después) toma su valor por defecto, y una integración añadida después se incluye; ambas cosas se informan por la salida de error.

Cuando existe una selección guardada, el modo interactivo ofrece primero «Partir de» (las elecciones anteriores, una selección guardada o todo activado), y al final pregunta si quieres guardar la selección actual con un nombre.

## Cuánto ahorra

Medido en Linux el 2026-10-29 en el commit `65999ef5c`, a partir de la compilación completa, con la diferencia respecto a ella entre paréntesis
(detalles en `docs/developer/binary-size.md`):

| Compilación (desde `full` salvo indicación) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Completa (todo activado) | 64,6 | 50,5 | 31,7 |
| Solo inglés | 52,0 (-12,6) | 40,2 (-10,3) | 31,7 (0,0) |
| MCP desactivado | 61,6 (-3,0) | 49,6 (-0,8) | no se compila |
| IA desactivada | 62,6 (-2,0) | 48,6 (-1,8) | 30,5 (-1,3) |
| Todas las integraciones desactivadas | 56,3 (-8,2) | 43,7 (-6,7) | 26,7 (-5,0) |
| IA + integraciones + descarga de avatares desactivadas | 52,2 (-12,3) | 39,9 (-10,6) | 23,0 (-8,8) |
| Editor del historial desactivado (`--no-history-rewrite`) | 61,6 (-3,0) | 48,9 (-1,5) | 28,5 (-3,2) |
| Scripts de reescritura desactivados | 63,1 (-1,5) | 49,0 (-1,5) | 31,1 (-0,7) |
| Búsqueda global desactivada | 63,5 (-1,1) | 49,5 (-1,0) | 31,6 (-0,1) |
| Espacios de trabajo desactivados | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,2) |
| Radar desactivado | 63,4 (-1,2) | 49,3 (-1,2) | 31,7 (0,0) |
| Board desactivado | 63,6 (-1,0) | 49,5 (-1,0) | 31,7 (0,0) |
| Review desactivado | 63,2 (-1,4) | 49,0 (-1,4) | 31,7 (0,0) |
| Mermaid desactivado | 63,6 (-1,0) | 49,5 (-0,9) | 31,6 (-0,1) |
| LFS desactivado | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,3) |
| Worktree desactivado | 63,7 (-0,9) | 49,6 (-0,8) | 31,7 (0,0) |
| Submódulos desactivados | 63,7 (-0,9) | 49,6 (-0,9) | 31,6 (-0,1) |
| Descarga de avatares desactivada | 63,7 (-0,9) | 49,6 (-0,9) | 31,7 (0,0) |
| Vista previa de imágenes desactivada | 63,3 (-1,2) | 49,2 (-1,3) | 31,7 (0,0) |
| Todo desactivado, solo inglés (LFS se mantiene) | 35,2 (-29,4) | 28,9 (-21,5) | no se compila |
| **Todo desactivado, solo inglés** | **34,9 (-29,7)** | **28,8 (-21,6)** | no se compila |

## Notas

- Una compilación personalizada es lenta como cualquier compilación `dist` (LTO completo,
  unos minutos).
- Se ejecuta sin sccache, igual que `build-opti`.
- Los códigos de idioma se comprueban contra los idiomas incluidos; un código desconocido se
  rechaza con la lista de los válidos.
