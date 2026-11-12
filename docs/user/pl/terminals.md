[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · **Polski** · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Wybór terminala

GitPoulpi działa w każdym terminalu z obsługą myszy, ale wygląda i działa
najlepiej w takim, który rysuje obrazy, pokazuje true color i wysyła pełną
klawiaturę. Ta strona mówi, co każdy terminal daje w każdym systemie, i które
ustawienie zmienić.

**Jak to czytać.** Tabele pochodzą z serii testów z **listopada 2026**: każdy
terminal był sterowany surowymi sekwencjami sterującymi oraz, tam gdzie dało się go
uruchomić, samym GitPoulpi, w Linuksie, Windows 11 i macOS 15. Wersje wypisano
[niżej](#przetestowane-wersje). Każda komórka to jedno z:

- **działa**;
- **częściowo** (i czego brakuje);
- **nie działa** (i dlaczego: *terminal*, *warstwa systemu* albo *GitPoulpi*);
- **n/d** (terminal nie istnieje w tym systemie);
- **niezmierzone** (i dlaczego).

Multipleksery (tmux, zellij, screen, herdr, psmux, panele) mają własną stronę:
[Multipleksery](multiplexers.md).

## Najlepsze wybory

| System | Najlepsze wybory, w kolejności | Dlaczego |
|---|---|---|
| **Windows** | **Windows Terminal**, potem **WezTerm** i **Rio** | Windows Terminal rysuje graf i awatary jako obrazy sixel, zakotwiczone poprawnie, i jest domyślny w launcherze. WezTerm i Rio rysują je jako obrazy iTerm2, zakotwiczone poprawnie od listopadowej poprawki ConPTY 2026 (zob. [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty i Ghostty dają obrazy, kształty wskaźnika i pełną klawiaturę. foot rysuje obrazy sixel i ma pełną klawiaturę oraz kształty wskaźnika. |
| **macOS** | **iTerm2**, potem **kitty** i **Ghostty** | iTerm2 3.7.7 rysuje obrazy przez protokół kitty. Terminal.app działa, ale nie ma ani obrazów, ani true color. kitty i Ghostty zachowują się jak w Linuksie (niezmierzone w macOS). |

W każdym systemie **screen** i **zellij** ukrywają przed GitPoulpi protokół obrazów
terminala, więc program rysuje znaki; tmux przepuszcza sixel. Zob.
[Multipleksery](multiplexers.md).

## Czego używa GitPoulpi

- **Obrazy** (kolumna grafu, awatary autorów, diff obrazów i widok pliku):
  **protokół grafiki kitty** z zastępnikami Unicode, **obrazy inline iTerm2**
  (`OSC 1337`) albo **sixel**. Wykrywanie działa na zasadzie „domyślnie odmawiaj”, na podstawie `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` i zmiennych multiplekserów. Gdy
  środowisko nikogo nie wskazuje (xterm uruchomiony z `-ti vt340`, terminal VS Code
  z włączonymi obrazami, dowolny terminal osiągany przez SSH, gdzie `TERM_PROGRAM` nie jest
  przekazywany), decydują własne odpowiedzi terminala: GitPoulpi pyta raz przy starcie
  (zob. „Zapytanie startowe” niżej) i używa nazwy XTVERSION terminala (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) albo sixela w jego atrybutach urządzenia.
  W multiplekserze odpowiedzi są ignorowane, bo tmux twierdzi, że sixel jest, cokolwiek
  za nim stoi.
- **Który protokół gdzie.** kitty, Ghostty i iTerm2 3.7.7 lub nowszy: protokół
  kitty. WezTerm i Rio: protokół iTerm2, z uwzględnieniem przezroczystości PNG. mintty i
  iTerm2 starszy niż 3.7.7: protokół iTerm2 z nieprzezroczystym tłem. Windows Terminal,
  foot, Tabby, Konsole i mlterm: **sixel**. Cała reszta: znaki, a awatar staje się plakietką
  z inicjałami. `ui.image_protocol` wymusza jeden z nich.
- **iTerm2 3.7.7.** Każdy obraz inline `OSC 1337` otwiera modalne okno „Allow
  Terminal-Initiated Display?”, które blokuje okno, dopóki nie zaznaczysz
  „Remember my choice”. Te same obrazy wysłane protokołem kitty, wraz z zastępnikami
  Unicode, rysują się bez pytania, więc GitPoulpi używa tam protokołu kitty.
  `ui.image_protocol = "iterm2"` nadal daje obrazy inline.
- **Sixel** jest rysowany dla **kolumny grafu** (linie, węzły i, przy
  `ui.avatars = "graph"`, awatary w kółkach commitów) oraz dla **awatarów**
  gdzie indziej (Log, szczegóły commita, Historia pliku, Blame, widok obrazu). Sixel
  nie ma kanału alfa, identyfikatorów ani skalowania, więc obraz ma dokładnie `kolumny x
  szerokość komórki` na `wiersze x wysokość komórki` pikseli, rysowany tylko tam, gdzie coś jest, nad
  pustymi komórkami zachowującymi tło; jest kodowany ponownie tylko przy zmianie linii,
  przewinięcia lub powiększenia. Sixel nie ma operacji usuwania: GitPoulpi zapisuje tekst
  na jego komórkach, żeby go usunąć. Sixel potrzebuje prawdziwego rozmiaru komórki w pikselach: Windows
  Terminal, Tabby i mintty odpowiadają na `CSI 16 t` raz przy starcie i po zmianie powiększenia
  (nigdy nie blokuje, gdy milczą); gdzie indziej bierze się go z rozmiaru okna terminala
  lub z `GITPOULPI_TERMINAL_CELL_PX=10x20`. Terminal, który rysuje
  sixel, nie mówiąc o tym, wymaga `ui.image_protocol = "sixel"`. Windows Terminal
  przed 1.22 nie rysuje sixela; ustaw tam `ui.image_protocol = "none"`.
- **True color**: motywy są w RGB. `ui.color_depth = "auto"` schodzi do 256 kolorów na
  `TERM` typu `*-256color` bez `COLORTERM=truecolor` (oraz w Terminal.app, który nie ma
  true color), a na konsoli Linuksa do 16 kolorów.
- **Mysz**: raportowanie myszy SGR z przyciskami, przeciąganiem i ruchem (reakcja na najechanie, drag
  and drop, prawy przycisk, podwójne kliknięcie).
- **Zapytanie startowe.** Cztery pytania wychodzą jednym zapisem: flagi klawiatury kitty
  (`CSI ? u`), kolor tła (`OSC 11`, używany tylko przy `ui.theme = system`),
  nazwa terminala (XTVERSION, `CSI > 0 q`) i jego atrybuty urządzenia (`CSI c`, ostatnie: każdy
  terminal na nie odpowiada, więc zamyka odczyt). Odczyt to jedna pętla `poll` z limitem 100 ms,
  zakończona, zanim ruszy czytnik klawiatury. Terminal, który nic nie odpowiada,
  kosztuje te 100 ms jednorazowo, a wszystko zostaje tak, jak mówi środowisko. Tylko Unix: w
  Windows konsola sama raportuje klawisze, a decyduje środowisko. Odpowiedzi są w
  raporcie środowiska Dziennika aktywności (wiersz `start-up query:`).
- **Klawiatura**: **protokół klawiatury kitty** (tylko flaga disambiguate) jest żądany przy
  starcie. Dzięki niemu docierają `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` i `Ctrl+Shift+Z`.
  Bez niego zostają klawisze dawnego typu (`Ctrl+Y` ponawia, warianty z `Alt` zastępują część
  skrótów). Zob.
  [Skróty klawiszowe](keybindings.md#klawisze-których-terminal-nie-może-wysłać-i-protokół-klawiatury-kitty).
- **Schowek**: każda akcja kopiowania idzie jedną polityką. Lokalnie to **schowek systemowy**
  maszyny, na której działa GitPoulpi, plus `OSC 52` do terminala, gdy wiadomo, że terminal
  go respektuje (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` jest też rozwiązaniem zapasowym, gdy schowek systemowy zawiedzie. **Przez
  SSH lub bez ekranu** (konsola Linuksa, kontener) wysyłane jest tylko `OSC 52`, a
  w tmux jest wysyłane zarówno opakowane do przekazywania, jak i gołe. Wiersz stanu mówi
  „skopiowano” tylko wtedy, gdy schowek systemowy przyjął tekst; to, co poszło do terminala, mówi
  „wysłano ... do schowka terminala (OSC 52, bez potwierdzenia)”, bo nic nie potwierdza,
  że terminal go usłuchał. Raport środowiska Dziennika aktywności nazywa trasę w swoim wierszu
  `clipboard:`.
- **Kształt wskaźnika myszy** (wskaźnik zmiany rozmiaru nad separatorami, dłoń nad elementami do kliknięcia, I-beam
  nad polami tekstowymi): `OSC 22`, dodatek tam, gdzie terminal go implementuje. GitPoulpi zawsze
  go wysyła (z wyjątkiem konsoli Linuksa oraz za screen, zellij i herdr, które go połykają
  lub drukują; za tmux jest opakowany do przekazywania). Pod `TERM_PROGRAM=mintty`
  wysyła własne nazwy kursorów mintty (`hand`, `ibeam`, `sizewe`, ...), jedyne, które
  mintty czyta. **Rio nie implementuje `OSC 22` w żadnym systemie**: jego kod źródłowy parsuje
  sekwencję i ją odrzuca, więc żadne ustawienie po żadnej stronie tego nie zmienia.
- **Karetka tekstu**: przy `ui.text_cursor = "terminal"` (domyślnie) własny kursor terminala jest
  ustawiany w miejscu karetki aktywnego pola tekstowego, jako migająca pionowa kreska (`DECSCUSR`), i ukrywany
  gdzie indziej; przy wyjściu wraca Twój styl kursora. `drawn` rysuje zamiast tego własną karetkę GitPoulpi.
- **Zdarzenia fokusu** są włączone (podpowiedzi zamykają się, gdy okno traci fokus).
  **Bracketed paste** jest żądany przy starcie i zwalniany przy każdym wyjściu (Unix), więc
  wklejenie wielu wierszy dociera jako jedno wklejenie: jego łamania wierszy nigdy nie zatwierdzają pola, a jego
  litery nigdy nie uruchamiają skrótów. W Windows konsola nie ma nawiasów wklejania; zob.
  [Windows](#windows), co robi zamiast tego GitPoulpi.
  GitPoulpi nie żąda hiperłączy OSC 8 ani falistego podkreślenia.
  Tylko na ścieżce z obrazami każda klatka jest opakowana w zsynchronizowaną aktualizację
  (tryb 2026, ignorowany przez terminale, które go nie znają), żeby obraz nigdy nie migał
  nad wyskakującym oknem.
- **Emoji i symbole**: GitPoulpi używa tylko glifów z domyślną prezentacją emoji
  albo jednokomórkowych kształtów z BMP, bo terminale i czcionki różnie traktują
  selektory wariantów.

## Tabela możliwości

Wiersze to terminale, kolumny to systemy. „GitPoulpi” w komórce oznacza, że uruchomiono tam
samego GitPoulpi; „próba” oznacza, że uruchomiono tylko surowe sekwencje sterujące. Komórki **n/d**: kitty nie ma
natywnego buildu na Windows (wydania są na Linux i macOS); Ghostty nie ma buildu na Windows
(tylko jego biblioteka libghostty jest wieloplatformowa); foot to terminal Wayland (tylko Linux);
xterm to klient X11 (Linux, a w macOS przez XQuartz); iTerm2 i Terminal.app działają tylko
w macOS; Windows Terminal, conhost i mintty tylko w Windows (mintty także pod Cygwin i
MSYS2).

**Niezmierzone w macOS: kitty, Ghostty, WezTerm, Rio i Warp.** Potrzebują GPU
(Metal lub OpenGL 3.3). Ich wyniki z Linuksa są wzorcem tego, co robi ten sam program;
powtórz je na Macu z GPU, zanim zaufasz komórce z macOS.

### Obrazy: który protokół wybiera GitPoulpi i czy się pokazuje

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **działa**: protokół kitty (GitPoulpi) | n/d | niezmierzone (potrzebuje GPU) |
| **Ghostty** | **działa**: protokół kitty (GitPoulpi) | n/d | niezmierzone (potrzebuje GPU) |
| **WezTerm** | **działa**: protokół iTerm2 z przezroczystością (GitPoulpi). Zastępniki rysują się ściśnięte, więc protokół kitty nie jest używany | **działa**: protokół iTerm2 z przezroczystością, graf i awatary we właściwych wierszach (oficjalny 20240203 i build z upstreamowym PR #8092). Oficjalny 20240203 umieszczał obrazy w złych wierszach, zanim GitPoulpi zaczął pokazywać kursor wokół nich, zob. „Limit ConPTY”. | niezmierzone (potrzebuje GPU) |
| **foot** | **działa**: sixel z przezroczystością (GitPoulpi) | n/d | n/d |
| **Rio** | **działa**: protokół iTerm2 z przezroczystością (GitPoulpi) | **działa**: protokół iTerm2 z przezroczystością, co do komórki przy każdym wypróbowanym rozmiarze czcionki (0.5.28; GitPoulpi). Sixel i kitty nigdy nie docierają (ConPTY je odrzuca), więc nie ma czego wybierać | niezmierzone (potrzebuje GPU) |
| **Alacritty** | tekst: **działa** (terminal nie obsługuje obrazów) | tekst: **częściowo** (bez obrazów; diff obrazów jako bloki) | tekst: **działa** |
| **Tabby** | **działa**: sixel z przezroczystością (próba; GitPoulpi nie uruchomiony w Tabby w Linuksie) | **działa**: sixel, wyrównany (GitPoulpi) | **działa**: sixel, przezroczystość zachowana (GitPoulpi) |
| **Warp** | niezmierzone (potrzebuje XWayland i błędnie dekoduje syntetyczne naciśnięcia klawiszy, więc nic nie dało się wpisać) | **nie działa** (terminal: brak obrazów sixel, iTerm2 ani kitty); GitPoulpi niezmierzony (nie dało się go uruchomić z pola wprowadzania Warp) | niezmierzone (potrzebuje GPU) |
| **xterm** | próba: sixel **działa** z `-ti vt340`; GitPoulpi wybiera go z atrybutów urządzenia (zapytanie startowe doszło po teście), albo ustaw `ui.image_protocol = "sixel"` | n/d | tekst: **działa**; sixel się rysuje (próba) i wymaga `ui.image_protocol = "sixel"` |
| **iTerm2** | n/d | n/d | **działa**: protokół kitty w 3.7.7 i nowszych, bez okna (GitPoulpi). Starsze wersje dostają obrazy inline z nieprzezroczystym tłem. Sixel się rysuje, ale nieustawione piksele maluje nieprzezroczyście, więc `ui.image_protocol = "sixel"` daje pełny prostokąt |
| **Terminal.app** | n/d | n/d | tekst: **działa** (bez obrazów) |
| **Windows Terminal** | n/d | **działa**: sixel (GitPoulpi, 1.25) | n/d |
| **conhost** | n/d | tekst: **częściowo** (graf znakowy, diff obrazów z półbloków) | n/d |
| **mintty** | n/d | **nie działa** (terminal za ConPTY: żaden obraz nie jest rysowany, nawet zwykły `OSC 1337` ustawiony przy pokazanym kursorze; kolumna grafu zostaje pusta; tekst i paleta są w porządku) | n/d |

### Kształt wskaźnika myszy (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **działa** | n/d | niezmierzone (potrzebuje GPU) |
| **Ghostty** | **działa** | n/d | niezmierzone (potrzebuje GPU) |
| **WezTerm** | **nie działa** (terminal: niezaimplementowane w testowanym buildzie) | oficjalny: **nie działa** (terminal). Build z upstreamowym PR #8092 WezTerm, który dodaje kształty wskaźnika: **działa** (dłoń, I-beam, strzałki zmiany rozmiaru) | niezmierzone (potrzebuje GPU) |
| **foot** | **działa** | n/d | n/d |
| **Rio** | **nie działa** (terminal: parsowane i odrzucane) | **nie działa** (terminal, ten sam powód; nie ConPTY) | **nie działa** (terminal, to samo źródło; uruchomione tylko w Linuksie i Windows) |
| **Alacritty** | **nie działa** (terminal) | **nie działa** (terminal) | **nie działa** (terminal) |
| **Tabby** | **nie działa** (terminal: xterm.js) | **nie działa** (terminal) | **nie działa** (terminal) |
| **Warp** | niezmierzone | **nie działa** (terminal) | niezmierzone |
| **xterm** | **nie działa** (terminal: nazwy CSS nieznane) | n/d | niezmierzone (wskaźnik był poza oknem) |
| **iTerm2** | n/d | n/d | **nie działa** (terminal) |
| **Terminal.app** | n/d | n/d | **nie działa** (terminal) |
| **Windows Terminal** | n/d | **nie działa** (terminal) | n/d |
| **conhost** | n/d | **nie działa** (terminal) | n/d |
| **mintty** | n/d | **działa** z własnymi nazwami mintty; GitPoulpi je wysyła | n/d |

Tam, gdzie komórka mówi „nie działa”, separator lub przycisk pod myszą nadal się podświetla.

### Protokół klawiatury

GitPoulpi prosi o protokół klawiatury kitty w Linuksie i macOS. W Windows nie
prosi: konsola sama raportuje modyfikatory, a poniższe skróty są ograniczone tym, co
terminal zachowuje dla siebie.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **działa** | n/d | niezmierzone (potrzebuje GPU) |
| **Ghostty** | **działa** | n/d | niezmierzone (potrzebuje GPU) |
| **WezTerm** | **częściowo**: brak odpowiedzi, dopóki nie ma `enable_kitty_keyboard = true` w `wezterm.lua` (domyślnie `false`) | **częściowo**: WezTerm zachowuje `Ctrl+Shift+N` (otwiera swoje okno); `Ctrl+,` nie jest dostarczane (oficjalny build). Build z PR #8092 nie zachowuje żadnego ze skrótów, ale `Ctrl+,` nadal nie otwiera Ustawień (brak odpowiedzi klawiatury kitty przez ConPTY) | niezmierzone (potrzebuje GPU) |
| **foot** | **działa** | n/d | n/d |
| **Rio** | **działa** (domyślnie włączone) | **częściowo**: Rio zachowuje `Ctrl+Shift+N` (otwiera swoje okno) | niezmierzone (potrzebuje GPU) |
| **Alacritty** | **działa** (0.13 i nowszy) | **częściowo**: `Ctrl+Shift+N` dociera jako `Ctrl+N`, `Ctrl+,` nie jest dostarczane | **działa** |
| **Tabby** | **nie działa** (terminal: brak odpowiedzi; zachowuje `Ctrl+Shift+N` i `Ctrl+,` dla siebie) | **częściowo**: Tabby zachowuje `Ctrl+Shift+N` i `Ctrl+,` | **nie działa** (terminal: brak odpowiedzi); część skrótów nie dotarła |
| **Warp** | niezmierzone | niezmierzone | niezmierzone (potrzebuje GPU) |
| **xterm** | **nie działa** (terminal) | n/d | **nie działa** (terminal) |
| **iTerm2** | n/d | n/d | **działa** (3.7.7; sprawdź *Profiles > Keys*, jeśli nie) |
| **Terminal.app** | n/d | n/d | **nie działa** (terminal: brak odpowiedzi; klawisze dawnego typu) |
| **Windows Terminal** | n/d | **częściowo**: WT zachowuje `Ctrl+Shift+N` (nowe okno) i `Ctrl+,` (swoje ustawienia) | n/d |
| **conhost** | n/d | niezmierzone | n/d |
| **mintty** | n/d | **częściowo**: żaden ze skrótów nie miał widocznego skutku | n/d |

Gdy skrót nie dociera, użyj wariantów z `Alt`
([Skróty klawiszowe](keybindings.md#klawisze-których-terminal-nie-może-wysłać-i-protokół-klawiatury-kitty)),
albo przypisz skrót na nowo w *Ustawienia > Klawiatura i mysz*.

### Schowek (akcje kopiowania)

Kopiowanie zawsze używa schowka systemowego, gdy jest lokalny ekran, więc te komórki
opisują ścieżkę `OSC 52` terminala, która ma znaczenie przez SSH, w kontenerach i jako
rozwiązanie zapasowe. Każdy test kopiowania GitPoulpi, który użył schowka systemowego, zadziałał.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **działa** | n/d | niezmierzone (potrzebuje GPU) |
| **Ghostty** | **działa** (widziane za tmux, zellij, herdr); gołe: niezmierzone (zapis do schowka wymaga fokusu klawiatury, którego test mu nie dał) | n/d | niezmierzone (potrzebuje GPU) |
| **WezTerm** | **działa** | **działa** | niezmierzone (potrzebuje GPU) |
| **foot** | niezmierzone (zapis wymaga fokusu klawiatury, którego test mu nie dał) | n/d | n/d |
| **Rio** | niezmierzone (ten sam powód) | **działa** | niezmierzone (potrzebuje GPU) |
| **Alacritty** | niezmierzone (ten sam powód) | **nie działa** (schowek bez zmian przy domyślnej konfiguracji; nie zbadano) | **działa** |
| **Tabby** | **działa** | **działa** | **działa** |
| **Warp** | niezmierzone | **nie działa** (schowek bez zmian) | niezmierzone (potrzebuje GPU) |
| **xterm** | niezmierzone | n/d | **nie działa** (schowek bez zmian) |
| **iTerm2** | n/d | n/d | **nie działa**, dopóki nie włączysz *Settings > General > Selection > Applications in terminal may access clipboard* |
| **Terminal.app** | n/d | n/d | **nie działa** (terminal) |
| **Windows Terminal** | n/d | **działa** (przy fokusie) | n/d |
| **conhost** | n/d | niezmierzone (nie uruchomiono próby) | n/d |
| **mintty** | n/d | **nie działa** (schowek bez zmian przy jego domyślnych ustawieniach) | n/d |

### Wklejanie

W Linuksie i macOS GitPoulpi żąda bracketed paste, więc wklejenie wielu wierszy to jedno wklejenie.
Nie sprawdzano tego terminal po terminalu (**niezmierzone** dla każdego). W Windows konsola
nie ma nawiasów wklejania, więc terminal, który najpierw nie zapyta, dostarcza tekst jako
naciśnięcia klawiszy, a każde łamanie wiersza byłoby `Enter`. Dlatego GitPoulpi traktuje naciśnięcia klawiszy,
które następują po sobie w ciągu 30 ms (wewnętrzny `Enter` albo osiem lub więcej znaków), jako jedno
wklejenie. Przytrzymane klawisze i „abc, `Enter`” wpisane ręcznie nie są wklejeniami.

| Terminal | Windows (wklejenie dwóch wierszy do pola nowej gałęzi) |
|---|---|
| **Windows Terminal** | **działa**: WT najpierw pyta „wkleić kilka wierszy?” (własne ostrzeżenie), potem żadna gałąź nie powstaje |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **działa** (żadna gałąź nie powstaje) |
| **Warp** | niezmierzone (nie dało się uruchomić w nim GitPoulpi) |
| **conhost** | niezmierzone (`Ctrl+V` i `Shift+Insert` wysłane przez test nie wkleiły do klasycznej konsoli) |

### Szerokość emoji i ikon

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **działa** (szerokie komórki wyrównane; kitty rysuje koło zębate U+2699 puste, znana sprawa czcionki) | WezTerm, Rio, Tabby, Windows Terminal: **działa**. Alacritty: **częściowo** (emoji jako jednokolorowe kontury) | Alacritty, Tabby: **działa**. Pozostałe: niezmierzone (potrzebują GPU) |
| **Tabby** | **działa** | **działa** | **działa** |
| **Warp** | niezmierzone | **działa** | niezmierzone |
| **xterm** | **częściowo** (kwadraty: czcionka) | n/d | **częściowo** (kwadraty: czcionka) |
| **iTerm2**, **Terminal.app** | n/d | n/d | **działa** |
| **conhost** | n/d | **częściowo**: kolorowe emoji narysowane, szerokie komórki wyrównane | n/d |
| **mintty** | n/d | **działa** | n/d |

### Przetestowane wersje

kitty 0.49.2 (macOS 0.50.0 się nie uruchomił), Ghostty 1.4.0, WezTerm 20260716 w Linuksie i
20240203 (ostatnie otagowane wydanie) w Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) i
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) i 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminale zmieniają się szybko: wersja nowsza od tych może się różnić.

## Terminal po terminalu

### Windows

- **Windows Terminal.** Domyślny w launcherze i najlepsza obsługa obrazów w Windows:
  graf i awatary sixel, OSC 52, dobre emoji, a także odpowiada na zapytanie o rozmiar komórki. Wybierz
  Nerd Font w *Settings > Profiles > Defaults > Appearance > Font face*. Przytrzymaj `Shift`, by
  zaznaczać tekst, gdy mysz jest przechwycona. Zachowuje `Ctrl+Shift+N` i `Ctrl+,`: użyj
  wariantów z `Alt` albo przypisz na nowo. Kształt wskaźnika się nie zmienia.
- **WezTerm.** W `wezterm.lua` ustaw `config.enable_kitty_keyboard = true` (buildy na Linux i
  macOS) oraz czcionkę z zapasowymi emoji, na przykład
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Obrazy używają protokołu iTerm2 i nic nie trzeba ustawiać: ostatnie otagowane wydanie,
  20240203-110809 (luty 2024), kotwiczy je w prawidłowych wierszach (sprawdzone ponownie w listopadzie 2026 z
  aktualnym GitPoulpi; przed poprawką ConPTY opisaną niżej tak nie było). Upstreamowy PR #8092 WezTerm dodaje kształty wskaźnika.
- **Rio** (MSI, wymaga UAC). Obrazy (graf, awatary, diff obrazów) używają protokołu iTerm2
  z przezroczystością i lądują dokładnie na swoich komórkach: zmierzone w 0.5.28 przy domyślnej czcionce
  oraz przy 18 i 22,5 px (komórki 8x16, 11x21 i 13x26 pikseli, zawsze całe piksele), obraz 20x10 komórek i
  obraz 4x27 komórek wyszły co do komórki. Nic nie trzeba ustawiać. Pseudokonsola
  połyka zapytanie o rozmiar komórki, więc GitPoulpi zakłada, że komórki Rio są dwa razy wyższe niż szerokie
  (prawda przy wszystkich trzech rozmiarach); przy czcionce o innych proporcjach ustaw
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (na przykład `10x24`), by mieć okrągłe znaczniki commitów. Rio
  nie ma kształtów wskaźnika w żadnym systemie.
- **Tabby.** Graf i awatary sixel, kopiowanie i wklejanie działają. Tabby zachowuje `Ctrl+Shift+N` i
  `Ctrl+,` dla siebie.
- **Alacritty.** Wymaga OpenGL 3.3. Graf znakowy; kopiowanie przez `OSC 52` nie było włączone przy domyślnej konfiguracji.
- **mintty** (Git Bash, MSYS2, Cygwin). Kształty wskaźnika działają (własne nazwy mintty). Obrazy
  się nie pokazują (kolumna grafu zostaje pusta; mintty 3.8.3 nie rysuje żadnego obrazu `OSC 1337` za
  ConPTY, nawet ustawionego zwykłą próbą) i nie odpowiada tam na zapytanie o rozmiar komórki: ustaw
  `ui.graph_renderer = "text"`.
- **Warp.** Niezmierzone z GitPoulpi: nie dało się go uruchomić z pola wprowadzania Warp. Terminal
  nie ma obrazów sixel, iTerm2 ani kitty.
- **conhost, cmd, okno PowerShell.** Graf znakowy, diff obrazów z półbloków. Domyślnie
  GitPoulpi uruchomiony z takiego okna otwiera się ponownie w wybranym przez Ciebie terminalu; zob.
  „Reguła ponownego uruchomienia”.

**Limit ConPTY.** Windows uruchamia program każdego terminala za pseudokonsolą
(ConPTY). Windows Terminal i buildy WezTerm, które dołączają nowszą, przepuszczają sekwencje
obrazów; ta wewnątrz Windows (używana przez Rio i mintty) oraz ta dołączona
do WezTerm 20240203 tego nie robią. Co jest tracone: sekwencje DCS (sixel) i APC (grafika kitty)
są odrzucane, więc docierają tylko obrazy iTerm2 (`OSC 1337`), dlatego Rio i
WezTerm ich używają; `CSI 16 t` (rozmiar komórki) odpowiada tylko Windows Terminal i Tabby.
Co naprawiono: `OSC 1337` przechodzi przez ConPTY natychmiast i ląduje w miejscu, gdzie ConPTY ostatnio
namalował kursor, a ConPTY maluje kursor tylko wtedy, gdy jest widoczny. GitPoulpi trzymał kursor
ukryty, więc jego obrazy lądowały w starych wierszach (graf nad stopką, rozdarty ekran po
przewinięciu). Teraz w Windows pokazuje kursor wokół każdego zapisywanego obrazu, w tym samym
zapisie, i znów go ukrywa; Rio i WezTerm 20240203 kotwiczą każdy obraz na jego komórce
(`pty_conpty_picture_anchor` odczytuje ruch kursora za ConPTY). Poza Windows ani dla sixela nic się nie zmienia.

**Wklejanie w Windows.** Zob. tabelę „Wklejanie” wyżej: GitPoulpi zwija serię naciśnięć klawiszy
w jedno wklejenie, więc wklejenie wielu wierszy nigdy nie zatwierdza pola w testowanych terminalach.

**Reguła ponownego uruchomienia.** Dwukrotne kliknięcie pliku exe uruchamia go w klasycznym oknie
konsoli (conhost) o własnym przeznaczeniu. Gdy `ui.launch_terminal` wskazuje inny terminal (domyślnie: Windows
Terminal), GitPoulpi otwiera się tam ponownie, a pierwsze okno się zamyka
(`ui.relaunch_in_terminal`). Robi to **tylko**, gdy jest sam w klasycznym oknie
konsoli. Terminal, który uruchamia exe bezpośrednio (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`), jest tym, w którym chcesz być, i nic nie jest uruchamiane ponownie; z powłoki
nie jest sam, więc też nic się nie dzieje. Wyłącz to przez `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Wszystko działa bez ustawień. `Shift+klik` należy do kitty, więc użyj
  `Alt+klik`, by rozszerzyć zaznaczenie ([Mysz](mouse-and-gestures.md)).
- **Ghostty.** Wszystko działa bez ustawień. Jeśli ikony są dziurami, dodaj emoji
  lub Nerd Font do `font-family` (powtórz klucz, by dodać zapasowe).
- **WezTerm.** Obrazy przez protokół iTerm2, przezroczystość mieszana. Ustaw
  `enable_kitty_keyboard = true` dla pełnych skrótów. Bez kształtów wskaźnika.
- **foot.** Graf i awatary sixel, kształty wskaźnika, pełna klawiatura, szybki. Ustaw
  `font=JetBrainsMono Nerd Font:size=11` w `foot.ini`. Kopiowanie przez `OSC 52` reguluje
  `security.osc52`.
- **Rio.** Obrazy przez protokół iTerm2 z przezroczystością, pełna klawiatura. Bez
  kształtów wskaźnika (Rio odrzuca `OSC 22`).
- **Alacritty.** Graf znakowy, klawiatura kitty od 0.13. Bez obsługi obrazów.
- **Tabby.** Obrazy sixel; bez klawiatury kitty, bez kształtów wskaźnika (Electron, xterm.js). Zachowuje
  `Ctrl+Shift+N` i `Ctrl+,`.
- **xterm.** Uruchom jako `xterm -u8 -ti vt340` dla sixela (UTF-8 jest potrzebne do rysowania ramek).
  Kwadraty zamiast emoji: czcionka.
- **Warp.** Niezmierzone: potrzebuje XWayland i w teście błędnie dekodował syntetyczne naciśnięcia klawiszy.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Nie wchodziły w listopadową serię 2026. Działają
  ze znakami i klawiszami dawnego typu: użyj wariantów z `Alt` i `Ctrl+Y` do ponawiania. VTE
  ignoruje `OSC 52`, więc kopiowanie przez SSH nie trafia do schowka. Metoda wprowadzania
  (IBus, Fcitx) może przechwycić przełącznik stage `Ctrl+Space`; przypisz go na nowo w *Ustawienia >
  Klawiatura i mysz*. Konsole jest rozpoznawana pod kątem sixela z własnej dokumentacji.
- **Wayland, Hyprland, Omarchy.** Wskazówka wskaźnika (`OSC 22`) działa w kitty, Ghostty i
  foot. Wpis launchera Omarchy przechodzi przez `xdg-terminal-exec`.

### macOS

- **iTerm2.** 3.7.7 i nowsze rysują obrazy przez protokół kitty, bez pytania.
  Ustawienia, które pomagają:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    potrzebne do kopiowania przez `OSC 52` (inaczej iTerm2 pokazuje „the terminal attempted to
    access the clipboard but it was denied”).
  - *Profiles > Keys > Left Option key: Esc+*: potrzebne dla skrótów z `Alt`.
  - Jeśli ustawisz `ui.image_protocol = "iterm2"` (lub używasz wersji starszej niż 3.7.7), zaznacz
    „Remember my choice” w oknie „Allow Terminal-Initiated Display?”. Obrazy
    mają wtedy nieprzezroczyste tło (widoczny prostokąt, jeśli Twoje tło różni się od
    tła motywu; `ui.graph_renderer = "text"` go usuwa).
  - Protokół klawiatury kitty jest domyślnie włączony (3.6.9 i nowsze).
- **Terminal.app.** Tylko 256 kolorów: GitPoulpi sam wybiera `ui.color_depth = 256`, a
  motywy wyglądają płasko. Bez obrazów, bez klawiatury kitty, bez schowka przez `OSC 52`. Zaznacz
  *Use Option as Meta key* dla skrótów z `Alt`.
- **Alacritty, Tabby.** Alacritty: graf znakowy, klawiatura kitty, kopiowanie `OSC 52`; ustaw
  `option_as_alt = "Both"` dla skrótów z `Alt`. Tabby: graf i awatary sixel.
- **Option jako Meta.** Option składa znaki (Option+B wpisuje znak całki), chyba że
  terminal zamieni go w Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. Skróty z `Alt` w GitPoulpi tego wymagają i nigdzie nie jest to
  domyślnie włączone.
- **kitty, Ghostty, WezTerm, Rio, Warp**: niezmierzone w macOS (potrzebują GPU). Spodziewaj się
  zachowania z Linuksa.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel wymaga `ui.image_protocol = "sixel"`.

### VS Code i Warp

Terminal VS Code (`TERM_PROGRAM=vscode`) nie wchodził w serię testów. Jest rozpoznawany
po atrybutach urządzenia, gdy `terminal.integrated.enableImages` jest włączone, w przeciwnym razie GitPoulpi rysuje
znaki; `ui.image_protocol = "sixel"` wymusza obrazy. Warp: zob. uwagi per system wyżej.

## Multipleksery

tmux, zellij, screen, herdr, psmux i własne panele terminali stoją między GitPoulpi a
terminalem i zmieniają to, co przechodzi. Krótko: tmux przepuszcza sixel i, z
`allow-passthrough on`, kształty wskaźnika i obrazy kitty; zellij i screen nie przepuszczają
ani obrazów, ani kształtów wskaźnika; herdr przepuszcza obrazy kitty. Wszystkie szczegóły i
konfiguracja do dodania są w [Multipleksery](multiplexers.md).

## Co się pogarsza i jak

| Czego brakuje | Co dostajesz zamiast tego |
|---|---|
| Protokół obrazów | Graf znakowy. Awatar staje się kolorową plakietką z inicjałem (jedna litera), cokolwiek mówi `ui.avatar_fallback`: identicon ściśnięty w komórki tekstu jest nie do rozpoznania. Diff obrazów i widok pliku używają kolorowych półbloków albo karty informacyjnej. |
| Truecolor | Kolory są mapowane na paletę 256 lub 16 kolorów (`ui.color_depth`). |
| Protokół klawiatury kitty | Klawisze dawnego typu; część skrótów dociera jako inny klawisz. Ustawienia ostrzegają, a Ty przypisujesz na nowo albo używasz wariantów z `Alt`. |
| OSC 52 | Lokalnie nic się nie zmienia (używany jest schowek systemowy). Przez SSH lub bez ekranu akcje kopiowania nie wkładają nic do schowka: zaznacz myszą (przytrzymaj `Shift`) i użyj własnego kopiowania terminala. |
| OSC 22 | Wskaźnik się nie zmienia; separator lub przycisk pod myszą nadal się podświetla. |
| Czcionka emoji | Kwadraty lub dziury zamiast ikon; graf i tekst nadal działają. Zainstaluj Nerd Font lub czcionkę emoji i wpisz ją jako zapasową. |

## Ustawienia, które warto znać

| Ustawienie | Zastosowanie |
|---|---|
| `ui.image_protocol` | `auto` (domyślnie), `kitty`, `iterm2`, `sixel`, `none`. Ustaw `none`, jeśli widzisz śmieci tam, gdzie powinny być obrazy. Restart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` jest zawsze poprawne (użyj go tam, gdzie obowiązuje limit ConPTY). Restart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` lub `off`, jeśli terminal albo multiplekser źle działa. Restart. |
| `ui.text_cursor` | `terminal` (domyślnie) lub `drawn`. |
| `ui.mouse_enabled` | Wyłączone zostawia mysz terminalowi. Restart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Terminal otwierany przez skróty systemu i to, czy dwukrotne kliknięcie otwiera tam GitPoulpi ponownie ([Ustawienia](settings.md)). |

Zob. też [Multipleksery](multiplexers.md), [Obrazy](image-view.md),
[Skróty klawiszowe](keybindings.md) i [Rozwiązywanie problemów](troubleshooting.md).

## Źródła

Dokumentację sprawdzono w październiku 2026, zachowanie przetestowano w listopadzie 2026. Źródła pierwotne, chyba że zaznaczono inaczej.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); klawiatura kitty z [macierzy terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (wtórne)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (wtórne), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (brak wiersza `OSC 22`), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 z [opisu poradnika Warp OSC 52](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (wtórne)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (brak truecolor): [dyskusja Apple](https://discussions.apple.com/thread/255315820) (wtórne)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Strony porównawcze (wtórne, używane tylko do weryfikacji krzyżowej): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
