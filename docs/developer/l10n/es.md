# Spanish (es) localisation glossary

Binding for every change to `crates/gitpoulpi-i18n/locales/es.toml`. Sources: Microsoft Spanish terminology, git's own `po/es` (git-l10n), GitHub Desktop / VS Code / JetBrains / TortoiseGit Spanish UIs. Two vocabularies: **git** (default: the English git word, as Spanish developers say it) and **local** ("Translate git terms": proper Spanish). A message holding a git word is either a `{$term}` token or a `{ git = .., local = .. }` pair. The local text must not contain the English stem (lint `tests/git_vocabulary.rs`).

| Concept | git vocabulary | local vocabulary | Notes |
|---|---|---|---|
| commit (n.) | commit (m.) | confirmación (f.) | verb local: confirmar; git: "hacer commit" |
| push / pull / fetch | push / pull / fetch | enviar / traer / obtener | noun local: envío (m.), traída (f.), obtención (f.) |
| merge | merge | fusionar / fusión (f.) | "ninguna fusión", "una fusión" |
| rebase | rebase | reaplicar / reaplicado (m.) | NOT "rebasar": the lint flags the stem `rebas` |
| cherry-pick, revert, squash, fixup, reword, drop | same | cherry-pick (kept), reversión / revertir, reformular, absorber, descartar | `[term]` table |
| stash | stash (m.) | guardado rápido (m.); verb "guardar aparte" | auto-stash: "guardado rápido automático" |
| tag | tag (m.) | etiqueta (f.) | issue labels are also "etiqueta" (different context) |
| diff | diff (m.) | diferencias (f. pl.) | "las diferencias son", "vista de diferencias" |
| stage / unstage / staged / unstaged | stage, unstage, staged, unstaged | preparar, quitar de la preparación, preparado, sin preparar | Status halves: "Sin preparar" / "Preparados" |
| pull request / merge request | pull request / merge request (f.) | solicitud de incorporación / solicitud de fusión (PR/MR -> SI/SF) | host picks pull or merge |
| HEAD detached | HEAD desacoplado | same | |
| working tree / index | árbol de trabajo / índice | same | "directorio de trabajo" only for EN "working directory" |
| worktree | worktree | árbol de trabajo | submodule: submódulo |
| branch, remote, upstream, checkout | rama, remoto, upstream, checkout | same (checkout local: "cambiar a" / "cambiar de rama") | |
| blame, hunk, ref, SHA, LFS, WIP, Log | kept English | same | "hunk" everywhere (never "bloque"; "bloque" = merge-editor chunk) |
| conflict card / info card / LFS card | tarjeta | same | never "ficha" |
| ref pill / badge | chip de ref | same | never "etiqueta" (clashes with tag) |
| settings, palette | Ajustes, paleta de comandos | same | never "Preferencias" (the palette entry is "App: Ajustes..."). Key names stay English (Up/Down, Shift+Up/Down, Enter, Esc). One word per concept: "fin de línea" / "fines de línea" (never "salto de línea"), "preajuste" for a preset (never "ajuste predefinido") |
| activity log | Registro de actividad | same | |
| terminal | el terminal (m.) | same | |
| shell | el shell (m.) | same | "un shell", "tu shell"; "wrapper de shell" (never "la shell") |
| chord | acorde (m.) | same | `Ctrl+G` then a key; not "secuencia" |
| badge (the `Ctrl+N` letter on a pane) | insignia (f.) | same | not to be confused with the ref chip |
| info card | tarjeta de información | same | never "ficha" |
| scopes (token permissions) | ámbitos | same | as in Settings > Integrations |
| line ending | fin de línea (Settings, "Fin de línea"); "fines de línea" in the Convert menu; prose "finales de línea" / "saltos de línea" | same | the catalog itself mixes these three |
| undo safety net | Historial de deshacer, Deshacer hasta aquí, Restaurar solo esta, Rehacer hasta aquí, Trabajo perdido | same | "Reflog" stays English (as "blame", "hunk"); an action is feminine ("bloqueada", "deshecha", "restaurable"); "pin de deshacer" for the undo pin; the restore branch is `restore/<rama>-<sha>` |
| "check out" (English verb, UI text) | Checkout de .. | cambiar a .. | "Check out source branch" -> "Checkout de la rama de origen" / "Cambiar a la rama de origen" |
| workspace | espacio de trabajo (m.) | same | never "área de trabajo"; chip = "chip", never "etiqueta"; "No workspace" = "Sin espacio de trabajo"; missing repo = "ausente" |
| pin / snooze (Radar) | fijar / posponer ("Fijados", "Pospuestos", "Dejar de fijar") | same | noun "fijación" / "posposición" in prose |
| issue (Radar, search, sidebar) | issue (m.) | same | never "incidencia"; scope = "Ámbito" (Radar), "Radar" itself stays English |
| palette banner prefix | `Aviso:` | same | never "Banner:" in a palette entry; "banner" alone is fine in prose |
| code review (the feature) | revisión (de código); palette category "Revisión:"; verb "Revisar..." | same | the in-app review view; "request" (generic PR/MR/Gerrit change) = "solicitud" |
| pending comment / pending review | comentario pendiente / revisión pendiente | same | kept on the machine until "Terminar revisión" |
| thread | hilo | same | "Responder al hilo", "plegar / desplegar el hilo"; badges "resuelto" / "obsoleto" |
| resolve / reopen | resolver / reabrir | same | threads and issues alike ("Cerrar issue" / "Reabrir issue") |
| approve / request changes / comment (verdict) | Aprobar / Solicitar cambios / Comentar; the choice is the "Veredicto" | same | badges on a submitted review: "aprobada", "con cambios solicitados", "comentada", "descartada" |
| submit / finish review | "Terminar revisión" (button, opens the form), "Enviar revisión" (sends it), "Descartar revisión" | same | |
| suggestion (AI review) | sugerencia; Aceptar / Editar / Descartar; severities información / aviso / problema | same | the star glyph stays in front: "Revisión: ✦ Sugerir revisión" |
| patch set (Gerrit) | patch set (m., kept English like hunk/blame) | same | "Elegir patch set..."; Gerrit labels (`Code-Review`) and `refs/for/` stay as is |
| vote (Gerrit) | voto | same | |
| board / column / card | tablero / columna / tarjeta | same | column names "Abiertos" / "Cerrados"; palette category "Tablero:"; "Issues: Abrir tablero" |
| milestone | hito | same | "Establecer hito...", "Sin hito" |
| assignee / reviewer | asignado (pl. asignados) / revisor (pl. revisores) | same | "Editar asignados...", "Editar revisores..." |
| label (issue / PR) | etiqueta | same | same word as the local "tag"; context separates them |
| encoding (file text) | codificación | same | "Detección automática", "Elegir codificación..." |
| checks (CI) | comprobaciones ("superadas", "fallidas", "en curso") | same | the doc may keep "checks" in prose |
| mentioning me (Radar) | Me mencionan | same | |
| notes folder / vault | carpeta de notas / vault de Obsidian ("vault" kept) | same | "Issues locales:" palette category; "base de Obsidian" |
| workspace AI instructions | Instrucciones de IA (del espacio de trabajo) | same | levels: globales, espacio de trabajo, repositorio, acción; "Añadir a las globales" / "Reemplazar las globales" |
| hosting palette category | Alojamiento | same | never "Hosting:" in a palette entry |
| workflow (branching) | flujo de trabajo (de ramas) | same | never the bare English "workflow" in UI, docs prose or CLI help (code, file names and keys like `workflow.toml` stay); `Flow` stays English for the toolbar button, the `Flow:` palette category and the menu; plural "flujos de trabajo" |
| base branch / topic branch / topic type | rama base / rama temática / tipo de rama | same | badge `[base, protegida]`; folder `Otras` for unclassified; template names (`Git Flow`, `GitHub Flow`, `GitLab Flow`, `Trunk-based`, `Custom`) stay |
| parent (branch) | padre (m.) | same | "Padre", "Establecer padre...", "al día con su padre"; "rama padre" only in a full sentence; never "superior" |
| environment branch | rama de entorno | same | GitLab Flow |
| start / finish / sync / promote / set parent | Empezar / Terminar / Sincronizar / Promover / Establecer padre | same | "Terminar" is the workflow action (never "Finalizar"); a stopped action: Continuar / Abortar |
| hook | hook (m., kept English like git's own hooks) | same | "Hooks", "Ejecutar los hooks"; "comandos de shell" |
| pick up (a host-made branch) | recoger | same | "Recoger una rama", "Recoger rama del host...", button "Recoger"; never "adoptar" |
| issue flow (host-driven) | flujo de issues (dirigido por el host) | same | "issue" stays (see above) |
| tracker (issue) | gestor (de issues) | same | "clave del gestor" for the `{issue_key}` token; never "tracker" alone in UI |
| merge strategy / merge commit | estrategia de merge / commit de merge (git) ; estrategia de fusión / confirmación de fusión (local) | same | written as pairs, never "{$merge}" inside a noun phrase ("hacer {$merge}" breaks in local mode) |
| submit / abandon (Gerrit change) | integrar / abandonar | same | NOT "enviar" (clashes with push in local mode and with "Enviar revisión"); "Revisión: Integrar el cambio..." |
| gitGraph detail levels (Mermaid popup) | Estructura / Refs / Asuntos / Completo | same | stored words `structure`, `refs`, `subjects`, `full` stay Latin in config and CLI; "Dirección" (never "orientación"); "De izquierda a derecha" / "De arriba abajo"; "subject" of a commit = "asunto" (as git's po/es) |
| preview | vista previa | same | the Flow dialogs' "Preview" button and the Mermaid box |
| token (name-pattern placeholder) | token / marcador | same | "token" in field labels and help (`Tokens:`), "marcador" in doc prose; `slug` kept as `Slug` |

## Style

- Address: informal "tú" everywhere (imperatives "elige", "pulsa", "haz clic"). Never "usted" forms.
- Capitalisation: sentence case for labels, buttons, titles, headings; English Title Case is not copied. Palette and keymap entries are `Category: Action` with a capital after the colon; status and error sentences stay lower case after a colon.
- Actions: infinitive for buttons, menu rows and palette commands ("Guardar", "Crear rama..."); gerund for progress ("Guardando..."); participle for results ("{path} guardado").
- Punctuation: no space before `: ; ? !`; `¿ ¡` opening marks; guillemets «...» for quoted names; plain ASCII apostrophe; no em dash (use ` - `).
- Keys: `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Space` stay English (never "Mayús", "Intro").
- Dates: "5 de marzo de 2024" (`[date].day_month_year`), months and weekdays lower case, "hace 3 días", "ahora mismo".
- "Default": "predeterminado" before programs, apps and editors, "por defecto" for setting values and adverbially.
- Placeholders, plural shapes (`one` iff n = 1) and `{$term}` tokens are never altered.

## Known residue (lint baseline `tests/git_vocabulary/es.toml`)

Inherent, cannot be written without the English stem: placeholder or key names that are code (`{commit}`, `{tag}`, `{stage}`, `{{merged}}`, `{{diff.edit}}`, `git.pull_mode`, `general.auto_stash`) and the English prompt chips sent to the model (`ai_page.chip_*_text`). The baseline may only shrink.

- Radar actions are bare nouns ("Abrir repositorio", "Cambiar a rama", "Crear worktree...", "Mostrar en repositorio", "Empezar a trabajar en issue..."); status lines impersonal and lower case ("posposición de {item} cancelada", "posiciones cargadas: {n}").
