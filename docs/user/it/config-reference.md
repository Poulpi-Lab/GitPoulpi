[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · **Italiano** · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Riferimento della configurazione

Ogni chiave di `config.toml`, il suo valore predefinito e cosa fa. Raramente serve
modificare il file a mano: la [schermata Impostazioni](settings.md) (`Ctrl+,`) lo
scrive mentre cambi le opzioni, e lo fa anche `gitpoulpi config set`.

## Dove si trova e come viene scelto un valore

| | |
|---|---|
| File | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` lo stampa. |
| Un'altra cartella | `GITPOULPI_CONFIG_DIR=<dir>` sposta l'intera cartella di configurazione (il file, `sessions/`, i file delle porte del daemon): un'installazione portatile, un test, o un agente tenuto lontano dalle tue impostazioni. |
| Precedenza | valore predefinito integrato, poi `config.toml`, poi la variabile `GITPOULPI_*`, poi un flag da riga di comando. Vince il più alto. Un valore errato (una scelta scritta male nel file, una variabile non valida) è un errore, mai ignorato in silenzio. |
| Variabili | Metti la chiave in maiuscolo e trasforma i punti in trattini bassi: `general.auto_prune` diventa `GITPOULPI_GENERAL_AUTO_PRUNE`. Solo per le chiavi che GitPoulpi legge davvero (non quelle segnate *non usata* più sotto, né elenchi come `ui.log_columns_hidden`). |
| Flag | `--backend git2\|system` sostituisce `general.default_backend` per un daemon avviato da quel comando. |
| Token e chiavi | Mai in questo file. I token di hosting e le chiavi API dell'IA vanno nel credential helper del sistema (Git Credential Manager, libsecret / `pass`, Keychain); il file contiene solo un riferimento (`token_ref`). Senza un helper, il salvataggio di un token fallisce con un messaggio e il token dura solo per la sessione. |

```
gitpoulpi config path                          # il file, e se esiste già
gitpoulpi config get general.auto_stash        # il valore, il livello da cui viene (default/file/env/flag) e se qualcosa lo legge
gitpoulpi --json config get ui                 # un'intera sezione
gitpoulpi config set general.auto_stash always # verificato sullo schema, poi scritto
```

`config set` mantiene il file snello (solo le chiavi impostate da qualcuno) e
conserva le chiavi di cui non gli è stato chiesto nulla, ma elimina i commenti.
Rifiuta elenchi e tabelle intere, tranne `keybindings.*` (più combinazioni
separate da spazi; vuoto = nessuna associazione). Modifica `[[profiles]]`,
`[[integrations]]` e `[[ai.profiles]]` in Impostazioni o a mano.

Nelle tabelle, **Riavvio** significa che il valore viene letto all'avvio; tutto il
resto si applica subito (o al successivo aggiornamento, recupero o chiusura di
Impostazioni).

## `[general]`

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, con ripiego sul programma `git` per staging, merge, rebase, stash e rete) oppure `system` (solo il `git` installato). Un daemon in esecuzione mantiene il suo backend: **riavvialo** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Quando modifiche locali bloccano un checkout, pull, merge o rebase: `ask` (una finestra nella TUI; CLI e MCP falliscono e indicano `--autostash`), `always` (stash, esecuzione, pop) o `never` (rifiuta). Vedi [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | L'auto-stash include anche i file non tracciati. |
| `general.initial_commits_in_graph` | `5000` | Commit caricati per primi dal Log (almeno 500). Un valore più alto costa memoria e tempo di disegno su repository enormi. |
| `general.lazy_load_commits_in_graph` | `true` | Carica i commit più vecchi quando scorri vicino alla fine. Disattivato: il Log si ferma al numero iniziale. |
| `general.commits_per_lazy_page` | `1000` | Commit per pagina caricata in background (almeno 50). |
| `general.max_commits_in_graph` | `100000` | Dove si ferma il caricamento progressivo (circa 300 byte per commit); `0` = nessun limite. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All in background ogni N minuti, da 0 a 60; `0` = disattivato. **Riavvio.** |
| `general.auto_fetch_interval_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano l'intervallo: `minutes`, `hours` o `auto`. Solo visualizzazione: l'intervallo è sempre `auto_fetch_interval_minutes` minuti. `auto` (e una configurazione senza la chiave) sceglie l'unità più grande che divide esattamente i minuti (60 si legge «1 ora», 45 «45 minuti»); un'unità che non va bene viene ignorata. Le Impostazioni scrivono entrambe le chiavi; un GitPoulpi più vecchio ignora l'unità e legge ancora i minuti. |
| `general.fetch_on_tab_open` | `true` | Fetch All all'apertura di una scheda (saltato se GitPoulpi ha fatto il fetch di quel repository meno di 60 s prima). |
| `general.auto_prune` | `false` | I fetch eseguono anche il prune dei branch di tracciamento remoto scomparsi. |
| `general.auto_prune_tags` | `false` | Un fetch elimina anche i tag locali che il remote ha eliminato (solo i tag che erano su di esso; un tag che non hai mai inviato resta). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Aggiornamento degli elenchi di pull request / issue e del chip della barra degli strumenti, da 0 a 120; `0` = solo su richiesta e su eventi. |
| `general.hosting_refresh_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano l'aggiornamento: `minutes`, `hours` o `auto`, letta come `general.auto_fetch_interval_unit`. Solo visualizzazione. |
| `general.keep_submodules_up_to_date` | `false` | Dopo un checkout o un pull, `git submodule update --init --recursive` quando un sottomodulo non è allineato. Vedi [Sottomoduli](submodules.md). |
| `general.default_branch_name` | `"main"` | Primo branch di un repository creato con Init; vuoto lascia decidere a git. Non rinomina nulla. |
| `general.delete_orig_files_after_merge` | `false` | Dopo un merge andato a buon fine, elimina i file di backup `*.orig`. |
| `general.remember_tabs` | `true` | Riapre le schede dell'ultima sessione. Se disattivato, un avvio dentro un repository apre solo quel repository, e un avvio fuori da un repository apre solo Home. **Riavvio.** |
| `general.radar_enabled` | `true` | Il [Radar](radar.md): la sua scheda, i comandi di palette, `Alt+R` e il suo caricamento in background. Disattivato nasconde tutto e non si carica nulla per lui. Si applica subito; non compare in una build senza la funzione `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | La scheda su cui si apre un avvio quando la cartella da cui parte si trova dentro un repository: `"current-repo"` (la scheda di quel repository, aperta se serve), `"last"` (la scheda attiva all'ultima chiusura), `"home"` o `"radar"`. Un repository indicato con `--repo` apre sempre la propria scheda. `"last"` è Home con `remember_tabs` disattivato o se non c'è nulla di ricordato; `"radar"` è Home se il Radar è disattivato o assente dalla build. Letto una sola volta all'avvio; richiede un riavvio. |
| `general.startup_tab_outside_repo` | `"home"` | La scheda su cui si apre un avvio quando la cartella da cui parte non è un repository (ad esempio la cartella utente): `"home"`, `"last"` o `"radar"`. `"last"` è Home con `remember_tabs` disattivato o se non c'è nulla di ricordato; `"radar"` è Home se il Radar è disattivato o assente dalla build. Letto una sola volta all'avvio; richiede un riavvio. |
| `general.startup_tab_launcher` | `"last"` | La scheda su cui si apre un avvio quando GitPoulpi viene avviato dal collegamento sul desktop o nel menu Start che ha creato (passa `--launcher`), qualunque sia la cartella: `"home"`, `"last"` o `"radar"`. Ha la precedenza sulle due impostazioni qui sopra. `"last"` è Home con `remember_tabs` disattivato o se non c'è nulla di ricordato; `"radar"` è Home se il Radar è disattivato o assente dalla build. Letto una sola volta all'avvio; richiede un riavvio. |
| `general.group_worktree_tabs` | `true` | Tiene le schede dei worktree collegati di un repository subito dopo la scheda di quel repository: quando si apre un worktree, quando una scheda di worktree viene trascinata altrove (torna accanto al suo gruppo; la scheda del repository è la testa del gruppo e i suoi worktree la seguono) e quando l'impostazione viene attivata. Disattivato lascia le schede dove le metti. |
| `general.undo_depth` | `100` | Quante operazioni può annullare Annulla; `0` = illimitato. |
| `general.conflict_detection` | `true` | Rilevamento proattivo dei conflitti: un badge nel Log quando il merge del branch corrente nel branch da cui è partito andrebbe in conflitto, calcolato in background (un merge in memoria: nulla cambia nei tuoi file, nell'indice o nei ref). Un repository può disattivarlo per sé (`[preferences] conflict_detection`). I branch base con cui confronta hanno liste proprie: [`[conflict_prevention]`](#conflict_prevention-liste-dei-branch-base). Vedi [Risolvere i conflitti di merge](merge-conflicts.md#prevenzione-dei-conflitti). |
| `general.dev_mode` | assente (automatico) | Per chi compila GitPoulpi da un checkout; non compare mai nelle Impostazioni. Assente: attivo quando l'eseguibile in uso si trova in `<checkout>/target/<profile>/` (`Cargo.toml` e `crates/gitpoulpi-tui` accanto a `target`). `true` / `false` lo forza. La modalità sviluppatore sblocca solo `ui.launcher_build`. |
| `general.journal_outside` | `true` | Registra ciò che una shell o un agente IA fa a branch, tag, stash e HEAD come voci della cronologia annullamenti, segnate come esterne, così `Ctrl+Z` può annullarlo. Le modifiche ai file non vengono mai catturate. Richiede `undo_enabled`. Vedi [Annulla e ripeti](undo-redo.md#modifiche-fatte-fuori-da-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: nessuna registrazione per l'annullamento (il più veloce), la cronologia di annullamento viene cancellata, i pulsanti Annulla/Ripeti scompaiono. Vedi [Annulla e ripeti](undo-redo.md). |
| `general.active_profile` | `""` | Id del `[[profiles]]` con cui vengono creati i commit. Vuoto, o un profilo con nome o email vuoti: `user.name` / `user.email` di git. |
| `general.external_editor` | `""` | Comando per *Apri nell'editor esterno* di un file, ad es. `code -r {file}`. Vuoto: VS Code se installato, altrimenti il programma predefinito del sistema. |
| `general.external_diff_tool` | `""` | Comando per *Apri nello strumento diff esterno*: `{local}` (versione committata / HEAD), `{remote}` (versione della directory di lavoro / dell'indice). Vuoto: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Comando per *Apri nello strumento di merge esterno* su una scheda di conflitto (un file binario, LFS o troppo grande): `{base}`, `{local}` (A, corrente), `{remote}` (B, in arrivo) e `{merged}` (il file in cui lo strumento scrive il risultato; GitPoulpi lo mette in stage quando lo strumento termina con successo). Vuoto: `git mergetool --no-prompt`. |
| `general.file_explorer` | secondo l'OS | Comando per *Mostra nella cartella*: `{path}` (il file), `{dir}` (la sua cartella). Predefiniti: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

I tre modelli di comando vengono prima suddivisi in argomenti (`"..."` raggruppa
una parola), poi i segnaposto vengono sostituiti, così i percorsi con spazi
restano un solo argomento. Non interviene nessuna shell.

```toml
[[profiles]]                # identità dei commit (Impostazioni > Profili)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log e grafo**

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (un'immagine dove il terminale può disegnarla, caratteri altrove), `image`, `text`. **Riavvio.** |
| `ui.image_protocol` | `"auto"` | Come le immagini (grafo, avatar) arrivano al terminale: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protocollo Kitty in kitty, Ghostty e in un herdr con `kitty_graphics = true`; iTerm2 in WezTerm, iTerm2, Rio, mintty; sixel (solo avatar) in Windows Terminal, foot, Konsole, mlterm; nessuno in tmux, zellij, screen e terminali sconosciuti. **Riavvio.** |
| `ui.theme` | `"system"` | Il tema di colori: uno slug integrato (`gitpoulpi theme list` li elenca tutti), lo slug di un file in `<config>/themes/`, oppure `system` (segue lo sfondo chiaro o scuro del terminale, rilevato all'avvio; il vecchio valore `default` significa lo stesso). `config set` rifiuta i nomi sconosciuti. Vedi [Temi](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Palette delle corsie: `theme` (segue il tema) o il nome di una palette di corsie: `gitpoulpi`, `light`, `material`, `serie`, e le palette degli altri temi. |
| `ui.graph_lane_background` | `"off"` | `off` o `row_tint` (una leggera fascia del colore della corsia dietro il grafo, più una barra al suo bordo). **App: Attiva/disattiva tinta righe del grafo** (`Ctrl+N Shift+T`) lo commuta; **App: Cambia tema colori del grafo...** (`Ctrl+N Shift+Y`) sceglie `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (solo con `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` e `right` disegnano una colonna; una finestra sotto le 80 colonne torna a `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: se una finestra che hai spostato o ridimensionato (trascina la barra del titolo o i bordi, oppure `Ctrl+N m`) mantiene la sua posizione. Un'unica impostazione per tutte le finestre (ognuna conserva la propria posizione e dimensione): fino alla chiusura di GitPoulpi (predefinito), su disco per il prossimo avvio (tutte le finestre nell'unico `dialog_geometry.txt` nella cartella dei dati), oppure per niente (ogni finestra si apre nella posizione e con la dimensione predefinite). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: dove stanno i pulsanti lungo la barra (sinistra, centro, destra di una riga; alto, centro, basso di una colonna). |
| `ui.toolbar_pr_pinned` | `true` | Il pulsante della pull request è fissato all'estremità della barra degli strumenti; `false` lo lascia scorrere con gli altri pulsanti. |
| `ui.statusbar_position` | `"bottom"` | `top` (sopra le schede dei repository) o `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (l'ultima riga della finestra, sotto la barra di stato) o `left` (una colonna al bordo sinistro; sotto le 80 colonne resta in alto). |
| `ui.graph_show_stashes` | `true` | Una riga per ogni stash nel Log. |
| `ui.ref_connector` | `true` | Una linea sottile da un'etichetta di ref al suo commit. |
| `ui.column_resize` | `"line"` | Come si afferra il bordo di una colonna nel Log, nel Radar e nella colonna dei commit delle viste diff, file e blame. `line`: ogni bordo è uno spazio di una cella con un piccolo segno nell'intestazione; mentre il puntatore è sul bordo, nell'intestazione o in qualsiasi punto della tabella, viene disegnata una linea sottile d'accento su tutta l'altezza e un trascinamento da qualsiasi punto di essa ridimensiona. `header`: nessuno spazio e nessun segno; il primo carattere della cella d'intestazione dopo il bordo si accende e un trascinamento da lì ridimensiona (solo dall'intestazione). I tasti di dimensionamento da tastiera e le voci della palette sono gli stessi nelle due modalità. |
| `ui.ref_tree` | `true` | Raggruppa branch e tag in cartelle sul carattere `/` nella barra laterale Refs (`y` lo commuta). |
| `ui.graph_ghost_ref` | `true` | La colonna Ref del Log mostra una pillola sbiadita con il nome del branch (altrimenti il branch remoto) a cui appartiene un commit evidenziato, quando il commit non ha un ref proprio e la colonna ha spazio; mai un tag. Un branch eliminato viene dedotto dall'oggetto del commit di unione che lo ha unito e disegnato barrato. Passando sulla pillola se ne vede il tipo. |
| `ui.reveal_on_ref_click` | `true` | Un singolo clic su un ref (branch, tag, branch remoto, stash) nel pannello Refs scorre il Log fino al suo commit e fa lampeggiare la riga; la selezione del commit non si sposta. |
| `ui.log_wip_counts_in_message` | `true` | I conteggi delle modifiche WIP nel messaggio WIP mentre la colonna Changes è nascosta. |
| `ui.sidebar_sections_scope` | `"repository"` | Di chi è la scelta delle sezioni del pannello sinistro: `repository` (ogni repository ha la propria, con `ui.sidebar_sections_hidden` come valore predefinito) oppure `global` (ogni repository mostra `ui.sidebar_sections_hidden` e una modifica viene scritta lì). Passare dall'uno all'altro non fa perdere nessuno dei due insiemi: finché `global` è attivo, le scelte dei singoli repository restano nei loro file di stato. Impostazioni: *Sezioni della barra laterale*. |
| `ui.sidebar_sections_hidden` | vuoto | Le sezioni della barra laterale che hai nascosto (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), scritte dal menu del clic destro della barra o dalla palette. Almeno una sezione esistente resta sempre visibile. È il valore predefinito: ogni repository può nascondere le proprie sezioni (salvate nel suo file di stato) e questa chiave vale per un repository senza scelta propria; la scrive *Usa per tutti i repository*. |
| `ui.log_columns_layout` | `"fit"` | Come il Log dispone le sue colonne. `fit`: ogni colonna visibile sta esattamente nel pannello e solo `message` si adatta; quando il pannello è stretto cedono intere colonne. `scroll`: le colonne mantengono le loro larghezze (anche `message`), nessuna cede, e la tabella scorre lateralmente quando è più larga del pannello. |
| `ui.log_column_widths_scroll` | vuoto | Le larghezze delle colonne della disposizione `scroll`, nome -> celle (`message` ammesso, a differenza di `ui.log_column_widths`, che resta quella della disposizione `fit` e non contiene mai `message`). Scritte quando trascini, ridimensioni o adatti una colonna mentre la disposizione è `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | vuoto | Le colonne del Log (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), scritte quando nascondi, trascini o ridimensioni una colonna. `message` non può essere nascosta. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | vuoto | Le parti della [colonna dei commit](interface-tour.md#la-colonna-dei-commit) accanto a un diff, a sé stanti e separate da quelle del Log: le parti mostrate per nome (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; vuoto significa corsie, messaggio, autore e data), il loro ordine da sinistra a destra con gli stessi nomi (vuoto è l'ordine appena elencato; un nome omesso segue in quell'ordine) e le larghezze trascinate in celle per parte (una parte omessa è automatica), scritte quando scegli, sposti o dimensioni le parti. `message` non si può nascondere. Un file di sessione scritto prima che queste chiavi esistessero viene letto una volta al loro posto. |
| `ui.language` | `"auto"` | La lingua dell'interfaccia: `auto` (la lingua del sistema, altrimenti inglese), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Immediato.** `GITPOULPI_LANG` la sostituisce. Una lingua non presente in questa build mostra l'inglese. Anche **App: Cambia lingua...** (`Ctrl+N g`) la imposta, con anteprima in tempo reale. L'output di CLI e MCP resta in inglese. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... restano in inglese in ogni lingua. `true`: il vocabolario git della lingua scelta (in francese "remiser", "étiquette", "valider"...). **Immediato.** |
| `ui.git_terms_english` | `false` | Solo ja, ko e zh-CN: mostra le parole git inglesi (stage, unstage, staged, unstaged...) invece della loro trascrizione. Nessun effetto finché `ui.translate_git_terms` è `true`. **Immediato.** |
| `ui.mouse_enabled` | `true` | `false` lascia il mouse (e la selezione del testo) al terminale. **Riavvio.** |
| `ui.mouse_scroll_lines` | `3` | Righe del Log per scatto della rotella, da 1 a 20. **Riavvio.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` chiede al terminale il protocollo tastiera di kitty (così arrivano `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`); `off` non lo chiede mai. **Riavvio.** |
| `ui.logo_color` | `""` | Combinazione di colori del logo nella finestra Informazioni, uno slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); vuoto o sconosciuto significa il predefinito della build (`lavender`). Scritto quando clicchi il logo o scegli un colore in Informazioni. |

**Avatar** (disattivati per impostazione predefinita; attivarli invia a Gravatar un
hash SHA-256 delle email degli autori e interroga il tuo account GitHub/GitLab
collegato)

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (nel cerchio del commit del grafo a immagine), `author` (nella colonna Author), `off`. Il vecchio `ui.show_avatars` (`true`) vale ancora come `author` finché `ui.avatars` è assente. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Ordine di ricerca: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Un host GitLab viene interrogato solo tramite un profilo connesso. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (generata in locale) o `initials`. Su un terminale senza protocollo di immagini si usa sempre il badge con le iniziali. |

**Diff ed editor**

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` o `split` (`m` in un diff). |
| `ui.diff_hunks_only` | `true` | `false` mostra l'intero file con le modifiche al loro posto (`H`). |
| `ui.diff_show_whitespace` | `true` | Mostra le modifiche di soli spazi (`s`). |
| `ui.diff_word_wrap` | `false` | Manda a capo le righe lunghe del diff (`w`; Impostazioni > Diff ed editor). |
| `ui.minimap` | `false` | Una minimappa accanto alla barra di scorrimento del diff e dell'editor (`M`). |
| `ui.mermaid_detail` | `refs` | Dettaglio del Copia come Mermaid gitGraph del Log: `structure`, `refs`, `subjects`, `full` (ricordato dalla finestra; il default di `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Direzione di quel grafo: `lr` (da sinistra a destra) o `tb` (dall'alto in basso). |
| `ui.mermaid_fence` | `false` | Racchiude lo script Mermaid copiato in un blocco di codice `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Gli oggetti dei commit vengono tagliati a questo numero di caratteri negli id Mermaid (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` o `16`: con quanti colori disegnare. `auto` è truecolor tranne su Apple Terminal (256), sulla console Linux (16) e sui terminali `*-256color` senza `COLORTERM` (256). Immediato. Vedi [Temi](themes.md). |
| `ui.input_color` | `"theme"` | Sfondo dei campi di testo: `theme` (quello del tema), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | Il colore `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | Il compositore di commit IA si apre con i file di ogni proposta `expanded` (espansi) o `collapsed` (compressi). |
| `ui.blame_side` | `"right"` | Quale lato della vista Blame ospita la colonna di annotazioni (avatar, autore, età, riepilogo): `right` (predefinito) o `left`. |
| `ui.blame_width` | `46` | Larghezza della colonna di annotazioni di Blame in celle (da 24 a 90). Trascina il suo bordo o premi `<` / `>` nella vista; la nuova larghezza è salvata qui. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request su GitHub, merge request su GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Nasconde il chip della barra degli strumenti con l'ultima pull request o merge request del branch attivo quando è chiusa o ne è avvenuto il merge; al suo posto compare il chip attenuato «+ PR» (avvia una nuova richiesta), come se il branch non ne avesse una. `false`: il chip mostra sempre l'ultima richiesta, qualunque sia il suo stato. |

Le opzioni del diff che cambi dentro un diff vengono salvate qui subito e usate da
ogni diff aperto in seguito.

**Date**: `ui.date_format` (`"YYYY-MM-DD"`, la colonna data di Blame / Cronologia del file),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, la colonna Data del Log) e
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, i dettagli del commit, il
tooltip della data e *Copia > Data*). Token: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (giorno della settimana, esteso / abbreviato) `HH` `mm` `ss`; tutto
il resto viene copiato così com'è. I nomi dei mesi e dei giorni seguono
`ui.language`, e un mese scritto dopo il numero del giorno è in forma genitiva
dove la lingua ce l'ha (russo, ucraino, polacco, ceco). Una sequenza `D MMMM YYYY`
è scritta con l'anno per primo in giapponese, cinese e coreano (`2024年3月5日`).
`ui.date_locale` (`"system"`, il predefinito, oppure `"english"`) mantiene i nomi
in inglese qualunque sia la lingua.

**Agenti, terminali e cartella della scheda**

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | La finestra accetta del tutto `ui show` / `ui propose` dagli agenti (Impostazioni > Agenti e MCP > *Consenti agli agenti IA di pilotare la vista*). Disattivato, nessun agente la raggiunge. Vedi [MCP e agenti](mcp-and-agents.md#mostrare-le-cose-allutente). |
| `ui.launch` | `"off"` | Se un agente può aprire una finestra di GitPoulpi quando non ce n'è nessuna aperta: `off`, un preset (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) oppure `custom`. |
| `ui.launch_command` | `""` | Il modello `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Suddiviso in argomenti, senza shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (la riga delle schede offre *Esci e cd qui*) oppure `auto` (ogni uscita cambia la cartella della shell, e una nuova scheda esegue la voce *Apri in* predefinita). Vedi [Seguire la cartella della scheda](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Obsoleta, vedi sotto e [Apri in](open-in.md). Come il vecchio *Terminale qui* apriva un terminale: `auto` (il preset di `ui.launch`, altrimenti `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), un preset, oppure `custom`. |
| `ui.terminal_launch_command` | `""` | Il modello `custom`; `{repo}` è la cartella. |
| `ui.open_in` | *(assente)* | L'elenco [Apri in](open-in.md), un array di tabelle `[[ui.open_in]]` con `label` e `command`. Assente significa "non ancora configurato": l'elenco viene calcolato dalle vecchie chiavi `ui.terminal_launch*` (un valore diverso da `auto` diventa la prima voce), dal gestore file del sistema operativo, dai multiplexer e terminali rilevati dall'ambiente (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) e dagli editor trovati nel `PATH`. Il primo salvataggio da Impostazioni scrive l'elenco. `open_in = []` è un elenco vuoto. Non impostabile con `config set` (modifica il file). |
| `ui.open_in_default` | `""` | Etichetta della voce eseguita da "Apri nella voce predefinita"; vuoto o sconosciuto significa la prima voce. |
| `ui.launch_terminal` | `"auto"` | Il terminale in cui i collegamenti del desktop (Impostazioni > Launcher) e la riapertura su Windows avviano GitPoulpi. `auto` sceglie tra quelli installati (Windows: Windows Terminal, altrimenti WezTerm, Alacritty, la console semplice; Linux: `xdg-terminal-exec`, altrimenti il primo terminale trovato; macOS: iTerm2, Ghostty, WezTerm o kitty se installati, altrimenti Terminal), `default` è quello del sistema, oppure un id di terminale come `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Solo in modalità sviluppatore (vedi `general.dev_mode`; solo allora compare in Impostazioni > Launcher, altrimenti viene ignorato). `"current"`: i collegamenti e l'alias del comando avviano la build che li ha creati. `"newest"`: avviano la build più recente tra `target/dist` e `target/release` del checkout, decisa a ogni clic, e avviare l'una o l'altra build non li riscrive mai. |
| `ui.command_alias` | `""` | Il nome del comando che avvia GitPoulpi da un terminale, per esempio `gp` (Impostazioni > Launcher, `gitpoulpi alias create`). Vuoto: nessuno. Solo lettere, cifre, `-`, `_` e `.`. È solo il nome che usa il pulsante **Crea: alias del comando**; finché non lo crei non esiste nulla. |
| `ui.command_alias_add` | `""` | Il nome del secondo comando, per esempio `gpa`, che esegue `gitpoulpi --add`: apre la cartella corrente in una nuova scheda del GitPoulpi già avviato (Impostazioni > Launcher, `gitpoulpi alias create gpa --add`). Vuoto: nessuno. Stesse regole per il nome; creato insieme a `ui.command_alias` dal pulsante **Crea**. |
| `ui.relaunch_in_terminal` | `true` | Solo Windows: quando l'exe viene avviato con un doppio clic in una console tutta sua, GitPoulpi si riapre in `ui.launch_terminal` ed esce. Disattivato: resta in quella console. |
| `ui.reuse_window` | `true` | Un clic su un collegamento di GitPoulpi (`gitpoulpi launch`) mostra la finestra di un GitPoulpi già aperto invece di avviarne un altro. Disattivato: ogni clic avvia una nuova finestra. |

**I comandi di `ui.open_in`** vengono prima suddivisi in argomenti (spazi, `'...'`, `"..."`), poi le variabili vengono sostituite dentro ogni argomento, così una cartella con spazi resta un solo argomento; non interviene nessuna shell, a meno che il comando non ne avvii una (`sh -c '... "$GITPOULPI_CWD"'`). Variabili: `${cwd}` (la cartella della scheda), `${repo}` (la radice del repository), `${name}` (il nome base della cartella), `${branch}` (vuoto se HEAD è scollegato), `${web_url}` (la pagina web del remoto `origin` quando è un progetto GitHub o GitLab, altrimenti vuoto; una voce che la usa si rifiuta di partire se è vuota), `${env:NAME}` (vuoto se non impostata). Il vecchio `{repo}` significa ancora `${cwd}`. Qualsiasi altro `${...}` è un errore che lo nomina. Esempio:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Obsoleto: `ui.terminal_launch` e `ui.terminal_launch_command` sono ancora letti (popolano l'elenco finché `ui.open_in` è assente: `custom` diventa "Terminal (custom)" con `{repo}` riscritto in `${cwd}`, l'id di un preset diventa quel preset) ma sono superati da `ui.open_in`.

### Immagini

| Chiave | Predefinito | Significato |
|---|---|---|
| `ui.image_view` | `true` | Mostra come immagine un file binario che è un'immagine (PNG, JPEG, GIF, WebP, BMP, ...) nella vista Diff e nella Vista file. Disattivato: il semplice avviso di file binario. |
| `ui.image_view_max_mib` | `32` | Le immagini più grandi di questo valore (MiB) non vengono decodificate; compare invece una scheda informativa. |
| `ui.image_view_max_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano il limite: `mib`, `gib` (1024 MiB) o `auto`. Solo visualizzazione: il limite è sempre `image_view_max_mib` MiB, da 1 a 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Su un terminale senza protocollo per immagini: `halfblock` (caratteri a mezzo blocco colorati) o `card` (solo scheda informativa). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom da cui l'ingrandimento mostra pixel quadrati invece di sfumarli: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | I pannelli prima e dopo condividono lo stesso zoom e la stessa posizione di scorrimento. |
| `ui.image_editors` | *(assente)* | Gli editor di immagini di "Modifica in", un array di tabelle `[[ui.image_editors]]` con `label` e `command`. Assente significa "non ancora configurato": l'elenco sono gli editor rilevati su questa macchina (programmi nel `PATH`, installazioni Flatpak, cartelle di installazione note su Windows, `/Applications` su macOS), poi l'app predefinita del sistema. Il primo salvataggio da Impostazioni scrive l'elenco. `image_editors = []` è un elenco vuoto. Non impostabile con `config set` (modifica il file). |
| `ui.image_editor_default` | `""` | Etichetta dell'editor eseguito da "Modifica nell'editor predefinito"; vuoto o sconosciuto significa la prima voce. |

**I comandi di `ui.image_editors`** vengono prima suddivisi in argomenti (spazi, `'...'`, `"..."`), poi le variabili vengono sostituite dentro ogni argomento, così un percorso con spazi resta un solo argomento; non interviene nessuna shell. Variabili: `${file}` (il percorso assoluto dell'immagine), `${dir}` (la sua cartella), `${name}` (il nome del file), `${repo}` (la radice del repository, vuota per una copia temporanea), `${env:NAME}` (vuoto se non impostata). Qualsiasi altro `${...}` è un errore che lo nomina. Un'immagine che non è il file della directory di lavoro (la versione di un commit, l'indice, un file eliminato) viene aperta come **copia temporanea di sola lettura**, così un editor non può toccare il repository. Gli editor senza riga di comando (Affinity Photo dal Microsoft Store, alcune installazioni di Photoshop) non sono tra i preset: scrivi una voce personalizzata. Esempio:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Non ancora usate** (accettate, salvate, ma nulla le legge):
`ui.graph_colorization` (rimosse da Impostazioni;
ancora accettate al caricamento), `ui.date_word_format` (rimossa da Impostazioni;
ancora accettata al caricamento). Il vecchio `ui.diff_view_mode`
viene letto una volta come `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `editor.tab_size` | `4` | Colonne per tabulazione, da 1 a 8 (editor e diff). |
| `editor.eol` | `"auto"` | Fine riga scritta dall'editor integrato: `auto` (mantiene quella del file), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Colori del linguaggio nell'editor. |
| `editor.line_numbers` | `true` | Numeri di riga nell'editor e nei diff. |
| `editor.follow_selection` | `true` | Con l'editor aperto, selezionare un altro file nel pannello destro lo apre anche nell'editor, dopo aver chiesto se salvare le modifiche non salvate. Un file che non può essere modificato si apre nella vista abituale. Un file di un commit esce dall'editor e si apre nel suo diff. |

## `[merge]`

L'editor dei conflitti di merge ([Risolvere i conflitti di merge](merge-conflicts.md)). Lo mostra Impostazioni > Diff ed editor.

| Chiave | Predefinito | Significato |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | La larghezza, in colonne, dell'area dell'editor a partire dalla quale A e B vengono mostrati affiancati (le righe lunghe scorrono di lato: Shift+rotella, Shift+frecce). Più stretta, mostra un lato alla volta dietro delle schede. `40` è il valore minimo, `0` significa sempre affiancati (l'avviso sotto le 40 colonne resta). L'editor nasconde i pannelli Refs e Status mentre è aperto, quindi la sua area è di solito l'intera finestra. |

## `[git]`, `[credentials]`, `[ssh]`

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `git.pull_mode` | `"merge"` | Cosa fa Pull (pulsante, `Ctrl+G p`, `Git: Pull`): `merge` (fast-forward quando possibile), `ff_only`, `rebase`. Sempre passato a git in modo esplicito. Il vecchio `fetch_all` vale come `merge`. |
| `git.push_behavior` | `"direct"` | `direct` fa il push sul remoto predefinito; `ask` mostra prima il menu dei remoti. |
| `git.merge_fast_forward` | `"never"` | Cosa fa un merge che avvii tu (menu del branch, drag and drop, palette, `gitpoulpi merge`) quando il branch potrebbe avanzare con un fast-forward: `never` crea sempre un commit di merge (`--no-ff`), `allowed` fa il fast-forward quando possibile (il comportamento predefinito di git, vale `merge.ff`), `only` fa il fast-forward oppure rifiuta. La scelta propria di un repository (Impostazioni > Opzioni del merge, `merge_fast_forward` nel suo `repo-state.toml`) prevale; `--ff`, `--no-ff` e `--ff-only` della CLI prevalgono su entrambe. Immediata. |
| `git.default_encoding` | `"utf-8"` | Codifica per il testo che **non** è UTF-8 valido (contenuto dei file, blame, diff, messaggi di commit senza intestazione `encoding`): qualsiasi etichetta WHATWG (`windows-1252`, `shift_jis`, ...). Solo visualizzazione; lo staging non ricodifica mai. Un repository può sostituirla. |
| `git.lfs_enabled` | `true` | Mostra e gestisce Git LFS. Un repository può sostituirla. Vedi [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Prima di un pull o push di un repository che usa LFS, installa i suoi filtri LFS / l'hook pre-push (`git lfs install --local`, solo questo repository) se mancano. Vedi [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Non usate*: `core.autocrlf` / `core.eol` di un repository sono impostazioni di git stesso, modificate in Impostazioni > Git e codifica (o con `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Autenticazione HTTPS per fetch, pull, push, clone: `credential_helper` (quello di git), `prompt` (ignora gli helper; git chiede su un terminale, quindi fallisce nel daemon in background), `none` (solo remoti pubblici). I vecchi valori `encrypted_file` valgono come `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Tipo creato da *Genera coppia di chiavi*: `ed25519`, `rsa` (4096 bit) o `ecdsa` (NIST P-256); il file è `gitpoulpi_<tipo>`. |
| `ssh.private_key_path`, `ssh.public_key_path` | non impostato | Registrati da *Genera coppia di chiavi*. Quando `ssh.private_key_path` è impostato, fetch, pull, push e clone eseguono git con `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` fa lo stesso (ssh sceglie la chiave corrispondente dell'agente) quando il percorso privato è vuoto. Un `GIT_SSH_COMMAND`/`GIT_SSH` nell'ambiente o un `core.sshCommand` nella configurazione di git (il pin per repository, `gitpoulpi ssh-command set --key`) ha la precedenza. Letto prima di ogni trasferimento, quindi le modifiche valgono dal successivo. |

## `[[integrations]]` (hosting e issue tracker)

Scritto da Impostazioni > Integrazioni o da `gitpoulpi hosting integrations add`. Ogni
tipo ha un client (solo token personali, niente OAuth).

```toml
[[integrations]]
id = "corp"                         # ciò che accetta --id; il token è salvato sotto integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # la radice dell'API ne viene ricavata
token_ref = "integration:corp"      # un riferimento, mai il token
name = "Corp GitLab"                # nome visualizzato, facoltativo
account = "me@example.com"          # facoltativo, vedi sotto
enabled = true                      # false: ignorato nell'abbinamento degli host
```

Un server self-hosted può essere in semplice http: `url = "http://localhost:3000"` viene
mantenuto come scritto (lo schema e la porta arrivano a ogni richiesta, e un remoto su
quell'host corrisponde al profilo). Un token inviato in semplice http viaggia non cifrato,
quindi usa http solo su una rete locale o fidata.

`kind` accetta anche `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` e `gerrit`
(Gitea e Forgejo condividono un solo client; un profilo Forgejo ha per impostazione predefinita `https://codeberg.org`). Jira,
Trello, Linear, YouTrack e Redmine sono solo issue tracker; Gerrit è solo revisione (le sue modifiche sono le
pull request, non ha issue). Bitbucket Cloud non ha un issue tracker.

`account` è l'identità non segreta accanto al token, vuota e omessa per la
maggior parte dei tipi: l'email dell'account Atlassian per `jira_cloud` e
`bitbucket` (un account Bitbucket vuoto indica un token di accesso a workspace o
repository, inviato come Bearer), la chiave API per `trello`, il nome utente per `gerrit` (il token è la sua password
HTTP, inviata come HTTP basic). Il token resta
l'unico segreto. Per `azure_devops` l'`url` è
`https://dev.azure.com/<organization>`; un URL `https://<organization>.visualstudio.com`
viene letto come la stessa organizzazione. Per `azure_devops_server` è la radice
della collection, ad esempio `https://tfs.example.com/tfs/DefaultCollection`.

`user` (l'account connesso) e `ssh` (la scelta della chiave del profilo) vengono
scritti per te. Il token viene da `GITPOULPI_TOKEN_<ID>` (id in maiuscolo,
caratteri non alfanumerici come `_`), altrimenti dal token salvato, altrimenti dal
tuo credential helper per quell'host quando un solo profilo lo usa (solo profili
GitHub e GitLab: gli altri tipi non leggono mai il credential helper). Dettagli:
[Integrazioni di hosting e issue tracker](hosting-integration.md). La vecchia
tabella `[[hosting]]` è *non usata*.

Per `local_markdown` (una cartella di note Markdown come sorgente delle issue) `url` è la
cartella, relativa a ogni repository o assoluta, e non c'è alcun token. La sua tabella
facoltativa `[integrations.local]` imposta `statuses` (l'ordine delle colonne della bacheca,
predefinito `todo`, `doing`, `done`), `done_status` (predefinito `done`),
`reference_prefix` (`LOC` mostra `LOC-12`) e `me` (cosa significa «assegnata a me»).
Vedi [Issue locali e Obsidian](local-issues.md).

### `[integrations.radar]` (elenchi a livello di account)

Tabella facoltativa di un profilo di host di codice (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; non Bitbucket Cloud). Se assente,
ogni chiave è disattivata. Scritta da `gitpoulpi hosting integrations radar`; letta da
`gitpoulpi hosting radar` e dal Radar, e modificabile in Impostazioni > Integrazioni (il blocco Radar di un profilo connesso).

```toml
[integrations.radar]
review_requests = true    # richieste in attesa della mia revisione, ogni repository
open_requests = false     # ogni richiesta aperta dei miei namespace
include_drafts = false    # mantiene le richieste in bozza in questi elenchi
assigned_issues = true    # issue aperte assegnate a me, ogni repository
namespaces = ["acme"]     # organizzazioni / gruppi in cui cercare le richieste aperte
review_mode = "ask"       # ask | clone | api
```

| Chiave | Predefinito | Significato |
|---|---|---|
| `review_requests` | `false` | Richieste aperte che chiedono la tua revisione, che il repository sia clonato o no |
| `open_requests` | `false` | Ogni richiesta aperta del tuo namespace e di ogni organizzazione o gruppo di cui fai parte (o di `namespaces`), mai «tutto ciò che il token vede» |
| `include_drafts` | `false` | Le richieste in bozza sono escluse da questi elenchi a meno che questa chiave sia attiva |
| `assigned_issues` | `false` | Issue aperte assegnate a te, in ogni repository |
| `namespaces` | `[]` | Organizzazioni / gruppi in cui cercare per `open_requests`; vuoto significa il tuo namespace più ogni appartenenza (GitHub: al massimo 12 vengono cercati, GitLab: 10 gruppi e 10 progetti propri, Gitea, Forgejo e Gerrit: 10; Azure DevOps accetta nomi di progetto, Bitbucket Data Center chiavi di progetto, Gerrit prefissi di percorso di progetto) |
| `review_mode` | `"ask"` | Cosa fa la revisione di una richiesta il cui repository non è clonato: `ask`, `clone` (lo clona, poi rivede) o `api` (rivede tramite l'API dell'host) |

Le risposte vengono messe in cache per profilo in `radar-account.json`, accanto a
`config.toml`, per cinque minuti; il file è una cache, si può eliminare senza rischi.

### Lo store di fiducia (non in `config.toml`)

Ciò per cui ogni integrazione e ogni profilo IA con chiave API è stato considerato attendibile (il suo
indirizzo, l'id dell'account, una chiave di certificato fissata, se http in chiaro è stato consentito)
vive in `trust.toml` nella cartella dati privata riservata all'utente (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` su Windows, `~/.local/share/gitpoulpi/private/` su Linux). Non fa
volutamente parte di `config.toml`: chi può modificare la configurazione non deve poter ampliare la
fiducia modificandola. Cambialo con `gitpoulpi hosting integrations trust` e `gitpoulpi ai trust` (vedi
[il controllo di sicurezza](hosting-integration.md#il-controllo-di-sicurezza)); il file viene riscritto da
GitPoulpi, non a mano.

## `[mcp]` (cosa possono fare gli agenti IA)

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `mcp.allow_mutations` | `false` | Disattivato: gli strumenti MCP che cambiano qualcosa vengono rifiutati; le letture, `ui_*` e le proposte continuano a funzionare. `true` permette agli agenti di modificare il repository. |
| `mcp.mutation_allowlist` | `[]` | Non vuoto: solo questi nomi di strumento possono modificare qualcosa (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: rifiuta gli strumenti `ui_*` che muovono la tua finestra (non `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (la finestra mostra un chip da confermare), `auto` (applicato a meno che tu non sia occupato), `off`. `--ui-control` sul comando del server lo sostituisce per singolo agente. Vale anche per `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Consente `ui_propose_*` / `gitpoulpi ui propose`: finestre precompilate che confermi tu stesso. |
| `mcp.allow_history_purge` | `false` | Solo in `config.toml` (nessuna riga nelle Impostazioni). `true` elenca per gli agenti lo strumento `rewrite_purge` (richiede comunque `allow_mutations`). Disattivato, un agente non vede affatto lo strumento: un'eliminazione cancella per sempre la vecchia cronologia di una riscrittura, il suo annullamento e i suoi reflog. La CLI e la TUI non ne sono limitate. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` e `gitpoulpi-mcp` si rifiutano di avviarsi, con un messaggio che nomina questa chiave. Impostazioni > Agenti e MCP > *Server MCP*. Letto quando un agente avvia il server. |
| `mcp.bind` | `"127.0.0.1:8808"` | L'indirizzo su cui `--http` ascolta quando non ne dai uno (`--http ADDR` / `--bind` prevalgono). Solo loopback, salvo `--allow-remote`. Vuoto o il vecchio `"stdio"` significano il predefinito. Impostazioni > Agenti e MCP > *Porta del server MCP* modifica la porta e mantiene l'host. |
| `mcp.http_auth` | `true` | `--http` risponde solo a una richiesta con `Authorization: Bearer <token>` (il token di `gitpoulpi mcp token`). `false` rimuove il controllo; il server avvisa allora su stderr, e `--allow-remote` viene rifiutato. Impostazioni > Agenti e MCP > *Autenticazione HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (un certificato generato una volta nella cartella dati privata, con la sua impronta SHA-256 stampata all'avvio) o `files` (`tls_cert` e `tls_key`). Con TLS attivo, il server termina TLS da sé e non esiste alcun listener in chiaro. Impostazioni > Agenti e MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Catena di certificati PEM usata con `tls = "files"`. |
| `mcp.tls_key` | `""` | Chiave privata PEM usata con `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Dichiari che un tunnel o un reverse proxy davanti all'endpoint garantisce la cifratura. Non si può verificare dall'interno del server, quindi `--allow-remote` senza TLS richiede che sia impostato. Il tratto tra il proxy e il server viaggia in chiaro su quella rete. Impostazioni > Agenti e MCP > *Dietro un tunnel o un reverse proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Solo in `config.toml`. Disattivato, gli strumenti di stage MCP non offrono `allow_sensitive`, e uno stage che aggiungerebbe un file segnalato (vedi `[privacy]`) viene rifiutato con un messaggio che dice all'agente di chiedere a te. Attivo, un agente può passare `allow_sensitive`. La CLI e la TUI non sono soggette a questo limite: chiedono a te. |

`allow_mutations`, `mutation_allowlist` e `allow_ui` controllano solo il server
MCP; la CLI è la tua shell e non è soggetta a questi limiti. Vedi [MCP e
agenti](mcp-and-agents.md#permessi).

## `[ai]` (IA integrata, disattivata per impostazione predefinita)

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `ai.enabled` | `false` | L'interruttore. Disattivato, l'IA non compare da nessuna parte e non viene inviato nulla. |
| `ai.default_profile` | `"default"` | Id del profilo usato dalle azioni che non ne indicano uno. |
| `ai.global_instructions` | `""` | Testo aggiunto al prompt di ogni azione. Può usare `${repo}`, `${uilang}`, ... (vedi [IA > Variabili del prompt](ai.md#variabili-del-prompt)); lo stesso vale per le istruzioni per azione e per repository. |
| `ai.actions.<action>.profile` | `""` | Profilo per una sola azione; vuoto = il predefinito. Azioni: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Un'area di lavoro (`workspaces.toml`, `[workspace.ai_instructions]` e `[workspace.ai_overrides.<action>]`) e un repository possono aggiungere a queste istruzioni o sostituirle: vedi [IA](ai.md#istruzioni). |
| `ai.actions.<action>.instructions` | `""` | Testo aggiunto solo per quell'azione. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` o `no`: se l'agente locale di una bozza di revisione del codice lavora dentro il repository per quell'esecuzione, sostituendo il `workdir` del profilo IA. Un repository può impostare il proprio (`gitpoulpi ai review-context --repo`). Vedi [IA > Contesto della revisione](ai.md#contesto-della-revisione). |
| `ai.review.include_claude_md` | `true` | Fornisce a una bozza di revisione del codice il `CLAUDE.md` del repository (radice, letto sul branch di destinazione della richiesta; tagliato a 8 KB). |
| `ai.review.include_agents_md` | `true` | Lo stesso per `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Un file in più per una bozza di revisione del codice: un percorso relativo alla radice del repository, un percorso assoluto o un link a un file di un host di codice (letto tramite il profilo di integrazione corrispondente, altrimenti in modo anonimo). Tutti i file insieme vengono tagliati a 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Sostituisce ogni segreto che lo scanner riconosce in una richiesta (un token, un blocco di chiave privata, una password in un URL, un valore `password = ...` che sembra casuale) con `[REDACTED:<rule>]` prima che parta. Impostazioni > IA mostra un avviso rosso finché è disattivato. |
| `ai.privacy.exclude_sensitive_files` | `true` | Esclude da ciò che viene inviato il diff di ogni file che corrisponde a `privacy.sensitive_patterns`. Un agente locale che lavora nella cartella del repository può comunque leggere i file da sé: vedi [IA > Privacy](ai.md#privacy). |
| `ai.limits.conflict_bytes` | `40000` | Numero massimo di byte di un conflitto di merge (i suoi blocchi base, corrente e in entrata insieme) quando «Suggerisci una risoluzione» lo invia. Oltre, GitPoulpi non fallisce: avvisa e offre *Invia comunque* (solo questa richiesta), *Apri impostazione* e *Annulla*; un conflitto non viene mai tagliato. Il limite è una protezione di GitPoulpi, non del modello: alzalo quando il tuo modello ha spazio. Mantenuto tra 1000 e 50000000; modificabile in Impostazioni > IA > Limiti. Gli altri limiti qui sotto funzionano allo stesso modo e stanno tutti in quel gruppo. |
| `ai.limits.diff_bytes` | `60000` | Numero massimo di byte del diff che un messaggio di commit, una spiegazione, il nome di uno stash, la descrizione di una pull request, un piano di commit o una divisione di commit possono portare, e dell'intero diff di una revisione del codice. Oltre, nulla viene tagliato in silenzio: nella TUI *Oltre il limite IA* offre *Invia tutto* (solo questa richiesta, nulla tagliato), *Invia troncato* (la fine viene esclusa, come prima), *Apri impostazione* e *Annulla*; la CLI e MCP troncano e lo riportano in `truncated` (`--send-all` invia tutto). Mantenuto tra 1000 e 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Numero massimo di byte del diff di un file in una revisione del codice. Oltre, la stessa domanda e le stesse due risposte; *Invia troncato* accorcia quel file con una nota visibile. Stesso intervallo. |
| `ai.limits.review_text_bytes` | `4000` | Numero massimo di byte della descrizione della pull request in revisione. Oltre, *Invia troncato* taglia la descrizione con una nota `[text cut]`. Stesso intervallo. |
| `ai.limits.review_threads_bytes` | `6000` | Numero massimo di byte dell'elenco dei commenti di revisione già esistenti (dati perché il modello non li ripeta). Oltre, *Invia troncato* taglia l'elenco. Stesso intervallo. |
| `ai.limits.review_context_file_bytes` | `8192` | Numero massimo di byte di un file di contesto del progetto (`CLAUDE.md`, `AGENTS.md`, il file aggiuntivo) dato a una revisione del codice. Oltre, *Invia troncato* accorcia il file con una nota. Stesso intervallo. |
| `ai.limits.review_context_bytes` | `16384` | Numero massimo di byte di tutti i file di contesto insieme. Oltre, *Invia troncato* accorcia il file che lo supera ed esclude quelli dopo di lui. Stesso intervallo. |
| `ai.sent_preview.<action>` | `true` | Una chiave per azione (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): mostra «Controlla cosa viene inviato» prima che la richiesta parta. Il «Non mostrare più per ...» del dialogo scrive `false`; Impostazioni > IA > *Mostra cosa viene inviato prima di...* li riattiva. |

La tabella `[preferences.ai_review]` di un repository (nel suo `repo-state.toml`, o la voce di un repository non clonato in
`remote-reviews.toml`) accetta le stesse chiavi, tutte facoltative, più `no_extra_file = true`: nessun file in più per
quel repository, qualunque cosa dica `ai.review.extra_file`. Una chiave non impostata, un `extra_file` non impostato e un
`no_extra_file` mancante ereditano il valore globale; impostare `extra_file` su un file azzera l'interruttore e viceversa.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # vuoto = il predefinito del tipo
base_url = ""           # vuoto = il predefinito del tipo (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # dove è salvata la chiave API; mai la chiave
command = ""            # solo local_agent: il programma esatto ("claude", un alias, un wrapper, un percorso)
harness = ""            # solo local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; vuoto = dedotto da `command`
args = []               # solo local_agent, ad es. ["-p", "--model", "haiku"]
run_mode = "headless"   # solo local_agent: headless (predefinita) | terminal (la schermata interattiva dell'agente in un terminale virtuale)
terminal_args = []      # solo local_agent: gli argomenti della modalità terminal (args conserva quelli headless)
workdir = "default"     # solo local_agent: default | dedicated | repository (default = cartella dedicata in modalità terminal, la cartella di GitPoulpi in headless)
timeout_secs = 0        # solo local_agent: secondi prima che l'agente venga fermato; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # solo local_agent, facoltativo: variabili impostate solo per quel processo
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = una delle variabili di GitPoulpi stesso
```

`harness` sceglie i suggerimenti offerti da Impostazioni > IA (argomenti
headless, opzioni, flag e elenco dei modelli, variabili d'ambiente)
indipendentemente da `command`; entrambe le chiavi vengono omesse dal file quando
sono vuote. I valori d'ambiente non vengono mai registrati nei log e sono
nascosti in Impostazioni quando sembrano segreti.

Impostazioni > IA offre preset per `harness`/`command`/`args`/`env` (le CLI di agenti più comuni, con
i loro flag headless, il flag del modello e le variabili d'ambiente utili) e per `kind`/`base_url`/`model` (i
provider HTTP ospitati e locali); vedi [IA > Configurare](ai.md#configurare-lia).

Un profilo chiamato *Default* viene creato quando non ce n'è nessuno. I vecchi
file con `[[ai.presets]]` / `[ai.defaults]` vengono migrati al caricamento e
conservati in `[[ai.legacy_presets]]`. Per tutto il resto: [Assistenza IA](ai.md).

## `[repos]`, `[session]`

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `repos.clone_root` | `""` | Da dove parte la finestra Clona; vuoto = `%USERPROFILE%\source\repos` o `~/src`. |
| `repos.clone_start_source` | `"last"` | L'origine su cui si apre la finestra Clona: `last` (predefinito, o vuoto) = quella mostrata alla precedente chiusura, `url` = sempre Clona con URL, oppure l'id di un profilo di integrazione (`id` nella sua voce `[[integrations]]`). Un profilo scomparso, disattivato o non collegato ripiega su Clona con URL. L'ultima origine in sé è uno stato, conservato in `clone-source.toml` nella cartella di configurazione, non qui. |
| `repos.browse_root` | `""` | Dove si apre Sfoglia; vuoto = l'ultima cartella scelta, poi la home. |
| `repos.show_hidden` | `false` | Il selettore di cartelle mostra le cartelle nascoste (`Alt+H` lo commuta e lo salva). |
| `repos.show_siblings` | `false` | L'albero a sinistra del selettore di cartelle elenca anche le altre cartelle di ogni livello aperto, come una vera gerarchia (`Alt+S` lo commuta e lo salva). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Dove GitPoulpi conserva ciò che ricorda di un repository (filtri dei ref, stato di compressione, la personalizzazione personale del workflow, la cache dell'hosting): `in_git_dir` = `<cartella git>/gitpoulpi` (condivisa da tutti i worktree, non compare mai in `git status`), `in_worktree` = `<repo>/.gitpoulpi` (l'unico posto dove una squadra può fare il commit di `workflow.toml`), `user_data` = `<cartella dati del sistema>/repos/<nome>-<id>` (`%APPDATA%\GitPoulpi\data` su Windows, `$XDG_DATA_HOME/gitpoulpi` su Linux, `~/Library/Application Support/GitPoulpi` su macOS). Il vecchio valore `".gitpoulpi"` significa ancora `in_worktree`. Cambiarlo non sposta nulla all'indietro: i dati trovati in una vecchia `<repo>/.gitpoulpi` vengono spostati nel posto scelto la prima volta che servono. |

## `[log]` (il registro dell'applicazione)

Ciò che mostra la scheda «Applicazione» del Registro attività e ciò che cita «Copia diagnostica». Dettagli: [Quando qualcosa non funziona](troubleshooting.md#quando-qualcosa-non-funziona).

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `log.level` | `"info"` | La riga più dettagliata che viene registrata: `error`, `warn`, `info`, `debug` o `trace`. La casella del livello nel Registro attività la cambia (e la salva qui). |
| `log.retention_days` | `7` | Giorni di file di registro conservati, oggi compreso: da 1 a 3650, dieci anni (un valore più grande o più piccolo viene riportato nei limiti). È sempre un numero di giorni, qualunque sia l'unità in cui le Impostazioni lo mostrano. I file più vecchi vengono eliminati all'avvio di GitPoulpi. Il file di un giorno ha anche il tetto di `log.max_file_mb`, quindi il caso peggiore è circa `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano `log.retention_days`: `days`, `weeks`, `months`, `years` o `auto`. Solo visualizzazione, non cambia mai per quanto tempo i file vengono conservati. Una settimana vale 7 giorni, un mese 30 giorni (non un mese di calendario, così il numero di giorni è lo stesso in qualsiasi momento lo si applichi) e un anno 365 giorni. `auto`, e una configurazione scritta prima che esistesse questa chiave, sceglie l'unità più grande che divide esattamente i giorni (14 giorni si leggono «2 settimane», 45 giorni «45 giorni»). Un'unità che non divide i giorni (`retention_days = 10` con `weeks`) viene ignorata e si vedono i giorni. Le Impostazioni scrivono entrambe le chiavi; un GitPoulpi più vecchio ignora l'unità e legge ancora i giorni. |
| `log.max_file_mb` | `5` | Il file di un giorno viene ruotato in un unico backup `.old` a questa dimensione, quindi un giorno non occupa mai più del doppio. |
| `log.max_file_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano la dimensione: `mib`, `gib` (1024 MiB) o `auto`. Solo visualizzazione: la dimensione è sempre `max_file_mb` MiB, da 1 a 1024. |
| `log.timings` | `true` | La scheda Operazioni git del Registro attività mostra quanto è durata ogni operazione (`Push - 123 ms`: apertura di un repository, aggiornamento, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). Le durate sono misurate dalla finestra, dalla richiesta alla risposta del daemon, quindi comprendono l'andata e ritorno IPC. Disattivato, nasconde le durate e le voci delle operazioni riuscite. I fallimenti sono sempre elencati. |
| `log.extended` | `false` | «Registrazione estesa»: la scheda Operazioni git elenca anche le operazioni in background (l'aggiornamento periodico che interroga lo stato, l'auto-fetch) con le loro durate. Sono sempre scritte nel registro dell'applicazione a livello `debug`. |
| `log.commit_count` | `"system-git"` | Come il `caricati / totale` della barra di stato ricava il totale. `"system-git"` esegue `git rev-list --count` (un processo, il più veloce su una macchina normale: git legge il file commit-graph). `"libgit2"` conta nel processo con un revwalk di libgit2 e non avvia nulla: sceglilo quando ogni processo da riga di comando parte lentamente, di solito perché un antivirus con scansione in tempo reale li ispeziona tutti; su 50.000 commit richiede circa 200 ms in background invece di 12 ms, ma non viene avviato nessun processo. `"off"` non conta mai: la barra mostra solo i commit caricati. Si applica subito. Un conteggio filtrato per percorso usa sempre il git di sistema. |
| `log.use_mailmap` | `true` | Mostra ogni autore come lo nomina il `.mailmap` del repository (e `mailmap.file`) nel Log, in blame e nella cronologia del file: il daemon chiede a git `%aN`/`%aE` (backend di sistema) oppure applica da sé gli stessi file (backend libgit2), così la risposta è identica. I commit salvati non cambiano e nulla viene riscritto. La ricerca nel Log per autore corrisponde ai nomi come mostrati. Si applica subito; il Log rilegge entro il suo polling di 2 s. |

I file sono `gitpoulpi-YYYY-MM-DD.log` nella cartella di stato del sistema operativo (`~/.local/state/gitpoulpi/logs` su Linux; `<cache>\logs` su Windows e macOS; `<config>/cache/logs` con `GITPOULPI_CONFIG_DIR`). Token, password e intestazioni `Authorization` vengono oscurati prima che venga scritto qualsiasi cosa.

## `[search]` (la ricerca nell'area di lavoro)

Ciò che `gitpoulpi search` (e il suo strumento MCP) esamina. Dettagli: [Aree di lavoro e ricerca nella CLI](cli-reference.md#aree-di-lavoro-e-ricerca).

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `search.commit_limit` | `200000` | Commit (dal più recente, su ogni branch, branch remoto e tag) i cui messaggi l'indice di ricerca tiene in memoria per repository, circa 100 byte più il messaggio ciascuno. Una cronologia più grande viene tagliata lì e la risposta dice che è `partial`. `0` disattiva la ricerca dei commit (branch, tag, stash e worktree vengono comunque cercati). Letto dal vivo: la ricerca successiva rispetta una modifica. |

## `[rewrite]` (riscrittura della cronologia)

Ciò che `gitpoulpi rewrite` e il pannello di riscrittura della cronologia proteggono. Dettagli: [Riscrittura della cronologia](history-rewrite.md).

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branch (nomi, o pattern in cui `*` corrisponde a qualsiasi testo) che una riscrittura lascia stare quando sono già stati inviati, a meno che l'esecuzione non li elenchi (`--include-protected BRANCH`; `--include-all-protected` per il push; la casella per branch nel pannello). Si aggiungono alle basi che il `workflow.toml` di un repository marca come `protected`. Anche il push di una riscrittura li salta allo stesso modo. È una protezione di GitPoulpi stesso: la protezione dei branch dell'host decide comunque se il force push viene accettato. Letto dal vivo: la prossima anteprima rispetta una modifica. |
| `rewrite.external_scripts` | `["shell"]` | Quali linguaggi di script esterni la regola Script può usare: uno qualsiasi tra `"shell"` (bash o sh; pwsh o powershell per gli script PowerShell), `"python"`, `"lua"` (un Lua installato; il Lua integrato è sempre attivo e non richiede voci). **Uno script esterno non è isolato: viene eseguito con i tuoi permessi.** Un linguaggio non elencato è in grigio nel pannello e rifiutato (`scripts_disabled`) da CLI, MCP e daemon. Anche se elencato, uno script esterno che arriva dal repository o da una ricetta condivisa viene eseguito solo dopo che lo hai ritenuto attendibile (`gitpoulpi rewrite script trust FILE`, o la domanda nel pannello); l'attendibilità è legata all'hash del contenuto dello script, quindi uno script modificato chiede di nuovo. Attiva Python modificando `config.toml` (un elenco non si può impostare con `config set`): `[rewrite]` poi `external_scripts = ["shell", "python"]`. Letto dal vivo. |
| `rewrite.script_timeout_ms` | `5000` | Quanto tempo ha uno script esterno per rispondere a un commit. Allo scadere l'intero albero di processi viene terminato e la riscrittura fallisce (`script_failed`, con il nome del commit); nulla si è spostato. `0` vale come il predefinito. Letto dal vivo. |
| `rewrite.script_timeout_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano il limite di tempo: `milliseconds`, `seconds` o `auto`. Solo visualizzazione: il limite è sempre `script_timeout_ms` millisecondi. Le Impostazioni accettano da 1 a 600000 (dieci minuti); `0` vale ancora come il predefinito. Letto dal vivo. |
| `rewrite.script_total_timeout_s` | `3600` | Quanto può durare in tutto un passaggio di uno script esterno. `0` vale come il predefinito. Letto dal vivo. |
| `rewrite.script_total_timeout_unit` | `"auto"` | L'unità in cui le Impostazioni mostrano il limite di tempo: `seconds`, `minutes`, `hours` o `auto`. Solo visualizzazione: il limite è sempre `script_total_timeout_s` secondi. Le Impostazioni accettano da 1 a 86400 (un giorno); `0` vale ancora come il predefinito. Letto dal vivo. |

## `[updates]` (la ricerca di una release più recente)

Cosa fa il chip della versione nella barra del titolo per le release più recenti. Dettagli: [Aggiornamenti](updates.md).

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` chiede a GitHub una volta al giorno, in background, se esiste una release più recente, e trasforma il chip della versione in `vX.Y.Z disponibile`. `"auto"` scarica e prepara da sé anche l'aggiornamento quando questa build viene da un archivio di release (chiede comunque prima di riavviare); per ogni altra build si comporta come `"notify"`. `"off"` non fa mai nessuna richiesta. Si applica subito. |
| `updates.channel` | `"stable"` | `"stable"` considera solo le release complete; `"prerelease"` propone anche le release candidate (`-rc.1`). Si applica subito. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | Il `proprietario/nome` su github.com di cui si controllano le release. Finché quel repository è privato GitHub risponde 404 a meno che per github.com sia memorizzato un token (Impostazioni > Integrazioni); senza, la ricerca fallisce in silenzio e scrive una riga nel registro dell'applicazione. |

Un amministratore può impostare `mode`, `channel` (come `release_channel`) e `repo`, più alcune altre opzioni di aggiornamento, in un file di policy valido per tutta la macchina o nel registro di Windows. Una chiave impostata dalla policy prevale su `config.toml` e compare nelle Impostazioni come **Impostato dal tuo amministratore**. `mode` nella policy è un tetto: puoi scegliere un valore più basso, mai più alto. Vedi [Distribuzione aziendale](enterprise-distribution.md#il-file-di-policy-e-il-registro-di-sistema).

## `[privacy]` (file sensibili)

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Pattern di percorso in stile gitignore. Un pattern senza `/` corrisponde al nome del file a qualsiasi profondità, uno con `/` all'intero percorso (un `/` iniziale lo ancora alla radice); `*` e `?` restano dentro una cartella, `**` attraversa le cartelle; un `!` riconsente ciò che un pattern precedente aveva trovato e vince l'ultima corrispondenza; le maiuscole vengono ignorate. Usato dalla protezione dello stage e da `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Mettere in stage un file che corrisponde a un pattern, o il cui contenuto contiene un segreto riconosciuto, chiede prima conferma (TUI), viene rifiutato a meno che sia dato `--allow-sensitive` (CLI) o a meno che `mcp.allow_sensitive_staging` lasci passare `allow_sensitive` all'agente (MCP). Vengono letti i file di testo fino a 1 MiB ciascuno e 32 MiB in totale; i file binari sono giudicati solo dal nome. Vedi [Stage e commit](staging-and-committing.md#file-sensibili). |

## `[conflict_prevention]` (liste dei branch base)

I branch che il rilevamento proattivo dei conflitti può prendere come base del branch corrente sono una lista di pattern (vedi [Risolvere i conflitti di merge](merge-conflicts.md#prevenzione-dei-conflitti)). Le liste possono avere un nome ed essere condivise; Impostazioni > Prevenzione dei conflitti le modifica.

| Chiave | Predefinito | Valori ed effetto |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Il nome della lista che un repository usa quando non ne ha una propria. Vuoto, o un nome che nessuna lista ha, significa il `**` incorporato (ogni branch). Riletto a ogni controllo, quindi una modifica si applica ai repository aperti senza riavvio. |
| `conflict_prevention.lists` | nessuna | Le liste con nome, una tabella ciascuna: `[conflict_prevention.lists.release]` con `patterns = ["origin/release/**", "!origin/release/old"]`. Un `patterns` vuoto ammette ogni branch. Modificare una lista si applica a ogni repository che la segue. |

La lista che un repository segue sta nel suo `repo-state.toml` (qui sotto). Precedenza: i pattern propri del repository, poi la lista con nome che segue, poi `default_list`, poi `**`.

## `[keybindings]`

Riassegna le scorciatoie della TUI; le modifichi in Impostazioni > Tastiera e mouse.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # predefinito Ctrl+G
nav_prefix = "Alt+n"             # predefinito Ctrl+N
settings   = ["Ctrl+,", "F2"]    # un elenco = alternative
tab_menu   = []                  # senza associazione

[keybindings.git_prefix]         # il tasto premuto dopo il prefisso git
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; un valore sostituisce i
predefiniti di quell'azione, rimuovere la riga li ripristina. Combinazioni:
`Ctrl+Shift+N`, `Alt+n`, `F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Una
voce errata non blocca mai nulla: la TUI la ignora, mantiene il predefinito e lo
segnala nella barra di stato. Contesti, id delle azioni, predefiniti e regole sui
conflitti: [Scorciatoie da tastiera](keybindings.md).

## Impostazioni per repository: `.gitpoulpi/repo-state.toml`

Conservato dentro il repository; la cartella si ignora da sola (un `.gitignore`
con `*`), quindi non compare mai come modifica. Scritto dalla TUI e dalla CLI;
raramente lo modifichi.

| Chiave | Cos'è | Si imposta con |
|---|---|---|
| `[preferences] encoding` | Sostituisce `git.default_encoding` qui | Impostazioni > Git e codifica |
| `[preferences] lfs_enabled` | Sostituisce `git.lfs_enabled` qui | Impostazioni > Git e codifica |
| `[preferences] profile` | L'id del `[[profiles]]` che firma i commit qui; assente significa `general.active_profile` | il combobox del profilo nella barra degli strumenti, **Repository: Cambia profilo...** |
| `[preferences] autocrlf`, `eol` | *Non ancora usate* | Impostazioni > Git e codifica |
| `[preferences] push_after_commit` | Fa il push del branch verso il suo upstream dopo ogni commit (disattivato se assente) | Impostazioni > Impostazioni specifiche del repository > Commit |
| `[preferences] skip_hooks_by_default` | La casella Salta gli hook del modulo di commit parte selezionata (disattivata se assente) | Impostazioni > Impostazioni specifiche del repository > Commit |
| `[preferences] squash_merge` | Il merge di un branch locale fa uno squash: mette in stage, nessun commit di merge (disattivato se assente) | Impostazioni > Impostazioni specifiche del repository > Opzioni del merge |
| `[preferences] template_apply`, `template_strip_comments` | Precompila un modulo di commit vuoto con il modello; rimuove le righe di commento `#` al momento del commit (entrambi disattivati se assenti) | Impostazioni > Impostazioni specifiche del repository > Modello di messaggio |
| `[preferences] conflict_detection` | Rilevamento proattivo dei conflitti per questo repository; se assente segue `general.conflict_detection` | Impostazioni > Impostazioni specifiche del repository > Prevenzione dei conflitti |
| `[preferences] conflict_base_branches` | I pattern dei Branch base propri del repository, uno per riga (ancora usati come **Lista propria**) | Impostazioni > Impostazioni specifiche del repository > Prevenzione dei conflitti |
| `[preferences] conflict_base_source` | Da dove vengono i Branch base: `default` (la lista predefinita globale), `list` (`conflict_base_list`) oppure `own` (`conflict_base_branches`). Se assente: i pattern propri quando ce ne sono, altrimenti la lista predefinita | Impostazioni > Impostazioni specifiche del repository > Prevenzione dei conflitti |
| `[preferences] conflict_base_list` | Il `[conflict_prevention.lists.<nome>]` che questo repository segue quando la sorgente è `list`; una lista eliminata ripiega sulla lista predefinita | Impostazioni > Impostazioni specifiche del repository > Prevenzione dei conflitti |
| `commit-template.txt` (un file accanto a `repo-state.toml`) | Il modello di commit; il `commit.template` di git punta a lui | Impostazioni > Impostazioni specifiche del repository > Modello di messaggio |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: le istruzioni IA di questo repository | Impostazioni > Impostazioni specifiche del repository > IA |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` per una singola azione qui | Impostazioni > Impostazioni specifiche del repository > IA |
| `default_remote` | Il remoto usato da Push/Pull/Fetch con un clic | menu dei remoti, `gitpoulpi remote default <name>` |
| `hosting_integration` | Il profilo `[[integrations]]` fissato qui | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Il progetto del provider che gli elenchi di PR / issue leggono, come `{ integration, project, filter }`. `filter` è facoltativo: una clausola extra nativa del provider (un frammento JQL di Jira, una condizione WIQL di Azure, nomi di liste di Trello) aggiunta in AND alla query dell'elenco; GitHub e GitLab la ignorano. | sezioni della barra laterale, `gitpoulpi hosting link` |
| `ref_visibility` | Ref nascosti / in solo | barra laterale Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remoti nascosti dal grafo per intero (ogni branch, anche quelli recuperati in seguito) | l'occhio e il menu del remoto, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Sezioni della barra laterale e cartelle di branch e tag compresse. `[folds]` contiene anche `status_folders` (cartelle compresse degli alberi di file non in stage / in stage) e `commit_folders` (cartelle compresse dell'albero dei file modificati di un commit, lo stesso per ogni commit), ciascuno limitato a 500 percorsi, scartando prima i più vecchi | la barra laterale, gli alberi di file |
| `pending_auto_stash` | Uno stash che GitPoulpi ha creato e non ha ancora rimesso a posto | scritto dal daemon; vedi [Auto-stash](log-and-history.md#auto-stash) |

## Altre variabili d'ambiente

| Variabile | Effetto |
|---|---|
| `GITPOULPI_CONFIG_DIR` | La cartella di configurazione (vedi sopra). |
| `GITPOULPI_TOKEN_<ID>` | Un token di hosting per il profilo `<ID>`, con precedenza su quello salvato (CI, uso occasionale). |
| `GITPOULPI_GIT` | L'eseguibile `git` da lanciare (predefinito: `git` nel `PATH`; su Windows il vero `mingw64\bin\git.exe` dietro lo shim `cmd\git.exe` di Git for Windows, che risparmia un processo per chiamata). |
| `GITPOULPI_LANG=fr` | La lingua dell'interfaccia, con precedenza su `ui.language` (qualsiasi valore di `ui.language`; `qps` è una pseudo-lingua per i traduttori). Un valore sconosciuto o vuoto viene ignorato. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Nessuna immagine, qualunque cosa dica la configurazione. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Suggerimenti sulla risoluzione delle immagini (dimensione della cella in pixel; sovracampionamento, predefinito 2, `1` risparmia CPU). |
| `GITPOULPI_NO_BROWSER=1` | Non avvia mai un browser web (l'URL viene comunque mostrato). |
| `GITPOULPI_TIMING=1` | Scrive i tempi delle richieste in `gitpoulpi-timing.log` nella cartella temporanea (rapporti sulle prestazioni). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Non attende `Enter` dopo l'uscita della TUI per un errore. |
