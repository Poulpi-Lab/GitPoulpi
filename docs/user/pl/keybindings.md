[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · **Polski** · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Skróty klawiszowe

Ta strona wymienia klawisze **domyślne**. Większość z nich można zmienić.
Aplikacja zawsze pokazuje klawisze, które obowiązują: w palecie poleceń, w
pomocy `?`, na paskach stanu, w menu kontekstowych i w podpowiedziach przycisków. Gdy
przeniesiesz klawisz, te miejsca pokazują nowy. O myszy przeczytasz w
[Myszy i gestach](mouse-and-gestures.md).

## Zmiana przypisań

Otwórz **Ustawienia > Klawiatura i mysz**. Stronę opisują
[Ustawienia](settings.md#klawiatura-i-mysz). Zmiany działają od razu i są
zapisywane w tabeli `[keybindings]` pliku `config.toml`. Tę tabelę można też
edytować ręcznie:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux i GNU screen często używają Ctrl+B i Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Składnię, reguły konfliktów i ostrzeżenia opisuje
[Opis konfiguracji](config-reference.md) w części `[keybindings]`. W skrócie:

- Błąd jest zgłaszany na pasku stanu, a klawisz domyślny zostaje.
- Dwie akcje na jednym skrócie w tym samym kontekście są odrzucane.
- Prefiks wymaga `Ctrl` lub `Alt`.
- Skrót panelu, który przesłania skrót globalny, daje tylko ostrzeżenie.

Poniższa tabela wymienia każdy klawisz, który można zmienić. Niektóre klawisze
są stałe i nie można ich zmienić: ruch po listach (`j` `k` `h` `l`, strzałki,
`PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, cyfry (prefiks
licznika), klawisze edycji tekstu oraz klawisze `Enter` / `Tab` / strzałek w
oknach. Paleta ma `Aplikacja: Skróty klawiszowe...` i `Skróty: Resetuj wszystkie...`.

## Domyślne skróty klawiszowe

Każdy klawisz, który można zmienić, z identyfikatorem do użycia w
`[keybindings.<context>]` i jego wartością domyślną. Tabela jest generowana z
aplikacji (test dba o jej aktualność). Klawisze po prefiksie są pokazane razem z
prefiksem (`Ctrl+G s`).

<!-- keymap:begin -->
| Kontekst | Id akcji | Co robi | Domyślne klawisze |
|---|---|---|---|
| `global` | `git_prefix` | Prefiks akcji Git | `Ctrl+G` |
| `global` | `nav_prefix` | Prefiks nawigacji | `Ctrl+N` |
| `global` | `palette` | Paleta poleceń | `Ctrl+P` |
| `global` | `settings` | Ustawienia | `Ctrl+,` |
| `global` | `home` | Home (zarządzanie repozytoriami) | `Ctrl+O` |
| `global` | `clone` | Klonuj repozytorium | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nowa karta | `Alt+T` |
| `global` | `workspace_switch` | Zmień obszar roboczy | `Alt+W` |
| `global` | `radar` | Otwórz Radar | `Alt+R` |
| `global` | `workspaces` | Otwórz obszary robocze | `Alt+J` |
| `global` | `tab_menu` | Menu karty | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Otwórz w (lista) | `Alt+O` |
| `global` | `tab_open_in_default` | Otwórz w domyślnym | `Ctrl+Alt+O` |
| `global` | `close_tab` | Zamknij kartę | `Ctrl+W` |
| `global` | `next_tab` | Następna karta | `Ctrl+PageDown` |
| `global` | `prev_tab` | Poprzednia karta | `Ctrl+PageUp` |
| `global` | `help` | Pomoc | `?` |
| `global` | `quit` | Zakończ | `q` |
| `global` | `refresh` | Odśwież | `r` |
| `global` | `refresh_all` | Odśwież (fetch, pull requesty, zgłoszenia) | `F5` |
| `global` | `find_in_log` | Szukaj w grafie commitów | `f` |
| `global` | `workspace_search` | Szukaj w obszarze roboczym | `Ctrl+Alt+F` |
| `global` | `select_tab` | Wybierz kartę | `t` |
| `global` | `undo` | Cofnij ostatnią operację | `Ctrl+Z` |
| `global` | `redo` | Ponów cofniętą operację | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Otwórz historię cofania | `Alt+Z` |
| `global` | `commit_column` | Pokaż lub ukryj kolumnę commitów | `Alt+C` |
| `global` | `undo_offer` | Przywróć gałąź tam, gdzie zostawiła ją zablokowana akcja | `Alt+Shift+Z` |
| `global` | `context_menu` | Menu kontekstowe | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Napisz opis commita | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Zrób commit zmian ze stage od razu | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: Podziel zmiany na commity | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: Wyjaśnij zmiany | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash z opisem | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Napisz opis commita | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage wszystkie zmiany | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Odrzuć wszystkie zmiany | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage zaznaczony plik | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage zaznaczony plik | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Otwórz powiązany pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push wszystkich tagów | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch wszystkich zdalnych | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nowy Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nowa gałąź | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash zmian | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick zaznaczonego commita | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert zaznaczonego commita | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Edytuj opis commita | `Ctrl+G e` |
| `git_prefix` | `amend` | Popraw wybrany commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Popraw autora wybranego commita | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) do zaznaczonego commita | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase na zaznaczoną gałąź | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop stasha | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Usuń zaznaczony ref lub drop stasha | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Otwórz menu LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Otwórz menu Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch wszystkich obiektów | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout wszystkich plików | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull obiektów | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push wszystkich obiektów | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Wyczyść lokalne obiekty | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Otwórz warsztat przepisywania historii | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Zwiń lub przywróć panel Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Zwiń lub przywróć panel Status | `Ctrl+N l` |
| `nav_prefix` | `settings` | Otwórz ustawienia | `Ctrl+N ,` |
| `nav_prefix` | `about` | Otwórz „O programie” | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Otwórz Dziennik aktywności | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Otwórz globalny dziennik aktywności | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Pokaż zadania w tle | `Ctrl+N Shift+J` |
| `nav_prefix` | `help` | Przełącz pomoc | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Wybierz kartę | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Zmień motyw | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Zmień język | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Zmień motyw kolorów grafu | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Przełącz zabarwienie wierszy grafu | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Powiększ sekcję Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Pomniejsz sekcję Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Powiększ blok opisu commita | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Pomniejsz blok opisu commita | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Poszerz panel Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Zwęź panel Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Poszerz panel Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Zwęź panel Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: Pokaż oczekujący widok | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: Wstecz (lub odrzuć oczekujący widok) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filtruj refy po nazwie | `/` |
| `refs` | `solo` | Solo dla zaznaczenia | `o` |
| `refs` | `clear_solo` | Zakończ solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Przełącz widok hierarchii gałęzi | `y` |
| `refs` | `show_all` | Pokaż wszystkie ukryte refy | `Shift+H` |
| `refs` | `reveal` | Pokaż w grafie | `g` |
| `refs` | `mark_drop` | Oznacz jako źródło upuszczenia | `m` |
| `refs` | `drop_marked` | Upuść tutaj oznaczony ref | `Shift+M` |
| `refs` | `expand_all` | Rozwiń wszystkie foldery | `=` |
| `refs` | `collapse_all` | Zwiń wszystkie foldery | `-` |
| `refs` | `expand_below` | Rozwiń folder i jego podfoldery | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Zwiń folder i jego podfoldery | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Pliki: Ścieżka lub drzewo | `m` |
| `log` | `view_all_files` | Pliki: Pokaż wszystkie pliki | `a` |
| `log` | `compare_selection` | Porównaj zaznaczone wiersze | `v` |
| `log` | `compare_swap` | Zamień końce porównania | `x` |
| `log` | `compare_next` | Przejdź do następnego porównywanego wiersza | `n` |
| `log` | `pick_column` | Wybierz kolumnę do przesunięcia lub zmiany szerokości | `Shift+C` |
| `log` | `scroll_left` | Przewiń kolumny w lewo | `Shift+Left` |
| `log` | `scroll_right` | Przewiń kolumny w prawo | `Shift+Right` |
| `log` | `copy_mermaid` | Kopiuj commity jako gitGraph Mermaid | `Shift+Y` |
| `log_columns` | `next` | Wybierz następną kolumnę | `Tab` |
| `log_columns` | `prev` | Wybierz poprzednią kolumnę | `Shift+BackTab` |
| `log_columns` | `move_left` | Przesuń kolumnę w lewo | `<` |
| `log_columns` | `move_right` | Przesuń kolumnę w prawo | `>` |
| `log_columns` | `wider` | Poszerz kolumnę | `]` |
| `log_columns` | `narrower` | Zwęź kolumnę | `[` |
| `log_columns` | `reset_width` | Zresetuj szerokości kolumn | `0` |
| `log_columns` | `fit` | Dopasuj kolumnę do zawartości | `=` |
| `log_columns` | `fit_all` | Dopasuj wszystkie kolumny do zawartości | `+` |
| `log_columns` | `layout` | Przewijanie w poziomie lub dopasowanie do panelu | `s` |
| `log_files` | `collapse_all` | Zwiń lub rozwiń wszystkie foldery | `z` |
| `log_files` | `filter` | Filtruj zmienione pliki | `/` |
| `log_files` | `expand_all` | Rozwiń wszystkie foldery | `=` |
| `log_files` | `collapse_all_folders` | Zwiń wszystkie foldery | `-` |
| `log_files` | `expand_below` | Rozwiń folder i jego podfoldery | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Zwiń folder i jego podfoldery | `_`, `Shift+Left` |
| `status` | `compose` | Przejdź do opisu commita | `c` |
| `status` | `toggle_file_mode` | Pliki: Ścieżka lub drzewo | `m` |
| `status` | `view_all_files` | Pliki: Pokaż wszystkie pliki | `a` |
| `status` | `amend` | Przełącz amend | `Shift+A` |
| `status` | `skip_hooks` | Przełącz pomijanie hooków | `Shift+H` |
| `status` | `collapse_all` | Zwiń lub rozwiń wszystkie foldery | `z` |
| `status` | `stage_toggle` | Stage lub unstage zaznaczony wiersz | `Ctrl+Space` |
| `status` | `open_diff` | Otwórz diff zaznaczonego pliku | `d` |
| `status` | `expand_all` | Rozwiń wszystkie foldery | `=` |
| `status` | `collapse_all_folders` | Zwiń wszystkie foldery | `-` |
| `status` | `expand_below` | Rozwiń folder i jego podfoldery | `*`, `Shift+Right` |
| `status` | `collapse_below` | Zwiń folder i jego podfoldery | `_`, `Shift+Left` |
| `home` | `favorite` | Dodaj repozytorium do ulubionych lub usuń | `f`, `*` |
| `home` | `close_repo` | Zamknij, usuń lub zdejmij gwiazdkę z wiersza | `x`, `Delete` |
| `home` | `browse` | Przeglądaj w poszukiwaniu repozytorium | `b` |
| `home` | `clone` | Klonuj repozytorium | `c` |
| `home` | `init` | Zainicjuj repozytorium | `i` |
| `home` | `collapse_all` | Zwiń wszystkie sekcje | `-` |
| `home` | `expand_all` | Rozwiń wszystkie sekcje | `+`, `=` |
| `home` | `search` | Szukaj repozytoriów | `/` |
| `home` | `workspace_new` | Nowy obszar roboczy | `n` |
| `home` | `workspace_edit` | Edytuj obszar roboczy | `e` |
| `home` | `workspace_from_tabs` | Nowy obszar roboczy z otwartych kart | `a` |
| `home` | `workspace_fetch` | Fetch wszystkich w obszarze roboczym | `Shift+F` |
| `home` | `workspace_pull` | Pull wszystkich w obszarze roboczym | `Shift+P` |
| `home` | `workspace_clone_missing` | Klonuj brakujące repozytoria | `m` |
| `home` | `workspace_locate` | Wskaż brakujące repozytorium | `l` |
| `refs` | `hosting_filter` | Pull requesty / zgłoszenia: Wybierz filtr | `v` |
| `refs` | `hosting_edit` | Pull request: Edytuj na hoście | `e` |
| `refs` | `new_pull_request` | Pull requesty: Utwórz | `n` |
| `refs` | `add_remote` | Zdalne: Dodaj zdalne | `+` |
| `diff` | `close` | Zamknij diff | `q` |
| `diff` | `edit` | Edytuj w drzewie roboczym | `e` |
| `diff` | `lfs_raw` | LFS: Karta lub surowy wskaźnik | `r` |
| `diff` | `copy` | Kopiuj zaznaczenie lub linię | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage lub unstage hunk | `Space` |
| `diff` | `stage_lines` | Stage lub unstage zaznaczone linie | `Enter` |
| `diff` | `discard_hunk` | Odrzuć hunk | `x` |
| `diff` | `discard_lines` | Odrzuć zaznaczone linie | `Shift+X` |
| `diff` | `select_lines` | Zacznij lub porzuć zaznaczanie linii | `v` |
| `diff` | `next_hunk` | Następny hunk | `n` |
| `diff` | `prev_hunk` | Poprzedni hunk | `p` |
| `diff` | `toggle_layout` | Układ: W linii lub obok siebie | `m` |
| `diff` | `toggle_hunks_only` | Tylko hunki lub cały plik | `Shift+H` |
| `diff` | `toggle_minimap` | Przełącz minimapę | `Shift+M` |
| `diff` | `choose_encoding` | Wybierz kodowanie | `Shift+C` |
| `diff` | `toggle_wrap` | Przełącz zawijanie linii | `w` |
| `diff` | `toggle_whitespace` | Pokaż lub ukryj zmiany tylko w białych znakach | `s` |
| `diff` | `merge_editor` | Otwórz edytor merge (plik z konfliktem) | `Shift+E` |
| `file_insight` | `close` | Zamknij widok pliku | `q` |
| `file_insight` | `edit` | Edytuj w drzewie roboczym | `e` |
| `file_insight` | `copy` | Kopiuj zaznaczenie lub wiersz | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Skocz do ostatniego wiersza | `Shift+G` |
| `file_insight` | `open_commit` | Przejdź do commita tego wiersza | `Enter` |
| `history` | `show_in_log` | Pokaż w Log | `Enter` |
| `history` | `copy_sha` | Kopiuj SHA commita | `y` |
| `history` | `copy_summary` | Kopiuj podsumowanie | `Shift+Y` |
| `history` | `show_diff` | Pokaż diff tego commita | `d`, `Alt+1` |
| `history` | `open_file` | Otwórz plik z tego commita | `o`, `Alt+2` |
| `history` | `blame` | Blame dla tego commita | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Porównaj z drzewem roboczym | `c` |
| `history` | `restore` | Przywróć tę wersję | `Shift+R` |
| `history` | `range_up` | Rozszerz zaznaczenie w górę | `Shift+Up` |
| `history` | `range_down` | Rozszerz zaznaczenie w dół | `Shift+Down` |
| `history` | `switch_focus` | Przełącz między listą a diffem | `Tab` |
| `history` | `widen_list` | Poszerz listę | `Alt+Right` |
| `history` | `narrow_list` | Zwęź listę | `Alt+Left` |
| `history` | `reset_split` | Zresetuj szerokość listy | `Alt+0` |
| `commit_column` | `scroll_left` | Przewiń kolumnę w lewo | `Shift+Left` |
| `commit_column` | `scroll_right` | Przewiń kolumnę w prawo | `Shift+Right` |
| `commit_column` | `next_part` | Wybierz następną część | `Tab` |
| `commit_column` | `move_left` | Przesuń część w lewo | `<` |
| `commit_column` | `move_right` | Przesuń część w prawo | `>` |
| `commit_column` | `wider_part` | Poszerz część | `]` |
| `commit_column` | `narrower_part` | Zwęź część | `[` |
| `commit_column` | `reset_widths` | Zresetuj szerokości części | `0` |
| `file_insight` | `blame_card` | Blame: Pokaż lub ukryj kartę szczegółów | `i` |
| `file_insight` | `blame_next` | Blame: Następny hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Poprzedni hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Następny hunk z tego commita | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Poprzedni hunk z tego commita | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Przejdź do linii | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Kopiuj SHA commita | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Pokaż diff commita dla pliku | `d` |
| `file_insight` | `blame_parent` | Blame: Blame na rodzicu commita | `b` |
| `file_insight` | `blame_history` | Blame: Otwórz historię pliku | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Poszerz kolumnę adnotacji | `>` |
| `file_insight` | `blame_narrower` | Blame: Zwęź kolumnę adnotacji | `<` |
| `image` | `zoom_in` | Powiększ obraz | `+`, `=` |
| `image` | `zoom_out` | Pomniejsz obraz | `-` |
| `image` | `fit` | Dopasuj obraz do widoku | `0` |
| `image` | `actual_size` | Pokaż obraz w rzeczywistym rozmiarze | `1` |
| `image` | `flip_side` | Pokaż drugą stronę (przed / po) | `Tab` |
| `image` | `widen_before` | Poszerz panel Przed | `Alt+Right` |
| `image` | `widen_after` | Poszerz panel Po | `Alt+Left` |
| `image` | `reset_split` | Przywróć podział przed / po | `Alt+0` |
| `image` | `edit_in` | Otwórz za pomocą... | `Shift+O` |
| `image` | `edit_default` | Otwórz w aplikacji domyślnej | `o` |
| `editor` | `save` | Zapisz | `Ctrl+S` |
| `editor` | `undo` | Cofnij | `Ctrl+Z` |
| `editor` | `redo` | Ponów | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Szukaj | `Ctrl+F` |
| `editor` | `find_next` | Szukaj dalej (Shift: wstecz) | `F3` |
| `editor` | `paste` | Wklej | `Ctrl+V` |
| `editor` | `toggle_minimap` | Przełącz minimapę | `Alt+M` |
| `editor_prompt` | `save` | Niezapisane zmiany: Zapisz i zamknij | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Niezapisane zmiany: Odrzuć i zamknij | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Niezapisane zmiany: Edytuj dalej | `c`, `Shift+C` |
| `merge` | `next_conflict` | Następny konflikt | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Poprzedni konflikt | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Następny plik z konfliktem | `Alt+PageDown` |
| `merge` | `prev_file` | Poprzedni plik z konfliktem | `Alt+PageUp` |
| `merge` | `next_pane` | Przejdź do następnego panelu | `F6` |
| `merge` | `prev_pane` | Przejdź do poprzedniego panelu | `Shift+F6` |
| `merge` | `toggle_panes` | Pokaż lub ukryj panele boczne (Refs i Status) | `F9` |
| `merge` | `toggle_base` | Pokaż lub ukryj panel Bazy | `Alt+D` |
| `merge` | `toggle_full_file` | Przełącz diff całego pliku | `Alt+F` |
| `merge` | `undo` | Cofnij (wybory i pisanie) | `Ctrl+Z` |
| `merge` | `redo` | Ponów (wybory i pisanie) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Poszerz panel bieżący (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Poszerz panel przychodzący (B) | `Alt+Left` |
| `merge` | `reset_panes` | Przywróć rozmiary paneli | `Alt+0` |
| `merge` | `save` | Zapisz bez oznaczania jako rozwiązany | `Alt+S` |
| `merge` | `save_resolve` | Zapisz i oznacz jako rozwiązany | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Zasugeruj rozwiązanie (AI) | `Alt+I` |
| `merge` | `unpick` | Cofnij wybór tego konfliktu (znów nierozwiązany) | `Alt+U` |
| `merge` | `reset_all` | Przywróć cały plik do pierwotnych konfliktów | `Alt+R` |
| `merge` | `focus_a` | Przejdź do panelu bieżącego (A) | `Alt+1` |
| `merge` | `focus_b` | Przejdź do panelu przychodzącego (B) | `Alt+2` |
| `merge` | `focus_output` | Przejdź do panelu Wyniku | `Alt+3` |
| `merge` | `focus_base` | Przejdź do panelu Bazy | `Alt+4` |
| `merge_sides` | `toggle_line` | Wybierz lub porzuć linię | `Space` |
| `merge_sides` | `toggle_chunk` | Wybierz lub porzuć cały blok | `Enter` |
| `merge_sides` | `take_a` | Tylko bieżący (A) | `a` |
| `merge_sides` | `take_b` | Tylko przychodzący (B) | `b` |
| `merge_sides` | `take_both` | Oba, A potem B | `c` |
| `merge_sides` | `take_both_reversed` | Oba, B potem A | `Shift+C` |
| `merge_sides` | `clear` | Nic dla tego konfliktu | `x` |
| `merge_sides` | `unpick` | Cofnij wybór tego konfliktu (znów nierozwiązany) | `u` |
| `merge_sides` | `reset_all` | Przywróć cały plik do pierwotnych konfliktów | `Shift+U` |
| `merge_sides` | `swap_order` | Zamień kolejność obu stron | `s` |
| `merge_sides` | `next_conflict` | Następny konflikt | `n` |
| `merge_sides` | `prev_conflict` | Poprzedni konflikt | `Shift+N` |
| `merge_sides` | `take_all_a` | Cały bieżący (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Cały przychodzący (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Pokaż lub ukryj panel Bazy | `d` |
| `merge_sides` | `scroll_left` | Przewiń panele w lewo | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Przewiń panele w prawo | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Przełącz diff całego pliku | `f` |
| `merge_sides` | `cycle_height` | Przełącz wysokość Wyniku | `z` |
| `merge_card` | `take_a` | Zachowaj A (bieżący) bez zmian | `a` |
| `merge_card` | `take_b` | Użyj B (przychodzący) bez zmian | `b` |
| `merge_card` | `take_base` | Użyj Bazy (wspólny przodek) | `o` |
| `merge_card` | `keep` | Zachowaj pozostały plik | `k` |
| `merge_card` | `delete` | Zaakceptuj usunięcie | `Shift+D` |
| `merge_card` | `external_tool` | Otwórz w zewnętrznym narzędziu merge | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (zatrzymaj, by zmienić) | `e` |
| `rebase` | `autosquash` | Przełącz autosquash | `a` |
| `rebase` | `start` | Rozpocznij plan | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Resetuj plan | `Ctrl+R` |
| `rebase` | `split` | Podziel commit z wiersza | `t` |
| `rebase` | `move_up` | Przesuń wiersz w górę | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Przesuń wiersz w dół | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Utwórz commity | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Przełącz zaznaczanie linii | `v` |
| `split` | `move_up` | Przesuń commit w górę | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Przesuń commit w dół | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Zresetuj podział | `Shift+R` |
| `rewrite` | `preview` | Podgląd przepisania | `F5` |
| `rewrite` | `apply` | Zastosuj przepisanie | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Anuluj podgląd | `Ctrl+X` |
| `rewrite` | `add_rule` | Dodaj regułę | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Włącz lub wyłącz regułę | `Space` |
| `rewrite` | `move_up` | Przenieś regułę w górę | `Alt+Up` |
| `rewrite` | `move_down` | Przenieś regułę w dół | `Alt+Down` |
| `rewrite` | `delete_rule` | Usuń regułę | `d`, `Delete` |
| `rewrite` | `restore_rule` | Przywróć usuniętą regułę | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplikuj regułę | `Ctrl+D` |
| `rewrite` | `next_tab` | Następna karta podglądu | `Alt+Right` |
| `rewrite` | `prev_tab` | Poprzednia karta podglądu | `Alt+Left` |
| `rewrite` | `save_shared` | Zapisz przepis w repozytorium | `Alt+S` |
| `rewrite` | `save_mine` | Zapisz przepis tylko dla mnie | `Alt+M` |
| `rewrite` | `load` | Wczytaj przepis | `Alt+L` |
| `rewrite` | `name` | Zmień nazwę przepisu | `F2` |
| `rewrite` | `protected_all` | Uwzględnij wszystkie chronione gałęzie | `Alt+A` |
| `rewrite` | `protected_none` | Pomiń wszystkie chronione gałęzie | `Alt+U` |
| `rewrite` | `check_tools` | Sprawdź narzędzia ponownie | `F6` |
| `rewrite` | `analyze` | Analizuj rozmiary historii | `F7` |
| `rewrite` | `host_howto` | Wyjaśnij ochronę gałęzi po stronie hosta | `Alt+H` |
| `rewrite` | `ai_rules` | Utwórz reguły przepisania z AI | `Alt+I` |
| `reword` | `apply` | Zastosuj nowy opis | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Zmień historię głębiej | `Alt+H` |
| `composer` | `commit` | Commit z kompozytora | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Wstaw odwołanie do zgłoszenia | `Alt+R` |
| `dialog` | `browse` | Przeglądaj w poszukiwaniu folderu | `Ctrl+B` |
| `dialog` | `sources` | Przejdź do źródeł klonowania | `Alt+S` |
| `dialog` | `submit` | Wyślij formularz (pull request) | `Ctrl+S` |
| `dialog` | `web` | Otwórz formularz www dostawcy | `Alt+W` |
| `folder_picker` | `choose` | Wybierz folder | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nowy folder | `Alt+N` |
| `folder_picker` | `filter` | Filtruj wpisy folderu | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Pokaż lub ukryj foldery sąsiednie | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Pokaż lub ukryj ukryte foldery | `Alt+H` |
| `preferences` | `search` | Szukaj ustawień | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Dodaj element listy | `Insert` |
| `preferences` | `list_duplicate` | Duplikuj zaznaczony element listy | `Ctrl+D` |
| `preferences` | `list_remove` | Usuń zaznaczony element listy | `Delete` |
| `preferences` | `list_move_up` | Przesuń zaznaczony element listy w górę | `Alt+Up` |
| `preferences` | `list_move_down` | Przesuń zaznaczony element listy w dół | `Alt+Down` |
| `preferences` | `prev_tab` | Poprzednia karta strony ustawień | `Alt+Left` |
| `preferences` | `next_tab` | Następna karta strony ustawień | `Alt+Right` |
| `ai_popup` | `copy` | Kopiuj wynik | `c` |
| `integrations` | `add` | Dodaj profil | `a` |
| `integrations` | `edit` | Edytuj zaznaczony profil | `e` |
| `integrations` | `open_token` | Otwórz stronę tokenu | `o` |
| `integrations` | `remove` | Usuń zaznaczony profil | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Wygeneruj opis stasha | `Ctrl+G` |
<!-- keymap:end -->

### Klawisze, których terminal nie może wysłać, i protokół klawiatury kitty

Starszy terminal nie potrafi wysłać niektórych skrótów: `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` i `Ctrl+<cyfra>`. Nie wysyła nic albo inny
klawisz: na przykład `Ctrl+Shift+N` dociera jako `Ctrl+N`, `Ctrl+I` jako `Tab`,
a `Ctrl+M` jako `Enter`. Strona Ustawień ostrzega o takim przypisaniu („ten
terminal nie wysyła ...”).

Terminale z **protokołem klawiatury kitty** potrafią wysłać te skróty. Przykłady:
kitty, WezTerm, Ghostty, foot, Alacritty 0.13 i nowszy oraz tmux lub herdr, gdy
przepuszczają protokół. GitPoulpi domyślnie używa tego protokołu:

- Przy starcie GitPoulpi pyta terminal i czeka na odpowiedź co najwyżej około
  100 ms. Jeśli terminal nie odpowie, zostają klawisze starszego trybu.
- Jeśli terminal odpowie, GitPoulpi włącza protokół. Wyłącza go z powrotem przy
  wyjściu, błędzie lub awarii.
- Ostrzeżenia na stronie Ustawień podążają za protokołem: gdy jest włączony,
  powyższe skróty da się wysłać.
- **Ustawienia > Klawiatura i mysz > Protokół klawiatury Kitty** (`ui.kitty_keyboard`)
  ustawione na `off` nigdy nie zmienia trybu klawiatury terminala. Użyj tego,
  jeśli terminal lub multiplekser działa nieprawidłowo. Potrzebny jest restart.

Konsole Windows same raportują modyfikatory i nie potrzebują protokołu.

## Paleta poleceń (`Ctrl+P`)

Paleta wymienia **każde** polecenie aplikacji, więc sięga wszystkiego bez
skrótu. Pełna lista jest [niżej](#każde-polecenie-palety).

| Klawisz | Działanie |
|---|---|
| `Ctrl+P` | Otwiera paletę |
| *(wpisywanie)* | Filtruje listę. Najpierw dokładne dopasowanie po prefiksie `Kategoria: `, potem dopasowanie gdziekolwiek w etykiecie, potem dopasowanie rozmyte |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Przesuwają podświetlenie (zob. [Filtrowalne listy w oknach](#filtrowalne-listy-w-oknach)) |
| `Enter` albo dwukrotne kliknięcie | Uruchamia podświetlone polecenie |
| `Esc` | Zamyka |

`j` i `k` są wpisywane do filtra. Nie przesuwają podświetlenia.

Każdy wiersz pokazuje swój skrót po prawej. Polecenie, które teraz nie ma
zastosowania (na przykład nie zaznaczono refa), zostaje na liście, wyszarzone,
a powód widać pod listą. Etykiety zaczynają się od kategorii, takiej jak
`Nawigacja:`, `Refs:`, `Git:`, `Widok:`, `Diff:` lub `Aplikacja:`. Wpisz
kategorię, by zobaczyć całą jej grupę.

### Klawisz dla dowolnego polecenia

Każdy wiersz palety kończy się przyciskiem klawisza (`[k]`), a `Alt+K` robi to samo dla zaznaczonego wiersza: otwiera **Ustawienia > Klawiatura i mysz** przefiltrowane po skrócie tego polecenia, gotowe do zmiany przypisania. Polecenie bez własnego skrótu dostaje go tutaj: jest wymienione w **Polecenia palety**, domyślnie bez klawisza, a przypisany klawisz uruchamia polecenie dokładnie tak, jak jego wybranie w palecie (polecenie, którego teraz nie można uruchomić, podaje powód). Wiersz pokazuje wtedy twój klawisz.

W `config.toml` jest to tabela `[keybindings.palette]`, nazwana od polecenia (`toggle_minimap = "Ctrl+Alt+M"`). Klawisz kolidujący z inną akcją jest pokazany na czerwono, a `c` w jego wierszu wypisuje wszystkie akcje, z którymi koliduje.

Pasek podpowiedzi pod listą podaje ten klawisz (domyślnie `Alt+K`). To akcja `palette.configure_shortcut`, więc można ją przypisać na nowo jak każdą inną.

### Każde polecenie palety

Generowane z aplikacji (test dba o aktualność). Polecenia AI pojawiają się
tylko przy włączonym AI; pozostałe są wyszarzone, z powodem, gdy nie mają
zastosowania. Klawisze są domyślne.

<!-- palette:begin -->
| Polecenie | Klawisz domyślny |
|---|---|
| Nawigacja: Przejdź do panelu Refs | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do panelu Log | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do panelu Status | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do filtra Refs | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Local | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Remote | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Worktrees | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Submodules | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Stashes | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Refs > Tags | `Ctrl+N <badge>` |
| Panel boczny: Pokaż/ukryj sekcje... |  |
| Panel boczny: Przywróć domyślne sekcje |  |
| Panel boczny: Użyj tych sekcji we wszystkich repozytoriach |  |
| Panel boczny: Pokaż/ukryj gałęzie lokalne |  |
| Panel boczny: Pokaż/ukryj zdalne |  |
| Panel boczny: Pokaż/ukryj worktree |  |
| Panel boczny: Pokaż/ukryj Stashe |  |
| Panel boczny: Pokaż/ukryj Tagi |  |
| Panel boczny: Pokaż/ukryj Pull requesty |  |
| Panel boczny: Pokaż/ukryj zgłoszenia |  |
| Panel boczny: Pokaż/ukryj submoduły |  |
| Nawigacja: Przejdź do Status > Unstaged | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do Status > Staged | `Ctrl+N <badge>` |
| Nawigacja: Przejdź do następnego panelu | `Tab` |
| Nawigacja: Przejdź do poprzedniego panelu | `Shift+Tab` |
| Nawigacja: Szukaj commitów... | `f` |
| Nawigacja: Zwiń/rozwiń panel Refs | `Ctrl+N h` |
| Nawigacja: Poszerz panel Refs | `Ctrl+N Right` |
| Nawigacja: Zwęź panel Refs | `Ctrl+N Left` |
| Nawigacja: Poszerz panel Status | `Ctrl+N Shift+Left` |
| Nawigacja: Zwęź panel Status | `Ctrl+N Shift+Right` |
| Nawigacja: Zwiń/rozwiń panel Status | `Ctrl+N l` |
| Nawigacja: Powiększ sekcję Refs | `Ctrl+N +` |
| Nawigacja: Pomniejsz sekcję Refs | `Ctrl+N -` |
| Nawigacja: Resetuj rozmiary sekcji Refs |  |
| Nawigacja: Resetuj układ |  |
| Szczegóły commita: Powiększ opis | `Ctrl+N ]` |
| Szczegóły commita: Pomniejsz opis | `Ctrl+N [` |
| Szczegóły commita: Resetuj rozmiar opisu |  |
| Szczegóły commita: Przewiń opis w dół |  |
| Szczegóły commita: Przewiń opis w górę |  |
| Pliki: Przełącz sortowanie plików commita (A-Z / Z-A) |  |
| Refs: Checkout zaznaczonego refu | `dwuklik` |
| Refs: Kopiuj nazwę refu |  |
| Refs: Kopiuj pełny ref (refs/heads/...) |  |
| Refs: Kopiuj SHA celu refu |  |
| Refs: Merge zaznaczonej gałęzi do bieżącej |  |
| Refs: Rebase bieżącej gałęzi na zaznaczoną | `Ctrl+G r` |
| Refs: Upuść zaznaczony ref na bieżącą gałąź... |  |
| Refs: Oznacz jako źródło upuszczenia | `m` |
| Refs: Upuść tutaj oznaczony ref... | `M` |
| Refs: Zmień nazwę zaznaczonej gałęzi... |  |
| Refs: Fast-forward zaznaczonej gałęzi do bieżącej |  |
| Refs: Fast-forward bieżącej gałęzi do zaznaczonej |  |
| Refs: Utwórz worktree z zaznaczonej gałęzi lub tagu... |  |
| Refs: Reflog zaznaczonej gałęzi... |  |
| Refs: Rozpocznij pull request z zaznaczonej gałęzi... |  |
| Refs: Otwórz worktree w karcie | `Enter / dwuklik` |
| Refs: Aktualizuj submoduł (zaznaczony) |  |
| Refs: Inicjuj submoduł (zaznaczony) |  |
| Refs: Otwórz submoduł w karcie | `Enter / dwuklik` |
| Git: Aktualizuj wszystkie submoduły |  |
| Git: Utwórz worktree dla zaznaczonego commita... |  |
| Git: Utwórz patch z zaznaczonego commita... |  |
| Git: Utwórz patch od zaznaczonego commita do HEAD... |  |
| Git: Utwórz patch ze zmian w drzewie roboczym... |  |
| Przepisanie: Usuń starą historię na stałe... |  |
| Git: Zastosuj patch... |  |
| Przepisanie: Warsztat historii | `Ctrl+G H` |
| Przepisanie: Zamień autora... |  |
| Przepisanie: Zanonimizuj autorów... |  |
| Przepisanie: Zastosuj .mailmap do historii... |  |
| Przepisanie: Merge tożsamości (.mailmap)... |  |
| Przepisanie: Przesuń daty commitów... |  |
| Przepisanie: Zmień strefę czasową commitów... |  |
| Przepisanie: Ogranicz commity do godzin... |  |
| Przepisanie: Ustaw datę committera na datę autora |  |
| Przepisanie: Znajdź i zamień w opisach... |  |
| Przepisanie: Usuń linie opisu... |  |
| Przepisanie: Dodaj lub usuń trailery... |  |
| Przepisanie: Usuń podpisy... |  |
| Przepisanie: Napraw kodowanie opisów... |  |
| Przepisanie: Znajdź i zamień w plikach... |  |
| Przepisanie: Usuń pliki z historii... |  |
| Przepisanie: Usuń duże pliki z historii... |  |
| Przepisanie: Przenieś lub zmień nazwę ścieżek... |  |
| Przepisanie: Migruj pliki do LFS... |  |
| Przepisanie: Przywróć pliki z LFS... |  |
| Przepisanie: Konwertuj końce linii... |  |
| Przepisanie: Usuń bit wykonywalności... |  |
| Przepisanie: Konwertuj kodowanie plików... |  |
| Przepisanie: Uruchom skrypt... |  |
| ✦ Przepisanie: Napisz skrypt z AI... |  |
| ✦ Przepisanie: Utwórz reguły z AI... |  |
| Przepisanie: Od zaznaczonego commita do HEAD... |  |
| Przepisanie: Zaznaczone Commity... |  |
| LFS: Zainicjuj dla tego repozytorium |  |
| LFS: Śledź wzorzec... |  |
| LFS: Przestań śledzić wzorzec... |  |
| LFS: Pobierz obiekty (fetch) |  |
| LFS: Otwórz menu | `Ctrl+G l` |
| LFS: Fetch wszystkich obiektów | `Ctrl+G F` |
| LFS: Checkout wszystkich plików | `Ctrl+G K` |
| LFS: Push wszystkich obiektów | `Ctrl+G J` |
| LFS: Pobierz obiekty (pull) | `Ctrl+G L` |
| LFS: Wyślij obiekty (push) |  |
| LFS: Wyczyść lokalne obiekty | `Ctrl+G V` |
| LFS: Pokaż blokady |  |
| LFS: Otwórz stronę instalacji |  |
| LFS: Przełącz surowy wskaźnik w diffie | `r` |
| LFS: Śledź zaznaczony plik |  |
| LFS: Śledź rozszerzenie zaznaczonego pliku |  |
| LFS: Zablokuj zaznaczony plik |  |
| LFS: Odblokuj zaznaczony plik |  |
| LFS: Wymuś odblokowanie zaznaczonego pliku |  |
| Refs: Usuń zaznaczoną gałąź lub tag... | `Ctrl+G d` |
| Refs: Usuń zaznaczoną gałąź z jej zdalnego... |  |
| Refs: Usuń zaznaczoną gałąź lokalnie i na zdalnym... |  |
| Refs: Kopiuj link do zaznaczonej gałęzi na zdalnym |  |
| Git: Kopiuj link do zaznaczonego commita na zdalnym |  |
| Git: Opisany tag dla zaznaczonego commita... |  |
| Refs: Push zaznaczonego tagu do domyślnego zdalnego |  |
| Refs: Usuń zaznaczony tag z jego zdalnego... |  |
| Git: Fetch zaznaczony tag |  |
| Refs: Opisz zaznaczony tag... |  |
| Refs: Przełącz ukrycie w grafie |  |
| Refs: Pokaż w grafie | `g` |
| Refs: Przełącz solo dla zaznaczonego refu | `o` |
| Refs: Zakończ solo (wyczyść każdy ref w solo) | `O` |
| Refs: Pokaż wszystkie ukryte refy | `H` |
| Refs: Przełącz ukrycie dla zaznaczonego folderu/zdalnego |  |
| Refs: Przełącz solo dla zaznaczonego folderu/zdalnego | `o` |
| Refs: Filtruj po nazwie... | `/` |
| Refs: Wyczyść filtr nazwy | `Esc` |
| Refs: Rozwiń wszystkie foldery | `=` |
| Refs: Zwiń wszystkie foldery | `-` |
| Refs: Rozwiń zaznaczony folder i jego podfoldery | `*` |
| Refs: Zwiń zaznaczony folder i jego podfoldery | `_` |
| Refs: Następna sekcja | `l` |
| Refs: Poprzednia sekcja | `h` |
| Refs: Rozwiń lub zwiń aktywną sekcję | `Enter` |
| Refs: Przełącz widok hierarchii gałęzi | `y` |
| Git: Stage zaznaczony plik | `Ctrl+G s` |
| Git: Unstage zaznaczony plik | `Ctrl+G u` |
| Git: Stage wszystkie zmiany | `Ctrl+G a` |
| Zmiany robocze: Odrzuć zaznaczony plik... |  |
| Zmiany robocze: Ignoruj zaznaczony plik |  |
| Zmiany robocze: Ignoruj rozszerzenie zaznaczonego pliku |  |
| Zmiany robocze: Traktuj zaznaczony plik jako binarny... |  |
| Zmiany robocze: Traktuj zaznaczony plik jako tekstowy... |  |
| Zmiany robocze: Traktuj rozszerzenie zaznaczonego pliku jako binarne... |  |
| Zmiany robocze: Traktuj rozszerzenie zaznaczonego pliku jako tekstowe... |  |
| Zmiany robocze: Stash zaznaczonego pliku |  |
| Zmiany robocze: Historia zaznaczonego pliku |  |
| Zmiany robocze: Blame zaznaczonego pliku |  |
| Zmiany robocze: Otwórz zaznaczony plik w zewnętrznym narzędziu diff |  |
| Zmiany robocze: Otwórz zaznaczony plik w zewnętrznym edytorze |  |
| Zmiany robocze: Otwórz zaznaczony plik w programie domyślnym |  |
| Zmiany robocze: Pokaż zaznaczony plik w folderze |  |
| Zmiany robocze: Kopiuj ścieżkę zaznaczonego pliku |  |
| Zmiany robocze: Utwórz patch z zaznaczonego pliku... |  |
| Zmiany robocze: Edytuj zaznaczony plik |  |
| Zmiany robocze: Usuń zaznaczony plik... |  |
| Zmiany robocze: Otwórz menu zaznaczonego folderu |  |
| Zmiany robocze: Utwórz plik |  |
| Zmiany robocze: Odrzuć wszystkie zmiany... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Zrób commit zmian ze stage od razu | `Ctrl+G Enter` |
| Git: Edytuj opis commita WIP (w panelu Log) | `c w Log` |
| Git: Fetch wszystkich zdalnych | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward, jeśli się da) |  |
| Git: Pull (tylko fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Opcje pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push do zdalnego... |  |
| Git: Force push (z lease)... |  |
| Git: Ustaw upstream... |  |
| Zdalne: Dodaj zdalne... |  |
| Zdalne: Usuń zdalne... |  |
| Zdalne: Edytuj zdalne... |  |
| Zdalne: Ustaw domyślne zdalne... |  |
| Zdalne: Kopiuj nazwę zdalnego |  |
| Zdalne: Kopiuj link do zdalnego |  |
| Zdalne: Fetch zdalnego... |  |
| Zdalne: Akcje dla zaznaczonego zdalnego... | `.` |
| Repozytorium: Klucz SSH dla tego repozytorium... |  |
| Repozytorium: Częściowy checkout... |  |
| Flow: Otwórz menu | `Ctrl+G W` |
| Flow: Rozpocznij gałąź... |  |
| Flow: Nowe zgłoszenie... |  |
| Flow: Przejmij gałąź hosta... |  |
| Flow: Zakończ bieżącą gałąź... |  |
| Flow: Synchronizuj z gałęzią nadrzędną... |  |
| Flow: Promuj... |  |
| Flow: Ustaw gałąź nadrzędną... |  |
| Flow: Kontynuuj lub przerwij zatrzymaną akcję... |  |
| Flow: Przełącz grupowanie gałęzi |  |
| Flow: Ustawienia przepływu pracy... |  |
| Zdalne: Ukryj zdalne w grafie... |  |
| Zdalne: Pokaż zdalne w grafie... |  |
| Git: Nowa gałąź w HEAD... | `Ctrl+G b` |
| Git: Nowy tag w HEAD... | `Ctrl+G t` |
| Git: Push wszystkich niewypchniętych tagów | `Ctrl+G U` |
| Git: Stash wszystkich zmian | `Ctrl+G T` |
| Git: Stash wszystkich zmian z opisem... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: Cherry-pick zaznaczonego commita na HEAD | `Ctrl+G y` |
| Git: Checkout zaznaczonego commita (odłączony HEAD) |  |
| Git: Utwórz gałąź dla zaznaczonego commita... |  |
| Git: Utwórz tag dla zaznaczonego commita... |  |
| Commit: Kopiuj opis |  |
| Commit: Kopiuj imię autora |  |
| Commit: Kopiuj e-mail autora |  |
| Commit: Kopiuj datę |  |
| Commit: Kopiuj krótki SHA |  |
| Commit: Kopiuj pełny SHA |  |
| Git: Revert zaznaczonego commita... | `Ctrl+G v` |
| Git: Edytuj opis commita... | `Ctrl+G e` |
| Commit: Popraw... | `Ctrl+G E` |
| Commit: Popraw autora... | `Ctrl+G A` |
| Commit: Popraw podsumowanie... |  |
| Commit: Popraw opis... |  |
| Git: Podziel zaznaczony commit... |  |
| Git: Interaktywny rebase od zaznaczonego commita... |  |
| Rebase: Kontynuuj |  |
| Rebase: Pomiń |  |
| Rebase: Przerwij |  |
| git am: Kontynuuj |  |
| git am: Pomiń ten patch |  |
| git am: Przerwij |  |
| Repozytorium: Usuń nieaktualny index.lock |  |
| Baner: Pop oczekującego auto-stasha |  |
| Baner: Zakończ oczekujący auto-stash (zachowaj wynik) |  |
| Baner: Pokaż oczekujący auto-stash |  |
| Konflikty: Pokaż przewidywane konflikty |  |
| Baner: Drop oczekującego auto-stasha... |  |
| Baner: Zacznij od nowa... |  |
| Git: Reset gałęzi do zaznaczonego commita (soft - zachowaj indeks i drzewo robocze) |  |
| Git: Reset gałęzi do zaznaczonego commita (mixed - zachowaj drzewo robocze) | `Ctrl+G R` |
| Git: Reset gałęzi do zaznaczonego commita (hard - odrzuć wszystko)... |  |
| Git: Rozwiąż konflikt przez Ours (ta gałąź) |  |
| Git: Rozwiąż konflikt przez Theirs (przychodzące) |  |
| Git: Rozwiąż konflikt przez Bazę (wspólny przodek) |  |
| Konflikty: Otwórz edytor merge | `Enter na pliku z konfliktem` |
| Merge: Następny konflikt | `F7` |
| Merge: Poprzedni konflikt | `Shift+F7` |
| Merge: Następny plik z konfliktem | `Alt+PageDown` |
| Merge: Poprzedni plik z konfliktem | `Alt+PageUp` |
| Merge: Użyj bloku z A |  |
| Merge: Użyj bloku z B |  |
| Merge: Użyj bieżącego dla tego konfliktu | `a` |
| Merge: Użyj przychodzącego dla tego konfliktu | `b` |
| Merge: Użyj obu, A potem B | `c` |
| Merge: Użyj obu, B potem A | `C` |
| Merge: Użyj Bazy dla tego konfliktu |  |
| Merge: Wyczyść ten konflikt | `x` |
| Merge: Zamień kolejność | `s` |
| Merge: Użyj całego bieżącego | `A` |
| Merge: Użyj całego przychodzącego | `B` |
| Merge: Edytuj ten konflikt w Wyniku |  |
| Merge: Pokaż lub ukryj panele boczne | `F9` |
| Merge: Przewiń panele w lewo | `Shift+Left` |
| Merge: Przewiń panele w prawo | `Shift+Right` |
| Merge: Przełącz panel Bazy | `Alt+D` |
| Merge: Przełącz diff całego pliku | `Alt+F` |
| Merge: Cofnij | `Ctrl+Z` |
| Merge: Ponów | `Ctrl+Y` |
| Merge: Zapisz bez oznaczania jako rozwiązany | `Alt+S` |
| Merge: Zapisz i oznacz jako rozwiązany | `Ctrl+S` |
| Merge: Przywróć Wynik do wyniku merge | `Alt+R` |
| Merge: Przełącz wysokość Wyniku | `z` |
| Merge: Poszerz panel bieżący | `Alt+Right` |
| Merge: Poszerz panel przychodzący | `Alt+Left` |
| Merge: Przywróć rozmiary paneli | `Alt+0` |
| Merge: Cofnij wybór tego konfliktu | `Alt+U` |
| Merge: Przejdź do panelu bieżącego (A) | `Alt+1` |
| Merge: Przejdź do panelu przychodzącego (B) | `Alt+2` |
| Merge: Przejdź do panelu Wyniku | `Alt+3` |
| Merge: Przejdź do panelu Bazy | `Alt+4` |
| Merge: Przejdź do panelu po lewej (ten sam konflikt) |  |
| Merge: Przejdź do panelu po prawej (ten sam konflikt) |  |
| Merge: Przejdź do następnego panelu | `F6` |
| Merge: Przejdź do poprzedniego panelu | `Shift+F6` |
| Merge: Kontynuuj z pliku na dysku |  |
| Merge: Zacznij od nowa od merge |  |
| Merge: Cofnij rozwiązanie i otwórz ponownie |  |
| Merge: Wczytaj plik ponownie |  |
| Merge: Użyj pliku bieżącego bez zmian | `a` |
| Merge: Użyj pliku przychodzącego bez zmian | `b` |
| Merge: Użyj pliku Bazy bez zmian | `o` |
| Merge: Zachowaj pozostały plik | `k` |
| Merge: Zaakceptuj usunięcie | `D` |
| Merge: Otwórz w zewnętrznym narzędziu merge | `m` |
| ✦ AI: Zasugeruj rozwiązanie konfliktu merge | `Alt+I` |
| Merge: Zamknij edytor merge |  |
| Git: Przerwij merge, cherry-pick lub revert |  |
| Git: Pomiń cherry-pick lub revert |  |
| Repozytorium: Otwórz lub przełącz (zarządzanie repozytoriami)... | `Ctrl+O` |
| Repozytorium: Przeglądaj w poszukiwaniu repozytorium... | `b (karta zarządzania)` |
| Repozytorium: Utwórz nowe (Init)... | `i (karta zarządzania)` |
| Repozytoria: Otwórz zaznaczone | `Enter` |
| Repozytoria: Dodaj lub usuń zaznaczone z ulubionych | `f` |
| Repozytoria: Zamknij lub usuń zaznaczone | `x` |
| Repozytoria: Menu zaznaczonego repozytorium... | `.` |
| Repozytoria: Szukaj | `/ / Ctrl+F` |
| Repozytoria: Zwiń wszystkie sekcje | `-` |
| Repozytoria: Rozwiń wszystkie sekcje | `+` |
| Repozytoria: Zamknij wszystkie karty... |  |
| Repozytoria: Usuń zaznaczone z listy |  |
| Repozytoria: Usuń wszystkie ostatnie |  |
| Obszar roboczy: Nowy... | `n` |
| Obszar roboczy: Nowy z otwartych kart... | `a` |
| Obszar roboczy: Przełącz... | `Alt+W` |
| Obszar roboczy: Edytuj... | `e` |
| Obszar roboczy: Zamknij (brak obszaru roboczego) |  |
| Obszar roboczy: Zarządzaj... | `Alt+J` |
| Obszar roboczy: Zamknij kartę |  |
| Obszar roboczy: Wskaż brakujące repozytorium... | `l` |
| Obszar roboczy: Usuń... |  |
| Obszar roboczy: Fetch wszystkich | `F` |
| Obszar roboczy: Pull wszystkich | `P` |
| Obszar roboczy: Klonuj brakujące | `m` |
| Obszar roboczy: Dodaj kartę do obszaru roboczego |  |
| Obszar roboczy: Usuń kartę z obszaru roboczego |  |
| Radar: Otwórz | `Alt+R` |
| Radar: Odśwież | `r` |
| Radar: Szukaj | `/` |
| Radar: Otwórz w przeglądarce | `o` |
| Radar: Otwórz repozytorium | `Enter` |
| Radar: Checkout gałęzi | `c` |
| Radar: Utwórz worktree... | `t` |
| Radar: Pokaż w repozytorium | `s` |
| Radar: Zacznij pracę nad zgłoszeniem... | `n` |
| Radar: Przypnij lub odepnij | `p` |
| Radar: Odłóż... | `z` |
| Radar: Przejrzyj żądanie... | `e` |
| Radar: Przejrzyj przez API |  |
| Radar: Sklonuj i przejrzyj... |  |
| Radar: Edytuj przypisanych... |  |
| Radar: Edytuj recenzentów... |  |
| Radar: Edytuj etykiety... |  |
| Radar: Ustaw kamień milowy... |  |
| Radar: Wybierz zakres | `w` |
| Radar: Wybierz widok | `v` |
| Radar: Wybierz filtr | `f` |
| Radar: Pokaż niesklonowane żądania | `a` |
| Radar: Pokaż szkice | `d` |
| Radar: Włącz żądania do przeglądu z całego konta... | `T` |
| Radar: Menu wiersza... | `.` |
| Radar: Wybierz kolumnę | `C` |
| Radar: Poszerz kolumnę | `]` |
| Radar: Zwęź kolumnę | `[` |
| Radar: Resetuj szerokości kolumn | `0` |
| Radar: Wróć do strony głównej | `Esc` |
| Radar: Zamknij kartę |  |
| Karta: Nowa karta | `Alt+T` |
| Karta: Nowa... |  |
| Karta: Menu akcji... | `Ctrl+T` |
| Karta: Zamknij inne karty... |  |
| Karta: Zamknij karty po prawej... |  |
| Karta: Dodaj do ulubionych lub usuń |  |
| Karta: Kopiuj ścieżkę repozytorium |  |
| Karta: Otwórz w menedżerze plików |  |
| Karta: Otwórz w terminalu |  |
| Karta: Otwórz w... | `Alt+O` |
| Karta: Otwórz w domyślnym | `Ctrl+Alt+O` |
| Repozytorium: Zmień profil... |  |
| Pomoc: O programie GitPoulpi | `Ctrl+N a` |
| Pomoc: Licencje i podziękowania |  |
| Widok: Dziennik aktywności | `Ctrl+N A` |
| Widok: Globalny dziennik aktywności | `Ctrl+N G` |
| Pomoc: Informacje o wersji |  |
| Aktualizacje: Sprawdź aktualizacje teraz |  |
| Aktualizacje: Pobierz |  |
| Aktualizacje: Zaktualizuj teraz |  |
| Aktualizacje: Uruchom ponownie, by zaktualizować |  |
| Aktualizacje: Wróć do poprzedniej wersji |  |
| Aktualizacje: Pomiń tę wersję |  |
| Informacje o wersji: Zamknij kartę |  |
| Zadania: pokaż zadania w tle | `Ctrl+N J` |
| Zadania: anuluj wszystko, co można anulować |  |
| Widok: Kopiuj diagnostykę |  |
| Dziennik aktywności: pokaż lub ukryj poziom Błąd |  |
| Dziennik aktywności: pokaż lub ukryj poziom Ostrzeżenie |  |
| Dziennik aktywności: pokaż lub ukryj poziom Info |  |
| Dziennik aktywności: pokaż lub ukryj poziom Debugowanie |  |
| Dziennik aktywności: pokaż lub ukryj poziom Śledzenie |  |
| Dziennik aktywności: pokaż wszystkie poziomy |  |
| Dziennik aktywności: nie pokazuj żadnego poziomu |  |
| Dziennik aktywności: Wyczyść dziennik aplikacji |  |
| Dziennik aktywności: Wybierz zakres |  |
| Dziennik aktywności: Operacje git |  |
| Dziennik aktywności: Dziennik aplikacji |  |
| Dziennik aktywności: Kopiuj ostatni błąd |  |
| Dziennik aktywności: Wykonaj sugerowaną akcję |  |
| Dziennik aktywności: Otwórz ostatni błąd |  |
| Karta: Zakończ i cd tutaj |  |
| Repozytorium: Klonuj... | `Ctrl+Shift+N` |
| Repozytorium: Następna karta | `Ctrl+PageDown` |
| Repozytorium: Poprzednia karta | `Ctrl+PageUp` |
| Karta: Przesuń w lewo |  |
| Karta: Przesuń w prawo |  |
| Repozytorium: Zamknij kartę | `Ctrl+W` |
| Repozytorium: Wybierz kartę... | `t` |
| Szukaj: Obszar roboczy... | `Ctrl+Alt+F` |
| Edycja: Cofnij | `Ctrl+Z` |
| Edycja: Ponów | `Ctrl+Y` |
| Edycja: Zapomnij zablokowany wpis cofania |  |
| Edycja: Historia cofania... | `Alt+Z` |
| Edycja: Reflog... |  |
| Edycja: Odzyskaj utracone commity... |  |
| Edycja: Przywróć gałąź tam, gdzie zostawiła ją zablokowana akcja | `Alt+Shift+Z` |
| Przegląd: Pokaż podsumowanie |  |
| Przegląd: Pokaż pliki |  |
| Przegląd: Pokaż commity |  |
| Przegląd: Skomentuj linię... | `c` |
| Przegląd: Skomentuj dyskusję... | `C` |
| Przegląd: Odpowiedz w wątku... | `r` |
| Przegląd: Rozwiąż lub otwórz ponownie wątek | `x` |
| Przegląd: Zwiń lub rozwiń wątek | `z` |
| Przegląd: Edytuj oczekujący komentarz... | `e` |
| Przegląd: Usuń oczekujący komentarz | `d` |
| Przegląd: Oznacz plik jako przejrzany | `v` |
| Przegląd: Następny plik | `]` |
| Przegląd: Poprzedni plik | `[` |
| Przegląd: Zakończ przegląd... | `f` |
| Przegląd: Odrzuć oczekujący przegląd... |  |
| Przegląd: Odśwież | `F5` |
| Przegląd: Otwórz w przeglądarce | `o` |
| Przegląd: Checkout gałęzi | `b` |
| Przegląd: Zamknij | `Esc` |
| Przegląd: Edytuj przypisanych... | `a` |
| Przegląd: Edytuj recenzentów... | `p` |
| Przegląd: Edytuj etykiety... | `l` |
| Przegląd: Ustaw kamień milowy... | `m` |
| Przegląd: Wybierz patch set... | `P` |
| Przegląd: Porównaj patch sety... | `D` |
| Przegląd: Wprowadź zmianę... | `U` |
| Przegląd: Porzuć zmianę... | `Q` |
| Przegląd: Przełącz układ w linii / obok siebie | `V` |
| Przegląd: Przełącz cały plik / tylko hunki | `H` |
| Przegląd: Pokaż lub ukryj zmiany tylko w białych znakach | `S` |
| Przegląd: Przełącz zawijanie linii | `W` |
| Przegląd: Następny hunk | `J` |
| Przegląd: Poprzedni hunk | `K` |
| Przegląd: Wybierz kodowanie... | `E` |
| Przegląd: Przełącz minimapę | `M` |
| Przegląd: Przewiń diff w lewo | `Left` |
| Przegląd: Przewiń diff w prawo | `Right` |
| Przegląd: Kopiuj zaznaczony tekst lub linię | `Ctrl+C` |
| Tablica: Przenieś kartę do... | `m` |
| Tablica: Przenieś kartę w lewo |  |
| Tablica: Przenieś kartę w prawo |  |
| Tablica: Otwórz kartę | `Enter` |
| Tablica: Edytuj przypisanych... |  |
| Tablica: Edytuj etykiety... |  |
| Tablica: Ustaw kamień milowy... |  |
| Tablica: Nowe zgłoszenie w tej kolumnie | `n` |
| Tablica: Filtruj według przypisanego... | `a` |
| Tablica: Filtruj według etykiety... | `l` |
| Tablica: Filtruj według kamienia milowego... | `M` |
| Tablica: Filtruj według tekstu | `/` |
| Tablica: Wyczyść filtry | `x` |
| Tablica: Ustaw etykiety kolumn... | `c` |
| Tablica: Odśwież | `r` |
| Tablica: Zamknij | `Esc` |
| Przegląd: ✦ Przygotuj szkic przeglądu... | `s` |
| Przegląd: Przyjmij sugestię | `y` |
| Przegląd: Edytuj i przyjmij sugestię... | `w` |
| Przegląd: Odrzuć sugestię | `n` |
| Przegląd: Przyjmij wszystkie sugestie | `A` |
| Przegląd: Odrzuć wszystkie sugestie | `N` |
| Przegląd: Użyj podsumowania AI w treści przeglądu | `i` |
| Przegląd: ✦ Użyj szkicu... | `u` |
| Przegląd: Edytuj tekst ✦ szkicu... | `t` |
| Przegląd: Odrzuć ✦ szkic | `N` |
| Historia cofania: Cofnij do tego miejsca... | `u` |
| Historia cofania: Ponów do tego miejsca... | `y` |
| Historia cofania: Przywróć tylko tę... | `r` |
| Historia cofania: Przywróć gałąź tam, gdzie była... | `o` |
| Historia cofania: Odśwież |  |
| Historia cofania: Zamknij | `Esc` |
| Reflog: Pokaż w Log | `s` |
| Reflog: Utwórz gałąź tutaj... | `b` |
| Reflog: Przywróć do tej pozycji... | `r` |
| Reflog: Wczytaj więcej | `m` |
| Reflog: Odśwież |  |
| Utracona praca: Pokaż w Log | `s` |
| Utracona praca: Utwórz gałąź tutaj... | `b` |
| Utracona praca: Cherry-pick | `c` |
| Utracona praca: Szukaj ponownie |  |
| Widok: Odśwież | `r` |
| Widok: Odśwież (fetch, pull requesty, zgłoszenia) | `F5` |
| Widok: Przełącz pomoc | `?` |
| Widok: Przełącz kolumnę Ref |  |
| Widok: Przełącz kolumnę Graf |  |
| Widok: Przełącz kolumnę Opis |  |
| Widok: Przełącz kolumnę Autor |  |
| Widok: Przełącz kolumnę Data |  |
| Widok: Przełącz kolumnę Zmiany |  |
| Widok: Przełącz kolumnę SHA |  |
| Widok: Kolumna Logu: Wybierz kolumnę do przesunięcia lub zmiany szerokości | `C` |
| Widok: Kolumna Logu: Wybierz następną kolumnę | `Tab` |
| Widok: Kolumna Logu: Wybierz poprzednią kolumnę | `Shift+BackTab` |
| Widok: Kolumna Logu: Przesuń kolumnę w lewo | `<` |
| Widok: Kolumna Logu: Przesuń kolumnę w prawo | `>` |
| Widok: Kolumna Logu: Poszerz kolumnę | `]` |
| Widok: Kolumna Logu: Zwęź kolumnę | `[` |
| Widok: Kolumna Logu: Zresetuj szerokości kolumn | `0` |
| Widok: Kolumna Logu: Dopasuj kolumnę do zawartości | `=` |
| Widok: Kolumna Logu: Dopasuj wszystkie kolumny do zawartości | `+` |
| Widok: Log: Przewiń kolumny w lewo | `Shift+Left` |
| Widok: Log: Przewiń kolumny w prawo | `Shift+Right` |
| Widok: Log: Przełącz przewijanie kolumn w poziomie | `s` |
| Widok: Log: Wybierz kolumny do pokazania |  |
| Diff: Otwórz diff zaznaczonego pliku | `Enter` |
| Diff: Zamknij widok diff | `Esc` |
| Diff: Następny hunk | `n` |
| Diff: Poprzedni hunk | `p` |
| Diff: Stage hunk pod kursorem | `Space` |
| Diff: Unstage hunk pod kursorem | `Space` |
| Diff: Zaznacz linie (lub wyczyść zaznaczenie) | `v` |
| Diff: Stage zaznaczone linie | `Enter` |
| Diff: Unstage zaznaczone linie | `Enter` |
| Diff: Odrzuć hunk pod kursorem... | `x` |
| Diff: Odrzuć zaznaczone linie... | `X` |
| Kopiuj: Zaznaczony tekst (najpierw przeciągnij po tekście) | `Ctrl+C` |
| Kopiuj: Linię pod kursorem (lub zaznaczone linie) | `y` |
| Diff: Kopiuj hunk pod kursorem |  |
| Diff: Układ w linii (+/- w jednej kolumnie) | `m` |
| Diff: Układ obok siebie (przed i po) | `m` |
| Diff: Przełącz tylko hunki (cały plik / tylko zmiany) | `H` |
| Pliki: Przełącz ścieżka/drzewo | `m` |
| Pliki: Przełącz widok wszystkich plików | `a` |
| Commit: Przełącz pomijanie hooków | `H` |
| Pliki: otwórz menu wiersza lub zaznaczenia | `.` |
| Pliki: Rozwiń wszystkie foldery | `=` |
| Pliki: Zwiń wszystkie foldery | `-` |
| Pliki: Rozwiń zaznaczony folder i jego podfoldery | `*` |
| Pliki: Zwiń zaznaczony folder i jego podfoldery | `_` |
| Pliki: zaznacz wszystko |  |
| Pliki: wyczyść zaznaczenie |  |
| Pliki: przełącz zaznaczenie bieżącego wiersza |  |
| Log: Kopiuj jako gitGraph Mermaid... | `Y` |
| Log: Porównaj zaznaczenie | `v` |
| Log: Zamień końce porównania | `x` |
| Log: przejdź do następnego porównywanego wiersza | `n` |
| Diff: Przełącz zawijanie linii | `w` |
| Diff: Wybierz kodowanie... | `C` |
| Widok: Przełącz minimapę | `M` |
| Widok: Przełącz kolumnę commitów | `Alt+C` |
| Widok: Kolumna commitów: Ukryj |  |
| Widok: Kolumna commitów: Historia pliku |  |
| Widok: Kolumna commitów: Wszystkie commity |  |
| Widok: Przenieś fokus na kolumnę commitów |  |
| Widok: Poszerz kolumnę commitów |  |
| Widok: Zawęź kolumnę commitów |  |
| Widok: Kolumna commitów: Przewiń w lewo | `Shift+Left` |
| Widok: Kolumna commitów: Przewiń w prawo | `Shift+Right` |
| Widok: Kolumna commitów: Wybierz następną część | `Tab` |
| Widok: Kolumna commitów: Przesuń część w lewo | `<` |
| Widok: Kolumna commitów: Przesuń część w prawo | `>` |
| Widok: Kolumna commitów: Poszerz część | `]` |
| Widok: Kolumna commitów: Zwęź część | `[` |
| Widok: Kolumna commitów: Zresetuj szerokości części | `0` |
| Widok: Kolumna commitów: Menu zaznaczonego wiersza | `.` |
| Widok: Kolumna commitów: Wybierz kolumny |  |
| Widok: Kolumna commitów: Przełącz Ref |  |
| Widok: Kolumna commitów: Przełącz Graf |  |
| Widok: Kolumna commitów: Przełącz Autor |  |
| Widok: Kolumna commitów: Przełącz Data |  |
| Widok: Kolumna commitów: Przełącz Zmiany |  |
| Widok: Kolumna commitów: Przełącz SHA |  |
| Widok: Kolumna commitów: Pokaż zaznaczony wiersz w Log |  |
| Widok: Kolumna commitów: Pokaż zmiany pliku z zaznaczonego wiersza |  |
| Widok: Kolumna commitów: Pokaż plik w stanie z zaznaczonego wiersza |  |
| Widok: Kolumna commitów: Blame pliku z zaznaczonego wiersza |  |
| Edytor: przełącz podążanie za zaznaczeniem |  |
| Obraz: Powiększ | `+` |
| Obraz: Pomniejsz | `-` |
| Obraz: Dopasuj do widoku | `0` |
| Obraz: Rzeczywisty rozmiar (1:1) | `1` |
| Obraz: Przełącz przed / po | `Tab` |
| Obraz: Poszerz panel Przed | `Alt+Right` |
| Obraz: Poszerz panel Po | `Alt+Left` |
| Obraz: Przywróć rozmiary paneli | `Alt+0` |
| Obraz: Edytuj w... | `O` |
| Obraz: Edytuj w domyślnym edytorze | `o` |
| Historia pliku: Pokaż w Log | `Enter` |
| Historia pliku: Kopiuj SHA commita | `y` |
| Historia pliku: Kopiuj podsumowanie | `Y` |
| Historia pliku: Pokaż diff tego commita | `d` |
| Historia pliku: Otwórz plik z tego commita | `o` |
| Historia pliku: Blame dla tego commita | `b` |
| Historia pliku: Porównaj z drzewem roboczym | `c` |
| Historia pliku: Przywróć tę wersję | `R` |
| Historia pliku: Rozszerz zaznaczenie w górę | `Shift+Up` |
| Historia pliku: Rozszerz zaznaczenie w dół | `Shift+Down` |
| Historia pliku: Przełącz między listą a diffem | `Tab` |
| Historia pliku: Poszerz listę | `Alt+Right` |
| Historia pliku: Zwęź listę | `Alt+Left` |
| Historia pliku: Zresetuj szerokość listy | `Alt+0` |
| Plik: Otwórz za pomocą... | `O` |
| Plik: Otwórz w aplikacji domyślnej | `o` |
| Diff: Przełącz zmiany tylko w białych znakach | `s` |
| ✦ AI: Wygeneruj opis commita | `Ctrl+G i` |
| ✦ AI: Wygeneruj opis stasha |  |
| ✦ AI: Wstaw zmienną promptu |  |
| ✦ AI: Edytuj profil... |  |
| ✦ AI: Otwórz agenta... |  |
| ✦ AI: Wyjaśnij zaznaczony commit | `Ctrl+G w` |
| ✦ AI: Podziel zaznaczony commit... |  |
| ✦ AI: Wyjaśnij zmiany robocze | `Ctrl+G w` |
| ✦ AI: Skomponuj commity ze zmian roboczych... | `Ctrl+G C` |
| Kompozytor: Przenieś plik do commita... | `m` |
| Diff: Blame |  |
| Blame: Pokaż commit w Log | `Enter` |
| Blame: Kopiuj SHA commita | `Y` |
| Blame: Pokaż diff commita dla pliku | `d` |
| Blame: Blame na rodzicu commita | `b` |
| Blame: Otwórz historię pliku | `H` |
| Blame: Pokaż lub ukryj kartę szczegółów | `i` |
| Blame: Następny hunk | `n` |
| Blame: Poprzedni hunk | `p` |
| Blame: Następny hunk z tego commita | `N` |
| Blame: Poprzedni hunk z tego commita | `P` |
| Blame: Przejdź do linii | `g` |
| Blame: Poszerz kolumnę adnotacji | `>` |
| Blame: Zwęź kolumnę adnotacji | `<` |
| Diff: Historia pliku |  |
| Diff: Widok pliku |  |
| Plik: Edytuj w drzewie roboczym | `e` |
| Edytor: Zmień kodowanie |  |
| Edytor: Otwórz ponownie z kodowaniem |  |
| Edytor: Konwertuj na kodowanie |  |
| Edytor: Konwertuj końce linii |  |
| Edytor: Zmień wcięcia |  |
| Aplikacja: Ustawienia... | `Ctrl+, / Ctrl+N ,` |
| Aplikacja: Szukaj ustawień... | `/ w Ustawieniach` |
| Aplikacja: Skróty klawiszowe... |  |
| Agent: Pokaż oczekujący widok | `Ctrl+N Enter` |
| Agent: Odrzuć oczekujący widok |  |
| Agent: Wstecz | `Ctrl+N Backspace` |
| Skróty: Resetuj wszystkie... |  |
| Integracje: Otwórz powiązany pull request | `Ctrl+G g` |
| Integracje: Edytuj powiązany pull request na hoście | `e (na wierszu)` |
| Integracje: Kopiuj URL (powiązany pull request) |  |
| Integracje: Odśwież pull requesty i zgłoszenia |  |
| Integracje: Checkout pull request... |  |
| Integracje: Otwórz odwołanie z commita (#zgłoszenie / !żądanie)... |  |
| Integracje: Otwórz zgłoszenie... |  |
| Hosting: Przejrzyj żądanie... |  |
| Hosting: Push do przeglądu... |  |
| Hosting: Edytuj przypisanych... |  |
| Hosting: Edytuj recenzentów... |  |
| Hosting: Edytuj etykiety... |  |
| Hosting: Ustaw kamień milowy... |  |
| Zgłoszenia lokalne: Zamknij zgłoszenie... |  |
| Zgłoszenia lokalne: Otwórz zgłoszenie ponownie... |  |
| Zgłoszenia lokalne: Otwórz w Obsidian... |  |
| Zgłoszenia: Otwórz tablicę |  |
| Zgłoszenia lokalne: Utwórz Obsidian Base |  |
| Pull requesty: Utwórz pull request... | `n` |
| Hosting: Nowe zgłoszenie... | `n` |
| Pull requesty: Szukaj | `Enter na wierszu szukania` |
| Pull requesty: Wybierz filtr (otwarte / merged / wszystkie)... | `v` |
| Pull requesty: Wybierz dostawcę... | `Enter na wierszu dostawcy` |
| Pull requesty: Wybierz repozytorium... | `Enter na wierszu repozytorium` |
| Pull requesty: Odłącz źródło (brak) |  |
| Zgłoszenia: Wybierz dostawcę... | `Enter na wierszu dostawcy` |
| Zgłoszenia: Odłącz źródło (brak) |  |
| Integracje: Odłącz profil (automatycznie) |  |
| Zgłoszenia: Wybierz repozytorium... | `Enter na wierszu repozytorium` |
| Zgłoszenia: Szukaj | `Enter na wierszu szukania` |
| Zgłoszenia: Wybierz filtr... | `v` |
| Zgłoszenia: Edytuj zapytanie / listy... |  |
| Commit: Wstaw odwołanie do zgłoszenia... |  |
| Integracje: Użyj profilu dla tego repozytorium... |  |
| Integracje: Zarządzaj profilem tego repozytorium |  |
| Integracje: Połącz integrację... |  |
| Ustawienia: Opcje Radaru profilu... |  |
| Aplikacja: Zakończ | `q` |
| Aplikacja: Motyw... |  |
| Aplikacja: Zmień motyw... | `Ctrl+N y` |
| Aplikacja: Zmień język... | `Ctrl+N g` |
| Uruchamianie: Utwórz skrót |  |
| Uruchamianie: Utwórz skrót na pulpicie |  |
| Uruchamianie: Usuń skróty |  |
| Uruchamianie: Utwórz alias polecenia |  |
| Uruchamianie: Usuń alias polecenia |  |
| Uruchamianie: Dodaj do PATH |  |
| Uruchamianie: Usuń z PATH |  |
| Aplikacja: Zmień motyw kolorów grafu... | `Ctrl+N Y` |
| Aplikacja: Przełącz zabarwienie wierszy grafu | `Ctrl+N T` |
<!-- palette:end -->

## Poruszanie się

| Klawisz | Działanie |
|---|---|
| `Tab` / `Shift+Tab` | Fokus na następny / poprzedni panel |
| `Ctrl+N`, potem plakietka | Fokus na tej części okna (zob. [Prefiks nawigacji](#prefiks-nawigacji-ctrln)) |
| `j` / `k`, `Up` / `Down` | Przesuwają zaznaczenie, w każdym panelu |
| `PageUp` / `PageDown`, `Home` / `End` | Przesuwają o stronę; skaczą do pierwszego / ostatniego wiersza |
| liczba, potem `j` / `k` / `Up` / `Down` | Prefiks licznika: `4j` przesuwa o cztery wiersze. Z `PageUp` / `PageDown` licznik oznacza liczbę stron. Pasek stanu pokazuje oczekujący licznik (`4…`). `Esc` lub inny klawisz go czyści. Początkowe `0` nigdy nie zaczyna licznika |
| `h` / `l`, `Left` / `Right` | Panel Refs: zmiana sekcji. Log: wyjście (`h`) z listy zmienionych plików commita lub wejście (`l`) do niej |
| `?` | Otwiera lub zamyka pomoc (zob. [Pomoc w aplikacji](#pomoc-w-aplikacji-)) |
| `r` | Odświeża |
| `F5` | Robi fetch wszystkich zdalnych, wczytuje ponownie pull requesty i zgłoszenia oraz odświeża |
| `q` | Kończy program |
| `Ctrl+Z` | **Cofnij** ostatnią operację. Zob. [Cofnij i ponów](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Ponów**. `Ctrl+Shift+Z` wymaga protokołu klawiatury kitty; `Ctrl+Y` działa zawsze |
| `.`, `Menu`, `Shift+F10` | Otwiera menu kontekstowe zaznaczonego wiersza |

`Ctrl+Z` i `Ctrl+Y` nic nie robią, gdy klawiaturę ma pole tekstowe. Edytor ma
własne cofanie tekstu.

## Filtrowalne listy w oknach

Wszystkie filtrowalne listy w oknach używają tych samych klawiszy: paleta
poleceń i jej selektory (motyw, język, kolory grafu), selektor kart i każda
lista rozwijana z polem wyszukiwania (wybory w Ustawieniach, formularz pull
requesta, selektor odwołań do zgłoszeń, lista **Otwórz w**). Wpisuj, by
filtrować listę. Te klawisze przesuwają podświetlenie:

| Klawisz | Działanie |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Przesuwają o jeden wiersz |
| `Home` / `End` | Pierwszy / ostatni wiersz |
| `PageUp` / `PageDown` | Przesuwają o jedną widoczną stronę |
| `Enter` | Wybiera podświetlony wiersz |
| `Esc` | Zamyka |

W tych oknach `Home`, `End` i `Ctrl+K` przesuwają listę, a nie kursor pola
filtra. `Ctrl+A` i `Ctrl+E` nadal przesuwają kursor na początek i na koniec.
Zwykłe `j` i `k` są wpisywane do filtra.

Z myszą:

- Kółko **przewija** listę. Nie przesuwa podświetlenia.
- Kliknięcie **zaznacza** wiersz. Dwukrotne kliknięcie go **wybiera**, jak
  `Enter`.
- Lista wyższa niż jej okno pokazuje pasek przewijania przy prawej krawędzi.
  Można go przeciągać.
- Krótka lista rozwijana bez pola wyszukiwania (dwa lub trzy wybory) wybiera
  po pierwszym kliknięciu.

**Podgląd na żywo.** Selektory motywu, motywu kolorów grafu i języka stosują
podświetlony wiersz od razu, za oknem. `Enter` go zachowuje. `Esc` albo
kliknięcie poza oknem przywraca poprzednią wartość.

**Filtr motywów.** Selektor motywów ma wiersz pod polem filtra: **Wszystkie**,
**Ciemne**, **Jasne** i **Kolorowe**. `Tab` / `Shift+Tab` przechodzą do
następnego / poprzedniego. `Alt+1` do `Alt+4` wybierają jeden bezpośrednio.
Można też w nie klikać.

## Pomoc w aplikacji (`?`)

`?`, `Ctrl+N ?` albo polecenie palety `Widok: Przełącz pomoc` otwiera pomoc.
Wymienia klawisze i gesty myszy każdej części aplikacji. Otwiera się nad
wszystkim, także nad ekranem Ustawień.

| Klawisz | Działanie |
|---|---|
| `?` | Zamyka pomoc (i czyści filtr) |
| *(wpisywanie)* | Filtr: zostają tylko sekcje zawierające tekst. `j`, `k` i `q` też są wpisywane |
| `Backspace`, `Ctrl+U` | Usuwa ostatni znak / czyści filtr |
| `Esc` | Czyści filtr; jeśli jest pusty, zamyka pomoc |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Przewijają o jedną linię |
| `PageUp` / `PageDown`, `Home` / `End` | Przewijają o stronę; skaczą na górę / na dół |
| `Ctrl+D` albo kliknięcie linku | Otwiera tę dokumentację na GitHubie w przeglądarce |

Kółko przewija pomoc, a jej pasek przewijania można przeciągać. Kliknięcie
poza pomocą ją zamyka.

## Prefiks nawigacji (`Ctrl+N`)

`Ctrl+N` czeka na jeszcze jeden klawisz. Pasek stanu pokazuje klawisze, które
przyjmuje. Następny klawisz uruchamia akcję albo anuluje.

| Klawisz | Działanie |
|---|---|
| `Ctrl+N` potem plakietka | Fokus na części okna, która pokazuje tę plakietkę |
| `Ctrl+N h` / `Ctrl+N l` | Zwija lub przywraca panel Refs / prawy panel |
| `Ctrl+N Left` / `Ctrl+N Right` | Przesuwa separator między panelem Refs a Log o jeden krok w lewo / w prawo (zwinięty panel Refs wraca najpierw) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Przesuwa separator między Log a prawym panelem o jeden krok w lewo / w prawo |
| `Ctrl+N +` / `Ctrl+N -` | Powiększa / zmniejsza aktywną sekcję Refs o jeden wiersz |
| `Ctrl+N ]` / `Ctrl+N [` | Powiększa / zmniejsza blok opisu commita w prawym panelu o jeden wiersz |
| `Ctrl+N t` | Wybiera kartę (selektor kart; samo `t` robi to samo) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Zmienia motyw / język / motyw kolorów grafu |
| `Ctrl+N Shift+T` | Włącza lub wyłącza odcienie wierszy grafu |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Pokazuje widok przygotowany przez agenta / wraca (zob. [MCP i agenci](mcp-and-agents.md)) |
| `Ctrl+N ,` | Otwiera Ustawienia |
| `Ctrl+N ?` | Otwiera lub zamyka pomoc |
| `Esc` albo dowolny inny klawisz | Anuluje |

### Plakietki

Gdy prefiks czeka, każda część okna, która może przyjąć fokus, pokazuje
jednoznakową plakietkę na lewej górnej krawędzi ramki. Naciśnij ten znak, by ją
ustawić jako aktywną. Plakietki są rozdawane od nowa za każdym razem, w
kolejności na ekranie: najpierw `1` do `9`, potem `a`, `b`, `c`... Czytaj więc
plakietkę z ekranu: część nie zawsze dostaje tę samą plakietkę.

Części z plakietką: filtr Refs i każda widoczna sekcja Refs, Log (albo widok
Diff), widok pliku, gdy jest otwarty, listy Unstaged i Staged, pole
podsumowania w kompozytorze commita oraz kontrolki listy plików commita. Na
karcie Home plakietki mają pole wyszukiwania, cztery sekcje i przyciski paska narzędzi (plakietka przycisku go naciska). Na karcie Radar mają je
pole filtra, listy rozwijane Zakres, Widok i Pokaż (plakietka listy rozwija ją),
przyciski Odśwież i Wstecz oraz każdy nagłówek sekcji widoczny na ekranie.
Plakietka sekcji ustawia zaznaczenie na jej nagłówku; `Enter` lub `Space` zwija
ją potem.

W oknie przeglądania folderów oraz w oknach Klonuj, Init, Utwórz worktree, Przestrzeń robocza, Częściowy checkout, Flow, Nowe zgłoszenie i pull request plakietkami są pola, pola wyboru i przyciski (plakietka przycisku go naciska, plakietka pola ustawia w nim kursor; na liście jedna plakietka stoi na jej pierwszym wierszu; okno przeglądania folderów ma jedną plakietkę dla listy Szybki dostęp i jedną dla listy Foldery, obok pierwszego wiersza każdej listy, a naciśnięcie jednej ustawia klawiaturę w tej liście bez wybierania wiersza). W Dzienniku aktywności są to dwie karty, pola Zakres i Poziom, filtr, pola wyboru poziomów, przyciski i pierwszy wpis listy. Plakietka robi to samo co kliknięcie tej kontrolki. W oknie wyboru folderu `Ctrl+N` uruchamia plakietki, więc jego klawiszem Nowy folder jest `Alt+N`.

W oknie Popraw (i Edytuj opis commita) plakietkami są podsumowanie, opis, imię, e-mail i data autora, **Zmień historię głębiej...**, **Anuluj**, **Zastosuj** i krzyżyk zamknięcia; w warsztacie przepisywania historii są to pole zakresu, jego pola wyboru i przycisk refów, nazwa przepisu, każda reguła, jej pole wyboru i przyciski, przycisk **+ Dodaj regułę**, pola wybranej reguły, karty podglądu, pole filtra, wiersze chronionych gałęzi z przyciskami Zaznacz wszystkie / Odznacz wszystkie oraz przyciski Podgląd, Zastosuj, Anuluj, Zapisz, Wczytaj i Utwórz reguły z AI (strony Sekrety, Tożsamości, Rozmiar i wynik dodają własne przyciski i wiersze; wiersze karty Commity, będące listą, plakietek nie mają). Plakietka robi to samo co kliknięcie. Nic nie dostaje plakietki, gdy otwarta jest lista rozwijana, pytanie, pole, do którego się pisze, okno wyboru folderu, ekran czyszczenia, edytor skryptu, okna AI, diff blobu lub instrukcja hostingu, ani gdy okno Popraw pyta „odrzucić?”; `Esc` zdejmuje plakietki, niczego nie zamykając.

W Ustawieniach plakietki znajdują się na polu wyszukiwania, liście stron (jedna plakietka, na bieżącej stronie), podkartach, liście i jej przyciskach, każdym polu strony widocznym na ekranie oraz krzyżyku zamykania. Plakietka robi to samo co kliknięcie tego elementu: pole tekstowe otwiera się do edycji, pole wyboru zostaje przełączone, lista rozwijana się otwiera, przycisk zostaje wykonany. `Ctrl+N` zachowuje własne znaczenie, gdy piszesz w polu albo otwarta jest lista rozwijana, edytor lub wyszukiwanie.

Paleta ma te same akcje, na przykład
`Nawigacja: Przejdź do Refs > Local` albo `Nawigacja: Przejdź do Status > Staged`.

## Repozytoria i karty

| Klawisz | Działanie |
|---|---|
| `Ctrl+O` albo kliknięcie `⌂` | Przechodzi na kartę Home (na karcie Home: fokus na polu wyszukiwania; przy otwartym Radarze lub otwartych informacjach o wersji: najpierw je zamyka) |
| `Ctrl+Shift+N` | Otwiera kartę Home z oknem Klonuj |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Następna / poprzednia karta |
| `t` | Wybiera kartę z filtrowalnej listy |
| `Ctrl+W` | Zamyka bieżącą kartę (najpierw pyta; repozytorium się nie zmienia) |
| `Ctrl+T` albo prawy klik na karcie | Menu karty (jego wiersze wymienia [Rząd kart](interface-tour.md#rząd-kart)) |
| `Alt+O` / `Ctrl+Alt+O` | Otwiera folder karty: wybór z listy **Otwórz w** / użycie pozycji domyślnej. Zob. [Otwórz w](open-in.md) |

### Karta Home

Karta `⌂` zarządza twoimi repozytoriami. Ma cztery sekcje: **Otwarte
repozytoria**, **Ulubione**, **Ostatnie repozytoria** i **Wszystkie
repozytoria**. Zob. [Przewodnik po interfejsie](interface-tour.md#home).

| Klawisz | Działanie |
|---|---|
| `j` / `k`, strzałki, `PageUp` / `PageDown`, `Home` / `End` | Ruch |
| `Enter` albo dwukrotne kliknięcie | Otwiera repozytorium (albo przełącza na jego kartę). Na nagłówku sekcji: zwija lub rozwija ją |
| `f` lub `*` albo kliknięcie gwiazdki | Dodaje do ulubionych / usuwa z ulubionych |
| `x`, `Delete` albo przycisk wiersza | Zamyka jego kartę (Otwarte), usuwa go z listy (Ostatnie), zdejmuje gwiazdkę (Ulubione), zapomina go (Wszystkie). Nic na dysku nie jest usuwane |
| `.`, prawy klik albo `⋮` wiersza | Menu wiersza |
| `/` lub `Ctrl+F` | Przeszukuje wszystkie sekcje po nazwie, gałęzi lub ścieżce. `Enter` otwiera pierwsze dopasowanie; `Esc` czyści wyszukiwanie, potem opuszcza pole |
| `Space`, `Left` / `Right` | Zwija / rozwija bieżącą sekcję |
| `-` / `+` | Zwiń wszystko / Rozwiń wszystko |
| `b` / `c` / `i` | Przeglądaj (otwórz folder) / Klonuj / Init |
| `Esc` | Czyści wyszukiwanie albo wraca do aktywnej karty repozytorium |
| `q` | Kończy program |

`Ctrl+P`, `Ctrl+,`, `?` i `Ctrl+N` działają na karcie Home. `Ctrl+G`,
`Ctrl+W` i `Ctrl+T` wymagają karty repozytorium: na karcie Home pokazują tylko
komunikat. Polecenia palety, które potrzebują repozytorium, są wyszarzone.

### Selektor folderów

**Przeglądaj** (i przycisk **Przeglądaj...** w oknach Init, Klonuj i Utwórz
worktree) otwiera selektor folderów. Ma pole ścieżki na górze, miejsca i drzewo
folderów po lewej oraz zawartość bieżącego folderu po prawej.

| Klawisz | Działanie |
|---|---|
| `Tab` / `Shift+Tab` | Pole ścieżki, potem lewa kolumna, potem zawartość |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Ruch w kolumnie z fokusem. W polu ścieżki: ruch po uzupełnieniach |
| litera (zawartość) | Przechodzi do następnego folderu zaczynającego się od tej litery |
| `Enter` / `Right` | Otwiera zaznaczony folder. W polu ścieżki: przechodzi do wpisanego folderu; na folderze już pokazanym wybiera go |
| `Backspace` / `Left` / `Alt+Up` | Folder nadrzędny (w polu ścieżki `Backspace` usuwa znak) |
| `Tab` (pole ścieżki) | Przyjmuje jedyne uzupełnienie albo podświetlone |
| `Alt+N` | Nowy folder (`Ctrl+N` uruchamia plakietki, zob. [Plakietki](#plakietki)) |
| `Alt+H` | Pokazuje lub ukrywa ukryte foldery |
| `Ctrl+F` albo `/` poza polem ścieżki | Filtruje wpisy folderu w trakcie pisania (bez rozróżniania wielkości liter; `..` zostaje; nagłówek liczy „x z n”; `Enter` skacze do pierwszego dopasowania, `Esc` czyści; filtr opróżnia się po zmianie folderu). Dostępne też przez `Tab` lub kliknięcie |
| `Ctrl+S`, `Ctrl+Enter` albo przycisk potwierdzenia | Wybiera zaznaczony podfolder, a jeśli go nie ma, pokazany folder |
| `Esc` | Zamyka uzupełnienia albo anuluje |

Gdy pole ścieżki pokazuje nową ścieżkę, cała ścieżka jest zaznaczona: pierwszy
wpisany klawisz ją zastępuje.

### Okna Init i Klonuj

| Klawisz | Działanie |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Następne / poprzednie pole lub przycisk |
| `Enter` | Naciska przycisk z fokusem albo tworzy / klonuje |
| `Space` | Przełącza pole wyboru z fokusem albo naciska przycisk z fokusem |
| `Ctrl+B` | Wybór folderu (Przeglądaj) |
| `j` / `k`, `g` / `G` | Na liście źródeł i liście repozytoriów okna Klonuj: w dół / w górę, pierwszy / ostatni (nie w polu wyszukiwania) |
| `Esc` | Zamyka. Podczas klonowania klonowanie trwa dalej w tle |

W oknie Klonuj lewa kolumna wymienia **Klonuj z URL** i jedno źródło na każdy
połączony profil hostingu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Wybierz
źródło przez `Up` / `Down` albo kliknięcie. Potem wpisuj w polu wyszukiwania
**Repozytorium**, poruszaj się po liście strzałkami, `PageUp` / `PageDown` lub
kółkiem i wybierz wiersz przez `Enter` albo kliknięcie. Pola opisuje
[Pierwsze kroki](getting-started.md).

## Panel Refs

| Klawisz | Działanie |
|---|---|
| `Enter` / `Space` | Rozwija lub zwija aktywną sekcję albo zaznaczony folder |
| `h` / `l`, `Left` / `Right` | Zmiana sekcji |
| `/` | Filtruje po nazwie (`Enter` zatwierdza, `Esc` anuluje) |
| `y` | Włącza lub wyłącza widok hierarchii gałęzi: foldery z `/` w nazwach gałęzi (LOKALNE i ZDALNE) |
| `o` | **Solo** dla zaznaczonego refa albo każdego refa pod zaznaczonym folderem lub zdalnym. Włącz solo dla kilku refów, by zobaczyć ich sumę |
| `Shift+O` | **Zakończ solo**: czyści każde solo |
| `Shift+H` | **Pokaż wszystko**: pokazuje znów każdy ukryty ref |
| `g` | **Pokaż w grafie**: przewija Log do commita zaznaczonego refu (lub stasha) i na chwilę podświetla jego wiersz; zaznaczenie się nie przesuwa |
| `m` | **Oznacza** zaznaczony ref jako źródło upuszczenia (`Esc` albo ponowne `m` na nim usuwa oznaczenie) |
| `Shift+M` | **Upuszcza oznaczony ref** na zaznaczony ref: otwiera menu, które otwiera przeciągnięcie (Merge, Rebase, Push, Fast-forward...) |
| `+` (sekcja ZDALNE) | Dodaje zdalne repozytorium |
| `.`, `Menu`, `Shift+F10` albo prawy klik | Menu wiersza: gałęzi, tagu, nazwy zdalnego albo folderu |
| `Enter` na wierszu WORKTREES | Otwiera ten worktree we własnej karcie |
| `Ctrl+G d` | Usuwa zaznaczoną gałąź lub tag (najpierw pyta). Na wierszu ZDALNE: usuwa tę gałąź zdalną |

Co robią menu, opisuje [Praca z Logiem](log-and-history.md).

### Pull requesty i zgłoszenia

Przy pasującej integracji panel Refs ma sekcję pull requestów (GitLab: merge
requesty). Przy połączonym profilu hostingu ma też sekcję ZGŁOSZENIA. Zob.
[Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md#5-pull-requesty-i-zgłoszenia-w-panelu-bocznym).

| Klawisz | Działanie |
|---|---|
| `Enter` | Otwiera zaznaczony wiersz w przeglądarce. Na wierszu `Połącz ponownie`: otwiera Ustawienia na tym profilu |
| `Space` | Rozwija lub zwija sekcję |
| `v` | Otwiera filtr sekcji. Pull requesty: Open, Merged / closed, All. Zgłoszenia: Open, Przypisane do mnie, Closed, All |
| `n` | Sekcja pull requestów: tworzy pull request. Sekcja ZGŁOSZENIA: nowe zgłoszenie. (Przycisk `+` w nagłówku) |
| `e` | Sekcja pull requestów: edytuje zaznaczony request na hoście (ołówek wiersza) |
| `Enter` na wierszu wyszukiwania | Wpisywanie w polu wyszukiwania. `Enter`, `Tab` lub `Esc` je opuszcza; `Ctrl+U` je czyści |
| `Enter` na nagłówku grupy | Zwija lub rozwija grupę (*Moje pull requesty*, *Przypisane do mnie*, *Czeka na mój review*, *Wszystkie pull requesty*) |
| `Enter` na wierszu filtra, dostawcy lub repozytorium | Otwiera tę listę rozwijaną. Lista dostawców kończy się pozycją **Brak (odłącz to repozytorium)** |
| `.` albo prawy klik | Menu wiersza: otwórz w przeglądarce, kopiuj URL lub numer, checkout gałęzi źródłowej (pull requesty); utwórz gałąź ze zgłoszenia (zgłoszenia) |

### Tablica zgłoszeń

**Tablica** w nagłówku ZGŁOSZENIA (paleta: **Zgłoszenia: Otwórz tablicę**) otwiera tablicę Kanban źródła zgłoszeń. Zob.
[tablicę](hosting-integration.md#5b-tablica-zgłoszeń) i [zgłoszenia lokalne](local-issues.md#tablica-kanban). Każdy
poniższy klawisz jest też poleceniem palety (**Tablica: ...**).

| Klawisz | Działanie |
|---|---|
| Strzałki | Zaznaczają kartę (`Up`, `Down`) albo sąsiednią kolumnę (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` skaczą w kolumnie |
| `Shift+Left`, `Shift+Right` | Przenoszą zaznaczoną kartę do poprzedniej lub następnej kolumny |
| `m` | **Tablica: Przenieś kartę do...**: lista rozwijana kolumn |
| `Enter` | Otwiera zgłoszenie (notatkę, dla zgłoszenia lokalnego; w przeciwnym razie stronę hosta) |
| `e`, `.` | Edytuje przypisanych, etykiety lub kamień milowy; menu karty |
| `n` | Nowe zgłoszenie w zaznaczonej kolumnie |
| `a`, `l`, `M`, `/` | Filtrują według przypisanego, etykiety, kamienia milowego, tekstu. `x` czyści filtry |
| `c` | Etykiety, które są kolumnami tablicy GitHub / GitLab |
| `r` | Czyta zgłoszenia ponownie |
| `Esc` | Zamyka tablicę |

Mysz: przeciągnij kartę na inną kolumnę, aby ją przenieść, kliknij kartę, aby ją zaznaczyć, kliknij dwukrotnie, aby ją otworzyć, kliknij prawym przyciskiem
dla menu karty, kółko przewija kolumnę, `Shift`+kółko (albo pasek pod tablicą) przewija kolumny.

### Flow (przepływy pracy z gałęziami)

Gdy dla repozytorium skonfigurowano przepływ pracy (zob. [przepływy pracy z gałęziami](branching-workflows.md)), pasek narzędzi
ma przycisk **Flow**, a pasek boczny grupuje LOKALNE i ZDALNE według typu gałęzi. Każdy wiersz menu jest poleceniem palety
(**Flow: ...**; **Flow: Rozpocznij Feature...** i **Flow: Nowe zgłoszenie Feature...** są generowane dla każdego typu gałęzi
tematycznych przepływu pracy). Wiersze sterowane przez host (*Nowe zgłoszenie <Typ>*, *Przejmij gałąź utworzoną przez hosta*)
i ich okna, karta oczekiwania i karta „pojawiła się gałąź” są opisane w
[przepływach pracy z gałęziami](branching-workflows.md#przepływy-sterowane-przez-host-w-aplikacji);
`Enter` uruchamia aktywny przycisk (*Utwórz zgłoszenie*, *Przejmij*), `Esc` przestaje czekać lub zamyka.

| Klawisz | Działanie |
|---|---|
| `Ctrl+G Shift+W` | Otwiera menu Flow (przycisk paska narzędzi) |
| `Enter`, `Tab`, `Shift+Tab` | W oknie Rozpocznij / Zakończ / Synchronizuj / Promuj / Ustaw nadrzędną: podgląd lub uruchomienie, następne pole, poprzednie pole |
| `Space` | Zaznacza pole wyboru (*Opublikuj*, *Uruchom hooki*, *Zaktualizuj też gałęzie oparte na niej*) |
| `Esc` | Zamyka listę rozwijaną, wraca z podglądu do formularza, zamyka okno (nie w trakcie uruchomienia) |
| Najechanie na gałąź | Jej typ, gałąź nadrzędna i o ile jest za gałęzią nadrzędną |

## Panel Log

| Klawisz | Działanie |
|---|---|
| `Enter` / `l` | Wchodzi do listy zmienionych plików zaznaczonego commita |
| `Enter` / `l` na wierszu pliku | Otwiera diff pliku pośrodku |
| `Esc` / `h` | Wraca do commitów |
| `Right` | (commit z więcej niż jednym refem) Otwiera listę jego pozostałych refów: `Up` / `Down` przesuwają, `Enter` robi checkout, `Esc` zamyka |
| `f` | Otwiera pasek wyszukiwania (zob. [Wyszukiwanie](#wyszukiwanie)) |
| `c` | (przy niezatwierdzonych zmianach) Zaznacza wiersz WIP i ustawia kursor w jego polu opisu |
| `Enter` na wierszu WIP | Ustawia kursor w polu opisu wiersza WIP |
| `.`, `Menu`, `Shift+F10` | Menu commita. `Up` / `Down` przesuwają, `Right` / `l` / `Enter` otwierają podmenu, `Left` / `h` / `Esc` wracają |
| `m` / `a` | Pliki prawego panelu: `Ścieżka` lub `Drzewo` / `Pokaż wszystkie pliki` |
| `Shift+C` | Wybiera kolumnę Logu do przesunięcia lub zmiany szerokości (zob. niżej) |

Przy wybranej kolumnie `Tab` / `Shift+Tab` wybierają inną, `<` / `>` przesuwają ją, `]` / `[` poszerzają lub zwężają, `0` przywraca wszystkim szerokość automatyczną, a `Esc` lub `Enter` kończy (kontekst `log_columns` w wygenerowanej tabeli wyżej).

Pole opisu wiersza WIP to podsumowanie kompozytora commita. Gdy ma kursor,
każdy klawisz jest wpisywany, ale skróty `Ctrl` nadal działają. `Enter`
przechodzi do opisu szczegółowego kompozytora. `Esc` opuszcza pole i zachowuje
tekst. Zob. [Stage i commity](staging-and-committing.md).

### Zaznaczanie wielu commitów

| Klawisz lub gest | Działanie |
|---|---|
| `Space` | Dodaje bieżący commit do zaznaczenia albo go z niego usuwa |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Rozszerza zaznaczenie (Alt dla terminali, które zachowują Shift dla siebie) |
| `Ctrl+klik` / `Shift+klik` lub `Alt+klik` | Przełącza commit / zaznacza zakres |
| `Esc` | Czyści zaznaczenie |
| `Shift+Y` | [Kopiuje zaznaczenie jako gitGraph Mermaid](log-and-history.md#kopiowanie-commitów-jako-gitgraph-mermaid) (okno z poziomami szczegółów i podglądem na żywo; `Enter` kopiuje) |

Przy kilku zaznaczonych commitach kliknij jeden z nich prawym przyciskiem i
wybierz **Cherry-pick N commitów na `<branch>`...**. Otwiera się ten sam widok
co przy interaktywnym rebase.

### Nazywanie gałęzi lub tagu

`Ctrl+G b`, `Ctrl+G t`, przyciski **Gałąź** i **Tag** na pasku narzędzi oraz
pozycje menu **Utwórz gałąź tutaj...** / **Utwórz tag tutaj...** pytają o nazwę
w polu tekstowym w wierszu commita. `Enter` tworzy ref. `Esc` anuluje.
Nieprawidłowa lub istniejąca nazwa jest pokazana na czerwono, a pole zostaje
otwarte.

### Edycja opisu commita

**Edytuj opis commita...** w menu commita, `Ctrl+G e` albo polecenie palety
`Git: Edytuj opis commita...` otwiera okno **Zmień opis commita**. Co zmienia,
opisuje [Praca z Logiem](log-and-history.md).

| Klawisz | Działanie |
|---|---|
| `Enter` w podsumowaniu | Przechodzi do opisu szczegółowego |
| `Enter` w opisie szczegółowym | Nowa linia. `Up` w pierwszej linii wraca do podsumowania |
| `Ctrl+Enter`, `Ctrl+S` albo **Zaktualizuj opis** | Zapisuje nowy opis |
| `Tab` / `Shift+Tab` | Przechodzi między podsumowaniem, opisem szczegółowym i przyciskami |
| `Esc` albo **Anuluj** | Zamyka bez zmian |

### Poprawianie commita

`Ctrl+G Shift+E` (wszystkie pola) i `Ctrl+G Shift+A` (autor) otwierają okno Popraw, podobnie jak menu commita, wiersze `Commit: Popraw...` w palecie i panel szczegółów commita; wszystkie drogi opisuje [Poprawianie commita](log-and-history.md#poprawianie-commita-opis-autor-daty). Oba skróty można przypisać na nowo: `git_prefix.amend` i `git_prefix.edit_author`. Klawisze w oknie:

| Klawisz | Działanie |
|---|---|
| `Tab` / `Shift+Tab` | Przechodzi między polami i przyciskami |
| `Enter` w polu jednowierszowym | Następne pole; po ostatnim polu: **Zastosuj** |
| `Up` / `Down` | Poprzednie / następne pole (w opisie od pierwszej / ostatniej linii) |
| `Ctrl+Enter`, `Ctrl+S` albo **Zastosuj** | Stosuje zmienione pola |
| `Esc` albo **Anuluj** | Zamyka bez zmian |
| `Alt+H` albo **Zmień historię głębiej...** | Zamyka okno i otwiera warsztat przepisywania historii od tego commita (tylko w kompilacjach z przepisywaniem historii); jeśli zmieniono jakieś pole, najpierw pyta (`Esc` potwierdza). Można przypisać na nowo: `reword.deeper` |

### Utwórz worktree

**Utwórz worktree z `<ref>`...** (menu gałęzi i tagu) oraz **Utwórz worktree
tutaj...** (menu commita) otwierają okno: folder, opcjonalna nowa gałąź i
**Otwórz nowy worktree jako kartę**. Klawisze: `Tab` / `Shift+Tab` przesuwają,
`Enter` tworzy, `Space` przełącza pole wyboru, `Ctrl+B` otwiera wybór folderu,
`Esc` anuluje.

### Interaktywny rebase

**Interaktywny rebase stąd...** w menu commita albo
`Git: Interaktywny rebase od zaznaczonego commita...` otwiera widok rebase.
Cherry-pick wielu commitów używa tego samego widoku. Zob.
[Praca z Logiem](log-and-history.md).

| Klawisz | Działanie |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Ustawia akcję wiersza: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup i Edit tylko w rebase) |
| `Space` | Otwiera listę akcji wiersza (`Up` / `Down` wybierają, `Enter` stosuje, `Esc` zamyka listę) |
| `Up` / `Down` | Przesuwają zaznaczenie |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Przesuwają zaznaczony wiersz w górę / w dół |
| `a` | Włącza lub wyłącza podgląd autosquash |
| `Enter` na wierszu Reword | Edytuje nowy opis tego commita |
| `Ctrl+S`, `Ctrl+Enter` albo **Uruchom rebase** / **Uruchom cherry-pick** | Uruchamia plan |
| `Ctrl+R` albo **Resetuj** | Przywraca plan takim, jaki był przy otwarciu widoku |
| `Esc` albo **Anuluj rebase** | Zamyka bez zmian |

Gdy rebase się zatrzyma (na konflikcie lub kroku Edit), baner nad Logiem oferuje
**Kontynuuj**, **Pomiń** i **Przerwij**. Paleta ma `Rebase: Kontynuuj`,
`Rebase: Pomiń` i `Rebase: Przerwij`.

## Wyszukiwanie

`f`, przycisk **Szukaj** na pasku narzędzi albo `Nawigacja: Szukaj commitów...`
otwiera pasek wyszukiwania nad Logiem. Przeszukuje cały opis commita, imię i
e-mail autora, SHA oraz nazwy refów. Nie ukrywa wierszy: wiersze, które nie
pasują, są przyciemnione.

| Klawisz | Działanie |
|---|---|
| *(wpisywanie)* | Edytuje tekst wyszukiwania (każdy klawisz jest wpisywany) |
| `Enter` / `Down` albo `↑` paska | Następne dopasowanie |
| `Shift+Enter` / `Up` albo `↓` paska | Poprzednie dopasowanie |
| `Esc` albo `✕` paska | Zamyka pasek wyszukiwania |

## Widok Diff

Zmieniony plik otwiera się pośrodku, zamiast Logu. Zob.
[Stage i commity](staging-and-committing.md).

| Klawisz | Działanie |
|---|---|
| `Enter` / `l` albo dwukrotne kliknięcie (na wierszu pliku) | Otwiera diff pliku |
| `Esc`, `q` albo `✕` w tytule | Zamyka diff |
| `j` / `k`, `Up` / `Down` | Przewijanie |
| `PageUp` / `PageDown`, `Home` / `End` | Przewija o stronę; skacze na początek / koniec |
| `h` / `l`, `Left` / `Right` | Przewija w bok (nie przy zawijaniu wierszy) |
| `n` / `p` | Następny / poprzedni hunk |
| `m` | Układ inline lub split |
| `Shift+H` | Tylko hunki albo cały plik |
| `w` | Włącza lub wyłącza zawijanie wierszy |
| `s` | Pokazuje lub ukrywa zmiany tylko w białych znakach |
| `Shift+M` | Włącza lub wyłącza minimapę |
| `Shift+C` | Wybiera kodowanie, w jakim czytany jest plik (lista: najpierw Wykryj automatycznie) |
| `e` | Edytuje plik we wbudowanym edytorze (zob. [Edytor](#edytor)) |
| `r` | (plik Git LFS) Karta LFS albo surowy wskaźnik. Zob. [Git LFS](lfs.md) |
| `Shift+E` | (plik z konfliktem) Otwiera edytor merge. Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Kopiuje zaznaczony tekst. Bez zaznaczenia: linię pod kursorem albo zaznaczone linie |
| `.` albo prawy klik na hunku | Menu hunka |

### Stage i odrzucanie z poziomu diffa

Te klawisze działają tylko w diffie zmian roboczych (otwartym z listy
Unstaged lub Staged). Odrzucanie działa tylko na liście Unstaged.

| Klawisz | Działanie |
|---|---|
| `Space` | Stage (Unstaged) / unstage (Staged) hunka pod kursorem |
| `v` | Zaczyna lub porzuca zaznaczenie linii; `j` / `k` je rozszerzają |
| `Enter` | Stage / unstage zaznaczonych linii albo linii pod kursorem |
| `x` | Odrzuca hunk pod kursorem (najpierw pyta: `y` / `Enter` potwierdzają, `n` / `Esc` anulują) |
| `Shift+X` | Odrzuca zaznaczone linie albo linię pod kursorem (najpierw pyta) |

Liczą się tylko zmienione linie: linie kontekstu w zaznaczeniu są ignorowane.
Zaznaczenie sięgające drugiego hunka działa tylko na pierwszym hunku.

## Widok obrazu

Obraz w widoku Diff albo w widoku pliku. Klawisze są w kontekście `image`
tabeli powyżej. Zob. [Obrazy](image-view.md).

## Widok pliku, Blame i historia pliku

| Klawisz | Działanie |
|---|---|
| `Esc` / `q` | Zamyka (przy zaznaczeniu tekstu `Esc` najpierw je czyści) |
| `j` / `k`, `Up` / `Down` | Przesuwają zaznaczenie |
| `PageUp` / `PageDown` | Przewijają o stronę |
| `Home` / `End`, `Shift+G` | Pierwsza / ostatnia linia |
| `h` / `l`, `Left` / `Right` | Przewijają tekst w bok |
| `Enter` | (Blame, historia pliku) Przechodzi do commita wiersza w Log |
| `e` | Edytuje plik we wbudowanym edytorze |
| `y` / `Ctrl+C` | Kopiuje zaznaczony tekst. Bez zaznaczenia: zaznaczony wiersz (Blame: tylko kod; Historia pliku: podsumowanie) |

Blame i historia pliku mają własne klawisze, wymienione w wygenerowanej tabeli powyżej (konteksty `file_insight` i `history`) oraz jako `Blame: ...` / `Historia pliku: ...` w palecie; wszystkie można przypisać na nowo. Szczegóły w [Blame](log-and-history.md#blame) i [Historia pliku](log-and-history.md#historia-pliku). Na liście historii same litery (`d` `o` `b` `c` `y`) działają na wybrany wiersz, ale akord z Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) zawsze zachowuje swoje globalne znaczenie.

## Kolumna commitów

Kolumna po lewej stronie widoku pliku (zob. [Kolumna commitów](interface-tour.md#kolumna-commitów)). `Alt+C` pokazuje ją lub ukrywa z dowolnego miejsca; pozostałe klawisze działają, gdy kolumna ma klawiaturę (`Ctrl+N` i jej odznaka albo `Widok: Przenieś fokus na kolumnę commitów`).

| Klawisz | Działanie |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Wybierają commit |
| `Enter` / `Right` | Przechodzi do listy plików wybranego commita |
| `Esc` | Oddaje klawiaturę widokowi |
| `.`, `Menu`, `Shift+F10` | Menu wybranego wiersza |

Jej pozostałe klawisze (`Shift+Left` / `Shift+Right` do przewijania na boki, `Tab`, `]`, `[` i `0` do rozmiaru części) są w wygenerowanej tabeli powyżej, kontekst `commit_column`.

## Edytor

**Edytuj** albo `e` w diffie lub widoku pliku otwiera wbudowany edytor na
kopii pliku z drzewa roboczego. Zob.
[Stage i commity](staging-and-committing.md).

| Klawisz | Działanie |
|---|---|
| *(wpisywanie)* | Wstawia w miejscu kursora (zastępuje zaznaczenie) |
| `Enter` | Nowa linia, z wcięciem bieżącej linii |
| `Tab` / `Shift+Tab` | Zwiększa / zmniejsza wcięcie (zaznaczonych linii, jeśli jest ich kilka) |
| strzałki, `Home` / `End` | Ruch. `Home` idzie do pierwszego niepustego znaku, potem do kolumny 0. `Shift` rozszerza zaznaczenie |
| `Ctrl+Left` / `Ctrl+Right` | Przesuwa o jedno słowo |
| `Ctrl+Home` / `Ctrl+End` | Początek / koniec pliku |
| `PageUp` / `PageDown` | Jedna strona w górę / w dół |
| `Ctrl+A` | Zaznacza wszystko |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopiuj / wytnij / wklej |
| `Ctrl+U` / `Ctrl+K` | Usuwa do początku / końca linii |
| `Ctrl+Z` | Cofnij |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Ponów |
| `Ctrl+S` | Zapisz |
| `Ctrl+F` / `F3` | Szukaj. W polu szukania: `Enter` / `Down` / `F3` następne, `Shift+Enter` / `Up` poprzednie, `Esc` zamyka |
| `Alt+M` | Włącza lub wyłącza minimapę (**Map**) |
| `Esc` | Zamyka. Przy niezapisanych zmianach pyta: **Zapisz** (`S` / `Enter`), **Odrzuć** (`D`), **Anuluj** (`C` / `Esc`) |

`Tab` w edytorze wcina, więc nie przesuwa fokusu. Inne skróty `Ctrl`
(`Ctrl+P`, `Ctrl+N`) nadal działają.

**Kodowanie**, **końce wierszy** i **wcięcia** na pasku stanu to przyciski: każdy otwiera listę rozwijaną
(strzałki lub kółko, wpisz tekst, aby filtrować, `Enter` wybiera, `Esc` zamyka). Paleta (`Ctrl+P`) otwiera te same
menu: *Edytor: Zmień kodowanie*, *Edytor: Otwórz ponownie z kodowaniem*, *Edytor: Konwertuj na kodowanie*, *Edytor: Konwertuj końce wierszy*, *Edytor: Zmień wcięcia*. Szczegóły w [Stage i commity](staging-and-committing.md).

## Akcje Git (`Ctrl+G`)

`Ctrl+G` czeka na jeszcze jeden klawisz:

| Klawisz | Działanie |
|---|---|
| `s` / `u` | Stage / unstage zaznaczonego pliku |
| `a` | Stage wszystkich zmian |
| `x` | Odrzuca wszystkie zmiany (najpierw pyta) |
| `c` | Otwiera kompozytor commita |
| `Enter` | Od razu robi commit zmian ze stage, z opisem z formularza, z dowolnego miejsca (zob. [Stage i commity](staging-and-committing.md#pisanie-commita)) |
| `m` | Stash wszystkich zmian z opisem (pusty jest dozwolony). Przy włączonym AI `Ctrl+G` w tym polu pisze opis |
| `Shift+T` | Stash wszystkich zmian (łącznie z nieśledzonymi plikami), bez opisu |
| `o` | Pop stasha: wiersza stasha zaznaczonego w Log, a w przeciwnym razie najnowszego |
| `p` | Pull, tak jak ustawiono przycisk Pull (`git.pull_mode`) |
| `Shift+P` | Push do domyślnego zdalnego (przy pierwszym pushu ustawia upstream) albo pyta o zdalne |
| `Shift+U` | Push wszystkich tagów |
| `f` | Fetch wszystkich zdalnych, z prune |
| `g` | Otwiera pull request bieżącej gałęzi w przeglądarce |
| `b` / `t` | Nowa gałąź / nowy tag w HEAD (zob. [Nazywanie gałęzi lub tagu](#nazywanie-gałęzi-lub-tagu)) |
| `d` | Usuwa zaznaczoną gałąź lub tag (najpierw pyta) |
| `e` | Edytuje opis zaznaczonego commita |
| `Shift+E` / `Shift+A` | Poprawia zaznaczony commit (wszystkie pola) / jego autora (zob. [Poprawianie commita](#poprawianie-commita)) |
| `y` | Cherry-pick zaznaczonego commita |
| `v` | Revert zaznaczonego commita |
| `Shift+R` | Reset (mixed) do zaznaczonego commita |
| `r` | Rebase na zaznaczoną gałąź |
| `Shift+W` | Otwiera menu Flow (w repozytorium z przepływem pracy z gałęziami) |
| `Shift+H` | Otwiera warsztat przepisywania historii (tylko w kompilacjach z przepisywaniem historii) |
| `l` | Otwiera menu LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. Zob. [Git LFS](lfs.md) |
| `i` | ✦ AI: pisze opis commita dla zmian w stage |
| `w` | ✦ AI: wyjaśnia zaznaczony commit albo zmiany robocze |
| `Shift+C` | ✦ AI: proponuje, jak podzielić zmiany robocze na commity |

Klawisze AI działają tylko przy włączonym AI (zob. [Wsparcie AI](ai.md)).
`Esc` anuluje trwające zapytanie AI.

## Prawy panel (zmiany robocze i szczegóły commita)

Przy zaznaczonym wierszu WIP prawy panel pokazuje listy **Unstaged** i
**Staged** oraz kompozytor commita. Przy zaznaczonym commicie pokazuje
szczegóły commita i jego pliki.

| Klawisz | Działanie |
|---|---|
| `j` / `k`, `Up` / `Down` | Ruch. Na końcu jednej listy zaznaczenie przechodzi do drugiej |
| `h` / `l`, `Left` / `Right` | Przechodzi do listy Unstaged / Staged |
| `Enter` | Zwija lub rozwija aktywną listę |
| `Space` / `Insert` | Dodaje bieżący plik do zaznaczenia albo go usuwa |
| `Ctrl+A` | Zaznacz wszystkie pliki listy |
| `Ctrl+Space` | Stage / unstage zaznaczonego pliku albo zwija wiersz folderu |
| `d` | Otwiera diff zaznaczonego pliku |
| `m` | `Ścieżka` (lista płaska) lub `Drzewo` (foldery) |
| `a` | `Pokaż wszystkie pliki`: całe drzewo, nie tylko zmienione pliki |
| `z` | Zwija wszystkie foldery albo je znów rozwija |
| `/` | Filtruje zmienione pliki commita po nazwie |
| `.`, `Menu`, `Shift+F10` | Menu pliku |
| `c` | Fokus na polu podsumowania kompozytora commita |
| `Shift+A` | Włącza lub wyłącza **Popraw poprzedni commit** |
| `Ctrl+N ]` / `Ctrl+N [` | Powiększa / zmniejsza blok opisu commita |

W kompozytorze commita:

| Klawisz | Działanie |
|---|---|
| `Enter` | Podsumowanie: przechodzi do opisu szczegółowego. Opis szczegółowy: nowa linia |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` robi to z dowolnego miejsca) |
| `Alt+R` | Wstawia odwołanie do zgłoszenia |
| `Esc` | Opuszcza kompozytor |

Menu pliku zawiera: Stage lub Unstage, Odrzuć zmiany, Ignoruj, Stash pliku,
Historia pliku, Blame pliku, Otwórz w zewnętrznym narzędziu diff, Otwórz w
zewnętrznym edytorze, Otwórz plik w programie domyślnym, Pokaż w folderze,
Kopiuj ścieżkę pliku, Utwórz patch ze zmian w pliku, Edytuj plik i Usuń plik.
Na pliku z konfliktem zaczyna się od **Otwórz w edytorze merge** i trzech
wierszy **Rozwiąż używając**. Przy włączonym Git LFS ma wiersze LFS. **Odrzuć
zmiany** i **Usuń plik** najpierw pytają.

## Przyciski okna

Każdy przycisk ma odpowiednik na klawiaturze:

| Przycisk | Klawisz |
|---|---|
| 🐙 po lewej stronie paska tytułu (O programie) | `Pomoc: O programie GitPoulpi` w palecie |
| **Otwórz w** na pasku tytułu | `Alt+O`, `Ctrl+Alt+O` |
| `×` po prawej stronie paska tytułu (zakończ) | `q` |
| Wersja na pasku tytułu (Informacje o wersji) | `Pomoc: Informacje o wersji` w palecie |
| `⌕ Szukaj wszędzie` na pasku tytułu, na lewo od `⚙` | `Ctrl+Alt+F` |
| `⚙` na pasku tytułu (Ustawienia) | `Ctrl+,` |
| Plakietka obszaru roboczego w rzędzie kart | `Alt+W` |
| `+` za ostatnią kartą | `Alt+T` |
| Karta **Radar** | `Alt+R` |
| Karta **Obszary robocze** | `Alt+J` |
| **Aktywność** na pasku stanu | `Ctrl+N Shift+A` |
| Pasek narzędzi: **Cofnij**, **Ponów** | `Ctrl+Z`, `Ctrl+Y` |
| Pasek narzędzi: **Pull** / jego `▾` | `Ctrl+G p` / `Git: Opcje pull...` |
| Pasek narzędzi: **Push** / jego `▾` | `Ctrl+G Shift+P` / `Git: Push do zdalnego...` |
| Pasek narzędzi: **Gałąź**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Pasek narzędzi: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Pasek narzędzi: **LFS** (gdy LFS jest włączony) | `Ctrl+G l` |
| Pasek narzędzi: **Flow** (gdy repozytorium ma przepływ pracy) | `Ctrl+G Shift+W` |
| Pasek narzędzi: **Szukaj** | `f` |
| Pasek narzędzi: powiązany pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Pasek narzędzi: lista profili (przy dwóch lub więcej profilach) | `Repozytorium: Zmień profil...` |

Pasek narzędzi opisuje [Przewodnik po interfejsie](interface-tour.md#pasek-narzędzi).

## Ustawienia

`Ctrl+,`, `Ctrl+N ,`, koło zębate `⚙` albo `Aplikacja: Ustawienia...`
otwiera ekran Ustawień.

| Klawisz | Działanie |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Przechodzą między listą stron a opcjami |
| `j` / `k`, `Up` / `Down` | Przesuwają zaznaczenie |
| `Enter` / `Space` | Otwiera stronę, przełącza pole wyboru, edytuje pole, otwiera listę rozwijaną albo naciska przycisk |
| `/`, `Ctrl+F` | Przeszukuje wszystkie opcje |
| `Alt+Left` / `Alt+Right` | Przełącza podkartę (Ogólne / lista) |
| `Esc` / `q` | Zamyka Ustawienia. Podczas edycji pola `Esc` anuluje tylko edycję |
| `?` | Otwiera pomoc nad ekranem Ustawień |

Zob. [Ustawienia](settings.md).

## Pola tekstowe

Wszystkie pola tekstowe mają te same klawisze: filtry, pola wyszukiwania,
paleta poleceń, okna z pytaniami, kompozytor commita, pole wiersza WIP, pola
okien i pola Ustawień. Gdy pole ma klawiaturę, te klawisze mają pierwszeństwo
przed skrótami paneli.

| Klawisz | Działanie |
|---|---|
| znak | Wpisuje go w miejscu kursora (zastępuje zaznaczenie) |
| `Left` / `Right` | Przesuwa o jeden znak (przy zaznaczeniu: na jego początek / koniec) |
| `Ctrl+Left` / `Ctrl+Right` | Przesuwa o jedno słowo |
| `Home` / `End` | Początek / koniec linii |
| `Shift` + powyższy ruch | Rozszerza lub zmniejsza zaznaczenie |
| `Up` / `Down` (tylko opis szczegółowy) | Poprzednia / następna linia |
| `Ctrl+Home` / `Ctrl+End` (tylko opis szczegółowy) | Początek / koniec tekstu |
| `Ctrl+A` | Zaznacza wszystko |
| `Backspace` / `Delete` | Usuwa znak przed / za kursorem albo zaznaczenie |
| `Ctrl+U` | Usuwa od kursora do początku linii (albo zaznaczenie) |
| `Ctrl+K` | Usuwa od kursora do końca linii (albo zaznaczenie) |
| `Ctrl+C` / `Ctrl+X` | Kopiuje / wycina zaznaczenie (nie w polu hasła) |
| wklejenie (własne wklejanie terminala) | Wstawia w miejscu kursora, zastępując zaznaczenie |

W opisie szczegółowym commita `Enter` wpisuje nową linię, a `Ctrl+Enter` robi
commit. Nie wszystkie terminale wysyłają `Ctrl+Enter` tak samo, więc commit
robią też `Alt+Enter` i `Ctrl+J`.

Niektóre konsole wysyłają `Ctrl+litera` jako znak sterujący (`Ctrl+A` jako
`0x01`). GitPoulpi rozumie obie formy. Znaki `AltGr` (`@`, `#`, `€` na
szwajcarskich lub francuskich klawiaturach) są wpisywane.

## Podpowiedzi

Podpowiedź pojawia się, gdy wskaźnik zostaje na celu około 0,6 s (komórka Log,
przycisk paska narzędzi, powiązany pull request). Naciśnięcie klawisza,
kliknięcie, przewinięcie, zmiana rozmiaru, otwarte okno lub menu albo
opuszczenie celu przez wskaźnik ją ukrywa. Potem wraca dopiero po ponownym
poruszeniu wskaźnikiem.
