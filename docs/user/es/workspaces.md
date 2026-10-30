[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · **Español** · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Espacios de trabajo

Un **espacio de trabajo** es un grupo con nombre de repositorios: los que usas
juntos (un front end, su API, la documentación). Recuerda cuáles están abiertos
como pestañas y puede llevar un **perfil de commit predeterminado**, de modo que
los commits de esos repositorios usen la identidad correcta sin que cambies de
perfil a mano. Un repositorio puede estar en varios espacios de trabajo, y no
estar en ninguno también está bien: "Sin espacio de trabajo" es la lista de
pestañas de siempre.

Los espacios de trabajo se guardan en `workspaces.toml`, junto a `config.toml`,
y los comparten todas las sesiones, la ventana, la CLI y MCP
(`gitpoulpi workspace ...`, consulta la [referencia de la CLI](cli-reference.md)).
Un espacio de trabajo añadido, renombrado o eliminado desde un shell o con `gitpoulpi workspace ...` mientras la ventana está
abierta aparece solo en cosa de un segundo (la pestaña Espacios de trabajo, el chip de la barra de pestañas y su lista), sin
pulsar ninguna tecla. Cerrar o eliminar un espacio de trabajo nunca toca un repositorio del disco.
Editar un espacio de trabajo es un ajuste, como los favoritos y los perfiles: no
forma parte de [Deshacer y rehacer](undo-redo.md), y Eliminar pregunta antes.

## El chip del espacio de trabajo

En cuanto tienes un espacio de trabajo, aparece un chip en la barra de pestañas
justo después de `⌂`: un punto de color y el nombre del espacio de trabajo
activo, o "Sin espacio de trabajo" (atenuado). En una barra estrecha se reduce al
punto, y en una muy estrecha desaparece.

![El desplegable de espacios de trabajo sobre la barra de pestañas, con Sin espacio de trabajo, Orbit (marcado), Side projects, Archive y Gestionar espacios de trabajo, con el chip Orbit y su perfil "Work - del espacio de trabajo Orbit" en la barra de herramientas](../images/workspace-switch.png)

- **Haz clic en él**, pulsa `Alt+W` o ejecuta `Espacio de trabajo: Cambiar...`
  desde la paleta para abrir un desplegable: tus espacios de trabajo, "Sin
  espacio de trabajo" y "Gestionar espacios de trabajo..." (que abre la pestaña Espacios de trabajo).
- **Cambiar intercambia las pestañas.** Las pestañas que tenías abiertas se
  guardan para el espacio de trabajo que dejas, y vuelven las del destino (la
  primera vez, sus repositorios que existen). Vuelve atrás y todo está donde lo
  dejaste. Reinicia la ventana como un cambio de pestaña; el daemon sigue.
- **El espacio de trabajo se recuerda**: el próximo arranque vuelve a él.
- **Cerrar la última pestaña** de un espacio de trabajo mantiene el espacio de
  trabajo activo y muestra la pestaña Inicio.
- **El `+` (nueva pestaña) con un espacio de trabajo activo** añade el
  repositorio que elijas a la lista de pestañas y no pregunta nada. Para ponerlo
  en el propio espacio de trabajo, usa "Añadir al espacio de trabajo" en el menú
  contextual de la pestaña (o `Espacio de trabajo: Añadir la pestaña al espacio
  de trabajo`); "Quitar del espacio de trabajo" lo saca de nuevo y deja la
  pestaña abierta.

Sin ningún espacio de trabajo todavía no hay chip: crea uno primero desde la
pestaña Espacios de trabajo o la paleta.

## La pestaña Espacios de trabajo

Ábrela desde el menú del `+`, con `Alt+J` o con `Espacio de trabajo: Gestionar...` en la paleta; su `x` (o `Ctrl+W`) la cierra, y sigue abierta entre sesiones hasta que la cierres. Cada
espacio de trabajo es una fila con su punto de color, su nombre, cuántos
repositorios lista (y cuántos faltan), su perfil predeterminado y "activo" cuando
la ventana está en él. `Right`, `Space` o un clic lo despliegan hasta sus
repositorios.

![La pestaña Espacios de trabajo: Orbit desplegado con sus dos repositorios, Side projects y Archive marcado con "1 ausente"](../images/workspaces.png)

| Por espacio de trabajo | Tecla | |
|---|---|---|
| Abrir | `Enter` | Lo activa y abre sus pestañas. |
| Editar... | `e` | El formulario de abajo. |
| Eliminar... | `x` | Pregunta antes. Los repositorios no se tocan. |
| Fetch todo | `Shift+F` | Un repositorio tras otro, progreso en la barra de estado, los fallos se listan al final. |
| Pull todo | `Shift+P` | Igual, solo fast-forward: nunca se detiene en un merge. |
| Clonar los ausentes | `m` | Clona cada repositorio ausente que tenga un remoto registrado. |
| Nuevo espacio de trabajo... | `n` | |
| Desde pestañas abiertas... | `a` | Un espacio de trabajo nuevo con tus pestañas abiertas marcadas. |

Un repositorio **ausente** (su carpeta ya no existe) sigue en la lista, marcado,
con **Localizar...** (`l` o `Enter`: elige dónde está ahora), **Clonar** (cuando
su URL de `origin` se registró al añadirlo) y **Quitar** (`x`). Cada fila tiene
además un menú `⋮` (clic derecho, `.` o `Shift+F10`) con las mismas acciones, y
todas las acciones están en la paleta bajo `Espacio de trabajo:`.

## El formulario Nuevo / Editar

![El formulario Editar espacio de trabajo: nombre, color, descripción, perfil predeterminado y la lista de repositorios](../images/workspace-edit.png)

- **Nombre**: obligatorio, único.
- **Color**: un desplegable de colores del tema; el punto del chip y de las filas.
- **Descripción**: texto libre.
- **Perfil**: un desplegable con tus `[[profiles]]`, o "Ninguno". Es opcional.
- **✦ Instrucciones de IA**: solo con la IA activada, ver [más abajo](#instrucciones-de-ia-de-un-espacio-de-trabajo).
- **Repositorios**: una lista de todos los repositorios que GitPoulpi conoce
  (pestañas abiertas, favoritos, recientes, los del propio espacio de trabajo,
  incluidos los ausentes). `Space` marca una fila; **Añadir carpeta...**
  (`Ctrl+B`) elige un repositorio del disco. La URL de origin de cada repositorio
  nuevo se lee del repositorio y se registra (sin ninguna credencial); es lo que
  Clonar usa después.

## Instrucciones de IA de un espacio de trabajo

Con la IA activada (*Ajustes > IA*), el formulario tiene una fila **✦ Instrucciones de IA** con un botón **Editar...** (`Enter` en
la fila). Abre un pequeño editor de lo que se le dice a la IA en **todos los repositorios del espacio de trabajo**: elige *Todas las
acciones de IA* o una acción (mensaje de commit, explicar, mensaje de stash, pull request, compositor de commits, resolución de
conflictos, revisión de código) en la lista **Para**, elige **Combinar** - *Añadir a las globales* (se envía después de las instrucciones
globales) o *Reemplazar las globales* (se envía en lugar de ellas) - y escribe el texto. **Aplicar** conserva todo el conjunto, **Guardar** en
el formulario lo escribe en `workspaces.toml` junto con el espacio de trabajo; **Cancelar** descarta lo que escribiste. Con la IA
desactivada la fila se oculta y las instrucciones del espacio de trabajo se dejan como están.

Las palabras del espacio de trabajo van entre las instrucciones globales y las propias del repositorio (ver
[Instrucciones de IA](ai.md#instrucciones) para el orden y para lo que hace *reemplazar*). Un repositorio que pertenece a
varios espacios de trabajo usa el espacio de trabajo activo de la ventana cuando lo contiene; si no, el primero que lo contenga. Desde un
script: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` las imprime como `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # todas las acciones de IA de este espacio de trabajo
text = "Use the Acme glossary."
mode = "add"                       # o "replace"

[workspace.ai_overrides.code_review]   # una acción
text = "Flag missing tests."
mode = "add"
```

## El perfil predeterminado

Los commits los firma el primero de estos que se aplique:

1. el perfil que elegiste **a mano** para ese repositorio (el campo de perfil de
   la barra de herramientas),
2. el perfil del **espacio de trabajo activo**, cuando el repositorio es uno de
   sus miembros,
3. `general.active_profile`.

El campo de perfil de la barra de herramientas indica de dónde viene: "Work - del
espacio de trabajo Acme". El campo también se muestra cuando tienes un único
perfil, siempre que el predeterminado de un espacio de trabajo esté en vigor. Un
espacio de trabajo cuyo perfil ya no existe simplemente deja de aplicarse. En la
CLI, `commit --workspace <id>` (o `GITPOULPI_WORKSPACE`) se resuelve igual.

## Teclas

| Tecla | Acción |
|---|---|
| `Alt+W` | Abrir el desplegable de espacios de trabajo |
| `Alt+J` | Abrir la pestaña Espacios de trabajo |
| `n`, `a` | Nuevo espacio de trabajo, nuevo desde las pestañas abiertas |
| `e` | Editar el espacio de trabajo |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch todo, Pull todo, Clonar los ausentes, Localizar |

Todas se pueden reasignar en Ajustes > Teclado y ratón; la lista completa está en
[Atajos de teclado](keybindings.md).
