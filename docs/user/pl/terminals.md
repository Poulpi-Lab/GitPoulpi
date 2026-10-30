[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · **Polski** · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Wybór terminala

GitPoulpi działa w każdym terminalu z obsługą myszy, ale wygląda i działa
najlepiej w takim, który rysuje obrazy, pokazuje true color i wysyła pełną
klawiaturę. Ta strona porządkuje terminale według systemu, mówi, co każdy z nich
daje, i które ustawienie zmienić. Fakty sprawdzono w dokumentacji, dzienniku
zmian lub śledzeniu błędów każdego projektu w październiku 2026 (zob.
[Źródła](#źródła)); nie wszystko przetestowano w każdym terminalu.
Tego, co oznaczono **(niezweryfikowane)**, nie dało się potwierdzić w źródle
pierwotnym.

## Najlepsze wybory

| System | Najlepsze wybory, w kolejności | Dlaczego |
|---|---|---|
| **Windows** | **WezTerm**, potem **Windows Terminal** | WezTerm rysuje graf i awatary jako obrazy, z przezroczystością. Windows Terminal ma najlepszą integrację z systemem (jest domyślny w launcherze); awatary rysuje jako obrazy sixel, a graf jako znaki. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, potem **foot** | kitty i Ghostty dają obrazy i pełną klawiaturę. WezTerm też daje obrazy. foot rysuje awatary jako obrazy sixel, jest szybki i ma pełną klawiaturę. W Omarchy wybierz Ghostty lub kitty, jeśli chcesz obrazy. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, potem **iTerm2** | Te same powody. Terminal.app działa, ale nie ma ani obrazów, ani true color. |

W każdym systemie **tmux, zellij i screen** ukrywają przed GitPoulpi protokół
obrazów terminala, więc program rysuje znaki (zob.
[Multipleksery](#multipleksery)).

## Czego używa GitPoulpi

- **Obrazy** (kolumna grafu, awatary autorów, diff obrazów i widok pliku):
  **protokół grafiki kitty** z zastępnikami Unicode, **obrazy inline iTerm2**
  (`OSC 1337`) albo **sixel**, w tej kolejności preferencji.
  Wykrywanie działa na zasadzie „domyślnie odmawiaj”, na podstawie `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`,
  `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` i zmiennych
  multiplekserów, i nigdy nie czyta klawiatury (więc nie pyta też terminala o
  jego atrybuty urządzenia).
  kitty i Ghostty używają protokołu kitty. WezTerm używa protokołu iTerm2
  (celowo) i uwzględnia przezroczystość PNG. iTerm2, Rio i mintty używają
  protokołu iTerm2 z nieprzezroczystym tłem. Windows Terminal, foot, Konsole i
  mlterm używają **sixela**. Gdy nic z tego nie pasuje, awatary to plakietki z
  inicjałami.
- **Sixel** jest rysowany dla **kolumny grafu** (linie, węzły i, przy `ui.avatars = "graph"`, awatary w kółkach commitów), a gdzie indziej dla **awatarów** (Log, szczegóły commita, Historia pliku, Blame, widok obrazu). Sixel nie ma kanału alfa, identyfikatorów ani skalowania, więc obraz ma dokładnie `kolumny x szerokość komórki` na `wiersze x wysokość komórki` pikseli, rysowany tylko tam, gdzie coś jest, nad pustymi komórkami zachowującymi tło; zakodowany obraz jest w pamięci podręcznej, kodowany ponownie tylko przy zmianie linii, przewinięcia lub powiększenia i wysyłany ponownie tylko gdy komórka pod nim została przemalowana. Sixel nie ma operacji usuwania: GitPoulpi zapisuje tekst na jego komórkach, żeby go usunąć. Sixel potrzebuje prawdziwego rozmiaru komórki w pikselach: Windows Terminal jest o niego pytany raz przy starcie i po zmianie powiększenia (`CSI 16 t`, odpowiedź w około 10 ms, nigdy nie blokuje, gdy milczy); gdzie indziej bierze się go z rozmiaru okna lub z `GITPOULPI_TERMINAL_CELL_PX=10x20`. Terminal, który rysuje sixel, ale nie jest rozpoznawany (terminal VS Code z `terminal.integrated.enableImages`, xterm uruchomiony z `-ti vt340`), wymaga `ui.image_protocol = "sixel"`. Windows Terminal przed 1.22 nie rysuje sixela; ustaw tam `ui.image_protocol = "none"` (graf zostaje wtedy znakowy).
- **True color**: motywy są w RGB. `ui.color_depth = "auto"` schodzi do 256
  kolorów na `TERM` typu `*-256color` bez `COLORTERM=truecolor` (oraz w
  Terminal.app), a na konsoli Linuksa do 16 kolorów. kitty, WezTerm i Windows
  Terminal liczą się jako true color.
- **Mysz**: raportowanie myszy SGR z przyciskami, przeciąganiem i ruchem (reakcja
  na najechanie, drag and drop, prawy przycisk, podwójne kliknięcie).
- **Klawiatura**: **protokół klawiatury kitty** (tylko flaga disambiguate) jest
  żądany przy starcie z limitem czasu 100 ms. Dzięki niemu docierają `Ctrl+,`,
  `Ctrl+Enter`, `Ctrl+Shift+N` i `Ctrl+Shift+Z`. Bez niego zostają klawisze
  dawnego typu (`Ctrl+Y` ponawia, warianty z `Alt` zastępują część skrótów).
  Zob. [Skróty klawiszowe](keybindings.md#klawisze-których-terminal-nie-może-wysłać-i-protokół-klawiatury-kitty).
- **Schowek**: akcje kopiowania zapisują `OSC 52`.
- **Kształt wskaźnika myszy** (wskaźnik zmiany rozmiaru nad separatorami, dłoń nad
  elementami do kliknięcia): `OSC 22`, dodatek tam, gdzie jest zaimplementowany.
- **Zdarzenia fokusu** są włączone (podpowiedzi zamykają się, gdy okno traci
  fokus). GitPoulpi nie żąda bracketed paste, hiperłączy OSC 8 ani falistego
  podkreślenia. Tylko na ścieżce z obrazami każda klatka jest opakowana w
  zsynchronizowaną aktualizację (tryb 2026, ignorowany przez terminale, które go
  nie znają), żeby obraz nigdy nie migał nad wyskakującym oknem.
- **Emoji i symbole**: GitPoulpi używa tylko glifów z domyślną prezentacją emoji
  albo jednokomórkowych kształtów z BMP, bo terminale i czcionki różnie traktują
  selektory wariantów.

## Tabela możliwości

W kolumnie „Obrazy” stoi to, co GitPoulpi rysuje dziś w danym terminalu
(protokół **kitty**, **iTerm2** lub **sixel**, albo **tekst**), a nie to, co terminal mógłby
zrobić.

| Terminal | System | Obrazy (graf, awatary, diff obrazów) | True color | Klawiatura kitty (pełne skróty) | Kopiowanie OSC 52 | Wskaźnik OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | tak | tak | tak | tak |
| **Ghostty** (1.3) | Linux, macOS (brak wersji na Windows) | kitty | tak | tak | tak | tak, od 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, z przezroczystością | tak | tylko z `enable_kitty_keyboard = true` (domyślnie `false`) | tak (tylko zapis) | nie (pull request nadal otwarty) |
| **foot** (1.28) | Linux (Wayland) | sixel (graf i awatary; bez grafiki kitty) | tak | tak, od 1.10.3 | tak (`security.osc52`) | tak, od 1.12 |
| **Alacritty** (0.17) | wszystkie | tekst (brak obsługi obrazów w projekcie) | tak | tak, od 0.13 | tak (zapis; wklejanie domyślnie wyłączone) | nie (niezaimplementowany) |
| **Konsole** | Linux | sixel (graf i awatary) | tak | w trakcie dodawania w 2026 (merge request !1188); wydanie (niezweryfikowane) | tylko zapis, włączone w połowie 2024 | (niezweryfikowane) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | tekst | tak | nie (brak w wydanym VTE) | nie (VTE to ignoruje) | nie |
| **iTerm2** (3.7) | macOS | iTerm2, nieprzezroczyste tło | tak | tak, od 3.6.9 (sprawdź Profiles > Keys) | domyślnie wyłączone: włącz „Applications in terminal may access clipboard” | (niezweryfikowane) |
| **Terminal.app** | macOS | tekst | nie (256 kolorów) | nie | (niezweryfikowane) | (niezweryfikowane) |
| **Windows Terminal** (1.25) | Windows | sixel (graf i awatary) | tak | tak, stabilnie od 1.25 (paź 2026); Preview od 1.25 (mar 2026) | tak (zapis; tylko przy fokusie) | nie (niezaimplementowany) |
| **Rio** | Windows, Linux, macOS | iTerm2, nieprzezroczyste tło (mówi też grafiką kitty) | tak | tak (domyślnie włączone) | tak | nie (niezaimplementowany) |
| **Warp** | macOS, Linux, Windows | tekst (niewykrywany) | tak | tak | tak (honoruje sekwencję) | (niezweryfikowane) |
| **Terminal VS Code** (xterm.js) | wszystkie | tekst (sixel i obrazy iTerm2 są za `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` używa sixela) | tak | (niezweryfikowane) | (niezweryfikowane) | nie |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, nieprzezroczyste tło | tak | (niezweryfikowane; brak w dokumentacji) | (niezweryfikowane; brak w dokumentacji) | tak |
| **Konsola Windows** (conhost, cmd, okno PowerShell) | Windows | tekst | tylko nowsze Windows 10/11 | niepotrzebna: sama zgłasza modyfikatory | nie | nie |
| **ConEmu** | Windows | tekst | (niezweryfikowane) | (niezweryfikowane) | (niezweryfikowane) | (niezweryfikowane) |
| **tmux, zellij, screen** | wszystkie | zawsze tekst | wymaga ustawień | wymaga ustawień | wymaga ustawień | nie |

Komórki „nie (niezaimplementowany)” przy OSC 22 wynikają z braku wzmianki w
dzienniku zmian i dokumentacji projektu oraz z relacji użytkowników, a nie z
testu; traktuj je jako „niedokumentowane”. Strona nie zestawia szerokości emoji
i selektorów wariantów: różni się ona między terminalami i czcionkami, dlatego
GitPoulpi unika glifów VS16 (zob. [Czego używa GitPoulpi](#czego-używa-gitpoulpi)).

## Terminal po terminalu

### Windows

- **WezTerm.** Najlepsza obsługa obrazów w Windows. Zostaw `ui.image_protocol`
  na `auto`. W `wezterm.lua` ustaw `config.enable_kitty_keyboard = true`, żeby
  mieć pełne skróty, oraz czcionkę z zapasową czcionką emoji, na przykład
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Ostatnie oznaczone wydanie stabilne WezTerm to 20240203-110809-5046fc22
  (luty 2024); nowsze poprawki trafiają do wydań nightly. WezTerm nie ma jeszcze
  kształtu wskaźnika OSC 22.
- **Windows Terminal.** True color, mysz, OSC 52 i dobre emoji. GitPoulpi rysuje
  awatary jako obrazy sixel (Windows Terminal 1.22 lub nowszy); graf składa się ze
  znaków. Wybierz Nerd Font w *Settings > Profiles >
  Defaults > Appearance > Font face*. Przytrzymaj `Shift`, żeby zaznaczać tekst, gdy mysz jest
  przechwycona. Kształt wskaźnika się nie zmienia. W wersji starszej niż 1.25
  (bez protokołu klawiatury kitty) używaj wariantów z `Alt`.
- **Rio** działa z nieprzezroczystym tłem obrazów; mniej przetestowane.
- **mintty** (Git Bash, MSYS2, Cygwin) rysuje obrazy przez protokół iTerm2 z
  nieprzezroczystym tłem; mniej przetestowane.
- **Konsola Windows, ConEmu, cmd**: unikaj. Obrazy są rysowane źle albo wcale.
  Podwójne kliknięcie pliku exe otwiera GitPoulpi ponownie w wybranym przez
  ciebie terminalu (*Ustawienia > Ogólne > Uruchamianie z systemu*).

### Linux

- **kitty.** Wszystko działa bez żadnych ustawień. `Shift+klik` należy do samego
  kitty, więc do rozszerzenia zaznaczenia używaj `Alt+klik`
  ([Mysz i gesty](mouse-and-gestures.md)).
- **Ghostty.** Wszystko działa bez żadnych ustawień. Jeśli ikony pokazują się
  jako dziury, dodaj emoji lub Nerd Font do `font-family` (powtórz klucz, by
  dodać czcionki zapasowe).
- **WezTerm.** Obrazy przez protokół iTerm2, przezroczystość uwzględniona. Ta sama
  uwaga o `enable_kitty_keyboard` co w Windows.
- **foot.** Awatary to obrazy sixel, graf jest znakowy, pełna klawiatura, szybki. Ustaw `font=JetBrainsMono Nerd Font:size=11` w
  `foot.ini`.
- **Alacritty.** Graf znakowy, klawiatura kitty od 0.13. Alacritty nie obsługuje
  obrazów.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Działają ze znakami i klawiszami
  dawnego typu: używaj wariantów z `Alt` oraz `Ctrl+Y` do ponawiania (nowsze
  Konsole może dodać protokół klawiatury kitty). Akcje kopiowania w VTE (`OSC 52`)
  nic nie wkładają do schowka. Metoda wprowadzania (IBus, Fcitx) może przejąć
  przełącznik stage `Ctrl+Space` - **(niezweryfikowane)**; zmień ten skrót w
  *Ustawienia > Klawiatura i mysz*.
- **Wayland, Hyprland, Omarchy.** Podpowiedź kształtu wskaźnika myszy (`OSC 22`)
  działa w kitty, Ghostty i foot. Wpis launchera w Omarchy przechodzi przez
  `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: tak jak w Linuksie.
- **iTerm2.** Obrazy z nieprzezroczystym tłem (widoczny prostokąt, jeśli twoje
  tło różni się od tła motywu; `ui.graph_renderer = "text"` to usuwa). Włącz
  *Settings > General > Selection > Applications in terminal may access clipboard*,
  żeby kopiować, oraz *Profiles > Keys > Left Option key: Esc+* dla skrótów z
  `Alt`.
- **Terminal.app.** Tylko 256 kolorów: GitPoulpi sam wybiera `ui.color_depth = 256`
  i motywy wyglądają płasko. Bez obrazów. Zaznacz *Use Option as Meta key* dla
  skrótów z `Alt`.

### VS Code i Warp

Terminal VS Code (`TERM_PROGRAM=vscode`) i Warp nie są rozpoznawane jako terminale
z obrazami, więc GitPoulpi rysuje znaki. Oba nadają się do użytku. W VS Code
włącz `terminal.integrated.enableImages` i ustaw `ui.image_protocol = "sixel"`, żeby
mieć obrazy awatarów. Warp
dokumentuje obsługę protokołu klawiatury kitty; obsługa w terminalu VS Code jest
**(niezweryfikowana)**.

## Multipleksery

GitPoulpi wykrywa tmux, zellij, screen i herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` lub `tmux*`, `HERDR_*`) i rysuje **znaki**, ponieważ multiplekser
przekazuje komórki, a nie sekwencje obrazów. Dla pozostałych funkcji ustaw w
`~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # true color
set -g extended-keys on                # pełne skróty (tmux 3.2 lub nowszy)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a lub nowszy; tylko jeśli wymuszasz obrazy, zob. niżej
```

Aby wymusić obrazy przez multiplekser, który je przekazuje, ustaw
`ui.image_protocol` na `kitty` lub `iterm2`.
herdr z `kitty_graphics = true` wewnątrz kitty, Ghostty lub WezTerm jest
wykrywany automatycznie; zob.
[Rozwiązywanie problemów i FAQ](troubleshooting.md#graf-jest-rysowany-znakami-a-nie-jako-obrazek-herdr-tmux-inne-terminale).
`GITPOULPI_TERMINAL_IMAGE=off` wyłącza wszystkie obrazy.

tmux sam nie przekazuje protokołu klawiatury kitty: `extended-keys` sprawia, że
wysyła klawisze z modyfikatorami jako sekwencje `CSI u` lub `xterm`. zellij 0.41 i
nowszy mówi protokołem klawiatury kitty i przekazuje OSC 52, według swojego
dziennika zmian.

## Co się pogarsza i jak

| Czego brakuje | Co dostajesz zamiast tego |
|---|---|
| Protokół obrazów | Graf znakowy. Awatar zamienia się w kolorową plakietkę z inicjałami (jedna litera), niezależnie od `ui.avatar_fallback`: identicon ściśnięty w komórki tekstu jest nie do rozpoznania. Diff obrazów i widok pliku używają kolorowych półbloków albo karty informacyjnej. |
| True color | Kolory są mapowane na paletę 256 lub 16 kolorów (`ui.color_depth`). |
| Protokół klawiatury kitty | Klawisze dawnego typu; część skrótów dociera jako inny klawisz. Ustawienia ostrzegają, a ty zmieniasz skróty albo używasz wariantów z `Alt`. |
| OSC 52 | Akcje kopiowania nic nie wkładają do schowka: zaznacz myszą (z przytrzymanym `Shift`) i użyj kopiowania samego terminala. |
| OSC 22 | Wskaźnik się nie zmienia; separator lub przycisk pod myszą nadal się podświetla. |
| Czcionka emoji | Kwadraty lub dziury zamiast ikon; graf i tekst nadal działają. Zainstaluj Nerd Font albo czcionkę emoji i wskaż ją jako zapasową. |

## Ustawienia, które warto znać

| Ustawienie | Zastosowanie |
|---|---|
| `ui.image_protocol` | `auto` (domyślnie), `kitty`, `iterm2`, `sixel`, `none`. Ustaw `none`, jeśli widzisz śmieci tam, gdzie mają być obrazy. Wymaga restartu. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` jest zawsze poprawne. Wymaga restartu. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` lub `off`, jeśli terminal albo multiplekser zachowuje się źle. Wymaga restartu. |
| `ui.mouse_enabled` | Wyłączone zostawia mysz terminalowi. Wymaga restartu. |
| `ui.launch_terminal` | Terminal, który otwierają skróty systemowe ([Ustawienia](settings.md)). |

Zob. też [Obrazy](image-view.md), [Skróty klawiszowe](keybindings.md) i
[Rozwiązywanie problemów i FAQ](troubleshooting.md).

## Źródła

Sprawdzono w październiku 2026. Źródła pierwotne, chyba że zaznaczono inaczej.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); klawiatura kitty z [macierzy terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (źródło wtórne)
- Windows Terminal: [ogłoszenie 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (wtórne), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 z [opisu zalecenia dotyczącego OSC 52 w Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (wtórne)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (brak true color): [dyskusja Apple](https://discussions.apple.com/thread/255315820) (wtórne)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Serwisy porównawcze (wtórne, tylko do weryfikacji krzyżowej): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Nadal niezweryfikowane: sixel w Konsole i mlterm (z własnych list funkcji projektów, nie sprawdzony ponownie na wydaniu i nie wypróbowany w prawdziwym oknie w żadnym terminalu); OSC 22 w Konsole, iTerm2, Terminal.app i Warp; OSC 52 w
Terminal.app, VS Code i mintty; protokół klawiatury kitty w VS Code, mintty i
ConEmu; każda komórka ConEmu; oraz szerokość emoji w poszczególnych terminalach.
Nie potwierdzono, w którym wydaniu Konsole dostarcza protokół klawiatury kitty.
