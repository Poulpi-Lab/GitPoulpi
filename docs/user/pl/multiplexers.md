[English](../multiplexers.md) · **Polski**

# Multipleksery i panele

Multiplekser stoi między GitPoulpi a Twoim terminalem: sam przerysowuje komórki i
przekazuje dalej tylko te sekwencje sterujące, które rozumie. Dlatego to, co działa w
samym terminalu ([Wybór terminala](terminals.md)), może nie działać w tmux, zellij, screen
ani herdr. Ta strona mówi, co przechodzi, i jaką konfigurację dodać. Wyniki pochodzą z tej
samej listopadowej serii testów 2026 co tabele terminali: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 i
0.46.0, GNU screen 5.0 (macOS dostarcza 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi rozpoznaje multiplekser po `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` lub `tmux*`
oraz `HERDR_*`. W środku obrazy domyślnie są znakami, chyba że tabela poniżej mówi, że
GitPoulpi wykrywa działającą ścieżkę. `GITPOULPI_TERMINAL_IMAGE=off` wyłącza wszystkie
obrazy, a `ui.image_protocol` wymusza protokół.

## W skrócie

| | Obrazy | Kształt wskaźnika (OSC 22) | Schowek (OSC 52) | Klawisze |
|---|---|---|---|---|
| **tmux** | sixel, gdy terminal za nim go obsługuje (wykrywane); obrazy kitty z `allow-passthrough on` i `ui.image_protocol = "kitty"` | z `allow-passthrough on`, w kitty i foot; nie w Ghostty | z `set-clipboard on` | brak klawiatury kitty; `Ctrl+Enter` i `Ctrl+,` nigdy nie docierają |
| **zellij** | znaki | nie | tak | klawiatura kitty działa |
| **GNU screen** | znaki | nie | nie | klawisze dawnego typu |
| **herdr** | obrazy kitty (wykrywane przy `kitty_graphics = true`) | nie | tak | niezmierzone |
| **psmux** (Windows) | znaki (z założenia) | nie (Windows Terminal go nie ma) | schowek systemowy działa | niezmierzone |
| **tmux z MSYS2** (Windows) | znaki (z założenia) | nie | schowek systemowy działa | niezmierzone; ekran jest zniekształcony |
| **Panele WezTerm**, **panele Windows Terminal**, **podziały kitty i Ghostty**, **panele podzielone iTerm2** | jak sam terminal | jak terminal | jak terminal | jak terminal |
| **iTerm2 `tmux -CC`** | znaki | nie | nie (iTerm2 domyślnie to odrzuca) | niezmierzone |

## tmux

**Obrazy.**

- **Sixel** rysuje sam tmux, kodując go ponownie dla klienta, który go obsługuje. GitPoulpi
  pyta tmux raz przy starcie (`tmux display-message -p '#{client_termfeatures}'`,
  z limitem 400 ms) i używa sixela, gdy klient go wymienia. Tak jest za foot,
  WezTerm, Rio i Tabby; nie tak jest za kitty i Ghostty (tmux drukuje wtedy tekst
  „SIXEL IMAGE”). Wymaga tmux zbudowanego z sixelem (ten z Arch i Homebrew taki jest;
  sprawdź przez `tmux -V` i test sixela). Bez ustawienia. Własna odpowiedź tmux na
  zapytanie o atrybuty urządzenia twierdzi, że sixel jest, cokolwiek za nim stoi, dlatego GitPoulpi
  pyta zamiast tego o możliwości klienta.
- **iTerm2 za tmux** zapowiada sixel, ale maluje nieustawione piksele nieprzezroczyście, więc
  w większości przezroczysty graf stałby się pełnym prostokątem. GitPoulpi rozpoznaje iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, typ klienta) i rysuje znaki.
- **Obrazy kitty** przechodzą przez przekazywanie DCS w tmux (kitty, Ghostty; widziane też w
  iTerm2 i Rio w surowej próbie). Potrzebują `set -g allow-passthrough on` (tmux 3.3a lub
  nowszy) **oraz** `ui.image_protocol = "kitty"` w GitPoulpi, który wtedy opakowuje wysyłanie
  i usuwanie. Bez opcji tmux opakowanie jest odrzucane, a graf zostaje pusty.
- Obrazy inline iTerm2 przechodzą przez tmux, ale tmux nie umie ich ustawić w miejscu:
  GitPoulpi nigdy ich tam nie używa.

**Kształt wskaźnika.** GitPoulpi opakowuje `OSC 22` na potrzeby przekazywania. Dociera do okna w
kitty i foot z `allow-passthrough on`. Ghostty nie pokazuje go po opakowaniu (zostaje
strzałka). WezTerm, Rio, Alacritty, Tabby i iTerm2 nie mają `OSC 22` do pokazania.

**Schowek.** `set -g set-clipboard on`. GitPoulpi wysyła też postać opakowaną do przekazywania
(działa z `allow-passthrough on`). Za kitty, Ghostty, Alacritty i Tabby
dociera; iTerm2 odrzuca go, dopóki jego uprawnienie do schowka nie jest włączone.

**Klawisze.** tmux nie odpowiada na zapytanie o klawiaturę kitty, a `extended-keys` nie
przepuszcza `Ctrl+Enter` ani `Ctrl+,`: zmierzone z tmux 3.8 w kitty, oba docierają
jako zwykłe `Enter` i `,` przy `extended-keys` off, on i always. Skróty z literami, takie jak
`Ctrl+Shift+Z`, docierają tylko przy `extended-keys` on lub always **oraz**
`extended-keys-format csi-u` (tmux 3.5 lub nowszy) **oraz** aplikacji, która prosi o
rozszerzone klawisze; GitPoulpi nigdy tego nie robi, więc ustawienie niczego tu nie zmienia. W tmux używaj
wariantów z `Alt`
([Skróty klawiszowe](keybindings.md#klawisze-których-terminal-nie-może-wysłać-i-protokół-klawiatury-kitty)).

**Konfiguracja** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a lub nowszy: kształty wskaźnika, wymuszone obrazy kitty
```

**tmux w WSL.** Niezmierzone.

## zellij

Obrazy: zellij przyjmuje protokół kitty, ale nie jego zastępniki Unicode, i nie przenosi
ani sixela, ani obrazów iTerm2, więc GitPoulpi rysuje znaki (celowe weto). Kształt wskaźnika:
jest połykany, więc GitPoulpi go nie wysyła. Schowek: `OSC 52` jest przekazywany (działa za
kitty, Ghostty, Alacritty, Tabby; iTerm2 i Terminal.app odrzucają go lub ignorują). Klawisze: zellij
odpowiada na zapytanie o klawiaturę kitty, więc pełna klawiatura działa.

zellij 0.46 przy świeżym starcie otwiera okienko „Welcome to Zellij”: naciśnij `Esc`. Sesja, która
się zakończyła, zachowuje nazwę, a następne uruchomienie o tej samej nazwie pyta, co zrobić.
Konfiguracja nie jest potrzebna.

## GNU screen

Nic nie przechodzi: obrazy i `OSC 22` są drukowane jako znaki albo odrzucane, `OSC 52` jest
ignorowany, a klawiatura jest dawnego typu. GitPoulpi rysuje znaki. screen 4.00.03 (wersja
dostarczana z macOS) zniekształca też kolory i szerokie glify w interfejsie; screen 5 był
testowany tylko w Linuksie. Do użytecznej sesji użyj tmux lub zellij.

## herdr

herdr osadza terminal, który ponownie emituje **obrazy kitty**. GitPoulpi ich używa, gdy
`config.toml` herdr mówi `kitty_graphics = true` (w `[terminal]`, a w starszych wersjach w `[experimental]`)
i terminal wokół herdr je rysuje (kitty, Ghostty, WezTerm): wykrywane,
bez ustawienia. Jeśli konfiguracja leży gdzie indziej (na przykład w `HERDR_HOME`, którego GitPoulpi
nie czyta), ustaw `ui.image_protocol = "kitty"`. Przez SSH (`herdr --remote`) GitPoulpi
nie widzi lokalnego klienta, więc rysuje znaki; zob.
[Rozwiązywanie problemów](troubleshooting.md#graf-jest-rysowany-znakami-a-nie-jako-obrazek-herdr-tmux-inne-terminale).
`OSC 22` nie jest przekazywany przez herdr 0.9.5, więc GitPoulpi go nie wysyła. `OSC 52` dociera
za kitty, Ghostty, Alacritty i Tabby. W surowej próbie herdr rysował obrazy kitty w
kitty, Ghostty i iTerm2, także z domyślną konfiguracją. Klawisze: niezmierzone.

## psmux i tmux z MSYS2 (Windows)

- **psmux** (Rust, ConPTY, język poleceń tmux; `winget install psmux`) ustawia `TMUX`, więc
  GitPoulpi rysuje tam znaki z założenia. W Windows Terminal działa normalnie,
  a kopiowanie działa przez schowek systemowy; wskaźnik się nie zmienia (Windows
  Terminal nie ma `OSC 22`). Inne terminale-gospodarze nie były mierzone.
- **tmux z MSYS2 lub Cygwin** też ustawia `TMUX` (znaki z założenia), a w teście
  ekran był zniekształcony: stare klatki rysowane jedna na drugiej w mintty i w Windows Terminal
  (z `-u` i bez; bez niego rysowanie ramek drukowało się jako `qqqq`). Natywny program Windows
  w pseudoterminalu MSYS2 nie jest przez tmux poprawnie przerysowywany. Tego nie da się
  naprawić w GitPoulpi: użyj psmux, paneli Windows Terminal lub paneli WezTerm.
- zellij i screen nie mają wydania na Windows. herdr 0.9.5 został zainstalowany, ale nie uruchomiony.

## Panele terminala

Panele należące do samego terminala nie są dla GitPoulpi multiplekserami:

- **Panele WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domeny SSH): WezTerm rysuje
  wszystko, więc obrazy działają (graf i awatary w panelu), a wskaźnik zachowuje się jak w
  samym WezTerm (kształty działają w buildzie z upstreamowym PR #8092 WezTerm; separator paneli
  daje wskaźnik zmiany rozmiaru). Niezmierzone w Linuksie ani macOS.
- **Panele Windows Terminal** (`wt ; split-pane`): graf i awatary sixel działają w panelu,
  kopiowanie działa, bez kształtów wskaźnika.
- **Podziały kitty, Ghostty i iTerm2**: natywne, jak terminal. Niezmierzone w macOS dla
  kitty i Ghostty (potrzebują GPU).

## iTerm2 `tmux -CC` (tryb sterowania)

iTerm2 otwiera natywne okno dla każdego okna tmux i trzyma okno bramy „[tmux bash]”.
`TMUX` jest ustawione, więc GitPoulpi rysuje znaki w natywnym oknie. Obrazy docierają do iTerm2
przez połączenie sterujące (okno „Allow Terminal-Initiated Display?” pojawiło się
przy teście `OSC 1337`), ale GitPoulpi ich tam nie używa. Ładunek kitty przy
wymuszonym `ui.image_protocol = "kitty"` jest drukowany w oknie bramy, więc zostaw `auto`.
`OSC 22` i `OSC 52` nie docierają.

## Ustawienia, które warto znać

| Ustawienie | Zastosowanie |
|---|---|
| `ui.image_protocol` | `kitty` wymusza obrazy kitty przez multiplekser, który je przekazuje (tmux z `allow-passthrough on`, herdr z konfiguracją, której GitPoulpi nie czyta); `none` dla znaków. |
| `ui.graph_renderer` | `text` jest zawsze poprawne. |
| `ui.kitty_keyboard` | `off`, jeśli multiplekser źle reaguje na zapytanie o klawiaturę. |

Zob. [Wybór terminala](terminals.md) dla terminala za multiplekserem oraz
[Rozwiązywanie problemów](troubleshooting.md), gdy graf jest rysowany znakami.
