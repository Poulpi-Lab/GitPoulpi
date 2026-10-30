# French (fr) terminology and style

Written during the user-guide review (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/fr.toml` and to `docs/user/fr/` follow this page. Sources: git's own French translation (git-l10n `po/fr`), Microsoft and Apple French style guides, how GitHub, GitLab, VS Code and JetBrains word things in French, and what French-speaking developers say aloud (the English git word, conjugated: "commiter", "pusher", "stasher"). Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Formal "vous", in sentences only (dialogs, hints, errors, the guide): "Choisissez", "Ouvrez", "votre dépôt". Never "tu". Most catalog strings avoid addressing the user. |
| Buttons, menu rows, palette commands | Infinitive, capitalised first word: "Créer une branche...", "Ouvrir le dossier". Palette: `Catégorie : Action`. |
| Progress and results | Progress: a noun phrase ending in `…` ("Application de l'extraction partielle…"). Results: past participle in lower case ("extraction partielle appliquée : {dirs}"). |
| Reasons for a greyed row (`why_*`, `select_*`) | Lower case, imperative or noun phrase: "ouvrez d'abord la vue Blame d'un fichier". |
| Capitalisation | Sentence case for labels, tabs, titles, columns; sidebar headings upper case (`LOCALES`, `DISTANTES`, `SOUS-MODULES`). Pane names keep their capital mid-sentence ("le volet Status", "dans le Log"). |
| Typography | A no-break space (U+00A0) before `:`, `;`, `!`, `?`, and inside guillemets: `« texte »`. Guillemets « » for UI names and quoted values in prose. Never straight `"` in French prose (only inside code). The plain ASCII apostrophe `'` is required by the catalog test; never `’`. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Ellipsis | Three ASCII dots `...` everywhere, like English (`Parcourir...`, `chargement...`, `Édition : Reflog...`); never `…`. A command that opens a dialog or asks for confirmation ends with `...` where the English does (`Supprimer {name}...`). |
| Status-line messages | Lower case first letter, like English (`impossible d'ouvrir {path} : {why}`, `enregistrement de {path}...`); a message starting with a token or an acronym keeps its case. |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Up`/`Down`, `PageUp`/`PageDown`, `Home`/`End` stay Latin, because the chord printer shows them; never "Entrée", "Maj", "Haut" for a key. Mouse chords: `Ctrl+clic`, `Shift+clic`, `Alt+clic`, "double-clic", "clic droit". A menu "Entrée" (an entry) is not a key. |
| Dates and times | `{day} {month} {year}` with lower-case months ("5 mars 2024"); relative "il y a 5 minutes". |
| Plurals | CLDR fr: `one` iff n is 0 or 1 ("0 fichier", "1 fichier", "2 fichiers"). |
| Numbers | Decimal comma in prose ("32,0 Mio"); sample program output keeps the dot. |
| Settings screen | "Paramètres" (Windows, GNOME, VS Code, JetBrains French; the catalog follows the English "Settings"). Menu, tab, tooltip, palette `Application : Paramètres...`, "Paramètres du dépôt" for the repo page; one setting = "un paramètre" (never "réglage"/"préférence"). The guide follows. |
| Palette capitals | Every palette command is `Catégorie : Action` with a capital after the colon, including `Fichiers : Tout sélectionner` and `Notification : Fermer`; only `why_*` reasons start in lower case. A colon inside parentheses (`(soft : garde index et fichiers)`) is not a palette colon. |
| Headers | Column headers stay no longer than the English one: `Graph` (not "Graphe"), `Modifs`. |

## Genders (the ones that bite)

le commit (la validation in local mode), la branche, le tag (l'étiquette), le stash (la remise), le merge (la fusion), le rebase, le fetch, le push, le pull, le diff (la différence), le hunk, l'index, la pull request (la demande de tirage), la merge request (la demande de fusion), le worktree, le HEAD ("HEAD détaché"), la ref, la palette, le remote / le distant, le ticket, le jeton, le dépôt, le démon. Where grammar changes with the term (le commit / la validation), the catalog writes a `{ git = .., local = .. }` pair instead of relying on the token.

## Glossary

`git` = default vocabulary (English git words, "Traduire les termes git" off). `local` = translated vocabulary (setting on), following git-l10n where it translates. The user guide quotes the `git` variant. Where both columns are the same, the English word is the French word.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit (noun) | commit | validation | verb in prose: "commiter", "modifications non commitées" |
| branch | branche | branche | git-l10n; "branche locale", "branche distante" |
| merge | merge | fusionner / fusion | "éditeur de merge" / "éditeur de fusion" |
| rebase | rebase | rebaser | |
| cherry-pick | cherry-pick | cherry-pick | |
| push / pull / fetch | push / pull / fetch | pousser / tirer / récupérer | |
| stash | stash | remise (remiser) | git-l10n "remisage"; pop = dépiler, apply = appliquer, drop = supprimer (terms `pop`, `apply`, `stash_drop`) |
| tag | tag | étiquette | git-l10n. The issue "label" is "étiquette" too in hosting prose; the small `LFS` marker is the "étiquette `LFS`" |
| diff | diff | différence | |
| stage / unstage / staged / unstaged | stage / unstage / staged / unstaged | indexer / désindexer / indexé / non indexé | the guide says "stager", "stagé" in git mode prose |
| interactive rebase actions | pick, reword, squash, fixup, drop, edit | garder, reformuler, compresser, absorber, abandonner, modifier | terms `pick`, `reword`, ... |
| pull request / merge request | pull request / merge request | demande de tirage / demande de fusion | host dependent; PR / MR abbreviations become DT / DF in local mode |
| working tree | arbre de travail | arbre de travail | git-l10n. A linked worktree (feature) is "worktree" in git mode, "arbre de travail" in local mode (sidebar heading `ARBRES DE TRAVAIL`). English "Working Directory" (as in "Edit in Working Directory") is "dossier de travail" |
| revert | revert | inverser / inversion | "Inverser la validation...", "Inversion en cours"; term not tokenised, so a `{ git, local }` pair |
| reset (soft/mixed/hard) | Reset (soft/mixed/hard) | Réinitialiser (douce/mixte/dure) | |
| fast-forward | fast-forward | avance rapide | |
| checkout | checkout | basculer (sur) | "extraire" only for "already checked out" / worktree wording |
| prune | Auto-prune | Élagage automatique | |
| edit (verb) | Modifier | Modifier | "Éditer" only for the width-limited toolbar button `Éditer` and "Éditer dans..." (open in an external app) |
| working changes | modifications locales | modifications locales | |
| repository | dépôt | dépôt | git-l10n |
| remote | distant | distant | "Push vers un distant", "un distant github.com"; never "remote" in prose (only inside code such as `{{remote}}`) |
| upstream | upstream (ref menu "Définir l'upstream de ...") / amont elsewhere | amont | the ref-menu row keeps the English word in git mode (test-pinned) |
| submodule | sous-module | sous-module | |
| workspace | espace de travail | espace de travail | a named group of repositories (`[workspaces]`); "Espaces de travail" for the Home page, "Nouvel espace de travail" for the button |
| pin / snooze (Radar) | épingler, désépingler / reporter ("Reportés", "Jusqu'à demain") | same | "snooze" is never left in English; a Radar item is "un élément"; `Radar` and `WIP` keep their English names (chip, heading "Travail en cours") |
| New tab, Load more | Nouvel onglet, Charger la suite | same | Reflog "Load more" = "Charger la suite" (status line "positions chargées : {n}", lower case); a missing repository of a workspace = "absent" (`{n} absent(s)`) |
| conflict | conflit | conflit | merge editor sides: "Courant" (A) / "Entrant" (B) / "Base" / "Résultat" |
| issue | ticket | ticket | "suivi de tickets" for issue tracker |
| host, provider, token | hébergeur, fournisseur, jeton | same | hosting service = "hébergeur"; an AI or tracker service = "fournisseur" |
| settings | Paramètres | Paramètres | see the style table |
| command palette | palette de commandes | same | "palette" alone is fine in context |
| tab, pane, popup, dialog | onglet, volet, fenêtre contextuelle, boîte de dialogue | same | |
| footer, status bar, banner | pied de page, barre d'état, bandeau | same | "pied de page" for the bottom key-hint line, "barre d'état" for the editor and theme `statusbar_*`, "ligne d'état" for status messages |
| tooltip, toast, card, chip | infobulle, notification, carte, pastille | same | |
| rebind, shortcut, chord | réaffecter, raccourci, enchaînement | same | |
| sparse checkout | extraction partielle | same | badge `partiel` |
| undo / redo | Annuler / Rétablir | same | |
| daemon, session | démon, session | same | git-l10n and Debian use "démon" |
| blame, hunk, minimap, avatar, graph | Blame, hunk, minimap, avatar, graphe | same | "fiche" for the blame hunk card; "bande de chaleur d'âge" for the age heat strip |
| Home / Log / Refs / Status | Accueil / Log / Refs / Status | same | pane names Log, Refs, Status stay Latin |
| activity log | Journal d'activité | same | |
| undo history, reflog, lost work | Historique d'annulation, Reflog, Travail perdu | same | `Reflog` stays Latin like Log. "Restore only this" = "Restaurer seulement celle-ci" (une action); "undo pin" = "ancre d'annulation"; "dangling" = "orphelin"; status lines avoid agreement with the operation name (`{op} : fait - {key} pour annuler`) |
| terminal pictures | images | same | `terminals.md`: "images" (never "photos"), "vraie couleur" for truecolor, "clavier kitty", "multiplexeur"; (unverified) = (non vérifié) |

## User guide (docs/user/fr)

Decisions of the user-guide review (2026-10), on top of the glossary above.

| Topic | Decision |
|---|---|
| Quoted labels | UI text is quoted exactly as the catalog shows it in the `git` variant ("Commiter N fichiers", "Mettre à jour le message", `Navigation : Rechercher des commits...`). |
| Keymap and palette tables | `keybindings.md` has `keymap:` and `palette:` markers like the English page. They are generated from the French catalog with a temporary local patch of the two docs tests (`keymap::drift::docs_table`, `palette::tests::docs`: point them at `docs/user/fr/keybindings.md`, use `Lang::Fr`, French table headers; then revert). The few key cells that the code spells in English ("double-click", "Enter on the search row") are put back into French by hand after regenerating. |
| Page names | Same as the README of the French guide: "Stage et commit", "Travailler avec le Log", "Paramètres", "Hébergeurs et suivi de tickets", "Référence CLI", "Référence de configuration", "Dépannage et FAQ". |
| Anchors | Links to headings use the slug of the French heading (`settings.md#clavier-et-souris`); change both together. |

## Catalog review decisions (2026-10, fr.toml pass)

- Local-mode text must read as French: no inline "diff", "push", "revert", "checkout", "stash" (use `{ git, local }` pairs or `{$term}` tokens). The lint baseline `tests/git_vocabulary/fr.toml` keeps only inherent residue: placeholders and identifiers (`stash@{{id}}`, `git.pull_mode`, `{commit}`), English AI prompt chips, and the sentences that explain the setting itself.
- Hunk stays "hunk" in both vocabularies ("Hunk suivant"); "bloc" is only the merge-editor chunk.
- "Log" is the pane name (never "Journal"); "Journal d'activité" / "Journal de l'application" is the application log.
- Merge editor: "éditeur de merge" (git) / "éditeur de fusion" (local), never "éditeur de conflits".
- Compositeur (never "composeur").
- Tests that pin French text (`clone_dialog`, `preferences`, `ai_ui`, `ui_intent`, `hosting_ui`, `ref_menu`, `log_table`, keymap label widths) fail when a word here changes: update their expectation in the same commit.

## Undo history labels (2026-10)

- `[undo_label]` rows reuse the catalog's command words: "Checkout {name}" / "Basculer sur {name}", "Remiser" (stash), "Abandonner" (discard and abort, like the rest of the catalog), "Indexer", "Définir l'upstream" / "Définir l'amont", "Réinitialiser (douce/mixte/dure)", "Inverser".
- Outside changes ("extérieur", "hors de GitPoulpi"): rows that quote git's reason (`outside_checkout`, `outside_reset`, `outside_amend`, worktree added/removed) are `{ git, local }` pairs so local mode has no inline English git word.
- Radar row actions keep their article in French ("Ouvrir le dépôt", "Basculer sur la branche", "Créer un worktree..."): the English bare-noun rule does not carry over.
- "... all" rows (workspace, LFS) are `{ git, local }` pairs ("Fetch All" / "Tout récupérer"), never `{$Fetch} tout`: a git term followed by a French word reads badly.

## Review, metadata, boards and local issues (2026-10, round 4)

New vocabulary of the in-app code review, the metadata pickers, the Kanban board and local Markdown issues. The `request` of the English source is "demande" in prose (a pull or merge request, a Gerrit change); the row and palette labels keep the host word where the catalog already did (`Relire une {$pr}...`).

| Concept | French | Notes |
|---|---|---|
| review (the activity, the pending state, the view) | revue ; verb "relire" | `Revue : Terminer la revue...`, "Revue de code" (AI action). Never "révision" (git's own word for a revision/rev) |
| code review suggestion by the AI | suggestion | "Revue par l'IA", "Suggérer une revue", "Accepter", "Écarter" (dismiss), "Tout écarter" |
| pending comment / pending review | commentaire en attente / revue en attente | "en attente" (never "brouillon", which is the draft PR and the Gerrit draft) |
| thread | fil de discussion ; "fil" alone in palette rows and buttons | GitHub and GitLab French. "Répondre", "Résoudre", "Rouvrir"; "résolu", "obsolète" (outdated) |
| conversation | discussion | the non-code comments of a request |
| Finish review / Submit review / Discard review | Terminer la revue / Envoyer la revue / Abandonner la revue | "Abandonner" is the catalog's discard word |
| verdict, Approve, Request changes, Comment | verdict, Approuver, Demander des modifications, Commenter | badges: "approuvée", "modifications demandées", "commentée", "écartée" (dismissed) |
| viewed (file) | vu | "Marquer le fichier comme vu" |
| patch set, vote, label (Gerrit) | patch set, vote, label | Gerrit words stay English like the host; "label" (Gerrit vote label) is not the "étiquette" of an issue label |
| assignee / reviewer | assigné / relecteur | pickers "Modifier les assignés...", "Modifier les relecteurs..." |
| label (issue, PR) | étiquette | as in the glossary |
| milestone | jalon | "Définir le jalon...", "Aucun jalon" |
| encoding | encodage | "Détection automatique" for Auto-detect; button `Encodage`, value `Auto` |
| board, column, card | tableau, colonne, carte | `Tickets : Ouvrir le tableau`, `Tableau : Déplacer la carte vers...`; the Kanban board and a Trello board are both "tableau" |
| local issues, notes folder, vault | tickets locaux, dossier des notes, coffre | Obsidian's French word is "coffre"; "Base Obsidian" for the file type; issue stays "ticket" everywhere (never "issue") |
| checks (CI) | vérifications | "Vérifications réussies / en échec / en cours"; `check runs` stay English |
| mentioning me | qui me mentionnent | Radar group and filter |
| head ref, merge base | head ref, base du merge | refs and git words stay as in CLI help |

Style notes of this round: status lines and `why_*` reasons in lower case with no final period; sentences shown in a view or dialog (`too_small`, `retry_hint`, notes) keep their capital and period; key names in footer hints stay Latin (`Space`, `Enter`, `Esc`, `Shift+Left/Right`).

## Branching workflows, Mermaid, Gerrit and board follow-ups (2026-10, round 5)

Vocabulary of the `[flow]` and `[settings_workflow]` catalog sections, the CLI `flow` help and `docs/user/fr/branching-workflows.md`.

| Concept | French | Notes |
|---|---|---|
| workflow | workflow | the English loanword, as in GitHub Actions' French UI; never "flux de travail". Title `Workflow`, "Modèle de workflow", "Paramètres du workflow...". The undo label is `Workflow {rest}` (the label test requires words different from English, so not `Flow {rest}`) |
| Flow (toolbar button, palette category) | Flow | proper name like Log and Reflog: `Flow : Démarrer une branche...`, "Ouvrir le menu Flow". Not "Flux" (that word is only used in "Flux de tickets" for the issue flow) |
| base branch | branche de base | badge `[base]`, `[base, protégée]`; "tronc" for the trunk |
| topic branch / topic type | branche thématique (git's own word) / type de branche | the settings say "Types de branche"; CLI prose "branche thématique" |
| parent branch, parent | parent | "À jour avec son parent"; "branche parente" only for a type's parent branch in errors |
| environment branch | branche d'environnement | GitLab Flow |
| start / finish / sync / promote / set parent | Démarrer / Terminer / Synchroniser / Promouvoir / Définir le parent | buttons `Démarrer`, `Terminer`, `Synchroniser`, `Promouvoir`, `Définir` |
| abort (a stopped action) | Abandonner | the catalog's discard/abort word; Continue = Continuer |
| pick up (take over a host-made branch) | Reprendre ("Reprendre une branche créée par l'hôte...") | never "Récupérer": that is the local-mode word for fetch |
| hook | hook | git-l10n says "crochet", developers say "hook"; plural "hooks" |
| name pattern, token, slug | modèle de nom, variable, slug | `{slug}` etc. are "variables" in prose; "Slug" stays Latin |
| stacked branches | branches empilées | "Mettre aussi à jour les branches empilées dessus" |
| issue flow (host-driven) | Flux de tickets (piloté par l'hôte) | |
| gitGraph detail levels | Structure, Refs, Sujets, Complet | stored words `structure`, `refs`, `subjects`, `full` stay as in config and CLI |
| Mermaid `Direction` | Direction | not "Sens"; values "De gauche à droite" / "De haut en bas" |
| merge strategy / tag on merge | Stratégie de merge / Tag au merge (local: Stratégie de fusion / Étiquette à la fusion) | `{ git, local }` pairs where the gender changes |
| Gerrit compare, submit, abandon | Comparer des patch sets, Soumettre le change, Abandonner le change | "change" and "patch set" stay English like the host |
