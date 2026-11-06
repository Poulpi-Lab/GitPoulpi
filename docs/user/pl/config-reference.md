[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · **Polski** · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Opis konfiguracji

Każdy klucz `config.toml`, jego wartość domyślna i działanie. Rzadko trzeba
edytować plik ręcznie: [ekran Ustawień](settings.md) (`Ctrl+,`) zapisuje go
przy każdej zmianie, a robi to także `gitpoulpi config set`.

## Gdzie jest plik i jak wybierana jest wartość

| | |
|---|---|
| Plik | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` wypisuje jego położenie. |
| Inny folder | `GITPOULPI_CONFIG_DIR=<dir>` przenosi cały folder konfiguracji (plik, `sessions/`, pliki portów demona): instalacja przenośna, test albo agent trzymany z dala od własnych ustawień. |
| Pierwszeństwo | wartość wbudowana, potem `config.toml`, potem zmienna `GITPOULPI_*`, potem flaga wiersza poleceń. Wygrywa najwyższe. Błędna wartość (literówka w pliku, zła zmienna) jest błędem, nigdy nie jest po cichu ignorowana. |
| Zmienne | Zapisz klucz wielkimi literami i zamień kropki na podkreślenia: `general.auto_prune` to `GITPOULPI_GENERAL_AUTO_PRUNE`. Tylko dla kluczy, które GitPoulpi faktycznie czyta (nie dla oznaczonych niżej jako *nieużywane* ani list, jak `ui.log_columns_hidden`). |
| Flagi | `--backend git2\|system` nadpisuje `general.default_backend` dla demona uruchamianego przez to polecenie. |
| Tokeny i klucze | Nigdy w tym pliku. Tokeny hostingu i klucze API AI trafiają do systemowego menedżera poświadczeń (Git Credential Manager, libsecret / `pass`, Keychain); plik zawiera tylko odwołanie (`token_ref`). Bez menedżera poświadczeń zapis tokenu kończy się komunikatem, a token żyje tylko do końca sesji. |

```
gitpoulpi config path                          # plik i to, czy już istnieje
gitpoulpi config get general.auto_stash        # wartość, warstwa, z której pochodzi (default/file/env/flag), i to, czy cokolwiek ją czyta
gitpoulpi --json config get ui                 # cała sekcja
gitpoulpi config set general.auto_stash always # sprawdzane ze schematem, potem zapisywane
```

`config set` utrzymuje plik oszczędnym (tylko klucze, które ktoś ustawił)
i zachowuje klucze, o które go nie pytano, ale usuwa komentarze. Odmawia list
i całych tabel, z wyjątkiem `keybindings.*` (kilka skrótów oddzielonych
spacjami; puste odpina). `[[profiles]]`, `[[integrations]]` i `[[ai.profiles]]`
edytuj w Ustawieniach albo ręcznie.

W tabelach **Restart** oznacza, że wartość jest czytana przy starcie; wszystko
inne działa od razu (lub przy następnym odświeżeniu, fetch albo zamknięciu
Ustawień).

## `[general]`

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, z powrotem do programu `git` dla staging, merge, rebase, stash i sieci) lub `system` (tylko zainstalowany `git`). Działający demon zachowuje swój backend: **zrestartuj** go (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Gdy lokalne zmiany blokują checkout, pull, merge lub rebase: `ask` (okno w TUI; CLI i MCP kończą się błędem i wskazują `--autostash`), `always` (stash, wykonanie, pop) lub `never` (odmowa). Zob. [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | Auto-stash obejmuje też pliki nieśledzone. |
| `general.initial_commits_in_graph` | `5000` | Commity wczytywane przez Log na początku (minimum 500). Więcej kosztuje pamięć i czas rysowania w ogromnych repozytoriach. |
| `general.lazy_load_commits_in_graph` | `true` | Wczytuje starsze commity, gdy przewijasz blisko końca. Wyłączone: Log zatrzymuje się na liczbie początkowej. |
| `general.commits_per_lazy_page` | `1000` | Commity na stronę wczytywaną w tle (minimum 50). |
| `general.max_commits_in_graph` | `100000` | Gdzie zatrzymuje się leniwe wczytywanie (około 300 bajtów na commit); `0` = bez limitu. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch wszystkich zdalnych w tle co N minut, od 0 do 60; `0` = wyłączone. **Restart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują interwał: `minutes`, `hours` albo `auto`. Tylko wyświetlanie: interwał to zawsze `auto_fetch_interval_minutes` minut. `auto` (i konfiguracja bez tego klucza) wybiera największą jednostkę, która dzieli minuty bez reszty (60 czyta się „1 godzina”, 45 „45 minut”); jednostka, która nie pasuje, jest ignorowana. Ustawienia zapisują oba klucze; starszy GitPoulpi ignoruje jednostkę i nadal czyta minuty. |
| `general.fetch_on_tab_open` | `true` | Fetch wszystkich zdalnych po otwarciu karty (pomijany, jeśli GitPoulpi pobrał to repozytorium mniej niż 60 s temu). |
| `general.auto_prune` | `false` | Fetch usuwa też gałęzie śledzące zdalnych repozytoriów, których już nie ma. |
| `general.auto_prune_tags` | `false` | Fetch usuwa też lokalne tagi, które zniknęły ze zdalnego repozytorium (tylko te, które na nim były; tag, którego nigdy nie wypchnięto, zostaje). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Odświeżanie list pull requestów / zgłoszeń i znacznika na pasku narzędzi, od 0 do 120; `0` = tylko na żądanie i przy zdarzeniach. |
| `general.hosting_refresh_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują odświeżanie: `minutes`, `hours` albo `auto`, czytana tak jak `general.auto_fetch_interval_unit`. Tylko wyświetlanie. |
| `general.keep_submodules_up_to_date` | `false` | Po checkout lub pull `git submodule update --init --recursive`, gdy submoduł jest nieaktualny. Zob. [Submoduły](submodules.md). |
| `general.default_branch_name` | `"main"` | Pierwsza gałąź repozytorium utworzonego przez Init; puste zostawia decyzję gitowi. Niczego nie zmienia. |
| `general.delete_orig_files_after_merge` | `false` | Po udanym merge usuwa pliki kopii zapasowych `*.orig`. |
| `general.remember_tabs` | `true` | Otwiera ponownie karty z ostatniej sesji. Gdy wyłączone, start wewnątrz repozytorium otwiera tylko to repozytorium, a start poza repozytorium otwiera tylko kartę Start. **Restart.** |
| `general.radar_enabled` | `true` | [Radar](radar.md): jego karta, polecenia palety, `Alt+R` i pobieranie w tle. Wyłączone ukrywa to wszystko i nic się dla niego nie pobiera. Działa od razu; nie jest pokazywane w kompilacji bez funkcji `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | Karta, na której otwiera się start, gdy folder, z którego go uruchomiono, leży w repozytorium: `"current-repo"` (karta tego repozytorium, w razie potrzeby otwierana), `"last"` (karta aktywna przy ostatnim zamknięciu), `"home"` lub `"radar"`. Repozytorium podane przez `--repo` zawsze otwiera własną kartę. `"last"` to Home przy wyłączonym `remember_tabs` lub gdy nic nie zapamiętano; `"radar"` to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Czytane raz przy starcie; wymaga restartu. |
| `general.startup_tab_outside_repo` | `"home"` | Karta, na której otwiera się start, gdy folder, z którego go uruchomiono, nie jest repozytorium (na przykład folder użytkownika): `"home"`, `"last"` lub `"radar"`. `"last"` to Home przy wyłączonym `remember_tabs` lub gdy nic nie zapamiętano; `"radar"` to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Czytane raz przy starcie; wymaga restartu. |
| `general.startup_tab_launcher` | `"last"` | Karta, na której otwiera się start, gdy GitPoulpi uruchomiono ze skrótu na pulpicie lub w menu Start, który sam utworzył (przekazuje `--launcher`), niezależnie od folderu: `"home"`, `"last"` lub `"radar"`. Ma pierwszeństwo przed dwoma ustawieniami powyżej. `"last"` to Home przy wyłączonym `remember_tabs` lub gdy nic nie zapamiętano; `"radar"` to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Czytane raz przy starcie; wymaga restartu. |
| `general.group_worktree_tabs` | `true` | Trzyma karty powiązanych worktree repozytorium tuż za kartą tego repozytorium: gdy worktree się otwiera, gdy karta worktree zostanie przeciągnięta gdzie indziej (wraca obok swojej grupy; karta repozytorium jest głową grupy, a jej worktree idą za nią) i gdy ustawienie zostanie włączone. Wyłączone zostawia karty tam, gdzie je ułożysz. |
| `general.undo_depth` | `100` | O ile operacji wstecz może cofnąć Cofanie; `0` = bez limitu. |
| `general.conflict_detection` | `true` | Proaktywne wykrywanie konfliktów: plakietka w Log, gdy merge bieżącej gałęzi do gałęzi, z której powstała, spowodowałby konflikt, liczona w tle (merge w pamięci: nic w twoich plikach, indeksie ani refach się nie zmienia). Repozytorium może je wyłączyć dla siebie (`[preferences] conflict_detection`). Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md#zapobieganie-konfliktom). |
| `general.dev_mode` | brak (automatycznie) | Dla osób, które budują GitPoulpi z checkoutu; nigdy niepokazywane w Ustawieniach. Brak: włączone, gdy uruchomiony plik wykonywalny leży w `<checkout>/target/<profile>/` (`Cargo.toml` i `crates/gitpoulpi-tui` obok `target`). `true` / `false` wymusza wartość. Tryb deweloperski odblokowuje tylko `ui.launcher_build`. |
| `general.journal_outside` | `true` | Zapisuje to, co powłoka lub agent AI zrobi z gałęziami, tagami, stashami i HEAD, jako wpisy historii cofania, oznaczone jako zewnętrzne, więc `Ctrl+Z` może to cofnąć. Zmiany plików nigdy nie są przechwytywane. Wymaga `undo_enabled`. Zob. [Cofnij i ponów](undo-redo.md#zmiany-wykonane-poza-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: brak jakiejkolwiek księgowości cofania (najszybciej), historia cofania jest kasowana, a przyciski Cofnij/Ponów znikają. Zob. [Cofnij i ponów](undo-redo.md). |
| `general.active_profile` | `""` | Id tożsamości `[[profiles]]`, z którą tworzone są commity. Puste albo profil z pustą nazwą lub e-mailem: `user.name` / `user.email` gita. |
| `general.external_editor` | `""` | Polecenie dla pozycji pliku *Otwórz w edytorze*, np. `code -r {file}`. Puste: VS Code, jeśli jest zainstalowany, w przeciwnym razie domyślna aplikacja systemu. |
| `general.external_diff_tool` | `""` | Polecenie dla *Otwórz w zewnętrznym narzędziu diff*: `{local}` (wersja zatwierdzona / HEAD), `{remote}` (wersja z drzewa roboczego / indeksu). Puste: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Polecenie dla *Otwórz w zewnętrznym narzędziu merge* na karcie konfliktu (plik binarny, LFS lub zbyt duży): `{base}`, `{local}` (A, bieżący), `{remote}` (B, przychodzący) i `{merged}` (plik, do którego narzędzie zapisuje wynik; GitPoulpi dodaje go do poczekalni, gdy narzędzie zakończy się powodzeniem). Puste: `git mergetool --no-prompt`. |
| `general.file_explorer` | zależnie od OS | Polecenie dla *Pokaż w folderze*: `{path}` (plik), `{dir}` (jego folder). Domyślnie: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Trzy szablony poleceń są najpierw dzielone na argumenty (`"..."` grupuje słowo),
dopiero potem wstawiane są symbole zastępcze, więc ścieżki ze spacjami pozostają
jednym argumentem. Powłoka nie jest używana.

```toml
[[profiles]]                # tożsamości commitów (Ustawienia > Profile)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log i graf**

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (obraz tam, gdzie terminal potrafi go narysować, znaki gdzie indziej), `image`, `text`. **Restart.** |
| `ui.image_protocol` | `"auto"` | Jak obrazy (graf, awatary) trafiają do terminala: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protokół Kitty w kitty, Ghostty i herdr z `kitty_graphics = true`; iTerm2 w WezTerm, iTerm2, Rio, mintty; sixel (tylko awatary) w Windows Terminal, foot, Konsole, mlterm; none w tmux, zellij, screen i nieznanych terminalach. **Restart.** |
| `ui.theme` | `"system"` | Motyw kolorystyczny: wbudowany slug (`gitpoulpi theme list` wylicza wszystkie), slug pliku w `<config>/themes/` albo `system` (idzie za jasnym lub ciemnym tłem terminala, wykrytym przy starcie; dawna wartość `default` znaczy to samo). `config set` odrzuca nieznane nazwy. Zob. [Motywy](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Paleta ścieżek: `theme` (idzie za motywem) lub dowolna nazwa palety ścieżek: `gitpoulpi`, `light`, `material`, `serie` oraz palety innych motywów. |
| `ui.graph_lane_background` | `"off"` | `off` lub `row_tint` (delikatny pas koloru ścieżki za grafem oraz pasek na jego brzegu). **Aplikacja: Przełącz zabarwienie wierszy grafu** (`Ctrl+N Shift+T`) go przełącza; **Aplikacja: Zmień motyw kolorów grafu...** (`Ctrl+N Shift+Y`) wybiera `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (tylko z `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` i `right` rysują kolumnę; okno węższe niż 80 kolumn wraca do `bottom`. |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: gdzie leżą przyciski wzdłuż paska (lewo, środek, prawo wiersza; góra, środek, dół kolumny). |
| `ui.toolbar_pr_pinned` | `true` | Przycisk pull requesta jest przypięty na końcu paska narzędzi; `false` pozwala mu płynąć razem z innymi przyciskami. |
| `ui.statusbar_position` | `"bottom"` | `top` (nad kartami repozytoriów) albo `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (ostatni wiersz okna, pod paskiem stanu) lub `left` (kolumna przy lewej krawędzi; poniżej 80 kolumn zostaje u góry). |
| `ui.graph_show_stashes` | `true` | Wiersz na każdy stash w Logu. |
| `ui.ref_connector` | `true` | Cienka linia od etykiety ref do jej commita. |
| `ui.column_resize` | `"line"` | Jak chwyta się granicę kolumny w Logu, Radarze i kolumnie commitów w widokach diffa, pliku i blame. `line`: każda granica to odstęp jednej komórki z małym znacznikiem w nagłówku; gdy wskaźnik jest na granicy, w nagłówku lub gdziekolwiek w dół tabeli, rysowana jest cienka linia akcentu na całej wysokości, a przeciągnięcie z dowolnego jej miejsca zmienia szerokość. `header`: bez odstępu i bez znacznika; pierwszy znak komórki nagłówka za granicą zapala się, a przeciągnięcie stamtąd zmienia szerokość (tylko z nagłówka). Klawisze do zmiany szerokości i wiersze palety są takie same w obu trybach. |
| `ui.ref_tree` | `true` | Grupuje gałęzie i tagi po `/` w foldery w panelu bocznym Refs (`y` przełącza). |
| `ui.graph_ghost_ref` | `true` | Kolumna Ref w Log pokazuje wyblakłą pigułkę z nazwą gałęzi (w razie braku gałęzi zdalnej), do której należy najechany commit, gdy commit nie ma własnego refa, a kolumna ma miejsce; nigdy tagu. Usunięta gałąź jest wnioskowana z tematu merge commita, który ją scalił, i rysowana przekreślona. Najechanie na pigułkę pokazuje jej rodzaj. |
| `ui.reveal_on_ref_click` | `true` | Pojedyncze kliknięcie refu (gałęzi, tagu, gałęzi zdalnej, stasha) w panelu Refs przewija Log do jego commita i na chwilę podświetla wiersz; zaznaczenie commita się nie zmienia. |
| `ui.log_wip_counts_in_message` | `true` | Liczniki zmian WIP w opisie WIP, gdy kolumna Zmiany jest ukryta. |
| `ui.log_columns_layout` | `"fit"` | Jak Log układa swoje kolumny. `fit`: każda widoczna kolumna mieści się dokładnie w panelu i tylko `message` się rozciąga; gdy panel jest wąski, ustępują całe kolumny. `scroll`: kolumny zachowują swoje szerokości (`message` też), nic nie ustępuje, a tabela przewija się w poziomie, gdy jest szersza niż panel. |
| `ui.log_column_widths_scroll` | puste | Szerokości kolumn układu `scroll`, nazwa -> komórki (dozwolone `message`, inaczej niż w `ui.log_column_widths`, które zostaje szerokościami układu `fit` i nigdy nie zawiera `message`). Zapisywane, gdy przeciągasz, zmieniasz lub dopasowujesz kolumnę przy układzie `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | puste | Kolumny Logu (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), zapisywane po ukryciu, przeciągnięciu lub zmianie rozmiaru kolumny. Kolumny `message` nie można ukryć. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | puste | Części [kolumny commitów](interface-tour.md#kolumna-commitów) obok diffa, osobno od części Logu: widoczne części według nazwy (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; puste oznacza tory, opis, autora i datę), ich kolejność od lewej do prawej według tych samych nazw (puste oznacza kolejność wypisaną przed chwilą; pominięta nazwa idzie za nimi w tej kolejności) oraz przeciągnięte szerokości w komórkach na część (pominięta część jest automatyczna), zapisywane po wybraniu, przesunięciu lub zmianie rozmiaru części. Części `message` nie można ukryć. Plik sesji zapisany przed tymi kluczami jest raz wczytywany do nich. |
| `ui.sidebar_sections_hidden` | puste | Sekcje panelu bocznego, które ukryłeś (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), zapisywane z menu prawego przycisku panelu lub z palety. Co najmniej jedna istniejąca sekcja zawsze pozostaje widoczna. To wartość domyślna: każde repozytorium może ukryć własne sekcje (zapisane w jego pliku stanu), a ten klucz dotyczy repozytorium bez własnego wyboru; zapisuje go *Użyj we wszystkich repozytoriach*. |
| `ui.sidebar_sections_scope` | `"repository"` | Od czego zależy wybór sekcji panelu bocznego: `repository` (każde repozytorium ma własny wybór, a `ui.sidebar_sections_hidden` jest wartością domyślną) albo `global` (każde repozytorium pokazuje `ui.sidebar_sections_hidden`, a zmiana jest zapisywana tam). Przełączanie nie traci żadnego z zestawów: wybory poszczególnych repozytoriów zostają w ich plikach stanu, gdy `global` jest włączone. Ustawienia: *Sekcje panelu bocznego*. |
| `ui.language` | `"auto"` | Język interfejsu: `auto` (ustawienia regionalne systemu, w przeciwnym razie angielski), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Na żywo.** `GITPOULPI_LANG` go nadpisuje. Język, którego nie ma w tej kompilacji, pokazuje angielski. **Aplikacja: Zmień język...** (`Ctrl+N g`) też go ustawia, z podglądem na żywo. Wyjście CLI i MCP pozostaje po angielsku. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... zostają po angielsku w każdym języku. `true`: słownictwo gita wybranego języka (polskie „schowek”, „znacznik”, „zatwierdzenie”...). **Na żywo.** |
| `ui.git_terms_english` | `false` | Tylko ja, ko i zh-CN: pokazuje angielskie słowa git (stage, unstage, staged, unstaged...) zamiast transkrypcji. Bez efektu, gdy `ui.translate_git_terms` ma wartość `true`. **Na żywo.** |
| `ui.mouse_enabled` | `true` | `false` zostawia mysz (i zaznaczanie tekstu) terminalowi. **Restart.** |
| `ui.mouse_scroll_lines` | `3` | Wierszy Logu na krok kółka, od 1 do 20. **Restart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` prosi terminal o protokół klawiatury kitty (dzięki czemu docierają `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`); `off` nigdy nie prosi. **Restart.** |
| `ui.logo_color` | `""` | Wariant kolorystyczny logo w oknie About, slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); puste lub nieznane oznacza domyślny wariant kompilacji (`lavender`). Zapisywany po kliknięciu logo lub wybraniu koloru w About. |

**Awatary** (domyślnie wyłączone; ich włączenie wysyła do Gravatar hasz SHA-256
e-maili autorów i odpytuje podłączone konto GitHub/GitLab)

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (w kółku commita grafu obrazkowego), `author` (w kolumnie Autor), `off`. Dawne `ui.show_avatars` (`true`) jest nadal czytane jako `author`, dopóki brak `ui.avatars`. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Kolejność wyszukiwania: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Host GitLab jest odpytywany tylko przez podłączony profil. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (generowany lokalnie) lub `initials`. W terminalu bez protokołu obrazów zawsze jest używana plakietka z inicjałami. |

**Wyświetlanie diff i edytora**

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` lub `split` (`m` w diff). |
| `ui.diff_hunks_only` | `true` | `false` pokazuje cały plik ze zmianami na miejscu (`H`). |
| `ui.diff_show_whitespace` | `true` | Pokazuje zmiany tylko w białych znakach (`s`). |
| `ui.diff_word_wrap` | `false` | Zawija długie linie diff (`w`; Ustawienia > Diff i edytor). |
| `ui.minimap` | `false` | Minimapa obok paska przewijania diff i edytora (`M`). |
| `ui.mermaid_detail` | `refs` | Poziom szczegółów funkcji Kopiuj jako gitGraph Mermaid w Logu: `structure`, `refs`, `subjects`, `full` (zapamiętywany z okna; domyślny dla `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Kierunek tego grafu: `lr` (od lewej do prawej) lub `tb` (od góry do dołu). |
| `ui.mermaid_fence` | `false` | Opakowuje skopiowany skrypt Mermaid w blok kodu `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Opisy commitów są przycinane do tylu znaków w identyfikatorach Mermaid (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` lub `16`: ilu kolorów używać przy rysowaniu. `auto` to truecolor, z wyjątkiem Apple Terminal (256), konsoli Linuksa (16) i terminali `*-256color` bez `COLORTERM` (256). Na żywo. Zob. [Motywy](themes.md). |
| `ui.input_color` | `"theme"` | Tło pól tekstowych: `theme` (własne motywu), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | Kolor `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | Kompozytor commitów AI otwiera się z plikami każdej propozycji `expanded` (rozwiniętymi) lub `collapsed` (zwiniętymi). |
| `ui.blame_side` | `"right"` | Po której stronie widoku Blame jest kolumna adnotacji (awatar, autor, wiek, podsumowanie): `right` (domyślnie) lub `left`. |
| `ui.blame_width` | `46` | Szerokość kolumny adnotacji Blame w komórkach (24 do 90). Przeciągnij jej krawędź lub naciśnij w widoku `<` / `>`; nowa szerokość zapisuje się tutaj. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request w GitHub, merge request w GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Ukrywa chip paska narzędzi z najnowszym pull requestem lub merge requestem bieżącej gałęzi, gdy tylko zostanie zamknięty lub scalony; zamiast niego pojawia się przyciemniony chip „+ PR” (rozpocznij nowy), tak jakby gałąź go nie miała. `false`: chip zawsze pokazuje najnowsze żądanie, niezależnie od jego stanu. |

Przełączniki diff zmieniane wewnątrz diff są od razu tu zapisywane i używane
przez każdy diff otwarty potem.

**Daty**: `ui.date_format` (`"YYYY-MM-DD"`, kolumna daty w Blame / Historii pliku),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, kolumna Data w Logu)
i `ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, szczegóły commita,
podpowiedź Data i *Kopiuj > Data*). Znaczniki: `YYYY` `YY` `MMMM` `MMM` `MM` `M`
`DD` `D` `dddd` `ddd` (dzień tygodnia, pełny / skrócony) `HH` `mm` `ss`; wszystko
inne jest kopiowane bez zmian. Nazwy miesięcy i dni tygodnia idą za `ui.language`,
a miesiąc zapisany po numerze dnia ma formę dopełniacza w językach, które ją mają
(rosyjski, ukraiński, polski, czeski). Ciąg `D MMMM YYYY` jest zapisywany w
japońskim, chińskim i koreańskim od roku (`2024年3月5日`). `ui.date_locale`
(`"system"`, domyślnie, lub `"english"`) zostawia nazwy po angielsku niezależnie
od języka.

**Agenci, terminale i folder karty**

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Czy okno w ogóle przyjmuje od agentów `ui show` / `ui propose` (Ustawienia > Agenci i MCP > *Pozwól agentom AI sterować widokiem*). Wyłączone: żaden agent do niego nie sięga. Zob. [MCP i agenci](mcp-and-agents.md#pokazywanie-rzeczy-użytkownikowi). |
| `ui.launch` | `"off"` | Czy agent może otworzyć okno GitPoulpi, gdy żadne nie jest otwarte: `off`, ustawienie wstępne (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) lub `custom`. |
| `ui.launch_command` | `""` | Szablon `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Dzielony na argumenty, bez powłoki. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (pasek kart oferuje *Zakończ i cd tutaj*) lub `auto` (każde zakończenie zmienia folder powłoki, a nowa karta uruchamia domyślny wpis *Otwórz w*). Zob. [Podążanie za folderem karty](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Przestarzałe, zob. niżej i [Otwórz w](open-in.md). Jak dawny *Terminal tutaj* otwierał terminal: `auto` (ustawienie wstępne `ui.launch`, w przeciwnym razie `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), ustawienie wstępne lub `custom`. |
| `ui.terminal_launch_command` | `""` | Szablon `custom`; `{repo}` to folder. |
| `ui.open_in` | *(brak)* | Lista [Otwórz w](open-in.md), tablica tabel `[[ui.open_in]]` z `label` i `command`. Brak oznacza „jeszcze nieskonfigurowane”: lista jest wyliczana ze starych kluczy `ui.terminal_launch*` (wartość inna niż `auto` staje się pierwszym wpisem), eksploratora plików systemu, multiplekserów i terminali wykrytych ze środowiska (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) oraz edytorów znalezionych w `PATH`. Pierwszy zapis z Ustawień zapisuje listę. `open_in = []` to lista pusta. Nie da się ustawić przez `config set` (edytuj plik). |
| `ui.open_in_default` | `""` | Etykieta wpisu uruchamianego przez „Otwórz w domyślnym”; puste lub nieznane oznacza pierwszy wpis. |
| `ui.launch_terminal` | `"auto"` | Terminal, w którym skróty na pulpicie (Ustawienia > Ogólne > Uruchamianie z systemu) i ponowne uruchomienie w Windows otwierają GitPoulpi. `auto` wybiera spośród zainstalowanych (Windows: Windows Terminal, w przeciwnym razie WezTerm, Alacritty, zwykła konsola; Linux: `xdg-terminal-exec`, w przeciwnym razie pierwszy znaleziony terminal; macOS: iTerm2, Ghostty, WezTerm lub kitty, jeśli są zainstalowane, w przeciwnym razie Terminal), `default` to terminal systemu, albo id terminala, np. `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Tylko w trybie deweloperskim (zob. `general.dev_mode`; tylko wtedy pokazywane w Ustawieniach > Uruchamianie z systemu, w przeciwnym razie ignorowane). `"current"`: skróty i alias polecenia uruchamiają kompilację, która je utworzyła. `"newest"`: uruchamiają najnowszą z `target/dist` i `target/release` checkoutu, wybieraną przy każdym kliknięciu, a uruchomienie którejkolwiek z nich nigdy ich nie przepisuje. |
| `ui.command_alias` | `""` | Nazwa polecenia, które uruchamia GitPoulpi z terminala, na przykład `gp` (Ustawienia > Ogólne > Uruchamianie z systemu, `gitpoulpi alias create`). Puste: brak. Tylko litery, cyfry, `-`, `_` i `.`. To tylko nazwa, której używa przycisk **Utwórz: alias polecenia**; nic nie istnieje, dopóki go nie utworzysz. |
| `ui.command_alias_add` | `""` | Nazwa drugiego polecenia, na przykład `gpa`, które uruchamia `gitpoulpi --add`: otwiera bieżący folder jako nową kartę w już uruchomionym GitPoulpi (Ustawienia > Ogólne > Uruchamianie z systemu, `gitpoulpi alias create gpa --add`). Puste: brak. Te same zasady nazw; tworzone razem z `ui.command_alias` przyciskiem **Utwórz**. |
| `ui.relaunch_in_terminal` | `true` | Tylko Windows: gdy plik exe zostanie uruchomiony dwukrotnym kliknięciem we własnej konsoli, GitPoulpi otwiera się ponownie w `ui.launch_terminal` i kończy działanie. Wyłączone: zostaje w tej konsoli. |
| `ui.reuse_window` | `true` | Kliknięcie skrótu GitPoulpi (`gitpoulpi launch`) pokazuje okno już otwartego GitPoulpi zamiast uruchamiać kolejne. Wyłączone: każde kliknięcie uruchamia nowe okno. |

**Polecenia `ui.open_in`** są najpierw dzielone na argumenty (białe znaki, `'...'`, `"..."`), dopiero potem zmienne są podstawiane wewnątrz każdego argumentu, więc folder ze spacjami pozostaje jednym argumentem; powłoka nie jest używana, chyba że polecenie ją uruchamia (`sh -c '... "$GITPOULPI_CWD"'`). Zmienne: `${cwd}` (folder karty), `${repo}` (katalog główny repozytorium), `${name}` (nazwa folderu), `${branch}` (puste przy odłączonym HEAD), `${web_url}` (strona WWW zdalnego repozytorium `origin`, gdy to projekt GitHub lub GitLab, w przeciwnym razie puste; wpis, który jej używa, odmawia uruchomienia, gdy jest pusta), `${env:NAME}` (puste, gdy nieustawiona). Dawne `{repo}` nadal znaczy `${cwd}`. Każda inna `${...}` jest błędem wskazującym jej nazwę. Przykład:

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

Przestarzałe: `ui.terminal_launch` i `ui.terminal_launch_command` są nadal czytane (zasilają listę, dopóki brak `ui.open_in`: `custom` staje się „Terminal (custom)” z `{repo}` zamienionym na `${cwd}`, id ustawienia wstępnego staje się tym ustawieniem), ale zastępuje je `ui.open_in`.

### Obrazy

| Klucz | Domyślnie | Znaczenie |
|---|---|---|
| `ui.image_view` | `true` | Pokazuje plik binarny będący obrazem (PNG, JPEG, GIF, WebP, BMP, ...) jako obraz w widoku Diff i widoku pliku. Wyłączone: zwykła informacja o pliku binarnym. |
| `ui.image_view_max_mib` | `32` | Obrazy większe niż to (MiB) nie są dekodowane; zamiast tego pokazuje się karta informacyjna. |
| `ui.image_view_max_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują limit: `mib`, `gib` (1024 MiB) albo `auto`. Tylko wyświetlanie: limit to zawsze `image_view_max_mib` MiB, od 1 do 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | W terminalu bez protokołu obrazów: `halfblock` (kolorowe znaki półbloków) lub `card` (tylko karta informacyjna). |
| `ui.image_view_pixelated_from` | `"2x"` | Powiększenie, od którego powiększanie pokazuje kwadratowe piksele zamiast wygładzania: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Panele przed i po dzielą jedno powiększenie i jedną pozycję przewijania. |
| `ui.image_editors` | *(brak)* | Edytory obrazów „Edytuj w”, tablica tabel `[[ui.image_editors]]` z `label` i `command`. Brak oznacza „jeszcze nieskonfigurowane”: lista to edytory wykryte na tym komputerze (programy w `PATH`, instalacje Flatpak, znane foldery instalacji w Windows, `/Applications` w macOS), potem domyślna aplikacja systemu. Pierwszy zapis z Ustawień zapisuje listę. `image_editors = []` to lista pusta. Nie da się ustawić przez `config set` (edytuj plik). |
| `ui.image_editor_default` | `""` | Etykieta edytora uruchamianego przez „Edytuj w domyślnym”; puste lub nieznane oznacza pierwszy wpis. |

**Polecenia `ui.image_editors`** są najpierw dzielone na argumenty (białe znaki, `'...'`, `"..."`), dopiero potem zmienne są podstawiane wewnątrz każdego argumentu, więc ścieżka ze spacjami pozostaje jednym argumentem; powłoka nie jest używana. Zmienne: `${file}` (bezwzględna ścieżka obrazu), `${dir}` (jego folder), `${name}` (nazwa pliku), `${repo}` (katalog główny repozytorium, puste dla kopii tymczasowej), `${env:NAME}` (puste, gdy nieustawiona). Każda inna `${...}` jest błędem wskazującym jej nazwę. Obraz, który nie jest plikiem z drzewa roboczego (wersja z commita, indeks, usunięty plik), jest otwierany jako **kopia tymczasowa tylko do odczytu**, więc edytor nie może dotknąć repozytorium. Edytorów bez wiersza poleceń (Affinity Photo ze Sklepu Microsoft, niektóre instalacje Photoshopa) nie ma wśród ustawień wstępnych: napisz wpis własny. Przykład:

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

**Jeszcze nieużywane** (przyjmowane, zapisywane, ale nic ich nie czyta):
`ui.graph_colorization` (usunięte z Ustawień;
nadal przyjmowane przy wczytaniu), `ui.date_word_format` (usunięty z Ustawień;
nadal przyjmowany przy wczytaniu). Dawne `ui.diff_view_mode` jest czytane jednorazowo jako
`diff_layout` / `diff_hunks_only`.

## `[editor]`

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `editor.tab_size` | `4` | Kolumn na tabulator, od 1 do 8 (edytor i diffy). |
| `editor.eol` | `"auto"` | Koniec linii zapisywany przez wbudowany edytor: `auto` (zachowaj z pliku), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Kolory języka w edytorze. |
| `editor.line_numbers` | `true` | Numery linii w edytorze i diffach. |
| `editor.follow_selection` | `true` | Gdy edytor jest otwarty, zaznaczenie innego pliku w prawym panelu otwiera go także w edytorze, po zapytaniu o zapisanie niezapisanych zmian. Plik, którego nie można edytować, otwiera się w zwykłym widoku; plik z commita opuszcza edytor na rzecz swojego diffa. |

## `[merge]`

Edytor konfliktów merge ([Rozwiązywanie konfliktów merge](merge-conflicts.md)). Pokazuje go Ustawienia > Diff i edytor.

| Klucz | Domyślnie | Znaczenie |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | Szerokość w kolumnach obszaru edytora, od której A i B są pokazywane obok siebie (długie linie przewijają się w bok: Shift+kółko, Shift+strzałki). Węższy pokazuje jedną stronę naraz za kartami. `40` to najniższa wartość, `0` oznacza zawsze obok siebie (informacja poniżej 40 kolumn pozostaje). Edytor ukrywa panele Refs i Status, gdy jest otwarty, więc jego obszar to zwykle całe okno. |

## `[git]`, `[credentials]`, `[ssh]`

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `git.pull_mode` | `"merge"` | Co robi Pull (przycisk, `Ctrl+G p`, `Git: Pull`): `merge` (fast-forward, gdy możliwy), `ff_only`, `rebase`. Zawsze jawnie przekazywane gitowi. Dawne `fetch_all` jest czytane jako `merge`. |
| `git.push_behavior` | `"direct"` | `direct` wypycha do domyślnego zdalnego repozytorium; `ask` najpierw pokazuje menu zdalnych repozytoriów. |
| `git.default_encoding` | `"utf-8"` | Kodowanie tekstu, który **nie** jest poprawnym UTF-8 (zawartość plików, blame, diffy, opisy commitów bez nagłówka `encoding`): dowolna etykieta WHATWG (`windows-1252`, `shift_jis`, ...). Tylko do wyświetlania; staging nigdy nie koduje ponownie. Repozytorium może to nadpisać. |
| `git.lfs_enabled` | `true` | Pokazuje i obsługuje Git LFS. Repozytorium może to nadpisać. Zob. [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Przed pull lub push repozytorium używającego LFS instaluje jego filtry LFS / hook pre-push (`git lfs install --local`, tylko to repozytorium), gdy ich brakuje. Zob. [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Nieużywane*: `core.autocrlf` / `core.eol` repozytorium to własne ustawienia gita, edytowane w Ustawieniach > Git i kodowanie (lub przez `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Uwierzytelnianie HTTPS dla fetch, pull, push, clone: `credential_helper` (gita), `prompt` (ignoruje menedżery poświadczeń; git pyta w terminalu, więc w demonie działającym w tle kończy się niepowodzeniem), `none` (tylko publiczne zdalne repozytoria). Dawne wartości `encrypted_file` są czytane jako `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Typ tworzony przez *Generuj parę kluczy*: `ed25519`, `rsa` (4096 bitów), `ecdsa` (NIST P-256); plik to `gitpoulpi_<type>`. Samo SSH zawsze używa twojego ssh-agent i konfiguracji ssh. |
| `ssh.private_key_path`, `ssh.public_key_path` | nieustawione | Zapisywane przez *Generuj parę kluczy*. Gdy ustawiono `ssh.private_key_path`, fetch, pull, push i clone uruchamiają git z `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` robi to samo (ssh wybiera pasujący klucz agenta), gdy ścieżka prywatna jest pusta. Wygrywa `GIT_SSH_COMMAND`/`GIT_SSH` w środowisku lub `core.sshCommand` w konfiguracji git (przypięcie dla repozytorium, `gitpoulpi ssh-command set --key`). Czytane przed każdym transferem, więc zmiany działają od następnego. |

## `[[integrations]]` (hosting i trackery zgłoszeń)

Zapisywane przez Ustawienia > Integracje lub `gitpoulpi hosting integrations add`.
Każdy rodzaj ma klienta (tylko osobiste tokeny, bez OAuth).

```toml
[[integrations]]
id = "corp"                         # to, co przyjmuje --id; token jest zapisany pod integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # z niego wyprowadzany jest korzeń API
token_ref = "integration:corp"      # odwołanie, nigdy sam token
name = "Corp GitLab"                # opcjonalna nazwa wyświetlana
account = "me@example.com"          # opcjonalne, zob. niżej
enabled = true                      # false: pomijane przy dopasowaniu hosta
```

Serwer hostowany samodzielnie może działać po zwykłym http: `url = "http://localhost:3000"` zostaje
zapisany tak, jak go wpisano (schemat i port trafiają do każdego żądania, a zdalne repozytorium na tym hoście
pasuje do profilu). Token wysłany zwykłym http jest przesyłany bez szyfrowania, więc używaj
http tylko w sieci lokalnej lub zaufanej.

`kind` przyjmuje też `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` i `gerrit`
(Gitea i Forgejo dzielą jednego klienta; profil Forgejo domyślnie ma `https://codeberg.org`). Jira,
Trello, Linear, YouTrack i Redmine to wyłącznie trackery zgłoszeń; Gerrit służy tylko do przeglądu (jego zmiany to
pull requesty, nie ma zgłoszeń). Bitbucket Cloud nie ma trackera
zgłoszeń.

`account` to niesekretna tożsamość obok tokenu, pusta i pomijana dla większości
rodzajów: e-mail konta Atlassian dla `jira_cloud` i `bitbucket` (puste konto
Bitbucket oznacza token dostępu workspace lub repozytorium, wysyłany jako
Bearer), klucz API dla `trello`, nazwa użytkownika dla `gerrit` (token to jego hasło HTTP,
wysyłane jako HTTP basic). Sam token pozostaje jedynym sekretem. Dla
`azure_devops` `url` to `https://dev.azure.com/<organization>`; adres
`https://<organization>.visualstudio.com` jest czytany jako ta sama organizacja.
Dla `azure_devops_server` to korzeń kolekcji, np.
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (podłączone konto) i `ssh` (wybór klucza profilu) są zapisywane za
ciebie. Token pochodzi z `GITPOULPI_TOKEN_<ID>` (id wielkimi literami, znaki
niealfanumeryczne jako `_`), w przeciwnym razie z zapisanego tokenu, w
przeciwnym razie z twojego menedżera poświadczeń dla tego hosta, gdy używa go
tylko jeden profil (tylko profile GitHub i GitLab: inne rodzaje nigdy nie czytają
menedżera poświadczeń). Szczegóły: [Integracje z hostingiem i trackerami
zgłoszeń](hosting-integration.md). Dawna tabela `[[hosting]]` jest *nieużywana*.

Dla `local_markdown` (folder notatek Markdown jako źródło zgłoszeń) `url` to
folder, względny względem każdego repozytorium lub bezwzględny, a tokenu nie ma. Jego
opcjonalna tabela `[integrations.local]` ustawia `statuses` (kolejność kolumn tablicy,
domyślnie `todo`, `doing`, `done`), `done_status` (domyślnie `done`),
`reference_prefix` (`LOC` pokazuje `LOC-12`) i `me` (kogo oznacza „przypisane do mnie”).
Zob. [Zgłoszenia lokalne i Obsidian](local-issues.md).

### `[integrations.radar]` (listy obejmujące całe konto)

Opcjonalna tabela profilu hosta kodu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; nie Bitbucket Cloud). Brak
oznacza, że każdy klucz jest wyłączony. Zapisywana przez `gitpoulpi hosting integrations radar`; czytana przez
`gitpoulpi hosting radar` i Radar, a edytowana w Ustawieniach > Integracje (blok Radar połączonego profilu).

```toml
[integrations.radar]
review_requests = true    # żądania czekające na mój przegląd, każde repozytorium
open_requests = false     # każde otwarte żądanie moich przestrzeni nazw
include_drafts = false    # zachowaj szkice żądań na tych listach
assigned_issues = true    # otwarte zgłoszenia przypisane do mnie, każde repozytorium
namespaces = ["acme"]     # organizacje / grupy do przeszukania pod kątem otwartych żądań
review_mode = "ask"       # ask | clone | api
```

| Klucz | Domyślnie | Znaczenie |
|---|---|---|
| `review_requests` | `false` | Otwarte żądania, które proszą o twój przegląd, niezależnie od tego, czy repozytorium jest sklonowane |
| `open_requests` | `false` | Każde otwarte żądanie twojej własnej przestrzeni nazw i każdej organizacji lub grupy, do której należysz (albo z `namespaces`), nigdy „wszystko, co widzi token” |
| `include_drafts` | `false` | Szkice żądań są pomijane na tych listach, chyba że to jest włączone |
| `assigned_issues` | `false` | Otwarte zgłoszenia przypisane do ciebie, każde repozytorium |
| `namespaces` | `[]` | Organizacje / grupy do przeszukania dla `open_requests`; puste oznacza twoją własną przestrzeń nazw plus każdą, do której należysz (GitHub: przeszukiwanych jest najwyżej 12, GitLab: 10 grup i 10 własnych projektów, Gitea, Forgejo i Gerrit: 10; Azure DevOps przyjmuje nazwy projektów, Bitbucket Data Center klucze projektów, Gerrit prefiksy ścieżek projektów) |
| `review_mode` | `"ask"` | Co robi przegląd żądania z niesklonowanego repozytorium: `ask`, `clone` (sklonuj, potem przejrzyj) albo `api` (przejrzyj przez API hosta) |

Odpowiedzi są przez pięć minut buforowane dla każdego profilu w `radar-account.json` obok
`config.toml`; plik jest pamięcią podręczną i można go bezpiecznie usunąć.

### Magazyn zaufania (nie w `config.toml`)

To, czemu zaufano każdej integracji i każdemu profilowi AI z kluczem API (jego adres,
id konta, przypięty klucz certyfikatu, czy zezwolono na zwykłe http), leży w `trust.toml`
w prywatnym folderze danych dostępnym tylko dla użytkownika (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` w Windows, `~/.local/share/gitpoulpi/private/`
w Linuksie). Celowo nie jest częścią `config.toml`: kto może edytować
konfigurację, nie może poszerzyć zaufania przez jej edycję. Zmieniaj go przez
`gitpoulpi hosting integrations trust` i `gitpoulpi ai trust` (zob.
[kontrola bezpieczeństwa](hosting-integration.md#kontrola-bezpieczeństwa)); plik
jest przepisywany przez GitPoulpi, nie ręcznie.

## `[mcp]` (co mogą robić agenci AI)

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `mcp.allow_mutations` | `false` | Wyłączone: narzędzia MCP zmieniające cokolwiek są odrzucane; odczyty, `ui_*` i propozycje nadal działają. `true` pozwala agentom zmieniać repozytorium. |
| `mcp.mutation_allowlist` | `[]` | Niepusta: tylko te nazwy narzędzi mogą coś zmieniać (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: odrzuca narzędzia `ui_*`, które poruszają twoim oknem (nie `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (okno pokazuje znacznik do potwierdzenia), `auto` (stosowane, chyba że jesteś zajęty), `off`. `--ui-control` w poleceniu serwera nadpisuje to dla danego agenta. Dotyczy też `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Zezwala na `ui_propose_*` / `gitpoulpi ui propose`: wstępnie wypełnione okna, które potwierdzasz sam. |
| `mcp.allow_history_purge` | `false` | Tylko w `config.toml` (bez wiersza w Ustawieniach). `true` udostępnia agentom narzędzie `rewrite_purge` (nadal wymaga `allow_mutations`). Wyłączone, agent w ogóle nie widzi tego narzędzia: purge usuwa na stałe starą historię przepisania, jego cofnięcie i jego reflogi. CLI i TUI nie podlegają tej blokadzie. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` i `gitpoulpi-mcp` odmawiają startu, z komunikatem wskazującym ten klucz. Ustawienia > Agenci i MCP > *Serwer MCP*. Czytane, gdy agent uruchamia serwer. |
| `mcp.bind` | `"127.0.0.1:8808"` | Adres, na którym nasłuchuje `--http`, gdy żadnego nie podasz (`--http ADDR` / `--bind` wygrywają). Tylko pętla zwrotna, chyba że `--allow-remote`. Puste lub dawne `"stdio"` oznaczają wartość domyślną. Ustawienia > Agenci i MCP > *Port serwera MCP* edytuje port i zachowuje host. |
| `mcp.http_auth` | `true` | `--http` odpowiada tylko na żądanie z `Authorization: Bearer <token>` (token z `gitpoulpi mcp token`). `false` usuwa sprawdzanie; serwer ostrzega wtedy na stderr, a `--allow-remote` jest odrzucane. Ustawienia > Agenci i MCP > *Uwierzytelnianie HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (certyfikat generowany raz w prywatnym folderze danych, jego odcisk SHA-256 jest wypisywany przy starcie) lub `files` (`tls_cert` i `tls_key`). Przy włączonym TLS serwer sam kończy TLS i nie istnieje żaden zwykły nasłuch. Ustawienia > Agenci i MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Łańcuch certyfikatów PEM używany z `tls = "files"`. |
| `mcp.tls_key` | `""` | Klucz prywatny PEM używany z `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Oświadczasz, że szyfrowanie zapewnia tunel lub zwrotne proxy przed punktem końcowym. Z wnętrza serwera nie da się tego sprawdzić, więc `--allow-remote` bez TLS wymaga tego ustawienia. Odcinek między proxy a serwerem jest w tej sieci zwykłym tekstem. Ustawienia > Agenci i MCP > *Za tunelem lub zwrotnym proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Tylko w `config.toml`. Wyłączone: narzędzia MCP do dodawania do poczekalni nie oferują `allow_sensitive`, a dodanie, które objęłoby oznaczony plik (zob. `[privacy]`), jest odrzucane z komunikatem mówiącym agentowi, by zapytał ciebie. Włączone: agent może przekazać `allow_sensitive`. CLI i TUI nie są przez to bramkowane: pytają ciebie. |

`allow_mutations`, `mutation_allowlist` i `allow_ui` bramkują tylko serwer MCP;
CLI to twoja własna powłoka i nie jest przez nie bramkowane. Zob. [MCP
i agenci](mcp-and-agents.md#uprawnienia).

## `[ai]` (wbudowane AI, domyślnie wyłączone)

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `ai.enabled` | `false` | Przełącznik. Wyłączone: nigdzie nie widać niczego z AI i nic nie jest wysyłane. |
| `ai.default_profile` | `"default"` | Id profilu, którego używają akcje niewskazujące żadnego. |
| `ai.global_instructions` | `""` | Tekst dodawany do promptu każdej akcji. Może używać `${repo}`, `${uilang}`, ... (zob. [AI > Zmienne promptu](ai.md#zmienne-promptu)); to samo dotyczy instrukcji akcji i repozytorium. |
| `ai.actions.<action>.profile` | `""` | Profil dla jednej akcji; puste = domyślny. Akcje: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Obszar roboczy (`workspaces.toml`, `[workspace.ai_instructions]` i `[workspace.ai_overrides.<action>]`) i repozytorium mogą te instrukcje uzupełnić lub zastąpić: zob. [AI](ai.md#instrukcje). |
| `ai.actions.<action>.instructions` | `""` | Tekst dodawany tylko dla tej akcji. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` lub `no`: czy lokalny agent szkicu przeglądu kodu pracuje wewnątrz repozytorium w tym przebiegu, nadpisując `workdir` profilu AI. Repozytorium może mieć własną wartość (`gitpoulpi ai review-context --repo`). Zob. [AI > Kontekst przeglądu](ai.md#kontekst-przeglądu). |
| `ai.review.include_claude_md` | `true` | Przekaż szkicowi przeglądu kodu plik `CLAUDE.md` repozytorium (katalog główny, odczytany w gałęzi, do której zmierza żądanie; obcinany przy 8 KB). |
| `ai.review.include_agents_md` | `true` | To samo dla `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Jeszcze jeden plik dla szkicu przeglądu kodu: ścieżka względem katalogu głównego repozytorium, ścieżka bezwzględna albo link do pliku na hoście kodu (czytany przez pasujący profil integracji, inaczej anonimowo). Wszystkie pliki razem są obcinane przy 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Zastępuje każdy sekret rozpoznany przez skaner w żądaniu (token, blok klucza prywatnego, hasło w URL, wartość `password = ...` wyglądającą na losową) przez `[REDACTED:<rule>]`, zanim żądanie wyjdzie. Ustawienia > AI pokazują czerwone ostrzeżenie, gdy jest wyłączone. |
| `ai.privacy.exclude_sensitive_files` | `true` | Pomija w wysyłanym diff każdego pliku pasującego do `privacy.sensitive_patterns`. Lokalny agent pracujący w folderze repozytorium nadal może sam czytać pliki: zob. [AI > Prywatność](ai.md#prywatność). |
| `ai.sent_preview.<action>` | `true` | Jeden klucz na akcję (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): pokazuje „Co zostanie wysłane” przed wysłaniem żądania. „Nie pokazuj ponownie dla ...” w oknie zapisuje `false`; Ustawienia > AI > *Pokazuj, co zostanie wysłane, przed...* włączają je z powrotem. |

Własna tabela `[preferences.ai_review]` repozytorium (w jego `repo-state.toml` albo wpis repozytorium, które nie jest
sklonowane, w `remote-reviews.toml`) przyjmuje te same klucze, wszystkie opcjonalne, plus `no_extra_file = true`: brak
dodatkowego pliku dla tego repozytorium, cokolwiek mówi `ai.review.extra_file`. Nieustawiony klucz, nieustawiony
`extra_file` i brakujące `no_extra_file` dziedziczą wartość globalną; ustawienie `extra_file` na plik czyści
przełącznik i odwrotnie.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # puste = domyślny model rodzaju
base_url = ""           # puste = domyślny adres rodzaju (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # gdzie jest zapisany klucz API; nigdy sam klucz
command = ""            # tylko local_agent: dokładny program ("claude", alias, skrypt opakowujący, ścieżka)
harness = ""            # tylko local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; puste = wnioskowane z `command`
args = []               # tylko local_agent, np. ["-p", "--model", "haiku"]
run_mode = "headless"   # tylko local_agent: headless (domyślnie) | terminal (interaktywny ekran agenta w terminalu wirtualnym)
terminal_args = []      # tylko local_agent: argumenty trybu terminal (args zachowuje zestaw headless)
workdir = "default"     # tylko local_agent: default | dedicated | repository (default = folder dedykowany w trybie terminal, folder GitPoulpi w trybie headless)
timeout_secs = 0        # tylko local_agent: sekundy do zatrzymania agenta; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # tylko local_agent, opcjonalnie: zmienne ustawiane tylko dla tego procesu
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = jedna z własnych zmiennych GitPoulpi
```

`harness` wybiera podpowiedzi oferowane przez Ustawienia > AI (argumenty
headless, opcje, flagę i listę modeli, zmienne środowiskowe) niezależnie od
`command`; oba klucze są pomijane w pliku, gdy są puste. Wartości zmiennych
środowiskowych nigdy nie trafiają do logów i są ukrywane w Ustawieniach, gdy
wyglądają na sekrety.

Ustawienia > AI oferuje ustawienia wstępne dla `harness`/`command`/`args`/`env`
(popularne agenty CLI, z ich flagami headless, flagą modelu i przydatnymi
zmiennymi środowiskowymi) oraz dla `kind`/`base_url`/`model` (hostowani i lokalni
dostawcy HTTP); zob. [AI > Konfiguracja](ai.md#konfiguracja-ai).

Profil o nazwie *Default* jest tworzony, gdy żadnego nie ma. Stare pliki
`[[ai.presets]]` / `[ai.defaults]` są migrowane przy wczytaniu i zachowywane w
`[[ai.legacy_presets]]`. Reszta: [Wsparcie AI](ai.md).

## `[repos]`, `[session]`

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `repos.clone_root` | `""` | Folder, od którego zaczyna się okno Clone; puste = `%USERPROFILE%\source\repos` lub `~/src`. |
| `repos.clone_start_source` | `"last"` | Źródło, od którego otwiera się okno Clone: `last` (domyślnie, albo puste) = to, które pokazywało przy ostatnim zamknięciu, `url` = zawsze Klonuj z URL, albo id profilu integracji (`id` we wpisie `[[integrations]]`). Profil, którego nie ma, jest wyłączony lub niepołączony, wraca do Klonuj z URL. Samo ostatnie źródło to stan, trzymany w `clone-source.toml` w folderze konfiguracji, nie tutaj. |
| `repos.browse_root` | `""` | Folder, w którym otwiera się Przeglądaj; puste = ostatnio wybrany folder, potem katalog domowy. |
| `repos.show_hidden` | `false` | Selektor folderów pokazuje foldery ukryte (`Alt+H` przełącza i zapisuje). |
| `repos.show_siblings` | `false` | Lewe drzewo selektora folderów wypisuje też pozostałe foldery każdego otwartego poziomu, jako prawdziwą hierarchię (`Alt+S` przełącza i zapisuje). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Gdzie GitPoulpi przechowuje to, co pamięta o jednym repozytorium (filtry refów, stan zwinięć, osobisty wariant przepływu pracy, pamięć podręczna hostingu): `in_git_dir` = `<git dir>/gitpoulpi` (wspólne dla wszystkich worktree, nigdy nie widać w `git status`), `in_worktree` = `<repo>/.gitpoulpi` (jedyne miejsce, w którym zespół może zatwierdzić `workflow.toml`), `user_data` = `<folder danych systemu>/repos/<name>-<id>` (`%APPDATA%\GitPoulpi\data` w Windows, `$XDG_DATA_HOME/gitpoulpi` w Linuksie, `~/Library/Application Support/GitPoulpi` w macOS). Stara wartość `".gitpoulpi"` nadal oznacza `in_worktree`. Zmiana niczego nie przenosi z powrotem: dane znalezione w starym `<repo>/.gitpoulpi` są przenoszone na wybrane miejsce przy pierwszej potrzebie. |

## `[log]` (dziennik aplikacji)

To, co pokazuje karta Aplikacja Dziennika aktywności i co cytuje „Kopiuj diagnostykę”. Szczegóły: [Gdy coś się nie powiedzie](troubleshooting.md#gdy-coś-się-nie-powiedzie).

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `log.level` | `"info"` | Najbardziej szczegółowy wiersz, który jest zapisywany: `error`, `warn`, `info`, `debug` lub `trace`. Pole poziomu w Dzienniku aktywności go zmienia (i zapisuje tutaj). |
| `log.retention_days` | `7` | Liczba dni, przez które przechowywane są pliki dziennika, wliczając dzisiejszy: od 1 do 3650, czyli dziesięć lat (większa lub mniejsza wartość jest przycinana). Zawsze liczba dni, niezależnie od jednostki, w której pokazują ją Ustawienia. Starsze pliki są usuwane przy uruchomieniu GitPoulpi. Plik z jednego dnia ma też limit `log.max_file_mb`, więc najgorszy przypadek to około `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują `log.retention_days`: `days`, `weeks`, `months`, `years` albo `auto`. Tylko wyświetlanie, nigdy nie zmienia czasu przechowywania plików. Tydzień to 7 dni, miesiąc to 30 dni (nie miesiąc kalendarzowy, więc zawsze ta sama liczba dni) i rok to 365 dni. `auto` oraz konfiguracja zapisana przed pojawieniem się tego klucza wybierają największą jednostkę, która dzieli dni bez reszty (14 dni czyta się „2 tygodnie”, 45 dni „45 dni”). Jednostka, która nie dzieli dni (`retention_days = 10` z `weeks`), jest ignorowana i pokazywane są dni. Ustawienia zapisują oba klucze; starszy GitPoulpi ignoruje jednostkę i nadal czyta dni. |
| `log.max_file_mb` | `5` | Plik z jednego dnia jest przy tym rozmiarze rotowany do jednej kopii `.old`, więc dzień nigdy nie zajmuje więcej niż dwukrotność. |
| `log.max_file_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują rozmiar: `mib`, `gib` (1024 MiB) albo `auto`. Tylko wyświetlanie: rozmiar to zawsze `max_file_mb` MiB, od 1 do 1024. |
| `log.timings` | `true` | Karta Operacje git Dziennika aktywności pokazuje, jak długo trwała każda operacja (`Push - 123 ms`: otwarcie repozytorium, odświeżenie, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). Czasy mierzy okno, od żądania do odpowiedzi demona, więc obejmują drogę IPC tam i z powrotem. Wyłączone ukrywa czasy i wpisy udanych operacji. Niepowodzenia są zawsze wypisane. |
| `log.extended` | `false` | „Rozszerzone logowanie”: karta Operacje git wypisuje też operacje w tle (okresowe odświeżanie odpytujące o status, auto-fetch) z ich czasami. Zawsze trafiają one do dziennika aplikacji na poziomie `debug`. |

Pliki nazywają się `gitpoulpi-YYYY-MM-DD.log` i leżą w folderze stanu systemu (`~/.local/state/gitpoulpi/logs` w Linuksie; `<cache>\logs` w Windows i macOS; `<config>/cache/logs` przy `GITPOULPI_CONFIG_DIR`). Tokeny, hasła i nagłówki `Authorization` są maskowane, zanim cokolwiek zostanie zapisane.

## `[search]` (wyszukiwanie w obszarze roboczym)

To, co przeszukuje `gitpoulpi search` (i jego narzędzie MCP). Szczegóły: [Obszary robocze i wyszukiwanie w CLI](cli-reference.md#obszary-robocze-i-wyszukiwanie).

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `search.commit_limit` | `200000` | Commity (od najnowszego, ze wszystkich gałęzi, gałęzi zdalnych i tagów), których opisy indeks wyszukiwania trzyma w pamięci dla każdego repozytorium, około 100 bajtów plus opis każdego. Dłuższa historia jest tu ucinana, a odpowiedź mówi, że jest `partial`. `0` wyłącza wyszukiwanie commitów (gałęzie, tagi, stashe i worktree nadal są przeszukiwane). Czytane na żywo: następne wyszukiwanie uwzględnia zmianę. |
| `log.commit_count` | `"system-git"` | Jak pasek stanu ustala łączną liczbę w `wczytane / łącznie`. `"system-git"` uruchamia `git rev-list --count` (jeden proces, zwykle najszybciej: git czyta plik commit-graph). `"libgit2"` liczy w procesie przez revwalk libgit2 i niczego nie uruchamia: wybierz go, gdy każdy proces wiersza poleceń startuje wolno, zwykle dlatego, że oprogramowanie antywirusowe ze skanowaniem w czasie rzeczywistym sprawdza każdy z nich; przy 50 000 commitów trwa to w tle około 200 ms zamiast 12 ms, ale żaden proces nie jest uruchamiany. `"off"` nigdy nie liczy: pasek pokazuje tylko wczytane commity. Działa na żywo. Liczenie z filtrem ścieżki zawsze używa systemowego gita. |
| `log.use_mailmap` | `true` | Pokazuje każdego autora tak, jak nazywa go `.mailmap` repozytorium (i `mailmap.file`) w Logu, w blame i w historii pliku: demon prosi git o `%aN`/`%aE` (backend systemowy) albo sam stosuje te same pliki (backend libgit2), więc odpowiedź jest taka sama. Zapisane commity się nie zmieniają i nic nie jest przepisywane. Wyszukiwanie w Logu po autorze dopasowuje nazwy w takiej postaci, w jakiej są pokazywane. Działa na żywo; Log wczytuje się ponownie w ciągu 2 s odpytywania. |

## `[rewrite]` (przepisywanie historii)

To, czego pilnuje `gitpoulpi rewrite` i warsztat przepisywania historii. Szczegóły: [Przepisywanie historii](history-rewrite.md).

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Gałęzie (nazwy albo wzorce, w których `*` pasuje do dowolnego tekstu), które przepisanie zostawia w spokoju, gdy są już wypchnięte, chyba że uruchomienie je wymienia (`--include-protected BRANCH`; `--include-all-protected` przy wypchnięciu; pole wyboru przy każdej gałęzi w warsztacie). Dochodzą do baz, które `workflow.toml` repozytorium oznacza jako `protected`. Wypchnięcie przepisania pomija je tak samo. To własna ochrona GitPoulpi: o przyjęciu force push nadal decyduje ochrona gałęzi po stronie hosta. Czytane na żywo: następny podgląd stosuje zmianę. |
| `rewrite.external_scripts` | `["shell"]` | Jakich zewnętrznych języków skryptów może używać reguła Skrypt: dowolne z `"shell"` (bash lub sh; pwsh albo powershell dla skryptów PowerShell), `"python"`, `"lua"` (zainstalowany Lua; wbudowany Lua jest zawsze włączony i nie potrzebuje wpisu). **Skrypt zewnętrzny nie działa w piaskownicy: działa z twoimi uprawnieniami.** Język, którego nie ma na liście, jest w warsztacie wyszarzony, a CLI, MCP i demon go odmawiają (`scripts_disabled`). Nawet przy wpisie skrypt zewnętrzny pochodzący z repozytorium lub wspólnego przepisu działa dopiero po tym, jak mu zaufasz (`gitpoulpi rewrite script trust FILE` albo pytanie w warsztacie); zaufanie jest powiązane z hashem zawartości skryptu, więc zmieniony skrypt pyta ponownie. Pythona włączysz, edytując `config.toml` (listy nie da się ustawić przez `config set`): `[rewrite]`, a potem `external_scripts = ["shell", "python"]`. Czytane na żywo. |
| `rewrite.script_timeout_ms` | `5000` | Ile czasu skrypt zewnętrzny ma na odpowiedź dla jednego commita. Po przekroczeniu całe drzewo procesów jest zabijane, a przepisanie kończy się błędem (`script_failed`, z nazwą commita); nic się nie przesunęło. `0` jest czytane jako wartość domyślna. Czytane na żywo. |
| `rewrite.script_timeout_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują ten limit czasu: `milliseconds`, `seconds` albo `auto`. Tylko wyświetlanie: limit to zawsze `script_timeout_ms` milisekund. Ustawienia przyjmują od 1 do 600000 (dziesięć minut); `0` nadal jest czytane jako wartość domyślna. Czytane na żywo. |
| `rewrite.script_total_timeout_s` | `3600` | Ile może trwać w sumie jeden przebieg skryptu zewnętrznego. `0` jest czytane jako wartość domyślna. Czytane na żywo. |
| `rewrite.script_total_timeout_unit` | `"auto"` | Jednostka, w której Ustawienia pokazują ten limit czasu: `seconds`, `minutes`, `hours` albo `auto`. Tylko wyświetlanie: limit to zawsze `script_total_timeout_s` sekund. Ustawienia przyjmują od 1 do 86400 (doba); `0` nadal jest czytane jako wartość domyślna. Czytane na żywo. |

## `[updates]` (szukanie nowszego wydania)

Co robi znaczek wersji na pasku tytułu w sprawie nowszych wydań. Szczegóły: [Aktualizacje](updates.md).

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` raz dziennie, w tle, pyta GitHub, czy istnieje nowsze wydanie, i zamienia znaczek wersji w `vX.Y.Z dostępna`. `"auto"` dodatkowo sam pobiera i przygotowuje aktualizację, gdy ta kompilacja pochodzi z archiwum wydania (przed ponownym uruchomieniem nadal pyta); dla każdej innej kompilacji zachowuje się jak `"notify"`. `"off"` nigdy nie wysyła zapytań. Działa na żywo. |
| `updates.channel` | `"stable"` | `"stable"` uwzględnia tylko pełne wydania; `"prerelease"` oferuje też wersje kandydujące (`-rc.1`). Działa na żywo. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | `owner/name` na github.com, którego wydania są sprawdzane. Dopóki to repozytorium jest prywatne, GitHub odpowiada 404, chyba że dla github.com zapisano token (Ustawienia > Integracje); bez niego sprawdzenie cicho się nie udaje i zapisuje jeden wiersz w dzienniku aplikacji. |

Administrator może ustawić `mode`, `channel` (jako `release_channel`) i `repo`, oraz kilka dalszych opcji aktualizacji, w pliku polityki obowiązującej na całym komputerze albo w rejestrze Windows. Klucz ustawiony przez politykę wygrywa z `config.toml` i w Ustawieniach pokazuje się jako **Ustawione przez administratora**. `mode` w polityce to pułap: możesz wybrać niższą wartość, nigdy wyższą. Zob. [Dystrybucja w przedsiębiorstwie](enterprise-distribution.md#plik-polityki-i-rejestr).

## `[privacy]` (wrażliwe pliki)

| Klucz | Domyślnie | Wartości i działanie |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Wzorce ścieżek w stylu gitignore. Wzorzec bez `/` pasuje do nazwy pliku na dowolnej głębokości, z `/` do całej ścieżki (początkowe `/` kotwiczy go w korzeniu); `*` i `?` zostają w obrębie jednego folderu, `**` przechodzi przez foldery; `!` ponownie zezwala na to, co dopasował wcześniejszy wzorzec, a wygrywa ostatnie dopasowanie; wielkość liter jest ignorowana. Używane przez strażnika dodawania do poczekalni i przez `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Dodanie do poczekalni pliku pasującego do wzorca albo mającego w treści rozpoznany sekret najpierw pyta (TUI), jest odrzucane, chyba że podano `--allow-sensitive` (CLI), albo chyba że `mcp.allow_sensitive_staging` pozwala agentowi przekazać `allow_sensitive` (MCP). Pliki tekstowe do 1 MiB każdy i 32 MiB łącznie są czytane; pliki binarne są dopasowywane tylko po nazwie. Zob. [Stage i commity](staging-and-committing.md#wrażliwe-pliki). |

## `[keybindings]`

Zmienia przypisania skrótów TUI; edytuje je Ustawienia > Klawiatura i mysz.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # domyślnie Ctrl+G
nav_prefix = "Alt+n"             # domyślnie Ctrl+N
settings   = ["Ctrl+,", "F2"]    # lista = alternatywy
tab_menu   = []                  # nieprzypisane

[keybindings.git_prefix]         # klawisz naciskany po prefiksie git
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; wartość zastępuje domyślne
klawisze tej akcji, usunięcie wiersza je przywraca. Skróty: `Ctrl+Shift+N`,
`Alt+n`, `F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Błędny wpis niczego
nie blokuje: TUI go ignoruje, zachowuje domyślny i mówi o tym na pasku stanu.
Konteksty, identyfikatory akcji, wartości domyślne i zasady konfliktów:
[Skróty klawiszowe](keybindings.md).

## Ustawienia repozytorium: `.gitpoulpi/repo-state.toml`

Trzymane wewnątrz repozytorium; folder sam się ignoruje (`.gitignore` z `*`),
więc nigdy nie pokazuje się jako zmiana. Zapisywane przez TUI i CLI; rzadko
trzeba je edytować.

| Klucz | Co to jest | Ustawiane przez |
|---|---|---|
| `[preferences] encoding` | Nadpisuje tu `git.default_encoding` | Ustawienia > Git i kodowanie |
| `[preferences] lfs_enabled` | Nadpisuje tu `git.lfs_enabled` | Ustawienia > Git i kodowanie |
| `[preferences] profile` | Id `[[profiles]]`, który podpisuje tu commity; brak oznacza `general.active_profile` | lista rozwijana profilu na pasku narzędzi, **Repozytorium: Zmień profil...** |
| `[preferences] autocrlf`, `eol` | *Jeszcze nieużywane* | Ustawienia > Git i kodowanie |
| `[preferences] push_after_commit` | Push gałęzi do jej upstreamu po każdym commicie (przy braku wyłączone) | Ustawienia > Ustawienia repozytorium > Commit |
| `[preferences] skip_hooks_by_default` | Pole Pomiń hooki w formularzu commita jest domyślnie zaznaczone (przy braku wyłączone) | Ustawienia > Ustawienia repozytorium > Commit |
| `[preferences] squash_merge` | Merge lokalnej gałęzi robi squash: dodaje do poczekalni, bez merge commita (przy braku wyłączone) | Ustawienia > Ustawienia repozytorium > Opcje: merge |
| `[preferences] template_apply`, `template_strip_comments` | Wstępnie wypełnia pusty formularz commita szablonem; usuwa linie komentarzy `#` przy tworzeniu commita (oba wyłączone przy braku) | Ustawienia > Ustawienia repozytorium > Szablon opisu |
| `[preferences] conflict_detection` | Proaktywne wykrywanie konfliktów dla tego repozytorium; brak podąża za `general.conflict_detection` | Ustawienia > Ustawienia repozytorium > Zapobieganie konfliktom |
| `[preferences] conflict_base_branches` | Wzorce Gałęzi bazowych, po jednym w wierszu (`**` przy braku) | Ustawienia > Ustawienia repozytorium > Zapobieganie konfliktom |
| `commit-template.txt` (plik obok `repo-state.toml`) | Szablon commita; `commit.template` gita wskazuje na niego | Ustawienia > Ustawienia repozytorium > Szablon opisu |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: instrukcje AI tego repozytorium | Ustawienia > Ustawienia repozytorium > AI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` dla jednej akcji tutaj | Ustawienia > Ustawienia repozytorium > AI |
| `default_remote` | Zdalne repozytorium używane przez jednoklikowe Push/Pull/Fetch | menu zdalnych repozytoriów, `gitpoulpi remote default <name>` |
| `hosting_integration` | Profil `[[integrations]]` przypięty tutaj | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Projekt dostawcy, który czytają listy PR / zgłoszeń, jako `{ integration, project, filter }`. `filter` jest opcjonalny: natywna dla dostawcy dodatkowa klauzula (fragment JQL Jira, warunek WIQL Azure, nazwy list Trello) dołączana przez AND do zapytania listy; GitHub i GitLab go ignorują. | sekcje panelu bocznego, `gitpoulpi hosting link` |
| `ref_visibility` | Ukryte / wyróżnione solo refy | panel boczny Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Zdalne ukryte w grafie w całości (każda gałąź, także pobrane później) | oko i menu zdalnego, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Zwinięte sekcje panelu bocznego oraz foldery gałęzi i tagów. `[folds]` zawiera też `status_folders` (zwinięte foldery drzew plików Unstaged / Staged) i `commit_folders` (zwinięte foldery drzewa zmienionych plików commita, takie same dla każdego commita), każde ograniczone do 500 ścieżek, najstarsze są odrzucane | panel boczny, drzewa plików |
| `pending_auto_stash` | Stash, który GitPoulpi utworzył i jeszcze nie przywrócił | zapisywany przez demona; zob. [Auto-stash](log-and-history.md#auto-stash) |

## Inne zmienne środowiskowe

| Zmienna | Działanie |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Folder konfiguracji (zob. wyżej). |
| `GITPOULPI_TOKEN_<ID>` | Token hostingu dla profilu `<ID>`, ponad zapisanym (CI, użycie jednorazowe). |
| `GITPOULPI_GIT` | Plik wykonywalny `git` do uruchamiania (domyślnie: `git` z `PATH`; w Windows prawdziwy `mingw64\bin\git.exe` za nakładką `cmd\git.exe` Git for Windows, co oszczędza proces na wywołanie). |
| `GITPOULPI_LANG=fr` | Język interfejsu, ponad `ui.language` (dowolna wartość `ui.language`; `qps` to pseudo-ustawienia regionalne dla tłumaczy). Nieznana lub pusta wartość jest ignorowana. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Żadnych obrazów, niezależnie od konfiguracji. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Wskazówki rozdzielczości obrazów (rozmiar komórki w pikselach; nadpróbkowanie, domyślnie 2, `1` oszczędza CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nigdy nie uruchamia przeglądarki (adres URL nadal jest pokazywany). |
| `GITPOULPI_TIMING=1` | Zapisuje czasy żądań do `gitpoulpi-timing.log` w folderze tymczasowym (raporty wydajności). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Nie czeka na Enter po zakończeniu TUI z błędem. |
