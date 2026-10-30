# pt-BR terminology and style (crates/gitpoulpi-i18n/locales/pt-BR.toml)

Sources: git's own pt_BR translation (git-l10n), Microsoft pt-BR terminology and style guide, and how GitHub Desktop, VS Code, Visual Studio, TortoiseGit, SourceTree and GitLab word the same things in Brazilian Portuguese. Follow this table for any new key.

## Two vocabularies

Default ("git" variant): what Brazilian developers actually say, so git words stay English. "Translate git terms" on ("local" variant): the Portuguese word. Every git word that has a different local form is a `{$term}` token; where the grammar changes with the term (gender, article, noun vs verb) write a pair `{ git = "..", local = ".." }`.

| Concept | git variant | local variant | Notes |
|---|---|---|---|
| commit (n./v.) | commit | commit | git-l10n and Microsoft tools keep it; "fazer commit", "commitar" in informal lines |
| push | push | enviar (noun: envio) | verb token `enviar`; after an article or "de" write a pair ("o envio de"); force push = "envio forçado" |
| pull | pull | puxar (noun: puxada) | same rule; "faça pull" / "puxe" |
| fetch | fetch | buscar (noun: busca) | "URL de busca", "Buscar tudo" (Fetch All) |
| merge | merge | mesclagem (noun, f.) | git-l10n; verb "mesclar" is written inline in a pair; the token is the noun, so "editor de mesclagem", "Mesclagem em andamento" |
| rebase, cherry-pick, stash | same | same | kept (git-l10n keeps them); pop/apply/drop become retirar/aplicar/descartar in a pair |
| tag | tag (f.) | etiqueta | git-l10n |
| diff | diff (m.) | diferença (f.) | gender flips: "o diff" / "a diferença" needs a pair; external tool = "ferramenta de comparação" |
| stage / unstage / staged / unstaged | same | preparar / remover da preparação / preparado / não preparado | git-l10n "área de preparação" |
| pull request / merge request | pull request / merge request (m.) | solicitação de integração / solicitação de mesclagem (f.) | follows the host; abbreviations PR / MR vs SI / SM; articles need a pair ("o" / "a") |
| squash / fixup / reword / pick / drop / edit | same | combinar / absorver / reescrever / escolher / descartar / editar | rebase todo actions |
| checkout, fast-forward, worktree, upstream, hunk, blame, amend, LFS, HEAD, SHA, revert | kept | kept | no token; revert/reset local arm = reverter / redefinir (soft/mixed/hard = suave/misto/forte) |
| branch (f.) | branch | branch | never translated in either variant ("a branch", "nova branch") |
| repository, remote | repositório, remoto | same | never English; "o remoto" |
| conflict, resolve | conflito, resolver | same | merge editor panes: Atual (A), Recebido (B), Base, Resultado |
| working tree | árvore de trabalho | same | git-l10n; "diretório de trabalho" only for the editor ("Editar no diretório de trabalho") |
| settings | Configurações | same | English says "Settings" everywhere; never "Preferências" |
| palette, log, graph, minimap, avatar, issue | paleta de comandos, Log (pane) / registro (application log), grafo, minimapa, avatar, issue | same | `Log` is the commit pane; "Registro de atividade" is the application log |
| lane | faixa | same | never "pista" |
| banner | aviso | same | a banner (merge in progress, auto-stash) is "aviso", never "faixa" (that is the lane) |
| detached HEAD | HEAD desanexado | same | git-l10n |
| working changes | Alterações em andamento | same | the pane and the WIP row; never "alterações de trabalho" |
| undo journal | diário (de desfazer) | same | the file stays `journal.json` in code spans |
| credential helper | auxiliar de credenciais | same | as the catalog; "Git Credential Manager" stays a product name |
| tooltip, combobox | dica, lista suspensa (caixa de combinação for the toolbar profile box) | same | never "tooltip" or "combo" in prose |
| issue tracker | rastreador de issues | same | the user guide uses it in the hosting page title and in every link to it |
| undo history | histórico de desfazer | same | the panel and its three views: Histórico de desfazer, Reflog, Trabalho perdido (lost work); `Desfazer até aqui`, `Refazer até aqui`, `Restaurar só esta` (restore only this); palette twins are `Histórico de desfazer: ...`, `Reflog: ...`, `Trabalho perdido: ...` (`buscar de novo` = Scan Again); a blocked entry is `bloqueada`, an undone one `desfeita` (feminine: the noun is "ação") |
| reflog, undo pin | reflog, pin de desfazer | same | kept as git words; "commits perdidos" for lost commits, "soltos" for dangling ones |

## Style

- Address: neutral "você" or no pronoun; imperative in instructions ("selecione antes um arquivo"), infinitive in buttons, menus and palette actions ("Salvar", "Criar branch"), noun phrase or gerund in progress messages ("criando branch...", "{$Merge} em andamento").
- Status-line messages (the catalog keys English writes in lower case, such as `msg_saved`, `act_*`, `msg_ai_done`) start in lower case, with a lower-case term token (`{$merge}`, not `{$Merge}`) when the token opens the message.
- Capitalisation: sentence case everywhere (only the first word and proper names), as in Microsoft pt-BR; English's Title Case in palette and keymap labels is not copied; section titles of the sidebar in capitals like the English ones. Palette: `Categoria: ação` with a lower-case action after the colon (`Refs: renomear a branch selecionada...`), category names are the translated English ones (Remoto, Repositório, Aba, Exibir, Arquivos, Alterações, Faixa, Mesclagem).
- Punctuation: no space before `:` `;` `?` `!`; ellipsis is always three ASCII dots `...` (catalog and user guide, as in English; never the `…` character); a menu or button that opens a dialog before acting ends with `...`; quotes for UI names are “ ” (code, shell and config examples keep `"`); ASCII `'` only (catalog test); no em dash, " - " instead.
- Mouse words: "clique", "clique duplo", "clique com o botão direito", "arrastar", "passar o mouse". Never "duplo clique", "clique direito" or "clique duas vezes" in the user guide. Keys stay `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Backspace`, `Delete`; arrow keys are "Cima/Baixo/Esquerda/Direita" in prose and ↑↓ in hints.
- Dates: `5 de março de 2024`, months and weekdays lower case (`março`, `terça-feira`, abbreviations `mar`, `ter`), relative `há 5 minutos`; 0 and 1 are singular (CLDR pt).
- Plural: `one` for 0 and 1, `other` otherwise.
- Width: toolbar and button labels stay as short as the English ones; abbreviations in tight spots: `Config.` (Setup tab), `SI`/`SM`, `EB` (WS), `{n}a` (files).
- Lint limits: placeholders named `{stage}`, `{tag}`, `{merged}`, keymap names such as `{diff.edit}` and code names (`git.push_behavior`) are flagged by the local-mode lint and stay in the baseline `tests/git_vocabulary/pt-BR.toml`.
- Undo history labels (`[undo_label]`): imperative verbs, nouns kept as git words (`Checkout de`, `Merge de`, `Revert de` in the git variant; `Reverter`, `Redefinir`, `Mesclagem de` in the local one); stash verbs are pairs (`Pop do stash` / `Retirar o stash`, `Drop do stash` / `Descartar o stash`); `push URL` = `URL de push` / `URL de envio`; reset modes `soft/mixed/hard` / `suave/misto/forte`.
- Mainline of a merge = "linha principal" (git-l10n); "pai {n}" for a parent; outside change = "alteração externa" (tag `externa`, feminine like "ação"); status messages for it stay lower case.
- The `[agent_reason]` section is English-only by design (agents read English); not translated.
- Workspace = "espaço de trabalho" (m.; never "workspace" or "área de trabalho"); short form in tight spots "espaço". Radar stays a product name. Pin = "fixar" / "Fixados" / "desafixar"; snooze = "adiar" / "Adiados" / "parar de adiar". Missing repository = "ausente" (clone/locate = "Clonar" / "Localizar"). Fetch all / Pull all = `{$Fetch} tudo` / `{$Pull} tudo`.
- Palette twins of the new features follow the rule above: lower-case action after the colon (`Radar: fazer checkout da branch`, `Espaço de trabalho: trocar...`, `Aba: nova aba`). Radar row actions are bare ("Abrir repositório", "Criar worktree...", "Começar a trabalhar na issue..."); status messages in lower case (`posições carregadas: {n}`).

## Round 4 terms (review, metadata, new providers, local issues, board)

| Concept | pt-BR | Notes |
|---|---|---|
| request (generic PR / MR / Gerrit change) | solicitação | "a solicitação"; the git/local pair is not needed because it names no PR token; the CLI help keeps "pull/merge request" |
| code review (the feature) | revisão (de código); palette category "Revisão:"; verb "Revisar..." | the in-app review view; GitHub pt-BR wording |
| pending comment / pending review | comentário pendente / revisão pendente | kept on the machine until "Concluir revisão" |
| thread | thread (f.) | GitLab pt-BR keeps it; badges "resolvida" / "desatualizada"; the conversation page is "Conversa" |
| resolve / reopen | Resolver / Reabrir | threads and issues alike |
| verdict (approve / request changes / comment) | Veredito: Aprovar / Solicitar alterações / Comentar | badges on a submitted review: "aprovada", "alterações solicitadas", "comentada", "dispensada" |
| finish / submit / discard review | Concluir revisão (opens the form) / Enviar revisão (sends it) / Descartar revisão | GitHub pt-BR |
| suggestion (AI review) | sugestão; Aceitar / Editar / Dispensar; severities info / aviso / problema | the star stays in front: "Revisão: ✦ sugerir revisão" |
| patch set, vote (Gerrit) | patch set (kept, like hunk), voto | Gerrit labels (`Code-Review`, `Verified`) and `refs/for/` stay as is |
| board / column / card | quadro / coluna / cartão | columns "Aberto" / "Fechado"; palette category "Quadro:"; "Issues: abrir quadro" |
| milestone | marco | "Definir marco...", "Sem marco" |
| assignee / reviewer | responsável (pl. responsáveis) / revisor | "Editar responsáveis...", "Editar revisores..." |
| label (issue / PR) | rótulo | different from the local-vocabulary tag ("etiqueta") |
| encoding | codificação | "Detecção automática", "Escolher a codificação..." |
| checks (CI) | verificações ("aprovadas", "com falha", "em andamento") | |
| mentioning me (Radar) | Menções a mim | |
| notes folder / vault / Base | pasta de notas / cofre do Obsidian (Obsidian's pt-BR word) / "Base do Obsidian" | local issues = "issues locais"; palette category "Issues locais:" |
| workspace AI instructions | Instruções para a IA (do espaço de trabalho) | levels: global, espaço de trabalho, repositório, ação; "Somar ao global" / "Substituir o global", "Combinar" |
| hosting palette category | Hospedagem | never "Hosting:" in a palette entry |
| fetch of a request's code (git variant) | "fazer fetch do código" (pair: local "buscando o código") | the vocabulary lint needs the git word in git mode |

## Round 5 terms (branching workflows, Mermaid, Gerrit compare / submit / abandon)

| Concept | pt-BR | Notes |
|---|---|---|
| branching workflow | fluxo de trabalho (de branches) | the whole definition; the toolbar button, the palette category and the dropdown stay `Flow` (a name, like git-flow) |
| base branch / topic branch / topic type | branch base / branch de tópico / tipo de tópico | "tipo" alone in forms (field `Tipo`); environment branch = "branch de ambiente" (GitLab Flow) |
| parent / "Set parent" | pai / "Definir o pai" | "pai" is the branch a branch starts from and finishes into; "Novo pai" for the new one; badge `[base, protegida]` |
| start / finish / sync / promote | iniciar / concluir / sincronizar / promover | buttons are bare infinitives; "Concluir" is also the review's "Concluir revisão" (different context, fine) |
| hook | hook (m.) | kept like hunk and patch set; "Após iniciar" / "Antes de concluir" for the two lists |
| pick up (a host- or bot-made branch) | assumir | "Assumir uma branch criada pelo host...", "{name} assumida"; never "pegar" |
| issue flow (host-driven) | fluxo de issues (conduzido pelo host) | modes "Local", "O host cria a branch", "Um bot cria a branch" |
| stacked branches | branches empilhadas | "Atualizar também as branches empilhadas sobre ela" |
| slug / token | slug / token | pattern placeholders `{slug}` etc. stay Latin |
| name pattern | padrão de nome | "Outros padrões", "Expressão regular" |
| Mermaid detail levels | Estrutura / Refs / Assuntos / Completo | CLI values `structure`, `refs`, `subjects`, `full` stay as code; "Direção": "Da esquerda para a direita" / "De cima para baixo"; "gitGraph do Mermaid" (m.) |
| Gerrit submit / abandon | Submeter / Abandonar | Gerrit's own word is submit; "Enviar revisão" stays for sending a review. The change is "a solicitação" in the catalog (generic request), "change" in the user guide as before |
| compare (patch sets) | Comparar: base / Comparar: patch set {n} | combobox rows of the review header |
| wait / timeout | aguardar / "desistindo após {max} s" | waiting card; status "issue {reference} criada" lower case |
| next fetch (pair) | o próximo {$fetch} / a próxima busca | gender flips with the vocabulary: write a pair (`git` arm masculine, `local` arm feminine); same for "Abrir um {$pr}" / "Abrir uma {$pr}" and "a {$merge}" (`{$Tag} no {$merge}` / `{$Tag} na {$merge}`) |
