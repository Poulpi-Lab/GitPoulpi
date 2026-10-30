# pt-PT terminology and style (crates/gitpoulpi-i18n/locales/pt-PT.toml)

European Portuguese, not Brazilian: it differs from `pt-BR.md` wherever usage differs (see "Not Brazilian" below). Sources: git's own pt_PT translation (git-l10n), the Microsoft pt-PT terminology and style guide, and how GitHub Desktop, VS Code, Visual Studio, TortoiseGit, and SourceTree word the same things. Follow this table for any new key.

## Two vocabularies

Default ("git" variant): what Portuguese developers actually say, so git words stay English. "Translate git terms" on ("local" variant): the Portuguese word. Every git word with a different local form is a `{$term}` token. Where the grammar changes with the term (gender, article, noun vs verb) write a pair `{ git = "..", local = ".." }`.

Token shape matters: `push`, `pull`, `fetch` are **infinitive verbs** locally (`enviar`, `puxar`, `obter`), so a button, menu row or palette action reads naturally ("Enviar", "Obter tudo"). Wherever a noun is needed (after an article, "Falha no ...", "... em curso", "URL de ...") write a pair with the noun: envio (m.), puxada (f.), obtenção (f.). `merge` is a **noun** locally (`integração`, f.), the verb is written inline ("a integrar", "integrar"), and articles before it need a pair ("o merge" / "a integração").

| Concept | git variant | local variant | Notes |
|---|---|---|---|
| commit (n./v.) | commit | commit | kept in both (git-l10n, Microsoft); "fazer commit", "não commitado" |
| push | push | enviar (noun: envio) | force push = "envio forçado" |
| pull | pull | puxar (noun: puxada) | |
| fetch | fetch | obter (noun: obtenção) | "Fetch All" = "Obter tudo", "URL de obtenção" |
| merge | merge (m.) | integração (f.) | git-l10n; "editor de integração", "commit de integração"; palette category "Integração:" |
| rebase, cherry-pick, stash, fast-forward, checkout, upstream, hunk, blame, amend, revert, worktree, HEAD, SHA, LFS | same | same | kept in both; reset local arm = "repor" (soft/mixed/hard = suave/misto/forte) |
| tag | tag (f.) | etiqueta | git-l10n |
| diff | diff (m.) | diferença (f.) | gender flips, so "o diff" / "a diferença" needs a pair; external tool = "ferramenta de comparação" |
| stage / unstage / staged / unstaged | same | preparar / remover da preparação / preparado / não preparado | git-l10n "área de preparação" |
| pull request / merge request | pull request / merge request | pedido de incorporação / pedido de integração | abbreviations PR / MR vs PIc / PIg; sidebar title local = "PEDIDOS" (the 18-cell budget) |
| rebase todo actions pick / reword / squash / fixup / drop / edit | same | escolher / reformular / combinar / absorver / descartar / editar | |
| branch (f.) | branch | branch | never translated in either variant, never "ramo" (git-l10n says "ramo", developers and the maintainer's tools do not) |
| repository, remote | repositório, remoto | same | never English; "o remoto" |
| conflict, resolve | conflito, resolver | same | merge editor panes: Atual (A), Recebido (B), Base, Resultado |
| working tree | árvore de trabalho | same | git-l10n; a linked `worktree` keeps the English word |
| settings | Definições | same | English says "Settings" everywhere; never "Preferências" |
| palette, log, graph, minimap, avatar, issue | paleta de comandos, Log (pane) / registo (application log), grafo, minimapa, avatar, issue | same | "Registo de atividade" is the application log, "Log" the commit pane |
| lane / banner | faixa / aviso | same | a banner (merge in progress, auto-stash) is "aviso", never "faixa" |
| undo / redo / reset | Desfazer / Refazer / Repor | same | never "Anular", never "Redefinir" (Microsoft pt-PT: "Repor"); in prose too: "o desfazer", "desfaz", "diário do desfazer" |
| download / transfer | transferir, transferência | same | never "baixar" or "download"; "descarregar" is avoided so one word is used |
| restore (undo history) / reflog / lost work | restaurar / Reflog / trabalho perdido | same | "Restaurar só esta" (restore only this), "Desfazer até aqui", "Refazer até aqui"; the history is "histórico de desfazer", an undone action is "desfeita" (never "anulada"); `Reflog` stays English; the reset modes in the restore dialog are pairs: git "Soft / Mixed / Hard", local "Suave / Misto / Forte" |
| scroll | deslocar, barra de deslocação | same | never "deslocamento" |
| e-mail | e-mail | same | with the hyphen, never "email" |

## Style

- Address: neutral, no pronoun, or formal imperative ("selecione antes um ficheiro", "prima Enter"); infinitive in buttons, menus and palette actions ("Guardar", "Criar branch"); "A + infinitive" in progress messages ("A abrir...", "a criar branch...", never the Brazilian gerund); noun phrase for states ("{$Merge} em curso").
- Capitalisation: sentence case (Microsoft pt-PT); sidebar section titles in capitals like English. Palette: `Categoria: ação`, translated English categories (Remoto, Repositório, Separador, Mostrar, Ficheiros, Alterações, Integração, Aviso).
- Punctuation: no space before `:` `;` `?` `!`; three ASCII dots `...` everywhere (English dropped `…` in 2026-10: dialog openers, progress texts, placeholders, prose), never `…`; a command that opens a dialog or asks first ends in `...` like the English row; status-line and in-flight messages start lower case like English ("a guardar {path}...", "não foi possível abrir {path}: ..."); UI names in « » or " " (code, shell and config examples keep `"`); ASCII `'` only (catalog test); no em dash, " - " instead.
- Mouse words: "clique", "clique duplo" (not "duplo clique"), "clique com o botão direito", "arrastar e largar", "passar o rato". Keys stay `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Backspace`, `Delete`; "prima" for pressing a key.
- Dates: `5 de março de 2024` (`msg::date`), months and weekdays lower case (`março`, `terça-feira`, abbreviations `mar`, `ter`), relative `há 5 minutos`. Plural: `one` iff n = 1 (CLDR pt-PT, unlike pt-BR).
- Spelling: Acordo Ortográfico 1990 as written in Portugal ("ação", "carateres", "Aspeto", "para" without accent).
- Width: toolbar and button labels as short as the English ones; abbreviations in tight spots (`Config.`, `PIc`/`PIg`, `EB`, `{n}a`).

## Not Brazilian

| Brazilian | European |
|---|---|
| arquivo, tela, mouse, salvar, excluir, usuário, senha | ficheiro, ecrã, rato, guardar, eliminar, utilizador, palavra-passe |
| configurações, padrão | definições, predefinição / predefinido |
| mesclagem, solicitação de mesclagem | integração, pedido de integração |
| rolar / barra de rolagem, borda, cinza | deslocar / barra de deslocação, margem, cinzento |
| redefinir, desfazer (reset) | repor |
| todo painel, toda ação | cada painel, cada ação |
| baixar, download | transferir, transferência |
| clique direito, duplo clique | clique com o botão direito, clique duplo |
| ausente, teto, refatoração | em falta, limite, refatorização |
| até o / até a | até ao / até à |
| gerund ("criando") | "a criar" |

## Lint limits

The local-mode lint baseline `tests/git_vocabulary/pt-PT.toml` keeps only what cannot be written without the English stem: placeholders and identifiers (`{tag}`, `{stage}`, `{{merged}}`, `{{diff.edit}}`, `{{diff.close}}`, `{{diff.merge_editor}}`, `git.pull_mode`, `git.push_behavior`, `diff.tool`, `merge.tool`) and the setting help that names the English words on purpose (`settings.translate_git_terms_help`). The baseline may only shrink.

## Undo history labels and outside changes (2026-10)

| English | pt-PT | Notes |
|---|---|---|
| outside (change made outside GitPoulpi) | externa / "feita fora do GitPoulpi" | tag is feminine ("externa") because it qualifies "ação"; status line "alteração externa registada" |
| mainline (revert of a merge) | linha principal | "pai {n}" for the parent number; the title uses the pair "commit de merge" / "commit de integração" |
| tip of a branch | topo | "antigo topo de {what}" |
| `[undo_label]` verbs | infinitive, same words as the menus | Commit, Checkout de, Amend de, Descartar, Eliminar, Reverter, Fast-forward de, Compor, `{$Merge} de` (noun in the local variant), `{$Cherry_pick} de {n} commits`; reset = pair "Reset"/"Repor" with soft/mixed/hard = suave/misto/forte |

## Workspaces, Radar, search (2026-10 review)

| English | pt-PT | Notes |
|---|---|---|
| workspace | área de trabalho | VS Code pt-PT; never "espaço de trabalho" (the search popup and palette used it by mistake) |
| Radar | Radar | feature name, kept |
| pin / unpin | afixar / desafixar | "Afixar no topo" |
| snooze / unsnooze | adiar / deixar de adiar | "Adiados" group; message "adiamento removido de {item}" |
| scope / view / show (combobox labels) | Âmbito / Vista / Mostrar | |
| work in progress | trabalho em curso | view chip stays "WIP" |
| New tab | Novo separador | |
| Load more | Carregar mais | status "posições carregadas: {n}" |
| Radar actions | bare, like English | "Abrir repositório", "Fazer checkout da branch", "Criar worktree...", "Mostrar no repositório", "Iniciar trabalho na issue..." |
| palette rows | lower case after the colon | "Radar: adiar...", "Área de trabalho: nova..."; git-term tokens lower case there (`{$fetch}`) |
| branch in running text | "branch" (f.), never "ramo" | |

## Review, metadata, boards, local issues (2026-10, round 4)

| English | pt-PT | Notes |
|---|---|---|
| review (the view, the act) | revisão / "rever" (verb) | "Rever pedido...", "Revisão: ..." palette category; never "análise" |
| request (PR / MR / Gerrit change, in review text) | pedido | the neutral word, so Gerrit changes fit |
| thread (comment thread) | discussão | GitLab pt-PT word; "conversa" is the Conversation heading (comments outside code), so the two never collide |
| pending comment / pending review | comentário pendente / revisão pendente | kept on the machine until the review is finished |
| Finish review / Submit review / Discard review | Concluir revisão / Submeter revisão / Descartar revisão | "enviar" is the local push verb, so "submeter" avoids a clash in the local vocabulary |
| verdict | parecer | Approve = Aprovar, Request changes = Pedir alterações, Comment = Comentar; badges "aprovou", "pediu alterações", "comentou", "dispensada" |
| resolve / reopen | resolver / reabrir | "resolvida", "desatualizada" (outdated) |
| viewed (file mark) | visto | "vistos" in counts |
| suggestion (AI) / Accept / Dismiss | sugestão / Aceitar / Dispensar | severities info / aviso / problema |
| patch set, vote, head ref | patch set, voto, head ref | Gerrit words kept English (patch set masculine); Code-Review and Verified are label names, never translated |
| assignee / reviewer / label / milestone | responsável / revisor / etiqueta / marco | "etiqueta" is also the local word for tag, but the two never share a screen; button chips "Responsáveis", "Revisores", "Etiquetas", "Marco" |
| encoding / Auto-detect | codificação / Deteção automática | button value stays "Auto" |
| board / column / card | quadro / coluna / cartão | Trello pt-PT words; palette category "Quadro:" ; "Issues: abrir quadro" |
| notes folder / vault | pasta de notas / cofre (Obsidian vault) | "pasta", never "diretoria" in the UI |
| local issues | issues locais | "Markdown local" for the provider |
| status (issue state) | estado | "Estado" field of a local issue and of Linear / YouTrack |
| Mentioning me | Mencionam-me | Radar group and filter |
| hosting (palette category) | Alojamento | "Alojamento: editar responsáveis..." |
| host (the server) | host | kept, as in the rest of the catalog; never "anfitrião" |
| Keycaps in labels | Latin: Space, Shift+Left/Right, Esc, Enter | per the style guide |

## Branching workflows, Mermaid, Gerrit (2026-10, round 5)

| English | pt-PT | Notes |
|---|---|---|
| workflow (branching) | fluxo de trabalho | "Definições > Repositório > Fluxo de trabalho"; Git Flow, GitHub Flow, GitLab Flow, "Trunk-based" and "Custom" are names (the template names come from code), kept |
| Flow (menu, toolbar button, palette category) | Flow | feature name kept, like Radar; the palette reads "Flow: iniciar branch..." (lower case after the colon) |
| base branch / protected / environment branch | branch base / protegida / branch de ambiente | "base" badge `[base]`, `[base, protegida]` |
| topic branch / topic type | branch de tópico / tipo de tópico | git's own word "topic"; the type labels (Feature, Release...) come from the workflow file |
| parent (branch), set parent | pai, definir o pai | "Pai: {name}", "novo pai" |
| name pattern / token / slug | padrão de nome / token / slug | `{slug}` etc. never translated; "marcador" only in the prose of the page for `{placeholder}` |
| start / finish / sync / promote | iniciar / concluir / sincronizar / promover | buttons as infinitives; "concluir" is also the review verb (a different screen); undo label stays "Flow {rest}" |
| finish targets / finish mode | destinos de conclusão / modo de conclusão | "Concluir por" (Finish by) |
| hook | hook (m.) | kept, as git-l10n does |
| pick up / take over (a host-made branch) | assumir | "Assumir uma branch criada pelo host...", button "Assumir", label "O que assumir"; "Mais tarde" / "Deixar de esperar" |
| bot | bot | kept |
| watched issue | issue vigiada | |
| waiting card | cartão de espera | |
| stacked branches | branches empilhadas | `--stack` |
| Detect / Import | Detetar / Importar | "Deteção" noun; score = pontuação |
| ahead / behind (a parent) | à frente / atrás | "{n} commits atrás de {parent}" |
| Copy as Mermaid gitGraph | Copiar como Mermaid gitGraph | the diagram name stays English |
| Mermaid detail levels Structure / Refs / Subjects / Full | Estrutura / Refs / Assuntos / Completo | the stored words `structure`, `refs`, `subjects`, `full` stay as config and CLI values; "Detalhe" label |
| Direction (left to right / top to bottom) | Direção (da esquerda para a direita / de cima para baixo) | never "orientação" in UI text |
| fence (```mermaid) | delimitador | "Incluir o delimitador ```mermaid" |
| Compare (patch sets), Submit / Abandon (Gerrit change) | Comparar / Submeter / Abandonar | "alteração" for a Gerrit change in these strings (pedido elsewhere); the server action "não o pode desfazer" |
| fetch inside prose | git "o próximo fetch" / local "a próxima obtenção" | write a `{ git, local }` pair because the `{$fetch}` token is a verb locally |
| merge as a modifier | "{$Merge} local", "Estratégia de {$merge}", "{$tag} em {$merge}" | prepositions without article so one text fits both genders |
