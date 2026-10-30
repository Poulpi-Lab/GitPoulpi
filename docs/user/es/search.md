[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · **Español** · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Búsqueda

`Ctrl+Alt+F` (o `Ctrl+P` y luego `Buscar: Espacio de trabajo...`) abre un
buscador aproximado sobre **todos los repositorios en los que trabajas**: los
repositorios del [espacio de trabajo](workspaces.md)
activo cuando hay uno, y si no, las pestañas abiertas. El botón **Búsqueda global** (la lupa y su etiqueta
en la barra de título, justo a la izquierda del engranaje) también lo abre. Encuentra:

- **repositorios** (por nombre, luego por ruta),
- **ramas** (locales y de seguimiento remoto) y **tags**,
- **worktrees** (por carpeta, luego por ruta y rama),
- **stashes** (por mensaje),
- **commits** (por asunto; el cuerpo de un commit solo se busca cuando su asunto
  no coincide y has escrito tres caracteres o más),
- **pull o merge requests** e **issues** (por título, `#número`, autor y rama).

![La ventana de búsqueda sobre dos repositorios abiertos: ramas, un stash y un commit que coinciden con "main", cada uno con su repositorio a la derecha](../images/workspace-search.png)

Los resultados se agrupan por tipo, con la mejor coincidencia primero dentro de
cada grupo. Cada fila tiene el icono del tipo, los caracteres coincidentes
resaltados, un detalle atenuado (el upstream de una rama, el autor y la rama de
una solicitud, la posición de un stash) y el nombre del repositorio a la
derecha, así que dos repositorios con una rama `main` nunca se confunden. Aparece
un indicador giratorio en el título cuando un repositorio aún está indexando sus
commits (la primera búsqueda en un repositorio grande puede tardar un momento);
los demás tipos están disponibles al instante y los commits se suman en cuanto
están listos.

## Ir allí

| Tecla | Hace |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Moverte por los resultados (los encabezados de grupo se saltan). |
| `Enter` | Va al resultado: se abre la pestaña del repositorio (o se cambia a ella) y luego se muestra el resultado. |
| `Esc` | Cierra la búsqueda. |
| `Alt+1` .. `Alt+8` | Activa o desactiva un tipo (los chips sobre la lista, en su orden). `Alt+0` los activa todos. |

Qué significa "ir allí":

| Resultado | Enter |
|---|---|
| Rama o tag | La selecciona en el panel Refs y selecciona su commit en el Log. |
| Commit | Lo selecciona en el Log, cargando antes más Log cuando es más antiguo que lo cargado. |
| Stash | Selecciona su fila en la sección STASHES. |
| Worktree | Lo abre como pestaña. |
| Pull/merge request, issue | Selecciona su fila en la barra lateral (en la pestaña de ese repositorio). |
| Repositorio | Cambia a su pestaña (abriéndola si no está abierta). |

`Ctrl+N Backspace` (o `Atrás` en la paleta) vuelve a donde estabas, como
cualquier cambio de vista que haga un agente por ti. Ir a un sitio no hace
checkout, merge ni cambia nada.

Con el ratón: un clic en una fila la selecciona, un doble clic va allí, un clic
en un chip activa o desactiva ese tipo, la rueda desplaza y un clic fuera cierra
la ventana.

## Acotar con un prefijo

Empieza la consulta con un prefijo para buscar solo un tipo, digan lo que digan
los chips:

| Prefijo | Tipo |
|---|---|
| `r:` | repositorios |
| `b:` | ramas |
| `t:` | tags |
| `w:` | worktrees |
| `s:` | stashes |
| `c:` | commits |
| `pr:` (o `mr:`, `p:`) | pull o merge requests |
| `i:` | issues |
| `#` | pull/merge requests e issues (`#12` encuentra el número 12) |

`fix: login` es una consulta normal, no un prefijo: solo cuenta una única letra
(o `pr:` / `mr:`) seguida de dos puntos, o un `#` inicial.

## Cómo coincide

Los mismos tres niveles que la paleta de comandos: tu texto literal, luego
literal en cualquier posición, luego sus caracteres en orden pero separados. Una
coincidencia más temprana y un texto más corto ganan. Mayúsculas y minúsculas
son lo mismo.

## Pull requests e issues

Vienen de lo que GitPoulpi ya cargó para cada repositorio (las listas que hay
tras las secciones PULL REQUESTS e ISSUES de la barra lateral), guardadas en un
pequeño archivo de caché en la carpeta `.gitpoulpi/` del repositorio. La
búsqueda **nunca llama a GitHub ni a GitLab**: un repositorio cuyas solicitudes
nunca se cargaron en una ventana no tiene nada que encontrar, y lo que ves es tan
reciente como la última actualización de la barra lateral. Sin una cuenta
vinculada no hay nada que encontrar, y nada que configurar.

## Desde la línea de comandos

`gitpoulpi search <consulta> [--workspace ID] [--kind K]... [--limit N]` hace lo
mismo e imprime JSON (consulta la
[referencia de la CLI](cli-reference.md#espacios-de-trabajo-y-búsqueda)); el
servidor MCP tiene una herramienta `search`, así que un agente puede encontrar lo
que encuentras tú. `search.commit_limit` en la
[referencia de configuración](config-reference.md#search-la-búsqueda-en-el-espacio-de-trabajo)
limita cuántos commits de cada repositorio se indexan (`0` desactiva la búsqueda
de commits).

## Cambiar la tecla

`Buscar: Espacio de trabajo...` es un comando de la paleta y la tecla es la
acción `global.workspace_search`: reasígnala en Ajustes > Teclado y ratón, o en la
tabla `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
