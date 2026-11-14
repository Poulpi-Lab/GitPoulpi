[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · **Nederlands** · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Sneltoetsen

Deze pagina toont de **standaardtoetsen**. De meeste kun je opnieuw toewijzen.
De app toont altijd de toetsen die gelden: in het opdrachtenpalet, de
`?`-help, de statusbalken, de contextmenu's en de tooltips van knoppen. Als je een
toets verplaatst, tonen deze plekken de nieuwe toets. Voor de muis, zie
[Muis en gebaren](mouse-and-gestures.md).

## Opnieuw toewijzen

Open **Instellingen > Toetsenbord & muis**. De pagina wordt beschreven in
[Instellingen](settings.md#toetsenbord--muis). Wijzigingen gelden meteen en worden
opgeslagen in de tabel `[keybindings]` van `config.toml`. Je kunt die tabel ook
met de hand bewerken:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

De syntaxis, de conflictregels en de waarschuwingen staan in de
[Configuratienaslag](config-reference.md) onder `[keybindings]`. Kort samengevat:

- Een fout wordt op de statusbalk gemeld en de standaardtoets blijft.
- Twee acties op één toetscombinatie in dezelfde context worden geweigerd.
- Een prefix heeft `Ctrl` of `Alt` nodig.
- Een toetscombinatie van een paneel die een globale toetscombinatie verbergt, geeft alleen een
  waarschuwing.

De tabel hieronder toont elke toets die je opnieuw kunt toewijzen. Sommige
toetsen zijn vast en kunnen niet worden toegewezen: lijstbewegingen (`j` `k`
`h` `l`, de pijlen, `PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`,
`Tab`, de cijfers (de tel-prefix), de tekstbewerkingstoetsen en de toetsen
`Enter` / `Tab` / pijlen van popups. Het palet heeft `App: Sneltoetsen...` en
`Sneltoetsen: Alles resetten...`.

## Standaard sneltoetsen

Elke toets die je opnieuw kunt toewijzen, met de id voor
`[keybindings.<context>]` en de standaardwaarde. De tabel wordt door de app
gegenereerd (een test houdt hem actueel). Toetsen na een prefix worden met de
prefix getoond (`Ctrl+G s`).

<!-- keymap:begin -->
| Context | Actie-id | Wat het doet | Standaardtoetsen |
|---|---|---|---|
| `global` | `git_prefix` | Prefix voor Git-acties | `Ctrl+G` |
| `global` | `nav_prefix` | Navigatieprefix | `Ctrl+N` |
| `global` | `palette` | Opdrachtenpalet | `Ctrl+P` |
| `global` | `settings` | Instellingen | `Ctrl+,` |
| `global` | `home` | Start (repositorybeheer) | `Ctrl+O` |
| `global` | `clone` | Repository klonen | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nieuw tabblad | `Alt+T` |
| `global` | `workspace_switch` | Van werkruimte wisselen | `Alt+W` |
| `global` | `radar` | Radar openen | `Alt+R` |
| `global` | `workspaces` | Werkruimtes openen | `Alt+J` |
| `global` | `tab_menu` | Tabbladmenu | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Openen in (lijst) | `Alt+O` |
| `global` | `tab_open_in_default` | Openen in standaard | `Ctrl+Alt+O` |
| `global` | `close_tab` | Tabblad sluiten | `Ctrl+W` |
| `global` | `next_tab` | Volgend tabblad | `Ctrl+PageDown` |
| `global` | `prev_tab` | Vorig tabblad | `Ctrl+PageUp` |
| `global` | `help` | Help | `?` |
| `global` | `quit` | Afsluiten | `q` |
| `global` | `refresh` | Vernieuwen | `r` |
| `global` | `refresh_all` | Vernieuwen (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Commit-grafiek doorzoeken | `f` |
| `global` | `workspace_search` | Werkruimte doorzoeken | `Ctrl+Alt+F` |
| `global` | `select_tab` | Tabblad kiezen | `t` |
| `global` | `undo` | Laatste bewerking ongedaan maken | `Ctrl+Z` |
| `global` | `redo` | Ongedane bewerking opnieuw doen | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | De geschiedenis van ongedaan maken openen | `Alt+Z` |
| `global` | `commit_column` | De commit-kolom tonen of verbergen | `Alt+C` |
| `global` | `undo_offer` | De branch herstellen naar waar de geblokkeerde actie hem achterliet | `Alt+Shift+Z` |
| `global` | `context_menu` | Contextmenu | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Commit-bericht opstellen | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Gestagede wijzigingen nu committen | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: wijzigingen in commits splitsen | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: wijzigingen uitleggen | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash met bericht | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Commit-bericht schrijven | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Alle wijzigingen stagen | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Alle wijzigingen verwerpen | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Geselecteerd bestand stagen | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Geselecteerd bestand unstagen | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Gekoppelde pull request openen | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Alle tags pushen | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Alle remotes fetchen | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nieuwe tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nieuwe branch | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Wijzigingen stashen | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Geselecteerde commit cherry-picken | `Ctrl+G y` |
| `git_prefix` | `revert` | Geselecteerde commit reverten | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Commit-bericht bewerken | `Ctrl+G e` |
| `git_prefix` | `amend` | Geselecteerde commit wijzigen | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Auteur van geselecteerde commit wijzigen | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (mixed) naar geselecteerde commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase op geselecteerde branch | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Stash poppen | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Geselecteerde ref verwijderen of stash droppen | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | LFS-menu openen | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Het Flow-menu openen | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch (alle objecten) | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Alle bestanden uitchecken | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull (objecten) | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push (alle objecten) | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Lokale objecten opruimen | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | De werkbank voor geschiedenis herschrijven openen | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Refs-paneel in-/uitklappen | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Statuspaneel in-/uitklappen | `Ctrl+N l` |
| `nav_prefix` | `settings` | Instellingen openen | `Ctrl+N ,` |
| `nav_prefix` | `about` | Over GitPoulpi openen | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Activiteitenlogboek openen | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Globaal activiteitenlogboek openen | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Achtergrondtaken tonen | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Het updatepaneel tonen | `Ctrl+N u` |
| `nav_prefix` | `help` | Help omschakelen | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Tabblad kiezen | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Thema wijzigen | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Taal wijzigen | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Kleurthema van de grafiek wijzigen | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Rijkleuring van de grafiek omschakelen | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Refs-sectie vergroten | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Refs-sectie verkleinen | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Commit-berichtblok vergroten | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Commit-berichtblok verkleinen | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Refs-paneel verbreden | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Refs-paneel versmallen | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Statuspaneel verbreden | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Statuspaneel versmallen | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: openstaande weergave tonen | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: terug (of openstaande weergave sluiten) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Het open dialoog schikken (verplaatsen, schalen) | `Ctrl+N m` |
| `refs` | `filter` | Refs filteren op naam | `/` |
| `refs` | `solo` | Selectie solo | `o` |
| `refs` | `clear_solo` | Solo stoppen | `Shift+O` |
| `refs` | `toggle_hierarchy` | Branchhiërarchie omschakelen | `y` |
| `refs` | `show_all` | Alle verborgen refs tonen | `Shift+H` |
| `refs` | `reveal` | In grafiek tonen | `g` |
| `refs` | `mark_drop` | Markeren als dropbron | `m` |
| `refs` | `drop_marked` | Gemarkeerde ref hier droppen | `Shift+M` |
| `refs` | `expand_all` | Alle mappen uitklappen | `=` |
| `refs` | `collapse_all` | Alle mappen inklappen | `-` |
| `refs` | `expand_below` | De map en zijn submappen uitklappen | `*`, `Shift+Right` |
| `refs` | `collapse_below` | De map en zijn submappen inklappen | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Bestanden: pad of boom | `m` |
| `log` | `view_all_files` | Bestanden: alle bestanden tonen | `a` |
| `log` | `compare_selection` | De geselecteerde rijen vergelijken | `v` |
| `log` | `compare_swap` | Uiteinden van de vergelijking omwisselen | `x` |
| `log` | `compare_next` | Naar de volgende vergeleken rij | `n` |
| `log` | `pick_column` | Kolom kiezen om te verplaatsen of de breedte te wijzigen | `Shift+C` |
| `log` | `scroll_left` | Kolommen naar links scrollen | `Shift+Left` |
| `log` | `scroll_right` | Kolommen naar rechts scrollen | `Shift+Right` |
| `log` | `copy_mermaid` | Commits kopiëren als Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Volgende kolom kiezen | `Tab` |
| `log_columns` | `prev` | Vorige kolom kiezen | `Shift+BackTab` |
| `log_columns` | `move_left` | Kolom naar links verplaatsen | `<` |
| `log_columns` | `move_right` | Kolom naar rechts verplaatsen | `>` |
| `log_columns` | `wider` | Kolom verbreden | `]` |
| `log_columns` | `narrower` | Kolom versmallen | `[` |
| `log_columns` | `reset_width` | Kolombreedtes herstellen | `0` |
| `log_columns` | `fit` | Kolom aan inhoud aanpassen | `=` |
| `log_columns` | `fit_all` | Alle kolommen aan inhoud aanpassen | `+` |
| `log_columns` | `layout` | Zijwaarts scrollen of aan paneel aanpassen | `s` |
| `log_files` | `collapse_all` | Alle mappen in-/uitklappen | `z` |
| `log_files` | `filter` | Gewijzigde bestanden filteren | `/` |
| `log_files` | `expand_all` | Alle mappen uitklappen | `=` |
| `log_files` | `collapse_all_folders` | Alle mappen inklappen | `-` |
| `log_files` | `expand_below` | De map en zijn submappen uitklappen | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | De map en zijn submappen inklappen | `_`, `Shift+Left` |
| `status` | `compose` | Focus op het commit-bericht | `c` |
| `status` | `toggle_file_mode` | Bestanden: pad of boom | `m` |
| `status` | `view_all_files` | Bestanden: alle bestanden tonen | `a` |
| `status` | `amend` | Amend omschakelen | `Shift+A` |
| `status` | `skip_hooks` | Hooks overslaan omschakelen | `Shift+H` |
| `status` | `collapse_all` | Alle mappen in-/uitklappen | `z` |
| `status` | `stage_toggle` | Geselecteerde rij stagen of unstagen | `Ctrl+Space` |
| `status` | `open_diff` | Diff van geselecteerd bestand openen | `d` |
| `status` | `expand_all` | Alle mappen uitklappen | `=` |
| `status` | `collapse_all_folders` | Alle mappen inklappen | `-` |
| `status` | `expand_below` | De map en zijn submappen uitklappen | `*`, `Shift+Right` |
| `status` | `collapse_below` | De map en zijn submappen inklappen | `_`, `Shift+Left` |
| `home` | `favorite` | Repository als favoriet (de)markeren | `f`, `*` |
| `home` | `close_repo` | Rij sluiten, verwijderen of favoriet wissen | `x`, `Delete` |
| `home` | `browse` | Naar een repository bladeren | `b` |
| `home` | `clone` | Een repository klonen | `c` |
| `home` | `init` | Een repository initialiseren | `i` |
| `home` | `collapse_all` | Alle secties inklappen | `-` |
| `home` | `expand_all` | Alle secties uitklappen | `+`, `=` |
| `home` | `search` | Repositories zoeken | `/` |
| `home` | `workspace_new` | Nieuwe werkruimte | `n` |
| `home` | `workspace_edit` | Werkruimte bewerken | `e` |
| `home` | `workspace_from_tabs` | Nieuwe werkruimte van open tabbladen | `a` |
| `home` | `workspace_fetch` | Alles fetchen in de werkruimte | `Shift+F` |
| `home` | `workspace_pull` | Alles pullen in de werkruimte | `Shift+P` |
| `home` | `workspace_clone_missing` | Ontbrekende repositories klonen | `m` |
| `home` | `workspace_locate` | Ontbrekende repository zoeken | `l` |
| `refs` | `hosting_filter` | Pull requests / issues: filter kiezen | `v` |
| `refs` | `hosting_edit` | Pull request: bewerken op de host | `e` |
| `refs` | `new_pull_request` | Pull requests: er een maken | `n` |
| `refs` | `add_remote` | Remotes: een remote toevoegen | `+` |
| `refs` | `remotes` | Branch: Edit its Remotes | `u` |
| `diff` | `close` | Diff sluiten | `q` |
| `diff` | `edit` | Bewerken in werkmap | `e` |
| `diff` | `lfs_raw` | LFS: kaart of ruwe pointer | `r` |
| `diff` | `copy` | Selectie of regel kopiëren | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Hunk stagen of unstagen | `Space` |
| `diff` | `stage_lines` | Geselecteerde regels stagen of unstagen | `Enter` |
| `diff` | `discard_hunk` | Hunk verwerpen | `x` |
| `diff` | `discard_lines` | Geselecteerde regels verwerpen | `Shift+X` |
| `diff` | `select_lines` | Regelselectie starten of laten vallen | `v` |
| `diff` | `next_hunk` | Volgende hunk | `n` |
| `diff` | `prev_hunk` | Vorige hunk | `p` |
| `diff` | `toggle_layout` | Lay-out: unified of naast elkaar | `m` |
| `diff` | `toggle_hunks_only` | Alleen hunks of heel bestand | `Shift+H` |
| `diff` | `toggle_minimap` | Minimap omschakelen | `Shift+M` |
| `diff` | `choose_encoding` | Codering kiezen | `Shift+C` |
| `diff` | `toggle_wrap` | Regelafbreking omschakelen | `w` |
| `diff` | `toggle_whitespace` | Alleen-witruimtewijzigingen tonen of verbergen | `s` |
| `diff` | `merge_editor` | Merge-editor openen (bestand met conflict) | `Shift+E` |
| `file_insight` | `close` | Bestandsweergave sluiten | `q` |
| `file_insight` | `edit` | Bewerken in werkmap | `e` |
| `file_insight` | `copy` | Selectie of rij kopiëren | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Naar de laatste rij springen | `Shift+G` |
| `file_insight` | `open_commit` | Naar de commit van de rij gaan | `Enter` |
| `history` | `show_in_log` | Tonen in de Log | `Enter` |
| `history` | `copy_sha` | SHA van de commit kopiëren | `y` |
| `history` | `copy_summary` | Samenvatting kopiëren | `Shift+Y` |
| `history` | `show_diff` | Diff van deze commit tonen | `d`, `Alt+1` |
| `history` | `open_file` | Bestand openen bij deze commit | `o`, `Alt+2` |
| `history` | `blame` | Blame bij deze commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Vergelijken met de werkmap | `c` |
| `history` | `restore` | Deze versie herstellen | `Shift+R` |
| `history` | `range_up` | Selectie omhoog uitbreiden | `Shift+Up` |
| `history` | `range_down` | Selectie omlaag uitbreiden | `Shift+Down` |
| `history` | `switch_focus` | Wisselen tussen lijst en diff | `Tab` |
| `history` | `widen_list` | Lijst verbreden | `Alt+Right` |
| `history` | `narrow_list` | Lijst versmallen | `Alt+Left` |
| `history` | `reset_split` | Lijstbreedte herstellen | `Alt+0` |
| `commit_column` | `scroll_left` | Kolom naar links scrollen | `Shift+Left` |
| `commit_column` | `scroll_right` | Kolom naar rechts scrollen | `Shift+Right` |
| `commit_column` | `next_part` | Volgend deel kiezen | `Tab` |
| `commit_column` | `move_left` | Deel naar links verplaatsen | `<` |
| `commit_column` | `move_right` | Deel naar rechts verplaatsen | `>` |
| `commit_column` | `wider_part` | Deel verbreden | `]` |
| `commit_column` | `narrower_part` | Deel versmallen | `[` |
| `commit_column` | `reset_widths` | Breedtes van de delen herstellen | `0` |
| `commit_column` | `fit` | Onderdeel aan inhoud aanpassen | `=` |
| `commit_column` | `fit_all` | Alle onderdelen aan inhoud aanpassen | `+` |
| `file_insight` | `blame_card` | Blame: Detailkaart tonen of verbergen | `i` |
| `file_insight` | `blame_next` | Blame: Volgende hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Vorige hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Volgende hunk van dezelfde commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Vorige hunk van dezelfde commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Naar regel gaan | `g` |
| `file_insight` | `blame_copy_sha` | Blame: SHA van commit kopiëren | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Diff van de commit voor dit bestand | `d` |
| `file_insight` | `blame_parent` | Blame: Blame op de ouder van de commit | `b` |
| `file_insight` | `blame_history` | Blame: Bestandsgeschiedenis openen | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Annotatiekolom verbreden | `>` |
| `file_insight` | `blame_narrower` | Blame: Annotatiekolom versmallen | `<` |
| `image` | `zoom_in` | Afbeelding inzoomen | `+`, `=` |
| `image` | `zoom_out` | Afbeelding uitzoomen | `-` |
| `image` | `fit` | Afbeelding aanpassen aan de weergave | `0` |
| `image` | `actual_size` | Afbeelding op ware grootte tonen | `1` |
| `image` | `flip_side` | De andere kant tonen (voor / na) | `Tab` |
| `image` | `widen_before` | Voor-paneel verbreden | `Alt+Right` |
| `image` | `widen_after` | Na-paneel verbreden | `Alt+Left` |
| `image` | `reset_split` | Voor/na-verdeling herstellen | `Alt+0` |
| `image` | `edit_in` | Openen met... | `Shift+O` |
| `image` | `edit_default` | Openen in de standaardapp | `o` |
| `editor` | `save` | Opslaan | `Ctrl+S` |
| `editor` | `undo` | Ongedaan maken | `Ctrl+Z` |
| `editor` | `redo` | Opnieuw uitvoeren | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Zoeken | `Ctrl+F` |
| `editor` | `find_next` | Volgende zoeken (Shift: vorige) | `F3` |
| `editor` | `paste` | Plakken | `Ctrl+V` |
| `editor` | `toggle_minimap` | Minimap omschakelen | `Alt+M` |
| `editor_prompt` | `save` | Niet-opgeslagen wijzigingen: opslaan en sluiten | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Niet-opgeslagen wijzigingen: verwerpen en sluiten | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Niet-opgeslagen wijzigingen: verder bewerken | `c`, `Shift+C` |
| `merge` | `next_conflict` | Volgend conflict | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Vorig conflict | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Volgend bestand met conflict | `Alt+PageDown` |
| `merge` | `prev_file` | Vorig bestand met conflict | `Alt+PageUp` |
| `merge` | `next_pane` | Focus op volgend paneel | `F6` |
| `merge` | `prev_pane` | Focus op vorig paneel | `Shift+F6` |
| `merge` | `toggle_panes` | Zijpanelen tonen of verbergen (Refs en Status) | `F9` |
| `merge` | `toggle_base` | Basispaneel tonen of verbergen | `Alt+D` |
| `merge` | `toggle_full_file` | Diff van heel bestand omschakelen | `Alt+F` |
| `merge` | `undo` | Ongedaan maken (keuzes en typen) | `Ctrl+Z` |
| `merge` | `redo` | Opnieuw uitvoeren (keuzes en typen) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Huidig paneel (A) verbreden | `Alt+Right` |
| `merge` | `widen_incoming` | Inkomend paneel (B) verbreden | `Alt+Left` |
| `merge` | `reset_panes` | Paneelgroottes herstellen | `Alt+0` |
| `merge` | `save` | Opslaan zonder als opgelost te markeren | `Alt+S` |
| `merge` | `save_resolve` | Opslaan en als opgelost markeren | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Oplossing voorstellen (AI) | `Alt+I` |
| `merge` | `unpick` | Keuze voor dit conflict ongedaan maken | `Alt+U` |
| `merge` | `reset_all` | Heel bestand terugzetten naar de oorspronkelijke conflicten | `Alt+R` |
| `merge` | `focus_a` | Focus op paneel Huidig (A) | `Alt+1` |
| `merge` | `focus_b` | Focus op paneel Inkomend (B) | `Alt+2` |
| `merge` | `focus_output` | Focus op paneel Uitvoer | `Alt+3` |
| `merge` | `focus_base` | Focus op paneel Basis | `Alt+4` |
| `merge_sides` | `toggle_line` | Regel kiezen of laten vallen | `Space` |
| `merge_sides` | `toggle_chunk` | Heel blok kiezen of laten vallen | `Enter` |
| `merge_sides` | `take_a` | Alleen Huidig (A) overnemen | `a` |
| `merge_sides` | `take_b` | Alleen Inkomend (B) overnemen | `b` |
| `merge_sides` | `take_both` | Beide overnemen, A dan B | `c` |
| `merge_sides` | `take_both_reversed` | Beide overnemen, B dan A | `Shift+C` |
| `merge_sides` | `clear` | Niets nemen voor dit conflict | `x` |
| `merge_sides` | `unpick` | Keuze voor dit conflict ongedaan maken | `u` |
| `merge_sides` | `reset_all` | Heel bestand terugzetten naar de oorspronkelijke conflicten | `Shift+U` |
| `merge_sides` | `swap_order` | Volgorde van de twee kanten omwisselen | `s` |
| `merge_sides` | `next_conflict` | Volgend conflict | `n` |
| `merge_sides` | `prev_conflict` | Vorig conflict | `Shift+N` |
| `merge_sides` | `take_all_a` | Alles van Huidig (A) overnemen | `Shift+A` |
| `merge_sides` | `take_all_b` | Alles van Inkomend (B) overnemen | `Shift+B` |
| `merge_sides` | `toggle_base` | Basispaneel tonen of verbergen | `d` |
| `merge_sides` | `scroll_left` | Panelen naar links scrollen | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Panelen naar rechts scrollen | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Diff van heel bestand omschakelen | `f` |
| `merge_sides` | `cycle_height` | Hoogte van de uitvoer doorlopen | `z` |
| `merge_card` | `take_a` | A (huidig) ongewijzigd behouden | `a` |
| `merge_card` | `take_b` | B (inkomend) ongewijzigd overnemen | `b` |
| `merge_card` | `take_base` | De basis (gemeenschappelijke voorouder) overnemen | `o` |
| `merge_card` | `keep` | Het overgebleven bestand behouden | `k` |
| `merge_card` | `delete` | De verwijdering accepteren | `Shift+D` |
| `merge_card` | `external_tool` | Openen in externe merge-tool | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (stoppen om te amenden) | `e` |
| `rebase` | `autosquash` | Autosquash omschakelen | `a` |
| `rebase` | `start` | Plan starten | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Plan resetten | `Ctrl+R` |
| `rebase` | `split` | De commit van de rij splitsen | `t` |
| `rebase` | `move_up` | Rij omhoog verplaatsen | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Rij omlaag verplaatsen | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | De commits maken | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Regelselectie omschakelen | `v` |
| `split` | `move_up` | De commit omhoog verplaatsen | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | De commit omlaag verplaatsen | `Alt+Down`, `Shift+J` |
| `split` | `reset` | De splitsing resetten | `Shift+R` |
| `rewrite` | `preview` | Voorbeeld van de herschrijving tonen | `F5` |
| `rewrite` | `apply` | De herschrijving toepassen | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Het voorbeeld annuleren | `Ctrl+X` |
| `rewrite` | `add_rule` | Een regel toevoegen | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Regel in- of uitschakelen | `Space` |
| `rewrite` | `move_up` | Regel omhoog verplaatsen | `Alt+Up` |
| `rewrite` | `move_down` | Regel omlaag verplaatsen | `Alt+Down` |
| `rewrite` | `delete_rule` | Regel verwijderen | `d`, `Delete` |
| `rewrite` | `restore_rule` | Verwijderde regel herstellen | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Regel dupliceren | `Ctrl+D` |
| `rewrite` | `next_tab` | Volgend voorbeeldtabblad | `Alt+Right` |
| `rewrite` | `prev_tab` | Vorig voorbeeldtabblad | `Alt+Left` |
| `rewrite` | `save_shared` | Recept in de repository opslaan | `Alt+S` |
| `rewrite` | `save_mine` | Recept alleen voor mij opslaan | `Alt+M` |
| `rewrite` | `load` | Een recept laden | `Alt+L` |
| `rewrite` | `name` | Recept hernoemen | `F2` |
| `rewrite` | `protected_all` | Alle beschermde branches meenemen | `Alt+A` |
| `rewrite` | `protected_none` | Alle beschermde branches overslaan | `Alt+U` |
| `rewrite` | `check_tools` | Programma's opnieuw controleren | `F6` |
| `rewrite` | `analyze` | De groottes in de geschiedenis analyseren | `F7` |
| `rewrite` | `host_howto` | De branchbescherming van de host uitleggen | `Alt+H` |
| `rewrite` | `ai_rules` | Herschrijfregels maken met AI | `Alt+I` |
| `reword` | `apply` | Het nieuwe bericht toepassen | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Geschiedenis dieper wijzigen | `Alt+H` |
| `composer` | `commit` | Commit vanuit de composer | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Een issue-verwijzing invoegen | `Alt+R` |
| `dialog` | `browse` | Naar een map bladeren | `Ctrl+B` |
| `dialog` | `sources` | Naar de kloonbronnen | `Alt+S` |
| `dialog` | `submit` | Het formulier voor pull request indienen | `Ctrl+S` |
| `dialog` | `web` | Webformulier van de provider openen | `Alt+W` |
| `folder_picker` | `choose` | De map kiezen | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nieuwe map | `Alt+N` |
| `folder_picker` | `filter` | De items van de map filteren | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Naastgelegen mappen tonen of verbergen | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Verborgen mappen tonen of verbergen | `Alt+H` |
| `preferences` | `search` | De instellingen doorzoeken | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Een lijstitem toevoegen | `Insert` |
| `preferences` | `list_duplicate` | Het geselecteerde lijstitem dupliceren | `Ctrl+D` |
| `preferences` | `list_remove` | Het geselecteerde lijstitem verwijderen | `Delete` |
| `preferences` | `list_move_up` | Het geselecteerde lijstitem omhoog verplaatsen | `Alt+Up` |
| `preferences` | `list_move_down` | Het geselecteerde lijstitem omlaag verplaatsen | `Alt+Down` |
| `preferences` | `prev_tab` | Vorig tabblad van de instellingenpagina | `Alt+Left` |
| `preferences` | `next_tab` | Volgend tabblad van de instellingenpagina | `Alt+Right` |
| `ai_popup` | `copy` | Het resultaat kopiëren | `c` |
| `activity_log` | `scroll_left` | Het activiteitenlogboek naar links scrollen | `Left` |
| `activity_log` | `scroll_right` | Het activiteitenlogboek naar rechts scrollen | `Right` |
| `integrations` | `add` | Een profiel toevoegen | `a` |
| `integrations` | `edit` | Het geselecteerde profiel bewerken | `e` |
| `integrations` | `open_token` | De tokenpagina openen | `o` |
| `integrations` | `remove` | Het geselecteerde profiel verwijderen | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Stash-bericht genereren | `Ctrl+G` |
<!-- keymap:end -->

### Toetsen die de terminal niet kan sturen, en het kitty-toetsenbordprotocol

Een oudere terminal kan sommige toetscombinaties niet sturen: `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` en `Ctrl+<cijfer>`. Hij stuurt niets, of een andere
toets: `Ctrl+Shift+N` komt bijvoorbeeld aan als `Ctrl+N`, `Ctrl+I` als `Tab` en
`Ctrl+M` als `Enter`. De pagina Instellingen waarschuwt voor zo'n koppeling ("deze
terminal stuurt ...").

Terminals met het **kitty-toetsenbordprotocol** kunnen deze toetscombinaties sturen.
Voorbeelden: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 en nieuwer, en zellij (tmux
antwoordt er niet op: zie [Multiplexers](multiplexers.md#tmux)). GitPoulpi gebruikt het protocol
standaard:

- Bij het opstarten vraagt GitPoulpi het aan de terminal en wacht hoogstens
  ongeveer 100 ms op het antwoord. Antwoordt de terminal niet, dan blijven de
  oude toetsen.
- Antwoordt de terminal, dan zet GitPoulpi het protocol aan. Bij afsluiten, een
  fout of een crash zet het het weer uit.
- De waarschuwingen op de pagina Instellingen volgen het protocol: als het aan
  staat, zijn de bovenstaande toetscombinaties te sturen.
- **Instellingen > Toetsenbord & muis > Kitty-toetsenbordprotocol**
  (`ui.kitty_keyboard`) op `off` wijzigt de toetsenbordmodus van de terminal
  nooit. Gebruik dit als een terminal of multiplexer zich misdraagt. Een
  herstart is nodig.

Windows-consoles melden de modifiers zelf en hebben het protocol niet nodig.

## Opdrachtenpalet (`Ctrl+P`)

Het palet toont **elke** opdracht van de app, dus alles is bereikbaar zonder
sneltoets. De volledige lijst staat [hieronder](#elke-opdracht-van-het-palet).

| Toets | Actie |
|---|---|
| `Ctrl+P` | Het palet openen |
| *(typen)* | De lijst filteren. Een exacte overeenkomst na de prefix `Categorie: ` komt eerst, dan een overeenkomst ergens in het label, dan een fuzzy overeenkomst |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | De markering verplaatsen (zie [Filterbare popuplijsten](#filterbare-popuplijsten)) |
| `Enter`, of een dubbelklik | De gemarkeerde opdracht uitvoeren |
| `Esc` | Sluiten |

`j` en `k` worden in het filter getypt. Ze verplaatsen de markering niet.

Elke rij toont rechts de sneltoets. Een opdracht die nu niet van toepassing is
(bijvoorbeeld geen ref geselecteerd) blijft in de lijst staan, grijs, en de
reden verschijnt onder de lijst. Labels beginnen met een categorie, zoals
`Navigatie:`, `Refs:`, `Git:`, `Weergave:`, `Diff:` of `App:`. Typ een categorie
om de hele groep te zien.

### Een toets voor elke opdracht

Elke rij van het palet eindigt met een toetsknop (`[k]`), en `Alt+K` doet hetzelfde op de gemarkeerde rij: het opent **Instellingen > Toetsenbord & muis** gefilterd op de sneltoets van die opdracht, klaar om opnieuw toe te wijzen. Een opdracht zonder eigen sneltoets krijgt er hier een: ze staat onder **Palet-opdrachten**, standaard zonder toets, en een toets die je eraan geeft voert de opdracht precies uit zoals het kiezen in het palet (een opdracht die nu niet kan, zegt waarom). De rij toont dan jouw toets.

In `config.toml` is dit de tabel `[keybindings.palette]`, genoemd naar de opdracht (`toggle_minimap = "Ctrl+Alt+M"`). Een toets die botst met een andere actie wordt rood getoond, en `c` op de rij toont alle acties waarmee hij botst.

De hintbalk onder de lijst noemt deze toets (standaard `Alt+K`). Het is de actie `palette.configure_shortcut`, dus hij is net als elke andere opnieuw toe te wijzen.

Het palet en de kiezers (thema, taal, tab, werkruimtezoeker) verplaatsen en schalen als een dialoog: sleep de bovenrand, druk op `Ctrl+N` en dan `m` (zoals boven elk dialoog), of druk op `Alt+G`. `Alt+G` is de actie `palette.arrange` ("Dialoog: het voorste dialoog verplaatsen of schalen"): een extra manier, geldig boven elk dialoog en elke popup, om de schikmodus te starten voor het dialoog vooraan (de paletrijen `Dialoog: Verplaatsen of schalen` en `Dialoog: Positie en grootte herstellen` werken op het palet zelf zolang het open is). Pijlen verplaatsen het, `Shift`+pijlen schalen het, `0` herstelt het, `Enter` of `Esc` verlaat de modus en de popup blijft open. `ui.dialog_geometry` bepaalt of de plaats wordt bewaard.

`Ctrl+N` is hier ook het navigatieprefix: het toont een badge op het filtervak, de eerste rij van de lijst en het sluitkruis, en de toets erna drukt dat bedieningselement in of `m` schikt de popup. Het verplaatst de markering niet meer omlaag, en `Ctrl+P` verplaatst haar niet omhoog: alleen de pijlen, `PageUp` / `PageDown` en `Home` / `End` verplaatsen een lijst met een filterveld.

### Elke opdracht van het palet

Gegenereerd door de app (een test houdt het actueel). AI-opdrachten verschijnen
alleen als AI aan staat; de andere zijn grijs, met de reden, als ze niet van
toepassing zijn. Toetsen zijn de standaardwaarden.

<!-- palette:begin -->
| Opdracht | Standaardtoets |
|---|---|
| Navigatie: Focus op Refs-paneel | `Ctrl+N <badge>` |
| Navigatie: Focus op Log-paneel | `Ctrl+N <badge>` |
| Navigatie: Focus op Statuspaneel | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs-filter | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Lokaal | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Remote | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Worktrees | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Submodules | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Stashes | `Ctrl+N <badge>` |
| Navigatie: Focus op Refs > Tags | `Ctrl+N <badge>` |
| Linkerpaneel: Secties tonen of verbergen... |  |
| Linkerpaneel: Secties terugzetten naar standaard |  |
| Linkerpaneel: Deze secties voor alle repositories gebruiken |  |
| Linkerpaneel: Lokale branches tonen of verbergen |  |
| Linkerpaneel: Remotes tonen of verbergen |  |
| Linkerpaneel: Worktrees tonen of verbergen |  |
| Linkerpaneel: Stashes tonen of verbergen |  |
| Linkerpaneel: Tags tonen of verbergen |  |
| Linkerpaneel: Pull requests tonen of verbergen |  |
| Linkerpaneel: Issues tonen of verbergen |  |
| Linkerpaneel: Submodules tonen of verbergen |  |
| Navigatie: Focus op Status > Unstaged | `Ctrl+N <badge>` |
| Navigatie: Focus op Status > Staged | `Ctrl+N <badge>` |
| Navigatie: Focus naar volgend paneel | `Tab` |
| Navigatie: Focus naar vorig paneel | `Shift+Tab` |
| Navigatie: Commits zoeken... | `f` |
| Navigatie: Refs-paneel in-/uitklappen | `Ctrl+N h` |
| Navigatie: Refs-paneel verbreden | `Ctrl+N Right` |
| Navigatie: Refs-paneel versmallen | `Ctrl+N Left` |
| Navigatie: Statuspaneel verbreden | `Ctrl+N Shift+Left` |
| Navigatie: Statuspaneel versmallen | `Ctrl+N Shift+Right` |
| Navigatie: Statuspaneel in-/uitklappen | `Ctrl+N l` |
| Navigatie: Refs-sectie vergroten | `Ctrl+N +` |
| Navigatie: Refs-sectie verkleinen | `Ctrl+N -` |
| Navigatie: Groottes Refs-secties resetten |  |
| Navigatie: Lay-out resetten |  |
| Commit-details: Bericht vergroten | `Ctrl+N ]` |
| Commit-details: Bericht verkleinen | `Ctrl+N [` |
| Commit-details: Berichtgrootte resetten |  |
| Commit-details: Bericht omlaag scrollen |  |
| Commit-details: Bericht omhoog scrollen |  |
| Commit-details: Link in bericht openen... |  |
| Bestanden: Sortering commit-bestanden omschakelen (A-Z / Z-A) |  |
| Refs: Geselecteerde ref uitchecken | `dubbelklik` |
| Refs: Refnaam kopiëren |  |
| Refs: Volledige ref kopiëren (refs/heads/...) |  |
| Refs: Doel-SHA van ref kopiëren |  |
| Refs: Geselecteerde branch mergen in huidige |  |
| Refs: Huidige branch rebasen op geselecteerde | `Ctrl+G r` |
| Refs: Geselecteerde ref droppen op uitgecheckte branch... |  |
| Refs: Markeren als dropbron | `m` |
| Refs: Gemarkeerde ref hier droppen... | `M` |
| Refs: Geselecteerde branch hernoemen... |  |
| Refs: Geselecteerde branch fast-forwarden naar huidige |  |
| Refs: Huidige branch fast-forwarden naar geselecteerde |  |
| Refs: Worktree maken van geselecteerde branch of tag... |  |
| Refs: Reflog van de geselecteerde branch... |  |
| Refs: Pull request starten vanaf geselecteerde branch... |  |
| Refs: Worktree openen in tabblad | `Enter / dubbelklik` |
| Refs: Submodule bijwerken (geselecteerd) |  |
| Refs: Submodule initialiseren (geselecteerd) |  |
| Refs: Submodule openen in tabblad | `Enter / dubbelklik` |
| Git: Alle submodules bijwerken |  |
| Git: Worktree maken op geselecteerde commit... |  |
| Git: Patch maken van geselecteerde commit... |  |
| Git: Patch maken van geselecteerde commit tot HEAD... |  |
| Git: Patch maken van werkmapwijzigingen... |  |
| Herschrijven: Oude geschiedenis opruimen... |  |
| Git: Patch toepassen... |  |
| Herschrijven: Werkbank voor geschiedenis | `Ctrl+G H` |
| Herschrijven: Een auteur vervangen... |  |
| Herschrijven: Auteurs anonimiseren... |  |
| Herschrijven: .mailmap op de geschiedenis toepassen... |  |
| Herschrijven: Identiteiten mergen (.mailmap)... |  |
| Herschrijven: Datums van commits verschuiven... |  |
| Herschrijven: Tijdzone van commits wijzigen... |  |
| Herschrijven: Commits binnen bepaalde uren houden... |  |
| Herschrijven: Committerdatum gelijkstellen aan auteursdatum |  |
| Herschrijven: Zoeken en vervangen in berichten... |  |
| Herschrijven: Berichtregels verwijderen... |  |
| Herschrijven: Trailers toevoegen of verwijderen... |  |
| Herschrijven: Handtekeningen verwijderen... |  |
| Herschrijven: Tekencodering van berichten herstellen... |  |
| Herschrijven: Zoeken en vervangen in bestanden... |  |
| Herschrijven: Bestanden uit de geschiedenis verwijderen... |  |
| Herschrijven: Grote bestanden uit de geschiedenis verwijderen... |  |
| Herschrijven: Paden verplaatsen of hernoemen... |  |
| Herschrijven: Bestanden naar LFS migreren... |  |
| Herschrijven: Bestanden uit LFS terughalen... |  |
| Herschrijven: Regeleindes omzetten... |  |
| Herschrijven: Het uitvoerbaarheidsbit verwijderen... |  |
| Herschrijven: Tekencodering van bestanden omzetten... |  |
| Herschrijven: Een script uitvoeren... |  |
| ✦ Herschrijven: Een script schrijven met AI... |  |
| ✦ Herschrijven: Regels maken met AI... |  |
| Herschrijven: Van de geselecteerde commit tot HEAD... |  |
| Herschrijven: Geselecteerde commits... |  |
| LFS: Initialiseren voor deze repository |  |
| LFS: Patroon volgen... |  |
| LFS: Patroon niet meer volgen... |  |
| LFS: Objecten fetchen |  |
| LFS: Menu openen | `Ctrl+G l` |
| LFS: Fetch (alle objecten) | `Ctrl+G F` |
| LFS: Alle bestanden uitchecken | `Ctrl+G K` |
| LFS: Push (alle objecten) | `Ctrl+G J` |
| LFS: Objecten pullen | `Ctrl+G L` |
| LFS: Objecten pushen |  |
| LFS: Lokale objecten opruimen | `Ctrl+G V` |
| LFS: Vergrendelingen tonen |  |
| LFS: Installatiepagina openen |  |
| LFS: Ruwe pointer in diff omschakelen | `r` |
| LFS: Geselecteerd bestand volgen |  |
| LFS: Extensie van geselecteerd bestand volgen |  |
| LFS: Geselecteerd bestand vergrendelen |  |
| LFS: Geselecteerd bestand ontgrendelen |  |
| LFS: Geselecteerd bestand geforceerd ontgrendelen |  |
| Refs: Geselecteerde branch of tag verwijderen... | `Ctrl+G d` |
| Refs: Geselecteerde branch van zijn remote verwijderen... |  |
| Refs: Geselecteerde branch lokaal en op zijn remote verwijderen... |  |
| Refs: Link naar geselecteerde branch op remote kopiëren |  |
| Git: Link naar geselecteerde commit op remote kopiëren |  |
| Git: Geannoteerde tag op geselecteerde commit... |  |
| Refs: Geselecteerde tag pushen naar standaardremote |  |
| Refs: Geselecteerde tag van zijn remote verwijderen... |  |
| Git: Geselecteerde tag fetchen |  |
| Refs: Geselecteerde tag annoteren... |  |
| Refs: Verbergen in grafiek omschakelen |  |
| Refs: In grafiek tonen | `g` |
| Refs: Solo op geselecteerde ref omschakelen | `o` |
| Refs: Solo stoppen (elke solo-ref wissen) | `O` |
| Refs: Alle verborgen refs tonen | `H` |
| Refs: Verbergen op geselecteerde map/remote omschakelen |  |
| Refs: Solo op geselecteerde map/remote omschakelen | `o` |
| Refs: Filteren op naam... | `/` |
| Refs: Naamfilter wissen | `Esc` |
| Refs: Alle mappen uitklappen | `=` |
| Refs: Alle mappen inklappen | `-` |
| Refs: De geselecteerde map en zijn submappen uitklappen | `*` |
| Refs: De geselecteerde map en zijn submappen inklappen | `_` |
| Refs: Volgende sectie | `l` |
| Refs: Vorige sectie | `h` |
| Refs: Actieve sectie in-/uitklappen | `Enter` |
| Refs: Branchhiërarchie omschakelen | `y` |
| Git: Geselecteerd bestand stagen | `Ctrl+G s` |
| Git: Geselecteerd bestand unstagen | `Ctrl+G u` |
| Git: Alle wijzigingen stagen | `Ctrl+G a` |
| Lokale wijzigingen: Geselecteerd bestand verwerpen... |  |
| Lokale wijzigingen: Geselecteerd bestand negeren |  |
| Lokale wijzigingen: Extensie van geselecteerd bestand negeren |  |
| Lokale wijzigingen: Geselecteerd bestand als binair behandelen... |  |
| Lokale wijzigingen: Geselecteerd bestand als tekst behandelen... |  |
| Lokale wijzigingen: Extensie van geselecteerd bestand als binair behandelen... |  |
| Lokale wijzigingen: Extensie van geselecteerd bestand als tekst behandelen... |  |
| Lokale wijzigingen: Geselecteerd bestand stashen |  |
| Lokale wijzigingen: Geschiedenis van geselecteerd bestand |  |
| Lokale wijzigingen: Blame van geselecteerd bestand |  |
| Lokale wijzigingen: Geselecteerd bestand openen in externe diff-tool |  |
| Lokale wijzigingen: Geselecteerd bestand openen in externe editor |  |
| Lokale wijzigingen: Geselecteerd bestand openen in standaardprogramma |  |
| Lokale wijzigingen: Geselecteerd bestand tonen in map |  |
| Lokale wijzigingen: Pad van geselecteerd bestand kopiëren |  |
| Lokale wijzigingen: Patch maken van geselecteerd bestand... |  |
| Lokale wijzigingen: Geselecteerd bestand bewerken |  |
| Lokale wijzigingen: Geselecteerd bestand verwijderen... |  |
| Lokale wijzigingen: Menu van geselecteerde map openen |  |
| Lokale wijzigingen: Bestand maken |  |
| Lokale wijzigingen: Alle wijzigingen verwerpen... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Gestagede wijzigingen nu committen | `Ctrl+G Enter` |
| Git: WIP-commit-bericht bewerken (in de Log) | `c in Log` |
| Git: Alle remotes fetchen | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward indien mogelijk) |  |
| Git: Pull (alleen fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Pull-opties... |  |
| Git: Push | `Ctrl+G P` |
| Git: Pushen naar remote... |  |
| Git: Force push (met lease)... |  |
| Git: Remotes of Branch... |  |
| Remote: Remote toevoegen... |  |
| Remote: Remote verwijderen... |  |
| Remote: Remote bewerken... |  |
| Remote: Standaardremote instellen... |  |
| Remote: Remotenaam kopiëren |  |
| Remote: Link naar remote kopiëren |  |
| Remote: Remote fetchen... |  |
| Remote: Acties voor geselecteerde remote... | `.` |
| Repository: Gedeeltelijke checkout... |  |
| Repository: SSH & inloggegevens... |  |
| Repository: Verbinding testen |  |
| Repository: Standaard inloggegevens gebruiken |  |
| Flow: Menu openen | `Ctrl+G W` |
| Flow: Branch starten... |  |
| Flow: Nieuw issue... |  |
| Flow: Door de host gemaakte branch oppakken... |  |
| Flow: Huidige branch afronden... |  |
| Flow: Synchroniseren met bovenliggende branch... |  |
| Flow: Promoveren... |  |
| Flow: Bovenliggende branch instellen... |  |
| Flow: Gestopte actie voortzetten of afbreken... |  |
| Flow: Gegroepeerde branches omschakelen |  |
| Flow: Workflowinstellingen... |  |
| Remote: Remote in grafiek verbergen... |  |
| Remote: Remote in grafiek tonen... |  |
| Git: Nieuwe branch op HEAD... | `Ctrl+G b` |
| Git: Nieuwe tag op HEAD... | `Ctrl+G t` |
| Git: Alle niet-gepushte tags pushen | `Ctrl+G U` |
| Git: Alle wijzigingen stashen | `Ctrl+G T` |
| Git: Alle wijzigingen stashen met bericht... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: Geselecteerde commit cherry-picken op HEAD | `Ctrl+G y` |
| Git: Geselecteerde commit uitchecken (losgekoppelde HEAD) |  |
| Git: Branch maken op geselecteerde commit... |  |
| Git: Tag maken op geselecteerde commit... |  |
| Commit: Bericht kopiëren |  |
| Commit: Naam van auteur kopiëren |  |
| Commit: E-mail van auteur kopiëren |  |
| Commit: Datum kopiëren |  |
| Commit: Korte SHA kopiëren |  |
| Commit: Volledige SHA kopiëren |  |
| Git: Geselecteerde commit reverten... | `Ctrl+G v` |
| Git: Commit-bericht bewerken... | `Ctrl+G e` |
| Commit: Wijzigen... | `Ctrl+G E` |
| Commit: Auteur wijzigen... | `Ctrl+G A` |
| Commit: Samenvatting wijzigen... |  |
| Commit: Beschrijving wijzigen... |  |
| Git: Geselecteerde Commit splitsen... |  |
| Git: Interactief rebasen vanaf geselecteerde commit... |  |
| Rebase: Doorgaan |  |
| Rebase: Overslaan |  |
| Rebase: Afbreken |  |
| git am: Doorgaan |  |
| git am: Deze patch overslaan |  |
| git am: Afbreken |  |
| Repository: Verouderde index.lock verwijderen |  |
| Banner: Openstaande auto-stash poppen |  |
| Banner: Openstaande auto-stash afronden (resultaat behouden) |  |
| Banner: Openstaande auto-stash tonen |  |
| Conflicten: Voorspelde conflicten tonen |  |
| Banner: Openstaande auto-stash droppen... |  |
| Banner: Opnieuw beginnen... |  |
| Git: Branch resetten naar geselecteerde commit (soft - index en werkmap behouden) |  |
| Git: Branch resetten naar geselecteerde commit (mixed - werkmap behouden) | `Ctrl+G R` |
| Git: Branch resetten naar geselecteerde commit (hard - alles verwerpen)... |  |
| Git: Conflict oplossen met ours (deze branch) |  |
| Git: Conflict oplossen met theirs (inkomend) |  |
| Git: Conflict oplossen met base (gemeenschappelijke voorouder) |  |
| Conflicten: Merge-editor openen | `Enter op een bestand met conflict` |
| Merge: Volgend conflict | `F7` |
| Merge: Vorig conflict | `Shift+F7` |
| Merge: Volgend bestand met conflict | `Alt+PageDown` |
| Merge: Vorig bestand met conflict | `Alt+PageUp` |
| Merge: Blok van A overnemen |  |
| Merge: Blok van B overnemen |  |
| Merge: Huidig overnemen voor dit conflict | `a` |
| Merge: Inkomend overnemen voor dit conflict | `b` |
| Merge: Beide overnemen, A dan B | `c` |
| Merge: Beide overnemen, B dan A | `C` |
| Merge: Basis overnemen voor dit conflict |  |
| Merge: Dit conflict wissen | `x` |
| Merge: Volgorde omwisselen | `s` |
| Merge: Alles van Huidig overnemen | `A` |
| Merge: Alles van Inkomend overnemen | `B` |
| Merge: Dit conflict bewerken in uitvoer |  |
| Merge: Zijpanelen tonen of verbergen | `F9` |
| Merge: Panelen naar links scrollen | `Shift+Left` |
| Merge: Panelen naar rechts scrollen | `Shift+Right` |
| Merge: Basispaneel omschakelen | `Alt+D` |
| Merge: Diff van heel bestand omschakelen | `Alt+F` |
| Merge: Ongedaan maken | `Ctrl+Z` |
| Merge: Opnieuw uitvoeren | `Ctrl+Y` |
| Merge: Opslaan zonder als opgelost te markeren | `Alt+S` |
| Merge: Opslaan en als opgelost markeren | `Ctrl+S` |
| Merge: Uitvoer terugzetten naar merge-resultaat | `Alt+R` |
| Merge: Hoogte van uitvoer doorlopen | `z` |
| Merge: Huidig paneel verbreden | `Alt+Right` |
| Merge: Inkomend paneel verbreden | `Alt+Left` |
| Merge: Paneelgroottes resetten | `Alt+0` |
| Merge: Keuze voor dit conflict ongedaan maken | `Alt+U` |
| Merge: Focus op paneel Huidig (A) | `Alt+1` |
| Merge: Focus op paneel Inkomend (B) | `Alt+2` |
| Merge: Focus op paneel Uitvoer | `Alt+3` |
| Merge: Focus op paneel Basis | `Alt+4` |
| Merge: Naar paneel links (zelfde conflict) |  |
| Merge: Naar paneel rechts (zelfde conflict) |  |
| Merge: Focus op volgend paneel | `F6` |
| Merge: Focus op vorig paneel | `Shift+F6` |
| Merge: Doorgaan vanaf het bestand op schijf |  |
| Merge: Opnieuw beginnen vanaf de merge |  |
| Merge: Oplossing ongedaan maken en heropenen |  |
| Merge: Bestand herladen |  |
| Merge: Huidig bestand ongewijzigd gebruiken | `a` |
| Merge: Inkomend bestand ongewijzigd gebruiken | `b` |
| Merge: Basisbestand ongewijzigd gebruiken | `o` |
| Merge: Het overgebleven bestand behouden | `k` |
| Merge: De verwijdering accepteren | `D` |
| Merge: Openen in externe merge-tool | `m` |
| ✦ AI: Oplossing voor merge-conflict voorstellen | `Alt+I` |
| Merge: Merge-editor sluiten |  |
| Git: Merge, cherry-pick of revert afbreken |  |
| Git: Cherry-pick of revert overslaan |  |
| Repository: Openen of wisselen (repositorybeheer)... | `Ctrl+O` |
| Repository: Naar een repository bladeren... | `b (beheertabblad)` |
| Repository: Nieuwe maken (init)... | `i (beheertabblad)` |
| Repositories: Geselecteerde openen | `Enter` |
| Repositories: Geselecteerde aan favorieten toevoegen of eruit verwijderen | `f` |
| Repositories: Geselecteerde sluiten of verwijderen | `x` |
| Repositories: Menu van geselecteerde repository... | `.` |
| Repositories: Zoeken | `/ / Ctrl+F` |
| Repositories: Alle secties inklappen | `-` |
| Repositories: Alle secties uitklappen | `+` |
| Repositories: Alle tabbladen sluiten... |  |
| Repositories: Geselecteerde uit lijst verwijderen |  |
| Repositories: Alle recente verwijderen |  |
| Werkruimte: Nieuw... | `n` |
| Werkruimte: Nieuw van open tabbladen... | `a` |
| Werkruimte: Wisselen... | `Alt+W` |
| Werkruimte: Bewerken... | `e` |
| Werkruimte: Sluiten (geen werkruimte) |  |
| Werkruimte: Beheren... | `Alt+J` |
| Werkruimte: Tabblad sluiten |  |
| Werkruimte: Ontbrekende repository zoeken... | `l` |
| Werkruimte: Verwijderen... |  |
| Werkruimte: Alles fetchen | `F` |
| Werkruimte: Alles pullen | `P` |
| Werkruimte: Ontbrekende repositories klonen | `m` |
| Werkruimte: Tabblad aan werkruimte toevoegen |  |
| Werkruimte: Tabblad uit werkruimte verwijderen |  |
| Radar: Openen | `Alt+R` |
| Radar: Vernieuwen | `F5` |
| Radar: Zoeken | `/` |
| Radar: Openen in browser | `o` |
| Radar: Repository openen | `Enter` |
| Radar: Branch uitchecken | `c` |
| Radar: Worktree maken... | `t` |
| Radar: Tonen in de repository | `s` |
| Radar: Werk aan issue starten... | `n` |
| Radar: Vastpinnen of losmaken | `p` |
| Radar: Uitstellen... | `z` |
| Radar: Verzoek reviewen... | `e` |
| Radar: Reviewen via de API |  |
| Radar: Klonen en reviewen... |  |
| Radar: Toegewezenen bewerken... |  |
| Radar: Reviewers bewerken... |  |
| Radar: Labels bewerken... |  |
| Radar: Mijlpaal instellen... |  |
| Radar: Bereik kiezen | `w` |
| Radar: Weergave kiezen | `v` |
| Radar: Filter kiezen | `f` |
| Radar: Niet-gekloonde verzoeken tonen | `a` |
| Radar: Concepten tonen | `d` |
| Radar: Accountbrede reviewverzoeken inschakelen... | `T` |
| Radar: Rijmenu... | `.` |
| Radar: Kolom kiezen | `C` |
| Radar: Kolom verbreden | `]` |
| Radar: Kolom versmallen | `[` |
| Radar: Kolombreedtes herstellen | `0` |
| Radar: Kolom aan inhoud aanpassen | `=` |
| Radar: Alle kolommen aan inhoud aanpassen | `+` |
| Radar: Terug naar Start | `Esc` |
| Radar: Tabblad sluiten |  |
| Tabblad: Nieuw tabblad | `Alt+T` |
| Tabblad: Nieuw... |  |
| Tabblad: Actiemenu... | `Ctrl+T` |
| Tabblad: Andere tabbladen sluiten... |  |
| Tabblad: Tabbladen rechts sluiten... |  |
| Tabblad: Aan favorieten toevoegen of eruit verwijderen |  |
| Tabblad: Repositorypad kopiëren |  |
| Tabblad: Openen in bestandsbeheer |  |
| Tabblad: Openen in terminal |  |
| Tabblad: Openen in... | `Alt+O` |
| Tabblad: Openen in standaard | `Ctrl+Alt+O` |
| Repository: Profiel wijzigen... |  |
| Help: Over GitPoulpi | `Ctrl+N a` |
| Help: Licenties en vermeldingen |  |
| Weergave: Activiteitenlogboek | `Ctrl+N A` |
| Weergave: Globaal activiteitenlogboek | `Ctrl+N G` |
| Help: Releasenotes |  |
| Updates: Updatepaneel tonen | `Ctrl+N u` |
| Updates: Beveiligingsmelding openen |  |
| Updates: Updateopdracht kopiëren |  |
| Updates: Updateopdracht in een terminal uitvoeren |  |
| Updates: Nu op updates controleren |  |
| Updates: Downloaden |  |
| Updates: Nu bijwerken |  |
| Updates: Herstarten om bij te werken |  |
| Updates: Terug naar de vorige versie |  |
| Updates: Deze versie overslaan |  |
| Releasenotes: Tabblad sluiten |  |
| Taken: achtergrondtaken tonen | `Ctrl+N J` |
| Taken: alles annuleren wat kan |  |
| Weergave: Diagnose kopiëren |  |
| Activiteitenlogboek: niveau tonen of verbergen: Fout |  |
| Activiteitenlogboek: niveau tonen of verbergen: Waarschuwing |  |
| Activiteitenlogboek: niveau tonen of verbergen: Info |  |
| Activiteitenlogboek: niveau tonen of verbergen: Debug |  |
| Activiteitenlogboek: niveau tonen of verbergen: Trace |  |
| Activiteitenlogboek: alle niveaus tonen |  |
| Activiteitenlogboek: geen niveau tonen |  |
| Activiteitenlogboek: Applicatielogboek wissen |  |
| Activiteitenlogboek: Het bereik kiezen |  |
| Activiteitenlogboek: Git-bewerkingen |  |
| Activiteitenlogboek: Applicatielogboek |  |
| Activiteitenlogboek: Bewerking van deze regel tonen |  |
| Activiteitenlogboek: Volgende mislukte bewerking |  |
| Activiteitenlogboek: Vorige mislukte bewerking |  |
| Activiteitenlogboek: Naar links scrollen | `Left` |
| Activiteitenlogboek: Naar rechts scrollen | `Right` |
| Activiteitenlogboek: Laatste fout kopiëren |  |
| Activiteitenlogboek: Voorgestelde actie uitvoeren |  |
| Activiteitenlogboek: Laatste fout openen |  |
| Tabblad: Afsluiten en cd hier |  |
| Repository: Klonen... | `Ctrl+Shift+N` |
| Repository: Volgend tabblad | `Ctrl+PageDown` |
| Repository: Vorig tabblad | `Ctrl+PageUp` |
| Tabblad: Naar links verplaatsen |  |
| Tabblad: Naar rechts verplaatsen |  |
| Repository: Tabblad sluiten | `Ctrl+W` |
| Repository: Tabblad kiezen... | `t` |
| Zoeken: Werkruimte... | `Ctrl+Alt+F` |
| Bewerken: Ongedaan maken | `Ctrl+Z` |
| Bewerken: Opnieuw uitvoeren | `Ctrl+Y` |
| Bewerken: Geblokkeerd ongedaan-maken-item vergeten |  |
| Bewerken: Geschiedenis van ongedaan maken... | `Alt+Z` |
| Bewerken: Reflog... |  |
| Bewerken: Verloren commits terughalen... |  |
| Bewerken: De branch herstellen naar waar de geblokkeerde actie hem achterliet | `Alt+Shift+Z` |
| Review: Overzicht tonen |  |
| Review: Bestanden tonen |  |
| Review: Commits tonen |  |
| Review: Opmerking bij regel plaatsen... | `c` |
| Review: Opmerking in het gesprek plaatsen... | `C` |
| Review: Op discussie reageren... | `r` |
| Review: Discussie oplossen of heropenen | `x` |
| Review: Discussie inklappen of uitklappen | `z` |
| Review: Conceptopmerking bewerken... | `e` |
| Review: Conceptopmerking verwijderen | `d` |
| Review: Bestand als bekeken markeren | `v` |
| Review: Volgend bestand | `]` |
| Review: Vorig bestand | `[` |
| Review: Review afronden... | `f` |
| Review: Concept-review verwerpen... |  |
| Review: Vernieuwen | `F5` |
| Review: Openen in browser | `o` |
| Review: Branch uitchecken | `b` |
| Review: Sluiten | `Esc` |
| Review: Toegewezenen bewerken... | `a` |
| Review: Reviewers bewerken... | `p` |
| Review: Labels bewerken... | `l` |
| Review: Mijlpaal instellen... | `m` |
| Review: Patchset kiezen... | `P` |
| Review: Patchsets vergelijken... | `D` |
| Review: Wijziging indienen... | `U` |
| Review: Wijziging opgeven... | `Q` |
| Review: Lay-out inline / split omschakelen | `V` |
| Review: Heel bestand / alleen hunks omschakelen | `H` |
| Review: Alleen-witruimtewijzigingen omschakelen | `S` |
| Review: Regelafbreking omschakelen | `W` |
| Review: Volgende hunk | `J` |
| Review: Vorige hunk | `K` |
| Review: Codering kiezen... | `E` |
| Review: Minimap omschakelen | `M` |
| Review: Diff naar links scrollen | `Left` |
| Review: Diff naar rechts scrollen | `Right` |
| Review: Geselecteerde tekst of regel kopiëren | `Ctrl+C` |
| Bord: Kaart verplaatsen naar... | `m` |
| Bord: Kaart naar links verplaatsen |  |
| Bord: Kaart naar rechts verplaatsen |  |
| Bord: Kaart openen | `Enter` |
| Bord: Toegewezenen bewerken... |  |
| Bord: Labels bewerken... |  |
| Bord: Mijlpaal instellen... |  |
| Bord: Nieuw issue in deze kolom | `n` |
| Bord: Filteren op toegewezene... | `a` |
| Bord: Filteren op label... | `l` |
| Bord: Filteren op mijlpaal... | `M` |
| Bord: Filteren op tekst | `/` |
| Bord: Filters wissen | `x` |
| Bord: Kolomlabels instellen... | `c` |
| Bord: Vernieuwen | `r` |
| Bord: Sluiten | `Esc` |
| Review: ✦ Review opstellen... | `s` |
| Review: Suggestie accepteren | `y` |
| Review: Suggestie bewerken en accepteren... | `w` |
| Review: Suggestie negeren | `n` |
| Review: Alle suggesties accepteren | `A` |
| Review: Alle suggesties negeren | `N` |
| Review: AI-samenvatting in reviewtekst gebruiken | `i` |
| Review: ✦ Het concept gebruiken... | `u` |
| Review: De ✦ concepttekst bewerken... | `t` |
| Review: Het ✦ concept verwerpen | `N` |
| Geschiedenis: Ongedaan maken tot hier... | `u` |
| Geschiedenis: Opnieuw tot hier... | `y` |
| Geschiedenis: Alleen dit herstellen... | `r` |
| Geschiedenis: De branch herstellen naar waar hij was... | `o` |
| Geschiedenis: Vernieuwen |  |
| Geschiedenis: Sluiten | `Esc` |
| Reflog: Tonen in het Log | `l` |
| Reflog: Branch hier maken... | `b` |
| Reflog: Tag hier maken... | `t` |
| Reflog: Wijzigingen toepassen op de werkmap | `w` |
| Reflog: Wijzigingen bewaren als stash | `s` |
| Reflog: Herstellen naar deze positie... | `r` |
| Reflog: Meer laden | `m` |
| Reflog: Vernieuwen |  |
| Verloren werk: Branch hier maken... | `b` |
| Verloren werk: Tag hier maken... | `t` |
| Verloren werk: Cherry-pick | `c` |
| Verloren werk: Wijzigingen toepassen op de werkmap | `w` |
| Verloren werk: Wijzigingen bewaren als stash | `s` |
| Verloren werk: Opnieuw zoeken |  |
| Reflog / Verloren werk: Volgend paneel | `Tab` |
| Reflog / Verloren werk: Bestandslijst als paden of boomstructuur | `v` |
| Reflog / Verloren werk: Paneelgroottes herstellen |  |
| Dialoog: Verplaatsen of schalen | `Ctrl+N m` |
| Dialoog: Positie en grootte herstellen |  |
| Weergave: Vernieuwen | `r` |
| Weergave: Vernieuwen (fetch, pull requests, issues) | `F5` |
| Weergave: Help-overlay omschakelen | `?` |
| Weergave: Ref-kolom omschakelen |  |
| Weergave: Grafiekkolom omschakelen |  |
| Weergave: Berichtkolom omschakelen |  |
| Weergave: Auteurkolom omschakelen |  |
| Weergave: Datumkolom omschakelen |  |
| Weergave: Wijzigingenkolom omschakelen |  |
| Weergave: SHA-kolom omschakelen |  |
| Weergave: Logkolom: Kolom kiezen om te verplaatsen of de breedte te wijzigen | `C` |
| Weergave: Logkolom: Volgende kolom kiezen | `Tab` |
| Weergave: Logkolom: Vorige kolom kiezen | `Shift+BackTab` |
| Weergave: Logkolom: Kolom naar links verplaatsen | `<` |
| Weergave: Logkolom: Kolom naar rechts verplaatsen | `>` |
| Weergave: Logkolom: Kolom verbreden | `]` |
| Weergave: Logkolom: Kolom versmallen | `[` |
| Weergave: Logkolom: Kolombreedtes herstellen | `0` |
| Weergave: Logkolom: Kolom aan inhoud aanpassen | `=` |
| Weergave: Logkolom: Alle kolommen aan inhoud aanpassen | `+` |
| Weergave: Log: Kolommen naar links scrollen | `Shift+Left` |
| Weergave: Log: Kolommen naar rechts scrollen | `Shift+Right` |
| Weergave: Log: Zijwaarts scrollen van de kolommen aan- of uitzetten | `s` |
| Weergave: Log: Kolommen kiezen om te tonen |  |
| Diff: Diff van geselecteerd bestand openen | `Enter` |
| Diff: Diff-weergave sluiten | `Esc` |
| Diff: Volgende hunk | `n` |
| Diff: Vorige hunk | `p` |
| Diff: Hunk onder cursor stagen | `Space` |
| Diff: Hunk onder cursor unstagen | `Space` |
| Diff: Regels selecteren (of selectie wissen) | `v` |
| Diff: Geselecteerde regels stagen | `Enter` |
| Diff: Geselecteerde regels unstagen | `Enter` |
| Diff: Hunk onder cursor verwerpen... | `x` |
| Diff: Geselecteerde regels verwerpen... | `X` |
| Kopiëren: Geselecteerde tekst (sleep eerst over de tekst) | `Ctrl+C` |
| Kopiëren: Regel onder cursor (of geselecteerde regels) | `y` |
| Diff: Hunk onder cursor kopiëren |  |
| Diff: Lay-out inline (+/- in één kolom) | `m` |
| Diff: Lay-out split (voor en na) | `m` |
| Diff: Alleen hunks omschakelen (heel bestand / alleen wijzigingen) | `H` |
| Bestanden: Pad/boom omschakelen | `m` |
| Bestanden: Alle bestanden tonen omschakelen | `a` |
| Commit: Hooks overslaan omschakelen | `H` |
| Bestanden: menu van de rij of selectie openen | `.` |
| Bestanden: Alle mappen uitklappen | `=` |
| Bestanden: Alle mappen inklappen | `-` |
| Bestanden: De geselecteerde map en zijn submappen uitklappen | `*` |
| Bestanden: De geselecteerde map en zijn submappen inklappen | `_` |
| Bestanden: alles selecteren |  |
| Bestanden: selectie wissen |  |
| Bestanden: selectie van de huidige rij omschakelen |  |
| Log: Kopiëren als Mermaid gitGraph... | `Y` |
| Log: Selectie vergelijken | `v` |
| Log: Uiteinden van de vergelijking omwisselen | `x` |
| Log: naar de volgende vergeleken rij | `n` |
| Diff: Regelafbreking omschakelen | `w` |
| Diff: Codering kiezen... | `C` |
| Weergave: Minimap omschakelen | `M` |
| Weergave: commit-kolom tonen of verbergen | `Alt+C` |
| Weergave: commit-kolom: verbergen |  |
| Weergave: commit-kolom: bestandsgeschiedenis |  |
| Weergave: commit-kolom: alle commits |  |
| Weergave: Focus op de commit-kolom |  |
| Weergave: commit-kolom verbreden |  |
| Weergave: commit-kolom versmallen |  |
| Weergave: commit-kolom: naar links scrollen | `Shift+Left` |
| Weergave: commit-kolom: naar rechts scrollen | `Shift+Right` |
| Weergave: commit-kolom: volgend deel kiezen | `Tab` |
| Weergave: commit-kolom: deel naar links verplaatsen | `<` |
| Weergave: commit-kolom: deel naar rechts verplaatsen | `>` |
| Weergave: commit-kolom: deel verbreden | `]` |
| Weergave: commit-kolom: deel versmallen | `[` |
| Weergave: commit-kolom: breedtes van de delen herstellen | `0` |
| Weergave: commit-kolom: Onderdeel aan inhoud aanpassen | `=` |
| Weergave: commit-kolom: Alle onderdelen aan inhoud aanpassen | `+` |
| Weergave: commit-kolom: menu van de geselecteerde rij | `.` |
| Weergave: commit-kolom: kolommen kiezen |  |
| Weergave: commit-kolom: Ref omschakelen |  |
| Weergave: commit-kolom: Grafiek omschakelen |  |
| Weergave: commit-kolom: Auteur omschakelen |  |
| Weergave: commit-kolom: Datum omschakelen |  |
| Weergave: commit-kolom: Wijzigingen omschakelen |  |
| Weergave: commit-kolom: SHA omschakelen |  |
| Weergave: commit-kolom: geselecteerde rij tonen in de Log |  |
| Weergave: commit-kolom: wijzigingen van het bestand bij de geselecteerde rij tonen |  |
| Weergave: commit-kolom: bestand tonen zoals het was bij de geselecteerde rij |  |
| Weergave: commit-kolom: blame van het bestand tonen bij de geselecteerde rij |  |
| Editor: Selectie volgen in- of uitschakelen |  |
| Afbeelding: Inzoomen | `+` |
| Afbeelding: Uitzoomen | `-` |
| Afbeelding: Aanpassen aan weergave | `0` |
| Afbeelding: Ware grootte (1:1) | `1` |
| Afbeelding: Voor / na wisselen | `Tab` |
| Afbeelding: Voor-paneel verbreden | `Alt+Right` |
| Afbeelding: Na-paneel verbreden | `Alt+Left` |
| Afbeelding: Paneelgroottes herstellen | `Alt+0` |
| Afbeelding: Bewerken in... | `O` |
| Afbeelding: Bewerken in de standaardeditor | `o` |
| Bestandsgeschiedenis: Tonen in de Log | `Enter` |
| Bestandsgeschiedenis: SHA van de commit kopiëren | `y` |
| Bestandsgeschiedenis: Samenvatting kopiëren | `Y` |
| Bestandsgeschiedenis: Diff van deze commit tonen | `d` |
| Bestandsgeschiedenis: Bestand openen bij deze commit | `o` |
| Bestandsgeschiedenis: Blame bij deze commit | `b` |
| Bestandsgeschiedenis: Vergelijken met de werkmap | `c` |
| Bestandsgeschiedenis: Deze versie herstellen | `R` |
| Bestandsgeschiedenis: Selectie omhoog uitbreiden | `Shift+Up` |
| Bestandsgeschiedenis: Selectie omlaag uitbreiden | `Shift+Down` |
| Bestandsgeschiedenis: Wisselen tussen lijst en diff | `Tab` |
| Bestandsgeschiedenis: Lijst verbreden | `Alt+Right` |
| Bestandsgeschiedenis: Lijst versmallen | `Alt+Left` |
| Bestandsgeschiedenis: Lijstbreedte herstellen | `Alt+0` |
| Bestand: Openen met... | `O` |
| Bestand: Openen in standaardtoepassing | `o` |
| Diff: Alleen-witruimtewijzigingen omschakelen | `s` |
| ✦ AI: Commit-bericht genereren | `Ctrl+G i` |
| ✦ AI: Stash-bericht genereren |  |
| ✦ AI: Promptvariabele invoegen |  |
| ✦ AI: Profiel bewerken... |  |
| ✦ AI: De agent openen... |  |
| ✦ AI: Geselecteerde commit uitleggen | `Ctrl+G w` |
| ✦ AI: Geselecteerde Commit splitsen... |  |
| ✦ AI: Lokale wijzigingen uitleggen | `Ctrl+G w` |
| ✦ AI: Commits samenstellen uit lokale wijzigingen... | `Ctrl+G C` |
| Composer: Bestand naar commit verplaatsen... | `m` |
| Diff: Blame |  |
| Blame: Commit tonen in de Log | `Enter` |
| Blame: SHA van commit kopiëren | `Y` |
| Blame: Diff van de commit voor dit bestand | `d` |
| Blame: Blame op de ouder van de commit | `b` |
| Blame: Bestandsgeschiedenis openen | `H` |
| Blame: Detailkaart tonen of verbergen | `i` |
| Blame: Volgende hunk | `n` |
| Blame: Vorige hunk | `p` |
| Blame: Volgende hunk van dezelfde commit | `N` |
| Blame: Vorige hunk van dezelfde commit | `P` |
| Blame: Naar regel gaan | `g` |
| Blame: Annotatiekolom verbreden | `>` |
| Blame: Annotatiekolom versmallen | `<` |
| Diff: Bestandsgeschiedenis |  |
| Diff: Bestandsweergave |  |
| Bestand: Bewerken in werkmap | `e` |
| Editor: Codering wijzigen |  |
| Editor: Opnieuw openen met codering |  |
| Editor: Converteren naar codering |  |
| Editor: Regeleinden converteren |  |
| Editor: Inspringing wijzigen |  |
| App: Instellingen... | `Ctrl+, / Ctrl+N ,` |
| App: Instellingen zoeken... | `/ in Instellingen` |
| App: Sneltoetsen... |  |
| Agent: Openstaande weergave tonen | `Ctrl+N Enter` |
| Agent: Openstaande weergave sluiten |  |
| Agent: Terug | `Ctrl+N Backspace` |
| Sneltoetsen: Alles resetten... |  |
| Integraties: Gekoppelde pull request openen | `Ctrl+G g` |
| Integraties: Gekoppelde pull request bewerken op de host | `e (op een rij)` |
| Integraties: URL van gekoppelde pull request kopiëren |  |
| Integraties: Pull requests en issues vernieuwen |  |
| Integraties: Pull request uitchecken... |  |
| Integraties: Commit-verwijzing openen (#issue / !verzoek)... |  |
| Integraties: Issue openen... |  |
| Hosting: Verzoek reviewen... |  |
| Hosting: Push voor review... |  |
| Hosting: Toegewezenen bewerken... |  |
| Hosting: Reviewers bewerken... |  |
| Hosting: Labels bewerken... |  |
| Hosting: Mijlpaal instellen... |  |
| Lokale issues: Issue sluiten... |  |
| Lokale issues: Issue heropenen... |  |
| Lokale issues: Openen in Obsidian... |  |
| Issues: Bord openen |  |
| Lokale issues: Obsidian Base maken |  |
| Pull requests: Pull request maken... | `n` |
| Hosting: Nieuw issue... | `n` |
| Pull requests: Zoeken | `Enter op de zoekrij` |
| Pull requests: Filter kiezen (open / merged / alle)... | `v` |
| Pull requests: Provider kiezen... | `Enter op de providerrij` |
| Pull requests: Repository kiezen... | `Enter op de repositoryrij` |
| Pull requests: Bron ontkoppelen (geen) |  |
| Issues: Provider kiezen... | `Enter op de providerrij` |
| Issues: Bron ontkoppelen (geen) |  |
| Integraties: Profiel ontkoppelen (automatisch) |  |
| Issues: Repository kiezen... | `Enter op de repositoryrij` |
| Issues: Zoeken | `Enter op de zoekrij` |
| Issues: Filter kiezen... | `v` |
| Issues: Query / lijsten bewerken... |  |
| Commit: Issue-verwijzing invoegen... |  |
| Integraties: Profiel gebruiken voor deze repository... |  |
| Integraties: Profiel van deze repository beheren |  |
| Integraties: Integratie verbinden... |  |
| Instellingen: Radar-opties van een profiel... |  |
| Integraties: Beveiligingscontrole bekijken |  |
| App: Afsluiten | `q` |
| App: Thema... |  |
| Conflicten: Lijsten met basisbranches bewerken... |  |
| Conflicten: Basisbranches van deze repository... |  |
| App: Thema wijzigen... | `Ctrl+N y` |
| App: Taal wijzigen... | `Ctrl+N g` |
| Starter: Snelkoppeling maken |  |
| Starter: Bureaublad-snelkoppeling maken |  |
| Starter: Snelkoppelingen verwijderen |  |
| Starter: Opdrachtalias maken |  |
| Starter: Opdrachtalias verwijderen |  |
| Starter: Toevoegen aan PATH |  |
| Starter: Verwijderen uit PATH |  |
| Starter: Menu Openen in GitPoulpi installeren |  |
| Starter: Menu Openen in GitPoulpi verwijderen |  |
| App: Kleurthema van de grafiek wijzigen... | `Ctrl+N Y` |
| App: Rijkleuring van de grafiek omschakelen | `Ctrl+N T` |
<!-- palette:end -->

## Rondbewegen

| Toets | Actie |
|---|---|
| `Tab` / `Shift+Tab` | Focus op het volgende / vorige paneel |
| `Ctrl+N`, dan een badge | Focus op dat deel van het venster, of dat bedieningselement indrukken van het dialoog of de popup vooraan (zie [Navigatieprefix](#navigatieprefix-ctrln)) |
| `j` / `k`, `Up` / `Down` | De selectie verplaatsen, in elk paneel |
| `PageUp` / `PageDown`, `Home` / `End` | Een pagina verplaatsen; naar de eerste / laatste rij |
| een getal, dan `j` / `k` / `Up` / `Down` | Tel-prefix: `4j` verplaatst vier rijen. Met `PageUp` / `PageDown` is het getal een aantal pagina's. De statusbalk toont het openstaande getal (`4...`). `Esc` of een andere toets wist het. Een voorloop-`0` start nooit een getal |
| `h` / `l`, `Left` / `Right` | Refs-paneel: van sectie wisselen. Log: de lijst met gewijzigde bestanden van een commit verlaten (`h`) of ingaan (`l`) |
| `?` | De help openen of sluiten (zie [Hulp in de app](#hulp-in-de-app-)) |
| `r` | Vernieuwen |
| `F5` | Alle remotes fetchen, de pull requests en issues herladen en vernieuwen |
| `q` | Afsluiten |
| `Ctrl+Z` | De laatste handeling **ongedaan maken**. Zie [Ongedaan maken en opnieuw](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Opnieuw doen**. `Ctrl+Shift+Z` heeft het kitty-toetsenbordprotocol nodig; `Ctrl+Y` werkt altijd |
| `.`, `Menu`, `Shift+F10` | Het contextmenu van de geselecteerde rij openen |

### Ctrl+Z / Ctrl+Y in een tekstveld

Zolang een tekstveld het toetsenbord heeft, werken `Ctrl+Z` en `Ctrl+Y` (ook
`Ctrl+Shift+Z`) op de tekst van dat veld, nooit op de repository: een veld dat het
toetsenbord heeft wint altijd, dus `Ctrl+Z` in het commitbericht kan geen commit
ongedaan maken. Overal elders houden ze hun globale betekenis (het journaal voor
ongedaan maken en opnieuw hierboven). Dit geldt voor elk tekstveld en elke editor met
meerdere regels: het commitbericht, de velden en teksteditors van Instellingen, de
formulieren voor pull requests en issues, de AI-dialogen en elk filter- of zoekvak.

- **Typen** wordt woord voor woord ongedaan gemaakt (een reeks letters, een reeks
  spaties, een reeks leestekens), en typen dat langer dan een seconde pauzeert, begint
  een nieuwe stap. Meerdere backspaces achter elkaar zijn één stap. Een plakactie, een
  knipactie, `Ctrl+U` / `Ctrl+K`, een ingevoegd fragment of een geaccepteerde aanvulling
  is telkens één stap.
- Een undo brengt de tekst, de cursor en de selectie terug. Een nieuwe bewerking na een
  undo laat weg wat opnieuw gedaan had kunnen worden.
- Elk veld houdt een begrensde geschiedenis bij (de laatste 200 stappen, ongeveer 1 MB
  tekst), en vergeet die als het veld van buitenaf wordt vervangen (een map gekozen in de
  kiezer, een formulier opnieuw geladen) of een ander veld het toetsenbord neemt.
- Als er niets meer ongedaan te maken is, doet `Ctrl+Z` niets: het valt nooit door naar het
  ongedaan maken van de repository zolang een veld het toetsenbord heeft.
- De **Instructie-editor** van de AI-instellingen heeft ook knoppen **Ongedaan maken** en
  **Opnieuw**, naast Opslaan en Annuleren, uitgeschakeld zolang er niets is om naar terug of vooruit te stappen.
- De code-editor en de mergeeditor houden hun eigen undo (`Bewerken: Ongedaan maken`,
  `Merge: Ongedaan maken`), die ook hun keuzes dekt.

## Filterbare popuplijsten

Alle filterbare popuplijsten gebruiken dezelfde toetsen: het opdrachtenpalet en
zijn kiezers (thema, taal, grafiekkleuren), de tabbladkiezer en elke keuzelijst
met zoekregel (keuzes in de Instellingen, het pull-requestformulier, de
issue-verwijzingskiezer, de lijst **Openen in**). Typ om de lijst te filteren.
Deze toetsen verplaatsen de markering:

| Toets | Actie |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Een rij omhoog / omlaag verplaatsen |
| `Home` / `End` | Eerste / laatste rij |
| `PageUp` / `PageDown` | Een zichtbare pagina verplaatsen |
| `Enter` | De gemarkeerde rij kiezen |
| `Esc` | Sluiten |

In deze popups verplaatsen `Home` en `End` de lijst, niet de cursor van het
filterveld, en `Ctrl+K` / `Ctrl+J` verplaatsen de lijst ook, dus `Ctrl+K` wist niet
langer tot het einde van het filter (`Ctrl+U` wist het nog wel; in een gewoon tekstveld
buiten een lijst blijft `Ctrl+K` tot het einde van de regel wissen). `Ctrl+A` en
`Ctrl+E` verplaatsen de cursor nog steeds naar het begin en het einde. `Ctrl+J` / `Ctrl+K` verplaatsen elke andere lijst ook, met of zonder
tekstveld (`Ctrl+N` is het navigatieprefix en `Ctrl+P` het opdrachtenpalet, dus zij zijn
geen lijsttoetsen). Gewone `j` en `k` worden in het filter getypt; in lijsten
zonder tekstveld zijn `j` / `k` een extra manier om omlaag / omhoog te gaan.

Met de muis:

- Het wiel **scrolt** de lijst. Het verplaatst de markering niet.
- Een klik **selecteert** een rij. Een dubbelklik **kiest** hem, zoals `Enter`.
- Een lijst die hoger is dan zijn popup toont een schuifbalk aan de rechterrand.
  Je kunt hem slepen.
- Een korte keuzelijst zonder zoekregel (twee of drie keuzes) kiest bij de
  eerste klik.

**Live voorbeeld.** De kiezers voor thema, grafiekkleurenthema en taal passen de
gemarkeerde rij meteen toe, achter de popup. `Enter` houdt het. `Esc`, of een
klik buiten de popup, herstelt de vorige waarde.

**Themafilter.** De themakiezer heeft onder het filterveld een rij: **Alle**,
**Donker**, **Licht** en **Gekleurd**. `Tab` / `Shift+Tab` gaan naar de volgende /
vorige. `Alt+1` tot `Alt+4` kiezen er rechtstreeks een. Je kunt er ook op
klikken.

## Hulp in de app (`?`)

`?`, `Ctrl+N ?` of `Weergave: Help-overlay omschakelen` in het palet opent de
help. Hij toont de toetsen en muisgebaren van elk deel van de app. Hij opent
boven alles, ook boven het scherm Instellingen.

| Toets | Actie |
|---|---|
| `?` | De help sluiten (en het filter wissen) |
| *(typen)* | Filter: alleen de secties bewaren die de tekst bevatten. `j`, `k` en `q` worden ook getypt |
| `Backspace`, `Ctrl+U` | Het laatste teken verwijderen / het filter wissen |
| `Esc` | Het filter wissen; als het leeg is, de help sluiten |
| `Up` / `Down` | Een regel scrollen |
| `PageUp` / `PageDown`, `Home` / `End` | Een pagina scrollen; naar boven / onderen |
| `Ctrl+D`, of een klik op de link | Deze documentatie op GitHub in de browser openen |

Het wiel scrolt de help en je kunt zijn schuifbalk slepen. Een klik buiten de
help sluit hem.

## Navigatieprefix (`Ctrl+N`)

`Ctrl+N` wacht op nog één toets. De statusbalk toont de toetsen die hij
accepteert. De volgende toets voert een actie uit, of annuleert.

| Toets | Actie |
|---|---|
| `Ctrl+N` dan een badge | Focus op het deel van het venster dat deze badge toont |
| `Ctrl+N h` / `Ctrl+N l` | Het Refs-paneel / het rechterpaneel inklappen of herstellen |
| `Ctrl+N +` / `Ctrl+N -` | De actieve Refs-sectie met één rij laten groeien / krimpen |
| `Ctrl+N ]` / `Ctrl+N [` | Het commit-berichtblok van het rechterpaneel met één rij laten groeien / krimpen |
| `Ctrl+N Left` / `Ctrl+N Right` | De scheiding tussen het Refs-paneel en de Log een stap naar links / rechts verplaatsen (een ingeklapt Refs-paneel komt eerst terug) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | De scheiding tussen de Log en het rechterpaneel een stap naar links / rechts verplaatsen |
| `Ctrl+N t` | Een tabblad kiezen (de tabbladkiezer; `t` alleen doet hetzelfde) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Het thema / de taal / het grafiekkleurenthema wijzigen |
| `Ctrl+N Shift+T` | De rijkleuring van de grafiek aan- of uitzetten |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | De weergave tonen die een agent heeft voorbereid / teruggaan (zie [MCP en agents](mcp-and-agents.md)) |
| `Ctrl+N ,` | De Instellingen openen |
| `Ctrl+N ?` | De help openen of sluiten |
| `Esc`, of een andere toets | Annuleren |

### Badges

Terwijl de prefix wacht, toont elk deel van het venster dat de focus kan krijgen
een badge van één teken op zijn linkerbovenrand. Druk op dat teken om te
focussen. Badges worden elke keer opnieuw uitgedeeld, in schermvolgorde: eerst
`1` tot `9`, dan `a`, `b`, `c`... Lees dus de badge op het scherm: een deel krijgt
niet altijd dezelfde badge.

Delen met een badge: het Refs-filter en elke zichtbare Refs-sectie, de Log (of de
Diff-weergave), de Bestandsweergave als die open is, de lijsten Unstaged en
Staged, het samenvattingsveld van de commit-composer en de bediening van de lijst
met commit-bestanden. Op het tabblad Start zijn de badges het zoekveld, de vier
secties en de knoppen van de werkbalk (de badge van een knop drukt erop). Op het tabblad Radar zijn dat het filterveld, de keuzelijsten
Bereik, Weergave en Toon (de badge van een keuzelijst opent die), de knoppen
Vernieuwen en Terug, en elke sectiekop op het scherm. De badge van een sectie zet
de selectie op de kop; met `Enter` of `Space` klap je de sectie dan in.

In de mapkiezer Bladeren en in de dialogen Klonen, Initialiseren, Worktree maken, Werkruimte, Gedeeltelijke checkout, Flow, Nieuw issue en pull request zijn de badges de velden, selectievakjes en knoppen (de badge van een knop drukt erop, die van een veld zet de cursor erin; in een lijst staat één badge op de eerste rij; de mapkiezer Bladeren heeft één badge voor de lijst Snelle toegang en één voor de lijst Mappen, naast de eerste rij van elke lijst, en een badge indrukken zet het toetsenbord in die lijst zonder een rij te kiezen). In het Activiteitenlogboek zijn het de twee tabbladen, de vakken Bereik en Niveau, het filter, de niveauselectievakjes, de knoppen en het eerste item van de lijst. Een badge doet wat een klik op dat element doet. In de mapkiezer start `Ctrl+N` de badges, dus zijn toets voor Nieuwe map is `Alt+N`.

In de dialoog Wijzigen (en Commit-bericht bewerken) zijn de badges de samenvatting, de beschrijving, de naam, het e-mailadres en de datum van de auteur, **Geschiedenis dieper wijzigen...**, **Annuleren**, **Toepassen** en het sluitkruis; in de werkbank voor geschiedenis herschrijven zijn het het bereikvak, zijn selectievakjes en refs-knop, de naam van het recept, elke regel, haar selectievakje en knoppen, de knop **+ Regel toevoegen**, de velden van de geselecteerde regel, de voorbeeldtabbladen, het filtervak, de rijen van beschermde branches en hun Alles aanvinken / Alles uitvinken, en de knoppen Voorbeeld, Toepassen, Annuleren, Opslaan, Laden en Regels maken met AI (de pagina's Geheimen, Identiteiten, Grootte en resultaat voegen hun eigen knoppen en rijen toe; de rijen van het tabblad Commits, een lijst zijnde, hebben er geen). Een badge doet wat een klik doet. Er wordt niets van een badge voorzien zolang een keuzelijst, een vraag, een veld waarin je typt, de mapkiezer, het opruimscherm, de scripteditor, de AI-dialogen, de blob-diff of de host-uitleg open is, noch terwijl de dialoog Wijzigen vraagt "weggooien?"; `Esc` haalt de badges weg zonder iets te sluiten.

In de Instellingen zijn de badges het zoekvak, de paginalijst (één badge, op de huidige pagina), de subtabbladen, de lijst met zijn knoppen, elk veld van de pagina dat op het scherm staat en het sluitkruis. Een badge doet wat een klik op dat element doet: een tekstveld gaat open om te bewerken, een selectievakje wordt aangevinkt, een keuzelijst gaat open, een knop wordt uitgevoerd. `Ctrl+N` behoudt zijn eigen betekenis terwijl je in een veld typt of een keuzelijst, een editor of het zoeken openstaat.

Het palet heeft dezelfde acties, bijvoorbeeld
`Navigatie: Focus op Refs > Lokaal` of `Navigatie: Focus op Status > Staged`.

## Repositories en tabbladen

| Toets | Actie |
|---|---|
| `Ctrl+O`, of een klik op `⌂` | Naar het tabblad Start gaan (op het tabblad Start: focus op het zoekveld; met de Radar of de Releasenotes open: sluit die eerst) |
| `Ctrl+Shift+N` | Het tabblad Start openen met het kloondialoogvenster |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Volgend / vorig tabblad |
| `t` | Een tabblad kiezen uit een filterbare lijst |
| `Ctrl+W` | Het huidige tabblad sluiten (vraagt eerst; de repository wordt niet gewijzigd) |
| `Ctrl+T`, of een rechtsklik op een tabblad | Het tabbladmenu (de rijen staan bij [Tabbalk](interface-tour.md#tabbalk)) |
| `Alt+O` / `Ctrl+Alt+O` | De map van het tabblad openen: kiezen uit de lijst **Openen in** / het standaarditem gebruiken. Zie [Openen in](open-in.md) |

### Het tabblad Start

Het tabblad `⌂` beheert je repositories. Het heeft vier secties: **Geopende
repositories**, **Favorieten**, **Recente repositories** en **Alle
repositories**. Zie [Rondleiding door de interface](interface-tour.md#start).

| Toets | Actie |
|---|---|
| `j` / `k`, pijlen, `PageUp` / `PageDown`, `Home` / `End` | Verplaatsen |
| `Enter`, of een dubbelklik | De repository openen (of naar haar tabblad wisselen). Op een sectiekop: in- of uitklappen |
| `f` of `*`, of een klik op de ster | Aan favorieten toevoegen / eruit halen |
| `x`, `Delete`, of de knop van de rij | Zijn tabblad sluiten (Geopend), uit de lijst verwijderen (Recent), de ster verwijderen (Favorieten), vergeten (Alle). Er wordt niets op schijf verwijderd |
| `.`, een rechtsklik, of de `⋮` van de rij | Het menu van de rij |
| `/` of `Ctrl+F` | In alle secties zoeken op naam, branch of pad. `Enter` opent de eerste treffer; `Esc` wist de zoekopdracht en verlaat dan het veld |
| `Space`, `Left` / `Right` | De huidige sectie in- / uitklappen |
| `-` / `+` | Alles inklappen / Alles uitklappen |
| `b` / `c` / `i` | Bladeren (een map openen) / Klonen / Initialiseren |
| `Esc` | De zoekopdracht wissen, of teruggaan naar het tabblad van de actieve repository |
| `q` | Afsluiten |

`Ctrl+P`, `Ctrl+,`, `?` en `Ctrl+N` werken op het tabblad Start. `Ctrl+G`,
`Ctrl+W` en `Ctrl+T` hebben een repositorytabblad nodig: op het tabblad Start
tonen ze alleen een melding. Opdrachten van het palet die een repository nodig hebben
zijn grijs.

### De mapkiezer

**Bladeren** (en de knop **Bladeren...** van de dialoogvensters Init, Clone en
Worktree maken) opent de mapkiezer. Hij heeft bovenaan een padveld, links de
plaatsen en de mapboom, en rechts de inhoud van de huidige map.

| Toets | Actie |
|---|---|
| `Tab` / `Shift+Tab` | Padveld, dan de linkerkolom, dan de inhoud |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Verplaatsen in de gefocuste kolom. In het padveld: door de aanvullingen gaan |
| een letter (inhoud) | Naar de volgende map gaan die met deze letter begint |
| `Enter` / `Right` | De geselecteerde map openen. In het padveld: naar de getypte map gaan; staat die al getoond, kies hem dan |
| `Backspace` / `Left` / `Alt+Up` | Bovenliggende map (in het padveld verwijdert `Backspace` een teken) |
| `Tab` (padveld) | De enige aanvulling accepteren, of de gemarkeerde |
| `Alt+N` | Nieuwe map (`Ctrl+N` start de badges, zie [Badges](#badges)) |
| `Alt+H` | Verborgen mappen tonen of verbergen |
| `Ctrl+F`, of `/` buiten het padveld | De items van de map filteren terwijl je typt (niet hoofdlettergevoelig; `..` blijft staan; de kop telt "x van n"; `Enter` springt naar de eerste treffer, `Esc` wist; het filter leegt zich als je van map wisselt). Ook bereikbaar met `Tab` of een klik |
| `Ctrl+S`, `Ctrl+Enter`, of de bevestigingsknop | De geselecteerde submap kiezen, of anders de getoonde map |
| `Esc` | De aanvullingen sluiten, of annuleren |

Als het padveld een nieuw pad toont, is het hele pad geselecteerd: de eerste
toets die je typt vervangt het.

### Dialoogvensters Init en Clone

| Toets | Actie |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Volgend / vorig veld of knop |
| `Enter` | De gefocuste knop indrukken, of maken / klonen |
| `Space` | Het gefocuste selectievakje omschakelen, of de gefocuste knop indrukken |
| `Ctrl+B` | Naar de map bladeren |
| `j` / `k`, `g` / `G` | In de bronnenlijst en de repositorylijst van het kloondialoogvenster: omlaag / omhoog, eerste / laatste (niet in het zoekveld) |
| `Esc` | Sluiten. Tijdens een clone gaat het klonen op de achtergrond door |

In het kloondialoogvenster toont de linkerkolom **Klonen met URL** en één bron
per verbonden hostingprofiel (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Kies een
bron met `Up` / `Down` of een klik. Typ dan in het zoekveld **Repository**,
beweeg door de lijst met de pijlen, `PageUp` / `PageDown` of het wiel, en kies
een rij met `Enter` of een klik. Zie [Aan de slag](getting-started.md) voor de
velden.

## Refs-paneel

| Toets | Actie |
|---|---|
| `Enter` / `Space` | De actieve sectie, of de geselecteerde map, uit- of inklappen |
| `h` / `l`, `Left` / `Right` | Van sectie wisselen |
| `/` | Op naam filteren (`Enter` past toe, `Esc` annuleert) |
| `y` | Branchhiërarchieweergave aan of uit: mappen uit de `/` in branchnamen (LOKAAL en REMOTE) |
| `o` | De geselecteerde ref **solo** zetten, of elke ref onder de geselecteerde map of remote. Zet er meerdere solo om hun vereniging te zien |
| `Shift+O` | **Solo stoppen**: elke solo wissen |
| `Shift+H` | **Alles tonen**: elke verborgen ref weer tonen |
| `g` | **In grafiek tonen**: scrolt de Log naar de commit van de geselecteerde ref (of stash) en laat de rij even oplichten; de selectie verplaatst niet |
| `m` | De geselecteerde ref **markeren** als dropbron (`Esc`, of nogmaals `m` erop, wist de markering) |
| `Shift+M` | **De gemarkeerde ref droppen** op de geselecteerde ref: opent het menu dat slepen en loslaten opent (Mergen, Rebasen, Pushen, Fast-forward...) |
| `+` (sectie REMOTE) | Een remote toevoegen |
| `.`, `Menu`, `Shift+F10`, of een rechtsklik | Het menu van de rij: een branch, een tag, de naam van een remote, of een map |
| `Enter` op een rij van WORKTREES | Die worktree in een eigen tabblad openen |
| `Ctrl+G d` | De geselecteerde branch of tag verwijderen (vraagt eerst). Op een REMOTE-rij: die remote branch verwijderen |

Voor wat de menu's doen, zie [Werken met de Log](log-and-history.md).

### Pull requests en issues

Met een passende integratie heeft het Refs-paneel een sectie voor pull requests
(GitLab: merge requests). Met een verbonden hostingprofiel heeft het ook een
sectie ISSUES. Zie
[Hosting en issuetrackers](hosting-integration.md#5-pull-requests-en-issues-in-de-zijbalk).

| Toets | Actie |
|---|---|
| `Enter` | De geselecteerde rij in de browser openen. Op een rij `Opnieuw verbinden`: de Instellingen op dat profiel openen |
| `Space` | De sectie uit- of inklappen |
| `v` | Het filter van de sectie openen. Pull requests: Open, Merged / gesloten, Alle. Issues: Open, Aan mij toegewezen, Gesloten, Alle |
| `n` | Sectie pull requests: een pull request maken. Sectie ISSUES: nieuw issue. (De knop `+` van de kop) |
| `e` | Sectie pull requests: het geselecteerde verzoek op de host bewerken (het potlood van de rij) |
| `Enter` op de zoekrij | In het zoekveld typen. `Enter`, `Tab` of `Esc` verlaat het; `Ctrl+U` wist het |
| `Enter` op een groepskop | De groep in- of uitklappen (*Mijn Pull Requests*, *Aan mij toegewezen*, *Wacht op mijn review*, *Alle Pull Requests*) |
| `Enter` op de rij van filter, provider of repository | Die keuzelijst openen. De providerlijst eindigt met **Geen (ontkoppel deze repository)** |
| `.`, of een rechtsklik | Het menu van de rij: in de browser openen, URL of nummer kopiëren, de bronbranch uitchecken (pull requests); een branch maken vanuit het issue (issues) |

### Het issuebord

**Bord** in de kop van ISSUES (palet: **Issues: Bord openen**) opent het kanbanbord van de issuebron. Zie
[het bord](hosting-integration.md#5b-het-issuebord) en [lokale issues](local-issues.md#het-kanbanbord). Elke toets
hieronder is ook een paletopdracht (**Bord: ...**).

| Toets | Actie |
|---|---|
| Pijlen | Een kaart selecteren (`Up`, `Down`) of de naburige kolom (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` springen in de kolom |
| `Shift+Left`, `Shift+Right` | De geselecteerde kaart naar de vorige of volgende kolom verplaatsen |
| `m` | **Kaart verplaatsen naar...**: een keuzelijst van de kolommen |
| `Enter` | Het issue openen (de notitie, bij een lokaal issue; anders de pagina van de host) |
| `e`, `.` | Toegewezenen, labels of mijlpaal bewerken; het kaartmenu |
| `n` | Nieuw issue in de geselecteerde kolom |
| `a`, `l`, `M`, `/` | Filteren op toegewezene, label, mijlpaal, tekst. `x` wist de filters |
| `c` | De labels die de kolommen zijn van een GitHub- / GitLab-bord |
| `r` | De issues opnieuw lezen |
| `Esc` | Het bord sluiten |

Muis: sleep een kaart naar een andere kolom om hem te verplaatsen, klik op een kaart om hem te selecteren, dubbelklik om hem te openen, klik met rechts voor
het kaartmenu, gebruik het wiel om een kolom te scrollen, `Shift`+wiel (of de balk onder het bord) om de kolommen te scrollen.

### Flow (branchingworkflows)

Met een workflow ingesteld voor de repository (zie [branchingworkflows](branching-workflows.md)) heeft de werkbalk een
knop **Flow**, en groepeert de zijbalk LOKAAL en REMOTE op branchtype. Elke rij van het menu is een paletopdracht
(**Flow: ...**; **Flow: Feature starten...** en **Flow: Nieuw Feature-issue...** worden voor elk branchtype van de
workflow gegenereerd). De door de host aangestuurde rijen (*Nieuw <Type>-issue*, *Een door de host gemaakte branch oppakken*)
en hun dialogen, de wachtkaart en de kaart "er is een branch verschenen" worden beschreven in
[branchingworkflows](branching-workflows.md#door-de-host-aangestuurde-flows-in-de-app); `Enter` voert de knop met focus uit
(*Issue maken*, *Oppakken*), `Esc` stopt het wachten of sluit.

| Toets | Actie |
|---|---|
| `Ctrl+G Shift+W` | Het Flow-menu openen (de werkbalkknop) |
| `Enter`, `Tab`, `Shift+Tab` | In de dialoog Starten / Afronden / Synchroniseren / Promoveren / Bovenliggende branch instellen: voorbeeld of uitvoeren, volgend veld, vorig veld |
| `Space` | Een selectievakje aanvinken (*De branch publiceren*, *De hooks uitvoeren*, *Ook de erop gestapelde branches bijwerken*) |
| `Esc` | Een keuzelijst sluiten, van een voorbeeld terug naar het formulier, de dialoog sluiten (niet terwijl een run bezig is) |
| Aanwijzer op een branch | Zijn type, zijn bovenliggende branch en hoe ver hij achter de bovenliggende branch loopt |

## Log-paneel

| Toets | Actie |
|---|---|
| `Enter` / `l` | De lijst met gewijzigde bestanden van de geselecteerde commit ingaan |
| `Enter` / `l` op een bestandsrij | De diff van het bestand in het midden openen |
| `Esc` / `h` | Terug naar de commits |
| `Right` | (commit met meer dan één ref) De lijst van zijn andere refs openen: `Up` / `Down` verplaatsen, `Enter` checkt uit, `Esc` sluit |
| `f` | De zoekbalk openen (zie [Zoeken](#zoeken)); dit was nooit `/`, dus het zoeken van de Log is ongewijzigd |
| `/` | Het filterveld van Refs focussen (dezelfde toets doet het als het paneel Refs de focus heeft); als teken getypt als een tekstveld de focus heeft |
| `c` | (met niet-gecommitte wijzigingen) De WIP-rij selecteren en de cursor in haar berichtveld zetten |
| `Enter` op de WIP-rij | De cursor in het berichtveld van de WIP-rij zetten |
| `.`, `Menu`, `Shift+F10` | Het commit-menu. `Up` / `Down` verplaatsen, `Right` / `l` / `Enter` openen een submenu, `Left` / `h` / `Esc` gaan terug |
| `m` / `a` | Bestanden van het rechterpaneel: `Pad` of `Boom` / `Alle bestanden tonen` |
| `Shift+C` | Een Logkolom kiezen om te verplaatsen of de breedte te wijzigen (zie hieronder) |

Met een kolom gekozen kiezen `Tab` / `Shift+Tab` een andere, verplaatsen `<` / `>` haar, maken `]` / `[` haar breder of smaller, zet `0` alle breedtes terug op automatisch en verlaat `Esc` of `Enter` de keuze (context `log_columns` in de gegenereerde tabel hierboven).

Het berichtveld van de WIP-rij is de samenvatting van de commit-composer. Zolang
het de cursor heeft, wordt elke toets getypt, maar `Ctrl`-toetscombinaties werken nog.
`Enter` gaat naar de beschrijving van de composer. `Esc` verlaat het veld en
houdt de tekst. Zie [Stagen en committen](staging-and-committing.md).

### Meerdere commits selecteren

| Toets of gebaar | Actie |
|---|---|
| `Space` | De huidige commit aan de selectie toevoegen, of eruit halen |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | De selectie uitbreiden (Alt voor terminals die Shift voor zichzelf houden) |
| `Ctrl+klik` / `Shift+klik` of `Alt+klik` | Een commit omschakelen / een bereik selecteren |
| `Esc` | De selectie wissen |
| `Shift+Y` | [De selectie kopiëren als Mermaid gitGraph](log-and-history.md#commits-kopiëren-als-mermaid-gitgraph) (een popup met detailniveaus en een live voorbeeld; `Enter` kopieert) |

Rechtsklik bij meerdere geselecteerde commits op een ervan en kies **N commits
cherry-picken op `<branch>`...**. Het opent dezelfde weergave als een
interactieve rebase.

### Een branch of tag een naam geven

`Ctrl+G b`, `Ctrl+G t`, de werkbalkknoppen **Branch** en **Tag** en de menu-items
**Branch hier maken...** / **Tag hier maken...** vragen de naam in een tekstvak
op de rij van de commit. `Enter` maakt de ref. `Esc` annuleert. Een ongeldige of
bestaande naam wordt rood getoond en het vak blijft open.

Is die rij niet in beeld (een diff is open, het Log staat solo, de Ref-kolom is
verborgen), dan wordt de naam in plaats daarvan getypt in een klein kader met een
sluitkruis en de knoppen **Annuleren** en **Maken**. Een branch hernoemen doe je
direct over zijn rij in het Refs-paneel, of in de Ref-cel van zijn Log-rij; een
bestand maken doe je in de lijst met werkwijzigingen, onder de map. Overal gelden
dezelfde toetsen: `Enter` bevestigt, `Esc` annuleert.

In de statusregel wordt nooit getypt. Invoer die meer ruimte nodig heeft, is een
dialoog met echte knoppen: het stash-bericht, het bericht van een geannoteerde tag
(`Ctrl+Enter` of `Ctrl+S` past toe, `Enter` voegt een regel toe), Upstream
instellen (een doorzoekbare lijst met remote branches, plus een knop **Upstream
wissen**) en de Git LFS-patronen om te volgen en niet meer te volgen. Een
getal- of tekstinstelling die je in het palet kiest, opent Instellingen op die
rij, klaar om te bewerken, en het Refs-filter (`/`) wordt getypt in het vak boven
in het Refs-paneel.

### Een commit-bericht bewerken

**Commit-bericht bewerken...** in het commit-menu, `Ctrl+G e`, of
`Git: Commit-bericht bewerken...` in het palet opent de popup **Commit-bericht
herformuleren**. Zie [Werken met de Log](log-and-history.md) voor wat het
wijzigt.

| Toets | Actie |
|---|---|
| `Enter` in de samenvatting | Naar de beschrijving gaan |
| `Enter` in de beschrijving | Nieuwe regel. `Up` op de eerste regel gaat terug naar de samenvatting |
| `Ctrl+Enter`, `Ctrl+S`, of **Bericht bijwerken** | Het nieuwe bericht opslaan |
| `Tab` / `Shift+Tab` | Tussen de samenvatting, de beschrijving en de knoppen wisselen |
| `Esc`, of **Annuleren** | Sluiten zonder wijziging |

### Een commit wijzigen

`Ctrl+G Shift+E` (alle velden) en `Ctrl+G Shift+A` (de auteur) openen het venster Commit wijzigen, net als het commit-menu, de rijen `Commit: ... wijzigen...` van het palet en het detailpaneel van de commit; elke manier staat beschreven bij [Een commit wijzigen](log-and-history.md#een-commit-wijzigen-bericht-auteur-datums). Beide sneltoetsen zijn opnieuw toe te wijzen: `git_prefix.amend` en `git_prefix.edit_author`. Toetsen in het venster:

| Toets | Actie |
|---|---|
| `Tab` / `Shift+Tab` | Tussen de velden en de knoppen wisselen |
| `Enter` in een veld van één regel | Volgend veld; na het laatste veld: **Toepassen** |
| `Up` / `Down` | Vorig / volgend veld (in de beschrijving vanaf de eerste / laatste regel) |
| `Ctrl+Enter`, `Ctrl+S`, of **Toepassen** | De gewijzigde velden toepassen |
| `Esc`, of **Annuleren** | Sluiten zonder wijziging |
| `Alt+H`, of **Geschiedenis dieper wijzigen...** | Het venster sluiten en de werkbank voor geschiedenis herschrijven openen vanaf deze commit (alleen in builds met geschiedenis herschrijven); vraagt eerst als je een veld hebt aangepast (`Esc` bevestigt). Opnieuw toe te wijzen: `reword.deeper` |

### Worktree maken

**Worktree maken van `<ref>`...** (menu's van branch en tag) en **Worktree hier
maken...** (commit-menu) openen een dialoogvenster: de map, een optionele nieuwe
branch en **De nieuwe worktree als tabblad openen**. Toetsen: `Tab` /
`Shift+Tab` verplaatsen, `Enter` maakt, `Space` schakelt het selectievakje om,
`Ctrl+B` bladert, `Esc` annuleert.

### Interactieve rebase

**Interactief rebasen vanaf hier...** in het commit-menu, of
`Git: Interactief rebasen vanaf geselecteerde commit...`, opent de rebaseweergave.
Een cherry-pick van meerdere commits gebruikt dezelfde weergave. Zie
[Werken met de Log](log-and-history.md).

| Toets | Actie |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | De actie van de rij instellen: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup en Edit alleen in een rebase) |
| `Space` | De actielijst van de rij openen (`Up` / `Down` kiezen, `Enter` past toe, `Esc` sluit de lijst) |
| `Up` / `Down` | De selectie verplaatsen |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | De geselecteerde rij omhoog / omlaag verplaatsen |
| `a` | Autosquash-voorbeeld aan of uit |
| `Enter` op een Reword-rij | Het nieuwe bericht van die commit bewerken |
| `Ctrl+S`, `Ctrl+Enter`, of **Rebase starten** / **Cherry-pick starten** | Het plan uitvoeren |
| `Ctrl+R`, of **Reset** | Het plan herstellen zoals het was toen de weergave opende |
| `Esc`, of **Annuleren** | Sluiten zonder wijziging |

Als een rebase stopt (bij een conflict of een Edit-stap), biedt een banner boven
de Log **Doorgaan**, **Overslaan** en **Afbreken** aan. Het palet heeft
`Rebase: Doorgaan`, `Rebase: Overslaan` en `Rebase: Afbreken`.

## Zoeken

`f`, de werkbalkknop **Zoeken**, of `Navigatie: Commits zoeken...` opent de
zoekbalk boven de Log. Hij zoekt in het volledige commit-bericht, de naam en het
e-mailadres van de auteur, de SHA en de refnamen. Hij verbergt geen rijen: rijen
die niet overeenkomen worden gedimd.

| Toets | Actie |
|---|---|
| *(typen)* | De zoektekst bewerken (elke toets wordt getypt) |
| `Enter` / `Down`, of de `↑` van de balk | Volgende treffer |
| `Shift+Enter` / `Up`, of de `↓` van de balk | Vorige treffer |
| `Esc`, of de `✕` van de balk | De zoekbalk sluiten |

## Diff-weergave

Een gewijzigd bestand opent in het midden, in plaats van de Log. Zie
[Stagen en committen](staging-and-committing.md).

| Toets | Actie |
|---|---|
| `Enter` / `l`, of een dubbelklik (op een bestandsrij) | De diff van het bestand openen |
| `Esc`, `q`, of de `✕` in de titel | De diff sluiten |
| `j` / `k`, `Up` / `Down` | Scrollen |
| `PageUp` / `PageDown`, `Home` / `End` | Een pagina scrollen; naar het begin / einde |
| `h` / `l`, `Left` / `Right` | Zijwaarts scrollen (niet bij regelafbreking) |
| `n` / `p` | Volgende / vorige hunk |
| `m` | Inline- of split-lay-out |
| `Shift+H` | Alleen hunks of het hele bestand |
| `w` | Regelafbreking aan of uit |
| `s` | Wijzigingen met alleen witruimte tonen of verbergen |
| `Shift+M` | Minimap aan of uit |
| `Shift+C` | De codering kiezen waarin het bestand wordt gelezen (een lijst: eerst Automatisch detecteren) |
| `e` | Het bestand bewerken in de ingebouwde editor (zie [Editor](#editor)) |
| `r` | (Git LFS-bestand) De LFS-kaart of de ruwe pointer. Zie [Git LFS](lfs.md) |
| `Shift+E` | (bestand in conflict) De merge-editor openen. Zie [Merge-conflicten oplossen](merge-conflicts.md) |
| `y` / `Ctrl+C` | De geselecteerde tekst kopiëren. Zonder selectie: de regel onder de cursor, of de geselecteerde regels |
| `.`, of een rechtsklik op een hunk | Het hunkmenu |

### Stagen en verwerpen vanuit de diff

Deze toetsen werken alleen op een diff van de lokale wijzigingen (geopend vanuit de
lijst Unstaged of Staged). Verwerpen werkt alleen vanuit de lijst Unstaged.

| Toets | Actie |
|---|---|
| `Space` | De hunk onder de cursor stagen (Unstaged) / unstagen (Staged) |
| `v` | Een regelselectie starten of laten vallen; `j` / `k` breiden hem uit |
| `Enter` | De geselecteerde regels, of de regel onder de cursor, stagen / unstagen |
| `x` | De hunk onder de cursor verwerpen (vraagt eerst: `y` / `Enter` bevestigen, `n` / `Esc` annuleren) |
| `Shift+X` | De geselecteerde regels, of de regel onder de cursor, verwerpen (vraagt eerst) |

Alleen gewijzigde regels tellen: contextregels in een selectie worden genegeerd.
Een selectie die doorloopt in een tweede hunk werkt alleen op de eerste hunk.

## Afbeeldingsweergave

Een afbeelding in de Diff-weergave of de Bestandsweergave. De toetsen staan in
de context `image` van de tabel hierboven. Zie [Afbeeldingen](image-view.md).

## Bestandsweergave, blame en bestandsgeschiedenis

| Toets | Actie |
|---|---|
| `Esc` / `q` | Sluiten (bij een tekstselectie wist `Esc` die eerst) |
| `j` / `k`, `Up` / `Down` | De selectie verplaatsen |
| `PageUp` / `PageDown` | Een pagina scrollen |
| `Home` / `End`, `Shift+G` | Eerste / laatste regel |
| `h` / `l`, `Left` / `Right` | De tekst zijwaarts scrollen |
| `Enter` | (Blame, Bestandsgeschiedenis) Naar de commit van de rij in de Log gaan |
| `e` | Het bestand bewerken in de ingebouwde editor |
| `y` / `Ctrl+C` | De geselecteerde tekst kopiëren. Zonder selectie: de geselecteerde rij (Blame: alleen de code; Bestandsgeschiedenis: de samenvatting) |

Blame en de bestandsgeschiedenis hebben eigen toetsen, vermeld in de gegenereerde tabel hierboven (contexten `file_insight` en `history`) en als `Blame: ...` / `Bestandsgeschiedenis: ...` in het palet; alle zijn te hertoewijzen. Details bij [Blame](log-and-history.md#blame) en [Bestandsgeschiedenis](log-and-history.md#bestandsgeschiedenis). In de geschiedenislijst werken de losse letters (`d` `o` `b` `c` `y`) op de geselecteerde rij, maar een Ctrl-combinatie (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) behoudt altijd haar globale betekenis.

## Commit-kolom

De kolom links van de weergave van een bestand (zie [De commit-kolom](interface-tour.md#de-commit-kolom)). `Alt+C` toont of verbergt haar vanaf elke plek; de andere toetsen werken zodra ze het toetsenbord heeft (`Ctrl+N` en dan haar badge, of `Weergave: Focus op de commit-kolom`).

| Toets | Actie |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Een commit kiezen |
| `Enter` / `Right` | Door naar de bestandenlijst van de gekozen commit |
| `Esc` | Het toetsenbord teruggeven aan de weergave |
| `.`, `Menu`, `Shift+F10` | Het menu van de gekozen rij |

Haar andere toetsen (`Shift+Left` / `Shift+Right` om zijwaarts te scrollen, `Tab`, `]`, `[` en `0` om de delen te schalen) staan in de gegenereerde tabel hierboven, context `commit_column`.

## Editor

**Bewerken**, of `e` in een diff of Bestandsweergave, opent de ingebouwde editor
op de kopie van het bestand in de werkmap. Zie
[Stagen en committen](staging-and-committing.md).

| Toets | Actie |
|---|---|
| *(typen)* | Invoegen bij de cursor (vervangt de selectie) |
| `Enter` | Nieuwe regel, met de inspringing van de huidige regel |
| `Tab` / `Shift+Tab` | Inspringen / uitspringen (de geselecteerde regels, als het er meer zijn) |
| pijlen, `Home` / `End` | Verplaatsen. `Home` gaat naar het eerste niet-lege teken, dan naar kolom 0. `Shift` breidt de selectie uit |
| `Ctrl+Left` / `Ctrl+Right` | Een woord verplaatsen |
| `Ctrl+Home` / `Ctrl+End` | Begin / einde van het bestand |
| `PageUp` / `PageDown` | Een pagina omhoog / omlaag |
| `Ctrl+A` | Alles selecteren |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopiëren / knippen / plakken |
| `Ctrl+U` / `Ctrl+K` | Tot het begin / einde van de regel verwijderen |
| `Ctrl+Z` | Ongedaan maken |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Opnieuw doen |
| `Ctrl+S` | Opslaan |
| `Ctrl+F` / `F3` | Zoeken. In het zoekveld: `Enter` / `Down` / `F3` volgende, `Shift+Enter` / `Up` vorige, `Esc` sluit |
| `Alt+M` | Minimap aan of uit (**Map**) |
| `Esc` | Sluiten. Met niet-opgeslagen wijzigingen vraagt het: **Opslaan** (`S` / `Enter`), **Verwerpen** (`D`), **Annuleren** (`C` / `Esc`) |

`Tab` springt in de editor in, dus het verplaatst de focus niet. Andere
`Ctrl`-toetscombinaties (`Ctrl+P`, `Ctrl+N`) werken nog.

De **codering**, de **regeleinden** en de **inspringing** in de statusbalk zijn knoppen: elk opent een
keuzelijst (pijltjes of wiel, typ om te filteren, `Enter` om te kiezen, `Esc` om te sluiten). Het palet
(`Ctrl+P`) opent dezelfde menu's: *Editor: Codering wijzigen*, *Editor: Opnieuw openen met codering*, *Editor: Converteren naar codering*, *Editor: Regeleinden converteren*, *Editor: Inspringing wijzigen*. Details in [Stagen en committen](staging-and-committing.md).

## Git-acties (`Ctrl+G`)

`Ctrl+G` wacht op nog één toets:

| Toets | Actie |
|---|---|
| `s` / `u` | Het geselecteerde bestand stagen / unstagen |
| `a` | Alle wijzigingen stagen |
| `x` | Alle wijzigingen verwerpen (vraagt eerst) |
| `c` | De commit-composer openen |
| `Enter` | De gestagede wijzigingen nu committen, met het bericht van het formulier, vanaf elke plek (zie [Stagen en committen](staging-and-committing.md#de-commit-schrijven)) |
| `m` | Alle wijzigingen stashen met een bericht, getypt in een kleine dialoog (leeg mag). Met AI aan schrijft de knop **AI-bericht** (`Ctrl+G` in de dialoog) het bericht |
| `Shift+T` | Alle wijzigingen stashen (untracked bestanden inbegrepen), zonder bericht |
| `o` | Een stash poppen: de stash-rij die in de Log is geselecteerd, of anders de laatste stash |
| `p` | Pull, zoals de knop Pull is ingesteld (`git.pull_mode`) |
| `Shift+P` | Pushen naar de standaardremote (stelt bij de eerste push de upstream in), of om de remote vragen |
| `Shift+U` | Alle tags pushen |
| `f` | Alle remotes fetchen, met prune |
| `g` | De pull request van de uitgecheckte branch in de browser openen |
| `b` / `t` | Nieuwe branch / nieuwe tag op HEAD (zie [Een branch of tag een naam geven](#een-branch-of-tag-een-naam-geven)) |
| `d` | De geselecteerde branch of tag verwijderen (vraagt eerst) |
| `e` | Het bericht van de geselecteerde commit bewerken |
| `Shift+E` / `Shift+A` | De geselecteerde commit wijzigen (alle velden) / zijn auteur (zie [Een commit wijzigen](#een-commit-wijzigen)) |
| `y` | De geselecteerde commit cherry-picken |
| `v` | De geselecteerde commit reverten |
| `Shift+R` | Reset (mixed) naar de geselecteerde commit |
| `r` | Rebase op de geselecteerde branch |
| `Shift+W` | Het Flow-menu openen (een repository met een branchingworkflow) |
| `Shift+H` | De werkbank voor geschiedenis herschrijven openen (alleen in builds met geschiedenis herschrijven) |
| `l` | Het LFS-menu openen. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS alles fetchen / alles uitchecken / pullen / alles pushen / prune. Zie [Git LFS](lfs.md) |
| `i` | ✦ AI: het commit-bericht voor de gestagede wijzigingen schrijven |
| `w` | ✦ AI: de geselecteerde commit, of de werkwijzigingen, uitleggen |
| `Shift+C` | ✦ AI: voorstellen hoe de lokale wijzigingen in commits worden gesplitst |

De AI-toetsen werken alleen als AI aan staat (zie [AI-ondersteuning](ai.md)).
`Esc` annuleert een lopend AI-verzoek.

## Rechterpaneel (lokale wijzigingen en commit-details)

Met de WIP-rij geselecteerd toont het rechterpaneel de lijsten **Unstaged** en
**Staged** en de commit-composer. Met een commit geselecteerd toont het de
commit-details en zijn bestanden.

| Toets | Actie |
|---|---|
| `j` / `k`, `Up` / `Down` | Verplaatsen. Aan het einde van de ene lijst gaat de selectie door in de andere |
| `h` / `l`, `Left` / `Right` | Naar de lijst Unstaged / Staged gaan |
| `Enter` | De actieve lijst in- of uitklappen |
| `Space` / `Insert` | Het huidige bestand aan de selectie toevoegen, of eruit halen |
| `Ctrl+A` | Alle bestanden van de lijst selecteren |
| `Ctrl+Space` | Het geselecteerde bestand stagen / unstagen, of een maprij inklappen |
| `d` | De diff van het geselecteerde bestand openen |
| `m` | `Pad` (platte lijst) of `Boom` (mappen) |
| `a` | `Alle bestanden tonen`: de hele boom, niet alleen de gewijzigde bestanden |
| `z` | Alle mappen inklappen, of ze weer uitklappen |
| `/` | De gewijzigde bestanden van een commit op naam filteren |
| `.`, `Menu`, `Shift+F10` | Het menu van het bestand |
| `c` | Focus op het samenvattingsveld van de commit-composer |
| `Shift+A` | **Vorige commit amenden** aan of uit |
| `Ctrl+N ]` / `Ctrl+N [` | Het commit-berichtblok laten groeien / krimpen |

In de commit-composer:

| Toets | Actie |
|---|---|
| `Enter` | Samenvatting: naar de beschrijving gaan. Beschrijving: nieuwe regel |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Committen (`Ctrl+G Enter` doet het vanaf elke plek) |
| `Alt+R` | Een issueverwijzing invoegen |
| `Esc` | De composer verlaten |

Het bestandsmenu bevat: Stagen of Unstagen, Wijzigingen verwerpen, Negeren,
Bestand stashen, Bestandsgeschiedenis, Bestands-blame, Openen in externe
diff-tool, Openen in externe editor, Bestand openen in standaardprogramma, Tonen
in map, Bestandspad kopiëren, Patch maken van bestandswijzigingen, Bestand
bewerken en Bestand verwijderen. Bij een bestand in conflict begint het met
**Openen in merge-editor** en de drie rijen **Oplossen met**. Met Git LFS aan
heeft het LFS-rijen. **Wijzigingen verwerpen** en **Bestand verwijderen** vragen
eerst.

## Vensterknoppen

Elke knop heeft een toetsenbordtegenhanger:

| Knop | Toets |
|---|---|
| 🐙 links in de titelbalk (Over) | `Help: Over GitPoulpi` in het palet |
| **Openen in** in de titelbalk | `Alt+O`, `Ctrl+Alt+O` |
| `×` rechts in de titelbalk (afsluiten) | `q` |
| De versie in de titelbalk (Releasenotes) | `Help: Releasenotes` in het palet |
| `⌕ Globaal zoeken` in de titelbalk, links van het `⚙` | `Ctrl+Alt+F` |
| `⚙` in de titelbalk (Instellingen) | `Ctrl+,` |
| De werkruimtechip in de tabbalk | `Alt+W` |
| `+` na het laatste tabblad | `Alt+T` |
| Het tabblad **Radar** | `Alt+R` |
| Het tabblad **Werkruimtes** | `Alt+J` |
| **Activiteit** in de statusbalk | `Ctrl+N Shift+A` |
| Werkbalk: **Ongedaan**, **Opnieuw** | `Ctrl+Z`, `Ctrl+Y` |
| Werkbalk: **Pull** / zijn `▾` | `Ctrl+G p` / `Git: Pull-opties...` |
| Werkbalk: **Push** / zijn `▾` | `Ctrl+G Shift+P` / `Git: Pushen naar remote...` |
| Werkbalk: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Werkbalk: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Werkbalk: **LFS** (als LFS aan staat) | `Ctrl+G l` |
| Werkbalk: **Flow** (als de repository een workflow heeft) | `Ctrl+G Shift+W` |
| Werkbalk: **Zoeken** | `f` |
| Werkbalk: de gekoppelde pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Werkbalk: de profiellijst (bij twee of meer profielen) | `Repository: Profiel wijzigen...` |

De werkbalk wordt beschreven in
[Rondleiding door de interface](interface-tour.md#werkbalk).

## Instellingen

`Ctrl+,`, `Ctrl+N ,`, het `⚙`-tandwiel of `App: Instellingen...` opent het scherm
Instellingen.

| Toets | Actie |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Tussen de paginalijst en de opties wisselen |
| `j` / `k`, `Up` / `Down` | De selectie verplaatsen |
| `Enter` / `Space` | Een pagina openen, een selectievakje omschakelen, een veld bewerken, een keuzelijst openen of een knop indrukken |
| `/`, `Ctrl+F` | In alle opties zoeken |
| `Alt+Left` / `Alt+Right` | Van subtabblad wisselen (Algemeen / de lijst) |
| `Esc` / `q` | De Instellingen sluiten. Terwijl een veld wordt bewerkt, annuleert `Esc` alleen de bewerking |
| `?` | De help boven het scherm Instellingen openen |

Zie [Instellingen](settings.md).

## Tekstvelden

Alle tekstvelden delen dezelfde toetsen: filters, zoekvelden, het opdrachtenpalet,
prompts, de commit-composer, het veld van de WIP-rij, dialoogvelden en velden in
de Instellingen. Terwijl een veld het toetsenbord heeft, gaan deze toetsen voor op
de paneelsneltoetsen.

| Toets | Actie |
|---|---|
| een teken | Het bij de cursor typen (vervangt de selectie) |
| `Left` / `Right` | Een teken verplaatsen (met een selectie: naar het begin / einde ervan) |
| `Ctrl+Left` / `Ctrl+Right` | Een woord verplaatsen |
| `Home` / `End` | Begin / einde van de regel |
| `Shift` + een beweging hierboven | De selectie uitbreiden of verkleinen |
| `Up` / `Down` (alleen beschrijving) | Vorige / volgende regel |
| `Ctrl+Home` / `Ctrl+End` (alleen beschrijving) | Begin / einde van de tekst |
| `Ctrl+A` | Alles selecteren |
| `Backspace` / `Delete` | Het teken voor / na de cursor verwijderen, of de selectie |
| `Ctrl+U` | Van de cursor tot het begin van de regel verwijderen (of de selectie) |
| `Ctrl+K` | Van de cursor tot het einde van de regel verwijderen (of de selectie) |
| `Ctrl+C` / `Ctrl+X` | De selectie kopiëren / knippen (niet in een wachtwoordveld) |
| plakken (het eigen plakken van de terminal) | Invoegen bij de cursor, de selectie vervangend |

In de commit-beschrijving typt `Enter` een nieuwe regel en commit `Ctrl+Enter`.
Niet alle terminals sturen `Ctrl+Enter` op dezelfde manier, dus `Alt+Enter` en
`Ctrl+J` committen ook.

Sommige consoles sturen `Ctrl+letter` als besturingsteken (`Ctrl+A` als `0x01`).
GitPoulpi begrijpt beide vormen. `AltGr`-tekens (`@`, `#`, `€` op Zwitserse of
Franse toetsenborden) worden getypt.

## Tooltips

Een tooltip verschijnt als de aanwijzer ongeveer 0,6 s op een doel blijft (een
Log-cel, een werkbalkknop, de gekoppelde pull request). Een toetsaanslag, een
klik, een scroll, een formaatwijziging, een geopende popup of menu, of de
aanwijzer die het doel verlaat verbergt hem. Daarna komt hij pas terug als de
aanwijzer weer beweegt.
