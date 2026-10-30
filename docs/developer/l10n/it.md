# Italian (it) localisation glossary

Binding for every change to `crates/gitpoulpi-i18n/locales/it.toml`. Sources: Microsoft Italian terminology and style guide, git's own `po/it` (git-l10n), GitHub Desktop / VS Code / JetBrains / TortoiseGit Italian UIs. Two vocabularies: **git** (default: the English git word, as Italian developers say it) and **local** ("Translate git terms": proper Italian where git-l10n or Italian usage has one). A message holding a git word is either a `{$term}` token or a `{ git = .., local = .. }` pair. The local text must not contain the English stem (lint `tests/git_vocabulary.rs`).

Tokens that are verbs in local mode (`{$push}` "invia", `{$pull}` "scarica", `{$fetch}` "recupera", `{$merge}` "unisci", `{$rebase}` "ribasa") never go where a noun or an article is needed: write a pair ("il push" / "l'invio", "il merge" / "l'unione").

| Concept | git vocabulary | local vocabulary | Notes |
|---|---|---|---|
| commit (n./v.) | commit (m.) | commit | git-l10n keeps it; "committato" only in informal status text |
| push / pull / fetch | push / pull / fetch | invia / scarica / recupera | nouns local: invio (m.), scaricamento (m.), recupero (m.) |
| merge | merge (m.) | unisci / unione (f.) | "l'unione", "una unione" never "fusione" |
| rebase | rebase (m.) | ribasa / ribasamento (m.) | never "ribasatura" |
| cherry-pick, revert | cherry-pick, revert | cherry-pick (kept), "annulla gli effetti" | revert row: "Annulla gli effetti del commit..." |
| pick, reword, squash, fixup, drop, edit | same | scegli, riformula, comprimi, correggi, scarta, modifica | `[term]` table |
| stash | stash (m.) | accantonamento (m.); verb "accantona" | "lo stash" / "l'accantonamento": pair, never `lo {$stash}` |
| stash pop / apply / drop | pop / applica / drop | estrai / applica / scarta | toolbar "Estrai" (<= 9 cells) |
| tag | tag (m.) | tag | git-l10n keeps "tag"; "etichetta" is only for issue labels and UI labels |
| diff | diff (m.) | diff | git-l10n keeps it; "differenza" only for the plain-language "file differ" |
| stage / unstage / staged / unstaged | stage, unstage, in stage, non in stage | same | git-l10n says "area di staging"; Italian developers say "stage", kept in both modes |
| pull request / merge request | pull request / merge request (f.) | richiesta di integrazione / richiesta di unione (PR/MR -> RI/RU) | host picks pull or merge |
| working tree | directory di lavoro (f.) | same | also for index + working tree |
| worktree (linked) | worktree (m.) | albero di lavoro (m.) | section "ALBERI DI LAVORO" (<= 18 cells) |
| branch, upstream, checkout | branch (m.), upstream, checkout | branch, upstream, "passa a" / "cambio di branch" | never "ramo" |
| remote | remoto (m.) | same | adjective "branch remoto"; never "remote" as a noun |
| blame, hunk, ref, SHA, LFS, WIP, Log | kept English | same | "il ref" (m.), "l'hunk"; "blocco" only for a merge-editor chunk |
| HEAD detached | HEAD scollegato | same | |
| settings, palette | Impostazioni, Palette dei comandi | same | never "Preferenze" (the palette entry is "App: Impostazioni...") |
| activity log | Registro attività | same | |
| submodule | sottomodulo | same | |
| conflict card / info card | scheda | same | |
| line endings, indentation | interruzioni di riga, rientro | same | never "terminatori di riga" / "indentazione" (user guide follows the catalog) |
| file manager | gestore file | same | never "file manager" in prose |
| working copy (of a file) | "il file nella directory di lavoro" | same | never "copia di lavoro" or "albero di lavoro" for the working tree |

## Style

- Address: informal "tu" everywhere (imperatives "scegli", "clicca", "premi"). Never "Lei" forms.
- Capitalisation: sentence case for labels, buttons, titles and headings; English Title Case is not copied. Palette and keymap entries are `Category: Action` with a capital after the colon; status and error sentences stay lower case after a colon.
- Actions: imperative ("Salva", "Crea branch...", "Apri nel terminale") for buttons, menu rows and palette commands; noun or participle for progress and results ("Salvataggio di {path}...", "{path} salvato"); help lines are third person ("apre", "chiude").
- Punctuation: no space before `: ; ? !`; guillemets «...» for quoted names and button names; plain ASCII apostrophe; no em dash (use ` - `); `...` or `…` as in the English source.
- Keys: `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Space`, `Tab`, `Home/End`, `PgUp/PgDn` stay as in English; arrows are "Su/Giù/Sinistra/Destra"; "Canc" for Del, never "Invio" or "Maiusc".
- Dates: `{day} {month} {year}` ("5 marzo 2024"), months and weekdays lower case, "3 giorni fa", "adesso"; `age_suffix` "{age} fa".
- "Default": "predefinito" (also for values and adverbially: "per impostazione predefinita").
- Placeholders, plural shapes (`one` iff n = 1) and `{$term}` tokens are never altered.

## Decisions of the 2026-10 catch-up (undo history, ellipses, terminals page)

| Topic | Decision |
|---|---|
| Undo history vocabulary | "Cronologia annullamenti" (panel, tab, `Alt+Z`), "Annulla fino a qui" / "Ripeti fino a qui", "Ripristina solo questa" (restore one entry without undoing the later ones; feminine, it agrees with "azione"), "Lavoro perso" (lost work), "Reflog" kept English, "pin di annullamento", "orfano" (dangling). |
| Status lines | Lower case after the start, like the English source (`salvataggio di {path}...`, `impossibile salvare {path}: {why}`); a line that starts with a placeholder (`{path} salvato`) is unchanged. Dialog sentences and questions keep the capital ("Le tue modifiche locali bloccano..."). The undo confirmation is `{op}: fatto - {key} per annullare`. |
| Ellipsis | Always three ASCII dots `...`, never `…`, in the catalog and the guide (the English source dropped `…`). A menu row that opens a dialog or asks first ends in `...` (Elimina {name}..., Scarta le modifiche..., Chiudi le altre schede...). |
| SHA | "lo SHA" (m.), never "l'ID del commit". |
| LFS store | "archivio LFS locale" (`lfs prune`), not "repository LFS". |
| Guide quoting | The guide quotes the catalog's git variant. English key cells that the code spells in English in the generated keymap and palette tables ("double-click", "Enter on the search row", "management tab") are put back into Italian by hand after regenerating (temporary local patch of `keymap::drift::docs_table` and `palette::tests::docs`: `Lang::It`, Italian headers, path `docs/user/it/keybindings.md`; run the two tests one at a time, they rewrite the same file; then revert). |
| Terminals page | `terminals.md` is "Scegliere un terminale"; the capability table keeps its cells translated ("sì", "no", "testo", "(non verificato)"), product names, versions, settings keys and sources unchanged. |

## Known residue (lint baseline `tests/git_vocabulary/it.toml`)

Inherent, cannot be written without the English stem: placeholder or key names that are code (`{tag}`, `{{merged}}`, `{{diff.merge_editor}}`, `git.pull_mode`, `git.push_behavior`, `general.auto_stash`, `ui.graph_show_stashes`), the git ref name `stash@{n}`, and the English prompt chips sent to the model (`ai_page.chip_*_text`). The baseline may only shrink.

## Decisions of the workspace / search / Radar pass

| Topic | Decision |
|---|---|
| workspace | "area di lavoro" (f.), never "workspace" or "spazio di lavoro"; the Home pages are "Repository" and "Aree di lavoro". Not to be confused with "directory di lavoro" (working tree). |
| New tab | "Nuova scheda"; the `+` button's tooltip is the same. |
| pin / snooze | "fissa" / "non fissare più" (state "Fissati"); "rimanda" / "non rimandare più" (state "Rimandati", "rinvio" as a noun in prose). |
| Radar | Kept as a product name; groups "In attesa della mia revisione", "Le mie richieste", "Assegnate a me", "Lavoro in corso". The English "WIP" chip stays "WIP" in the view list, "Lavoro in corso" in headings. |
| Keys in hints | "Enter", never "Invio" (also in the Radar hint line). |
| Load more | "Carica altro". |
| Status lines | Loading lines are nominal ("interrogazione di GitHub e GitLab...") like the other progress texts. |

## Follow-up to the English source review (2026-10)

- Radar row actions take a bare noun: "Apri repository", "Checkout del branch", "Crea worktree...", "Mostra nel repository", "Inizia a lavorare su issue...". Status lines are impersonal and lower case: "{item} non più rimandato", "posizioni caricate: {n}".
- `Area di lavoro: Fetch All` / `Pull All` are `{ git, local }` pairs ("Recupera tutto" / "Scarica tutto"); a bare `{$Fetch} tutto` read badly in git mode.
- One term for workspace everywhere: "area di lavoro" ("workspace" only inside multiplexer prose that names tmux/zellij workspaces).

## Decisions of translation round 4 (review, metadata, providers, local issues, board)

| English | Italian | Notes |
|---|---|---|
| review (n.), to review | revisione (f.), "rivedere" / "Rivedi..." | palette category "Revisione:"; never "recensione"; "code review" = "revisione del codice" |
| request (generic PR/MR/change) | richiesta | `{$pr}` only where the text names a pull/merge request specifically |
| thread | discussione (f.) | GitHub says "conversazione" for it; here "conversazione" is reserved for the Overview's *Conversation* (comments not on code) |
| pending comment / pending review | commento in sospeso / revisione in sospeso | badge "In sospeso - riga 12"; never "bozza" (that is the draft flag of a request) |
| resolve / reopen (thread) | risolvi / riapri; "risolta" | |
| approve / request changes / comment (verdicts) | Approva / Richiedi modifiche / Commenta | the choice is "esito" (verdict); submitted badges "approvata", "modifiche richieste", "commentata", "ignorata" (dismissed) |
| Finish review / Submit review / Discard review | Termina revisione / Invia la revisione / Scarta la revisione | |
| suggestion (AI) | suggerimento; Accept / Edit / Dismiss = Accetta / Modifica / Ignora | severities "info", "avviso", "problema"; "riepilogo IA" for the AI summary; the AI is always "IA" |
| patch set | patch set | kept English (Gerrit term, Italian devs say it); "ultimo" for latest |
| vote (Gerrit) | voto | labels `Code-Review`, `Verified` stay |
| viewed (file) | visto ("{n} visti") | "Segna il file come visto" |
| checks (CI) | controlli ("Controlli superati / falliti / in corso") | |
| board / column / card | bacheca / colonna / card (f.) | "scheda" is the tab and the info card, so a board card is the English "card"; Open / Closed columns "Aperte" / "Chiuse" (issue is feminine) |
| milestone | milestone (f.) | kept English like GitHub's Italian UI |
| assignee / reviewer / label | assegnatario / revisore / etichetta | "etichetta" also for UI labels |
| encoding | codifica | "Rilevamento automatico" for Auto-detect; button value "Auto" |
| notes folder / vault | cartella delle note / vault (Obsidian) | "Base di Obsidian" for the `.base` file |
| local issues | issue locali | palette category "Issue locali:" |
| Mentioning me | Che mi menzionano | Radar filter and group |
| ISSUES section | sezione ISSUE | as the sidebar title |
| gutter (of the diff) | margine | |

## Decisions of translation round 5 (branching workflows, Mermaid, Gerrit, board)

| English | Italian | Notes |
|---|---|---|
| workflow (the whole definition) | workflow (m.) | kept English like GitHub's Italian UI; "flusso di lavoro" is only GitHub Actions wording. "Workflow di branching" for branching workflow; templates `Git Flow`, `GitHub Flow`, `GitLab Flow`, `Trunk-based`, `Custom` keep their names, "modello" for the word template |
| base branch | branch base | badge `[base]`, `[base, protetto]`; "Branch base trovati" for Detect |
| topic type / topic branch | tipo di topic / topic branch | "topic branch" kept English (git docs); the Start form's `Type` is "Tipo"; heading "Tipi di topic" |
| parent (of a branch) | genitore | git-l10n word for commit parents; "Genitore: {name}", "Sincronizza con il genitore...", "Imposta il genitore..." |
| environment branch | branch di ambiente | GitLab Flow |
| start / finish / sync / promote | Avvia / Termina / Sincronizza / Promuovi | "Termina" matches "Termina revisione"; "modalità di termine", "destinazioni finali" (finish targets) |
| hook | hook (m.) | kept English |
| pick up (take over a host-made branch) | prendi in carico | never "recupera" (that is the local `fetch` verb); progress "presa in carico del branch..." |
| issue flow | issue flow | kept as the heading "Issue flow (guidato dall'host)"; prose "flussi guidati dall'host" |
| new issue of a type | Nuova issue {label}... | menu and palette `Flow: Nuova issue {label}...` |
| Flow (button, palette category) | Flow | kept English |
| slug, token, pattern | slug, token, pattern del nome | kept English |
| Mermaid detail levels | Struttura / Ref / Oggetti / Completo | stored words `structure`, `refs`, `subjects`, `full` stay lowercase English in config and CLI; "Oggetti" = commit subjects; "Direzione": "Da sinistra a destra" / "Dall'alto in basso" |
| gitGraph | Mermaid gitGraph | always in this order; "Copia come Mermaid gitGraph..." |
| Gerrit submit / abandon / compare | Submit / Abbandona / Confronto: base, Confronto: patch set {n} | "Submit" kept (Gerrit term); palette "Revisione: Esegui il submit della richiesta..."; Gerrit change = "richiesta" in the catalog, "modifica Gerrit" in the guide prose |
| merge strategy | strategia di merge (git) / strategia di unione (local) | `{ git, local }` pair, like the other noun uses of `{$merge}` |
| Why-reasons and statuses of Flow | lower case fragments / clauses | "fatto: {what}", "interrotto", "elaborazione dei passaggi..." |
