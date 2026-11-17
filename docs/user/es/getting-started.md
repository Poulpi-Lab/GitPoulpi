[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · **Español** · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Primeros pasos

Esta página te lleva desde una instalación recién hecha hasta tu primer commit.
El [recorrido por la interfaz](interface-tour.md) nombra cada zona de la
ventana. Los [Ajustes](settings.md) enumeran todas las opciones.

## 1. Instalar y arrancar

Sigue la [Instalación](installation.md). Después, abre un terminal en un
repositorio Git y ejecuta:

```
gitpoulpi-tui
```

`gitpoulpi` sin subcomando hace lo mismo. GitPoulpi abre el repositorio que
contiene la carpeta actual. Fuera de un repositorio, abre la pestaña **Inicio**,
donde puedes examinar, clonar o inicializar un repositorio (consulta el
[paso 7](#7-abrir-clonar-e-inicializar-repositorios)).

Los iconos necesitan una Nerd Font o una fuente con emojis. Si ves recuadros,
consulta la [Solución de problemas y preguntas frecuentes](troubleshooting.md).

![La ventana principal: la barra lateral Refs a la izquierda, el grafo del Log en el centro y los cambios de trabajo con el formulario de commit a la derecha](../images/hero.png)

## 2. Tres teclas para aprender todo lo demás

| Tecla | Abre |
|---|---|
| `?` | La **ayuda**: todas las teclas y gestos del ratón. Escribe para filtrarla. `Ctrl+D` abre esta documentación en tu navegador. |
| `Ctrl+P` | La **paleta de comandos**: escribe para encontrar un comando, `Enter` lo ejecuta. Cada fila muestra su atajo. |
| `Ctrl+,` | Los **Ajustes**. El engranaje de la barra de título, a la izquierda de la ×, también los abre. |

Puedes hacerlo todo con el teclado o con el ratón. La paleta da acceso a todos
los comandos. Consulta los [Atajos de teclado](keybindings.md) y
[Ratón y gestos](mouse-and-gestures.md).

![La paleta de comandos filtrada por la palabra stage, con los comandos de stage y sus atajos](../images/palette.png)

## 3. Mirar el historial

El centro de la ventana es el **Log**: una fila por commit, del más reciente al
más antiguo. Muestra las ramas y los tags como etiquetas de color, el grafo, el
mensaje, el autor, la fecha y el hash corto.

- Haz clic en un commit. Sus detalles y los archivos modificados aparecen a la
  derecha.
- Haz clic en un archivo modificado. Su diff se abre en el centro. `Esc` lo
  cierra.
- `j` / `k` o las flechas mueven la selección. `f` busca entre los commits.
- Haz clic derecho en un commit para ver su menú.

![El Log con un commit de merge seleccionado: su mensaje, su autor y los dos archivos que añade, a la derecha](../images/log-graph.png)

## 4. Hacer un cambio y pasarlo a stage

Edita un archivo en tu editor. La **fila superior del Log** muestra ahora los
cambios de trabajo: una fila discontinua con el número de archivos modificados.
Haz clic en ella. El panel derecho muestra **Archivos unstaged** y **Archivos
staged**.

- Pasa el puntero sobre un archivo y haz clic en su botón **Stage**. O haz clic
  en **Stage todo**.
- Haz clic en el nombre de un archivo para abrir su diff. Allí puedes hacer
  stage de un **hunk** (**Stage hunk**) o de una **línea**.

Consulta [Stage y commit](staging-and-committing.md).

![El diff de un archivo modificado, con botones Stage hunk y Descartar hunk encima de cada hunk](../images/diff.png)

## 5. Hacer commit

Escribe un resumen, y opcionalmente una descripción, en el formulario de commit
de la parte inferior derecha. Después, haz clic en el botón verde de commit
(**Commit de cambios en N archivos**) o pulsa `Ctrl+Enter` (o `Ctrl+G Enter`, desde cualquier sitio). Marca **Modificar el
commit anterior** para añadir tus cambios al último commit en lugar de crear
uno nuevo.

Si has activado la [IA](ai.md), el botón `✦` junto al resumen escribe un
mensaje a partir de los cambios en stage. GitPoulpi nunca hace commit hasta que
tú haces clic en el botón de commit.

## 6. Compartir tu trabajo

La barra de herramientas tiene los botones **Pull** y **Push**, cada uno con una
lista `▾`. Desde el teclado, pulsa `Ctrl+G` y después una letra: `Ctrl+G p` hace
pull, `Ctrl+G Shift+P` hace push. Tras `Ctrl+G`, la barra de estado enumera las
letras.

GitPoulpi usa la autenticación que ya usa tu `git` (credential helper,
ssh-agent). No necesita cuenta propia. Para ver y crear pull requests, conecta
una cuenta de alojamiento una sola vez. Consulta
[Integraciones de alojamiento y de gestores de issues](hosting-integration.md).

## 7. Abrir, clonar e inicializar repositorios

Haz clic en el icono de la casa, al extremo izquierdo de la fila de pestañas, o
pulsa `Ctrl+O`. La pestaña **Inicio** enumera tus repositorios abiertos,
favoritos y recientes, con los botones **Examinar**, **Clonar** e **Init**. Cada
repositorio se abre en su propia pestaña. `Ctrl+W` cierra la pestaña actual.

![La pestaña Inicio con la lista de repositorios abiertos y recientes y los botones Examinar, Clonar e Init](../images/home.png)

**Clonar** pide una URL y una carpeta, o te deja elegir un repositorio de una cuenta conectada (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; consulta [Clonar desde una cuenta conectada](hosting-integration.md#clonar-desde-una-cuenta-conectada)). **Init** convierte una carpeta en un
repositorio nuevo.
En los tres diálogos (y en el selector de carpetas que abren), `Ctrl+N` muestra una insignia en cada campo y botón: pulsa el carácter para poner el cursor en el campo o pulsar el botón.

| ![El diálogo Clonar: dónde clonar, la URL, el nombre de la carpeta, y los botones Cancelar y Clonar](../images/clone.png) | ![El diálogo Inicializar un repositorio: la carpeta, el nombre y la rama inicial](../images/init.png) |
|---|---|

### Checkout parcial

Un repositorio grande no tiene por qué llegar entero al disco. En el diálogo
Clonar, marca **Checkout parcial (solo algunos directorios)** y escribe los
directorios que quieres, separados por espacios o comas (`src docs/guide`). Solo
se hace checkout de esos directorios y de los archivos de la raíz del
repositorio; el historial completo se descarga igualmente, así que el Log, blame
y las ramas funcionan como siempre. Con la casilla marcada hace falta al menos un
directorio.

![El diálogo Checkout parcial con dos directorios escritos y los botones Cancelar, Desactivar y Aplicar](../images/sparse-checkout.png)

Cualquier repositorio se puede cambiar más tarde: abre la paleta de comandos
(`Ctrl+P`) y ejecuta **Repositorio: Checkout parcial...**, o usa **Checkout
parcial...** en el menú de la pestaña del repositorio. El diálogo lista en un
solo campo los directorios de los que hay checkout: añade o quita nombres y
pulsa **Aplicar** (`Enter`), o **Desactivar (checkout completo)** para recuperar
todos los archivos. Mientras el árbol de trabajo es parcial, la barra de estado
muestra una pequeña insignia `parcial`. Los mismos controles son una página de
*Ajustes > Checkout parcial* (una lista de rutas, una por línea, con los botones **Iniciar**, **Detener** y **Actualizar**).

Notas:

- Es el modo cone de git (directorios, no patrones); se rechazan `*`, `..` y las
  rutas absolutas.
- Deshacer no registra el cambio del conjunto: no se pierde nada, y
  **Desactivar** restaura todos los archivos. Git puede negarse a quitar un
  directorio que tiene cambios sin confirmar (el error aparece en la línea de
  estado); haz commit o stash antes.
- Lo mismo desde scripts y agentes: `gitpoulpi clone <url> --sparse src docs` y
  `gitpoulpi sparse-checkout status | set | add | disable` (consulta la
  [referencia de la CLI](cli-reference.md)); el servidor MCP tiene las mismas
  herramientas.

## Siguientes pasos

- [Recorrido por la interfaz](interface-tour.md): el nombre de todo lo que ves.
- [Trabajar con el Log](log-and-history.md) y
  [Stage y commit](staging-and-committing.md): el trabajo de cada día.
- [Temas](themes.md) y [Ajustes](settings.md).
- [Asistencia de IA](ai.md) y [Integraciones de alojamiento y de gestores de issues](hosting-integration.md):
  las dos integraciones opcionales.
- [Referencia de la CLI](cli-reference.md) y [MCP y agentes](mcp-and-agents.md):
  las mismas operaciones desde scripts y agentes.
- [Solución de problemas y preguntas frecuentes](troubleshooting.md).
