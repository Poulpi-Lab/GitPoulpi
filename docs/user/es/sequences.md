Una **secuencia** es una lista con nombre de comandos que se ejecutan uno tras otro: por
ejemplo *Hacer stage de todos los cambios*, *IA: Generar mensaje de commit*, *Commit*, *Push*
como una sola tecla **Backup**. Cada paso es un comando de la
[paleta de comandos](keybindings.md); una secuencia es una fila propia de la paleta
(`Secuencia: <nombre>`), se puede enlazar a una tecla y se edita en
**Ajustes > Secuencias** o en `config.toml`.

## La página Secuencias

**Ajustes > Secuencias** es una [página de lista](settings.md#listas): las secuencias a
la izquierda, las filas de la seleccionada a la derecha. Cada cambio se aplica y
se escribe al instante, y la paleta y el mapa de teclas lo siguen sin reiniciar.

```
 Secuencias
+- Secuencias -----------+  Editando la secuencia 1/2
| Esta instalación        |  Nombre       Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree y agente     |  Descripción  Stage de todo, ...
| Este repositorio        |  Ámbito       Esta instalación
|   Release               |  Atajo        [ Alt+B ] [ Quitar ]
+-------------------------+  Pasos        Stage ... > IA: ... > Commit > Push
| [+ Añadir v] [Dup] [Quitar]|            [ Editar pasos (4) ] [ Ejecutar ahora ]
| [Arriba] [Abajo]        |
+-------------------------+
```

* **+ Añadir** ofrece **Secuencia vacía** y los tres modelos, **Desde un modelo >
  Copia de seguridad**, **Componer y push** y **Worktree y agente**. Un modelo es un
  punto de partida: nada está activo hasta que lo añades, y lo que añades es una
  copia que puedes editar. Los modelos se nombran en el idioma de la interfaz cuando
  se añaden.
* **Duplicar**, **Quitar** (pregunta antes), **Arriba** / **Abajo** o un arrastre reordenan
  la lista (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). El orden de
  la lista es el orden en `config.toml`.
* Una fila con un problema se marca en rojo y dice por qué (sin pasos, un paso que esta
  versión no conoce, una pausa `Nunca` a la que le falta la opción que necesita).
  Una secuencia con problemas **se puede guardar**, los pasos futuros desconocidos se conservan tal
  cual, pero su fila de la paleta queda atenuada hasta que se corrija.

### Las filas de una secuencia

| Fila | Qué hace |
|---|---|
| **Nombre** | Lo que muestra la paleta: `Secuencia: <nombre>`. |
| **Id** | El nombre corto que se usa en `config.toml`, en la línea de comandos y como tecla del atajo (`a-z`, `0-9`, `_`, `-`). Al renombrarlo, el atajo se mueve con él, en la misma escritura. |
| **Descripción** | Opcional, para ti. |
| **Ámbito** | Véase más abajo. |
| **Atajo** | Un botón que muestra la tecla. `Enter` o un clic abre la misma captura de tecla que **Teclado y ratón**: los conflictos y los avisos del terminal se muestran mientras pulsas, y una tecla que usa otra acción se rechaza igual. **Quitar** la elimina. La tecla se guarda como `[keybindings.sequence] <id>`. |
| **Pasos** | Los pasos en orden. **Editar pasos** los abre. |
| **Ejecutar ahora** | Ejecuta la secuencia en la ventana actual. |

### Ámbito

Cada secuencia tiene un ámbito, que se guarda con ella:

| Ámbito | Dónde se guarda | Quién la ve |
|---|---|---|
| **Esta instalación** (por defecto) | `[[sequences]]` en `config.toml`, con esta instalación como propietaria | Solo esta instalación de GitPoulpi. |
| **Todas las instalaciones de este equipo** | `[[sequences]]` en `config.toml`, sin propietario | Cada instalación que comparte el archivo. |
| **Solo este repositorio** | `.gitpoulpi/sequences.toml` en el repositorio actual | Quien tenga el repositorio. Solo cuenta cuando confías en ella (véase más abajo). |

Cambiar el ámbito mueve la definición: elegir **Solo este repositorio**
la escribe en el archivo del repositorio y la quita de `config.toml`, y
a la inversa. Un ámbito de repositorio necesita que haya un repositorio abierto.

### Secuencias de un repositorio y confianza

El archivo `.gitpoulpi/sequences.toml` se puede versionar, así que puede venir de
otras personas, y una secuencia ejecuta comandos en tu máquina. Por eso las secuencias de un repositorio
**no se ofrecen hasta que confías en ellas**, igual que los comandos de un archivo de flujo de
trabajo versionado.

Cuando un repositorio tiene un archivo así en el que nadie ha confiado aún, aparece un aviso sobre
el Log: *Este repositorio define 2 secuencias* con **Revisar**, **Confiar** e
**Ignorar**. Nunca bloquea nada.

- **Revisar** lista cada secuencia con cada paso, su pausa y sus opciones,
  de solo lectura. **Confiar** ahí confía en lo que se listó; **Cerrar** (o `Esc`)
  no confía en nada. Si la lista es más larga que la ventana, Confiar no se ofrece
  ahí (agranda la ventana): nunca confías en lo que no pudiste leer.
- **Confiar** (el botón, o *Secuencia: Confiar en las secuencias de este repositorio* de la
  paleta) confía en el archivo exactamente como está ahora.
- **Ignorar** oculta el aviso hasta que el archivo cambie.
- Desde el teclado: la paleta tiene *Secuencia: Revisar / Confiar en / Ignorar las secuencias de este
  repositorio*.

Lo que se registra es un resumen de los ids, los comandos, las pausas y las
opciones, guardado en el estado local del repositorio (nunca en el repositorio). Si
alguno de ellos cambia, por ejemplo tras un pull, el aviso vuelve como *Las
secuencias de este repositorio cambiaron* y salen de la paleta y de las teclas
hasta que vuelvas a confiar en ellas. Renombrar una secuencia o editar su descripción
no cambia nada. Una secuencia que escribes tú mismo en Ajustes sigue siendo de confianza cuando
el archivo ya lo era (o estaba vacío) antes de que lo editaras.

Una vez de confianza, las secuencias del repositorio aparecen como `Secuencia: <nombre>` en la
paleta igual que las tuyas y se les puede dar un atajo en Ajustes. Una secuencia con
el mismo id que una tuya es un choque: gana la tuya y la otra se indica
como duplicada.

Desde la línea de comandos: `gitpoulpi sequence list` muestra las secuencias del archivo
con `trusted` y un estado `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (y `untrust`, `--repo PATH` para otro
repositorio) es para que lo ejecute una persona después de leer `sequence show`.
`sequence run` rechaza una secuencia de repositorio sin confianza. Los agentes de IA no pueden
confiar: no hay herramienta MCP para ello.

## Pasos

**Editar pasos** sustituye la lista por los pasos de la secuencia (el título
dice **Pasos de <nombre>**) y las filas por las del paso seleccionado. **Volver**
(la cruz arriba de las filas) o `Esc` regresa a las secuencias.

* **+ Añadir** abre **Añadir un paso**: cada comando que una secuencia puede ejecutar, con su
  nombre de la paleta (los de IA con la chispa), filtrados mientras escribes con la
  coincidencia de la paleta. Un paso que esta versión no puede ejecutar aparece atenuado con el
  motivo. `Enter` o un clic lo añade al final; `Esc` o la cruz cierra
  el popup.
* **Duplicar**, **Quitar**, **Arriba** / **Abajo** y el arrastre funcionan como en las
  secuencias.
* Un comando destructivo (eliminar una rama, descartar cambios...) nunca se
  ofrece: una secuencia no puede ejecutarlo.

### Pausa

Cada paso tiene una **Pausa**:

| Pausa | Significado |
|---|---|
| **Por defecto** | El paso solo se detiene cuando necesita una respuesta, en su diálogo habitual (un nombre de rama, un mensaje de commit que sigue vacío...). |
| **Siempre** | Se detiene antes de este paso y pide continuar. |
| **Nunca** | No se detiene nunca. Las opciones del paso deben darle todo lo que necesita, así que un paso que pide un nombre necesita su opción **Nombre** rellenada. |

Un paso que siempre pregunta antes de ejecutarse (un push forzado) no ofrece
**Nunca**: la fila dice por qué. **Nunca** en el paso del mensaje de la IA se permite
como elección explícita.

### Opciones

Las filas bajo **Pausa** son las opciones del paso: un texto **Nombre** o **Mensaje**,
una **Rama**, una casilla como **Abrir en una pestaña nueva**, o un desplegable
rellenado desde tu configuración (un **Remoto**, un **Perfil de IA**, una entrada de **Abrir
en**). Los textos admiten las variables `${branch}`, `${date}`, `${time}` y
`${n}` (un número que hace único el nombre). Vaciar un texto elimina la
opción.

## Crear y editar con IA

Con la [IA activada](ai.md#borrador-de-secuencia) (Ajustes > IA), aparecen dos entradas:
**Añadir > Crear con IA...** (también la fila de la paleta `Secuencia: Crear con IA...`,
que abre Ajustes en esta página con el diálogo) y **Editar con IA...** junto
a **Ejecutar ahora** para la secuencia seleccionada. Ninguna existe mientras la IA está desactivada.

```
+- Crear una secuencia con IA ------------------------------ x -+
| Describe la secuencia que quieres                             |
| +-----------------------------------------------------------+ |
| | hacer stage de todo, escribir el mensaje de commit con    | |
| | la IA, commit y después push                              | |
| +-----------------------------------------------------------+ |
| Enter añade una línea. Ctrl+Enter envía. Esc cierra.          |
| [ ] Enviar contexto (nombres de remotos, perfiles de IA, ...) |
|    Desactivado: solo se envía tu descripción.                 |
| La IA solo propone. Nada se guarda antes de que aceptes...    |
| [ Enviar ] [ Cancelar ]                                       |
+---------------------------------------------------------------+
```

Escribes lo que quieres con tus propias palabras; **Enviar** (o `Ctrl+Enter`, `Ctrl+S`)
consulta al modelo, con el mismo perfil, el mismo indicador de trabajos y el mismo `Esc` que cualquier
otra acción de IA (y la misma pregunta «Qué se envía»). La respuesta es una
**propuesta** que lees antes de que ocurra nada:

* el nombre, el id, la descripción y los pasos con su pausa y sus opciones;
* para **Editar con IA**, un diff frente a la secuencia actual: `+` un paso añadido,
  `-` un paso eliminado, `~` un paso cuya pausa o cuyas opciones cambiaron, `~ Nombre: "a"
  -> "b"` para un campo;
* **Omitido**: un paso que el modelo propuso y no está permitido (desconocido,
  destructivo, una secuencia dentro de una secuencia, una pausa imposible), con el
  motivo; los pasos válidos se conservan;
* las **notas** del modelo y un **atajo sugerido** que solo se muestra, con
  si la tecla ya está ocupada. Nunca se asigna: usa después el botón
  **Atajo**.

```
+- Cambio propuesto ---------------------------------------- x -+
| + añadido   - eliminado   ~ cambiado                          |
| ~ Nombre: "Daily" -> "Daily 2"                                |
| Pasos (3)                                                     |
| + 1. Git: Fetch de todos los remotos                          |
|   2. Git: Stage de todos los cambios                          |
| - Git: Commit...                                              |
| ~ 3. Git: Push: valores por defecto -> Pausa: Siempre         |
| Omitido                                                       |
| paso 4 `reset_hard`: los pasos destructivos no están permitidos |
| [ Aceptar ] [ Editar antes de guardar ] [ Rechazar ] [ Volver ] |
+---------------------------------------------------------------+
```

* **Aceptar** lo escribe con un solo guardado (una secuencia nueva, o el reemplazo de la
  editada, que conserva su id, su ámbito y su tecla) y la selecciona en la lista.
* **Editar antes de guardar** hace lo mismo y abre sus pasos para que ajustes
  las filas (esta página guarda cada edición al instante, así que la secuencia ya está
  guardada).
* **Rechazar** no escribe nada. **Volver** regresa a la descripción para enviarla de nuevo.
* **Aceptar** no está disponible cuando no hay nada que aceptar: ningún paso utilizable, un
  problema que la página señalaría, o una edición que no cambia nada.

No se escribe nada antes de **Aceptar**, y desde aquí nunca se ejecuta una secuencia.
Cerrar el diálogo (la cruz, `Esc`, un clic fuera) pregunta «¿Descartar los cambios?»
mientras hay una descripción escrita o una propuesta abierta; cerrar Ajustes también lo pregunta.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<id de esta instalación>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Las claves que esta versión no conoce, y los pasos que no conoce, se conservan cuando
la página guarda.

## Ejecutar una secuencia

Una secuencia es una lista con nombre de pasos, cada uno un comando de la paleta (stage de todo, escribir el mensaje de commit con la IA, commit, push...). Ejecútala desde la paleta (`Secuencia: <nombre>`), con la tecla que le diste en Ajustes > Teclado y ratón (`[keybindings.sequence]` en `config.toml`), o desde la línea de comandos (`gitpoulpi sequence run`).

Mientras se ejecuta, una franja sobre la parte superior del Log muestra `<nombre> k/n`, el paso y lo que espera. La ejecución también aparece en el panel de Trabajos como `Secuencia <nombre> k/n: <paso>`. Solo se ejecuta una secuencia a la vez; iniciar una segunda dice cuál está en ejecución.

**Dónde se detiene para preguntar.** Cada paso tiene un ajuste `pause` (`default`, `always`, `never`):

| Paso | Por defecto | `pause = "never"` |
|---|---|---|
| Mensaje de commit de la IA | el mensaje llega al campo de mensaje; la franja dice «Revisa el mensaje y pulsa Continuar» | continúa al instante |
| Commit | un mensaje vacío siempre se detiene: el campo tiene el teclado, y tu commit (o Continuar) sigue | no disponible cuando no hay mensaje que confirmar |
| Componer commits con la IA | se abre la revisión; Aplicar continúa, cerrarla cancela | aplica la propuesta tal cual |
| Crear worktree | se abre el diálogo con el nombre rellenado | lo crea desde `options.name` sin el diálogo |
| Push | cuando `git.push_behavior = ask` se abre el selector «Push a...» | hace push a `options.remote` |
| Push forzado con lease, hacer stage de archivos que parecen secretos | siempre pregunta | no permitido |

`pause = "always"` en cualquier paso muestra «Siguiente: <paso>» con **Continuar** y **Cancelar** antes de que se ejecute. Si el push se rechaza porque la rama divergió, se abre el diálogo habitual y la franja dice «Push rechazado: elige en el diálogo»; hacer pull y luego push (o un push forzado con lease) continúa la ejecución, cerrar el diálogo la cancela.

**Continuar** es el botón de la franja, `Secuencia: Continuar` en la paleta, o la tecla enlazada a `sequence.continue`. **Cancelar** es la cruz de cierre de la franja, `Secuencia: Cancelar` (`sequence.cancel`), el panel de Trabajos, o `Esc` cuando nada más lo quiere (un diálogo abierto se cierra primero, lo que también cancela). Cancelar conserva los pasos que ya se ejecutaron y lo dice: «<nombre> cancelada tras el paso k/n (<paso>); los pasos anteriores se conservan».

**Cuando un paso falla** la ejecución se detiene ahí: la línea de estado nombra la secuencia, el paso y el error, la franja se pone roja con **Detalles** (la entrada del Log de actividad, que lista las opciones del paso) y una cruz de cierre, y los pasos anteriores se quedan hechos.

**Worktrees.** Un paso de crear worktree con `open_tab = true` mueve la ejecución a la pestaña del worktree nuevo: los pasos siguientes (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) actúan allí. Si hay otra pestaña en pantalla, la franja dice «esperando a la pestaña <nombre>» con **Ir allí**. Cerrar esa pestaña cancela la ejecución. Una ejecución no sobrevive a cerrar GitPoulpi.

### Un agente que propone una ejecución

Con *Dejar que los agentes de IA propongan cambios para que yo los confirme* activado (`[mcp]
allow_ui_propose`), un agente puede pedir a tu ventana que ejecute una secuencia
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Aparece un aviso:
*✦ Propuesto por <agente>: ejecutar la secuencia <nombre>* con **Ejecutar** y **Descartar**.
No se ejecuta nada hasta que pulsas Ejecutar (haz clic, o `Ctrl+N` y luego `Enter`);
Descartar, `Esc` (cuando nada más lo quiere) o *Agente: Descartar la vista pendiente* de la paleta lo
descarta. La ejecución sigue entonces exactamente como si la hubieras iniciado tú,
con pausas donde lo indique la secuencia. La propuesta se rechaza cuando la secuencia
no existe, no es válida o es una secuencia de repositorio en la que no has confiado.

## Deshacer una ejecución

Cada paso que cambia el repositorio (stage, commit, tag, rama, stash, worktree...) deja su **propia entrada normal de Deshacer**, etiquetada con la ejecución, así que `Ctrl+Z` sigue deshaciendo un paso cada vez y el [Historial de deshacer](undo-redo.md#el-historial-de-deshacer) muestra `⟳ <nombre>` en ellas. Para retroceder la ejecución entera existe **Deshacer toda la secuencia**: el botón del Historial de deshacer (`x`), la fila de paleta `Edición: Deshacer toda la secuencia`, o `gitpoulpi undo --sequence`. Deshace las entradas de la ejecución de la más reciente a la más antigua, cada una con sus propias comprobaciones y preguntas (un commit que ya se envió pregunta antes), se detiene en la primera negativa nombrando el paso, y deja las más antiguas en el diario. El mensaje de la IA y un push no dejan entrada: un push nunca se deshace. Solo se toca el diario del repositorio actual; los pasos hechos en un worktree se deshacen desde ese repositorio. Rehacer sigue siendo por entrada. Detalles: [Ejecuciones de secuencias](undo-redo.md#ejecuciones-de-secuencias).
