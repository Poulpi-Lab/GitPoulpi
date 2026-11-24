[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · **Polski** · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Otwórz w

Przycisk **Otwórz w ▾** na pasku tytułu, obok ścieżki repozytorium, otwiera folder aktywnej karty
gdzie indziej: w twoim terminalu, nowej karcie lub panelu multipleksera, edytorze,
menedżerze plików. To ty wybierasz, co jest na liście. Nic nie jest wpisane na stałe: każdy
wpis to etykieta i polecenie.

## Użycie

- **Kliknij Otwórz w ▾**: rozwija się lista. Kliknij wpis, by go uruchomić. Wpis
  domyślny jest oznaczony `•`; ostatni wiersz, **Konfiguruj...**, otwiera
  [Ustawienia](settings.md#otwórz-w) na stronie Otwórz w.

  ![Lista Otwórz w: Terminal, dwa edytory, narzędzie Git i menedżer plików, domyślny oznaczony kropką, a na końcu Konfiguruj](../images/open-in-menu.png)

- **Klawiatura**: `Alt+O` otwiera tę samą listę (`Up`/`Down`, `Enter`, `Esc`; przy
  więcej niż siedmiu wpisach wpisz tekst, by szukać). `Ctrl+Alt+O` od razu uruchamia wpis
  domyślny. Paleta (`Ctrl+P`): `Karta: Otwórz w...` i `Karta: Otwórz w domyślnym`.
  Polecenia palety działają, nawet gdy na pasku tytułu nie ma miejsca na przycisk.
  Oba klawisze można przypisać na nowo; zob. [Skróty klawiszowe](keybindings.md).
- **Według etykiety**: krok `tab_open_in` [sekwencji](config-reference.md#sequences) uruchamia wpis, którego
  **etykietę** podasz (`label = "VS Code"`), więc zmiana kolejności listy nigdy nie zmienia tego,
  co się uruchamia. Nieznana etykieta zatrzymuje krok z komunikatem „żaden wpis Otwórz w nie nazywa się ...”.
  `Karta: Terminal tutaj` (paleta) otwiera twój terminal w folderze, dawny przycisk
  „Terminal tutaj” paska kart.
- **Agent AI w folderze**: `Karta: Otwórz agenta AI tutaj` otwiera twój terminal w folderze
  karty z uruchomionym lokalnym agentem AI; zob. [AI](ai.md#otwórz-agenta-w-folderze-karty).
- **Kliknij prawym przyciskiem kartę repozytorium**: menu karty ma podmenu **Otwórz w** z
  tymi samymi wpisami.
- Lista zaczyna się od wpisu, którego użyto ostatnio w tej sesji.
- Pasek stanu od razu mówi „otwieranie w ...”. Jeśli program zawiedzie w ciągu
  około 1,5 sekundy (na przykład tmux bez serwera albo zła flaga), pierwszy
  wiersz jego komunikatu o błędzie pojawia się na pasku stanu. Niezainstalowany program jest od razu wymieniony
  z nazwy.

Dopóki nie zapiszesz własnej listy, wpis domyślny jest wybierany na podstawie tego, co wykryto: w herdr „Herdr: new tab”; w Omarchy nowe okno kitty (kitty jest jego terminalem); poza tym menedżer plików.

Przycisk pokazuje się na każdej karcie repozytorium, także na folderze, w którym uruchomiono GitPoulpi,
oraz w trybie `ui.tab_dir_follow = "auto"`. W trybie `auto` wpis
domyślny uruchamia się też sam przy otwarciu nowej karty (nie dla kart przywracanych
przy starcie). Zob. [Podążanie za folderem aktywnej karty](following-the-tab-folder.md).

## Jak wygląda wpis

W `config.toml` jedna tabela `[[ui.open_in]]` na wpis:

```toml
[ui]
open_in_default = "Herdr: new tab"   # etykieta; puste oznacza pierwszy wpis

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Rzadko pisze się to ręcznie: edytuje to strona Ustawień. Każdy klucz jest w
[opisie konfiguracji](config-reference.md).

### Polecenia i zmienne

Polecenie jest najpierw dzielone na argumenty (spacje, `'...'` i `"..."` grupują
słowa), a potem zmienne są podstawiane wewnątrz każdego argumentu. Folder ze
spacją w nazwie pozostaje więc jednym argumentem. **Żadna powłoka nie jest używana**, chyba że
twoje polecenie ją uruchamia.

| Zmienna | Zastępowana przez |
|---|---|
| `${cwd}` | Folder karty. |
| `${repo}` | Katalog główny repozytorium (różni się od `${cwd}` dla karty worktree). |
| `${name}` | Nazwa bazowa folderu. |
| `${branch}` | Bieżąca gałąź; pusta, gdy HEAD jest odłączony. |
| `${web_url}` | Strona WWW zdalnego repozytorium origin, gdy jest to GitHub lub GitLab; w przeciwnym razie pusta. |
| `${env:NAME}` | Zmienna środowiskowa `NAME`; pusta, gdy nie jest ustawiona. |

Stare `{repo}` nadal działa i oznacza `${cwd}`. Każde inne `${...}` jest błędem,
który wskazuje zmienną.

Program jest uruchamiany z folderem karty jako katalogiem roboczym i dostaje
trzy zmienne środowiskowe, przydatne we fragmencie powłoki:

| Zmienna | Wartość |
|---|---|
| `GITPOULPI_CWD` | Folder karty. |
| `GITPOULPI_REPO` | Katalog główny repozytorium. |
| `GITPOULPI_BRANCH` | Gałąź albo pusta. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

W Windows program jest wyszukiwany przez `PATH` i `PATHEXT`, więc `code` i
`cursor` (pliki `.cmd`) działają i nie mignie żadne okno konsoli.

## Ustawienia wstępne

Ustawienia > Otwórz w > **Dodaj ustawienie wstępne** wypisuje gotowe wpisy tylko dla twojego systemu
operacyjnego (ustawienia wstępne Windows nigdy nie pokazują się w Linuksie i tak dalej), według kategorii: wiersz
zaczyna się od kategorii, `Eksplorator: ...`, `Multiplekser: ...`, `Terminal: ...`,
`Edytor: ...`, `Narzędzie Git: ...`, `Agent AI: ...`, `Web: ...`. Na górze **Wykryte** to
wpisy wyliczonej listy domyślnej. W każdej kategorii najpierw idą ustawienia wstępne, których możesz
użyć od razu (działasz wewnątrz tego multipleksera lub terminala, a
program jest w `PATH`), potem zainstalowane, potem reszta. Wpisz tekst w polu
kombinowanym, by szukać, na przykład `lazygit` lub `Narzędzie Git:`. **Testuj na bieżącej karcie**
wypróbowuje wpis, zanim zaczniesz na nim polegać.

![Lista + Dodaj na stronie Otwórz w: najpierw wpisy Wykryte, potem Eksplorator, Multiplekser i pozostałe kategorie, każde ustawienie wstępne poprzedzone swoją kategorią](../images/settings-open-in-presets.png)

Ustawienia wstępne „uruchom narzędzie w panelu” (`lazygit in Herdr tab`, `yazi in tmux window`,
`Claude Code in Herdr split`...) są oferowane tylko wtedy, gdy host (herdr, tmux,
zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal) i narzędzie są
zainstalowane. `$EDITOR in ...` uruchamia edytor wskazany przez twoją zmienną `EDITOR`
(w herdr z zapasowym `vi`) w herdr lub tmux.

Ustawienia wstępne narzędzi dla herdr tworzą kartę lub podział, a potem wpisują polecenie do jego
powłoki, więc narzędzie startuje w folderze, a panel pozostaje otwarty po jego zakończeniu.
Te z kategorii `Agent AI` używają `herdr agent start`, więc herdr śledzi stan agenta.

| Kategoria | Ustawienia wstępne (`V` = zweryfikowane przez uruchomienie lub względem zainstalowanej pomocy) |
|---|---|
| Eksplorator | Windows `explorer.exe` (V), Finder macOS (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger w panelu/karcie/terminalu |
| Multiplekser | herdr karta, workspace, podział w prawo/w dół (V); tmux okno, podział w prawo/w dół, odłączona sesja (V); zellij panel, karta, pływający; WezTerm karta, podział, nowe okno; kitty karta, okno podziału, okno systemu (V: flagi z pomocy; wymaga `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (okno, karta, podział i po jednej karcie dla PowerShell 7, Windows PowerShell, Wiersza polecenia, WSL), okna PowerShell 7 / Windows PowerShell / Wiersza polecenia / WSL, Git Bash |
| Edytor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` w panelu/karcie/terminalu |
| Narzędzie Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui w panelu/karcie/terminalu (te dla herdr, tmux, kitty i `xdg-terminal-exec` V) |
| Agent AI | Claude Code, Codex, Gemini CLI, opencode w karcie/podziale herdr (V, z Claude), tmux, zellij, WezTerm, kitty, nowym terminalu |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL w Windows: `wsl.exe --cd ${cwd}` przyjmuje ścieżkę Windows bez zmian, a
karta `wsl.exe` w Windows Terminal startuje w folderze, który WSL mapuje pod
`/mnt/`.

Eksplorator, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio
i Zed (Zed jest tylko dla Uniksa) pojawiają się tylko tam, gdzie istnieją; tmux, zellij, cmux,
screen, zdalne sterowanie kitty i ustawienia wstępne herdr oparte na `sh` są tylko dla Uniksa;
herdr (zwykłe ustawienia wstępne), WezTerm, Alacritty, VS Code, Cursor i IDE JetBrains są oferowane
wszędzie.

### Zmienna `${web_url}`

Strona WWW zdalnego repozytorium `origin`, gdy jest to projekt GitHub lub GitLab
(github.com, gitlab.com albo host self-managed, który ma wpis w
`[[integrations]]`); w przeciwnym razie pusta, a wtedy wpis, który jej używa, zatrzymuje się z
komunikatem zamiast otwierać pusty adres. Jest opcjonalna: żadne ustawienie wstępne jej nie potrzebuje,
poza `Repository page on the web`.

## Strona Ustawień

Ustawienia > **Otwórz w** edytuje listę. Wpisy tworzą [listę](settings.md#listy) po lewej
(wpis domyślny jest oznaczony `★`), pola zaznaczonego wpisu są po prawej; listą zarządzają
`+ Dodaj`, `Duplikuj`, `Usuń`, `↑` / `↓` i przeciąganie wiersza.

| Wiersz | Co robi |
|---|---|
| **Wpis domyślny** | Wpis, który uruchamia **Otwórz w domyślnym**. Jest oznaczony `•` na liście Otwórz w. |
| **Etykieta** | Nazwa pokazywana na liście. Unikalna, niepusta. |
| **Polecenie** | Polecenie, z przyciskami zmiennych (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) do wstawienia. |
| **Testuj na bieżącej karcie** | Uruchamia wpis teraz w folderze aktywnej karty. |
| **+ Dodaj ▾** | Dopisuje gotowy wpis (najpierw wykryte) albo **Własny**, czyli pusty wpis, którego Polecenie edytujesz zaraz potem. |
| **Przywróć wykryte ustawienia** | Zastępuje listę tym, co wykryto na tym komputerze, i czyści domyślny. |

Zmiany są zapisywane na bieżąco. Pierwsza edycja zapisuje całą tablicę `[[ui.open_in]]`
do `config.toml`; do tego czasu lista jest wyliczana (zob. [Starsze klucze](#starsze-klucze)), a plik
nie ma klucza `open_in`. Wyszukiwarka Ustawień znajduje też tę stronę.

## Pliki binarne: Otwórz w

Plik binarny (PDF, dokument Word, arkusz kalkulacyjny, archiwum...) nie ma tekstu do pokazania w widoku Diff ani w widoku pliku. Gdy system operacyjny zna aplikację dla jego typu, widok pokazuje obok zwykłego komunikatu przycisk **Otwórz w**. Jego lista zawiera najpierw aplikację, którą system otwiera ten typ (oznaczoną kropką), a potem wszystkie pozostałe aplikacje zarejestrowane w systemie dla tego typu (własną listę systemu *Otwórz za pomocą*). W Linuksie jest to typ MIME nazwy pliku, `mimeapps.list` dla aplikacji domyślnej i pliki `.desktop` dla pozostałych; w Windows rejestr (wybór użytkownika i programy zarejestrowane dla rozszerzenia); w macOS tylko aplikacja domyślna.

`o` uruchamia aplikację domyślną, a `Shift+O` otwiera listę. Oba są w palecie poleceń (*Plik: Otwórz w aplikacji domyślnej*, *Plik: Otwórz za pomocą...*), a **Otwórz za pomocą...** jest w menu kontekstowym pliku w panelu Status i na liście plików commita. Wyszukiwanie działa w tle, raz dla każdego rozszerzenia: w tym czasie widok pokazuje „wyszukiwanie aplikacji...”. Typ, dla którego system nie zna żadnej aplikacji, oraz zwykły tekst zachowują zwykły komunikat i nie mają przycisku.

![Plik PDF w widoku pliku z listą Otwórz w: najpierw aplikacja domyślna, oznaczona kropką, potem pozostałe aplikacje zarejestrowane w systemie dla tego typu](../images/open-with.png)

Plik z drzewa roboczego jest przekazywany aplikacji bez zmian. Wersja z commita (lub z indeksu) jest zapisywana w **tymczasowej kopii tylko do odczytu**, z komunikatem na pasku stanu, aby aplikacja nie mogła zmienić repozytorium. Żadne z nich nie zmienia repozytorium: brak wpisu w dzienniku, Cofnij pozostaje nietknięte.

Lista menu otwiera się obok menu, na wysokości wiersza **Otwórz za pomocą...**, jak podmenu (po lewej, gdy po prawej brak miejsca), zarówno z myszy, jak i z klawiatury. Lista edytorów obrazów (**Edytuj w...**) otwierana z menu pliku robi to samo.

### Pliki Git LFS

Karta obiektu Git LFS (zob. [Git LFS](lfs.md)) niesie pod swoim tekstem te same dwie akcje: **Otwórz w domyślnym** i **Otwórz w**, z tymi samymi klawiszami (`o`, `Shift+O`) i wierszami palety. Otwierają prawdziwy plik, nigdy wskaźnik: plik z drzewa roboczego bez zmian albo tymczasową kopię tylko do odczytu obiektu z lokalnego magazynu LFS dla wersji z commita lub z indeksu. Obiekt, który nie został jeszcze pobrany, nie ma przycisków na karcie, a pasek stanu mówi, że zawartość nie jest jeszcze pobrana. To samo odrzucenie dotyczy pliku z drzewa roboczego, który nadal zawiera tylko wskaźnik: **Otwórz w domyślnym**, **Otwórz w** i **Edytuj w**, z karty lub z dowolnego menu pliku, mówią, że zawartość nie jest pobrana, zamiast otwierać tekst wskaźnika (pobierz ją wierszami pull lub checkout paska LFS i spróbuj ponownie). Otwieranie jest tylko do odczytu: brak wpisu w dzienniku, nie ma czego cofać.

### Przeglądarki w Windows

Chrome i Edge rejestrują `--single-argument %1`, przez co czytają wszystko po przełączniku jako jeden adres, razem z cudzysłowami. GitPoulpi przekazuje ścieżkę takiemu programowi dosłownie i bez cudzysłowów, tak jak robi to Windows, więc plik ze spacjami w ścieżce się otwiera, zamiast kończyć się błędem `http://"c/...`. Każdy inny program dostaje ścieżkę jako jeden zwykły argument.

## Starsze klucze

Stare klucze `ui.terminal_launch` i `ui.terminal_launch_command` są czytane tylko
wtedy, gdy brakuje `ui.open_in`. Wtedy wyliczona lista to:

1. twój dawny wybór terminala, jeśli nie jest `auto` (`custom` staje się
   „Terminal (custom)” z `{repo}` przepisanym na `${cwd}`; identyfikator ustawienia wstępnego staje się
   tym ustawieniem). Wartość `auto` staje się pierwszym wpisem, „Default terminal”.
2. eksplorator plików systemu,
3. multipleksery i terminale wykryte ze środowiska,
4. edytory znalezione w `PATH`.

Gdy zapiszesz listę z Ustawień, `ui.open_in` przejmuje kontrolę, a stare klucze
są ignorowane. Polecenia palety **Karta: Otwórz w terminalu** i **Karta: Otwórz w
menedżerze plików** nie używają tej listy.

## Rozwiązywanie problemów

- *„program X was not found”*: programu nie ma w `PATH` widzianym przez
  GitPoulpi. Uruchom GitPoulpi z powłoki, która go ma, albo wpisz pełną ścieżkę.
- *wpis tmux/zellij/herdr zawodzi od razu*: rozmawiają z działającym serwerem, więc
  działają tylko wtedy, gdy GitPoulpi działa wewnątrz niego.
- *wpis karty kitty zawodzi*: wymaga `allow_remote_control` w `kitty.conf`.
- *Eksplorator Windows*: `explorer.exe` kończy się kodem 1 nawet przy powodzeniu; GitPoulpi
  traktuje to jako sukces.
