[English](README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · **Español** · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="assets/logo/gitpoulpi-512.png" alt="El pulpo de GitPoulpi" width="128"></p>

**Un cliente de Git para la terminal con la comodidad de una aplicación de
escritorio.** GitPoulpi te ofrece un grafo de commits completo en el que puedes
hacer clic, arrastrar y pulsar con el botón derecho, una paleta de comandos
pensada para el teclado que da acceso a todas las funciones, integración con servicios de alojamiento de código (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, en la nube o autoalojados) y con gestores de issues (Jira, Trello, Linear, YouTrack, Redmine o una carpeta de notas Markdown), y las mismas operaciones como CLI automatizable y como
servidor MCP, para que un agente de IA lo maneje igual que tú. GitPoulpi está
inspirado en GitKraken y reescrito desde cero en Rust: binarios nativos, ligeros en disco y en memoria, sin Electron, sin cuenta y sin telemetría. Funciona en tu terminal favorita ([¿cuál?](docs/user/es/terminals.md)), porque algunos vivimos ahí dentro.

![GitPoulpi mostrando el grafo de commits con ramas, tags y un stash, la barra lateral Refs a la izquierda y los cambios de trabajo con el formulario de commit a la derecha](docs/user/images/hero.png)

- **Ratón y teclado, a partes iguales.** Haz clic, doble clic, clic derecho y
  arrastra refs sobre los commits; o pulsa `Ctrl+P` y escribe lo que quieres.
  Cada gesto del ratón tiene su equivalente de teclado, y todos los comandos
  están en la paleta.
- **Un grafo de commits de verdad.** Ramas, merges, tags, stashes y la copia de
  trabajo en un único Log con desplazamiento, dibujado como píxeles en las
  terminales que lo admiten y como caracteres en el resto. Muchos temas de color
  oscuros y claros, avatares de autores, búsqueda.
- **Stage con precisión.** Haz stage, unstage o descarta un archivo, una
  carpeta, un hunk o una sola línea directamente desde el diff. En línea o en
  paralelo, con minimapa.
- **Edita el historial con seguridad.** Reformula o modifica cualquier commit
  (mensaje, autor, fechas), haz rebase interactivo con una pantalla de revisión,
  reescribe muchos commits a la vez con una vista previa y un solo deshacer, haz
  cherry-pick de varios commits, fast-forward, crea worktrees y parches.
- **Alojamiento e issue trackers integrados.** Explora, haz checkout y crea pull y merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) y revisa los cambios de Gerrit. Explora y crea issues, work items y tarjetas (también Jira, Trello, Linear, YouTrack, Redmine y notas Markdown locales), si lo prefieres en un tablero Kanban. Clona desde tu cuenta y gestiona las claves SSH.
- **CLI y MCP para agentes.** `gitpoulpi <comando> --json` cubre todo lo que
  hace la interfaz, y `gitpoulpi-mcp` lo sirve como herramientas MCP. Un daemon
  en segundo plano por repositorio mantiene todos los frontends en directo y
  sincronizados.
- **IA opcional, nunca obligatoria.** Desactivada por defecto y sin cuenta: usa
  tu propia clave de API o una CLI de agente local como `claude -p`. Genera
  mensajes de commit, explica cambios, redacta descripciones de pull requests,
  divide tus cambios de trabajo en varios commits y prepara reglas de
  reescritura del historial.
- **Git LFS, submódulos, worktrees, parches, blame e historial de archivos,
  diffs de imágenes, un editor integrado.**

## Capturas de pantalla

| | |
|---|---|
| ![El grafo del Log con el tema de color Material y el tinte de fila activado](docs/user/images/theme-material.png) **El grafo de commits**, aquí con el tema Material y el tinte de fila. | ![El diff de un archivo modificado con botones Stage hunk y Descartar hunk en cada hunk](docs/user/images/diff.png) **El diff**, con stage y descarte por hunk o por línea. |
| ![La paleta de comandos filtrada por la palabra stash](docs/user/images/palette.png) **La paleta de comandos** (`Ctrl+P`) da acceso a todos los comandos. | ![El compositor de commits proponiendo cuatro commits para los cambios de trabajo, listos para revisar](docs/user/images/composer.png) **El compositor de commits** revisa los commits propuestos por la IA antes de confirmar nada. |
| ![El menú contextual de un commit con acciones de cherry-pick, revert, reset, rebase, rama y tag](docs/user/images/context-menu.png) **Clic derecho** en cualquier elemento para ver sus acciones. | ![La página IA de los Ajustes con perfiles, instrucciones por acción y el interruptor de activación](docs/user/images/settings-ai.png) **Los Ajustes**, con búsqueda, incluida la página opcional de IA. |

Encontrarás más imágenes, con explicaciones, en la [documentación de usuario](docs/user/es/README.md).

## Instalación

Necesitas un `git` del sistema; `git-lfs` es opcional.

- **Con un instalador:** un `...-setup.exe` para Windows o `install.sh` para Linux y macOS (por usuario, sin root), adjunto a cada versión. Una compilación instalada desde un archivo de versión también puede actualizarse sola desde el chip de versión de la barra de estado.
- **Desde un archivo de release:** descomprímelo y sigue `INSTALL.txt`, que
  encontrarás dentro.
- **Desde el código fuente:**

  ```
  git clone https://github.com/Poulpi-Lab/GitPoulpi.git
  cd GitPoulpi
  cargo build --release
  ```

  Los binarios quedan en `target/release/`: `gitpoulpi` (la CLI, y la interfaz
  cuando se ejecuta sin subcomando), `gitpoulpi-tui` y `gitpoulpi-mcp`.
- **Crear tu propio archivo de release:** `cargo run -p xtask -- dist` genera
  `target/dist/gitpoulpi-<version>-<target>.zip` (Windows) o `.tar.gz`, con un
  archivo `.sha256`.
- **Un binario más pequeño con solo lo que necesitas:** `./build-custom.sh`
  (Windows: `build-custom`) pregunta qué idiomas, integraciones y funciones (IA,
  MCP, LFS, worktrees, submódulos, descarga de avatares) compilar; consulta
  [docs/user/es/custom-builds.md](docs/user/es/custom-builds.md).

Detalles completos, herramientas opcionales y notas por sistema operativo en
[docs/user/es/installation.md](docs/user/es/installation.md).

## Inicio rápido

```
cd path/to/your/repository
gitpoulpi-tui          # o simplemente: gitpoulpi
```

- `?` abre la ayuda con todas las teclas y gestos, `Ctrl+P` la paleta de
  comandos, `Ctrl+,` los Ajustes y `q` sale.
- Haz clic en un commit para ver sus cambios; haz clic en la fila superior del
  Log (los cambios de trabajo) para hacer stage de archivos y commit.
- En una terminal que lo admita (kitty, Ghostty, WezTerm, iTerm2 (y también Windows Terminal y foot, mediante sixel)) el grafo se
  dibuja como píxeles. En las demás (una consola de Windows sin más, tmux)
  vuelve automáticamente a los caracteres.

La versión paso a paso está en [Primeros pasos](docs/user/es/getting-started.md),
y [el recorrido por la interfaz](docs/user/es/interface-tour.md) nombra cada
zona de la pantalla.

```
gitpoulpi --json status            # la CLI, automatizable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # da a un agente de IA las mismas herramientas
```

La IA es opcional y está desactivada por defecto: activa la IA propia de
GitPoulpi en *Ajustes > IA* ([Asistencia de IA](docs/user/es/ai.md)); deja que
un agente externo maneje GitPoulpi con
[MCP y agentes](docs/user/es/mcp-and-agents.md) (permisos, todas las
herramientas) y con la skill de agente
[`skills/gitpoulpi/SKILL.md`](skills/gitpoulpi/SKILL.md).

## Documentación

La documentación de usuario completa empieza en **[docs/user/es/README.md](docs/user/es/README.md)**:

| | |
|---|---|
| [Primeros pasos](docs/user/es/getting-started.md) | Instalar, abrir un repositorio, hacer un primer commit |
| [Recorrido por la interfaz](docs/user/es/interface-tour.md) | Cada zona de la ventana, con su nombre |
| [Trabajar con el Log](docs/user/es/log-and-history.md) | Grafo, refs, ramas, tags, stashes, rebase, cherry-pick |
| [Stage y commit](docs/user/es/staging-and-committing.md) | Estado, diff, hunks y líneas, el compositor, el editor |
| [Ajustes](docs/user/es/settings.md) | Cada página y opción de los Ajustes |
| [Asistencia de IA](docs/user/es/ai.md) | Configurar la IA integrada, todas las acciones de IA, privacidad |
| [Integraciones de alojamiento y de gestores de issues](docs/user/es/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokens, vincular un repositorio, pull y merge requests |
| [CLI](docs/user/es/cli-reference.md) y [MCP](docs/user/es/mcp-and-agents.md) | Automatízalo, o deja que lo maneje un agente (permisos, todas las herramientas) |
| [Atajos de teclado](docs/user/es/keybindings.md) (y todos los comandos de la paleta), [ratón](docs/user/es/mouse-and-gestures.md), [configuración](docs/user/es/config-reference.md), [LFS](docs/user/es/lfs.md), [deshacer](docs/user/es/undo-redo.md), [solución de problemas](docs/user/es/troubleshooting.md) | Referencia y ayuda |

Para desarrolladores: [CLAUDE.md](CLAUDE.md) y `docs/architecture/overview.md`
describen los crates y el diseño; [CHANGELOG.md](CHANGELOG.md) enumera lo que
trae cada release.

## Licencia

[MIT](LICENSE).

## Créditos y licencias de terceros

GitPoulpi se basa en bibliotecas de código abierto y usa valores de color de temas publicados. Sus licencias y créditos, incluidos los autores de los temas, están en [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). El archivo se incluye en cada versión; la ventana Acerca de y `gitpoulpi licenses` también lo muestran.

## Marcas

GitKraken es una marca de su respectivo propietario. GitPoulpi es un proyecto independiente, sin afiliación, respaldo ni patrocinio por parte de este; el nombre se menciona solo para describir la compatibilidad y la inspiración.
