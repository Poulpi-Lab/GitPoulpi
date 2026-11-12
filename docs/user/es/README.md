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

## Uso diario

4. **[Trabajar con el Log](log-and-history.md)**: leer el grafo, búsqueda,
   ramas, tags, stashes, worktrees, arrastrar refs, rebase, cherry-pick,
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
8. **[Deshacer y rehacer](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, qué se puede y
   qué no se puede deshacer, cuándo se niega o pregunta, hasta dónde llega,
   cómo desactivarlo.
9. **[Git LFS](lfs.md)**: archivos grandes, seguimiento, bloqueos, y qué pasa
   sin `git-lfs` instalado.
10. **[Submódulos](submodules.md)**: la sección SUBMÓDULOS, cómo actualizarlos y
    cómo mantenerlos al ritmo de los checkouts y los pulls.
11. **[Temas](themes.md)**: los temas integrados, cualquier tema con cualquier
    paleta de carriles, vista previa en vivo, tus propios archivos de tema,
    `gitpoulpi theme`.
12. **[Espacios de trabajo](workspaces.md)**: grupos con nombre de repositorios
    con sus propias pestañas y un perfil de commit predeterminado opcional, el
    chip de la barra de pestañas, la pestaña Espacios de trabajo, Fetch todo, Pull todo, Clonar los ausentes.
13. **[Búsqueda](search.md)**: un buscador aproximado (`Ctrl+Alt+F`) sobre las
    ramas, tags, worktrees, stashes, commits, pull/merge requests e issues de
    todos los repositorios del espacio de trabajo o de las pestañas abiertas.
14. **[El Radar](radar.md)**: las pull y merge requests, los issues y el
    trabajo en curso de un espacio de trabajo (o de tus pestañas abiertas) en una
    sola lista: filtros, fijaciones, posposiciones y qué puede hacer cada fila.
    Leer y revisar una de esas solicitudes dentro de la aplicación, con hilos de
    comentarios, una revisión pendiente y Aprobar / Solicitar cambios:
    **[Revisar una pull request](review.md)**.
15. **[Abrir en](open-in.md)**: la lista de la barra de título que abre la
    carpeta de la pestaña activa en un terminal, un multiplexor, un editor o un
    gestor de archivos.
16. **[Seguir la carpeta de la pestaña activa](following-the-tab-folder.md)**:
    «Salir y cd aquí», y la función `gitpoulpi shell-init` (`gp`) para bash,
    zsh, fish y PowerShell.
    Qué terminal muestra mejor GitPoulpi: **[Elegir un terminal](terminals.md)**.
    Qué pasa a través de tmux, zellij, screen, herdr y los paneles de terminal:
    **[Multiplexores](multiplexers.md)**.

Además: **[Flujos de trabajo de ramas](branching-workflows.md)**, las plantillas Git
Flow, GitHub Flow, GitLab Flow y Trunk-based, o tus propias bases, patrones de
nombre de rama y tipos de rama (`gitpoulpi flow ...`).

**[Reescritura del historial](history-rewrite.md)**: el editor de reescritura
(`Ctrl+G Shift+H`) que reescribe autores, fechas, mensajes y contenido de archivos en
muchos commits, con vista previa, scripts Lua y un único deshacer.

## Teclado y ratón

17. **[Atajos de teclado](keybindings.md)**: cada tecla, por zona, y cada
    comando de la paleta. En la aplicación, `?` muestra la misma lista y
    `Ctrl+P` encuentra cualquier comando.
18. **[Ratón y gestos](mouse-and-gestures.md)**: hacer clic, arrastrar,
    seleccionar y copiar, barras de desplazamiento y respuesta al pasar el
    puntero.

## Ajustes e integraciones

19. **[Ajustes](settings.md)**: cada página y opción de la pantalla de Ajustes,
    el cuadro de búsqueda, los perfiles y cómo reasignar atajos.
20. **[Asistencia de IA](ai.md)**: la IA propia de GitPoulpi (desactivada por
    defecto): un agente local como `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama o LM Studio; instrucciones; mensajes de commit, explicaciones,
    descripciones de pull requests y el compositor de commits; privacidad y
    solución de problemas.
21. **[Alojamiento y gestores de issues](hosting-integration.md)**: GitHub, GitLab, Gitea y Forgejo,
    Azure DevOps, Bitbucket, Gerrit y los gestores de issues: tokens y sus permisos, conexión,
    vincular un repositorio, pull requests e issues, referencias en los mensajes
    de commit, claves SSH, límites conocidos. También
    **[Issues locales y Obsidian](local-issues.md)**: una carpeta de notas
    Markdown como origen de issues, sin token, con una Base de Obsidian.

## Automatización

22. **[Referencia de la CLI](cli-reference.md)**: cada comando de `gitpoulpi`,
    con `--json` para scripts.
23. **[MCP y agentes](mcp-and-agents.md)**: deja que un agente externo (Claude
    Code, Cursor, Codex, ...) maneje GitPoulpi: registrar el servidor,
    `.mcp.json`, los interruptores de permisos y sus valores por defecto, cada
    herramienta y cómo te muestra cosas un agente en tu ventana. La skill de
    agente ya preparada es
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referencia y ayuda

24. **[Referencia de configuración](config-reference.md)**: cada clave de
    `config.toml`, su valor por defecto y su efecto, los ajustes por
    repositorio, las variables de entorno.
25. **[Solución de problemas y preguntas frecuentes](troubleshooting.md)**:
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
