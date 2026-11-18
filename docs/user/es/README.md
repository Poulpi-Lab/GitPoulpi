[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · **Español** · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Documentación de usuario de GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="El pulpo de GitPoulpi" width="128"></p>

GitPoulpi es un cliente de Git para el terminal con un grafo de commits en el
que se puede hacer clic, una paleta de comandos, integración con servicios de
alojamiento e issue trackers, ayuda de IA opcional, y una CLI y un servidor MCP
que hacen todo lo que hace la interfaz. Estas páginas están en orden de
lectura: empieza por arriba y salta a las páginas de referencia cuando las
necesites.

![La ventana principal de GitPoulpi](../images/hero.png)

## Empieza aquí

1. **[Instalación](installation.md)**: qué necesitas, archivos de release,
   gestores de paquetes, compilar desde el código fuente. Para un binario más
   pequeño, consulta **[Compilaciones personalizadas](custom-builds.md)**. Cómo
   se mantiene al día GitPoulpi (el chip de versión, el panel de actualización,
   volver a la versión anterior): **[Actualizaciones](updates.md)**. Para
   ejecutarlo en un contenedor: **[Ejecutar en Docker](docker.md)**. Para
   administradores, feeds internos, espejos y la política de actualización:
   **[Distribución en empresas](enterprise-distribution.md)**.
2. **[Primeros pasos](getting-started.md)**: tus primeros cinco minutos, desde
   el arranque hasta tu primer commit.
3. **[Recorrido por la interfaz](interface-tour.md)**: el nombre de cada zona de
   la ventana, con imágenes.

## Encontrar una función

| Quiero... | Página | Cómo llegar en la aplicación |
|---|---|---|
| Trabajar en dos ramas a la vez | [Worktrees](worktrees.md) | La sección **WORKTREES** de la barra lateral Refs (`+` crea uno; oculta hasta que existe uno: paleta **Panel izquierdo: Mostrar la sección Worktrees**); paleta **Refs: Crear worktree...** |
| Trabajar con repositorios anidados | [Submódulos](submodules.md) | La sección **SUBMÓDULOS** de la barra lateral Refs; paleta **Git: Actualizar todos los submódulos**, **Refs: Actualizar submódulo (seleccionado)** |
| Resolver un conflicto de merge | [Resolver conflictos de merge](merge-conflicts.md) | `Enter` sobre una fila `!` del panel Estado, el botón **Resolver conflictos...** del banner, o `Shift+E` en la vista Diff; paleta **Conflictos: Abrir el editor de merge** |
| Revisar una pull o merge request | [Revisar una pull request](review.md) | `Enter` sobre una fila de la sección **PULL REQUESTS**, `e` sobre una fila del Radar, `Alt+R` en la búsqueda del espacio de trabajo; paleta **Alojamiento: Revisar solicitud...** |
| Explorar, crear y gestionar pull requests e issues | [Pull requests e issues](pull-requests-and-issues.md) | Las secciones **PULL REQUESTS** e **ISSUES** de la barra lateral Refs (`n` crea uno); paleta **Pull requests: Crear pull request...**, **Alojamiento: Nuevo issue...**. Antes, conecta una cuenta: [Alojamiento y gestores de issues](hosting-integration.md) |
| Leer y responder la discusión de un issue o de una request | [Conversaciones](conversations.md) | `Enter` sobre una fila de issue, una tarjeta del tablero o una fila del Radar; paleta **Alojamiento: Abrir la conversación de un issue...** |
| Ver mis issues como un tablero | [El tablero de issues](pull-requests-and-issues.md#el-tablero-de-issues) | El botón **Tablero** del encabezado de ISSUES; paleta **Issues: Abrir tablero** |
| Guardar issues como notas Markdown, sin token | [Issues locales y Obsidian](local-issues.md) | La sección ISSUES, proveedor `Local Markdown - <nombre>`; paleta **Issues locales: Abrir en Obsidian...** |
| Reescribir autores, fechas, mensajes o archivos en muchos commits | [Reescritura del historial](history-rewrite.md) | `Ctrl+G Shift+H`; paleta **Reescritura: Editor del historial** |
| Deshacer (o rehacer) una operación | [Deshacer y rehacer](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` abre el historial de deshacer; paleta **Edición: Deshacer** |
| Seguir archivos grandes con Git LFS | [Git LFS](lfs.md) | Paleta **LFS: Seguir patrón...**, **LFS: Inicializar en este repositorio** |
| Encontrar una rama, un commit, una pull request o un issue en varios repositorios | [Búsqueda](search.md) | `Ctrl+Alt+F`, o el botón Búsqueda global de la barra de título; `f` busca en el grafo de commits del repositorio actual |
| Ver en una sola lista todo lo que me espera | [El Radar](radar.md) | `Alt+R`; paleta **Radar: Abrir** |
| Agrupar repositorios y hacerles fetch o pull juntos | [Espacios de trabajo](workspaces.md) | `Alt+W` cambia, `Alt+J` abre la pestaña Espacios de trabajo; paleta **Espacio de trabajo: Gestionar...** |
| Recibir ayuda de IA (mensajes de commit, explicaciones, descripciones) | [Asistencia de IA](ai.md) | `Ctrl+,` > **IA** > **Activar IA**, y luego `Ctrl+G i` escribe un mensaje de commit; paleta **IA: ...** |
| Encontrar cualquier comando o tecla | [Atajos de teclado](keybindings.md) | `?` en la aplicación lista las teclas, `Ctrl+P` encuentra cualquier comando |

## Uso diario

4. **[Trabajar con el Log](log-and-history.md)**: leer el grafo, búsqueda,
   ramas, tags, stashes, arrastrar refs, rebase, cherry-pick,
   reword y modificación de commits, banners, pull y push, auto-stash.
5. **[Stage y commit](staging-and-committing.md)**: los cambios de trabajo,
   diffs, stage de hunks y de líneas sueltas, escribir commits, el compositor de
   commits, blame, historial de archivos y el editor integrado.
6. **[Imágenes](image-view.md)**: imágenes en la vista Diff y en la Vista de
   archivo, antes y después, zoom, Editar en.
7. **[Resolver conflictos de merge](merge-conflicts.md)**: el editor de merge (A
   y B en paralelo sobre el resultado), elegir líneas y bloques, tarjetas para
   conflictos binarios, de LFS, de archivos borrados, de enlaces simbólicos y de
   submódulos, y cómo propone una resolución un agente.
8. **[Worktrees](worktrees.md)**: una segunda carpeta con checkout en otra rama:
   la sección WORKTREES, crear, abrir, hacer merge y eliminar worktrees
   vinculados.
9. **[Submódulos](submodules.md)**: la sección SUBMÓDULOS, cómo actualizarlos y
   cómo mantenerlos al ritmo de los checkouts y los pulls.
10. **[Deshacer y rehacer](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, qué se puede y
    qué no se puede deshacer, cuándo se niega o pregunta, hasta dónde llega,
    cómo desactivarlo.
11. **[Git LFS](lfs.md)**: archivos grandes, seguimiento, bloqueos, y qué pasa
    sin `git-lfs` instalado.
12. **[Temas](themes.md)**: los temas integrados, cualquier tema con cualquier
    paleta de carriles, vista previa en vivo, tus propios archivos de tema,
    `gitpoulpi theme`.
13. **[Espacios de trabajo](workspaces.md)**: grupos con nombre de repositorios
    con sus propias pestañas y un perfil de commit predeterminado opcional, el
    chip de la barra de pestañas, la pestaña Espacios de trabajo, Fetch todo, Pull todo, Clonar los ausentes.
14. **[Búsqueda](search.md)**: un buscador aproximado (`Ctrl+Alt+F`) sobre las
    ramas, tags, worktrees, stashes, commits, pull/merge requests e issues de
    todos los repositorios del espacio de trabajo o de las pestañas abiertas.
15. **[El Radar](radar.md)**: las pull y merge requests, los issues y el
    trabajo en curso de un espacio de trabajo (o de tus pestañas abiertas) en una
    sola lista: filtros, fijaciones, posposiciones y qué puede hacer cada fila.
16. **[Pull requests e issues](pull-requests-and-issues.md)**: las secciones
    PULL REQUESTS e ISSUES de la barra lateral, el tablero de issues, crear una
    pull request o un issue, el chip de la barra de herramientas, las referencias
    en los mensajes de commit.
17. **[Conversaciones](conversations.md)**: leer y responder la discusión de un
    issue o de una request dentro de la ventana, publicar como, qué no puede
    hacer cada host.
18. **[Revisar una pull request](review.md)**: la vista de revisión con hilos de
    comentarios, una revisión pendiente y Aprobar / Solicitar cambios.
19. **[Abrir en](open-in.md)**: la lista de la barra de título que abre la
    carpeta de la pestaña activa en un terminal, un multiplexor, un editor o un
    gestor de archivos.
20. **[Seguir la carpeta de la pestaña activa](following-the-tab-folder.md)**:
    «Salir y cd aquí», y la función `gitpoulpi shell-init` (`gp`) para bash,
    zsh, fish y PowerShell.
    Qué terminal muestra mejor GitPoulpi: **[Elegir un terminal](terminals.md)**.
    Qué pasa a través de tmux, zellij, screen, herdr y los paneles de terminal:
    **[Multiplexores](multiplexers.md)**.

Además: **[Varios remotos para una rama](multiple-remotes.md)**: destinos de push y
fuentes de pull, el diálogo Remotos, Push a todos o a remotos seleccionados, resolver un
remoto que se movió, Continuar tras un conflicto, la CLI y un alias de git a secas.

**[Flujos de trabajo de ramas](branching-workflows.md)**, las plantillas Git
Flow, GitHub Flow, GitLab Flow y Trunk-based, o tus propias bases, patrones de
nombre de rama y tipos de rama (`gitpoulpi flow ...`).

**[Reescritura del historial](history-rewrite.md)**: el editor de reescritura
(`Ctrl+G Shift+H`) que reescribe autores, fechas, mensajes y contenido de archivos en
muchos commits, con vista previa, scripts Lua y un único deshacer.

## Teclado y ratón

21. **[Atajos de teclado](keybindings.md)**: cada tecla, por zona, y cada
    comando de la paleta. En la aplicación, `?` muestra la misma lista y
    `Ctrl+P` encuentra cualquier comando.
22. **[Ratón y gestos](mouse-and-gestures.md)**: hacer clic, arrastrar,
    seleccionar y copiar, barras de desplazamiento y respuesta al pasar el
    puntero.

## Ajustes e integraciones

23. **[Ajustes](settings.md)**: cada página y opción de la pantalla de Ajustes,
    el cuadro de búsqueda, los perfiles y cómo reasignar atajos.
24. **[Asistencia de IA](ai.md)**: la IA propia de GitPoulpi (desactivada por
    defecto): un agente local como `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama o LM Studio; instrucciones; mensajes de commit, explicaciones,
    descripciones de pull requests y el compositor de commits; privacidad y
    solución de problemas.
25. **[Alojamiento y gestores de issues](hosting-integration.md)**: GitHub, GitLab, Gitea y Forgejo,
    Azure DevOps, Bitbucket, Gerrit y los gestores de issues: tokens y sus permisos,
    conexión, vincular un repositorio, claves SSH, límites conocidos. También
    **[Issues locales y Obsidian](local-issues.md)**: una carpeta de notas
    Markdown como origen de issues, sin token, con una Base de Obsidian.

## Automatización

26. **[Referencia de la CLI](cli-reference.md)**: cada comando de `gitpoulpi`,
    con `--json` para scripts.
27. **[MCP y agentes](mcp-and-agents.md)**: deja que un agente externo (Claude
    Code, Cursor, Codex, ...) maneje GitPoulpi: registrar el servidor,
    `.mcp.json`, los interruptores de permisos y sus valores por defecto, cada
    herramienta y cómo te muestra cosas un agente en tu ventana. La skill de
    agente ya preparada es
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referencia y ayuda

28. **[Referencia de configuración](config-reference.md)**: cada clave de
    `config.toml`, su valor por defecto y su efecto, los ajustes por
    repositorio, las variables de entorno.
29. **[Solución de problemas y preguntas frecuentes](troubleshooting.md)**:
    iconos, dibujo del grafo, teclas que no llegan, daemons bloqueados,
    velocidad, punteros de LFS.

## Capturas de pantalla

Cada imagen de estas páginas es una captura real de una build de release, sobre
un pequeño repositorio de demostración con autores inventados. Las imágenes
están en [`images/`](../images/). `scripts/docs-screenshots.sh` crea los
repositorios de demostración y las vuelve a tomar. Se toman en el terminal
`kitty`, que dibuja el grafo de commits y los avatares de los autores como
imágenes. Los terminales sin protocolo gráfico muestran en su lugar un grafo de
caracteres.

## Para quien contribuye

[CLAUDE.md](../../../CLAUDE.md), la [visión general de la arquitectura](../../architecture/overview.md)
y la [hoja de ruta](../../roadmap.md) describen el diseño y lo que está
planificado; [CHANGELOG.md](../../../CHANGELOG.md) enumera lo que cambió en cada
release.

---

GitKraken es una marca de su respectivo propietario. GitPoulpi es un proyecto independiente, sin afiliación, respaldo ni patrocinio por parte de este; el nombre se menciona solo para describir la compatibilidad y la inspiración.

Todos los demás nombres de productos y servicios mencionados en GitPoulpi y en su documentación son marcas de sus respectivos propietarios, por ejemplo los alojamientos de código (GitHub, GitLab, Bitbucket, etc.), los gestores de incidencias (Jira, Trello, Linear, etc.) y los editores y herramientas que GitPoulpi puede abrir (Visual Studio Code, IDE de JetBrains, lazygit, etc.). Se usan solo para nombrar aquello con lo que GitPoulpi funciona; GitPoulpi no tiene afiliación, respaldo ni patrocinio por parte de ninguno de ellos.
