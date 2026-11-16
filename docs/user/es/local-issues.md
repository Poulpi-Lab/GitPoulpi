[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · **Español** · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Issues locales y Obsidian

Guarda tus issues como una carpeta de notas Markdown: sin cuenta, sin token, sin red. La carpeta puede estar dentro del
repositorio (`docs/issues/`, con commit junto al código) o en cualquier parte del disco, por ejemplo una subcarpeta de un
vault de [Obsidian](https://obsidian.md). GitPoulpi lee y escribe las notas. Obsidian, cualquier editor o un shell pueden editarlas
a la vez, y las listas lo siguen.

![Issues locales de Markdown en la barra lateral, con el menú de la fila de una nota: abrir la nota, cerrarla, editar sus etiquetas o su hito](../images/local-issues.png)

## Configurarlo

1. **Ajustes > Integraciones > Gestores de issues > Markdown local > Añadir perfil.** Escribe la carpeta, o pulsa
   **Examinar...** y elígela. Una carpeta relativa (`docs/issues`) se lee respecto a cada repositorio al que esté vinculada; una
   absoluta (`/home/me/Vault/Projects/Issues`) es la misma en todas partes. No hay ningún token que introducir.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. En un repositorio, abre la sección **ISSUES** de la barra lateral y elige el proveedor **Markdown local - <nombre>** (la carpeta
   se elige con él; el desplegable de repositorio lista la carpeta y sus subcarpetas directas por si quieres otra).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (la carpeta del perfil es el `--project` por defecto).

Nada se vincula solo: como todo origen de issues, se activa por repositorio.

## Comentarios en una nota

`Enter` en la fila de una nota en la sección ISSUES, o en su tarjeta, abre su conversación: el texto de la nota y luego los comentarios. Los comentarios se guardan en el propio archivo de la nota, en una sección final `## Comments`, con una entrada `### <autor> - <hora UTC>` para cada uno (uno editado añade `(edited <hora>)`). Nada por encima de ese encabezado se toca nunca, así que Obsidian y una shell leen y editan el mismo archivo. Puedes escribir, editar y eliminar los comentarios del autor local (el `local.me` del perfil, si no el usuario conectado, el nombre que usa *Asignados a mí*); los de nadie más. La caja es el editor de Markdown habitual con vista previa, y *Escribiendo como <nombre> (nota local)* nombra al autor. Nada va por la red. Una nota con un encabezado `## Comments` escrito a mano se lee como tal.

## Las notas

Una nota por issue, llamada `<id>-<slug>.md` (`12-crash-on-empty-name.md`), con un bloque de frontmatter:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- Los issues nuevos reciben `max(id) + 1`.
- GitPoulpi solo escribe las propiedades que le pertenecen (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Todas las demás propiedades, el cuerpo, los fines de línea y el orden de las claves se conservan byte por byte.** Las escrituras son
  atómicas (un archivo temporal y luego un renombrado).
- La lectura es tolerante: una nota sin `id` lo toma de su nombre de archivo, una nota sin `status` es `todo`, una nota
  sin frontmatter toma su título del primer `# heading`. Un archivo que no es un issue (`readme.md`) se ignora.
- Un estado es texto libre. **Abierto** significa «que no es el estado de hecho».

### Comentarios en una nota

Un comentario sobre una nota se escribe en la propia nota, en una última sección `## Comments` (sin red, y se lee bien en Obsidian):

```markdown
(the note's text)

## Comments

### kai - 2026-11-15T10:00:00Z
First comment, **any Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
Second one.
```

- Una entrada empieza en una línea que es exactamente `### <autor> - <hora UTC>`, con ` (edited <hora UTC>)` tras una edición; su texto llega hasta la entrada siguiente. La sección es el último encabezado `## Comments` fuera de un bloque de código y llega hasta el final del archivo, así que no escribas nada después.
- Una línea de comentario que se leería como una entrada o como el encabezado se guarda con una barra invertida delante (`\### ...`, que Markdown muestra como el mismo texto) y se lee sin ella.
- Publicar añade al final y deja todo lo de arriba byte por byte (saltos de línea incluidos); una edición o una eliminación reescribe solo la sección. Una publicación también mueve el `updated` de la nota (a una nota sin frontmatter no se le da ninguno).
- El autor es el autor local del perfil (el «yo» de los ajustes de Issues locales). Solo puedes editar o eliminar tus propios comentarios; el id de un comentario es su hora y su autor, y dos publicaciones en el mismo segundo se distinguen adelantando un segundo la segunda.
- No hay respuestas, ni comentarios internos, ni historial de eventos para una nota.

## Qué puedes hacer

| En la sección ISSUES | Teclado / paleta |
|---|---|
| Issue nuevo (el `+`, o `n`): título, descripción, estado, asignados, etiquetas, hito | `Alojamiento: Nuevo issue...` |
| Editar asignados / etiquetas / hito desde el menú de la fila. Elige entre lo que la carpeta ya usa, o **escribe una etiqueta o un login nuevos** y toma la fila `Añadir "..."` | `Alojamiento: Editar etiquetas...` y compañeras |
| **Cerrar issue** / **Reabrir issue** (fija el estado de hecho, o el primer otro estado) | `Issues locales: Cerrar issue...` / `Reabrir issue...` |
| **Abrir nota** (tu programa predeterminado para `.md`) | `Integraciones: Abrir issue...` |
| **Abrir en Obsidian** (solo cuando la carpeta está dentro de un vault, es decir, hay una carpeta `.obsidian/` por encima) | `Issues locales: Abrir en Obsidian...` |
| Filtros: Abiertos / Cerrados / Todos, «Asignados a mí» | el desplegable de filtro |

«Asignados a mí» coincide con el ajuste `me` del perfil, si no con el nombre conectado, si no con el nombre del perfil.

Las ramas y los commits funcionan como en GitHub: «Crear rama desde el issue» da `12-crash-on-empty-name`, y `#12` en un mensaje de
commit o `12-...` en un nombre de rama se reconoce.

Los cambios hechos fuera de GitPoulpi (Obsidian, un shell, un `git pull`) aparecen en cosa de un segundo: los nombres de archivo de la carpeta
y sus fechas de modificación se miran en un hilo en segundo plano, nunca en el hilo que dibuja.

## Archivo de configuración

En `config.toml`, junto al perfil (todas las claves son opcionales):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # el orden de las columnas del tablero
done_status = "done"                            # cierra un issue
reference_prefix = "LOC"                         # se muestra como LOC-12 en lugar de #12
me = "kai"                                       # qué significa «asignados a mí»
```

Los estados que usan las notas pero no figuran en `statuses` van detrás de los listados. `statuses` es el orden de las columnas del
[tablero](#el-tablero-kanban) (`gitpoulpi hosting issue options` lo imprime).

## El tablero Kanban

![El tablero de una carpeta de notas: una columna por estado, las tarjetas con sus asignados, etiquetas e hito](../images/issue-board.png)

Si trabajas solo, o en un equipo que guarda sus issues junto al código, la carpeta es también un tablero Kanban.
**Encabezado ISSUES > Tablero**, o **Issues: Abrir tablero** de la paleta, lo abre sobre la ventana; `Esc` lo cierra.

- Las **columnas** son los estados: primero los listados en `statuses`, en ese orden, y después cualquier otro estado que use una nota.
  Una columna vacía se muestra igualmente. Cada encabezado lleva su recuento de tarjetas.
- Las **tarjetas** muestran el número del issue, el título (dos líneas), los asignados como iniciales, las etiquetas y el hito.
  Cuando no caben todas las columnas, el tablero se desplaza lateralmente (la barra bajo él, la rueda con `Shift`, o las flechas).
- **Mover una tarjeta**: arrástrala a otra columna (la columna se ilumina bajo el puntero), con
  `Shift+Left` / `Shift+Right`, o con **Tablero: Mover tarjeta a...** (`m`), un desplegable con las columnas. La tarjeta se mueve
  al instante; el `status` de la nota se reescribe como en [la sección de las notas](#las-notas), sin tocar nada más del archivo. Si la escritura
  falla, la tarjeta vuelve y el error aparece en la barra de estado.
- **Abre** una tarjeta con `Enter` o un doble clic: la [conversación](hosting-integration.md#5c-conversaciones) de la nota se abre en la ventana, con el texto de la nota y sus comentarios (el **Abrir nota** del menú de la tarjeta abre el archivo en tu programa predeterminado para `.md`). `e` edita sus
  asignados, etiquetas o hito, `.` o un clic derecho abre el menú de la tarjeta, `n` inicia un issue nuevo en la columna
  seleccionada (el formulario vuelve al tablero al cerrarse).
- **Filtra** por asignado (`a`), etiqueta (`l`), hito (`M`) o texto (`/`); `x` los quita. Los filtros son
  desplegables: escribe para acotar, `Enter` elige.
- **En vivo**: una nota cambiada desde Obsidian o un shell mueve su tarjeta en menos de un segundo.

Cada gesto del ratón tiene una tecla y un comando de la paleta (**Tablero: ...**, listados en la página de
[atajos de teclado](keybindings.md)).

## Bases de Obsidian

**Paleta: `Issues locales: Crear base de Obsidian`** (CLI: `gitpoulpi hosting issue base`) escribe `issues.base` junto a las
notas, una sola vez: una vista de *tabla* y una vista *kanban* agrupada por `status`, en el orden de `statuses`. Ábrelo en Obsidian para
ver los mismos issues como un tablero allí. GitPoulpi nunca edita un archivo `.base` existente (tuyo o de una vez anterior): se
niega y lo dice. El formato sigue la sintaxis de Bases de Obsidian
(<https://obsidian.md/help/bases/syntax>); la vista kanban necesita un Obsidian reciente.

## CLI y MCP

Funcionan sobre él los mismos comandos que en cualquier origen de issues: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Cuatro más son para notas:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` y `hosting issue base`.
Ver la [referencia de la CLI](cli-reference.md).

## Límites

- Un **hito** nuevo no se puede escribir en el selector (elige uno que la carpeta ya use), pero `milestone:` en una nota puede
  ser cualquier cosa y aparece en cuanto se guarda.
- Solo las notas que están directamente en la carpeta son issues (no las de subcarpetas).
- No hay eliminar: borra la nota en tu gestor de archivos o en Obsidian (su papelera es tuya).
