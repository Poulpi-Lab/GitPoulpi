[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · **Deutsch** · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Tastenkürzel

Diese Seite listet die **Standardtasten**. Die meisten lassen sich neu belegen. Die App
zeigt immer die geltenden Tasten: in der Befehlspalette, der Hilfe `?`, den Statuszeilen, den
Kontextmenüs und den Tooltips der Schaltflächen. Verschieben Sie eine Taste, zeigen diese
Stellen die neue. Zur Maus siehe [Maus und Gesten](mouse-and-gestures.md).

## Neu belegen

Öffnen Sie **Einstellungen > Tastatur & Maus**. Die Seite ist unter
[Einstellungen](settings.md#tastatur--maus) beschrieben. Änderungen gelten sofort und
werden in der Tabelle `[keybindings]` der `config.toml` gespeichert. Sie können diese
Tabelle auch von Hand bearbeiten:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Die Syntax, die Konfliktregeln und die Warnungen stehen in der
[Konfigurationsreferenz](config-reference.md) unter `[keybindings]`. Kurz:

- Ein Fehler wird in der Statuszeile gemeldet, und die Standardtaste bleibt.
- Zwei Aktionen auf einer Tastenfolge im selben Kontext werden abgelehnt.
- Ein Präfix braucht `Ctrl` oder `Alt`.
- Eine Bereichs-Tastenfolge, die eine globale verdeckt, gibt nur eine Warnung.

Die Tabelle unten listet jede Taste, die Sie neu belegen können. Manche Tasten sind fest und
lassen sich nicht neu belegen: die Bewegung in Listen (`j` `k` `h` `l`, die Pfeiltasten,
`PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, die Ziffern (das
Zählpräfix), die Tasten zur Textbearbeitung und die Tasten `Enter` / `Tab` / Pfeile in
Popups. Die Palette hat `App: Tastenbelegung...` und `Tastenbelegung: Alle zurücksetzen...`.

## Standard-Tastenkürzel

Jede Taste, die Sie neu belegen können, mit der ID für `[keybindings.<Kontext>]` und ihrem
Standard. Die Tabelle wird aus der App erzeugt (ein Test hält sie aktuell). Tasten nach
einem Präfix werden mit dem Präfix gezeigt (`Ctrl+G s`).

<!-- keymap:begin -->
| Kontext | Aktions-ID | Was sie tut | Standardtasten |
|---|---|---|---|
| `global` | `git_prefix` | Präfix für Git-Aktionen | `Ctrl+G` |
| `global` | `nav_prefix` | Navigationspräfix | `Ctrl+N` |
| `global` | `palette` | Befehlspalette | `Ctrl+P` |
| `global` | `settings` | Einstellungen | `Ctrl+,` |
| `global` | `home` | Start (Repository-Verwaltung) | `Ctrl+O` |
| `global` | `clone` | Repository klonen | `Ctrl+Shift+N` |
| `global` | `new_tab` | Neuer Tab | `Alt+T` |
| `global` | `workspace_switch` | Workspace wechseln | `Alt+W` |
| `global` | `radar` | Radar öffnen | `Alt+R` |
| `global` | `workspaces` | Workspaces öffnen | `Alt+J` |
| `global` | `tab_menu` | Tab-Menü | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Öffnen in (Liste) | `Alt+O` |
| `global` | `tab_open_in_default` | Im Standardeintrag öffnen | `Ctrl+Alt+O` |
| `global` | `close_tab` | Tab schließen | `Ctrl+W` |
| `global` | `next_tab` | Nächster Tab | `Ctrl+PageDown`, `Ctrl+Tab` |
| `global` | `prev_tab` | Vorheriger Tab | `Ctrl+PageUp`, `Ctrl+Shift+Tab` |
| `global` | `help` | Hilfe | `?` |
| `global` | `quit` | Beenden | `q` |
| `global` | `refresh` | Aktualisieren | `r` |
| `global` | `refresh_all` | Aktualisieren (Fetch, Pull Requests, Issues) | `F5` |
| `global` | `find_in_log` | Commit-Graph durchsuchen | `f` |
| `global` | `workspace_search` | Im Workspace suchen | `Ctrl+Alt+F` |
| `global` | `select_tab` | Tab auswählen | `t` |
| `global` | `undo` | Letzten Vorgang rückgängig machen | `Ctrl+Z` |
| `global` | `redo` | Rückgängig gemachten Vorgang wiederholen | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Rückgängig-Verlauf öffnen | `Alt+Z` |
| `global` | `commit_column` | Commit-Spalte ein- oder ausblenden | `Alt+C` |
| `global` | `undo_offer` | Branch dorthin zurücksetzen, wo die blockierte Aktion ihn gelassen hat | `Alt+Shift+Z` |
| `global` | `context_menu` | Kontextmenü | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Commit-Nachricht verfassen | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Staged Änderungen jetzt committen | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | KI: Änderungen in Commits aufteilen | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | KI: Änderungen erklären | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash mit Nachricht | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | KI: Commit-Nachricht schreiben | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Alle Änderungen stagen | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Alle Änderungen verwerfen | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Ausgewählte Datei stagen | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Ausgewählte Datei unstagen | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `pull_all_branches` | Pull aller Branches | `Ctrl+G Ctrl+P` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Verknüpften Pull Request öffnen | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push aller Tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch aller Remotes | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Neuer Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Neuer Branch | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Änderungen per Stash beiseitelegen | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Ausgewählten Commit per Cherry-Pick übernehmen | `Ctrl+G y` |
| `git_prefix` | `revert` | Ausgewählten Commit rückgängig machen (Revert) | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Commit-Nachricht bearbeiten | `Ctrl+G e` |
| `git_prefix` | `amend` | Ausgewählten Commit ändern | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Autor im ausgewählten Commit ändern | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) auf ausgewählten Commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase auf ausgewählten Branch | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Stash wiederherstellen (Pop) | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Ausgewählte Ref löschen oder Stash verwerfen | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | LFS-Menü öffnen | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Flow-Menü öffnen | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch (alle Objekte) | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Alle Dateien auschecken | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull (Objekte) | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push (alle Objekte) | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Lokale Objekte bereinigen | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Werkbank zum Umschreiben des Verlaufs öffnen | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Refs-Bereich ein- oder ausklappen | `Ctrl+N Shift+H` |
| `nav_prefix` | `collapse_status` | Status-Bereich ein- oder ausklappen | `Ctrl+N Shift+L` |
| `nav_prefix` | `settings` | Einstellungen öffnen | `Ctrl+N ,` |
| `nav_prefix` | `about` | „Über GitPoulpi“ öffnen | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Aktivitätsprotokoll öffnen | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Globales Aktivitätsprotokoll öffnen | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Hintergrundaufgaben anzeigen | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Update-Fenster anzeigen | `Ctrl+N u` |
| `nav_prefix` | `help` | Hilfe umschalten | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Tab auswählen | `Ctrl+N Tab` |
| `nav_prefix` | `change_theme` | Design wechseln | `Ctrl+N t` |
| `nav_prefix` | `change_language` | Sprache wechseln | `Ctrl+N l` |
| `nav_prefix` | `change_graph_color_theme` | Farbschema des Graphen wechseln | `Ctrl+N Shift+T` |
| `nav_prefix` | `toggle_graph_row_tints` | Zeilenfärbung des Graphen umschalten | `Ctrl+N Shift+Y` |
| `nav_prefix` | `grow_section` | Refs-Abschnitt vergrößern | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Refs-Abschnitt verkleinern | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Commit-Nachrichtenblock vergrößern | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Commit-Nachrichtenblock verkleinern | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Refs-Bereich verbreitern | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Refs-Bereich verschmälern | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Status-Bereich verbreitern | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Status-Bereich verschmälern | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: Ausstehende Ansicht zeigen | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: Zurück (oder ausstehende Ansicht verwerfen) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Geöffneten Dialog anordnen (verschieben, Größe ändern) | `Ctrl+N m` |
| `refs` | `filter` | Refs nach Namen filtern | `/` |
| `refs` | `solo` | Auswahl solo schalten | `o` |
| `refs` | `clear_solo` | Solo beenden | `Shift+O` |
| `refs` | `toggle_hierarchy` | Branch-Hierarchie umschalten | `y` |
| `refs` | `show_all` | Alle ausgeblendeten Refs zeigen | `Shift+H` |
| `refs` | `reveal` | Im Graph anzeigen | `g` |
| `refs` | `mark_drop` | Als Ablagequelle markieren | `m` |
| `refs` | `drop_marked` | Markierte Ref hier ablegen | `Shift+M` |
| `refs` | `expand_all` | Alle Ordner aufklappen | `=` |
| `refs` | `collapse_all` | Alle Ordner zuklappen | `-` |
| `refs` | `expand_below` | Ordner und seine Unterordner aufklappen | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Ordner und seine Unterordner zuklappen | `_`, `Shift+Left` |
| `refs` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `refs` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `log` | `toggle_file_mode` | Dateien: Pfad oder Baum | `m` |
| `log` | `lfs_only` | Files: LFS Only | `o` |
| `log` | `view_all_files` | Dateien: Alle Dateien ansehen | `a` |
| `log` | `compare_selection` | Ausgewählte Zeilen vergleichen | `v` |
| `log` | `compare_swap` | Vergleichsenden tauschen | `x` |
| `log` | `compare_next` | Zur nächsten verglichenen Zeile springen | `n` |
| `log` | `pick_column` | Spalte zum Verschieben oder Anpassen wählen | `Shift+C` |
| `log` | `scroll_left` | Spalten nach links scrollen | `Shift+Left` |
| `log` | `scroll_right` | Spalten nach rechts scrollen | `Shift+Right` |
| `log` | `details_scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `log` | `details_scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `log` | `copy_mermaid` | Commits als Mermaid gitGraph kopieren | `Shift+Y` |
| `log_columns` | `next` | Nächste Spalte wählen | `Tab` |
| `log_columns` | `prev` | Vorherige Spalte wählen | `Shift+BackTab` |
| `log_columns` | `move_left` | Spalte nach links verschieben | `<` |
| `log_columns` | `move_right` | Spalte nach rechts verschieben | `>` |
| `log_columns` | `wider` | Spalte verbreitern | `]` |
| `log_columns` | `narrower` | Spalte verschmälern | `[` |
| `log_columns` | `reset_width` | Spaltenbreiten zurücksetzen | `0` |
| `log_columns` | `fit` | Spalte an Inhalt anpassen | `=` |
| `log_columns` | `fit_all` | Alle Spalten an Inhalt anpassen | `+` |
| `log_columns` | `layout` | Seitlich scrollen oder an Bereich anpassen | `s` |
| `log_files` | `collapse_all` | Alle Ordner ein- oder ausklappen | `z` |
| `log_files` | `filter` | Geänderte Dateien filtern | `/` |
| `log_files` | `expand_all` | Alle Ordner aufklappen | `=` |
| `log_files` | `collapse_all_folders` | Alle Ordner zuklappen | `-` |
| `log_files` | `expand_below` | Ordner und seine Unterordner aufklappen | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Ordner und seine Unterordner zuklappen | `_`, `Shift+Left` |
| `log_files` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `log_files` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `status` | `compose` | Auf die Commit-Nachricht fokussieren | `c` |
| `status` | `toggle_file_mode` | Dateien: Pfad oder Baum | `m` |
| `status` | `lfs_only` | Files: LFS Only | `o` |
| `status` | `view_all_files` | Dateien: Alle Dateien ansehen | `a` |
| `status` | `amend` | Amend umschalten | `Shift+A` |
| `status` | `skip_hooks` | Hooks überspringen umschalten | `Shift+H` |
| `status` | `collapse_all` | Alle Ordner ein- oder ausklappen | `z` |
| `status` | `stage_toggle` | Ausgewählte Zeile stagen oder unstagen | `Ctrl+Space` |
| `status` | `open_diff` | Diff der ausgewählten Datei öffnen | `d` |
| `status` | `expand_all` | Alle Ordner aufklappen | `=` |
| `status` | `collapse_all_folders` | Alle Ordner zuklappen | `-` |
| `status` | `expand_below` | Ordner und seine Unterordner aufklappen | `*`, `Shift+Right` |
| `status` | `collapse_below` | Ordner und seine Unterordner zuklappen | `_`, `Shift+Left` |
| `status` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `status` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `home` | `favorite` | Repository mit Stern markieren oder Stern entfernen | `f`, `*` |
| `home` | `close_repo` | Zeile schließen, entfernen oder Stern entfernen | `x`, `Delete` |
| `home` | `copy_path` | Repository-Pfad kopieren | `y` |
| `home` | `open_in` | Repository im Standardwerkzeug öffnen | `o` |
| `home` | `browse` | Nach einem Repository suchen | `b` |
| `home` | `clone` | Ein Repository klonen | `c` |
| `home` | `init` | Ein Repository initialisieren | `i` |
| `home` | `collapse_all` | Alle Abschnitte einklappen | `-` |
| `home` | `expand_all` | Alle Abschnitte ausklappen | `+`, `=` |
| `home` | `search` | Repositories durchsuchen | `/` |
| `home` | `workspace_new` | Neuer Workspace | `n` |
| `home` | `workspace_edit` | Workspace bearbeiten | `e` |
| `home` | `workspace_from_tabs` | Neuer Workspace aus offenen Tabs | `a` |
| `home` | `workspace_fetch` | Fetch All im Workspace | `Shift+F` |
| `home` | `workspace_pull` | Pull All im Workspace | `Shift+P` |
| `home` | `workspace_clone_missing` | Fehlende Repositories klonen | `m` |
| `home` | `workspace_locate` | Fehlendes Repository suchen | `l` |
| `refs` | `hosting_filter` | Pull Requests / Issues: Filter wählen | `v` |
| `refs` | `hosting_edit` | Pull Request: Auf dem Host bearbeiten | `e` |
| `refs` | `new_pull_request` | Pull Requests: Einen erstellen | `n` |
| `refs` | `add_remote` | Remotes: Ein Remote hinzufügen | `+` |
| `refs` | `remotes` | Branch: Remotes bearbeiten | `u` |
| `diff` | `close` | Diff schließen | `q` |
| `diff` | `edit` | Im Arbeitsverzeichnis bearbeiten | `e` |
| `diff` | `lfs_raw` | LFS: Karte oder Rohzeiger | `r` |
| `diff` | `copy` | Auswahl oder Zeile kopieren | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Hunk stagen oder unstagen | `Space` |
| `diff` | `stage_lines` | Ausgewählte Zeilen stagen oder unstagen | `Enter` |
| `diff` | `discard_hunk` | Hunk verwerfen | `x` |
| `diff` | `discard_lines` | Ausgewählte Zeilen verwerfen | `Shift+X` |
| `diff` | `select_lines` | Zeilenauswahl beginnen oder aufgeben | `v` |
| `diff` | `next_hunk` | Nächster Hunk | `n` |
| `diff` | `prev_hunk` | Vorheriger Hunk | `p` |
| `diff` | `toggle_layout` | Layout: Vereint oder nebeneinander | `m` |
| `diff` | `toggle_hunks_only` | Nur Hunks oder ganze Datei | `Shift+H` |
| `diff` | `toggle_minimap` | Minimap umschalten | `Shift+M` |
| `diff` | `choose_encoding` | Kodierung wählen | `Shift+C` |
| `diff` | `toggle_wrap` | Zeilenumbruch umschalten | `w` |
| `diff` | `toggle_whitespace` | Reine Leerraum-Änderungen zeigen oder ausblenden | `s` |
| `diff` | `merge_editor` | Merge-Editor öffnen (Datei im Konflikt) | `Shift+E` |
| `file_insight` | `close` | Dateiansicht schließen | `q` |
| `file_insight` | `edit` | Im Arbeitsverzeichnis bearbeiten | `e` |
| `file_insight` | `copy` | Auswahl oder Zeile kopieren | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Zur letzten Zeile springen | `Shift+G` |
| `file_insight` | `open_commit` | Zum Commit der Zeile gehen | `Enter` |
| `history` | `show_in_log` | Im Log zeigen | `Enter` |
| `history` | `copy_sha` | SHA des Commits kopieren | `y` |
| `history` | `copy_summary` | Zusammenfassung kopieren | `Shift+Y` |
| `history` | `show_diff` | Diff dieses Commits zeigen | `d`, `Alt+1` |
| `history` | `open_file` | Datei bei diesem Commit öffnen | `o`, `Alt+2` |
| `history` | `blame` | Blame bei diesem Commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Mit dem Arbeitsverzeichnis vergleichen | `c` |
| `history` | `restore` | Diese Version wiederherstellen | `Shift+R` |
| `history` | `range_up` | Auswahl nach oben erweitern | `Shift+Up` |
| `history` | `range_down` | Auswahl nach unten erweitern | `Shift+Down` |
| `history` | `switch_focus` | Zwischen Liste und Diff wechseln | `Tab` |
| `history` | `widen_list` | Liste verbreitern | `Alt+Right` |
| `history` | `narrow_list` | Liste verschmälern | `Alt+Left` |
| `history` | `reset_split` | Listenbreite zurücksetzen | `Alt+0` |
| `commit_column` | `scroll_left` | Spalte nach links scrollen | `Shift+Left` |
| `commit_column` | `scroll_right` | Spalte nach rechts scrollen | `Shift+Right` |
| `commit_column` | `next_part` | Nächsten Teil wählen | `Tab` |
| `commit_column` | `move_left` | Teil nach links verschieben | `<` |
| `commit_column` | `move_right` | Teil nach rechts verschieben | `>` |
| `commit_column` | `wider_part` | Teil verbreitern | `]` |
| `commit_column` | `narrower_part` | Teil verschmälern | `[` |
| `commit_column` | `reset_widths` | Teilbreiten zurücksetzen | `0` |
| `commit_column` | `fit` | Spalte an Inhalt anpassen | `=` |
| `commit_column` | `fit_all` | Alle Spalten an Inhalt anpassen | `+` |
| `file_insight` | `blame_card` | Blame: Detailkarte ein- oder ausblenden | `i` |
| `file_insight` | `blame_next` | Blame: Nächster Hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Vorheriger Hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Nächster Hunk desselben Commits | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Vorheriger Hunk desselben Commits | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Zu Zeile springen | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Commit-SHA kopieren | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Datei-Diff des Commits zeigen | `d` |
| `file_insight` | `blame_parent` | Blame: Blame beim Eltern-Commit | `b` |
| `file_insight` | `blame_history` | Blame: Dateiverlauf öffnen | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Annotationsspalte verbreitern | `>` |
| `file_insight` | `blame_narrower` | Blame: Annotationsspalte verschmälern | `<` |
| `image` | `zoom_in` | Bild vergrößern | `+`, `=` |
| `image` | `zoom_out` | Bild verkleinern | `-` |
| `image` | `fit` | Bild an die Ansicht anpassen | `0` |
| `image` | `actual_size` | Bild in Originalgröße zeigen | `1` |
| `image` | `flip_side` | Die andere Seite zeigen (vorher / nachher) | `Tab` |
| `image` | `widen_before` | Vorher-Bereich verbreitern | `Alt+Right` |
| `image` | `widen_after` | Nachher-Bereich verbreitern | `Alt+Left` |
| `image` | `reset_split` | Vorher/Nachher-Teilung zurücksetzen | `Alt+0` |
| `image` | `edit_in` | Öffnen mit... | `Shift+O` |
| `image` | `edit_default` | In der Standard-App öffnen | `o` |
| `editor` | `save` | Speichern | `Ctrl+S` |
| `editor` | `undo` | Rückgängig | `Ctrl+Z` |
| `editor` | `redo` | Wiederholen | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Suchen | `Ctrl+F` |
| `editor` | `find_next` | Weitersuchen (Shift: zurück) | `F3` |
| `editor` | `paste` | Einfügen | `Ctrl+V` |
| `editor` | `toggle_minimap` | Minimap umschalten | `Alt+M` |
| `editor_prompt` | `save` | Ungespeicherte Änderungen: Speichern und schließen | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Ungespeicherte Änderungen: Verwerfen und schließen | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Ungespeicherte Änderungen: Weiter bearbeiten | `c`, `Shift+C` |
| `merge` | `next_conflict` | Nächster Konflikt | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Vorheriger Konflikt | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Nächste Datei im Konflikt | `Alt+PageDown` |
| `merge` | `prev_file` | Vorherige Datei im Konflikt | `Alt+PageUp` |
| `merge` | `next_pane` | Nächsten Bereich fokussieren | `F6` |
| `merge` | `prev_pane` | Vorherigen Bereich fokussieren | `Shift+F6` |
| `merge` | `toggle_panes` | Seitenbereiche (Refs und Status) zeigen oder ausblenden | `F9` |
| `merge` | `toggle_base` | Basis-Bereich zeigen oder ausblenden | `Alt+D` |
| `merge` | `toggle_full_file` | Diff der ganzen Datei umschalten | `Alt+F` |
| `merge` | `undo` | Rückgängig (Auswahl und Tippen) | `Ctrl+Z` |
| `merge` | `redo` | Wiederholen (Auswahl und Tippen) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Aktuellen Bereich (A) verbreitern | `Alt+Right` |
| `merge` | `widen_incoming` | Eingehenden Bereich (B) verbreitern | `Alt+Left` |
| `merge` | `reset_panes` | Bereichsgrößen zurücksetzen | `Alt+0` |
| `merge` | `save` | Speichern, ohne als gelöst zu markieren | `Alt+S` |
| `merge` | `save_resolve` | Speichern und als gelöst markieren | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Lösung vorschlagen (KI) | `Alt+I` |
| `merge` | `unpick` | Auswahl dieses Konflikts aufheben (ungelöst) | `Alt+U` |
| `merge` | `reset_all` | Die ganze Datei auf die ursprünglichen Konflikte zurücksetzen | `Alt+R` |
| `merge` | `focus_a` | Zum Bereich Aktuell (A) wechseln | `Alt+1` |
| `merge` | `focus_b` | Zum Bereich Eingehend (B) wechseln | `Alt+2` |
| `merge` | `focus_output` | Zum Ergebnisbereich wechseln | `Alt+3` |
| `merge` | `focus_base` | Zum Basisbereich wechseln | `Alt+4` |
| `merge_sides` | `toggle_line` | Zeile wählen oder abwählen | `Space` |
| `merge_sides` | `toggle_chunk` | Ganzen Block wählen oder abwählen | `Enter` |
| `merge_sides` | `take_a` | Nur Aktuell (A) übernehmen | `a` |
| `merge_sides` | `take_b` | Nur Eingehend (B) übernehmen | `b` |
| `merge_sides` | `take_both` | Beide übernehmen, A dann B | `c` |
| `merge_sides` | `take_both_reversed` | Beide übernehmen, B dann A | `Shift+C` |
| `merge_sides` | `clear` | Für diesen Konflikt nichts übernehmen | `x` |
| `merge_sides` | `unpick` | Auswahl dieses Konflikts aufheben (ungelöst) | `u` |
| `merge_sides` | `reset_all` | Die ganze Datei auf die ursprünglichen Konflikte zurücksetzen | `Shift+U` |
| `merge_sides` | `swap_order` | Reihenfolge der beiden Seiten tauschen | `s` |
| `merge_sides` | `next_conflict` | Nächster Konflikt | `n` |
| `merge_sides` | `prev_conflict` | Vorheriger Konflikt | `Shift+N` |
| `merge_sides` | `take_all_a` | Alles von Aktuell (A) übernehmen | `Shift+A` |
| `merge_sides` | `take_all_b` | Alles von Eingehend (B) übernehmen | `Shift+B` |
| `merge_sides` | `toggle_base` | Basis-Bereich zeigen oder ausblenden | `d` |
| `merge_sides` | `scroll_left` | Bereiche nach links scrollen | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Bereiche nach rechts scrollen | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Diff der ganzen Datei umschalten | `f` |
| `merge_sides` | `cycle_height` | Höhe des Ergebnisses durchschalten | `z` |
| `merge_card` | `take_a` | A (Aktuell) unverändert behalten | `a` |
| `merge_card` | `take_b` | B (Eingehend) unverändert übernehmen | `b` |
| `merge_card` | `take_base` | Die Basis (gemeinsamer Vorfahre) übernehmen | `o` |
| `merge_card` | `keep` | Die verbleibende Datei behalten | `k` |
| `merge_card` | `delete` | Die Löschung akzeptieren | `Shift+D` |
| `merge_card` | `external_tool` | Im externen Merge-Tool öffnen | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (zum Amend anhalten) | `e` |
| `rebase` | `autosquash` | Autosquash umschalten | `a` |
| `rebase` | `start` | Plan starten | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Plan zurücksetzen | `Ctrl+R` |
| `rebase` | `split` | Commit der Zeile aufteilen | `t` |
| `rebase` | `move_up` | Zeile nach oben | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Zeile nach unten | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Commits erstellen | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Zeilenauswahl umschalten | `v` |
| `split` | `move_up` | Commit nach oben | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Commit nach unten | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Aufteilung zurücksetzen | `Shift+R` |
| `rewrite` | `preview` | Vorschau des Umschreibens | `F5` |
| `rewrite` | `apply` | Umschreiben anwenden | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Vorschau abbrechen | `Ctrl+X` |
| `rewrite` | `add_rule` | Regel hinzufügen | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Regel ein- oder ausschalten | `Space` |
| `rewrite` | `move_up` | Regel nach oben | `Alt+Up` |
| `rewrite` | `move_down` | Regel nach unten | `Alt+Down` |
| `rewrite` | `delete_rule` | Regel löschen | `d`, `Delete` |
| `rewrite` | `restore_rule` | Gelöschte Regel wiederherstellen | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Regel duplizieren | `Ctrl+D` |
| `rewrite` | `next_tab` | Nächster Vorschau-Tab | `Alt+Right` |
| `rewrite` | `prev_tab` | Vorheriger Vorschau-Tab | `Alt+Left` |
| `rewrite` | `save_shared` | Rezept im Repository speichern | `Alt+S` |
| `rewrite` | `save_mine` | Rezept nur für mich speichern | `Alt+M` |
| `rewrite` | `load` | Rezept laden | `Alt+L` |
| `rewrite` | `name` | Rezept umbenennen | `F2` |
| `rewrite` | `protected_all` | Alle geschützten Branches einbeziehen | `Alt+A` |
| `rewrite` | `protected_none` | Alle geschützten Branches auslassen | `Alt+U` |
| `rewrite` | `check_tools` | Programme erneut prüfen | `F6` |
| `rewrite` | `analyze` | Größe des Verlaufs analysieren | `F7` |
| `rewrite` | `host_howto` | Schutz des Branches auf dem Host erklären | `Alt+H` |
| `rewrite` | `ai_rules` | Umschreib-Regeln mit KI erstellen | `Alt+I` |
| `reword` | `apply` | Neue Nachricht anwenden | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Verlauf weitergehend ändern | `Alt+H` |
| `composer` | `commit` | Commit aus dem Composer | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Issue-Referenz einfügen | `Alt+R` |
| `md_editor` | `preview` | Markdown-Editor: Schreiben und Vorschau umschalten | `Alt+P` |
| `md_editor` | `split` | Markdown-Editor: Geteilte Ansicht umschalten | `Alt+L` |
| `md_editor` | `submit` | Markdown-Editor: Text senden | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Markdown-Editor: Mit KI schreiben | `Alt+E` |
| `dialog` | `browse` | Nach einem Ordner suchen | `Ctrl+B` |
| `dialog` | `sources` | Zu den Klon-Quellen | `Alt+S` |
| `dialog` | `submit` | Formular für Pull Request absenden | `Ctrl+S` |
| `dialog` | `web` | Webformular des Anbieters öffnen | `Alt+W` |
| `folder_picker` | `choose` | Ordner wählen | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Neuer Ordner | `Alt+N` |
| `folder_picker` | `filter` | Einträge des Ordners filtern | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Nachbarordner zeigen oder ausblenden | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Versteckte Ordner zeigen oder ausblenden | `Alt+H` |
| `preferences` | `search` | Einstellungen durchsuchen | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Listeneintrag hinzufügen | `Insert` |
| `preferences` | `list_duplicate` | Ausgewählten Listeneintrag duplizieren | `Ctrl+D` |
| `preferences` | `list_remove` | Ausgewählten Listeneintrag entfernen | `Delete` |
| `preferences` | `list_move_up` | Ausgewählten Listeneintrag nach oben | `Alt+Up` |
| `preferences` | `list_move_down` | Ausgewählten Listeneintrag nach unten | `Alt+Down` |
| `preferences` | `prev_tab` | Vorheriger Tab der Einstellungsseite | `Alt+Left` |
| `preferences` | `next_tab` | Nächster Tab der Einstellungsseite | `Alt+Right` |
| `ai_popup` | `copy` | Ergebnis kopieren | `c` |
| `activity_log` | `scroll_left` | Aktivitätsprotokoll nach links scrollen | `Left` |
| `activity_log` | `scroll_right` | Aktivitätsprotokoll nach rechts scrollen | `Right` |
| `integrations` | `add` | Ein Profil hinzufügen | `a` |
| `integrations` | `edit` | Ausgewähltes Profil bearbeiten | `e` |
| `integrations` | `open_token` | Token-Seite öffnen | `o` |
| `integrations` | `remove` | Ausgewähltes Profil entfernen | `d`, `Delete` |
| `prompt` | `ai_stash_message` | KI: Stash-Nachricht erzeugen | `Ctrl+G` |
<!-- keymap:end -->

### Tasten, die das Terminal nicht senden kann, und das Kitty-Tastaturprotokoll

Ein altes Terminal kann manche Tastenfolgen nicht senden: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` und `Ctrl+<Ziffer>`. Es sendet nichts oder eine andere Taste: Zum Beispiel
kommt `Ctrl+Shift+N` als `Ctrl+N` an, `Ctrl+I` als `Tab` und `Ctrl+M` als `Enter`. Die
Einstellungsseite warnt vor einer solchen Belegung („dieses Terminal sendet nicht ...“).

Terminals mit dem **Kitty-Tastaturprotokoll** können diese Tastenfolgen senden. Beispiele:
kitty, WezTerm, Ghostty, foot, Alacritty ab 0.13 und zellij (tmux beantwortet es nicht:
siehe [Multiplexer](multiplexers.md#tmux)). GitPoulpi nutzt das Protokoll standardmäßig:

- Beim Start fragt GitPoulpi das Terminal und wartet höchstens etwa 100 ms auf die Antwort.
  Antwortet das Terminal nicht, bleiben die klassischen Tasten.
- Antwortet das Terminal, schaltet GitPoulpi das Protokoll ein. Es schaltet es wieder aus,
  wenn es beendet wird, scheitert oder abstürzt.
- Die Warnungen der Einstellungsseite folgen dem Protokoll: Ist es an, sind die obigen
  Tastenfolgen sendbar.
- **Einstellungen > Tastatur & Maus > Kitty-Tastaturprotokoll** (`ui.kitty_keyboard`) auf
  `off` ändert nie den Tastaturmodus des Terminals. Nutzen Sie es, wenn sich ein Terminal
  oder Multiplexer schlecht verhält. Ein Neustart ist nötig.

Windows-Konsolen melden die Modifikatoren selbst und brauchen das Protokoll nicht.

## Befehlspalette (`Ctrl+P`)

Die Palette listet **jeden** Befehl der App, sodass sie alles ohne Tastenkürzel erreicht. Die
vollständige Liste steht [unten](#alle-palettenbefehle).

| Taste | Aktion |
|---|---|
| `Ctrl+P` | Die Palette öffnen |
| *(tippen)* | Die Liste filtern. Ein exakter Treffer nach dem Präfix `Kategorie: ` kommt zuerst, dann ein Treffer irgendwo in der Bezeichnung, dann ein unscharfer Treffer |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Die Hervorhebung bewegen (siehe [Filterbare Popup-Listen](#filterbare-popup-listen)) |
| `Enter` oder ein Doppelklick | Den hervorgehobenen Befehl ausführen |
| `Esc` | Schließen |

`j` und `k` werden in den Filter getippt. Sie bewegen die Hervorhebung nicht.

Jede Zeile zeigt rechts ihr Tastenkürzel. Ein Befehl, der jetzt nicht passt (zum Beispiel
keine Ref ausgewählt), bleibt in der Liste, ausgegraut, und der Grund steht unter der Liste.
Bezeichnungen beginnen mit einer Kategorie, etwa `Navigation:`, `Refs:`, `Git:`, `Ansicht:`,
`Diff:` oder `App:`. Tippen Sie eine Kategorie, um ihre ganze Gruppe zu sehen.

### Eine Taste für jeden Befehl

Jede Zeile der Palette endet mit einer Tasten-Schaltfläche (`[k]`), und `Alt+K` tut dasselbe für die markierte Zeile: Es öffnet **Einstellungen > Tastatur & Maus**, gefiltert auf das Kürzel dieses Befehls, bereit zum Neubelegen. Ein Befehl ohne eigenes Kürzel bekommt hier eines: Er steht unter **Palette-Befehle**, standardmäßig ohne Taste, und eine Taste, die Sie ihm geben, führt den Befehl genauso aus wie seine Auswahl in der Palette (ein Befehl, der jetzt nicht laufen kann, nennt den Grund). Die Zeile zeigt dann Ihre Taste.

In der `config.toml` ist das die Tabelle `[keybindings.palette]`, benannt nach dem Befehl (`toggle_minimap = "Ctrl+Alt+M"`). Eine Taste, die mit einer anderen Aktion kollidiert, erscheint rot, und `c` in ihrer Zeile listet alle Aktionen auf, mit denen sie kollidiert.

Die Hinweiszeile unter der Liste nennt diese Taste (standardmäßig `Alt+K`). Es ist die Aktion `palette.configure_shortcut` und lässt sich wie jede andere neu belegen.

Die Palette und die Auswahllisten (Theme, Sprache, Tab, Workspace-Suche) lassen sich wie ein Dialog verschieben und in der Größe ändern: Ziehen Sie den oberen Rahmen, drücken Sie `Ctrl+N` und dann `m` (wie über jedem Dialog) oder drücken Sie `Alt+G`. `Alt+G` ist die Aktion `palette.arrange` („Dialog: Vordersten Dialog verschieben/Größe ändern“): ein zusätzlicher Weg, über jedem Dialog und Popup gültig, um den Anordnungsmodus für den vordersten zu starten (die Palettenzeilen `Dialog: Verschieben/Größe ändern` und `Dialog: Position und Größe zurücksetzen` wirken auf die Palette selbst, solange sie offen ist). Pfeile verschieben ihn, `Shift`+Pfeile ändern die Größe, `0` stellt ihn zurück, `Enter` oder `Esc` beenden den Modus, und das Popup bleibt offen. `ui.dialog_geometry` legt fest, ob die Position gemerkt wird.

`Ctrl+N` ist auch hier das Navigationspräfix: Es zeigt ein Badge am Filterfeld, an der ersten Zeile der Liste und am Schließkreuz, und die Taste danach drückt dieses Bedienelement, oder `m` ordnet das Popup an. Es bewegt die Hervorhebung nicht mehr nach unten, und `Ctrl+P` bewegt sie nicht nach oben: Nur die Pfeile, `PageUp` / `PageDown` und `Home` / `End` bewegen eine Liste mit Filterfeld.

### Alle Palettenbefehle

Aus der App erzeugt (ein Test hält sie aktuell). KI-Befehle erscheinen nur, solange die KI
an ist; die anderen sind ausgegraut, mit dem Grund, wenn sie nicht passen. Die Tasten sind
die Standardtasten.

<!-- palette:begin -->
| Befehl | Standardtaste |
|---|---|
| Navigation: Refs-Bereich fokussieren | `Ctrl+N <badge>` |
| Navigation: Log-Bereich fokussieren | `Ctrl+N <badge>` |
| Navigation: Status-Bereich fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs-Filter fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Lokal fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Remote fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Worktrees fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Submodule fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Stashes fokussieren | `Ctrl+N <badge>` |
| Navigation: Refs > Tags fokussieren | `Ctrl+N <badge>` |
| Seitenleiste: Abschnitte ein-/ausblenden... |  |
| Seitenleiste: Abschnitte auf Standard zurücksetzen |  |
| Seitenleiste: Diese Abschnitte für alle Repositories verwenden |  |
| Seitenleiste: Lokale Branches ein-/ausblenden |  |
| Seitenleiste: Remotes ein-/ausblenden |  |
| Seitenleiste: Worktrees ein-/ausblenden |  |
| Seitenleiste: Abschnitt Worktrees anzeigen |  |
| Seitenleiste: Stashes ein-/ausblenden |  |
| Seitenleiste: Tags ein-/ausblenden |  |
| Seitenleiste: Pull Requests ein-/ausblenden |  |
| Seitenleiste: Issues ein-/ausblenden |  |
| Seitenleiste: Submodule ein-/ausblenden |  |
| Navigation: Status > Unstaged fokussieren | `Ctrl+N <badge>` |
| Navigation: Status > Staged fokussieren | `Ctrl+N <badge>` |
| Navigation: Fokus zum nächsten Bereich wechseln | `Tab` |
| Navigation: Fokus zum vorherigen Bereich wechseln | `Shift+Tab` |
| Navigation: Commits suchen... | `f` |
| Navigation: Refs-Bereich ein-/ausklappen | `Ctrl+N H` |
| Navigation: Refs-Bereich verbreitern | `Ctrl+N Right` |
| Navigation: Refs-Bereich verschmälern | `Ctrl+N Left` |
| Navigation: Status-Bereich verbreitern | `Ctrl+N Shift+Left` |
| Navigation: Status-Bereich verschmälern | `Ctrl+N Shift+Right` |
| Navigation: Status-Bereich ein-/ausklappen | `Ctrl+N L` |
| Navigation: Refs-Abschnitt vergrößern | `Ctrl+N +` |
| Navigation: Refs-Abschnitt verkleinern | `Ctrl+N -` |
| Navigation: Größen der Refs-Abschnitte zurücksetzen |  |
| Navigation: Layout zurücksetzen |  |
| Commit-Details: Nachricht vergrößern | `Ctrl+N ]` |
| Commit-Details: Nachricht verkleinern | `Ctrl+N [` |
| Commit-Details: Nachrichtengröße zurücksetzen |  |
| Commit-Details: Nachricht nach unten scrollen |  |
| Commit-Details: Nachricht nach oben scrollen |  |
| Commit-Details: Link in Nachricht öffnen... |  |
| Dateien: Sortierung der Commit-Dateien umschalten (A-Z / Z-A) |  |
| Refs: Ausgewählte Ref auschecken | `Doppelklick` |
| Refs: Ref-Namen kopieren |  |
| Refs: Vollständige Ref kopieren (refs/heads/...) |  |
| Refs: Ziel-SHA der Ref kopieren |  |
| Refs: Ausgewählten Branch in den aktuellen mergen |  |
| Refs: Aktuellen Branch auf den ausgewählten rebasen | `Ctrl+G r` |
| Refs: Ausgewählte Ref auf dem ausgecheckten Branch ablegen... |  |
| Refs: Als Ablagequelle markieren | `m` |
| Refs: Markierte Ref hier ablegen... | `M` |
| Refs: Ausgewählten Branch umbenennen... |  |
| Refs: Ausgewählten Branch per Fast-Forward auf den aktuellen setzen |  |
| Refs: Aktuellen Branch per Fast-Forward auf den ausgewählten setzen |  |
| Refs: Worktree aus ausgewähltem Branch oder Tag erstellen... |  |
| Refs: Reflog des ausgewählten Branches... |  |
| Refs: Pull Request aus ausgewähltem Branch starten... |  |
| Refs: Worktree in Tab öffnen | `Enter / Doppelklick` |
| Refs: Worktree öffnen in... |  |
| Refs: Merge des Worktree-Branches in... |  |
| Refs: Worktree entfernen... |  |
| Refs: Worktree erstellen... | `+` |
| Refs: Worktree aus aktuellem Branch erstellen |  |
| Refs: Worktree mit Branch-Auswahl erstellen |  |
| Refs: Submodul aktualisieren (ausgewählt) |  |
| Refs: Submodul initialisieren (ausgewählt) |  |
| Refs: Submodul in Tab öffnen | `Enter / Doppelklick` |
| Git: Alle Submodule aktualisieren |  |
| Git: Worktree am ausgewählten Commit erstellen... |  |
| Git: Patch aus ausgewähltem Commit erstellen... |  |
| Git: Patch vom ausgewählten Commit bis HEAD erstellen... |  |
| Git: Patch aus Änderungen im Arbeitsverzeichnis erstellen... |  |
| Umschreiben: Alten Verlauf bereinigen... |  |
| Git: Patch anwenden... |  |
| Umschreiben: Werkbank für den Verlauf | `Ctrl+G H` |
| Umschreiben: Autor ersetzen... |  |
| Umschreiben: Autoren anonymisieren... |  |
| Umschreiben: .mailmap auf den Verlauf anwenden... |  |
| Umschreiben: Identitäten mergen (.mailmap)... |  |
| Umschreiben: Datum der Commits verschieben... |  |
| Umschreiben: Zeitzone der Commits ändern... |  |
| Umschreiben: Commits auf Uhrzeiten beschränken... |  |
| Umschreiben: Committer-Datum auf Autordatum setzen |  |
| Umschreiben: In Nachrichten suchen und ersetzen... |  |
| Umschreiben: Nachrichtenzeilen löschen... |  |
| Umschreiben: Trailer hinzufügen oder entfernen... |  |
| Umschreiben: Signaturen entfernen... |  |
| Umschreiben: Kodierung der Nachrichten korrigieren... |  |
| Umschreiben: In Dateien suchen und ersetzen... |  |
| Umschreiben: Dateien aus dem Verlauf entfernen... |  |
| Umschreiben: Große Dateien aus dem Verlauf entfernen... |  |
| Umschreiben: Pfade verschieben oder umbenennen... |  |
| Umschreiben: Dateien zu LFS migrieren... |  |
| Umschreiben: Dateien aus LFS zurückholen... |  |
| Umschreiben: Zeilenenden umwandeln... |  |
| Umschreiben: Ausführbar-Bit entfernen... |  |
| Umschreiben: Dateikodierung umwandeln... |  |
| Umschreiben: Skript ausführen... |  |
| ✦ Umschreiben: Skript mit KI schreiben... |  |
| ✦ Umschreiben: Regeln mit KI erstellen... |  |
| Umschreiben: Ab dem ausgewählten Commit bis HEAD... |  |
| Umschreiben: Ausgewählte Commits... |  |
| LFS: Für dieses Repository initialisieren |  |
| LFS: Muster verfolgen... |  |
| LFS: Muster nicht mehr verfolgen... |  |
| LFS: Objekte fetchen |  |
| LFS: Menü öffnen | `Ctrl+G l` |
| LFS: Fetch (alle Objekte) | `Ctrl+G F` |
| LFS: Alle Dateien auschecken | `Ctrl+G K` |
| LFS: Push (alle Objekte) | `Ctrl+G J` |
| LFS: Objekte pullen | `Ctrl+G L` |
| LFS: Objekte pushen |  |
| LFS: Lokale Objekte bereinigen | `Ctrl+G V` |
| LFS: Sperren anzeigen |  |
| LFS: Installationsseite öffnen |  |
| LFS: Rohzeiger im Diff umschalten | `r` |
| LFS: Ausgewählte Datei verfolgen |  |
| LFS: Endung der ausgewählten Datei verfolgen |  |
| LFS: Ausgewählte Datei sperren |  |
| LFS: Ausgewählte Datei entsperren |  |
| LFS: Ausgewählte Datei zwangsweise entsperren |  |
| Refs: Ausgewählten Branch oder Tag löschen... | `Ctrl+G d` |
| Refs: Ausgewählten Branch von seinem Remote löschen... |  |
| Refs: Ausgewählten Branch lokal und auf seinem Remote löschen... |  |
| Refs: Link zum ausgewählten Branch auf dem Remote kopieren |  |
| Git: Link zum ausgewählten Commit auf dem Remote kopieren |  |
| Git: Annotierter Tag am ausgewählten Commit... |  |
| Refs: Ausgewählten Tag an das Standard-Remote pushen |  |
| Refs: Ausgewählten Tag von seinem Remote löschen... |  |
| Git: Fetch für den ausgewählten Tag |  |
| Git: Use the Remote's Version of the Selected Tag |  |
| Git: Force Push the Selected Tag to Its Remote |  |
| Refs: Ausgewählten Tag annotieren... |  |
| Refs: Move Selected Tag to Selected Commit... |  |
| Refs: Ausblenden im Graph umschalten |  |
| Refs: Im Graph anzeigen | `g` |
| Refs: Solo für ausgewählte Ref umschalten | `o` |
| Refs: Solo beenden (alle Solo-Refs aufheben) | `O` |
| Refs: Alle ausgeblendeten Refs zeigen | `H` |
| Refs: Ausblenden für ausgewählten Ordner/Remote umschalten |  |
| Refs: Solo für ausgewählten Ordner/Remote umschalten | `o` |
| Refs: Nach Namen filtern... | `/` |
| Refs: Namensfilter löschen | `Esc` |
| Refs: Alle Ordner aufklappen | `=` |
| Refs: Alle Ordner zuklappen | `-` |
| Refs: Gewählten Ordner und seine Unterordner aufklappen | `*` |
| Refs: Gewählten Ordner und seine Unterordner zuklappen | `_` |
| View: Scroll the Pane Left | `Alt+Left` |
| View: Scroll the Pane Right | `Alt+Right` |
| Refs: Nächster Abschnitt | `l` |
| Refs: Vorheriger Abschnitt | `h` |
| Refs: Aktiven Abschnitt ein- oder ausklappen | `Enter` |
| Refs: Branch-Hierarchie umschalten | `y` |
| Git: Ausgewählte Datei stagen | `Ctrl+G s` |
| Git: Ausgewählte Datei unstagen | `Ctrl+G u` |
| Git: Alle Änderungen stagen | `Ctrl+G a` |
| Änderungen: Ausgewählte Datei verwerfen... |  |
| Änderungen: Ausgewählte Datei ignorieren |  |
| Änderungen: Endung der ausgewählten Datei ignorieren |  |
| Änderungen: Ausgewählte Datei als Binärdatei behandeln... |  |
| Änderungen: Ausgewählte Datei als Textdatei behandeln... |  |
| Änderungen: Endung der ausgewählten Datei als Binär behandeln... |  |
| Änderungen: Endung der ausgewählten Datei als Text behandeln... |  |
| Änderungen: Ausgewählte Datei stashen |  |
| Änderungen: Verlauf der ausgewählten Datei |  |
| Änderungen: Blame der ausgewählten Datei |  |
| Änderungen: Ausgewählte Datei im externen Diff-Tool öffnen |  |
| Änderungen: Ausgewählte Datei im externen Editor öffnen |  |
| Änderungen: Ausgewählte Datei im Standardprogramm öffnen |  |
| Änderungen: Ausgewählte Datei im Ordner anzeigen |  |
| Änderungen: Pfad der ausgewählten Datei kopieren |  |
| Änderungen: Patch aus ausgewählter Datei erstellen... |  |
| Änderungen: Ausgewählte Datei bearbeiten |  |
| Änderungen: Ausgewählte Datei löschen... |  |
| Änderungen: Menü des ausgewählten Ordners öffnen |  |
| Änderungen: Datei erstellen |  |
| Änderungen: Alle Änderungen verwerfen... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Staged Änderungen jetzt committen | `Ctrl+G Enter` |
| Git: WIP-Commit-Nachricht bearbeiten (im Log) | `c im Log` |
| Git: Alle Remotes abrufen (Fetch) | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (Fast-Forward, wenn möglich) |  |
| Git: Pull (nur Fast-Forward) |  |
| Git: Pull (Rebase) |  |
| Git: Pull aller Branches | `Ctrl+G Ctrl+P` |
| Git: Pull aller Branches (Fast-Forward, wenn möglich) |  |
| Git: Pull aller Branches (nur Fast-Forward) |  |
| Git: Pull aller Branches (Rebase) |  |
| Git: Pull-Optionen... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push an Remote... |  |
| Git: Push an ausgewählte Remotes... |  |
| Git: Push an alle Ziele |  |
| Git: Pull von allen Quellen |  |
| Git: Pull von einem Remote... |  |
| Git: Push an mehrere Remotes anzeigen |  |
| Git: Push an mehrere Remotes fortsetzen |  |
| Git: Push an mehrere Remotes abbrechen |  |
| Git: Push an fehlgeschlagene Remotes wiederholen |  |
| Git: Force-Push (mit Lease)... |  |
| Git: Remotes des Branches... |  |
| Remote: Remote hinzufügen... |  |
| Remote: Remote entfernen... |  |
| Remote: Remote bearbeiten... |  |
| Remote: Standard-Remote festlegen... |  |
| Remote: Remote-Namen kopieren |  |
| Remote: Link zum Remote kopieren |  |
| Remote: Remote abrufen (Fetch)... |  |
| Remote: Aktionen für ausgewähltes Remote... | `.` |
| Repository: Sparse Checkout... |  |
| Repository: SSH & Zugangsdaten... |  |
| Repository: Verbindung testen |  |
| Repository: Standard-Zugangsdaten verwenden |  |
| Flow: Menü öffnen | `Ctrl+G W` |
| Flow: Branch starten... |  |
| Flow: Neues Issue... |  |
| Flow: Host-Branch aufgreifen... |  |
| Flow: Aktuellen Branch abschließen... |  |
| Flow: Mit Eltern-Branch synchronisieren... |  |
| Flow: Hochstufen... |  |
| Flow: Eltern-Branch festlegen... |  |
| Flow: Angehaltene Aktion fortsetzen oder abbrechen... |  |
| Flow: Gruppierte Branches umschalten |  |
| Flow: Workflow-Einstellungen... |  |
| Remote: Remote im Graph ausblenden... |  |
| Remote: Remote im Graph einblenden... |  |
| Git: Neuer Branch an HEAD... | `Ctrl+G b` |
| Git: Neuer Tag an HEAD... | `Ctrl+G t` |
| Git: Alle noch nicht gepushten Tags pushen | `Ctrl+G U` |
| Git: Alle Änderungen stashen | `Ctrl+G T` |
| Git: Alle Änderungen mit Nachricht stashen... | `Ctrl+G m` |
| Git: Stash Pop | `Ctrl+G o` |
| Git: Stash Apply |  |
| Git: Stash Drop... |  |
| Git: Ausgewählten Commit per Cherry-Pick auf HEAD übernehmen | `Ctrl+G y` |
| Git: Ausgewählten Commit auschecken (losgelöster HEAD) |  |
| Git: Branch am ausgewählten Commit erstellen... |  |
| Git: Tag am ausgewählten Commit erstellen... |  |
| Commit: Nachricht kopieren |  |
| Commit: Autorennamen kopieren |  |
| Commit: Autoren-E-Mail kopieren |  |
| Commit: Datum kopieren |  |
| Commit: Kurz-SHA kopieren |  |
| Commit: Voll-SHA kopieren |  |
| Git: Ausgewählten Commit rückgängig machen (Revert)... | `Ctrl+G v` |
| Git: Commit-Nachricht bearbeiten... | `Ctrl+G e` |
| Commit: Ändern... | `Ctrl+G E` |
| Commit: Autor ändern... | `Ctrl+G A` |
| Commit: Zusammenfassung ändern... |  |
| Commit: Beschreibung ändern... |  |
| Git: Ausgewählten Commit aufteilen... |  |
| Git: Interaktiver Rebase ab ausgewähltem Commit... |  |
| Rebase: Fortsetzen |  |
| Rebase: Überspringen |  |
| Rebase: Abbrechen |  |
| git am: Fortsetzen |  |
| git am: Diesen Patch überspringen |  |
| git am: Abbrechen |  |
| Repository: Veraltete index.lock entfernen |  |
| Banner: Ausstehenden Auto-Stash poppen |  |
| Banner: Ausstehenden Auto-Stash abschließen (Ergebnis behalten) |  |
| Banner: Ausstehenden Auto-Stash zeigen |  |
| Konflikte: Vorhergesagte Konflikte zeigen |  |
| Banner: Ausstehenden Auto-Stash verwerfen... |  |
| Banner: Neu beginnen... |  |
| Git: Branch auf ausgewählten Commit zurücksetzen (soft - Index und Arbeitsverzeichnis behalten) |  |
| Git: Branch auf ausgewählten Commit zurücksetzen (mixed - Arbeitsverzeichnis behalten) | `Ctrl+G R` |
| Git: Branch auf ausgewählten Commit zurücksetzen (hard - alles verwerfen)... |  |
| Git: Konflikt mit Ours lösen (dieser Branch) |  |
| Git: Konflikt mit Theirs lösen (eingehend) |  |
| Git: Konflikt mit Basis lösen (gemeinsamer Vorfahre) |  |
| Konflikte: Merge-Editor öffnen | `Enter auf einer Datei im Konflikt` |
| Merge: Nächster Konflikt | `F7` |
| Merge: Vorheriger Konflikt | `Shift+F7` |
| Merge: Nächste Datei im Konflikt | `Alt+PageDown` |
| Merge: Vorherige Datei im Konflikt | `Alt+PageUp` |
| Merge: Block aus A übernehmen |  |
| Merge: Block aus B übernehmen |  |
| Merge: Aktuell für diesen Konflikt übernehmen | `a` |
| Merge: Eingehend für diesen Konflikt übernehmen | `b` |
| Merge: Beide übernehmen, A dann B | `c` |
| Merge: Beide übernehmen, B dann A | `C` |
| Merge: Basis für diesen Konflikt übernehmen |  |
| Merge: Diesen Konflikt leeren | `x` |
| Merge: Reihenfolge tauschen | `s` |
| Merge: Alles Aktuelle übernehmen | `A` |
| Merge: Alles Eingehende übernehmen | `B` |
| Merge: Diesen Konflikt im Ergebnis bearbeiten |  |
| Merge: Seitenbereiche zeigen oder ausblenden | `F9` |
| Merge: Bereiche nach links scrollen | `Shift+Left` |
| Merge: Bereiche nach rechts scrollen | `Shift+Right` |
| Merge: Basis-Bereich umschalten | `Alt+D` |
| Merge: Diff der ganzen Datei umschalten | `Alt+F` |
| Merge: Rückgängig | `Ctrl+Z` |
| Merge: Wiederholen | `Ctrl+Y` |
| Merge: Speichern, ohne als gelöst zu markieren | `Alt+S` |
| Merge: Speichern und als gelöst markieren | `Ctrl+S` |
| Merge: Ergebnis auf Merge-Ergebnis zurücksetzen | `Alt+R` |
| Merge: Höhe des Ergebnisses durchschalten | `z` |
| Merge: Aktuellen Bereich verbreitern | `Alt+Right` |
| Merge: Eingehenden Bereich verbreitern | `Alt+Left` |
| Merge: Bereichsgrößen zurücksetzen | `Alt+0` |
| Merge: Auswahl dieses Konflikts aufheben | `Alt+U` |
| Merge: zum Bereich Aktuell (A) | `Alt+1` |
| Merge: zum Bereich Eingehend (B) | `Alt+2` |
| Merge: zum Ergebnisbereich | `Alt+3` |
| Merge: zum Basisbereich | `Alt+4` |
| Merge: Bereich links (gleicher Konflikt) |  |
| Merge: Bereich rechts (gleicher Konflikt) |  |
| Merge: Nächsten Bereich fokussieren | `F6` |
| Merge: Vorherigen Bereich fokussieren | `Shift+F6` |
| Merge: Mit der Datei auf dem Datenträger fortfahren |  |
| Merge: Neu ab dem Merge beginnen |  |
| Merge: Lösung aufheben und erneut öffnen |  |
| Merge: Datei neu laden |  |
| Merge: Aktuelle Datei unverändert verwenden | `a` |
| Merge: Eingehende Datei unverändert verwenden | `b` |
| Merge: Basis-Datei unverändert verwenden | `o` |
| Merge: Die verbleibende Datei behalten | `k` |
| Merge: Die Löschung akzeptieren | `D` |
| Merge: Im externen Merge-Tool öffnen | `m` |
| ✦ KI: Lösung für Merge-Konflikt vorschlagen | `Alt+I` |
| Merge: Merge-Editor schließen |  |
| Git: Merge, Cherry-Pick oder Revert abbrechen |  |
| Git: Cherry-Pick oder Revert überspringen |  |
| Repository: Öffnen oder wechseln (Repository-Verwaltung)... | `Ctrl+O` |
| Repository: Nach einem Repository suchen... | `b (Verwaltungs-Tab)` |
| Repository: Neu erstellen (Init)... | `i (Verwaltungs-Tab)` |
| Repositories: Ausgewähltes öffnen | `Enter` |
| Repositories: Ausgewähltes zu Favoriten hinzufügen oder daraus entfernen | `f` |
| Repositories: Ausgewähltes schließen oder entfernen | `x` |
| Repositories: Menü des ausgewählten Repositorys... | `.` |
| Repositories: Ausgewählten Pfad kopieren | `y` |
| Repositories: Ausgewähltes im Standardwerkzeug öffnen | `o` |
| Repositories: Suchen | `/ / Ctrl+F` |
| Repositories: Alle Abschnitte einklappen | `-` |
| Repositories: Alle Abschnitte ausklappen | `+` |
| Repositories: Alle Tabs schließen... |  |
| Repositories: Ausgewähltes aus der Liste entfernen |  |
| Repositories: Alle zuletzt verwendeten entfernen |  |
| Workspace: Neu... | `n` |
| Workspace: Neu aus offenen Tabs... | `a` |
| Workspace: Wechseln... | `Alt+W` |
| Workspace: Bearbeiten... | `e` |
| Workspace: Schließen (kein Workspace) |  |
| Workspace: Verwalten... | `Alt+J` |
| Workspace: Tab schließen |  |
| Workspace: Fehlendes Repository suchen... | `l` |
| Workspace: Löschen... |  |
| Workspace: Fetch All | `F` |
| Workspace: Pull All | `P` |
| Workspace: Fehlende klonen | `m` |
| Workspace: Tab zum Workspace hinzufügen |  |
| Workspace: Tab aus dem Workspace entfernen |  |
| Radar: Öffnen | `Alt+R` |
| Radar: Aktualisieren | `F5` |
| Radar: Suchen | `/` |
| Radar: Im Browser öffnen | `o` |
| Radar: Repository öffnen | `Enter` |
| Radar: Branch auschecken | `c` |
| Radar: Worktree erstellen... | `t` |
| Radar: Im Repository zeigen | `s` |
| Radar: Arbeit am Issue beginnen... | `n` |
| Radar: Anheften / Anheften aufheben | `p` |
| Radar: Zurückstellen... | `z` |
| Radar: Anfrage prüfen... | `e` |
| Radar: Issue-Unterhaltung öffnen | `i` |
| Radar: Über die API prüfen |  |
| Radar: Klonen und prüfen... |  |
| Radar: Zugewiesene bearbeiten... |  |
| Radar: Reviewer bearbeiten... |  |
| Radar: Labels bearbeiten... |  |
| Radar: Meilenstein festlegen... |  |
| Radar: Umfang wählen | `w` |
| Radar: Ansicht wählen | `v` |
| Radar: Filter wählen | `f` |
| Radar: Nicht geklonte Anfragen zeigen | `a` |
| Radar: Entwürfe zeigen | `d` |
| Radar: Kontoweite Prüfanfragen aktivieren... | `T` |
| Radar: Zeilenmenü... | `.` |
| Radar: Spalte wählen | `C` |
| Radar: Spalte verbreitern | `]` |
| Radar: Spalte verschmälern | `[` |
| Radar: Spaltenbreiten zurücksetzen | `0` |
| Radar: Spalte an Inhalt anpassen | `=` |
| Radar: Alle Spalten an Inhalt anpassen | `+` |
| Radar: Zurück zum Home-Tab | `Esc` |
| Radar: Tab schließen |  |
| Tab: Neuer Tab | `Alt+T` |
| Tab: Neu... |  |
| Tab: Aktionsmenü... | `Ctrl+T` |
| Tab: Andere Tabs schließen... |  |
| Tab: Tabs rechts schließen... |  |
| Tab: Zu Favoriten hinzufügen oder daraus entfernen |  |
| Tab: Repository-Pfad kopieren |  |
| Tab: Im Dateimanager öffnen |  |
| Tab: Im Terminal öffnen |  |
| Tab: Öffnen in... | `Alt+O` |
| Tab: Terminal Here |  |
| ✦ Tab: Open AI Agent Here |  |
| Tab: Im Standardeintrag öffnen | `Ctrl+Alt+O` |
| Repository: Profil wechseln... |  |
| Hilfe: Über GitPoulpi | `Ctrl+N a` |
| Hilfe: Lizenzen und Danksagungen |  |
| Ansicht: Aktivitätsprotokoll | `Ctrl+N A` |
| Ansicht: Globales Aktivitätsprotokoll | `Ctrl+N G` |
| Hilfe: Versionshinweise |  |
| Updates: Update-Fenster anzeigen | `Ctrl+N u` |
| Updates: Sicherheitshinweis öffnen |  |
| Updates: Update-Befehl kopieren |  |
| Updates: Update-Befehl in einem Terminal ausführen |  |
| Updates: Jetzt nach Updates suchen |  |
| Updates: Herunterladen |  |
| Updates: Jetzt aktualisieren |  |
| Updates: Für Update neu starten |  |
| Updates: Zur vorherigen Version zurückkehren |  |
| Updates: Diese Version überspringen |  |
| Versionshinweise: Tab schließen |  |
| Aufgaben: Hintergrundaufgaben anzeigen | `Ctrl+N J` |
| Aufgaben: alles abbrechen, was sich abbrechen lässt |  |
| Ansicht: Diagnose kopieren |  |
| Aktivitätsprotokoll: Stufe ein- oder ausblenden: Fehler |  |
| Aktivitätsprotokoll: Stufe ein- oder ausblenden: Warnung |  |
| Aktivitätsprotokoll: Stufe ein- oder ausblenden: Info |  |
| Aktivitätsprotokoll: Stufe ein- oder ausblenden: Debug |  |
| Aktivitätsprotokoll: Stufe ein- oder ausblenden: Trace |  |
| Aktivitätsprotokoll: alle Stufen anzeigen |  |
| Aktivitätsprotokoll: keine Stufe anzeigen |  |
| Aktivitätsprotokoll: Anwendungsprotokoll leeren |  |
| Aktivitätsprotokoll: Bereich wählen |  |
| Aktivitätsprotokoll: Git-Vorgänge |  |
| Aktivitätsprotokoll: Anwendungsprotokoll |  |
| Aktivitätsprotokoll: Vorgang dieser Zeile anzeigen |  |
| Aktivitätsprotokoll: Nächster fehlgeschlagener Vorgang |  |
| Aktivitätsprotokoll: Vorheriger fehlgeschlagener Vorgang |  |
| Aktivitätsprotokoll: nach links scrollen | `Left` |
| Aktivitätsprotokoll: nach rechts scrollen | `Right` |
| Aktivitätsprotokoll: Letzten Fehler kopieren |  |
| Aktivitätsprotokoll: Vorgeschlagene Aktion ausführen |  |
| Aktivitätsprotokoll: Letzten Fehler öffnen |  |
| Tab: Beenden und cd hierher |  |
| Repository: Klonen... | `Ctrl+Shift+N` |
| Repository: Nächster Tab | `Ctrl+PageDown / Ctrl+Tab` |
| Repository: Vorheriger Tab | `Ctrl+PageUp / Ctrl+Shift+Tab` |
| Tab: Nach links verschieben |  |
| Tab: Nach rechts verschieben |  |
| Repository: Tab schließen | `Ctrl+W` |
| Repository: Tab auswählen... | `t` |
| Suche: Workspace... | `Ctrl+Alt+F` |
| Bearbeiten: Rückgängig | `Ctrl+Z` |
| Bearbeiten: Wiederholen | `Ctrl+Y` |
| Bearbeiten: Blockierten Rückgängig-Eintrag vergessen |  |
| Bearbeiten: Rückgängig-Verlauf... | `Alt+Z` |
| Bearbeiten: Reflog... |  |
| Bearbeiten: Verlorene Commits wiederfinden... |  |
| Bearbeiten: Branch dorthin zurücksetzen, wo die blockierte Aktion ihn gelassen hat | `Alt+Shift+Z` |
| Edit: Undo Whole Sequence |  |
| Review: Übersicht zeigen |  |
| Review: Dateien zeigen |  |
| Review: Commits zeigen |  |
| Review: Zeile kommentieren... | `c` |
| Review: Unterhaltung kommentieren... | `C` |
| Review: Auf Thread antworten... | `r` |
| Review: Thread auflösen oder wieder öffnen | `x` |
| Review: Zurück zum Kommentar in der Übersicht | `Esc` |
| Review: Thread ein- oder ausklappen | `z` |
| Review: Ausstehenden Kommentar bearbeiten... | `e` |
| Review: Ausstehenden Kommentar löschen | `d` |
| Review: Datei als angesehen markieren | `v` |
| Review: Nächste Datei | `]` |
| Review: Vorherige Datei | `[` |
| Review: Review abschließen... | `f` |
| Review: Ausstehendes Review verwerfen... |  |
| Review: Aktualisieren | `F5` |
| Review: Im Browser öffnen | `o` |
| Review: Branch auschecken | `b` |
| Review: Schließen | `Esc` |
| Review: Zugewiesene bearbeiten... | `a` |
| Review: Reviewer bearbeiten... | `p` |
| Review: Labels bearbeiten... | `l` |
| Review: Meilenstein festlegen... | `m` |
| Review: Patch Set wählen... | `P` |
| Review: Patch Sets vergleichen... | `D` |
| Review: Change einreichen... | `U` |
| Review: Change aufgeben... | `Q` |
| Review: Layout vereint oder nebeneinander | `V` |
| Review: Ganze Datei oder nur Hunks | `H` |
| Review: Reine Leerraum-Änderungen zeigen oder ausblenden | `S` |
| Review: Zeilenumbruch umschalten | `W` |
| Review: Nächster Hunk | `J` |
| Review: Vorheriger Hunk | `K` |
| Review: Kodierung wählen... | `E` |
| Review: Minimap umschalten | `M` |
| Review: Diff nach links scrollen | `Left` |
| Review: Diff nach rechts scrollen | `Right` |
| Review: Markierten Text oder Zeile kopieren | `Ctrl+C` |
| Board: Karte verschieben nach... | `m` |
| Board: Karte nach links verschieben |  |
| Board: Karte nach rechts verschieben |  |
| Board: Karte öffnen | `Enter` |
| Board: Zugewiesene bearbeiten... |  |
| Board: Labels bearbeiten... |  |
| Board: Meilenstein festlegen... |  |
| Board: Neues Issue in dieser Spalte | `n` |
| Board: Nach Zugewiesenen filtern... | `a` |
| Board: Nach Label filtern... | `l` |
| Board: Nach Meilenstein filtern... | `M` |
| Board: Nach Text filtern | `/` |
| Board: Filter zurücksetzen | `x` |
| Board: Spalten-Labels festlegen... | `c` |
| Board: Aktualisieren | `r` |
| Board: Schließen | `Esc` |
| Unterhaltung: Zurück |  |
| Unterhaltung: Aktualisieren | `F5` |
| Unterhaltung: Ältere Einträge laden | `m` |
| Unterhaltung: Vorheriger Eintrag | `Up` |
| Unterhaltung: Nächster Eintrag | `Down` |
| Unterhaltung: Nächster ungelesener Eintrag | `n` |
| Unterhaltung: Auf Eintrag antworten | `r` |
| Unterhaltung: Eintrag zitieren | `Q` |
| Unterhaltung: Neuer Kommentar | `c` |
| Unterhaltung: Links des Eintrags... | `l` |
| Unterhaltung: Diagramm des Eintrags öffnen | `g` |
| Unterhaltung: Code des Threads öffnen | `G` |
| Unterhaltung: Link des Eintrags kopieren | `y` |
| Conversation: Copy Text | `Ctrl+C` |
| Unterhaltung: Im Browser öffnen | `o` |
| Unterhaltung: Kommentar bearbeiten | `e` |
| Unterhaltung: Kommentar löschen | `d` |
| Unterhaltung: Ereignisse zeigen oder ausblenden | `v` |
| Unterhaltung: Schreiben und Vorschau umschalten |  |
| Unterhaltung: Geteilte Ansicht umschalten |  |
| Unterhaltung: Kommentar senden |  |
| Unterhaltung: Interne Notiz umschalten |  |
| Unterhaltung: Schließen | `Esc` |
| Markdown-Editor: Mit KI schreiben |  |
| Markdown-Editor: KI-Kontext erneuern |  |
| Hosting: Issue-Unterhaltung öffnen... |  |
| Review: ✦ Review entwerfen... | `s` |
| Review: Vorschlag übernehmen | `y` |
| Review: Vorschlag bearbeiten und übernehmen... | `w` |
| Review: Vorschlag ablehnen | `n` |
| Review: Alle Vorschläge übernehmen | `A` |
| Review: Alle Vorschläge ablehnen | `N` |
| Review: KI-Zusammenfassung in den Review-Text übernehmen | `i` |
| Review: ✦ Entwurf verwenden... | `u` |
| Review: Text des ✦ Entwurfs bearbeiten... | `t` |
| Review: ✦ Entwurf verwerfen | `N` |
| Rückgängig-Verlauf: Bis hierher rückgängig... | `u` |
| Rückgängig-Verlauf: Bis hierher wiederholen... | `y` |
| Undo History: Undo Whole Sequence | `x` |
| Rückgängig-Verlauf: Nur diese wiederherstellen... | `r` |
| Rückgängig-Verlauf: Branch dorthin zurücksetzen, wo er war... | `o` |
| Rückgängig-Verlauf: Aktualisieren |  |
| Rückgängig-Verlauf: Schließen | `Esc` |
| Reflog: Im Log zeigen | `l` |
| Reflog: Branch hier erstellen... | `b` |
| Reflog: Tag hier erstellen... | `t` |
| Reflog: Änderungen auf Arbeitsverzeichnis anwenden | `w` |
| Reflog: Änderungen als Stash sichern | `s` |
| Reflog: Auf diese Position zurücksetzen... | `r` |
| Reflog: Mehr laden | `m` |
| Reflog: Aktualisieren |  |
| Verlorene Arbeit: Branch hier erstellen... | `b` |
| Verlorene Arbeit: Tag hier erstellen... | `t` |
| Verlorene Arbeit: Cherry-Pick | `c` |
| Verlorene Arbeit: Änderungen auf Arbeitsverzeichnis anwenden | `w` |
| Verlorene Arbeit: Änderungen als Stash sichern | `s` |
| Verlorene Arbeit: Erneut suchen |  |
| Reflog / Verlorene Arbeit: Nächster Bereich | `Tab` |
| Reflog / Verlorene Arbeit: Dateiliste als Pfade oder Baum | `v` |
| Reflog / Verlorene Arbeit: Bereichsgrößen zurücksetzen |  |
| Dialog: Verschieben/Größe ändern | `Ctrl+N m` |
| Dialog: Position und Größe zurücksetzen |  |
| Ansicht: Aktualisieren | `r` |
| Ansicht: Aktualisieren (Fetch, Pull Requests, Issues) | `F5` |
| Ansicht: Hilfe-Überlagerung umschalten | `?` |
| Ansicht: Ref-Spalte umschalten |  |
| Ansicht: Graph-Spalte umschalten |  |
| Ansicht: Nachrichtenspalte umschalten |  |
| Ansicht: Autorenspalte umschalten |  |
| Ansicht: Datumsspalte umschalten |  |
| Ansicht: Änderungsspalte umschalten |  |
| Ansicht: SHA-Spalte umschalten |  |
| Ansicht: Log-Spalte: Spalte zum Verschieben oder Anpassen wählen | `C` |
| Ansicht: Log-Spalte: Nächste Spalte wählen | `Tab` |
| Ansicht: Log-Spalte: Vorherige Spalte wählen | `Shift+BackTab` |
| Ansicht: Log-Spalte: Spalte nach links verschieben | `<` |
| Ansicht: Log-Spalte: Spalte nach rechts verschieben | `>` |
| Ansicht: Log-Spalte: Spalte verbreitern | `]` |
| Ansicht: Log-Spalte: Spalte verschmälern | `[` |
| Ansicht: Log-Spalte: Spaltenbreiten zurücksetzen | `0` |
| Ansicht: Log-Spalte: Spalte an Inhalt anpassen | `=` |
| Ansicht: Log-Spalte: Alle Spalten an Inhalt anpassen | `+` |
| Ansicht: Log: Spalten nach links scrollen | `Shift+Left` |
| Ansicht: Log: Spalten nach rechts scrollen | `Shift+Right` |
| Ansicht: Log: Seitliches Scrollen der Spalten umschalten | `s` |
| Ansicht: Log: Anzuzeigende Spalten wählen |  |
| Diff: Diff der ausgewählten Datei öffnen | `Enter` |
| Diff: Diff-Ansicht schließen | `Esc` |
| Diff: Nächster Hunk | `n` |
| Diff: Vorheriger Hunk | `p` |
| Diff: Hunk unter dem Cursor stagen | `Space` |
| Diff: Hunk unter dem Cursor unstagen | `Space` |
| Diff: Zeilen auswählen (oder Auswahl aufheben) | `v` |
| Diff: Ausgewählte Zeilen stagen | `Enter` |
| Diff: Ausgewählte Zeilen unstagen | `Enter` |
| Diff: Hunk unter dem Cursor verwerfen... | `x` |
| Diff: Ausgewählte Zeilen verwerfen... | `X` |
| Kopieren: Ausgewählten Text (zuerst über den Text ziehen) | `Ctrl+C` |
| Kopieren: Zeile unter dem Cursor (oder die ausgewählten Zeilen) | `y` |
| Diff: Hunk unter dem Cursor kopieren |  |
| Diff: Layout Inline (+/- in einer Spalte) | `m` |
| Diff: Layout Split (vorher und nachher) | `m` |
| Diff: Nur Hunks umschalten (ganze Datei / nur die Änderungen) | `H` |
| Dateien: Pfad/Baum umschalten | `m` |
| Dateien: Alle Dateien ansehen umschalten | `a` |
| Files: Toggle LFS Only | `o` |
| Commit: Hooks überspringen umschalten | `H` |
| Dateien: Menü der Zeile oder Auswahl öffnen | `.` |
| Dateien: Alle Ordner aufklappen | `=` |
| Dateien: Alle Ordner zuklappen | `-` |
| Dateien: Gewählten Ordner und seine Unterordner aufklappen | `*` |
| Dateien: Gewählten Ordner und seine Unterordner zuklappen | `_` |
| Dateien: Alles auswählen |  |
| Dateien: Auswahl aufheben |  |
| Dateien: Auswahl der aktuellen Zeile umschalten |  |
| Log: Als Mermaid gitGraph kopieren... | `Y` |
| Log: Auswahl vergleichen | `v` |
| Log: Vergleichsenden tauschen | `x` |
| Log: Zur nächsten verglichenen Zeile | `n` |
| Diff: Zeilenumbruch umschalten | `w` |
| Diff: Kodierung wählen... | `C` |
| Ansicht: Minimap umschalten | `M` |
| Diff: Toggle Changed Words Highlight |  |
| Ansicht: Commit-Spalte umschalten | `Alt+C` |
| Ansicht: Commit-Spalte: ausblenden |  |
| Ansicht: Commit-Spalte: Dateiverlauf |  |
| Ansicht: Commit-Spalte: alle Commits |  |
| Ansicht: Commit-Spalte fokussieren |  |
| Ansicht: Commit-Spalte verbreitern |  |
| Ansicht: Commit-Spalte verschmälern |  |
| Ansicht: Commit-Spalte: nach links scrollen | `Shift+Left` |
| Ansicht: Commit-Spalte: nach rechts scrollen | `Shift+Right` |
| Ansicht: Commit-Spalte: nächsten Teil wählen | `Tab` |
| Ansicht: Commit-Spalte: Teil nach links verschieben | `<` |
| Ansicht: Commit-Spalte: Teil nach rechts verschieben | `>` |
| Ansicht: Commit-Spalte: Teil verbreitern | `]` |
| Ansicht: Commit-Spalte: Teil verschmälern | `[` |
| Ansicht: Commit-Spalte: Teilbreiten zurücksetzen | `0` |
| Ansicht: Commit-Spalte: Spalte an Inhalt anpassen | `=` |
| Ansicht: Commit-Spalte: Alle Spalten an Inhalt anpassen | `+` |
| Ansicht: Commit-Spalte: Menü der gewählten Zeile | `.` |
| Ansicht: Commit-Spalte: Spalten wählen |  |
| Ansicht: Commit-Spalte: Refs umschalten |  |
| Ansicht: Commit-Spalte: Graph umschalten |  |
| Ansicht: Commit-Spalte: Autor umschalten |  |
| Ansicht: Commit-Spalte: Datum umschalten |  |
| Ansicht: Commit-Spalte: Änder. umschalten |  |
| Ansicht: Commit-Spalte: SHA umschalten |  |
| Ansicht: Commit-Spalte: gewählte Zeile im Log zeigen |  |
| Ansicht: Commit-Spalte: Änderungen der Datei an der gewählten Zeile zeigen |  |
| Ansicht: Commit-Spalte: Datei so zeigen, wie sie an der gewählten Zeile war |  |
| Ansicht: Commit-Spalte: Blame der Datei an der gewählten Zeile zeigen |  |
| Editor: Auswahl folgen umschalten |  |
| Bild: Vergrößern | `+` |
| Bild: Verkleinern | `-` |
| Bild: An Ansicht anpassen | `0` |
| Bild: Originalgröße (1:1) | `1` |
| Bild: Vorher / Nachher umschalten | `Tab` |
| Bild: Vorher-Bereich verbreitern | `Alt+Right` |
| Bild: Nachher-Bereich verbreitern | `Alt+Left` |
| Bild: Bereichsgrößen zurücksetzen | `Alt+0` |
| Bild: Bearbeiten in... | `O` |
| Bild: Im Standardeditor bearbeiten | `o` |
| Dateiverlauf: Im Log zeigen | `Enter` |
| Dateiverlauf: SHA des Commits kopieren | `y` |
| Dateiverlauf: Zusammenfassung kopieren | `Y` |
| Dateiverlauf: Diff dieses Commits zeigen | `d` |
| Dateiverlauf: Datei bei diesem Commit öffnen | `o` |
| Dateiverlauf: Blame bei diesem Commit | `b` |
| Dateiverlauf: Mit dem Arbeitsverzeichnis vergleichen | `c` |
| Dateiverlauf: Diese Version wiederherstellen | `R` |
| Dateiverlauf: Auswahl nach oben erweitern | `Shift+Up` |
| Dateiverlauf: Auswahl nach unten erweitern | `Shift+Down` |
| Dateiverlauf: Zwischen Liste und Diff wechseln | `Tab` |
| Dateiverlauf: Liste verbreitern | `Alt+Right` |
| Dateiverlauf: Liste verschmälern | `Alt+Left` |
| Dateiverlauf: Listenbreite zurücksetzen | `Alt+0` |
| Datei: Öffnen mit... | `O` |
| Datei: In Standardanwendung öffnen | `o` |
| Diff: Reine Leerraum-Änderungen umschalten | `s` |
| ✦ KI: Commit-Nachricht erzeugen | `Ctrl+G i` |
| ✦ KI: Stash-Nachricht erzeugen |  |
| ✦ KI: Prompt-Variable einfügen |  |
| ✦ KI: Profil bearbeiten... |  |
| ✦ KI: Agent öffnen... |  |
| ✦ KI: Ausgewählten Commit erklären | `Ctrl+G w` |
| ✦ KI: Ausgewählten Commit aufteilen... |  |
| ✦ KI: Änderungen im Arbeitsverzeichnis erklären | `Ctrl+G w` |
| ✦ KI: Commits aus Änderungen im Arbeitsverzeichnis zusammenstellen... | `Ctrl+G C` |
| Sensible Dateien: In Datei anzeigen | `S` |
| Composer: Datei in Commit verschieben... | `m` |
| Diff: Blame |  |
| Blame: Commit im Log zeigen | `Enter` |
| Blame: Commit-SHA kopieren | `Y` |
| Blame: Datei-Diff des Commits zeigen | `d` |
| Blame: Blame beim Eltern-Commit | `b` |
| Blame: Dateiverlauf öffnen | `H` |
| Blame: Detailkarte ein- oder ausblenden | `i` |
| Blame: Nächster Hunk | `n` |
| Blame: Vorheriger Hunk | `p` |
| Blame: Nächster Hunk desselben Commits | `N` |
| Blame: Vorheriger Hunk desselben Commits | `P` |
| Blame: Zu Zeile springen | `g` |
| Blame: Annotationsspalte verbreitern | `>` |
| Blame: Annotationsspalte verschmälern | `<` |
| Diff: Dateiverlauf |  |
| Diff: Dateiansicht |  |
| Datei: Im Arbeitsverzeichnis bearbeiten | `e` |
| Editor: Kodierung ändern |  |
| Editor: Mit Kodierung erneut öffnen |  |
| Editor: In Kodierung umwandeln |  |
| Editor: Zeilenumbrüche umwandeln |  |
| Editor: Einrückung ändern |  |
| App: Einstellungen... | `Ctrl+, / Ctrl+N ,` |
| ✦ Sequence: Create with AI... |  |
| App: Einstellungen durchsuchen... | `/ in den Einstellungen` |
| App: Tastenbelegung... |  |
| Agent: Ausstehende Ansicht zeigen | `Ctrl+N Enter` |
| Agent: Ausstehende Ansicht verwerfen |  |
| Agent: Zurück | `Ctrl+N Backspace` |
| Sequence: Cancel |  |
| Sequence: Continue |  |
| Sequence: Review This Repository's Sequences |  |
| Sequence: Trust This Repository's Sequences |  |
| Sequence: Ignore This Repository's Sequences |  |
| Tastenbelegung: Alle zurücksetzen... |  |
| Integrationen: Verknüpften Pull Request öffnen | `Ctrl+G g` |
| Integrationen: Verknüpften Pull Request auf dem Host bearbeiten | `e (auf einer Zeile)` |
| Integrationen: URL des verknüpften Pull Request kopieren |  |
| Integrationen: Pull Requests und Issues aktualisieren |  |
| Integrationen: Pull Request auschecken... |  |
| Integrationen: Commit-Referenz öffnen (#Issue / !Request)... |  |
| Integrationen: Issue öffnen... |  |
| Hosting: Anfrage prüfen... |  |
| Hosting: Zum Review senden (Push)... |  |
| Hosting: Zugewiesene bearbeiten... |  |
| Hosting: Reviewer bearbeiten... |  |
| Hosting: Labels bearbeiten... |  |
| Hosting: Meilenstein festlegen... |  |
| Lokale Issues: Issue schließen... |  |
| Lokale Issues: Issue wieder öffnen... |  |
| Lokale Issues: In Obsidian öffnen... |  |
| Issues: Board öffnen |  |
| Lokale Issues: Obsidian Base erstellen |  |
| Pull Requests: Pull Request erstellen... | `n` |
| Hosting: Neues Issue... | `n` |
| Pull Requests: Suchen | `Enter auf der Suchzeile` |
| Pull Requests: Filter wählen (Offen / Gemergt / Alle)... | `v` |
| Pull Requests: Anbieter wählen... | `Enter auf der Anbieterzeile` |
| Pull Requests: Repository wählen... | `Enter auf der Repository-Zeile` |
| Pull Requests: Quelle trennen (keine) |  |
| Issues: Anbieter wählen... | `Enter auf der Anbieterzeile` |
| Issues: Quelle trennen (keine) |  |
| Integrationen: Profil trennen (automatisch) |  |
| Issues: Repository wählen... | `Enter auf der Repository-Zeile` |
| Issues: Suchen | `Enter auf der Suchzeile` |
| Issues: Filter wählen... | `v` |
| Issues: Abfrage / Listen bearbeiten... |  |
| Commit: Issue-Referenz einfügen... |  |
| Integrationen: Profil für dieses Repository verwenden... |  |
| Integrationen: Profil dieses Repositorys verwalten |  |
| Integrationen: Integration verbinden... |  |
| Einstellungen: Radar-Optionen eines Profils... |  |
| Integrationen: Sicherheitsprüfung ansehen |  |
| App: Beenden | `q` |
| App: Design... |  |
| Konflikte: Basis-Branch-Listen bearbeiten... |  |
| Konflikte: Basis-Branches dieses Repositorys... |  |
| App: Design wechseln... | `Ctrl+N t` |
| App: Sprache wechseln... | `Ctrl+N l` |
| Starter: Verknüpfung erstellen |  |
| Starter: Desktop-Verknüpfung erstellen |  |
| Starter: Verknüpfungen entfernen |  |
| Starter: Befehlsalias erstellen |  |
| Starter: Befehlsalias entfernen |  |
| MCP: Freien Port vorschlagen |  |
| Starter: Zum PATH hinzufügen |  |
| Starter: Aus dem PATH entfernen |  |
| Starter: Menü „In GitPoulpi öffnen“ installieren |  |
| Starter: Menü „In GitPoulpi öffnen“ entfernen |  |
| Starter: Primäre Installation werden |  |
| Starter: Alte Starter entfernen |  |
| Starter: Verschwundene Installationen vergessen |  |
| App: Farbschema des Graphen wechseln... | `Ctrl+N T` |
| App: Zeilenfärbung des Graphen umschalten | `Ctrl+N Y` |
<!-- palette:end -->

## Sich bewegen

| Taste | Aktion |
|---|---|
| `Tab` / `Shift+Tab` | Den nächsten / vorherigen Bereich fokussieren |
| `Ctrl+N`, dann ein Badge | Diesen Teil des Fensters fokussieren oder das Bedienelement des vordersten Dialogs oder Popups drücken (siehe [Navigationspräfix](#navigationspräfix-ctrln)) |
| `j` / `k`, `Up` / `Down` | Die Auswahl bewegen, in jedem Bereich |
| `PageUp` / `PageDown`, `Home` / `End` | Eine Seite weiter; zur ersten / letzten Zeile |
| eine Zahl, dann `j` / `k` / `Up` / `Down` | Zählpräfix: `4j` bewegt vier Zeilen. Mit `PageUp` / `PageDown` ist die Zahl eine Seitenzahl. Die Statuszeile zeigt die ausstehende Zahl (`4...`). `Esc` oder jede andere Taste löscht sie. |
| `h` / `l`, `Left` / `Right` | Refs-Bereich: Abschnitt wechseln. Log: die Liste der geänderten Dateien eines Commits verlassen (`h`) oder betreten (`l`) |
| `?` | Die Hilfe öffnen oder schließen (siehe [Hilfe in der App](#hilfe-in-der-app-)) |
| `r` | Aktualisieren |
| `F5` | Jedes Remote fetchen, die Pull Requests und Issues neu laden und aktualisieren |
| `q` | Beenden |
| `Ctrl+Z` | Die letzte Operation **rückgängig machen**. Siehe [Rückgängig und Wiederholen](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Wiederholen**. `Ctrl+Shift+Z` braucht das Kitty-Tastaturprotokoll; `Ctrl+Y` funktioniert immer |
| `.`, `Menu`, `Shift+F10` | Das Kontextmenü der ausgewählten Zeile öffnen |

### Ctrl+Z / Ctrl+Y in einem Textfeld

Solange ein Textfeld die Tastatur hat, wirken `Ctrl+Z` und `Ctrl+Y` (auch `Ctrl+Shift+Z`)
auf den Text dieses Feldes, nie auf das Repository: Ein Feld, das die Tastatur hat, gewinnt
immer, sodass `Ctrl+Z` in der Commit-Nachricht keinen Commit rückgängig machen kann. Überall
sonst behalten sie ihre globale Bedeutung (das Journal für Rückgängig / Wiederholen oben).
Das gilt für jedes Textfeld und jeden mehrzeiligen Editor: die Commit-Nachricht, die Felder
und Texteditoren der Einstellungen, die Formulare für Pull Requests und Issues, die
KI-Dialoge und jeden Filter und jedes Suchfeld.

- **Tippen** wird wortweise rückgängig gemacht (eine Folge von Buchstaben, eine Folge von
  Leerzeichen, eine Folge von Satzzeichen), und Tippen, das länger als eine Sekunde pausiert,
  beginnt einen neuen Schritt. Aufeinanderfolgende Rücktasten sind ein Schritt. Ein Einfügen,
  ein Ausschneiden, `Ctrl+U` / `Ctrl+K`, ein eingefügtes Snippet oder eine übernommene
  Vervollständigung ist jeweils ein Schritt.
- Ein Rückgängig bringt den Text, den Cursor und die Markierung zurück. Eine neue Änderung
  nach einem Rückgängig verwirft, was sich hätte wiederholen lassen.
- Jedes Feld behält einen begrenzten Verlauf (die letzten 200 Schritte, etwa 1 MB Text) und
  vergisst ihn, wenn das Feld von außen ersetzt wird (ein im Auswahldialog gewählter Ordner,
  ein neu geladenes Formular) oder ein anderes Feld die Tastatur übernimmt.
- Ist nichts mehr rückgängig zu machen, tut `Ctrl+Z` nichts: Es fällt nie auf das
  Rückgängig des Repositorys durch, solange ein Feld die Tastatur hat.
- Der **Anweisungseditor** der KI-Einstellungen hat außerdem Schaltflächen **Rückgängig** und
  **Wiederholen**, neben Speichern und Abbrechen, deaktiviert, solange es nichts gibt, wohin
  man schreiten kann.
- Der Code-Editor und der Merge-Editor behalten ihr eigenes Rückgängig (`Bearbeiten:
  Rückgängig`, `Merge: Rückgängig`), das auch ihre Auswahlen umfasst.

## Filterbare Popup-Listen

Alle filterbaren Popup-Listen nutzen dieselben Tasten: die Befehlspalette und ihre
Auswahllisten (Design, Sprache, Graph-Farben), die Tab-Auswahl und jede Dropdownliste mit
Suchzeile (Auswahlen der Einstellungen, das Pull-Request-Formular, die Auswahl der
Issue-Referenz, die Liste **Öffnen in**). Tippen Sie, um die Liste zu filtern. Diese Tasten
bewegen die Hervorhebung:

| Taste | Aktion |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Eine Zeile nach oben / unten bewegen |
| `Home` / `End` | Erste / letzte Zeile |
| `PageUp` / `PageDown` | Eine sichtbare Seite bewegen |
| `Enter` | Die hervorgehobene Zeile wählen |
| `Esc` | Schließen |

In diesen Popups bewegen `Home` und `End` die Liste, nicht den Cursor des Filterfeldes,
und auch `Ctrl+K` / `Ctrl+J` bewegen die Liste, sodass `Ctrl+K` nicht mehr bis zum Ende des
Filters löscht (`Ctrl+U` leert ihn weiterhin; in einem gewöhnlichen Textfeld außerhalb einer
Liste löscht `Ctrl+K` weiterhin bis zum Zeilenende). `Ctrl+A` und `Ctrl+E` setzen den Cursor
weiterhin an den Anfang und das Ende. `Ctrl+J` / `Ctrl+K` bewegen jede andere Liste ebenfalls,
mit oder ohne Textfeld (`Ctrl+N` ist das Navigationspräfix und `Ctrl+P` die Befehlspalette,
sie sind also keine Listentasten). Einfaches `j` und `k` wird in den Filter getippt; in
Listen ohne Textfeld sind `j` / `k` ein zusätzlicher Weg, nach unten / oben zu gehen.

Mit der Maus:

- Das Mausrad **scrollt** die Liste. Es bewegt die Hervorhebung nicht.
- Ein Klick **wählt** eine Zeile **aus**. Ein Doppelklick **wählt** sie, wie `Enter`.
- Eine Liste, die höher als ihr Popup ist, zeigt an ihrem rechten Rand eine Bildlaufleiste.
  Sie lässt sich ziehen.
- Eine kurze Dropdownliste ohne Suchzeile (zwei oder drei Wahlmöglichkeiten) wählt beim
  ersten Klick.

**Live-Vorschau.** Die Auswahllisten für Design, Farbschema des Graphen und Sprache wenden
die hervorgehobene Zeile sofort an, hinter dem Popup. `Enter` behält sie. `Esc` oder ein
Klick außerhalb des Popups stellt den vorherigen Wert wieder her.

**Design-Filter.** Die Design-Auswahl hat unter dem Filterfeld eine Zeile: **Alle**,
**Dunkel**, **Hell** und **Farbig**. `Tab` / `Shift+Tab` gehen zum nächsten / vorherigen.
`Alt+1` bis `Alt+4` wählen direkt eines. Sie können auch darauf klicken.

## Hilfe in der App (`?`)

`?`, `Ctrl+N ?` oder `Ansicht: Hilfe-Überlagerung umschalten` der Palette öffnet die Hilfe. Sie listet
die Tasten und Mausgesten jedes Teils der App. Sie öffnet sich über allem, auch über dem
Einstellungsbildschirm.

| Taste | Aktion |
|---|---|
| `?` | Die Hilfe schließen (und den Filter löschen) |
| *(tippen)* | Filter: nur die Abschnitte behalten, die den Text enthalten. `j`, `k` und `q` werden ebenfalls getippt |
| `Backspace`, `Ctrl+U` | Das letzte Zeichen löschen / den Filter leeren |
| `Esc` | Den Filter leeren; ist er leer, die Hilfe schließen |
| `Up` / `Down` | Eine Zeile scrollen |
| `PageUp` / `PageDown`, `Home` / `End` | Eine Seite scrollen; nach oben / unten gehen |
| `Ctrl+D` oder ein Klick auf den Link | Diese Dokumentation auf GitHub im Browser öffnen |

Das Mausrad scrollt die Hilfe, und Sie können ihre Bildlaufleiste ziehen. Ein Klick außerhalb
der Hilfe schließt sie.

## Navigationspräfix (`Ctrl+N`)

`Ctrl+N` wartet auf eine weitere Taste. Die Statuszeile zeigt die Tasten, die es akzeptiert.
Die nächste Taste führt eine Aktion aus oder bricht ab.

| Taste | Aktion |
|---|---|
| `Ctrl+N` dann ein Badge | Den Teil des Fensters fokussieren, der dieses Badge zeigt |
| `Ctrl+N Shift+H` / `Ctrl+N Shift+L` | Den Refs-Bereich / den rechten Bereich einklappen oder wiederherstellen |
| `Ctrl+N Left` / `Ctrl+N Right` | Die Trennlinie zwischen Refs-Bereich und Log um eine Stufe nach links / rechts verschieben (ein eingeklappter Refs-Bereich kommt zuerst zurück) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Die Trennlinie zwischen Log und rechtem Bereich um eine Stufe nach links / rechts verschieben |
| `Ctrl+N +` / `Ctrl+N -` | Den aktiven Refs-Abschnitt um eine Zeile vergrößern / verkleinern |
| `Ctrl+N ]` / `Ctrl+N [` | Den Block der Commit-Nachricht des rechten Bereichs um eine Zeile vergrößern / verkleinern |
| `Ctrl+N Tab` | Einen Tab wählen (die Tab-Auswahl; `t` allein tut dasselbe) |
| `Ctrl+N t` / `Ctrl+N l` / `Ctrl+N Shift+T` | Das Design / die Sprache / das Farbschema des Graphen wechseln |
| `Ctrl+N Shift+Y` | Die Zeilentönung des Graphen ein- oder ausschalten |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Die von einem Agenten vorbereitete Ansicht zeigen / zurückgehen (siehe [MCP und Agenten](mcp-and-agents.md)) |
| `Ctrl+N ,` | Die Einstellungen öffnen |
| `Ctrl+N ?` | Die Hilfe öffnen oder schließen |
| `Esc` oder jede andere Taste | Abbrechen |

### Badges

Während das Präfix wartet, zeigt jeder Teil des Fensters, der den Fokus nehmen kann, ein
Badge mit einem Zeichen an seinem oberen linken Rand. Drücken Sie dieses Zeichen, um ihn zu
fokussieren. Badges werden jedes Mal neu vergeben, in Bildschirmreihenfolge: zuerst `1` bis
`9`, dann `a`, `b`, `c`... Lesen Sie also das Badge auf dem Bildschirm: Ein Teil bekommt
nicht immer dasselbe Badge.

Teile mit einem Badge: der Refs-Filter und jeder sichtbare Refs-Abschnitt, das Log (oder
die Diff-Ansicht), die Dateiansicht, wenn sie offen ist, die Listen Unstaged und Staged, das
Zusammenfassungsfeld des Commit-Composers und die Bedienelemente der Dateiliste des Commits.
Im Home-Tab sind die Badges das Suchfeld, die vier Abschnitte und die Schaltflächen der Werkzeugleiste (das Badge einer Schaltfläche löst sie aus). Im
Radar-Tab sind es das Filterfeld, die Auswahllisten Umfang, Ansicht und Anzeigen (das
Badge einer Auswahlliste öffnet sie), die Schaltflächen Aktualisieren und Zurück sowie
jede Abschnittsüberschrift auf dem Bildschirm. Das Badge eines Abschnitts setzt die
Auswahl auf seine Überschrift; mit `Enter` oder `Space` klappen Sie ihn dann zu.

In der Ordnerauswahl Durchsuchen und in den Dialogen Klonen, Init, Worktree erstellen, Workspace, Sparse Checkout, Flow sowie Neues Issue und Pull Request sind die Badges die Felder, Kontrollkästchen und Schaltflächen (das Badge einer Schaltfläche drückt sie, das eines Feldes setzt den Cursor hinein; in einer Liste liegt ein Badge auf ihrer ersten Zeile; die Ordnerauswahl Durchsuchen hat ein Badge für die Liste Schnellzugriff und eines für die Liste Ordner, neben der ersten Zeile jeder Liste, und ein Druck darauf legt die Tastatur in diese Liste, ohne eine Zeile zu wählen). Im Aktivitätsprotokoll sind es die beiden Tabs, die Felder Bereich und Level, der Filter, die Kontrollkästchen der Level, die Schaltflächen und der erste Eintrag der Liste. Ein Badge tut, was ein Klick auf dieses Bedienelement tut. In der Ordnerauswahl startet `Ctrl+N` die Badges, daher lautet ihre Taste für Neuer Ordner `Alt+N`.

Im Dialog Ändern (und Commit-Nachricht bearbeiten) sind die Badges die Zusammenfassung, die Beschreibung, Name, E-Mail und Datum des Autors, **Verlauf weitergehend ändern...**, **Abbrechen**, **Anwenden** und das Schließen-Kreuz; in der Werkbank zum Umschreiben des Verlaufs sind es das Bereichsfeld, seine Kontrollkästchen und die Schaltfläche Refs, der Name des Rezepts, jede Regel, ihr Kontrollkästchen und ihre Schaltflächen, die Schaltfläche **+ Regel hinzufügen**, die Felder der gewählten Regel, die Tabs der Vorschau, das Filterfeld, die Zeilen der geschützten Branches mit ihren Schaltflächen Alle wählen / Alle abwählen und die Schaltflächen Vorschau, Anwenden, Abbrechen, Speichern, Laden und Regeln mit KI erstellen (die Seiten Secrets, Identitäten, Größe und Ergebnis fügen eigene Schaltflächen und Zeilen hinzu; die Zeilen des Tabs Commits haben als Liste keine). Ein Badge tut, was ein Klick tut. Nichts erhält ein Badge, solange ein Dropdown, eine Frage, ein Feld in Eingabe, die Ordnerauswahl, der Purge-Bildschirm, der Skript-Editor, die KI-Dialoge, der Blob-Diff oder die Host-Anleitung offen ist, und auch nicht, solange der Dialog Ändern fragt, ob verworfen werden soll; `Esc` nimmt die Badges weg, ohne etwas zu schließen.

In den Einstellungen sind die Badges das Suchfeld, die Seitenliste (ein Badge, auf der aktuellen Seite), die Unter-Tabs, die Liste mit ihren Schaltflächen, jedes Feld der Seite auf dem Bildschirm und das Schließen-Kreuz. Ein Badge tut, was ein Klick auf dieses Element tut: Ein Textfeld wird zum Bearbeiten geöffnet, ein Kontrollkästchen wird umgeschaltet, eine Dropdownliste öffnet sich, eine Schaltfläche wird ausgeführt. `Ctrl+N` behält seine eigene Bedeutung, solange in ein Feld getippt wird oder eine Dropdownliste, ein Editor oder die Suche offen ist.

Die Palette hat dieselben Aktionen, zum Beispiel
`Navigation: Refs > Lokal fokussieren` oder `Navigation: Status > Staged fokussieren`.

## Repositories und Tabs

| Taste | Aktion |
|---|---|
| `Ctrl+O` oder ein Klick auf `⌂` | Zum Home-Tab gehen (im Home-Tab: das Suchfeld fokussieren; bei geöffnetem Radar oder geöffneten Versionshinweisen: zuerst schließen) |
| `Ctrl+Shift+N` | Den Home-Tab mit dem Klon-Dialog öffnen |
| `Ctrl+PageDown` / `Ctrl+PageUp`, `Ctrl+Tab` / `Ctrl+Shift+Tab` | Nächster / vorheriger Tab (das Paar mit `Ctrl+Tab` nur, wo das Terminal es sendet) |
| `t` | Einen Tab aus einer filterbaren Liste wählen |
| `Ctrl+W` | Den aktuellen Tab schließen (fragt vorher; das Repository wird nicht geändert) |
| `Ctrl+T` oder ein Rechtsklick auf einen Tab | Das Tab-Menü (seine Zeilen stehen unter [Tab-Leiste](interface-tour.md#tab-leiste)) |
| `Alt+O` / `Ctrl+Alt+O` | Den Ordner des Tabs öffnen: aus der Liste **Öffnen in** wählen / den Standardeintrag nutzen. Siehe [Öffnen in](open-in.md) |

### Der Home-Tab

Der Tab `⌂` verwaltet Ihre Repositories. Er hat vier Abschnitte: **Geöffnete Repositories**,
**Favoriten**, **Zuletzt verwendete Repositories** und **Alle Repositories**. Siehe
[Rundgang durch die Oberfläche](interface-tour.md#home).

| Taste | Aktion |
|---|---|
| `j` / `k`, Pfeile, `PageUp` / `PageDown`, `Home` / `End` | Bewegen |
| `Enter` oder ein Doppelklick | Das Repository öffnen (oder zu seinem Tab wechseln). Auf einer Abschnittsüberschrift: sie falten oder entfalten |
| `f` oder `*`, oder ein Klick auf den Stern | Zu Favoriten hinzufügen / aus Favoriten entfernen |
| `x`, `Delete` oder die Schaltfläche der Zeile | Seinen Tab schließen (Geöffnet), es aus der Liste entfernen (Zuletzt), den Stern entfernen (Favoriten), es vergessen (Alle). Auf dem Datenträger wird nichts gelöscht |
| `.`, ein Rechtsklick oder das `⋮` der Zeile | Das Menü der Zeile |
| `/` oder `Ctrl+F` | Alle Abschnitte nach Namen, Branch oder Pfad durchsuchen. `Enter` öffnet den ersten Treffer; `Esc` löscht die Suche und verlässt dann das Feld |
| `Space`, `Left` / `Right` | Den aktuellen Abschnitt falten / entfalten |
| `-` / `+` | Alle einklappen / Alle ausklappen |
| `b` / `c` / `i` | Suchen (einen Ordner öffnen) / Klonen / Init |
| `Esc` | Die Suche löschen oder zum aktiven Repository-Tab zurückkehren |
| `q` | Beenden |

`Ctrl+P`, `Ctrl+,`, `?` und `Ctrl+N` funktionieren im Home-Tab. `Ctrl+G`, `Ctrl+W` und
`Ctrl+T` brauchen einen Repository-Tab: Im Home-Tab zeigen sie nur eine Meldung.
Palettenbefehle, die ein Repository brauchen, sind ausgegraut.

### Die Ordnerauswahl

**Durchsuchen** (und die Schaltfläche **Durchsuchen...** der Dialoge Init, Klonen und Worktree
erstellen) öffnet die Ordnerauswahl. Sie hat oben ein Pfadfeld, links die Orte und den
Ordnerbaum und rechts den Inhalt des aktuellen Ordners.

| Taste | Aktion |
|---|---|
| `Tab` / `Shift+Tab` | Pfadfeld, dann die linke Spalte, dann der Inhalt |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | In der fokussierten Spalte bewegen. Im Pfadfeld: durch die Vervollständigungen gehen |
| ein Buchstabe (Inhalt) | Zum nächsten Ordner springen, der mit diesem Buchstaben beginnt |
| `Enter` / `Right` | Den ausgewählten Ordner öffnen. Im Pfadfeld: zum getippten Ordner gehen; ist er schon angezeigt, ihn wählen |
| `Backspace` / `Left` / `Alt+Up` | Übergeordneter Ordner (im Pfadfeld löscht `Backspace` ein Zeichen) |
| `Tab` (Pfadfeld) | Die einzige oder die hervorgehobene Vervollständigung übernehmen |
| `Alt+N` | Neuer Ordner (`Ctrl+N` startet die Badges, siehe [Badges](#badges)) |
| `Alt+H` | Versteckte Ordner zeigen oder verbergen |
| `Ctrl+F` oder `/` außerhalb des Pfadfelds | Die Einträge des Ordners beim Tippen filtern (ohne Beachtung der Groß- und Kleinschreibung; `..` bleibt; die Kopfzeile zählt „x von n“; `Enter` springt zum ersten Treffer, `Esc` leert den Filter; er leert sich beim Ordnerwechsel). Auch mit `Tab` oder einem Klick erreichbar |
| `Ctrl+S`, `Ctrl+Enter` oder die Bestätigungsschaltfläche | Den ausgewählten Unterordner wählen, sonst den angezeigten Ordner |
| `Esc` | Die Vervollständigungen schließen oder abbrechen |

Zeigt das Pfadfeld einen neuen Pfad, ist der ganze Pfad markiert: Die erste getippte Taste
ersetzt ihn.

### Dialoge Init und Klonen

| Taste | Aktion |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Nächstes / vorheriges Feld oder nächste / vorherige Schaltfläche |
| `Enter` | Die fokussierte Schaltfläche drücken oder erstellen / klonen |
| `Space` | Das fokussierte Kontrollkästchen umschalten oder die fokussierte Schaltfläche drücken |
| `Ctrl+B` | Nach dem Ordner suchen |
| `j` / `k`, `g` / `G` | In der Quellenliste und der Repository-Liste des Klonen-Dialogs: ab / auf, erste / letzte (nicht im Suchfeld) |
| `Esc` | Schließen. Beim Klonen läuft der Klon im Hintergrund weiter |

Im Klon-Dialog listet die linke Spalte **Mit URL klonen** und eine Quelle pro verbundenem
Hosting-Profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Wählen Sie eine Quelle mit `Up` /
`Down` oder per Klick. Tippen Sie dann im Suchfeld **Repository**, bewegen Sie sich mit den
Pfeilen, `PageUp` / `PageDown` oder dem Mausrad in der Liste und wählen Sie eine Zeile mit
`Enter` oder per Klick. Siehe [Erste Schritte](getting-started.md) für die Felder.

## Refs-Bereich

| Taste | Aktion |
|---|---|
| `Enter` / `Space` | Den aktiven Abschnitt oder den ausgewählten Ordner auf- oder zuklappen |
| `h` / `l`, `Left` / `Right` | Den Abschnitt wechseln |
| `/` | Nach Namen filtern (`Enter` übernimmt, `Esc` bricht ab) |
| `y` | Branch-Hierarchie ein- oder ausschalten: Ordner aus den `/` in Branch-Namen (LOKAL und REMOTE) |
| `o` | Die ausgewählte Ref **solo** stellen, oder jede Ref unter dem ausgewählten Ordner oder Remote. Mehrere Refs solo stellen, um ihre Vereinigung zu sehen |
| `Shift+O` | **Solo beenden**: jedes Solo aufheben |
| `Shift+H` | ** Alle zeigen **: jede ausgeblendete Ref wieder zeigen |
| `g` | **Im Graph anzeigen**: scrollt den Log zum Commit der ausgewählten Ref (oder des Stash) und lässt die Zeile aufblinken; die Auswahl bleibt unverändert |
| `m` | Die ausgewählte Ref als Ablagequelle **markieren** (`Esc` oder erneut `m` darauf entfernt die Markierung) |
| `Shift+M` | **Die markierte Ref ablegen** auf der ausgewählten Ref: öffnet das Menü, das auch Ziehen und Ablegen öffnet (Merge, Rebase, Push, Fast-Forward...) |
| `+` (Abschnitt REMOTE) | Ein Remote hinzufügen |
| `.`, `Menu`, `Shift+F10` oder ein Rechtsklick | Das Menü der Zeile: ein Branch, ein Tag, der Name eines Remotes oder ein Ordner |
| `Enter` auf einer Zeile WORKTREES | Diesen Worktree in einem eigenen Tab öffnen |
| `Ctrl+G d` | Den ausgewählten Branch oder Tag löschen (fragt vorher). Auf einer Zeile REMOTE: diesen Remote-Branch löschen |

Was die Menüs tun, steht unter [Mit dem Log arbeiten](log-and-history.md).

### Pull Requests und Issues

Mit einer passenden Integration hat der Refs-Bereich einen Abschnitt für Pull Requests
(GitLab: Merge Requests). Mit einem verbundenen Hosting-Profil hat er auch einen Abschnitt
ISSUES. Siehe [Hosting und Issue-Tracker](pull-requests-and-issues.md#pull-requests-und-issues-in-der-seitenleiste).

| Taste | Aktion |
|---|---|
| `Enter` | Die ausgewählte Zeile öffnen: ein Issue (und ein Request bei Azure DevOps oder Bitbucket) als [Unterhaltung](conversations.md), einen Pull Request im [Review](review.md), wo der Host eines hat, alles andere im Browser. Auf einer Zeile `Reconnect`: die Einstellungen auf diesem Profil öffnen |
| `Space` | Den Abschnitt auf- oder zuklappen |
| `v` | Den Filter des Abschnitts öffnen. Pull Requests: Offen, Gemergt / geschlossen, Alle. Issues: Offen, Mir zugewiesen, Geschlossen, Alle |
| `n` | Abschnitt der Pull Requests: einen Pull Request erstellen. Abschnitt ISSUES: neues Issue. (Die Schaltfläche `+` der Kopfzeile) |
| `e` | Abschnitt der Pull Requests: den ausgewählten Request beim Host bearbeiten (der Stift der Zeile) |
| `Enter` auf der Suchzeile | Im Suchfeld tippen. `Enter`, `Tab` oder `Esc` verlässt es; `Ctrl+U` leert es |
| `Enter` auf einer Gruppenüberschrift | Die Gruppe falten oder entfalten (*Meine Pull Requests*, *Mir zugewiesen*, *Wartet auf mein Review*, *Alle Pull Requests*) |
| `Enter` auf der Zeile von Filter, Anbieter oder Repository | Diese Dropdownliste öffnen. Die Anbieterliste endet mit **Keiner (dieses Repository trennen)** |
| `.` oder ein Rechtsklick | Das Menü der Zeile: die Unterhaltung öffnen, im Browser öffnen, URL oder Nummer kopieren, den Quell-Branch auschecken (Pull Requests); einen Branch aus dem Issue erstellen (Issues) |

### Das Issue-Board

**Board** in der Kopfzeile von ISSUES (Palette: **Issues: Board öffnen**) öffnet das Kanban-Board der Issue-Quelle. Siehe
[das Board](pull-requests-and-issues.md#das-issue-board) und [lokale Issues](local-issues.md#das-kanban-board). Jede Taste
unten ist auch ein Palettenbefehl (**Board: ...**).

| Taste | Aktion |
|---|---|
| Pfeile | Eine Karte auswählen (`Up`, `Down`) oder die benachbarte Spalte (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` springen in der Spalte |
| `Shift+Left`, `Shift+Right` | Die ausgewählte Karte in die vorherige oder nächste Spalte verschieben |
| `m` | **Karte verschieben nach...**: eine Dropdownliste der Spalten |
| `Enter` | Die [Unterhaltung](conversations.md) des Issues öffnen (die Seite des Hosts, wo der Host keine zu lesen hat) |
| `e`, `.` | Zugewiesene, Labels oder Meilenstein bearbeiten; das Kartenmenü |
| `n` | Neues Issue in der ausgewählten Spalte |
| `a`, `l`, `M`, `/` | Nach Zugewiesenen, Label, Meilenstein, Text filtern. `x` setzt die Filter zurück |
| `c` | Die Labels, die die Spalten eines GitHub- / GitLab-Boards sind |
| `r` | Die Issues erneut lesen |
| `Esc` | Das Board schließen |

Maus: Eine Karte auf eine andere Spalte ziehen, um sie zu verschieben, eine Karte anklicken, um sie auszuwählen, doppelklicken, um sie zu öffnen, Rechtsklick für
das Kartenmenü, Mausrad scrollt eine Spalte, `Shift`+Mausrad (oder die Leiste unter dem Board) scrollt die Spalten.

### Der Markdown-Editor und die Unterhaltungsansicht

Jedes Feld, dessen Text an einen Host geht (ein Kommentar, eine Antwort, eine Review-Zusammenfassung, die Beschreibung eines neuen Issues oder Pull Requests), ist dasselbe Textfeld. Seine Tasten stehen in der Tabelle `md_editor` oben und lassen sich neu belegen.

| Taste | Aktion |
|---|---|
| `Alt+P` | **Schreiben** und **Vorschau** tauschen (aus **Geteilt** geht es zur Vorschau) |
| `Alt+L` | **Geteilt** an, oder zurück zu Schreiben (braucht ein Fenster mit 100 Spalten) |
| `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` | Den Text senden (die eigene Schaltfläche des Formulars tut dasselbe) |
| `Alt+E` | **Mit KI schreiben**: öffnet oder schließt das Panel unter der Modusleiste (nur mit KI an) |
| `Alt+R` | Nur im Commit-Composer: einen Issue-Verweis einfügen |
| `Alt+W` | Die Formulare für neue Issues und Pull Requests: auf der eigenen Webseite des Anbieters erstellen |
| `Esc` | Im Panel von **Mit KI schreiben**: einen laufenden Entwurf stoppen, sonst das Panel schließen |

In der [Unterhaltungsansicht](conversations.md) eines Issues oder Requests (Palettenzeilen *Unterhaltung: ...*; die Zeilen mit einer Taste stehen in der Palettenliste oben):

| Taste | Aktion |
|---|---|
| `r` | Auf den ausgewählten Eintrag antworten (in seinen Thread, oder bei einem Host ohne Threads mit zitiertem Text) |
| `Q` | Den ausgewählten Eintrag in einem neuen Kommentar zitieren |
| `c` | Neuer Kommentar |
| `e`, `d` | Einen eigenen Kommentar bearbeiten, löschen |
| `Alt+I` | Im Kommentarfeld: **Interne Notiz** (GitLab, Redmine) |
| `l`, `y`, `o` | Links des Eintrags, seine Adresse kopieren, ihn im Browser öffnen |
| `n` | Nächster ungelesener Eintrag |
| `v`, `m`, `F5` | Ereignisse zeigen oder ausblenden, ältere Einträge laden, aktualisieren |
| `g` | Das Diagramm des Eintrags öffnen (ein `mermaid`-Block) |
| `Alt+Left`, `Backspace` | Zurück entlang des Stapels der aus Verweisen geöffneten Elemente |
| `Tab` | Zeitleiste, Kommentarfeld, Schaltflächen der Kopfzeile |
| `q`, `Esc` | Schließen |

Im Radar öffnet `i` die Unterhaltung einer Issue-Zeile.

Auf einer Pull-Request- oder Issue-Zeile der Seitenleistenbereiche öffnet `Enter` die Unterhaltung, und `o` öffnet die Zeile im Browser.

### Flow (Branching-Workflows)

Ist für das Repository ein Workflow eingerichtet (siehe [Branching-Workflows](branching-workflows.md)), hat die Symbolleiste
eine Schaltfläche **Flow**, und die Seitenleiste gruppiert LOKAL und REMOTE nach Branch-Typ. Jede Zeile des Menüs ist ein
Palettenbefehl (**Flow: ...**; **Flow: Feature starten...** und **Flow: Neues Feature-Issue...** werden für jeden Topic-Typ des
Workflows erzeugt). Die vom Host gesteuerten Zeilen (*Neues \<Typ\>-Issue*, *Vom Host erstellten Branch aufgreifen*) und ihre
Dialoge, die Wartekarte und die Karte „Ein Branch ist erschienen“ sind in
[Branching-Workflows](branching-workflows.md#vom-host-gesteuerte-flows-in-der-app) beschrieben;
`Enter` führt die fokussierte Schaltfläche aus (*Issue erstellen*, *Aufgreifen*), `Esc` hört auf zu warten oder schließt.

| Taste | Aktion |
|---|---|
| `Ctrl+G Shift+W` | Das Flow-Menü öffnen (die Schaltfläche der Symbolleiste) |
| `Enter`, `Tab`, `Shift+Tab` | Im Dialog Starten / Abschließen / Synchronisieren / Hochstufen / Eltern-Branch festlegen: Vorschau oder Ausführen, nächstes Feld, vorheriges Feld |
| `Space` | Ein Kästchen setzen (*Veröffentlichen*, *Hooks ausführen*, *Auch die darauf aufbauenden Branches aktualisieren*) |
| `Esc` | Ein Dropdown schließen, von einer Vorschau zum Formular zurückgehen, den Dialog schließen (nicht während ein Lauf läuft) |
| Mauszeiger über einem Branch | Sein Typ, sein Eltern-Branch und wie weit er hinter dem Eltern-Branch liegt |

## Log-Bereich

| Taste | Aktion |
|---|---|
| `Enter` / `l` | In die Liste der geänderten Dateien des ausgewählten Commits gehen |
| `Enter` / `l` auf einer Dateizeile | Den Diff der Datei in der Mitte öffnen |
| `Esc` / `h` | Zurück zu den Commits |
| `Right` | (Commit mit mehr als einer Ref) Die Liste seiner anderen Refs öffnen: `Up` / `Down` bewegen, `Enter` checkt aus, `Esc` schließt |
| `f` | Die Suchleiste öffnen (siehe [Suche](#suche)); das war nie `/`, die Suche des Logs ändert sich also nicht |
| `/` | Das Filterfeld der Refs fokussieren (dieselbe Taste tut es, wenn der Bereich Refs den Fokus hat); wird als Zeichen getippt, wenn ein Textfeld den Fokus hat |
| `c` | (mit nicht committeten Änderungen) Die WIP-Zeile auswählen und den Cursor in ihr Nachrichtenfeld setzen |
| `Enter` auf der WIP-Zeile | Den Cursor in das Nachrichtenfeld der WIP-Zeile setzen |
| `.`, `Menu`, `Shift+F10` | Das Commit-Menü. `Up` / `Down` bewegen, `Right` / `l` / `Enter` öffnen ein Untermenü, `Left` / `h` / `Esc` gehen zurück |
| `m` / `a` | Dateien des rechten Bereichs: `Pfad` oder `Baum` / `Alle Dateien ansehen` |
| `Shift+C` | Eine Log-Spalte zum Verschieben oder Anpassen wählen (siehe unten) |

Ist eine Spalte gewählt, wählen `Tab` / `Shift+Tab` eine andere, `<` / `>` verschieben sie, `]` / `[` verbreitern oder verschmälern sie, `0` setzt alle Breiten auf automatisch zurück, und `Esc` oder `Enter` beendet (Kontext `log_columns` in der oben generierten Tabelle).

Das Nachrichtenfeld der WIP-Zeile ist die Zusammenfassung des Commit-Composers. Solange es den
Cursor hat, wird jede Taste getippt, aber `Ctrl`-Tastenfolgen funktionieren weiterhin. `Enter`
wechselt zur Beschreibung des Composers. `Esc` verlässt das Feld und behält den Text. Siehe
[Staging und Committen](staging-and-committing.md).

### Mehrere Commits auswählen

| Taste oder Geste | Aktion |
|---|---|
| `Space` | Den aktuellen Commit zur Auswahl hinzufügen oder entfernen |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Auswahl erweitern (Alt für Terminals, die Shift für sich behalten) |
| `Ctrl+Klick` / `Shift+Klick` oder `Alt+Klick` | Einen Commit umschalten / einen Bereich auswählen |
| `Esc` | Die Auswahl löschen |
| `Shift+Y` | [Die Auswahl als Mermaid gitGraph kopieren](log-and-history.md#commits-als-mermaid-gitgraph-kopieren) (ein Popup mit Detailstufen und Live-Vorschau; `Enter` kopiert) |

Sind mehrere Commits ausgewählt, klicken Sie mit rechts auf einen davon und wählen Sie **N
Commits per Cherry-Pick auf `<Branch>`...**. Das öffnet dieselbe Ansicht wie ein interaktiver
Rebase.

### Einen Branch oder Tag benennen

`Ctrl+G b`, `Ctrl+G t`, die Schaltflächen **Branch** und **Tag** der Symbolleiste und die
Einträge **Branch hier erstellen...** / **Tag hier erstellen...** der Menüs fragen den Namen
in einem Textfeld in der Zeile des Commits ab. `Enter` legt die Ref an. `Esc` bricht ab. Ein
ungültiger oder vorhandener Name wird rot angezeigt, und das Feld bleibt offen.

Ist diese Zeile nicht sichtbar (ein Diff ist offen, das Log ist solo, die Ref-Spalte ist
ausgeblendet), wird der Name stattdessen in einem kleinen Rahmen mit Schließkreuz und den
Schaltflächen **Abbrechen** und **Erstellen** getippt. Das Umbenennen eines Branches geschieht
direkt über seiner Zeile im Refs-Bereich oder in der Ref-Zelle seiner Log-Zeile; eine Datei wird
in der Liste der Arbeitsänderungen unter dem Ordner angelegt. Überall gelten dieselben Tasten:
`Enter` bestätigt, `Esc` bricht ab.

In der Statuszeile wird nie getippt. Eingaben, die mehr Platz brauchen, sind Dialoge mit echten
Schaltflächen: die Stash-Nachricht, die Nachricht eines annotierten Tags (`Ctrl+Enter` oder
`Ctrl+S` wendet an, `Enter` fügt eine Zeile ein), Upstream festlegen (eine durchsuchbare Liste
der Remote-Branches samt Schaltfläche zum Entfernen) und die Git-LFS-Muster zum Verfolgen und
Beenden der Verfolgung. Eine Zahlen- oder Texteinstellung, die in der Palette gewählt wird,
öffnet die Einstellungen an dieser Zeile, bereit zum Bearbeiten, und der Refs-Filter (`/`)
wird in das Feld oben im Refs-Bereich getippt.

### Eine Commit-Nachricht bearbeiten

**Commit-Nachricht bearbeiten...** im Commit-Menü, `Ctrl+G e` oder der Palettenbefehl
`Git: Commit-Nachricht bearbeiten...` öffnet das Popup **Commit-Nachricht umformulieren**.
Was es ändert, steht unter [Mit dem Log arbeiten](log-and-history.md).

| Taste | Aktion |
|---|---|
| `Enter` in der Zusammenfassung | Zur Beschreibung gehen |
| `Enter` in der Beschreibung | Neue Zeile. `Up` in der ersten Zeile geht zurück zur Zusammenfassung |
| `Ctrl+Enter`, `Ctrl+S` oder **Nachricht aktualisieren** | Die neue Nachricht speichern |
| `Tab` / `Shift+Tab` | Zwischen Zusammenfassung, Beschreibung und Schaltflächen wechseln |
| `Esc` oder **Abbrechen** | Ohne Änderung schließen |

### Einen Commit ändern

`Ctrl+G Shift+E` (alle Felder) und `Ctrl+G Shift+A` (der Autor) öffnen den Ändern-Dialog, ebenso das Commit-Menü, die Palettenzeilen `Commit: ... ändern...` und der Detailbereich des Commits; alle Wege dorthin stehen unter [Einen Commit ändern](log-and-history.md#einen-commit-ändern-nachricht-autor-datum). Beide Kürzel lassen sich neu belegen: `git_prefix.amend` und `git_prefix.edit_author`. Tasten im Dialog:

| Taste | Aktion |
|---|---|
| `Tab` / `Shift+Tab` | Zwischen den Feldern und Schaltflächen wechseln |
| `Enter` in einem einzeiligen Feld | Nächstes Feld; im letzten Feld **Anwenden** |
| `Up` / `Down` | Vorheriges / nächstes Feld (in der Beschreibung erst von der ersten / letzten Zeile aus) |
| `Ctrl+Enter`, `Ctrl+S` oder **Anwenden** | Die geänderten Felder anwenden |
| `Esc` oder **Abbrechen** | Ohne Änderung schließen |
| `Alt+H` oder **Verlauf weitergehend ändern...** | Schließt den Dialog und öffnet die Werkbank zum Umschreiben des Verlaufs ab diesem Commit (nur in Builds mit Verlaufs-Umschreibung); fragt vorher, wenn Sie ein Feld geändert haben (`Esc` bestätigt). Neu belegbar: `reword.deeper` |

### Worktree erstellen

**Worktree erstellen aus `<Ref>`...** (Menüs von Branch und Tag) und **Worktree hier
erstellen...** (Commit-Menü) öffnen einen Dialog: den Ordner, einen optionalen neuen Branch
und **Den neuen Worktree als Tab öffnen**. Tasten: `Tab` / `Shift+Tab` bewegen, `Enter`
erstellt, `Space` schaltet das Kontrollkästchen um, `Ctrl+B` durchsucht, `Esc` bricht ab.

### Interaktiver Rebase

**Interaktiver Rebase ab hier...** im Commit-Menü oder
`Git: Interaktiver Rebase ab ausgewähltem Commit...` öffnet die Rebase-Ansicht. Ein
Cherry-Pick mehrerer Commits nutzt dieselbe Ansicht. Siehe
[Mit dem Log arbeiten](log-and-history.md).

| Taste | Aktion |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Die Aktion der Zeile setzen: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup und Edit nur in einem Rebase) |
| `Space` | Die Aktionsliste der Zeile öffnen (`Up` / `Down` wählen, `Enter` übernimmt, `Esc` schließt die Liste) |
| `Up` / `Down` | Die Auswahl bewegen |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Die ausgewählte Zeile nach oben / unten schieben |
| `a` | Autosquash-Vorschau ein- oder ausschalten |
| `Enter` auf einer Zeile Reword | Die neue Nachricht dieses Commits bearbeiten |
| `Ctrl+S`, `Ctrl+Enter` oder **Rebase starten** / **Cherry-Pick starten** | Den Plan ausführen |
| `Ctrl+R` oder **Zurücksetzen** | Den Plan so wiederherstellen, wie er beim Öffnen der Ansicht war |
| `Esc` oder **Abbrechen** | Ohne Änderung schließen |

Hält ein Rebase an (bei einem Konflikt oder einem Schritt Edit), bietet ein Banner über dem
Log **Fortsetzen**, **Überspringen** und **Abbrechen** an. Die Palette hat `Rebase:
Fortsetzen`, `Rebase: Überspringen` und `Rebase: Abbrechen`.

## Suche

`f`, die Schaltfläche **Suchen** der Symbolleiste oder `Navigation: Commits suchen...`
öffnet die Suchleiste über dem Log. Sie durchsucht die ganze Commit-Nachricht, Name und
E-Mail des Autors, den SHA und die Ref-Namen. Sie blendet keine Zeilen aus: Zeilen, die nicht
passen, werden abgedunkelt.

| Taste | Aktion |
|---|---|
| *(tippen)* | Den Suchtext bearbeiten (jede Taste wird getippt) |
| `Enter` / `Down` oder das `↑` der Leiste | Nächster Treffer |
| `Shift+Enter` / `Up` oder das `↓` der Leiste | Vorheriger Treffer |
| `Esc` oder das `✕` der Leiste | Die Suchleiste schließen |

## Diff-Ansicht

Eine geänderte Datei öffnet sich in der Mitte, anstelle des Logs. Siehe
[Staging und Committen](staging-and-committing.md).

| Taste | Aktion |
|---|---|
| `Enter` / `l` oder ein Doppelklick (auf einer Dateizeile) | Den Diff der Datei öffnen |
| `Esc`, `q` oder das `✕` im Titel | Den Diff schließen |
| `j` / `k`, `Up` / `Down` | Scrollen |
| `PageUp` / `PageDown`, `Home` / `End` | Eine Seite scrollen; zum Anfang / Ende gehen |
| `h` / `l`, `Left` / `Right` | Seitwärts scrollen (nicht mit Zeilenumbruch) |
| `n` / `p` | Nächster / vorheriger Hunk |
| `m` | Inline- oder geteiltes Layout |
| `Shift+H` | Nur Hunks oder ganze Datei |
| `w` | Zeilenumbruch ein- oder ausschalten |
| `s` | Änderungen nur an Leerraum zeigen oder verbergen |
| `Shift+M` | Minimap ein- oder ausschalten |
| `Shift+C` | Die Codierung wählen, in der die Datei gelesen wird (eine Liste: zuerst Automatisch erkennen) |
| `e` | Die Datei im eingebauten Editor bearbeiten (siehe [Editor](#editor)) |
| `r` | (Git-LFS-Datei) Die LFS-Karte oder der rohe Zeiger. Siehe [Git LFS](lfs.md) |
| `Shift+E` | (Datei im Konflikt) Den Merge-Editor öffnen. Siehe [Merge-Konflikte lösen](merge-conflicts.md) |
| `y` / `Ctrl+C` | Den markierten Text kopieren. Ohne Markierung: die Zeile unter dem Cursor oder die ausgewählten Zeilen |
| `.` oder ein Rechtsklick auf einen Hunk | Das Hunk-Menü |

### Aus dem Diff stagen und verwerfen

Diese Tasten funktionieren nur in einem Diff der Änderungen (aus der Liste Unstaged oder
Staged geöffnet). Verwerfen geht nur in der Liste Unstaged.

| Taste | Aktion |
|---|---|
| `Space` | Den Hunk unter dem Cursor stagen (Unstaged) / unstagen (Staged) |
| `v` | Eine Zeilenauswahl beginnen oder verwerfen; `j` / `k` erweitern sie |
| `Enter` | Die ausgewählten Zeilen oder die Zeile unter dem Cursor stagen / unstagen |
| `x` | Den Hunk unter dem Cursor verwerfen (fragt vorher: `y` / `Enter` bestätigen, `n` / `Esc` brechen ab) |
| `Shift+X` | Die ausgewählten Zeilen oder die Zeile unter dem Cursor verwerfen (fragt vorher) |

Nur geänderte Zeilen zählen: Kontextzeilen in einer Auswahl werden ignoriert. Eine Auswahl,
die in einen zweiten Hunk hineinreicht, wirkt nur auf den ersten Hunk.

## Bildansicht

Ein Bild in der Diff-Ansicht oder der Dateiansicht. Die Tasten stehen im Kontext `image` der
Tabelle oben. Siehe [Bilder](image-view.md).

## Dateiansicht, Blame und Dateiverlauf

| Taste | Aktion |
|---|---|
| `Esc` / `q` | Schließen (mit einer Textmarkierung löscht `Esc` sie zuerst) |
| `j` / `k`, `Up` / `Down` | Die Auswahl bewegen |
| `PageUp` / `PageDown` | Eine Seite scrollen |
| `Home` / `End`, `Shift+G` | Erste / letzte Zeile |
| `h` / `l`, `Left` / `Right` | Den Text seitwärts scrollen |
| `Enter` | (Blame, Dateiverlauf) Zum Commit der Zeile im Log gehen |
| `e` | Die Datei im eingebauten Editor bearbeiten |
| `y` / `Ctrl+C` | Den markierten Text kopieren. Ohne Markierung: die ausgewählte Zeile (Blame: nur der Code; Dateiverlauf: die Zusammenfassung) |

Blame und Dateiverlauf haben eigene Tasten, aufgeführt in der erzeugten Tabelle oben (Kontexte `file_insight` und `history`) und als `Blame: ...` / `Dateiverlauf: ...` in der Palette; alle lassen sich neu belegen. Details unter [Blame](log-and-history.md#blame) und [Dateiverlauf](log-and-history.md#dateiverlauf). In der Verlaufsliste wirken die einzelnen Buchstaben (`d` `o` `b` `c` `y`) auf die gewählte Zeile, ein Ctrl-Akkord (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) behält aber stets seine globale Bedeutung.

## Commit-Spalte

Die Spalte links neben der Ansicht einer Datei (siehe [Die Commit-Spalte](interface-tour.md#die-commit-spalte)). `Alt+C` blendet sie von überall aus ein oder aus; die anderen Tasten wirken, sobald sie die Tastatur hat (`Ctrl+N`, dann ihr Badge, oder `Ansicht: Commit-Spalte fokussieren`).

| Taste | Aktion |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Einen Commit wählen |
| `Enter` / `Right` | Weiter zur Dateiliste des gewählten Commits |
| `Esc` | Die Tastatur an die Ansicht zurückgeben |
| `.`, `Menu`, `Shift+F10` | Das Menü der gewählten Zeile |

Ihre anderen Tasten (`Shift+Left` / `Shift+Right` zum seitlichen Scrollen, `Tab`, `]`, `[` und `0` zum Einstellen der Teile) stehen in der erzeugten Tabelle oben, Kontext `commit_column`.

## Editor

**Bearbeiten** oder `e` in einem Diff oder einer Dateiansicht öffnet den eingebauten Editor
auf der Arbeitsverzeichnis-Kopie der Datei. Siehe
[Staging und Committen](staging-and-committing.md).

| Taste | Aktion |
|---|---|
| *(tippen)* | Am Cursor einfügen (ersetzt die Markierung) |
| `Enter` | Neue Zeile, mit der Einrückung der aktuellen Zeile |
| `Tab` / `Shift+Tab` | Einrücken / Ausrücken (die ausgewählten Zeilen, wenn es mehrere sind) |
| Pfeile, `Home` / `End` | Bewegen. `Home` geht zum ersten nicht leeren Zeichen, dann zu Spalte 0. `Shift` erweitert die Markierung |
| `Ctrl+Left` / `Ctrl+Right` | Ein Wort bewegen |
| `Ctrl+Home` / `Ctrl+End` | Anfang / Ende der Datei |
| `PageUp` / `PageDown` | Eine Seite nach oben / unten |
| `Ctrl+A` | Alles markieren |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopieren / Ausschneiden / Einfügen |
| `Ctrl+U` / `Ctrl+K` | Bis zum Anfang / Ende der Zeile löschen |
| `Ctrl+Z` | Rückgängig |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Wiederholen |
| `Ctrl+S` | Speichern |
| `Ctrl+F` / `F3` | Suchen. Im Suchfeld: `Enter` / `Down` / `F3` weiter, `Shift+Enter` / `Up` zurück, `Esc` schließt |
| `Alt+M` | Minimap ein- oder ausschalten (**Map**) |
| `Esc` | Schließen. Bei ungespeicherten Änderungen fragt es: **Speichern** (`S` / `Enter`), **Verwerfen** (`D`), **Abbrechen** (`C` / `Esc`) |

`Tab` rückt im Editor ein, verschiebt also nicht den Fokus. Andere `Ctrl`-Tastenfolgen
(`Ctrl+P`, `Ctrl+N`) funktionieren weiterhin.

**Kodierung**, **Zeilenenden** und **Einrückung** in der Statuszeile sind Schaltflächen:
Jede öffnet ein Dropdown (Pfeiltasten oder Mausrad, zum Filtern tippen, `Enter` zum
Wählen, `Esc` zum Schließen). Die Palette (`Ctrl+P`) öffnet dieselben Menüs: *Editor: Kodierung ändern*, *Editor: Mit Kodierung erneut öffnen*, *Editor: In Kodierung umwandeln*, *Editor: Zeilenumbrüche umwandeln*, *Editor: Einrückung ändern*.
Einzelheiten in [Staging und Committen](staging-and-committing.md).

## Git-Aktionen (`Ctrl+G`)

`Ctrl+G` wartet auf eine weitere Taste:

| Taste | Aktion |
|---|---|
| `s` / `u` | Die ausgewählte Datei stagen / unstagen |
| `a` | Alle Änderungen stagen |
| `x` | Alle Änderungen verwerfen (fragt vorher) |
| `c` | Den Commit-Composer öffnen |
| `Enter` | Die staged Änderungen sofort mit der Nachricht des Formulars committen, von überall (siehe [Staging und Committen](staging-and-committing.md#den-commit-schreiben)) |
| `m` | Alle Änderungen mit einer Nachricht stashen, getippt in einem kleinen Dialog (leer ist erlaubt). Bei eingeschalteter KI schreibt seine Schaltfläche **KI-Nachricht** (`Ctrl+G` im Dialog) die Nachricht |
| `Shift+T` | Alle Änderungen stashen (nicht verfolgte Dateien eingeschlossen), ohne Nachricht |
| `o` | Einen Stash poppen: die im Log ausgewählte Stash-Zeile, sonst den letzten Stash |
| `p` | Pull, so wie die Pull-Schaltfläche eingestellt ist (`git.pull_mode`) |
| `Shift+P` | Zum Standard-Remote pushen (setzt beim ersten Push den Upstream) oder nach dem Remote fragen |
| `Shift+U` | Alle Tags pushen |
| `f` | Jedes Remote fetchen, mit Prune |
| `g` | Den Pull Request des ausgecheckten Branches im Browser öffnen |
| `b` / `t` | Neuer Branch / neuer Tag bei HEAD (siehe [Einen Branch oder Tag benennen](#einen-branch-oder-tag-benennen)) |
| `d` | Den ausgewählten Branch oder Tag löschen (fragt vorher) |
| `e` | Die Nachricht des ausgewählten Commits bearbeiten |
| `Shift+E` / `Shift+A` | Den ausgewählten Commit ändern (alle Felder) / seinen Autor ändern (siehe [Einen Commit ändern](#einen-commit-ändern)) |
| `y` | Den ausgewählten Commit cherry-picken |
| `v` | Den ausgewählten Commit rückgängig machen (Revert) |
| `Shift+R` | Reset (mixed) auf den ausgewählten Commit |
| `r` | Rebase auf den ausgewählten Branch |
| `Shift+W` | Das Flow-Menü öffnen (in einem Repository mit Branching-Workflow) |
| `Shift+H` | Die Werkbank zum Umschreiben des Verlaufs öffnen (nur in Builds mit Verlaufs-Umschreibung) |
| `l` | Das LFS-Menü öffnen. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS alles fetchen / alles auschecken / pullen / alles pushen / bereinigen. Siehe [Git LFS](lfs.md) |
| `i` | ✦ KI: die Commit-Nachricht für die gestagten Änderungen schreiben |
| `w` | ✦ KI: den ausgewählten Commit oder die Änderungen erklären |
| `Shift+C` | ✦ KI: vorschlagen, wie die Änderungen auf Commits verteilt werden |

Die KI-Tasten funktionieren nur, solange die KI an ist (siehe [KI-Unterstützung](ai.md)). `Esc`
bricht eine laufende KI-Anfrage ab.

## Rechter Bereich (Änderungen und Commit-Details)

Ist die WIP-Zeile ausgewählt, zeigt der rechte Bereich die Listen **Unstaged** und **Staged**
und den Commit-Composer. Ist ein Commit ausgewählt, zeigt er die Commit-Details und seine
Dateien.

| Taste | Aktion |
|---|---|
| `j` / `k`, `Up` / `Down` | Bewegen. Am Ende einer Liste geht die Auswahl in die andere weiter |
| `h` / `l`, `Left` / `Right` | Zur Liste Unstaged / Staged gehen |
| `Enter` | Die aktive Liste ein- oder ausklappen |
| `Space` / `Insert` | Die aktuelle Datei zur Auswahl hinzufügen oder sie entfernen |
| `Ctrl+A` | Alle Dateien der Liste auswählen |
| `Ctrl+Space` | Die ausgewählte Datei stagen / unstagen oder eine Ordnerzeile falten |
| `d` | Den Diff der ausgewählten Datei öffnen |
| `m` | `Pfad` (flache Liste) oder `Baum` (Ordner) |
| `a` | `Alle Dateien ansehen`: der ganze Baum, nicht nur die geänderten Dateien |
| `z` | Alle Ordner einklappen oder wieder ausklappen |
| `/` | Die geänderten Dateien eines Commits nach Namen filtern |
| `.`, `Menu`, `Shift+F10` | Das Dateimenü |
| `c` | Das Zusammenfassungsfeld des Commit-Composers fokussieren |
| `Shift+A` | **Vorherigen Commit ändern** ein- oder ausschalten |
| `Ctrl+N ]` / `Ctrl+N [` | Den Block der Commit-Nachricht vergrößern / verkleinern |

Im Commit-Composer:

| Taste | Aktion |
|---|---|
| `Enter` | Zusammenfassung: zur Beschreibung gehen. Beschreibung: neue Zeile |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Committen (`Ctrl+G Enter` tut es von überall) |
| `Alt+R` | Eine Issue-Referenz einfügen |
| `Esc` | Den Composer verlassen |

Das Dateimenü enthält: Stagen oder Unstagen, Änderungen verwerfen, Ignorieren, Datei stashen,
Dateiverlauf, Datei-Blame, Im externen Diff-Tool öffnen, Im externen Editor öffnen, Datei im
Standardprogramm öffnen, Im Ordner anzeigen, Dateipfad kopieren, Patch aus Dateiänderungen
erstellen, Datei bearbeiten und Datei löschen. Bei einer Datei im Konflikt beginnt es mit **Im
Merge-Editor öffnen** und den drei Zeilen **Mit ... lösen**. Bei eingeschaltetem Git LFS hat
es LFS-Zeilen. **Änderungen verwerfen** und **Datei löschen** fragen vorher.

## Fenster-Schaltflächen

Jede Schaltfläche hat ein Tastatur-Gegenstück:

| Schaltfläche | Taste |
|---|---|
| <img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi-Symbol" height="16"> links in der Titelleiste (Über) | `Hilfe: Über GitPoulpi` in der Palette |
| **Öffnen in** in der Titelleiste | `Alt+O`, `Ctrl+Alt+O` |
| `×` rechts in der Titelleiste (beenden) | `q` |
| Die Version in der Titelleiste (Versionshinweise) | `Hilfe: Versionshinweise` in der Palette |
| `⌕ Globale Suche` in der Titelleiste, links vom `⚙` | `Ctrl+Alt+F` |
| `⚙` in der Titelleiste (Einstellungen) | `Ctrl+,` |
| Der Workspace-Chip in der Tab-Leiste | `Alt+W` |
| `+` hinter dem letzten Tab | `Alt+T` |
| Der Tab **Radar** | `Alt+R` |
| Der Tab **Workspaces** | `Alt+J` |
| **Aktivität** in der Statuszeile | `Ctrl+N Shift+A` |
| Symbolleiste: **Rückgängig**, **Wiederholen** | `Ctrl+Z`, `Ctrl+Y` |
| Symbolleiste: **Pull** / sein `▾` | `Ctrl+G p` / `Git: Pull-Optionen...` |
| **Pull aller Branches** (Eintrag des Pull-`▾`) | `Ctrl+G Ctrl+P` |
| Symbolleiste: **Push** / sein `▾` | `Ctrl+G Shift+P` / `Git: Push an Remote...` |
| Symbolleiste: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Symbolleiste: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Symbolleiste: **LFS** (wenn LFS an ist) | `Ctrl+G l` |
| Symbolleiste: **Flow** (wenn das Repository einen Workflow hat) | `Ctrl+G Shift+W` |
| Symbolleiste: **Suchen** | `f` |
| Symbolleiste: der verknüpfte Pull Request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Symbolleiste: die Profilliste (bei zwei oder mehr Profilen) | `Repository: Profil wechseln...` |

Die Symbolleiste ist im [Rundgang durch die Oberfläche](interface-tour.md#symbolleiste)
beschrieben.

## Einstellungen

`Ctrl+,`, `Ctrl+N ,`, das Zahnrad `⚙` oder `App: Einstellungen...` öffnet den
Einstellungsbildschirm.

| Taste | Aktion |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Zwischen Seitenliste und Optionen wechseln |
| `j` / `k`, `Up` / `Down` | Die Auswahl bewegen |
| `Enter` / `Space` | Eine Seite öffnen, ein Kontrollkästchen umschalten, ein Feld bearbeiten, eine Dropdownliste öffnen oder eine Schaltfläche drücken |
| `/`, `Ctrl+F` | Alle Optionen durchsuchen |
| `Alt+Left` / `Alt+Right` | Untertab wechseln (Allgemein / die Liste) |
| `Esc` / `q` | Die Einstellungen schließen. Beim Bearbeiten eines Feldes bricht `Esc` nur die Bearbeitung ab |
| `?` | Die Hilfe über dem Einstellungsbildschirm öffnen |

Siehe [Einstellungen](settings.md).

## Textfelder

Alle Textfelder teilen dieselben Tasten: Filter, Suchfelder, die Befehlspalette, Eingaben, der
Commit-Composer, das Feld der WIP-Zeile, Dialogfelder und Felder der Einstellungen. Solange ein
Feld die Tastatur hat, gewinnen diese Tasten gegen die Bereichs-Tastenkürzel.

| Taste | Aktion |
|---|---|
| ein Zeichen | Am Cursor tippen (ersetzt die Markierung) |
| `Left` / `Right` | Ein Zeichen bewegen (mit Markierung: zu ihrem Anfang / Ende gehen) |
| `Ctrl+Left` / `Ctrl+Right` | Ein Wort bewegen |
| `Home` / `End` | Anfang / Ende der Zeile |
| `Shift` + eine Bewegung oben | Die Markierung erweitern oder verkleinern |
| `Up` / `Down` (nur Beschreibung) | Vorherige / nächste Zeile |
| `Ctrl+Home` / `Ctrl+End` (nur Beschreibung) | Anfang / Ende des Textes |
| `Ctrl+A` | Alles markieren |
| `Backspace` / `Delete` | Das Zeichen vor / nach dem Cursor oder die Markierung löschen |
| `Ctrl+U` | Vom Cursor bis zum Zeilenanfang löschen (oder die Markierung) |
| `Ctrl+K` | Vom Cursor bis zum Zeilenende löschen (oder die Markierung) |
| `Ctrl+C` / `Ctrl+X` | Die Markierung kopieren / ausschneiden (nicht in einem Passwortfeld) |
| Einfügen (das eigene Einfügen des Terminals) | Am Cursor einfügen und die Markierung ersetzen |

In der Commit-Beschreibung tippt `Enter` eine neue Zeile, und `Ctrl+Enter` committet. Nicht alle
Terminals senden `Ctrl+Enter` gleich, daher committen auch `Alt+Enter` und `Ctrl+J`.

Manche Konsolen senden `Ctrl+Buchstabe` als Steuerzeichen (`Ctrl+A` als `0x01`). GitPoulpi
versteht beide Formen. `AltGr`-Zeichen (`@`, `#`, `€` auf Schweizer oder französischen
Tastaturen) werden getippt.

## Tooltips

Ein Tooltip erscheint, wenn der Zeiger etwa 0,6 s auf einem Ziel bleibt (eine Log-Zelle, eine
Schaltfläche der Symbolleiste, der verknüpfte Pull Request). Ein Tastendruck, ein Klick, ein
Scrollen, eine Größenänderung, ein geöffnetes Popup oder Menü oder das Verlassen des Ziels durch
den Zeiger blendet ihn aus. Danach kommt er erst wieder, wenn sich der Zeiger erneut bewegt.
