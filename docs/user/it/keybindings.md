[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · **Italiano** · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Scorciatoie da tastiera

Questa pagina elenca i tasti **predefiniti**. Puoi riassegnarne la maggior
parte. L'app mostra sempre i tasti in vigore: nella palette dei comandi, nell'aiuto
`?`, nelle barre di stato, nei menu contestuali e nei tooltip dei pulsanti.
Quando sposti un tasto, questi punti mostrano il tasto nuovo. Per il mouse, vedi
[Mouse e gesti](mouse-and-gestures.md).

## Riassegnare i tasti

Apri **Impostazioni > Tastiera e mouse**. La pagina è descritta in
[Impostazioni](settings.md#tastiera-e-mouse). Le modifiche si applicano subito e
sono salvate nella tabella `[keybindings]` di `config.toml`. Puoi anche
modificare quella tabella a mano:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

La sintassi, le regole sui conflitti e gli avvisi sono nel
[Riferimento della configurazione](config-reference.md) alla voce `[keybindings]`.
In breve:

- Un errore viene segnalato nella barra di stato e il tasto predefinito resta.
- Due azioni sulla stessa combinazione nello stesso contesto vengono rifiutate.
- Un prefisso richiede `Ctrl` o `Alt`.
- Una combinazione di un pannello che nasconde una combinazione globale dà solo
  un avviso.

La tabella qui sotto elenca ogni tasto che puoi riassegnare. Alcuni tasti sono
fissi e non si possono riassegnare: il movimento nelle liste (`j` `k` `h` `l`, le
frecce, `PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, le cifre
(il prefisso di conteggio), i tasti di modifica del testo e i tasti `Enter` /
`Tab` / frecce dei popup. La palette ha `App: Scorciatoie da tastiera...` e
`Scorciatoie: Reimposta tutto...`.

## Scorciatoie predefinite

Ogni tasto che puoi riassegnare, con l'id da usare in `[keybindings.<contesto>]`
e il suo valore predefinito. La tabella è generata dall'app (un test la tiene
aggiornata). I tasti dopo un prefisso sono mostrati con il prefisso (`Ctrl+G s`).

<!-- keymap:begin -->
| Contesto | ID azione | Cosa fa | Tasti predefiniti |
|---|---|---|---|
| `global` | `git_prefix` | Prefisso delle azioni Git | `Ctrl+G` |
| `global` | `nav_prefix` | Prefisso di navigazione | `Ctrl+N` |
| `global` | `palette` | Palette dei comandi | `Ctrl+P` |
| `global` | `settings` | Impostazioni | `Ctrl+,` |
| `global` | `home` | Home (gestione repository) | `Ctrl+O` |
| `global` | `clone` | Clona repository | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nuova scheda | `Alt+T` |
| `global` | `workspace_switch` | Cambia area di lavoro | `Alt+W` |
| `global` | `radar` | Apri Radar | `Alt+R` |
| `global` | `workspaces` | Apri aree di lavoro | `Alt+J` |
| `global` | `tab_menu` | Menu della scheda | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Apri in (elenco) | `Alt+O` |
| `global` | `tab_open_in_default` | Apri nella voce predefinita | `Ctrl+Alt+O` |
| `global` | `close_tab` | Chiudi scheda | `Ctrl+W` |
| `global` | `next_tab` | Scheda successiva | `Ctrl+PageDown` |
| `global` | `prev_tab` | Scheda precedente | `Ctrl+PageUp` |
| `global` | `help` | Aiuto | `?` |
| `global` | `quit` | Esci | `q` |
| `global` | `refresh` | Aggiorna | `r` |
| `global` | `refresh_all` | Aggiorna (fetch, pull request, issue) | `F5` |
| `global` | `find_in_log` | Cerca nel grafo dei commit | `f` |
| `global` | `workspace_search` | Cerca nell'area di lavoro | `Ctrl+Alt+F` |
| `global` | `select_tab` | Seleziona scheda | `t` |
| `global` | `undo` | Annulla l'ultima operazione | `Ctrl+Z` |
| `global` | `redo` | Ripeti l'operazione annullata | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Apri la cronologia annullamenti | `Alt+Z` |
| `global` | `commit_column` | Mostra o nascondi la colonna dei commit | `Alt+C` |
| `global` | `undo_offer` | Ripristina il branch dove l'aveva lasciato l'azione bloccata | `Alt+Shift+Z` |
| `global` | `context_menu` | Menu contestuale | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Scrivi il messaggio di commit | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Esegui ora il commit delle modifiche in stage | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | IA: dividi le modifiche in commit | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | IA: spiega le modifiche | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash con messaggio | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | IA: scrivi il messaggio di commit | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage di tutte le modifiche | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Scarta tutte le modifiche | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage del file selezionato | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage del file selezionato | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Apri la pull request collegata | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push di tutti i tag | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch di tutti i remoti | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Crea un tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nuovo branch | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash delle modifiche | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick del commit selezionato | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert del commit selezionato | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Modifica il messaggio del commit | `Ctrl+G e` |
| `git_prefix` | `amend` | Modifica il commit selezionato | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Modifica l'autore del commit selezionato | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (mixed) al commit selezionato | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase sul branch selezionato | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop dello stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Elimina il ref selezionato o fai il drop di uno stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Apri il menu LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Apri il menu Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch di tutti gli oggetti | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout di tutti i file | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull degli oggetti | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push di tutti gli oggetti | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Pulisci gli oggetti locali | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Apri il pannello di riscrittura della cronologia | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Comprimi o ripristina il pannello Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Comprimi o ripristina il pannello Stato | `Ctrl+N l` |
| `nav_prefix` | `settings` | Apri le impostazioni | `Ctrl+N ,` |
| `nav_prefix` | `about` | Apri Informazioni | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Apri il Registro attività | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Apri il registro attività globale | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Mostra le attività in background | `Ctrl+N Shift+J` |
| `nav_prefix` | `help` | Mostra o nascondi l'aiuto | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Seleziona scheda | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Cambia tema | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Cambia lingua | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Cambia tema colori del grafo | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Attiva/disattiva tinta righe del grafo | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Ingrandisci la sezione Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Riduci la sezione Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Ingrandisci il blocco del messaggio di commit | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Riduci il blocco del messaggio di commit | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Allarga il pannello Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Restringi il pannello Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Allarga il pannello Stato | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Restringi il pannello Stato | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agente: mostra la vista in sospeso | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agente: indietro (o chiudi la vista in sospeso) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filtra i Refs per nome | `/` |
| `refs` | `solo` | Solo sulla selezione | `o` |
| `refs` | `clear_solo` | Termina il solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Attiva o disattiva la vista gerarchica dei branch | `y` |
| `refs` | `show_all` | Mostra tutti i Refs nascosti | `Shift+H` |
| `refs` | `reveal` | Mostra nel grafo | `g` |
| `refs` | `mark_drop` | Segna come origine del rilascio | `m` |
| `refs` | `drop_marked` | Rilascia qui il ref segnato | `Shift+M` |
| `refs` | `expand_all` | Espandi tutte le cartelle | `=` |
| `refs` | `collapse_all` | Comprimi tutte le cartelle | `-` |
| `refs` | `expand_below` | Espandi la cartella e le sue sottocartelle | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Comprimi la cartella e le sue sottocartelle | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | File: percorso o albero | `m` |
| `log` | `view_all_files` | File: vedi tutti i file | `a` |
| `log` | `compare_selection` | Confronta le righe selezionate | `v` |
| `log` | `compare_swap` | Inverti gli estremi del confronto | `x` |
| `log` | `compare_next` | Vai alla riga confrontata successiva | `n` |
| `log` | `pick_column` | Scegli una colonna da spostare o ridimensionare | `Shift+C` |
| `log` | `copy_mermaid` | Copia i commit come Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Scegli la colonna successiva | `Tab` |
| `log_columns` | `prev` | Scegli la colonna precedente | `Shift+BackTab` |
| `log_columns` | `move_left` | Sposta la colonna a sinistra | `<` |
| `log_columns` | `move_right` | Sposta la colonna a destra | `>` |
| `log_columns` | `wider` | Allarga la colonna | `]` |
| `log_columns` | `narrower` | Restringi la colonna | `[` |
| `log_columns` | `reset_width` | Reimposta le larghezze delle colonne | `0` |
| `log_files` | `collapse_all` | Comprimi o espandi tutte le cartelle | `z` |
| `log_files` | `filter` | Filtra i file modificati | `/` |
| `log_files` | `expand_all` | Espandi tutte le cartelle | `=` |
| `log_files` | `collapse_all_folders` | Comprimi tutte le cartelle | `-` |
| `log_files` | `expand_below` | Espandi la cartella e le sue sottocartelle | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Comprimi la cartella e le sue sottocartelle | `_`, `Shift+Left` |
| `status` | `compose` | Porta il focus sul messaggio di commit | `c` |
| `status` | `toggle_file_mode` | File: percorso o albero | `m` |
| `status` | `view_all_files` | File: vedi tutti i file | `a` |
| `status` | `amend` | Attiva o disattiva amend | `Shift+A` |
| `status` | `skip_hooks` | Attiva o disattiva «Salta gli hook» | `Shift+H` |
| `status` | `collapse_all` | Comprimi o espandi tutte le cartelle | `z` |
| `status` | `stage_toggle` | Stage o unstage della riga selezionata | `Ctrl+Space` |
| `status` | `open_diff` | Apri il diff del file selezionato | `d` |
| `status` | `expand_all` | Espandi tutte le cartelle | `=` |
| `status` | `collapse_all_folders` | Comprimi tutte le cartelle | `-` |
| `status` | `expand_below` | Espandi la cartella e le sue sottocartelle | `*`, `Shift+Right` |
| `status` | `collapse_below` | Comprimi la cartella e le sue sottocartelle | `_`, `Shift+Left` |
| `home` | `favorite` | Aggiungi o togli il repository dai preferiti | `f`, `*` |
| `home` | `close_repo` | Chiudi, rimuovi o togli dai preferiti la riga | `x`, `Delete` |
| `home` | `browse` | Sfoglia per trovare un repository | `b` |
| `home` | `clone` | Clona un repository | `c` |
| `home` | `init` | Inizializza un repository | `i` |
| `home` | `collapse_all` | Comprimi tutte le sezioni | `-` |
| `home` | `expand_all` | Espandi tutte le sezioni | `+`, `=` |
| `home` | `search` | Cerca nei repository | `/` |
| `home` | `workspace_new` | Nuova area di lavoro | `n` |
| `home` | `workspace_edit` | Modifica l'area di lavoro | `e` |
| `home` | `workspace_from_tabs` | Nuova area dalle schede aperte | `a` |
| `home` | `workspace_fetch` | Fetch tutto nell'area di lavoro | `Shift+F` |
| `home` | `workspace_pull` | Pull tutto nell'area di lavoro | `Shift+P` |
| `home` | `workspace_clone_missing` | Clona i repository mancanti | `m` |
| `home` | `workspace_locate` | Individua il repository mancante | `l` |
| `refs` | `hosting_filter` | Pull request / Issue: scegli il filtro | `v` |
| `refs` | `hosting_edit` | Pull request: modifica sull'host | `e` |
| `refs` | `new_pull_request` | Pull request: creane una | `n` |
| `refs` | `add_remote` | Remoti: aggiungi un remoto | `+` |
| `diff` | `close` | Chiudi il diff | `q` |
| `diff` | `edit` | Modifica nella directory di lavoro | `e` |
| `diff` | `lfs_raw` | LFS: scheda o puntatore grezzo | `r` |
| `diff` | `copy` | Copia la selezione o la riga | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage o unstage dell'hunk | `Space` |
| `diff` | `stage_lines` | Stage o unstage delle righe selezionate | `Enter` |
| `diff` | `discard_hunk` | Scarta l'hunk | `x` |
| `diff` | `discard_lines` | Scarta le righe selezionate | `Shift+X` |
| `diff` | `select_lines` | Avvia o annulla una selezione di righe | `v` |
| `diff` | `next_hunk` | Hunk successivo | `n` |
| `diff` | `prev_hunk` | Hunk precedente | `p` |
| `diff` | `toggle_layout` | Layout: unificato o affiancato | `m` |
| `diff` | `toggle_hunks_only` | Solo hunk o file intero | `Shift+H` |
| `diff` | `toggle_minimap` | Mostra o nascondi la minimappa | `Shift+M` |
| `diff` | `choose_encoding` | Scegli la codifica | `Shift+C` |
| `diff` | `toggle_wrap` | Attiva o disattiva l'a capo automatico | `w` |
| `diff` | `toggle_whitespace` | Mostra o nascondi le modifiche di soli spazi | `s` |
| `diff` | `merge_editor` | Apri l'editor di merge (file in conflitto) | `Shift+E` |
| `file_insight` | `close` | Chiudi la vista del file | `q` |
| `file_insight` | `edit` | Modifica nella directory di lavoro | `e` |
| `file_insight` | `copy` | Copia la selezione o la riga | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Vai all'ultima riga | `Shift+G` |
| `file_insight` | `open_commit` | Vai al commit della riga | `Enter` |
| `history` | `show_in_log` | Mostra nel Log | `Enter` |
| `history` | `copy_sha` | Copia lo SHA del commit | `y` |
| `history` | `copy_summary` | Copia il riepilogo | `Shift+Y` |
| `history` | `show_diff` | Mostra il diff di questo commit | `d`, `Alt+1` |
| `history` | `open_file` | Apri il file a questo commit | `o`, `Alt+2` |
| `history` | `blame` | Blame a questo commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Confronta con la directory di lavoro | `c` |
| `history` | `restore` | Ripristina questa versione | `Shift+R` |
| `history` | `range_up` | Estendi la selezione verso l'alto | `Shift+Up` |
| `history` | `range_down` | Estendi la selezione verso il basso | `Shift+Down` |
| `history` | `switch_focus` | Passa tra l'elenco e il diff | `Tab` |
| `history` | `widen_list` | Allarga l'elenco | `Alt+Right` |
| `history` | `narrow_list` | Restringi l'elenco | `Alt+Left` |
| `history` | `reset_split` | Reimposta la larghezza dell'elenco | `Alt+0` |
| `commit_column` | `scroll_left` | Scorri la colonna a sinistra | `Shift+Left` |
| `commit_column` | `scroll_right` | Scorri la colonna a destra | `Shift+Right` |
| `commit_column` | `next_part` | Dimensiona la parte successiva | `Tab` |
| `commit_column` | `move_left` | Sposta la parte a sinistra | `<` |
| `commit_column` | `move_right` | Sposta la parte a destra | `>` |
| `commit_column` | `wider_part` | Allarga la parte | `]` |
| `commit_column` | `narrower_part` | Restringi la parte | `[` |
| `commit_column` | `reset_widths` | Reimposta le larghezze delle parti | `0` |
| `file_insight` | `blame_card` | Blame: Mostra o nascondi la scheda | `i` |
| `file_insight` | `blame_next` | Blame: Hunk successivo | `n` |
| `file_insight` | `blame_prev` | Blame: Hunk precedente | `p` |
| `file_insight` | `blame_next_same` | Blame: Hunk successivo dello stesso commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Hunk precedente dello stesso commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Vai alla riga | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Copia lo SHA del commit | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Diff del commit per questo file | `d` |
| `file_insight` | `blame_parent` | Blame: Blame sul genitore del commit | `b` |
| `file_insight` | `blame_history` | Blame: Apri la cronologia del file | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Allarga la colonna delle annotazioni | `>` |
| `file_insight` | `blame_narrower` | Blame: Restringi la colonna delle annotazioni | `<` |
| `image` | `zoom_in` | Ingrandisci l'immagine | `+`, `=` |
| `image` | `zoom_out` | Riduci l'immagine | `-` |
| `image` | `fit` | Adatta l'immagine alla vista | `0` |
| `image` | `actual_size` | Mostra l'immagine a grandezza reale | `1` |
| `image` | `flip_side` | Mostra l'altro lato (prima / dopo) | `Tab` |
| `image` | `widen_before` | Allarga il pannello Prima | `Alt+Right` |
| `image` | `widen_after` | Allarga il pannello Dopo | `Alt+Left` |
| `image` | `reset_split` | Ripristina la divisione prima / dopo | `Alt+0` |
| `image` | `edit_in` | Apri con... | `Shift+O` |
| `image` | `edit_default` | Apri nell'applicazione predefinita | `o` |
| `editor` | `save` | Salva | `Ctrl+S` |
| `editor` | `undo` | Annulla | `Ctrl+Z` |
| `editor` | `redo` | Ripeti | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Cerca | `Ctrl+F` |
| `editor` | `find_next` | Cerca successivo (Shift: precedente) | `F3` |
| `editor` | `paste` | Incolla | `Ctrl+V` |
| `editor` | `toggle_minimap` | Mostra o nascondi la minimappa | `Alt+M` |
| `editor_prompt` | `save` | Modifiche non salvate: salva e chiudi | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Modifiche non salvate: scarta e chiudi | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Modifiche non salvate: continua a modificare | `c`, `Shift+C` |
| `merge` | `next_conflict` | Conflitto successivo | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Conflitto precedente | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | File in conflitto successivo | `Alt+PageDown` |
| `merge` | `prev_file` | File in conflitto precedente | `Alt+PageUp` |
| `merge` | `next_pane` | Porta il focus sul pannello successivo | `F6` |
| `merge` | `prev_pane` | Porta il focus sul pannello precedente | `Shift+F6` |
| `merge` | `toggle_panes` | Mostra o nascondi i pannelli laterali (Refs e Stato) | `F9` |
| `merge` | `toggle_base` | Mostra o nascondi il pannello Base | `Alt+D` |
| `merge` | `toggle_full_file` | Attiva o disattiva il diff a file intero | `Alt+F` |
| `merge` | `undo` | Annulla (scelte e digitazione) | `Ctrl+Z` |
| `merge` | `redo` | Ripeti (scelte e digitazione) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Allarga il pannello Corrente (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Allarga il pannello In entrata (B) | `Alt+Left` |
| `merge` | `reset_panes` | Reimposta le dimensioni dei pannelli | `Alt+0` |
| `merge` | `save` | Salva senza segnare come risolto | `Alt+S` |
| `merge` | `save_resolve` | Salva e segna come risolto | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Suggerisci una risoluzione (IA) | `Alt+I` |
| `merge` | `unpick` | Annulla la scelta di questo conflitto (non risolto) | `Alt+U` |
| `merge` | `reset_all` | Riportare tutto il file ai conflitti originali | `Alt+R` |
| `merge` | `focus_a` | Vai al pannello Corrente (A) | `Alt+1` |
| `merge` | `focus_b` | Vai al pannello In entrata (B) | `Alt+2` |
| `merge` | `focus_output` | Vai al pannello Risultato | `Alt+3` |
| `merge` | `focus_base` | Vai al pannello Base | `Alt+4` |
| `merge_sides` | `toggle_line` | Scegli o scarta la riga | `Space` |
| `merge_sides` | `toggle_chunk` | Scegli o scarta l'intero blocco | `Enter` |
| `merge_sides` | `take_a` | Prendi solo Corrente (A) | `a` |
| `merge_sides` | `take_b` | Prendi solo In entrata (B) | `b` |
| `merge_sides` | `take_both` | Prendi entrambi, prima A poi B | `c` |
| `merge_sides` | `take_both_reversed` | Prendi entrambi, prima B poi A | `Shift+C` |
| `merge_sides` | `clear` | Non prendere nulla per questo conflitto | `x` |
| `merge_sides` | `unpick` | Annulla la scelta di questo conflitto (non risolto) | `u` |
| `merge_sides` | `reset_all` | Riportare tutto il file ai conflitti originali | `Shift+U` |
| `merge_sides` | `swap_order` | Inverti l'ordine dei due lati | `s` |
| `merge_sides` | `next_conflict` | Conflitto successivo | `n` |
| `merge_sides` | `prev_conflict` | Conflitto precedente | `Shift+N` |
| `merge_sides` | `take_all_a` | Prendi tutto da Corrente (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Prendi tutto da In entrata (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Mostra o nascondi il pannello Base | `d` |
| `merge_sides` | `scroll_left` | Scorri i pannelli a sinistra | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Scorri i pannelli a destra | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Attiva o disattiva il diff a file intero | `f` |
| `merge_sides` | `cycle_height` | Cambia l'altezza del risultato | `z` |
| `merge_card` | `take_a` | Tieni A (Corrente) così com'è | `a` |
| `merge_card` | `take_b` | Prendi B (In entrata) così com'è | `b` |
| `merge_card` | `take_base` | Prendi la Base (antenato comune) | `o` |
| `merge_card` | `keep` | Tieni il file rimasto | `k` |
| `merge_card` | `delete` | Accetta l'eliminazione | `Shift+D` |
| `merge_card` | `external_tool` | Apri nello strumento di merge esterno | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (fermati per fare amend) | `e` |
| `rebase` | `autosquash` | Attiva o disattiva autosquash | `a` |
| `rebase` | `start` | Avvia il piano | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Reimposta il piano | `Ctrl+R` |
| `rebase` | `split` | Dividi il commit della riga | `t` |
| `rebase` | `move_up` | Sposta la riga in su | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Sposta la riga in giù | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Crea i commit | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Attiva o disattiva la selezione per righe | `v` |
| `split` | `move_up` | Sposta il commit in su | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Sposta il commit in giù | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Reimposta la divisione | `Shift+R` |
| `rewrite` | `preview` | Mostra l'anteprima della riscrittura | `F5` |
| `rewrite` | `apply` | Applica la riscrittura | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Annulla l'anteprima | `Ctrl+X` |
| `rewrite` | `add_rule` | Aggiungi una regola | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Attiva o disattiva la regola | `Space` |
| `rewrite` | `move_up` | Sposta la regola in alto | `Alt+Up` |
| `rewrite` | `move_down` | Sposta la regola in basso | `Alt+Down` |
| `rewrite` | `delete_rule` | Elimina la regola | `d`, `Delete` |
| `rewrite` | `restore_rule` | Ripristina la regola eliminata | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplica la regola | `Ctrl+D` |
| `rewrite` | `next_tab` | Scheda di anteprima successiva | `Alt+Right` |
| `rewrite` | `prev_tab` | Scheda di anteprima precedente | `Alt+Left` |
| `rewrite` | `save_shared` | Salva la ricetta nel repository | `Alt+S` |
| `rewrite` | `save_mine` | Salva la ricetta solo per me | `Alt+M` |
| `rewrite` | `load` | Carica una ricetta | `Alt+L` |
| `rewrite` | `name` | Rinomina la ricetta | `F2` |
| `rewrite` | `protected_all` | Includi tutti i branch protetti | `Alt+A` |
| `rewrite` | `protected_none` | Escludi tutti i branch protetti | `Alt+U` |
| `rewrite` | `check_tools` | Ricontrolla gli strumenti | `F6` |
| `rewrite` | `analyze` | Analizza le dimensioni della cronologia | `F7` |
| `rewrite` | `host_howto` | Spiega la protezione dei branch dell'host | `Alt+H` |
| `rewrite` | `ai_rules` | Crea regole di riscrittura con l'IA | `Alt+I` |
| `reword` | `apply` | Applica il nuovo messaggio | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modifica la cronologia in profondità | `Alt+H` |
| `composer` | `commit` | Commit dal compositore | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Inserisci un riferimento a una issue | `Alt+R` |
| `dialog` | `browse` | Sfoglia per trovare una cartella | `Ctrl+B` |
| `dialog` | `sources` | Vai alle origini del clone | `Alt+S` |
| `dialog` | `submit` | Invia il modulo pull request | `Ctrl+S` |
| `dialog` | `web` | Apri il modulo web del provider | `Alt+W` |
| `folder_picker` | `choose` | Scegli la cartella | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nuova cartella | `Alt+N` |
| `folder_picker` | `filter` | Filtra le voci della cartella | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Mostra o nascondi le cartelle sorelle | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Mostra o nascondi le cartelle nascoste | `Alt+H` |
| `preferences` | `search` | Cerca nelle impostazioni | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Aggiungi una voce all'elenco | `Insert` |
| `preferences` | `list_duplicate` | Duplica la voce selezionata dell'elenco | `Ctrl+D` |
| `preferences` | `list_remove` | Rimuovi la voce selezionata dall'elenco | `Delete` |
| `preferences` | `list_move_up` | Sposta in su la voce selezionata dell'elenco | `Alt+Up` |
| `preferences` | `list_move_down` | Sposta in giù la voce selezionata dell'elenco | `Alt+Down` |
| `preferences` | `prev_tab` | Scheda precedente della pagina delle impostazioni | `Alt+Left` |
| `preferences` | `next_tab` | Scheda successiva della pagina delle impostazioni | `Alt+Right` |
| `ai_popup` | `copy` | Copia il risultato | `c` |
| `integrations` | `add` | Aggiungi un profilo | `a` |
| `integrations` | `edit` | Modifica il profilo selezionato | `e` |
| `integrations` | `open_token` | Apri la pagina del token | `o` |
| `integrations` | `remove` | Rimuovi il profilo selezionato | `d`, `Delete` |
| `prompt` | `ai_stash_message` | IA: genera il messaggio di stash | `Ctrl+G` |
<!-- keymap:end -->

### Tasti che il terminale non può inviare e il protocollo tastiera kitty

Un terminale tradizionale non può inviare alcune combinazioni: `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` e `Ctrl+<cifra>`. Non invia nulla, oppure un altro
tasto: per esempio `Ctrl+Shift+N` arriva come `Ctrl+N`, `Ctrl+I` come `Tab` e
`Ctrl+M` come `Enter`. La pagina Impostazioni avvisa di un'assegnazione di
questo tipo («questo terminale non invia ...»).

I terminali con il **protocollo tastiera kitty** possono inviare queste
combinazioni. Esempi: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 e
successivi, e tmux o herdr quando fanno passare il protocollo. GitPoulpi usa il
protocollo per impostazione predefinita:

- All'avvio, GitPoulpi interroga il terminale e attende la risposta per circa
  100 ms al massimo. Se il terminale non risponde, restano i tasti tradizionali.
- Se il terminale risponde, GitPoulpi attiva il protocollo. Lo disattiva di nuovo
  quando esce, fallisce o va in crash.
- Gli avvisi della pagina Impostazioni seguono il protocollo: quando è attivo, le
  combinazioni qui sopra sono inviabili.
- **Impostazioni > Tastiera e mouse > Protocollo tastiera Kitty**
  (`ui.kitty_keyboard`) su `off` non cambia mai la modalità tastiera del
  terminale. Usalo se un terminale o un multiplexer si comporta male. Serve un
  riavvio.

Le console di Windows riportano da sole i modificatori e non hanno bisogno del
protocollo.

## Palette dei comandi (`Ctrl+P`)

La palette elenca **ogni** comando dell'app, così raggiunge tutto senza una
scorciatoia. L'elenco completo è [qui sotto](#tutti-i-comandi-della-palette).

| Tasto | Azione |
|---|---|
| `Ctrl+P` | Apre la palette |
| *(scrivi)* | Filtra l'elenco. Una corrispondenza esatta dopo il prefisso `Categoria: ` viene per prima, poi una corrispondenza in qualsiasi punto dell'etichetta, poi una corrispondenza fuzzy |
| `Su` / `Giù`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Spostano l'evidenziazione (vedi [Elenchi popup filtrabili](#elenchi-popup-filtrabili)) |
| `Enter`, o un doppio clic | Esegue il comando evidenziato |
| `Esc` | Chiude |

`j` e `k` vengono digitati nel filtro. Non spostano l'evidenziazione.

Ogni riga mostra la sua scorciatoia a destra. Un comando che ora non si applica
(per esempio, nessun ref selezionato) resta nell'elenco, in grigio, e il motivo
compare sotto l'elenco. Le etichette iniziano con una categoria, come
`Navigazione:`, `Refs:`, `Git:`, `Vista:`, `Diff:` o `App:`. Digita una categoria
per vederne l'intero gruppo.

### Un tasto per qualsiasi comando

Ogni riga della palette termina con un pulsante tasto (`[k]`), e `Alt+K` fa lo stesso sulla riga evidenziata: apre **Impostazioni > Tastiera e mouse** filtrato sulla scorciatoia di quel comando, pronto per essere riassegnato. Un comando senza scorciatoia propria ne riceve una qui: compare in **Comandi della palette**, senza tasto di default, e il tasto che gli assegni esegue il comando esattamente come sceglierlo nella palette (un comando che ora non può partire ne dice il motivo). La riga mostra poi il tuo tasto.

In `config.toml` è la tabella `[keybindings.palette]`, con il nome del comando (`toggle_minimap = "Ctrl+Alt+M"`). Un tasto in conflitto con un'altra azione è mostrato in rosso, e `c` sulla sua riga elenca tutte le azioni con cui è in conflitto.

La riga di aiuto sotto l'elenco indica questo tasto (`Alt+K` di default). È l'azione `palette.configure_shortcut`, quindi si riassegna come le altre.

### Tutti i comandi della palette

Generato dall'app (un test lo tiene aggiornato). I comandi IA compaiono solo
quando l'IA è attiva; gli altri sono in grigio, con il motivo, quando non si
applicano. I tasti sono quelli predefiniti.

<!-- palette:begin -->
| Comando | Tasto predefinito |
|---|---|
| Navigazione: Vai al pannello Refs | `Ctrl+N <badge>` |
| Navigazione: Vai al pannello Log | `Ctrl+N <badge>` |
| Navigazione: Vai al pannello Stato | `Ctrl+N <badge>` |
| Navigazione: Vai al filtro dei Refs | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Locali | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Remoti | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Worktrees | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Sottomoduli | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Stash | `Ctrl+N <badge>` |
| Navigazione: Vai a Refs > Tag | `Ctrl+N <badge>` |
| Barra laterale: Mostra o nascondi le sezioni... |  |
| Barra laterale: Ripristina le sezioni predefinite |  |
| Barra laterale: Usa queste sezioni per tutti i repository |  |
| Barra laterale: Mostra o nascondi i branch locali |  |
| Barra laterale: Mostra o nascondi i remoti |  |
| Barra laterale: Mostra o nascondi i worktree |  |
| Barra laterale: Mostra o nascondi Stash |  |
| Barra laterale: Mostra o nascondi Tag |  |
| Barra laterale: Mostra o nascondi Pull request |  |
| Barra laterale: Mostra o nascondi le issue |  |
| Barra laterale: Mostra o nascondi i sottomoduli |  |
| Navigazione: Vai a Stato > Non in stage | `Ctrl+N <badge>` |
| Navigazione: Vai a Stato > In stage | `Ctrl+N <badge>` |
| Navigazione: Sposta il focus sul pannello successivo | `Tab` |
| Navigazione: Sposta il focus sul pannello precedente | `Shift+Tab` |
| Navigazione: Cerca commit... | `f` |
| Navigazione: Comprimi o ripristina il pannello Refs | `Ctrl+N h` |
| Navigazione: Allarga il pannello Refs | `Ctrl+N Right` |
| Navigazione: Restringi il pannello Refs | `Ctrl+N Left` |
| Navigazione: Allarga il pannello Stato | `Ctrl+N Shift+Left` |
| Navigazione: Restringi il pannello Stato | `Ctrl+N Shift+Right` |
| Navigazione: Comprimi o ripristina il pannello Stato | `Ctrl+N l` |
| Navigazione: Ingrandisci la sezione Refs | `Ctrl+N +` |
| Navigazione: Riduci la sezione Refs | `Ctrl+N -` |
| Navigazione: Reimposta le dimensioni delle sezioni Refs |  |
| Navigazione: Reimposta il layout |  |
| Dettagli commit: Ingrandisci il messaggio | `Ctrl+N ]` |
| Dettagli commit: Riduci il messaggio | `Ctrl+N [` |
| Dettagli commit: Reimposta la dimensione del messaggio |  |
| Dettagli commit: Scorri il messaggio in giù |  |
| Dettagli commit: Scorri il messaggio in su |  |
| File: Inverti l'ordine dei file del commit (A-Z / Z-A) |  |
| Refs: Checkout del ref selezionato | `doppio clic` |
| Refs: Copia il nome del ref |  |
| Refs: Copia il ref completo (refs/heads/...) |  |
| Refs: Copia lo SHA di destinazione del ref |  |
| Refs: Merge del branch selezionato in quello corrente |  |
| Refs: Rebase del branch corrente su quello selezionato | `Ctrl+G r` |
| Refs: Rilascia il ref selezionato sul branch attivo... |  |
| Refs: Segna come origine del rilascio | `m` |
| Refs: Rilascia qui il ref segnato... | `M` |
| Refs: Rinomina il branch selezionato... |  |
| Refs: Fast-forward del branch selezionato a quello corrente |  |
| Refs: Fast-forward del branch corrente a quello selezionato |  |
| Refs: Crea un worktree dal branch o tag selezionato... |  |
| Refs: Reflog del branch selezionato... |  |
| Refs: Avvia una pull request dal branch selezionato... |  |
| Refs: Apri il worktree in una scheda | `Enter / doppio clic` |
| Refs: Aggiorna il sottomodulo (selezionato) |  |
| Refs: Inizializza il sottomodulo (selezionato) |  |
| Refs: Apri il sottomodulo in una scheda | `Enter / doppio clic` |
| Git: Aggiorna tutti i sottomoduli |  |
| Git: Crea un worktree sul commit selezionato... |  |
| Git: Crea una patch dal commit selezionato... |  |
| Git: Crea una patch dal commit selezionato a HEAD... |  |
| Git: Crea una patch dalle modifiche della directory di lavoro... |  |
| Riscrittura: Elimina la vecchia cronologia... |  |
| Git: Applica una patch... |  |
| Riscrittura: Pannello della cronologia | `Ctrl+G H` |
| Riscrittura: Sostituisci un autore... |  |
| Riscrittura: Rendi anonimi gli autori... |  |
| Riscrittura: Applica .mailmap alla cronologia... |  |
| Riscrittura: Merge le identità (.mailmap)... |  |
| Riscrittura: Sposta le date dei commit... |  |
| Riscrittura: Cambia il fuso orario dei commit... |  |
| Riscrittura: Tieni i commit solo in certi orari... |  |
| Riscrittura: Usa la data dell'autore come data del committer |  |
| Riscrittura: Cerca e sostituisci nei messaggi... |  |
| Riscrittura: Elimina righe dai messaggi... |  |
| Riscrittura: Aggiungi o rimuovi trailer... |  |
| Riscrittura: Rimuovi le firme... |  |
| Riscrittura: Correggi la codifica dei messaggi... |  |
| Riscrittura: Cerca e sostituisci nei file... |  |
| Riscrittura: Rimuovi file dalla cronologia... |  |
| Riscrittura: Rimuovi i file grandi dalla cronologia... |  |
| Riscrittura: Sposta o rinomina percorsi... |  |
| Riscrittura: Migra i file a LFS... |  |
| Riscrittura: Riporta i file fuori da LFS... |  |
| Riscrittura: Converti i fine riga... |  |
| Riscrittura: Rimuovi il bit eseguibile... |  |
| Riscrittura: Converti la codifica dei file... |  |
| Riscrittura: Esegui uno script... |  |
| ✦ Riscrittura: Scrivi uno script con l'IA... |  |
| ✦ Riscrittura: Crea regole con l'IA... |  |
| Riscrittura: Dal commit selezionato a HEAD... |  |
| Riscrittura: Commit selezionati... |  |
| LFS: Inizializza per questo repository |  |
| LFS: Traccia un pattern... |  |
| LFS: Smetti di tracciare un pattern... |  |
| LFS: Fetch degli oggetti |  |
| LFS: Apri il menu | `Ctrl+G l` |
| LFS: Fetch di tutti gli oggetti | `Ctrl+G F` |
| LFS: Checkout di tutti i file | `Ctrl+G K` |
| LFS: Push di tutti gli oggetti | `Ctrl+G J` |
| LFS: Pull degli oggetti | `Ctrl+G L` |
| LFS: Push degli oggetti |  |
| LFS: Ripulisci gli oggetti locali | `Ctrl+G V` |
| LFS: Mostra i blocchi |  |
| LFS: Apri la pagina di installazione |  |
| LFS: Mostra o nascondi il puntatore grezzo nel diff | `r` |
| LFS: Traccia il file selezionato |  |
| LFS: Traccia l'estensione del file selezionato |  |
| LFS: Blocca il file selezionato |  |
| LFS: Sblocca il file selezionato |  |
| LFS: Sblocca con la forza il file selezionato |  |
| Refs: Elimina il branch o tag selezionato... | `Ctrl+G d` |
| Refs: Elimina il branch selezionato dal suo remoto... |  |
| Refs: Elimina il branch selezionato in locale e sul suo remoto... |  |
| Refs: Copia il link del branch selezionato sul remoto |  |
| Git: Copia il link del commit selezionato sul remoto |  |
| Git: Crea un tag annotato sul commit selezionato... |  |
| Refs: Push del tag selezionato verso il remoto predefinito |  |
| Refs: Elimina il tag selezionato dal suo remoto... |  |
| Git: Fetch del tag selezionato |  |
| Refs: Annota il tag selezionato... |  |
| Refs: Mostra o nascondi nel grafo |  |
| Refs: Mostra nel grafo | `g` |
| Refs: Attiva o disattiva il solo sul ref selezionato | `o` |
| Refs: Termina il solo (rimuove ogni ref in solo) | `O` |
| Refs: Mostra tutti i Refs nascosti | `H` |
| Refs: Mostra o nascondi la cartella/il remoto selezionato |  |
| Refs: Attiva o disattiva il solo sulla cartella/sul remoto selezionato | `o` |
| Refs: Filtra per nome... | `/` |
| Refs: Rimuovi il filtro per nome | `Esc` |
| Refs: Espandi tutte le cartelle | `=` |
| Refs: Comprimi tutte le cartelle | `-` |
| Refs: Espandi la cartella selezionata e le sue sottocartelle | `*` |
| Refs: Comprimi la cartella selezionata e le sue sottocartelle | `_` |
| Refs: Sezione successiva | `l` |
| Refs: Sezione precedente | `h` |
| Refs: Espandi o comprimi la sezione attiva | `Enter` |
| Refs: Attiva o disattiva la vista gerarchica dei branch | `y` |
| Git: Stage del file selezionato | `Ctrl+G s` |
| Git: Unstage del file selezionato | `Ctrl+G u` |
| Git: Stage di tutte le modifiche | `Ctrl+G a` |
| Modifiche in corso: Scarta il file selezionato... |  |
| Modifiche in corso: Ignora il file selezionato |  |
| Modifiche in corso: Ignora l'estensione del file selezionato |  |
| Modifiche in corso: Tratta il file selezionato come binario... |  |
| Modifiche in corso: Tratta il file selezionato come testo... |  |
| Modifiche in corso: Tratta l'estensione del file selezionato come binaria... |  |
| Modifiche in corso: Tratta l'estensione del file selezionato come testo... |  |
| Modifiche in corso: Stash del file selezionato |  |
| Modifiche in corso: Cronologia del file selezionato |  |
| Modifiche in corso: Blame del file selezionato |  |
| Modifiche in corso: Apri il file selezionato nello strumento diff esterno |  |
| Modifiche in corso: Apri il file selezionato nell'editor esterno |  |
| Modifiche in corso: Apri il file selezionato nel programma predefinito |  |
| Modifiche in corso: Mostra il file selezionato nella cartella |  |
| Modifiche in corso: Copia il percorso del file selezionato |  |
| Modifiche in corso: Crea una patch dal file selezionato... |  |
| Modifiche in corso: Modifica il file selezionato |  |
| Modifiche in corso: Elimina il file selezionato... |  |
| Modifiche in corso: Apri il menu della cartella selezionata |  |
| Modifiche in corso: Crea file |  |
| Modifiche in corso: Scarta tutte le modifiche... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Esegui ora il commit delle modifiche in stage | `Ctrl+G Enter` |
| Git: Modifica il messaggio del commit WIP (nel Log) | `c nel Log` |
| Git: Fetch di tutti i remoti | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward se possibile) |  |
| Git: Pull (solo fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Opzioni di pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push verso un remoto... |  |
| Git: Force push (con lease)... |  |
| Git: Imposta l'upstream... |  |
| Remoto: Aggiungi remoto... |  |
| Remoto: Rimuovi remoto... |  |
| Remoto: Modifica remoto... |  |
| Remoto: Imposta il remoto predefinito... |  |
| Remoto: Copia il nome del remoto |  |
| Remoto: Copia il link del remoto |  |
| Remoto: Fetch di un remoto... |  |
| Remoto: Azioni per il remoto selezionato... | `.` |
| Repository: Chiave SSH per questo repository... |  |
| Repository: Checkout parziale... |  |
| Flow: Apri il menu | `Ctrl+G W` |
| Flow: Avvia un branch... |  |
| Flow: Nuova issue... |  |
| Flow: Prendi in carico un branch dell'host... |  |
| Flow: Termina il branch corrente... |  |
| Flow: Sincronizza con il genitore... |  |
| Flow: Promuovi... |  |
| Flow: Imposta il genitore... |  |
| Flow: Continua o interrompi l'azione interrotta... |  |
| Flow: Attiva o disattiva il raggruppamento dei branch |  |
| Flow: Impostazioni del workflow... |  |
| Remote: Hide Remote in Graph... |  |
| Remote: Show Remote in Graph... |  |
| Git: Nuovo branch su HEAD... | `Ctrl+G b` |
| Git: Crea un tag su HEAD... | `Ctrl+G t` |
| Git: Push di tutti i tag non inviati | `Ctrl+G U` |
| Git: Stash di tutte le modifiche | `Ctrl+G T` |
| Git: Stash di tutte le modifiche con messaggio... | `Ctrl+G m` |
| Git: Pop dello stash | `Ctrl+G o` |
| Git: Applica lo stash |  |
| Git: Drop dello stash... |  |
| Git: Cherry-pick del commit selezionato su HEAD | `Ctrl+G y` |
| Git: Checkout del commit selezionato (HEAD scollegato) |  |
| Git: Crea un branch sul commit selezionato... |  |
| Git: Crea un tag sul commit selezionato... |  |
| Commit: Copia il messaggio |  |
| Commit: Copia il nome dell'autore |  |
| Commit: Copia l'e-mail dell'autore |  |
| Commit: Copia la data |  |
| Commit: Copia lo SHA breve |  |
| Commit: Copia lo SHA completo |  |
| Git: Revert del commit selezionato... | `Ctrl+G v` |
| Git: Modifica il messaggio del commit... | `Ctrl+G e` |
| Commit: Modifica... | `Ctrl+G E` |
| Commit: Modifica l'autore... | `Ctrl+G A` |
| Commit: Modifica il riepilogo... |  |
| Commit: Modifica la descrizione... |  |
| Git: Dividi il commit selezionato... |  |
| Git: Rebase interattivo dal commit selezionato... |  |
| Rebase: Continua |  |
| Rebase: Salta |  |
| Rebase: Interrompi |  |
| git am: Continua |  |
| git am: Salta questa patch |  |
| git am: Interrompi |  |
| Repository: Rimuovi index.lock obsoleto |  |
| Banner: Pop dell'auto-stash in sospeso |  |
| Banner: Termina l'auto-stash in sospeso (mantieni il risultato) |  |
| Banner: Mostra l'auto-stash in sospeso |  |
| Conflitti: Mostra i conflitti previsti |  |
| Banner: Drop dell'auto-stash in sospeso... |  |
| Banner: Ricomincia... |  |
| Git: Reset del branch al commit selezionato (soft - mantiene indice e directory di lavoro) |  |
| Git: Reset del branch al commit selezionato (mixed - mantiene la directory di lavoro) | `Ctrl+G R` |
| Git: Reset del branch al commit selezionato (hard - scarta tutto)... |  |
| Git: Risolvi il conflitto con Ours (questo branch) |  |
| Git: Risolvi il conflitto con Theirs (in entrata) |  |
| Git: Risolvi il conflitto con Base (antenato comune) |  |
| Conflitti: Apri l'editor di merge | `Enter su un file in conflitto` |
| Merge: Conflitto successivo | `F7` |
| Merge: Conflitto precedente | `Shift+F7` |
| Merge: File in conflitto successivo | `Alt+PageDown` |
| Merge: File in conflitto precedente | `Alt+PageUp` |
| Merge: Prendi il blocco da A |  |
| Merge: Prendi il blocco da B |  |
| Merge: Prendi Corrente per questo conflitto | `a` |
| Merge: Prendi In entrata per questo conflitto | `b` |
| Merge: Prendi entrambi, prima A poi B | `c` |
| Merge: Prendi entrambi, prima B poi A | `C` |
| Merge: Prendi base per questo conflitto |  |
| Merge: Svuota questo conflitto | `x` |
| Merge: Inverti l'ordine | `s` |
| Merge: Prendi tutto da Corrente | `A` |
| Merge: Prendi tutto da In entrata | `B` |
| Merge: Modifica questo conflitto nel risultato |  |
| Merge: Mostra o nascondi i pannelli laterali | `F9` |
| Merge: Scorri i pannelli a sinistra | `Shift+Left` |
| Merge: Scorri i pannelli a destra | `Shift+Right` |
| Merge: Mostra o nascondi il pannello Base | `Alt+D` |
| Merge: Attiva o disattiva il diff a file intero | `Alt+F` |
| Merge: Annulla | `Ctrl+Z` |
| Merge: Ripeti | `Ctrl+Y` |
| Merge: Salva senza segnare come risolto | `Alt+S` |
| Merge: Salva e segna come risolto | `Ctrl+S` |
| Merge: Reimposta il risultato sul merge | `Alt+R` |
| Merge: Cambia l'altezza del risultato | `z` |
| Merge: Allarga il pannello Corrente | `Alt+Right` |
| Merge: Allarga il pannello In entrata | `Alt+Left` |
| Merge: Reimposta le dimensioni dei pannelli | `Alt+0` |
| Merge: Annulla la scelta di questo conflitto | `Alt+U` |
| Merge: Vai al pannello Corrente (A) | `Alt+1` |
| Merge: Vai al pannello In entrata (B) | `Alt+2` |
| Merge: Vai al pannello Risultato | `Alt+3` |
| Merge: Vai al pannello Base | `Alt+4` |
| Merge: Vai al pannello a sinistra (stesso conflitto) |  |
| Merge: Vai al pannello a destra (stesso conflitto) |  |
| Merge: Porta il focus sul pannello successivo | `F6` |
| Merge: Porta il focus sul pannello precedente | `Shift+F6` |
| Merge: Continua dal file su disco |  |
| Merge: Ricomincia dal merge |  |
| Merge: Riapri il conflitto |  |
| Merge: Ricarica il file |  |
| Merge: Usa il file Corrente così com'è | `a` |
| Merge: Usa il file In entrata così com'è | `b` |
| Merge: Usa il file Base così com'è | `o` |
| Merge: Tieni il file rimasto | `k` |
| Merge: Accetta l'eliminazione | `D` |
| Merge: Apri nello strumento di merge esterno | `m` |
| ✦ IA: Suggerisci una risoluzione del conflitto di merge | `Alt+I` |
| Merge: Chiudi l'editor di merge |  |
| Git: Interrompi merge, cherry-pick o revert |  |
| Git: Salta cherry-pick o revert |  |
| Repository: Apri o passa a un altro (gestione repository)... | `Ctrl+O` |
| Repository: Sfoglia per trovare un repository... | `b (scheda di gestione)` |
| Repository: Crea nuovo (Init)... | `i (scheda di gestione)` |
| Repository: Apri il selezionato | `Enter` |
| Repository: Aggiungi o togli il selezionato dai preferiti | `f` |
| Repository: Chiudi o rimuovi il selezionato | `x` |
| Repository: Menu del repository selezionato... | `.` |
| Repository: Cerca | `/ / Ctrl+F` |
| Repository: Comprimi tutte le sezioni | `-` |
| Repository: Espandi tutte le sezioni | `+` |
| Repository: Chiudi tutte le schede... |  |
| Repository: Rimuovi il selezionato dall'elenco |  |
| Repository: Rimuovi tutti i recenti |  |
| Area di lavoro: Nuova... | `n` |
| Area di lavoro: Nuova dalle schede aperte... | `a` |
| Area di lavoro: Cambia... | `Alt+W` |
| Area di lavoro: Modifica... | `e` |
| Area di lavoro: Chiudi (nessuna area di lavoro) |  |
| Area di lavoro: Gestisci... | `Alt+J` |
| Area di lavoro: Chiudi scheda |  |
| Area di lavoro: Individua il repository mancante... | `l` |
| Area di lavoro: Elimina... |  |
| Area di lavoro: Fetch All | `F` |
| Area di lavoro: Pull All | `P` |
| Area di lavoro: Clona i mancanti | `m` |
| Area di lavoro: Aggiungi la scheda all'area di lavoro |  |
| Area di lavoro: Rimuovi la scheda dall'area di lavoro |  |
| Radar: Apri | `Alt+R` |
| Radar: Aggiorna | `r` |
| Radar: Cerca | `/` |
| Radar: Apri nel browser | `o` |
| Radar: Apri repository | `Enter` |
| Radar: Checkout del branch | `c` |
| Radar: Crea worktree... | `t` |
| Radar: Mostra nel repository | `s` |
| Radar: Inizia a lavorare su una issue... | `n` |
| Radar: Fissa o non fissare | `p` |
| Radar: Rimanda... | `z` |
| Radar: Rivedi la richiesta... | `e` |
| Radar: Rivedi tramite l'API |  |
| Radar: Clona e rivedi... |  |
| Radar: Modifica assegnatari... |  |
| Radar: Modifica revisori... |  |
| Radar: Modifica etichette... |  |
| Radar: Imposta milestone... |  |
| Radar: Scegli ambito | `w` |
| Radar: Scegli vista | `v` |
| Radar: Scegli filtro | `f` |
| Radar: Mostra le richieste non clonate | `a` |
| Radar: Mostra le bozze | `d` |
| Radar: Attiva le richieste di revisione dell'account... | `T` |
| Radar: Menu della riga... | `.` |
| Radar: Scegli colonna | `C` |
| Radar: Allarga colonna | `]` |
| Radar: Restringi colonna | `[` |
| Radar: Reimposta larghezze colonne | `0` |
| Radar: Torna alla home | `Esc` |
| Radar: Chiudi scheda |  |
| Scheda: Nuova scheda | `Alt+T` |
| Scheda: Nuova... |  |
| Scheda: Menu delle azioni... | `Ctrl+T` |
| Scheda: Chiudi le altre schede... |  |
| Scheda: Chiudi le schede a destra... |  |
| Scheda: Aggiungi o togli dai preferiti |  |
| Scheda: Copia il percorso del repository |  |
| Scheda: Apri nel gestore file |  |
| Scheda: Apri nel terminale |  |
| Scheda: Apri in... | `Alt+O` |
| Scheda: Apri nella voce predefinita | `Ctrl+Alt+O` |
| Repository: Cambia profilo... |  |
| Aiuto: Informazioni su GitPoulpi | `Ctrl+N a` |
| Aiuto: Licenze e crediti |  |
| Vista: Registro attività | `Ctrl+N A` |
| Vista: Registro attività globale | `Ctrl+N G` |
| Aiuto: Note di rilascio |  |
| Aggiornamenti: Cerca aggiornamenti ora |  |
| Aggiornamenti: Download |  |
| Aggiornamenti: Aggiorna ora |  |
| Aggiornamenti: Riavvia per aggiornare |  |
| Aggiornamenti: Torna alla versione precedente |  |
| Aggiornamenti: Salta questa versione |  |
| Note di rilascio: Chiudi scheda |  |
| Attività: mostra le attività in background | `Ctrl+N J` |
| Attività: annulla tutto ciò che si può annullare |  |
| Vista: Copia diagnostica |  |
| Registro attività: mostra o nascondi il livello Errore |  |
| Registro attività: mostra o nascondi il livello Avviso |  |
| Registro attività: mostra o nascondi il livello Info |  |
| Registro attività: mostra o nascondi il livello Debug |  |
| Registro attività: mostra o nascondi il livello Trace |  |
| Registro attività: mostra tutti i livelli |  |
| Registro attività: non mostrare nessun livello |  |
| Registro attività: Svuota il registro dell'applicazione |  |
| Registro attività: Scegli l'ambito |  |
| Registro attività: Operazioni git |  |
| Registro attività: Registro dell'applicazione |  |
| Registro attività: Copia l'ultimo errore |  |
| Registro attività: Esegui l'azione suggerita |  |
| Registro attività: Apri l'ultimo errore |  |
| Scheda: Esci e cd qui |  |
| Repository: Clona... | `Ctrl+Shift+N` |
| Repository: Scheda successiva | `Ctrl+PageDown` |
| Repository: Scheda precedente | `Ctrl+PageUp` |
| Scheda: Sposta a sinistra |  |
| Scheda: Sposta a destra |  |
| Repository: Chiudi scheda | `Ctrl+W` |
| Repository: Seleziona scheda... | `t` |
| Cerca: Area di lavoro... | `Ctrl+Alt+F` |
| Modifica: Annulla | `Ctrl+Z` |
| Modifica: Ripeti | `Ctrl+Y` |
| Modifica: Dimentica la voce di annullamento bloccata |  |
| Modifica: Cronologia annullamenti... | `Alt+Z` |
| Modifica: Reflog... |  |
| Modifica: Recupera commit persi... |  |
| Modifica: Ripristina il branch dove l'aveva lasciato l'azione bloccata | `Alt+Shift+Z` |
| Revisione: Mostra panoramica |  |
| Revisione: Mostra file |  |
| Revisione: Mostra Commit |  |
| Revisione: Commenta la riga... | `c` |
| Revisione: Commenta la conversazione... | `C` |
| Revisione: Rispondi alla discussione... | `r` |
| Revisione: Risolvi o riapri la discussione | `x` |
| Revisione: Comprimi o espandi la discussione | `z` |
| Revisione: Modifica il commento in sospeso... | `e` |
| Revisione: Elimina il commento in sospeso | `d` |
| Revisione: Segna il file come visto | `v` |
| Revisione: File successivo | `]` |
| Revisione: File precedente | `[` |
| Revisione: Termina la revisione... | `f` |
| Revisione: Scarta la revisione in sospeso... |  |
| Revisione: Aggiorna | `F5` |
| Revisione: Apri nel browser | `o` |
| Revisione: Checkout del branch | `b` |
| Revisione: Chiudi | `Esc` |
| Revisione: Modifica assegnatari... | `a` |
| Revisione: Modifica revisori... | `p` |
| Revisione: Modifica etichette... | `l` |
| Revisione: Imposta milestone... | `m` |
| Revisione: Scegli il patch set... | `P` |
| Revisione: Confronta i patch set... | `D` |
| Revisione: Esegui il submit della richiesta... | `U` |
| Revisione: Abbandona la richiesta... | `Q` |
| Revisione: Layout unificato o affiancato | `V` |
| Revisione: File intero o solo hunk | `H` |
| Revisione: Mostra o nascondi le modifiche di soli spazi | `S` |
| Revisione: Attiva o disattiva l'a capo automatico | `W` |
| Revisione: Hunk successivo | `J` |
| Revisione: Hunk precedente | `K` |
| Revisione: Scegli la codifica... | `E` |
| Revisione: Mostra o nascondi la minimappa | `M` |
| Revisione: Scorri il diff a sinistra | `Left` |
| Revisione: Scorri il diff a destra | `Right` |
| Revisione: Copia il testo selezionato o la riga | `Ctrl+C` |
| Bacheca: Sposta la card in... | `m` |
| Bacheca: Sposta la card a sinistra |  |
| Bacheca: Sposta la card a destra |  |
| Bacheca: Apri la card | `Enter` |
| Bacheca: Modifica assegnatari... |  |
| Bacheca: Modifica etichette... |  |
| Bacheca: Imposta milestone... |  |
| Bacheca: Nuova issue in questa colonna | `n` |
| Bacheca: Filtra per assegnatario... | `a` |
| Bacheca: Filtra per etichetta... | `l` |
| Bacheca: Filtra per milestone... | `M` |
| Bacheca: Filtra per testo | `/` |
| Bacheca: Azzera i filtri | `x` |
| Bacheca: Imposta le etichette delle colonne... | `c` |
| Bacheca: Aggiorna | `r` |
| Bacheca: Chiudi | `Esc` |
| Revisione: ✦ Bozza di revisione... | `s` |
| Revisione: Accetta il suggerimento | `y` |
| Revisione: Modifica e accetta il suggerimento... | `w` |
| Revisione: Ignora il suggerimento | `n` |
| Revisione: Accetta tutti i suggerimenti | `A` |
| Revisione: Ignora tutti i suggerimenti | `N` |
| Revisione: Usa il riepilogo IA nel testo della revisione | `i` |
| Revisione: ✦ Usa la bozza... | `u` |
| Revisione: Modifica il testo della ✦ bozza... | `t` |
| Revisione: Scarta la ✦ bozza | `N` |
| Cronologia annullamenti: Annulla fino a qui... | `u` |
| Cronologia annullamenti: Ripeti fino a qui... | `y` |
| Cronologia annullamenti: Ripristina solo questa... | `r` |
| Cronologia annullamenti: Ripristina il branch dove si trovava... | `o` |
| Cronologia annullamenti: Aggiorna |  |
| Cronologia annullamenti: Chiudi | `Esc` |
| Reflog: Mostra nel Log | `s` |
| Reflog: Crea branch qui... | `b` |
| Reflog: Ripristina a questa posizione... | `r` |
| Reflog: Carica altro | `m` |
| Reflog: Aggiorna |  |
| Lavoro perso: Mostra nel Log | `s` |
| Lavoro perso: Crea branch qui... | `b` |
| Lavoro perso: Cherry-pick | `c` |
| Lavoro perso: Cerca di nuovo |  |
| Vista: Aggiorna | `r` |
| Vista: Aggiorna (fetch, pull request, issue) | `F5` |
| Vista: Mostra o nascondi l'aiuto | `?` |
| Vista: Mostra o nascondi la colonna Ref |  |
| Vista: Mostra o nascondi la colonna Grafo |  |
| Vista: Mostra o nascondi la colonna Messaggio |  |
| Vista: Mostra o nascondi la colonna Autore |  |
| Vista: Mostra o nascondi la colonna Data |  |
| Vista: Mostra o nascondi la colonna Modifiche |  |
| Vista: Mostra o nascondi la colonna SHA |  |
| Vista: Colonna del Log: Scegli una colonna da spostare o ridimensionare | `C` |
| Vista: Colonna del Log: Scegli la colonna successiva | `Tab` |
| Vista: Colonna del Log: Scegli la colonna precedente | `Shift+BackTab` |
| Vista: Colonna del Log: Sposta la colonna a sinistra | `<` |
| Vista: Colonna del Log: Sposta la colonna a destra | `>` |
| Vista: Colonna del Log: Allarga la colonna | `]` |
| Vista: Colonna del Log: Restringi la colonna | `[` |
| Vista: Colonna del Log: Reimposta le larghezze delle colonne | `0` |
| Vista: Log: Scegli le colonne da mostrare |  |
| Diff: Apri il diff del file selezionato | `Enter` |
| Diff: Chiudi la vista diff | `Esc` |
| Diff: Hunk successivo | `n` |
| Diff: Hunk precedente | `p` |
| Diff: Stage dell'hunk sotto il cursore | `Space` |
| Diff: Unstage dell'hunk sotto il cursore | `Space` |
| Diff: Seleziona righe (o annulla la selezione) | `v` |
| Diff: Stage delle righe selezionate | `Enter` |
| Diff: Unstage delle righe selezionate | `Enter` |
| Diff: Scarta l'hunk sotto il cursore... | `x` |
| Diff: Scarta le righe selezionate... | `X` |
| Copia: Testo selezionato (prima trascina sul testo) | `Ctrl+C` |
| Copia: Riga sotto il cursore (o le righe selezionate) | `y` |
| Diff: Copia l'hunk sotto il cursore |  |
| Diff: Layout inline (+/- in una colonna) | `m` |
| Diff: Layout split (prima e dopo) | `m` |
| Diff: Solo hunk (file intero / solo le modifiche) | `H` |
| File: Percorso/Albero | `m` |
| File: Vedi tutti i file | `a` |
| Commit: Attiva o disattiva «Salta gli hook» | `H` |
| File: Apri il menu della riga o della selezione | `.` |
| File: Espandi tutte le cartelle | `=` |
| File: Comprimi tutte le cartelle | `-` |
| File: Espandi la cartella selezionata e le sue sottocartelle | `*` |
| File: Comprimi la cartella selezionata e le sue sottocartelle | `_` |
| File: Seleziona tutto |  |
| File: Annulla la selezione |  |
| File: Attiva/disattiva la selezione della riga corrente |  |
| Log: Copia come Mermaid gitGraph... | `Y` |
| Log: Confronta la selezione | `v` |
| Log: Inverti gli estremi del confronto | `x` |
| Log: vai alla riga confrontata successiva | `n` |
| Diff: Attiva o disattiva l'a capo automatico | `w` |
| Diff: Scegli la codifica... | `C` |
| Vista: Mostra o nascondi la minimappa | `M` |
| Vista: Mostra o nascondi la colonna dei commit | `Alt+C` |
| Vista: Colonna dei commit: nascondi |  |
| Vista: Colonna dei commit: cronologia del file |  |
| Vista: Colonna dei commit: tutti i commit |  |
| Vista: Porta il focus sulla colonna dei commit |  |
| Vista: Allarga la colonna dei commit |  |
| Vista: Restringi la colonna dei commit |  |
| Vista: Colonna dei commit: scorri a sinistra | `Shift+Left` |
| Vista: Colonna dei commit: scorri a destra | `Shift+Right` |
| Vista: Colonna dei commit: dimensiona la parte successiva | `Tab` |
| Vista: Colonna dei commit: sposta la parte a sinistra | `<` |
| Vista: Colonna dei commit: sposta la parte a destra | `>` |
| Vista: Colonna dei commit: allarga la parte | `]` |
| Vista: Colonna dei commit: restringi la parte | `[` |
| Vista: Colonna dei commit: reimposta le larghezze delle parti | `0` |
| Vista: Colonna dei commit: menu della riga selezionata | `.` |
| Vista: Colonna dei commit: scegli le colonne |  |
| Vista: Colonna dei commit: mostra o nascondi Ref |  |
| Vista: Colonna dei commit: mostra o nascondi Grafo |  |
| Vista: Colonna dei commit: mostra o nascondi Autore |  |
| Vista: Colonna dei commit: mostra o nascondi Data |  |
| Vista: Colonna dei commit: mostra o nascondi Modif. |  |
| Vista: Colonna dei commit: mostra o nascondi SHA |  |
| Vista: Colonna dei commit: mostra la riga selezionata nel Log |  |
| Vista: Colonna dei commit: mostra le modifiche del file alla riga selezionata |  |
| Vista: Colonna dei commit: mostra il file com'era alla riga selezionata |  |
| Vista: Colonna dei commit: blame del file alla riga selezionata |  |
| Editor: Attiva o disattiva segui la selezione |  |
| Immagine: Ingrandisci | `+` |
| Immagine: Riduci | `-` |
| Immagine: Adatta alla vista | `0` |
| Immagine: Grandezza reale (1:1) | `1` |
| Immagine: Alterna prima / dopo | `Tab` |
| Immagine: Allarga pannello Prima | `Alt+Right` |
| Immagine: Allarga pannello Dopo | `Alt+Left` |
| Immagine: Ripristina dimensioni pannelli | `Alt+0` |
| Immagine: Modifica in... | `O` |
| Immagine: Modifica nell'editor predefinito | `o` |
| Cronologia del file: Mostra nel Log | `Enter` |
| Cronologia del file: Copia lo SHA del commit | `y` |
| Cronologia del file: Copia il riepilogo | `Y` |
| Cronologia del file: Mostra il diff di questo commit | `d` |
| Cronologia del file: Apri il file a questo commit | `o` |
| Cronologia del file: Blame a questo commit | `b` |
| Cronologia del file: Confronta con la directory di lavoro | `c` |
| Cronologia del file: Ripristina questa versione | `R` |
| Cronologia del file: Estendi la selezione verso l'alto | `Shift+Up` |
| Cronologia del file: Estendi la selezione verso il basso | `Shift+Down` |
| Cronologia del file: Passa tra l'elenco e il diff | `Tab` |
| Cronologia del file: Allarga l'elenco | `Alt+Right` |
| Cronologia del file: Restringi l'elenco | `Alt+Left` |
| Cronologia del file: Reimposta la larghezza dell'elenco | `Alt+0` |
| File: Apri con... | `O` |
| File: Apri nell'app predefinita | `o` |
| Diff: Mostra o nascondi le modifiche di soli spazi | `s` |
| ✦ IA: Genera il messaggio di commit | `Ctrl+G i` |
| ✦ IA: Genera il messaggio di stash |  |
| ✦ IA: Inserisci variabile del prompt |  |
| ✦ IA: Modifica profilo... |  |
| ✦ IA: Apri l'agente... |  |
| ✦ IA: Spiega il commit selezionato | `Ctrl+G w` |
| ✦ IA: Dividi il commit selezionato... |  |
| ✦ IA: Spiega le modifiche in corso | `Ctrl+G w` |
| ✦ IA: Componi commit dalle modifiche in corso... | `Ctrl+G C` |
| Compositore: Sposta il file in un commit... | `m` |
| Diff: Blame |  |
| Blame: Mostra il commit nel Log | `Enter` |
| Blame: Copia lo SHA del commit | `Y` |
| Blame: Diff del commit per questo file | `d` |
| Blame: Blame sul genitore del commit | `b` |
| Blame: Apri la cronologia del file | `H` |
| Blame: Mostra o nascondi la scheda | `i` |
| Blame: Hunk successivo | `n` |
| Blame: Hunk precedente | `p` |
| Blame: Hunk successivo dello stesso commit | `N` |
| Blame: Hunk precedente dello stesso commit | `P` |
| Blame: Vai alla riga | `g` |
| Blame: Allarga la colonna delle annotazioni | `>` |
| Blame: Restringi la colonna delle annotazioni | `<` |
| Diff: Cronologia del file |  |
| Diff: Vista file |  |
| File: Modifica nella directory di lavoro | `e` |
| Editor: Cambia codifica |  |
| Editor: Riapri con codifica |  |
| Editor: Converti in codifica |  |
| Editor: Converti interruzioni di riga |  |
| Editor: Cambia rientro |  |
| App: Impostazioni... | `Ctrl+, / Ctrl+N ,` |
| App: Cerca nelle impostazioni... | `/ nelle Impostazioni` |
| App: Scorciatoie da tastiera... |  |
| Agente: Mostra la vista in sospeso | `Ctrl+N Enter` |
| Agente: Chiudi la vista in sospeso |  |
| Agente: Indietro | `Ctrl+N Backspace` |
| Scorciatoie: Reimposta tutto... |  |
| Integrazioni: Apri la pull request collegata | `Ctrl+G g` |
| Integrazioni: Modifica la pull request collegata sull'host | `e (su una riga)` |
| Integrazioni: Copia l'URL della pull request collegata |  |
| Integrazioni: Aggiorna pull request e issue |  |
| Integrazioni: Checkout della pull request... |  |
| Integrazioni: Apri il riferimento del commit (#issue / !richiesta)... |  |
| Integrazioni: Apri issue... |  |
| Hosting: Rivedi la richiesta... |  |
| Hosting: Push per la revisione... |  |
| Hosting: Modifica assegnatari... |  |
| Hosting: Modifica revisori... |  |
| Hosting: Modifica etichette... |  |
| Hosting: Imposta milestone... |  |
| Issue locali: Chiudi issue... |  |
| Issue locali: Riapri issue... |  |
| Issue locali: Apri in Obsidian... |  |
| Issue: Apri la bacheca |  |
| Issue locali: Crea Base di Obsidian |  |
| Pull request: Crea pull request... | `n` |
| Hosting: Nuova issue... | `n` |
| Pull request: Cerca | `Enter sulla riga di ricerca` |
| Pull request: Scegli il filtro (Aperte / Merged / Tutte)... | `v` |
| Pull request: Scegli il provider... | `Enter sulla riga del provider` |
| Pull request: Scegli il repository... | `Enter sulla riga del repository` |
| Pull request: Scollega l'origine (nessuna) |  |
| Issue: Scegli il provider... | `Enter sulla riga del provider` |
| Issue: Scollega l'origine (nessuna) |  |
| Integrazioni: Scollega il profilo (automatico) |  |
| Issue: Scegli il repository... | `Enter sulla riga del repository` |
| Issue: Cerca | `Enter sulla riga di ricerca` |
| Issue: Scegli il filtro... | `v` |
| Issue: Modifica query / elenchi... |  |
| Commit: Inserisci riferimento a una issue... |  |
| Integrazioni: Usa un profilo per questo repository... |  |
| Integrazioni: Gestisci il profilo di questo repository |  |
| Integrazioni: Connetti un'integrazione... |  |
| Impostazioni: Opzioni del Radar di un profilo... |  |
| App: Esci | `q` |
| App: Tema... |  |
| App: Cambia tema... | `Ctrl+N y` |
| App: Cambia lingua... | `Ctrl+N g` |
| Launcher: Crea collegamento |  |
| Launcher: Crea collegamento sul desktop |  |
| Launcher: Rimuovi collegamenti |  |
| Launcher: Crea alias del comando |  |
| Launcher: Rimuovi alias del comando |  |
| Launcher: Aggiungi al PATH |  |
| Launcher: Rimuovi dal PATH |  |
| App: Cambia tema colori del grafo... | `Ctrl+N Y` |
| App: Attiva/disattiva tinta righe del grafo | `Ctrl+N T` |
<!-- palette:end -->

## Spostarsi

| Tasto | Azione |
|---|---|
| `Tab` / `Shift+Tab` | Porta il focus sul pannello successivo / precedente |
| `Ctrl+N`, poi un badge | Porta il focus su quella parte della finestra (vedi [Prefisso di navigazione](#prefisso-di-navigazione-ctrln)) |
| `j` / `k`, `Su` / `Giù` | Spostano la selezione, in ogni pannello |
| `PageUp` / `PageDown`, `Home` / `End` | Si spostano di una pagina; vanno alla prima / ultima riga |
| un numero, poi `j` / `k` / `Su` / `Giù` | Prefisso di conteggio: `4j` si sposta di quattro righe. Con `PageUp` / `PageDown` il conteggio è un numero di pagine. La barra di stato mostra il conteggio in sospeso (`4...`). `Esc` o qualsiasi altro tasto lo azzera. Uno `0` iniziale non avvia mai un conteggio |
| `h` / `l`, `Sinistra` / `Destra` | Pannello Refs: cambia sezione. Log: esce (`h`) o entra (`l`) nell'elenco dei file modificati di un commit |
| `?` | Apre o chiude l'aiuto (vedi [Aiuto integrato](#aiuto-integrato-)) |
| `r` | Aggiorna |
| `F5` | Fa il fetch di ogni remoto, ricarica le pull request e le issue, e aggiorna |
| `q` | Esce |
| `Ctrl+Z` | **Annulla** l'ultima operazione. Vedi [Annulla e ripeti](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Ripeti**. `Ctrl+Shift+Z` richiede il protocollo tastiera kitty; `Ctrl+Y` funziona sempre |
| `.`, `Menu`, `Shift+F10` | Apre il menu contestuale della riga selezionata |

`Ctrl+Z` e `Ctrl+Y` non fanno nulla mentre un campo di testo ha la tastiera.
L'editor ha il proprio annulla per il testo.

## Elenchi popup filtrabili

Tutti gli elenchi popup filtrabili usano gli stessi tasti: la palette dei comandi
e i suoi selettori (tema, lingua, colori del grafo), il selettore di schede, e
ogni elenco a discesa con una riga di ricerca (le scelte delle Impostazioni, il
modulo delle pull request, il selettore dei riferimenti alle issue, l'elenco
**Apri in**). Scrivi per filtrare l'elenco. Questi tasti spostano
l'evidenziazione:

| Tasto | Azione |
|---|---|
| `Su` / `Giù`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Spostano di una riga |
| `Home` / `End` | Prima / ultima riga |
| `PageUp` / `PageDown` | Spostano di una pagina visibile |
| `Enter` | Sceglie la riga evidenziata |
| `Esc` | Chiude |

In questi popup, `Home`, `End` e `Ctrl+K` spostano l'elenco, non il cursore del
campo di filtro. `Ctrl+A` e `Ctrl+E` spostano comunque il cursore all'inizio e
alla fine. I semplici `j` e `k` vengono digitati nel filtro.

Con il mouse:

- La rotellina **fa scorrere** l'elenco. Non sposta l'evidenziazione.
- Un clic **seleziona** una riga. Un doppio clic la **sceglie**, come `Enter`.
- Un elenco più alto del suo popup mostra una barra di scorrimento sul bordo
  destro. Puoi trascinarla.
- Un breve elenco a discesa senza riga di ricerca (due o tre scelte) sceglie al
  primo clic.

**Anteprima dal vivo.** I selettori di tema, tema colori del grafo e lingua
applicano subito la riga evidenziata, dietro il popup. `Enter` la mantiene.
`Esc`, o un clic fuori dal popup, ripristina il valore precedente.

**Filtro dei temi.** Il selettore dei temi ha una riga sotto il campo di filtro:
**Tutti**, **Scuri**, **Chiari** e **Colorati**. `Tab` / `Shift+Tab` passano al
successivo / precedente. Da `Alt+1` ad `Alt+4` ne scelgono uno direttamente.
Puoi anche cliccarli.

## Aiuto integrato (`?`)

`?`, `Ctrl+N ?` o il comando della palette `Vista: Mostra o nascondi l'aiuto`
aprono l'aiuto. Elenca i tasti e i gesti del mouse di ogni parte dell'app. Si
apre sopra qualsiasi cosa, schermata Impostazioni compresa.

| Tasto | Azione |
|---|---|
| `?` | Chiude l'aiuto (e azzera il filtro) |
| *(scrivi)* | Filtra: tiene solo le sezioni che contengono il testo. Anche `j`, `k` e `q` vengono digitati |
| `Backspace`, `Ctrl+U` | Cancellano l'ultimo carattere / azzerano il filtro |
| `Esc` | Azzera il filtro; se è vuoto, chiude l'aiuto |
| `Su` / `Giù`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Scorrono di una riga |
| `PageUp` / `PageDown`, `Home` / `End` | Scorrono di una pagina; vanno in cima / in fondo |
| `Ctrl+D`, o un clic sul link | Apre questa documentazione su GitHub nel browser |

La rotellina fa scorrere l'aiuto e puoi trascinarne la barra di scorrimento. Un
clic fuori dall'aiuto lo chiude.

## Prefisso di navigazione (`Ctrl+N`)

`Ctrl+N` attende un altro tasto. La barra di stato mostra i tasti che accetta. Il
tasto successivo esegue un'azione, oppure annulla.

| Tasto | Azione |
|---|---|
| `Ctrl+N` poi un badge | Porta il focus sulla parte della finestra che mostra quel badge |
| `Ctrl+N h` / `Ctrl+N l` | Comprimono o ripristinano il pannello Refs / il pannello destro |
| `Ctrl+N +` / `Ctrl+N -` | Ingrandiscono / riducono di una riga la sezione Refs attiva |
| `Ctrl+N ]` / `Ctrl+N [` | Ingrandiscono / riducono di una riga il blocco del messaggio di commit del pannello destro |
| `Ctrl+N Left` / `Ctrl+N Right` | Spostano di un passo a sinistra / a destra il divisore tra il pannello Refs e il Log (un pannello Refs compresso torna prima) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Spostano di un passo a sinistra / a destra il divisore tra il Log e il pannello destro |
| `Ctrl+N t` | Sceglie una scheda (il selettore di schede; `t` da solo fa lo stesso) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Cambiano il tema / la lingua / il tema colori del grafo |
| `Ctrl+N Shift+T` | Attiva o disattiva la tinta delle righe del grafo |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Mostrano la vista preparata da un agente / tornano indietro (vedi [MCP e agenti](mcp-and-agents.md)) |
| `Ctrl+N ,` | Apre le Impostazioni |
| `Ctrl+N ?` | Apre o chiude l'aiuto |
| `Esc`, o qualsiasi altro tasto | Annulla |

### Badge

Mentre il prefisso attende, ogni parte della finestra che può ricevere il focus
mostra un badge di un carattere sul bordo in alto a sinistra. Premi quel
carattere per darle il focus. I badge vengono riassegnati ogni volta, in ordine
di schermo: prima da `1` a `9`, poi `a`, `b`, `c`... Quindi leggi il badge sullo
schermo: una parte non ottiene sempre lo stesso badge.

Parti con un badge: il filtro Refs e ogni sezione Refs visibile, il Log (o la
vista Diff), la vista File quando è aperta, gli elenchi File non in stage e File
in stage, il campo del riepilogo del compositore di commit, e i controlli
dell'elenco dei file del commit. Nella scheda Home i badge sono il campo di
ricerca, le quattro sezioni e i pulsanti della barra degli strumenti (il badge di un pulsante lo preme). Nella scheda Radar sono il
campo del filtro, le caselle combinate Ambito, Vista e Mostra (il badge di una
casella combinata ne apre l'elenco), i pulsanti Aggiorna e Indietro e ogni
titolo di sezione sullo schermo. Il badge di una sezione sposta la selezione sul
suo titolo; `Enter` o `Space` la comprime poi.

Nel selettore di cartelle Sfoglia e nelle finestre Clona, Init, Crea worktree, Area di lavoro, Checkout parziale, Flow, Nuova issue e pull request, i badge sono i campi, le caselle di controllo e i pulsanti (il badge di un pulsante lo preme, quello di un campo vi porta il cursore; in un elenco, un badge è sulla prima riga; il selettore di cartelle Sfoglia ha un badge per l'elenco Accesso rapido e uno per l'elenco Cartelle, accanto alla prima riga di ciascun elenco, e premerne uno porta la tastiera in quell'elenco senza scegliere una riga). Nel Registro attività sono le due schede, le caselle Ambito e Livello, il filtro, le caselle dei livelli, i pulsanti e la prima voce dell'elenco. Un badge fa ciò che farebbe un clic su quel controllo. Nel selettore di cartelle `Ctrl+N` avvia i badge, quindi il suo tasto Nuova cartella è `Alt+N`.

Nella finestra Modifica (e Modifica messaggio del commit) i badge sono il riepilogo, la descrizione, il nome, l'e-mail e la data dell'autore, **Modifica la cronologia in profondità...**, **Annulla**, **Applica** e la croce di chiusura; nel pannello di riscrittura della cronologia sono la casella dell'ambito, le sue caselle di controllo e il pulsante dei ref, il nome della ricetta, ogni regola, la sua casella e i suoi pulsanti, il pulsante **+ Aggiungi regola**, i campi della regola selezionata, le schede dell'anteprima, il campo del filtro, le righe dei branch protetti con Seleziona tutti / Deseleziona tutti, e i pulsanti Anteprima, Applica, Annulla, Salva, Carica e Crea regole con l'IA (le pagine Segreti, Identità, Dimensione e risultato aggiungono i propri pulsanti e righe; le righe della scheda Commit, essendo un elenco, non ne hanno). Un badge fa ciò che farebbe un clic. Non viene mostrato alcun badge mentre sono aperti un menu a tendina, una domanda, un campo in cui si sta scrivendo, il selettore di cartelle, la schermata di eliminazione, l'editor di script, le finestre IA, il diff del blob o la guida dell'host, né mentre la finestra Modifica chiede "scartare?"; `Esc` toglie i badge senza chiudere nulla.

Nelle Impostazioni i badge sono la casella di ricerca, l'elenco delle pagine (un solo badge, sulla pagina corrente), le sotto-schede, l'elenco con i suoi pulsanti, ogni campo della pagina visibile sullo schermo e la croce di chiusura. Un badge fa ciò che farebbe un clic su quel controllo: un campo di testo si apre in modifica, una casella si spunta, un menu a tendina si apre, un pulsante viene eseguito. `Ctrl+N` mantiene il proprio significato mentre si scrive in un campo o è aperto un menu a tendina, un editor o la ricerca.

La palette ha le stesse azioni, per esempio
`Navigazione: Vai a Refs > Locali` o `Navigazione: Vai a Stato > In stage`.

## Repository e schede

| Tasto | Azione |
|---|---|
| `Ctrl+O`, o un clic su `⌂` | Va alla scheda Home (nella scheda Home: dà il focus al campo di ricerca) |
| `Ctrl+Shift+N` | Apre la scheda Home con la finestra Clona |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Scheda successiva / precedente |
| `t` | Sceglie una scheda da un elenco filtrabile |
| `Ctrl+W` | Chiude la scheda corrente (chiede prima; il repository non viene modificato) |
| `Ctrl+T`, o un clic destro su una scheda | Il menu della scheda (le sue voci sono elencate in [Barra delle schede](interface-tour.md#barra-delle-schede)) |
| `Alt+O` / `Ctrl+Alt+O` | Aprono la cartella della scheda: scegli dall'elenco **Apri in** / usa la voce predefinita. Vedi [Apri in](open-in.md) |

### La scheda Home

La scheda `⌂` gestisce i tuoi repository. Ha quattro sezioni: **Repository
aperti**, **Preferiti**, **Repository recenti** e **Tutti i repository**. Vedi il
[Tour dell'interfaccia](interface-tour.md#home).

| Tasto | Azione |
|---|---|
| `j` / `k`, frecce, `PageUp` / `PageDown`, `Home` / `End` | Si spostano |
| `Enter`, o un doppio clic | Apre il repository (o passa alla sua scheda). Su un'intestazione di sezione: la comprime o la espande |
| `f` o `*`, o un clic sulla stella | Aggiungono / rimuovono dai preferiti |
| `x`, `Delete`, o il pulsante della riga | Chiude la sua scheda (Aperti), lo rimuove dall'elenco (Recenti), toglie la stella (Preferiti), lo dimentica (Tutti). Nulla viene eliminato dal disco |
| `.`, un clic destro, o il `⋮` della riga | Il menu della riga |
| `/` o `Ctrl+F` | Cerca in tutte le sezioni per nome, branch o percorso. `Enter` apre la prima corrispondenza; `Esc` azzera la ricerca, poi esce dal campo |
| `Space`, `Sinistra` / `Destra` | Comprimono / espandono la sezione corrente |
| `-` / `+` | Comprimi tutto / Espandi tutto |
| `b` / `c` / `i` | Sfoglia (apre una cartella) / Clona / Init |
| `Esc` | Azzera la ricerca, o torna alla scheda del repository attivo |
| `q` | Esce |

`Ctrl+P`, `Ctrl+,`, `?` e `Ctrl+N` funzionano nella scheda Home. `Ctrl+G`,
`Ctrl+W` e `Ctrl+T` richiedono una scheda di repository: nella scheda Home
mostrano solo un messaggio. I comandi della palette che richiedono un repository
sono in grigio.

### Il selettore di cartelle

**Sfoglia** (e il pulsante **Sfoglia...** delle finestre Init, Clona e Crea
worktree) apre il selettore di cartelle. Ha un campo del percorso in alto, i
luoghi e l'albero delle cartelle a sinistra, e il contenuto della cartella
corrente a destra.

| Tasto | Azione |
|---|---|
| `Tab` / `Shift+Tab` | Campo del percorso, poi la colonna di sinistra, poi il contenuto |
| `Su` / `Giù`, `PageUp` / `PageDown`, `Home` / `End` | Si spostano nella colonna con il focus. Nel campo del percorso: scorrono i completamenti |
| una lettera (contenuto) | Va alla cartella successiva che inizia con questa lettera |
| `Enter` / `Destra` | Apre la cartella selezionata. Nel campo del percorso: va alla cartella digitata; sulla cartella già mostrata, la sceglie |
| `Backspace` / `Sinistra` / `Alt+Up` | Cartella superiore (nel campo del percorso `Backspace` cancella un carattere) |
| `Tab` (campo del percorso) | Accetta l'unico completamento, o quello evidenziato |
| `Alt+N` | Nuova cartella (`Ctrl+N` avvia i badge, vedi [Badge](#badge)) |
| `Alt+H` | Mostra o nasconde le cartelle nascoste |
| `Ctrl+F`, o `/` fuori dal campo del percorso | Filtra le voci della cartella mentre scrivi (senza distinguere maiuscole e minuscole; `..` resta; l'intestazione conta «x di n»; `Enter` salta alla prima corrispondenza, `Esc` azzera; si svuota quando cambi cartella). Ci si arriva anche con `Tab` o con un clic |
| `Ctrl+S`, `Ctrl+Enter`, o il pulsante di conferma | Sceglie la sottocartella selezionata, altrimenti la cartella mostrata |
| `Esc` | Chiude i completamenti, oppure annulla |

Quando il campo del percorso mostra un nuovo percorso, l'intero percorso è
selezionato: il primo tasto che digiti lo sostituisce.

### Finestre Init e Clona

| Tasto | Azione |
|---|---|
| `Tab` / `Shift+Tab`, `Su` / `Giù` | Campo o pulsante successivo / precedente |
| `Enter` | Preme il pulsante con il focus, oppure crea / clona |
| `Space` | Attiva la casella con il focus, o preme il pulsante con il focus |
| `Ctrl+B` | Sfoglia per la cartella |
| `j` / `k`, `g` / `G` | Nell'elenco delle sorgenti e nell'elenco dei repository della finestra Clona: giù / su, primo / ultimo (non nel campo di ricerca) |
| `Esc` | Chiude. Durante un clone, il clone continua in background |

Nella finestra Clona, la colonna di sinistra elenca **Clona con URL** e una
sorgente per ogni profilo di hosting connesso (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Scegli una sorgente con `Su` / `Giù` o con un clic. Poi scrivi nel
campo di ricerca **Repository**, spostati nell'elenco con le frecce,
`PageUp` / `PageDown` o la rotellina, e scegli una riga con `Enter` o con un
clic. Per i campi vedi [Per iniziare](getting-started.md).

## Pannello Refs

| Tasto | Azione |
|---|---|
| `Enter` / `Space` | Espandono o comprimono la sezione attiva, o la cartella selezionata |
| `h` / `l`, `Sinistra` / `Destra` | Cambiano sezione |
| `/` | Filtra per nome (`Enter` applica, `Esc` annulla) |
| `y` | Attiva o disattiva la vista gerarchica dei branch: cartelle ricavate dai `/` nei nomi dei branch (LOCALI e REMOTI) |
| `o` | Mette in **Solo** il ref selezionato, o ogni ref sotto la cartella o il remoto selezionato. Metti in solo più ref per vederne l'unione |
| `Shift+O` | **Termina il solo**: azzera ogni solo |
| `Shift+H` | **Mostra tutto**: rende di nuovo visibile ogni ref nascosto |
| `g` | **Mostra nel grafo**: scorre il Log fino al commit del ref (o dello stash) selezionato e fa lampeggiare la riga; la selezione non si sposta |
| `m` | **Segna** il ref selezionato come origine del rilascio (`Esc`, o di nuovo `m` su di esso, toglie il segno) |
| `Shift+M` | **Rilascia il ref segnato** sul ref selezionato: apre il menu che apre un trascinamento (Merge, Rebase, Push, Fast-forward...) |
| `+` (sezione REMOTI) | Aggiunge un remoto |
| `.`, `Menu`, `Shift+F10`, o un clic destro | Il menu della riga: un branch, un tag, il nome di un remoto o una cartella |
| `Enter` su una riga WORKTREES | Apre quel worktree in una scheda propria |
| `Ctrl+G d` | Elimina il branch o il tag selezionato (chiede prima). Su una riga REMOTI: elimina quel branch remoto |

Per ciò che fanno i menu, vedi [Lavorare con il Log](log-and-history.md).

### Pull request e issue

Con un'integrazione corrispondente, il pannello Refs ha una sezione delle pull
request (GitLab: merge request). Con un profilo di hosting connesso ha anche una
sezione ISSUE. Vedi
[Integrazioni di hosting e issue tracker](hosting-integration.md#5-pull-request-e-issue-nella-barra-laterale).

| Tasto | Azione |
|---|---|
| `Enter` | Apre la riga selezionata nel browser. Su una riga `Riconnetti`: apre le Impostazioni su quel profilo |
| `Space` | Espande o comprime la sezione |
| `v` | Apre il filtro della sezione. Pull request: Aperte, Merged / chiuse, Tutte. Issue: Aperte, Assegnate a me, Chiuse, Tutte |
| `n` | Sezione delle pull request: crea una pull request. Sezione ISSUE: nuova issue. (Il pulsante `+` dell'intestazione) |
| `e` | Sezione delle pull request: modifica la richiesta selezionata sull'host (la matita della riga) |
| `Enter` sulla riga di ricerca | Scrivi nel campo di ricerca. `Enter`, `Tab` o `Esc` lo lasciano; `Ctrl+U` lo azzera |
| `Enter` su un'intestazione di gruppo | Comprime o espande il gruppo (*Le mie pull request*, *Assegnate a me*, *In attesa della mia revisione*, *Tutte le pull request*) |
| `Enter` sulla riga del filtro, del provider o del repository | Apre quell'elenco a discesa. L'elenco dei provider termina con **Nessuno (scollega questo repository)** |
| `.`, o un clic destro | Il menu della riga: apri nel browser, copia l'URL o il numero, fai il checkout del branch di origine (pull request); crea un branch dall'issue (issue) |

### La bacheca delle issue

**Bacheca** nell'intestazione ISSUE (palette: **Issue: Apri la bacheca**) apre la bacheca Kanban della sorgente delle issue. Vedi
[la bacheca](hosting-integration.md#5b-la-bacheca-delle-issue) e [le issue locali](local-issues.md#la-bacheca-kanban). Ogni tasto
qui sotto è anche un comando della palette (**Bacheca: ...**).

| Tasto | Azione |
|---|---|
| Frecce | Selezionano una card (`Su`, `Giù`) o la colonna vicina (`Sinistra`, `Destra`); `Home`, `End`, `PageUp`, `PageDown` saltano nella colonna |
| `Shift+Left`, `Shift+Right` | Spostano la card selezionata nella colonna precedente o successiva |
| `m` | **Sposta la card in...**: un elenco a discesa delle colonne |
| `Enter` | Apre la issue (la nota, per una issue locale; la pagina dell'host altrimenti) |
| `e`, `.` | Modificano assegnatari, etichette o milestone; il menu della card |
| `n` | Nuova issue nella colonna selezionata |
| `a`, `l`, `M`, `/` | Filtrano per assegnatario, etichetta, milestone, testo. `x` azzera i filtri |
| `c` | Le etichette che sono le colonne di una bacheca GitHub / GitLab |
| `r` | Rilegge le issue |
| `Esc` | Chiude la bacheca |

Mouse: trascina una card su un'altra colonna per spostarla, clicca una card per selezionarla, fai doppio clic per aprirla, clic destro
per il menu della card, rotellina per scorrere una colonna, `Shift`+rotellina (o la barra sotto la bacheca) per scorrere le colonne.

### Flow (workflow di branching)

Con un workflow configurato per il repository (vedi [workflow di branching](branching-workflows.md)) la barra degli
strumenti ha un pulsante **Flow** e la barra laterale raggruppa LOCALI e REMOTI per tipo di branch. Ogni voce del menu è
un comando della palette (**Flow: ...**; **Flow: Avvia Feature...** e **Flow: Nuova issue Feature...** sono generati per
ogni tipo di topic del workflow). Le voci guidate dall'host (*Nuova issue <Tipo>*, *Prendi in carico un branch creato
dall'host*) e le loro finestre, la card di attesa e la card «è comparso un branch» sono descritte in
[workflow di branching](branching-workflows.md#flussi-guidati-dallhost-nellapp); `Enter` esegue il pulsante con il focus
(*Crea issue*, *Prendi in carico*), `Esc` smette di attendere o chiude.

| Tasto | Azione |
|---|---|
| `Ctrl+G Shift+W` | Apre il menu Flow (il pulsante della barra degli strumenti) |
| `Enter`, `Tab`, `Shift+Tab` | Nella finestra Avvia / Termina / Sincronizza / Promuovi / Imposta genitore: anteprima o esecuzione, campo successivo, campo precedente |
| `Space` | Spunta una casella (*Pubblica*, *Esegui gli hook*, *Aggiorna anche i branch impilati su questo*) |
| `Esc` | Chiude un elenco a discesa, torna dall'anteprima al modulo, chiude la finestra (non mentre un'esecuzione è in corso) |
| Passa il mouse su un branch | Il suo tipo, il genitore e di quanto è indietro rispetto al genitore |

## Pannello Log

| Tasto | Azione |
|---|---|
| `Enter` / `l` | Entra nell'elenco dei file modificati del commit selezionato |
| `Enter` / `l` su una riga di file | Apre il diff del file al centro |
| `Esc` / `h` | Torna ai commit |
| `Destra` | (commit con più di un ref) Apre l'elenco dei suoi altri ref: `Su` / `Giù` si spostano, `Enter` fa il checkout, `Esc` chiude |
| `f` | Apre la barra di ricerca (vedi [Ricerca](#ricerca)) |
| `c` | (con modifiche non committate) Seleziona la riga WIP e mette il cursore nel suo campo del messaggio |
| `Enter` sulla riga WIP | Mette il cursore nel campo del messaggio della riga WIP |
| `.`, `Menu`, `Shift+F10` | Il menu del commit. `Su` / `Giù` si spostano, `Destra` / `l` / `Enter` aprono un sottomenu, `Sinistra` / `h` / `Esc` tornano indietro |
| `m` / `a` | File del pannello destro: `Percorso` o `Albero` / `Vedi tutti i file` |
| `Shift+C` | Scegli una colonna del Log da spostare o ridimensionare (vedi sotto) |

Con una colonna scelta, `Tab` / `Shift+Tab` ne scelgono un'altra, `<` / `>` la spostano, `]` / `[` la allargano o la restringono, `0` riporta ogni larghezza all'automatico e `Esc` o `Enter` esce (contesto `log_columns` nella tabella generata sopra).

Il campo del messaggio della riga WIP è il riepilogo del compositore di commit.
Finché ha il cursore, ogni tasto viene digitato, ma le combinazioni `Ctrl`
funzionano ancora. `Enter` passa alla descrizione del compositore. `Esc` lascia
il campo e conserva il testo. Vedi [Stage e commit](staging-and-committing.md).

### Selezionare più commit

| Tasto o gesto | Azione |
|---|---|
| `Space` | Aggiunge il commit corrente alla selezione, o lo rimuove |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Estendono la selezione (Alt per i terminali che tengono Shift per sé) |
| `Ctrl+click` / `Shift+click` o `Alt+click` | Aggiungono / tolgono un commit / selezionano un intervallo |
| `Esc` | Azzera la selezione |
| `Shift+Y` | [Copia la selezione come Mermaid gitGraph](log-and-history.md#copiare-i-commit-come-mermaid-gitgraph) (una finestra con livelli di dettaglio e anteprima in tempo reale; `Enter` copia) |

Con più commit selezionati, fai clic destro su uno di essi e scegli
**Cherry-pick di N commit su `<branch>`...**. Si apre la stessa vista di un
rebase interattivo.

### Dare un nome a un branch o a un tag

`Ctrl+G b`, `Ctrl+G t`, i pulsanti **Branch** e **Tag** della barra degli
strumenti e le voci di menu **Crea un branch qui...** / **Crea un tag qui...**
chiedono il nome in una casella di testo sulla riga del commit. `Enter` crea il
ref. `Esc` annulla. Un nome non valido o già esistente è mostrato in rosso, e la
casella resta aperta.

### Modificare il messaggio di un commit

**Modifica il messaggio del commit...** nel menu del commit, `Ctrl+G e`, o il
comando della palette `Git: Modifica il messaggio del commit...` aprono il popup
**Modifica il messaggio del commit**. Per cosa cambia, vedi
[Lavorare con il Log](log-and-history.md).

| Tasto | Azione |
|---|---|
| `Enter` nel riepilogo | Passa alla descrizione |
| `Enter` nella descrizione | A capo. `Su` sulla prima riga torna al riepilogo |
| `Ctrl+Enter`, `Ctrl+S`, o **Aggiorna il messaggio** | Salvano il nuovo messaggio |
| `Tab` / `Shift+Tab` | Si spostano tra riepilogo, descrizione e pulsanti |
| `Esc`, o **Annulla** | Chiudono senza modifiche |

### Modificare un commit

`Ctrl+G Shift+E` (tutti i campi) e `Ctrl+G Shift+A` (l'autore) aprono la finestra di modifica del commit, come anche il menu del commit, le voci `Commit: Modifica...` della palette e il pannello dei dettagli del commit; ogni modo di aprirla è descritto in [Modificare un commit](log-and-history.md#modificare-un-commit-messaggio-autore-date). Entrambe le scorciatoie si possono riassegnare: `git_prefix.amend` e `git_prefix.edit_author`. Tasti dentro la finestra:

| Tasto | Azione |
|---|---|
| `Tab` / `Shift+Tab` | Si spostano tra i campi e i pulsanti |
| `Enter` in un campo a riga singola | Campo successivo; dopo l'ultimo, **Applica** |
| `Su` / `Giù` | Campo precedente / successivo (nella descrizione, dalla prima / ultima riga) |
| `Ctrl+Enter`, `Ctrl+S`, o **Applica** | Applicano i campi modificati |
| `Esc`, o **Annulla** | Chiudono senza modifiche |
| `Alt+H`, o **Modifica la cronologia in profondità...** | Chiude la finestra e apre il pannello di riscrittura della cronologia a partire da questo commit (solo nelle build con la riscrittura della cronologia); chiede prima conferma se hai modificato un campo (`Esc` conferma). Si può riassegnare: `reword.deeper` |

### Crea worktree

**Crea un worktree da `<ref>`...** (menu di branch e tag) e **Crea un worktree
qui...** (menu del commit) aprono una finestra: la cartella, un nuovo branch
facoltativo, e **Apri il nuovo worktree come scheda**. Tasti: `Tab` /
`Shift+Tab` si spostano, `Enter` crea, `Space` attiva la casella, `Ctrl+B`
sfoglia, `Esc` annulla.

### Rebase interattivo

**Rebase interattivo da qui...** nel menu del commit, o
`Git: Rebase interattivo dal commit selezionato...`, apre la vista del rebase. Un
cherry-pick di più commit usa la stessa vista. Vedi
[Lavorare con il Log](log-and-history.md).

| Tasto | Azione |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Impostano l'azione della riga: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup ed Edit solo in un rebase) |
| `Space` | Apre l'elenco delle azioni della riga (`Su` / `Giù` scelgono, `Enter` applica, `Esc` chiude l'elenco) |
| `Su` / `Giù` | Spostano la selezione |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Spostano in su / in giù la riga selezionata |
| `a` | Attiva o disattiva l'anteprima dell'autosquash |
| `Enter` su una riga Reword | Modifica il nuovo messaggio di quel commit |
| `Ctrl+S`, `Ctrl+Enter`, o **Avvia il rebase** / **Avvia il cherry-pick** | Eseguono il piano |
| `Ctrl+R`, o **Reimposta** | Ripristinano il piano com'era all'apertura della vista |
| `Esc`, o **Annulla** | Chiudono senza modifiche |

Quando un rebase si ferma (su un conflitto o su un passo Edit), un banner sopra
il Log offre **Continua**, **Salta** e **Interrompi**. La palette ha
`Rebase: Continua`, `Rebase: Salta` e `Rebase: Interrompi`.

## Ricerca

`f`, il pulsante **Cerca** della barra degli strumenti, o `Navigazione: Cerca commit...`
aprono la barra di ricerca sopra il Log. Cerca nel messaggio completo del commit,
nel nome e nell'email dell'autore, nello SHA e nei nomi dei ref. Non nasconde le
righe: le righe che non corrispondono sono attenuate.

| Tasto | Azione |
|---|---|
| *(scrivi)* | Modifica il testo di ricerca (ogni tasto viene digitato) |
| `Enter` / `Giù`, o il `↑` della barra | Corrispondenza successiva |
| `Shift+Enter` / `Su`, o il `↓` della barra | Corrispondenza precedente |
| `Esc`, o la `✕` della barra | Chiudono la barra di ricerca |

## Vista Diff

Un file modificato si apre al centro, al posto del Log. Vedi
[Stage e commit](staging-and-committing.md).

| Tasto | Azione |
|---|---|
| `Enter` / `l`, o un doppio clic (su una riga di file) | Apre il diff del file |
| `Esc`, `q`, o la `✕` nel titolo | Chiudono il diff |
| `j` / `k`, `Su` / `Giù` | Scorrono |
| `PageUp` / `PageDown`, `Home` / `End` | Scorrono di una pagina; vanno all'inizio / alla fine |
| `h` / `l`, `Sinistra` / `Destra` | Scorrono di lato (non con l'a capo automatico) |
| `n` / `p` | Hunk successivo / precedente |
| `m` | Layout in linea o diviso |
| `Shift+H` | Solo gli hunk o file intero |
| `w` | Attiva o disattiva l'a capo automatico |
| `s` | Mostra o nasconde le modifiche di soli spazi bianchi |
| `Shift+M` | Attiva o disattiva la minimappa |
| `Shift+C` | Sceglie la codifica con cui viene letto il file (un elenco: prima Rilevamento automatico) |
| `e` | Modifica il file nell'editor integrato (vedi [Editor](#editor)) |
| `r` | (file Git LFS) La scheda LFS o il puntatore grezzo. Vedi [Git LFS](lfs.md) |
| `Shift+E` | (file in conflitto) Apre l'editor di merge. Vedi [Risolvere i conflitti di merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copiano il testo selezionato. Senza selezione: la riga sotto il cursore, o le righe selezionate |
| `.`, o un clic destro su un hunk | Il menu dell'hunk |

### Stage e scarto dal diff

Questi tasti funzionano solo su un diff delle modifiche in corso (aperto
dall'elenco File non in stage o File in stage). Lo scarto funziona solo
dall'elenco File non in stage.

| Tasto | Azione |
|---|---|
| `Space` | Stage (File non in stage) / unstage (File in stage) dell'hunk sotto il cursore |
| `v` | Avvia o abbandona una selezione di righe; `j` / `k` la estendono |
| `Enter` | Stage / unstage delle righe selezionate, o della riga sotto il cursore |
| `x` | Scarta l'hunk sotto il cursore (chiede prima: `y` / `Enter` confermano, `n` / `Esc` annullano) |
| `Shift+X` | Scarta le righe selezionate, o la riga sotto il cursore (chiede prima) |

Contano solo le righe modificate: le righe di contesto in una selezione vengono
ignorate. Una selezione che arriva in un secondo hunk agisce solo sul primo hunk.

## Vista immagine

Un'immagine nella vista Diff o nella vista File. I tasti sono nel contesto
`image` della tabella qui sopra. Vedi [Immagini](image-view.md).

## Vista File, Blame e cronologia del file

| Tasto | Azione |
|---|---|
| `Esc` / `q` | Chiudono (con una selezione di testo, `Esc` prima la azzera) |
| `j` / `k`, `Su` / `Giù` | Spostano la selezione |
| `PageUp` / `PageDown` | Scorrono di una pagina |
| `Home` / `End`, `Shift+G` | Prima / ultima riga |
| `h` / `l`, `Sinistra` / `Destra` | Scorrono il testo di lato |
| `Enter` | (Blame, cronologia del file) Va al commit della riga nel Log |
| `e` | Modifica il file nell'editor integrato |
| `y` / `Ctrl+C` | Copiano il testo selezionato. Senza selezione: la riga selezionata (Blame: solo il codice; Cronologia del file: il riepilogo) |

Blame e la cronologia del file hanno tasti propri, elencati nella tabella generata qui sopra (contesti `file_insight` e `history`) e come `Blame: ...` / `Cronologia del file: ...` nella palette; tutti si possono riassegnare. Dettagli in [Blame](log-and-history.md#blame) e [Cronologia del file](log-and-history.md#cronologia-del-file). Nell'elenco della cronologia le lettere singole (`d` `o` `b` `c` `y`) agiscono sulla riga selezionata, ma un accordo Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) mantiene sempre il suo significato globale.

## Colonna dei commit

La colonna a sinistra della vista di un file (vedi [La colonna dei commit](interface-tour.md#la-colonna-dei-commit)). `Alt+C` la mostra o la nasconde da qualsiasi punto; gli altri tasti funzionano quando la colonna ha la tastiera (`Ctrl+N` e poi il suo badge, o `Vista: Porta il focus sulla colonna dei commit`).

| Tasto | Azione |
|---|---|
| `Su` / `Giù`, `PageUp` / `PageDown`, `Home` / `End` | Scelgono un commit |
| `Enter` / `Destra` | Passano all'elenco dei file del commit scelto |
| `Esc` | Restituisce la tastiera alla vista |
| `.`, `Menu`, `Shift+F10` | Il menu della riga scelta |

Gli altri suoi tasti (`Shift+Left` / `Shift+Right` per scorrere in orizzontale, `Tab`, `]`, `[` e `0` per dimensionare le parti) sono nella tabella generata qui sopra, contesto `commit_column`.

## Editor

**Modifica**, o `e` in un diff o in una vista File, apre l'editor integrato sulla
copia del file nella directory di lavoro. Vedi
[Stage e commit](staging-and-committing.md).

| Tasto | Azione |
|---|---|
| *(scrivi)* | Inserisce al cursore (sostituisce la selezione) |
| `Enter` | A capo, con il rientro della riga corrente |
| `Tab` / `Shift+Tab` | Aumentano / riducono il rientro (delle righe selezionate, se più di una) |
| frecce, `Home` / `End` | Si spostano. `Home` va al primo carattere non vuoto, poi alla colonna 0. `Shift` estende la selezione |
| `Ctrl+Left` / `Ctrl+Right` | Si spostano di una parola |
| `Ctrl+Home` / `Ctrl+End` | Inizio / fine del file |
| `PageUp` / `PageDown` | Una pagina in su / in giù |
| `Ctrl+A` | Seleziona tutto |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copia / taglia / incolla |
| `Ctrl+U` / `Ctrl+K` | Cancellano fino all'inizio / alla fine della riga |
| `Ctrl+Z` | Annulla |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Ripeti |
| `Ctrl+S` | Salva |
| `Ctrl+F` / `F3` | Cerca. Nel campo di ricerca: `Enter` / `Giù` / `F3` successivo, `Shift+Enter` / `Su` precedente, `Esc` chiude |
| `Alt+M` | Attiva o disattiva la minimappa (**Map**) |
| `Esc` | Chiude. Con modifiche non salvate chiede: **Salva** (`S` / `Enter`), **Scarta** (`D`), **Annulla** (`C` / `Esc`) |

`Tab` indenta nell'editor, quindi non sposta il focus. Le altre combinazioni
`Ctrl` (`Ctrl+P`, `Ctrl+N`) funzionano comunque.

La **codifica**, le **interruzioni di riga** e il **rientro** della barra di stato sono
pulsanti: ognuno apre un menu a discesa (frecce o rotellina, scrivi per filtrare, `Enter`
per scegliere, `Esc` per chiudere). La palette (`Ctrl+P`) apre gli stessi menu: *Editor: Cambia codifica*, *Editor: Riapri con codifica*, *Editor: Converti in codifica*, *Editor: Converti interruzioni di riga*, *Editor: Cambia rientro*.
Dettagli in [Stage e commit](staging-and-committing.md).

## Azioni Git (`Ctrl+G`)

`Ctrl+G` attende un altro tasto:

| Tasto | Azione |
|---|---|
| `s` / `u` | Stage / unstage del file selezionato |
| `a` | Stage di tutte le modifiche |
| `x` | Scarta tutte le modifiche (chiede prima) |
| `c` | Apre il compositore di commit |
| `Enter` | Esegue subito il commit delle modifiche in stage, con il messaggio del modulo, da qualsiasi punto (vedi [Stage e commit](staging-and-committing.md#scrivere-il-commit)) |
| `m` | Stash di tutte le modifiche con un messaggio (vuoto è ammesso). Con l'IA attiva, `Ctrl+G` in questa richiesta scrive il messaggio |
| `Shift+T` | Stash di tutte le modifiche (file non tracciati inclusi), senza messaggio |
| `o` | Pop di uno stash: la riga dello stash selezionata nel Log, altrimenti l'ultimo stash |
| `p` | Pull, nel modo in cui è impostato il pulsante Pull (`git.pull_mode`) |
| `Shift+P` | Push al remoto predefinito (imposta l'upstream al primo push), oppure chiede il remoto |
| `Shift+U` | Push di tutti i tag |
| `f` | Fetch di ogni remoto, con prune |
| `g` | Apre nel browser la pull request del branch in checkout |
| `b` / `t` | Nuovo branch / nuovo tag su HEAD (vedi [Dare un nome a un branch o a un tag](#dare-un-nome-a-un-branch-o-a-un-tag)) |
| `d` | Elimina il branch o il tag selezionato (chiede prima) |
| `e` | Modifica il messaggio del commit selezionato |
| `Shift+E` / `Shift+A` | Modifica il commit selezionato (tutti i campi) / il suo autore (vedi [Modificare un commit](#modificare-un-commit)) |
| `y` | Cherry-pick del commit selezionato |
| `v` | Revert del commit selezionato |
| `Shift+R` | Reset (mixed) al commit selezionato |
| `r` | Rebase sul branch selezionato |
| `Shift+W` | Apre il menu Flow (in un repository con un workflow di branching) |
| `Shift+H` | Apre il pannello di riscrittura della cronologia (solo nelle build con la riscrittura della cronologia) |
| `l` | Apre il menu LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch di tutto / checkout di tutto / pull / push di tutto / prune. Vedi [Git LFS](lfs.md) |
| `i` | ✦ IA: scrive il messaggio di commit per le modifiche in stage |
| `w` | ✦ IA: spiega il commit selezionato, o le modifiche in corso |
| `Shift+C` | ✦ IA: propone come dividere le modifiche in corso in commit |

I tasti IA funzionano solo quando l'IA è attiva (vedi [Assistenza IA](ai.md)).
`Esc` annulla una richiesta IA in corso.

## Pannello destro (modifiche in corso e dettagli del commit)

Con la riga WIP selezionata, il pannello destro mostra gli elenchi **File non
in stage** e **File in stage** e il compositore di commit. Con un commit
selezionato mostra i dettagli del commit e i suoi file.

| Tasto | Azione |
|---|---|
| `j` / `k`, `Su` / `Giù` | Si spostano. Alla fine di un elenco, la selezione prosegue nell'altro |
| `h` / `l`, `Sinistra` / `Destra` | Vanno all'elenco File non in stage / File in stage |
| `Enter` | Comprime o espande l'elenco attivo |
| `Space` / `Insert` | Aggiunge il file corrente alla selezione, o lo toglie |
| `Ctrl+A` | Selezionare tutti i file dell'elenco |
| `Ctrl+Space` | Stage / unstage del file selezionato, o comprime una riga di cartella |
| `d` | Apre il diff del file selezionato |
| `m` | `Percorso` (elenco piatto) o `Albero` (cartelle) |
| `a` | `Vedi tutti i file`: l'intero albero, non solo i file modificati |
| `z` | Comprime tutte le cartelle, o le espande di nuovo |
| `/` | Filtra per nome i file modificati di un commit |
| `.`, `Menu`, `Shift+F10` | Il menu del file |
| `c` | Dà il focus al campo del riepilogo del compositore di commit |
| `Shift+A` | Attiva o disattiva **Modifica il commit precedente** |
| `Ctrl+N ]` / `Ctrl+N [` | Ingrandiscono / riducono il blocco del messaggio di commit |

Nel compositore di commit:

| Tasto | Azione |
|---|---|
| `Enter` | Riepilogo: passa alla descrizione. Descrizione: a capo |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` lo fa da qualsiasi punto) |
| `Alt+R` | Inserisce un riferimento a un'issue |
| `Esc` | Lascia il compositore |

Il menu del file contiene: Stage o Unstage, Scarta le modifiche, Ignora, Stash
del file, Cronologia del file, Blame del file, Apri nello strumento diff esterno,
Apri nell'editor esterno, Apri il file nel programma predefinito, Mostra nella
cartella, Copia il percorso del file, Crea una patch dalle modifiche del file,
Modifica il file ed Elimina il file. Su un file in conflitto inizia con **Apri
nell'editor di merge** e le tre righe **Risolvi con**. Con Git LFS attivo ha le
righe LFS. **Scarta le modifiche** ed **Elimina il file** chiedono prima.

## Pulsanti della finestra

Ogni pulsante ha un gemello da tastiera:

| Pulsante | Tasto |
|---|---|
| 🐙 a sinistra della barra del titolo (Informazioni) | `Aiuto: Informazioni su GitPoulpi` nella palette |
| **Apri in** nella barra del titolo | `Alt+O`, `Ctrl+Alt+O` |
| `×` a destra della barra del titolo (esci) | `q` |
| La versione nella barra del titolo (Note di rilascio) | `Aiuto: Note di rilascio` nella palette |
| `⌕ Ricerca globale` nella barra del titolo, a sinistra del `⚙` | `Ctrl+Alt+F` |
| `⚙` nella barra del titolo (Impostazioni) | `Ctrl+,` |
| Il chip dell'area di lavoro nella barra delle schede | `Alt+W` |
| `+` dopo l'ultima scheda | `Alt+T` |
| La scheda **Radar** | `Alt+R` |
| La scheda **Aree di lavoro** | `Alt+J` |
| **Attività** nella barra di stato | `Ctrl+N Shift+A` |
| Barra degli strumenti: **Annulla**, **Ripeti** | `Ctrl+Z`, `Ctrl+Y` |
| Barra degli strumenti: **Pull** / il suo `▾` | `Ctrl+G p` / `Git: Opzioni di pull...` |
| Barra degli strumenti: **Push** / il suo `▾` | `Ctrl+G Shift+P` / `Git: Push verso un remote...` |
| Barra degli strumenti: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Barra degli strumenti: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Barra degli strumenti: **LFS** (quando LFS è attivo) | `Ctrl+G l` |
| Barra degli strumenti: **Flow** (quando il repository ha un workflow) | `Ctrl+G Shift+W` |
| Barra degli strumenti: **Cerca** | `f` |
| Barra degli strumenti: la pull request collegata (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Barra degli strumenti: l'elenco dei profili (con due o più profili) | `Repository: Cambia profilo...` |

La barra degli strumenti è descritta nel
[Tour dell'interfaccia](interface-tour.md#barra-degli-strumenti).

## Impostazioni

`Ctrl+,`, `Ctrl+N ,`, l'ingranaggio `⚙`, o `App: Impostazioni (Preferenze)...`
aprono la schermata Impostazioni.

| Tasto | Azione |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Sinistra` / `Destra` | Si spostano tra l'elenco delle pagine e le opzioni |
| `j` / `k`, `Su` / `Giù` | Spostano la selezione |
| `Enter` / `Space` | Aprono una pagina, attivano una casella, modificano un campo, aprono un elenco a discesa o premono un pulsante |
| `/`, `Ctrl+F` | Cercano in tutte le opzioni |
| `Alt+Left` / `Alt+Right` | Cambiano sotto-scheda (Generali / l'elenco) |
| `Esc` / `q` | Chiudono le Impostazioni. Mentre un campo è in modifica, `Esc` annulla solo la modifica |
| `?` | Apre l'aiuto sopra la schermata Impostazioni |

Vedi [Impostazioni](settings.md).

## Campi di testo

Tutti i campi di testo condividono gli stessi tasti: filtri, campi di ricerca, la
palette dei comandi, le richieste, il compositore di commit, il campo della riga
WIP, i campi delle finestre e i campi delle Impostazioni. Mentre un campo ha la
tastiera, questi tasti hanno la precedenza sulle scorciatoie dei pannelli.

| Tasto | Azione |
|---|---|
| un carattere | Lo digita al cursore (sostituisce la selezione) |
| `Sinistra` / `Destra` | Si spostano di un carattere (con una selezione: vanno al suo inizio / alla sua fine) |
| `Ctrl+Left` / `Ctrl+Right` | Si spostano di una parola |
| `Home` / `End` | Inizio / fine della riga |
| `Shift` + uno spostamento qui sopra | Estende o riduce la selezione |
| `Su` / `Giù` (solo descrizione) | Riga precedente / successiva |
| `Ctrl+Home` / `Ctrl+End` (solo descrizione) | Inizio / fine del testo |
| `Ctrl+A` | Seleziona tutto |
| `Backspace` / `Delete` | Cancellano il carattere prima / dopo il cursore, o la selezione |
| `Ctrl+U` | Cancella dal cursore all'inizio della riga (o la selezione) |
| `Ctrl+K` | Cancella dal cursore alla fine della riga (o la selezione) |
| `Ctrl+C` / `Ctrl+X` | Copiano / tagliano la selezione (non in un campo password) |
| incolla (l'incolla del terminale stesso) | Inserisce al cursore, sostituendo la selezione |

Nella descrizione del commit, `Enter` inserisce un a capo e `Ctrl+Enter` fa il
commit. Non tutti i terminali inviano `Ctrl+Enter` allo stesso modo, quindi
anche `Alt+Enter` e `Ctrl+J` fanno il commit.

Alcune console inviano `Ctrl+lettera` come carattere di controllo (`Ctrl+A` come
`0x01`). GitPoulpi capisce entrambe le forme. I caratteri `AltGr` (`@`, `#`, `€`
sulle tastiere svizzere o francesi) vengono digitati.

## Tooltip

Un tooltip compare quando il puntatore resta circa 0,6 s su un bersaglio (una
cella del Log, un pulsante della barra degli strumenti, la pull request
collegata). La pressione di un tasto, un clic, uno scorrimento, un
ridimensionamento, un popup o un menu aperto, o il puntatore che lascia il
bersaglio lo nascondono. Dopo di che ricompare solo quando il puntatore si muove
di nuovo.
