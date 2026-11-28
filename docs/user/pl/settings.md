[English](../settings.md) · [Čeština](../cs/settings.md) · [Deutsch](../de/settings.md) · [Español](../es/settings.md) · [Français](../fr/settings.md) · [Italiano](../it/settings.md) · [日本語](../ja/settings.md) · [한국어](../ko/settings.md) · [Nederlands](../nl/settings.md) · **Polski** · [Português (Brasil)](../pt-BR/settings.md) · [Português (Portugal)](../pt-PT/settings.md) · [Русский](../ru/settings.md) · [Svenska](../sv/settings.md) · [Türkçe](../tr/settings.md) · [Українська](../uk/settings.md) · [简体中文](../zh-CN/settings.md)

# Ustawienia

Ustawienia otwiera się skrótem `Ctrl+,`, sekwencją `Ctrl+N`, a potem `,`,
kołem zębatym na pasku tytułu (na lewo od `×`) albo przez `Ctrl+P` > *Ustawienia*.
Ekran Ustawień zastępuje okno na czas, gdy jest otwarty. Zamykasz go przez `✕` w prawym
górnym rogu, `Esc` lub `q`. Pyta wcześniej tylko wtedy, gdy zamknięcie coś by zgubiło (zob.
[Zamykanie Ustawień](#zamykanie-ustawień)).

Zmiany są od razu zapisywane: ustawienia interfejsu (motyw, układ, kolumny, wyświetlanie
diffów i obrazów, skróty...) w `tui.toml`, wszystko inne w `config.toml`, oba w twoim
folderze konfiguracji ([dlaczego dwa pliki](config-reference.md#dwa-pliki-configtoml-i-tuitoml)).
[Opis konfiguracji](config-reference.md)
podaje każdy klucz, jego wartość domyślną i zmienną środowiskową. Ta strona
opisuje sam ekran Ustawień.

![Ekran Ustawień: pole wyszukiwania i lista stron po lewej, strona Ogólne po prawej](../images/settings.png)

## Obsługa ekranu Ustawień

| Cel | Działanie |
|---|---|
| Przejście między listą stron a opcjami | `Tab` / `Shift+Tab` albo kliknięcie. |
| Poruszanie się | `j` / `k` lub klawisze strzałek. |
| Zmiana opcji | `Enter` lub `Spacja` albo kliknięcie jej wartości. Zob. niżej. |
| Zamknięcie | Kliknij `✕` albo naciśnij `Esc` lub `q`. Gdy otwarte jest pole, lista rozwijana lub potwierdzenie, `Esc` zamyka tylko to. |

Co robi `Enter` lub `Spacja`, zależy od wiersza:

- Pole wyboru zmienia stan.
- Lista rozwijana otwiera się. Strzałki wybierają, `Enter` zatwierdza.
- Pole tekstowe lub liczbowe zaczyna edycję. Działają strzałki, `Home`, `End`,
  `Backspace` i `Delete`, a kliknięcie ustawia kursor. `Enter`, `Tab`,
  `Up`, `Down` lub kliknięcie gdzie indziej zapisuje wartość. `Esc` anuluje.
  Odrzucona wartość zostaje w polu na czerwono, a powód pokazuje pasek stanu.
- Wiersz z przyciskami (**Utwórz**, **Usuń**, **Testuj**...) pokazuje tylko te, które mają
  teraz zastosowanie. `Left` / `Right` wybierają jeden, `Enter` go naciska, albo kliknij.
  Przycisk, który coś usuwa, najpierw pyta.

Na każdej stronie wartości zaczynają się w jednej kolumnie, więc etykiety wyrównują się
z lewej, a pola z prawej. Tekst lub liczba to przyciemniony pasek, wybór to pasek
zakończony `▾`, pole wyboru to `[x]`. Długa etykieta zawija się zamiast być ucinana,
a w wąskim oknie wartość przechodzi pod etykietę.

Wyszarzony wiersz podaje powód, dla którego jest nieaktywny. Na przykład
**Intensywność podbarwienia** jest wyszarzona, gdy **Tło ścieżek grafu** ma wartość
**Wył.**.

### Listy

Strony edytujące kilka elementów tego samego rodzaju (Profile, AI, Przepływ pracy,
Otwórz w, Obrazy) pokazują je jako **listę po lewej i pola zaznaczonego elementu po
prawej**. W wąskim oknie lista jest nad polami. Wszędzie ma ten sam kształt:

- Znacznik wiersza mówi o jego stanie: `●` aktywny profil, `★` wpis domyślny, `⚠`
  niedokończony element (profil bez e-maila), z powodem w miejscu szczegółów.
- Pod listą: **+ Dodaj**, **Duplikuj**, **Usuń**, `↑` i `↓`. Przycisk, który teraz nie ma
  zastosowania (`↑` na pierwszym wierszu), jest przyciemniony. **+ Dodaj ▾** otwiera
  listę rozwijaną, gdy strona oferuje ustawienia wstępne lub rodzaje.
- **Usuń** najpierw pyta, z wyjątkiem elementu dodanego przed chwilą i niezmienianego.
- Przeciągnij wiersz, by go przesunąć; pasek pokazuje, gdzie wyląduje, a `Esc` anuluje
  przeciąganie. Dwukrotne kliknięcie przechodzi do pól elementu.

| Klawisz | Działanie |
|---|---|
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Zaznacza element. |
| `Insert` | Dodaje element (otwiera listę rozwijaną, jeśli jest). |
| `Ctrl+D` | Duplikuje element. |
| `Delete` | Usuwa element, po potwierdzeniu. |
| `Alt+Up` / `Alt+Down` | Przesuwa element w górę lub w dół. |
| `Enter` lub `Right` | Przechodzi do pól elementu. |
| `Left` | Wraca do listy stron. |
| `Alt+Left` / `Alt+Right` | Przełącza między kartą Ogólne a kartą listy (w dowolnym miejscu strony). |

Każda z tych stron ma pod tytułem dwie **podkarty**, aby ustawienie strony nigdy nie było mylone z właściwością zaznaczonego elementu: **Ogólne**, własne ustawienia strony (aktywny profil, domyślny wpis, instrukcje, które dostaje każda akcja AI, szablon przepływu pracy...), oraz kartę nazwaną od elementów (**Profile**, **Profile (połączenia)**, **Wpisy**, **Edytory**, **Reguły**), która zawiera listę i pola zaznaczonego elementu. Strona otwiera się na karcie **Ogólne**; ustawienia pamiętają kartę, na której opuściłeś każdą stronę, aż do zamknięcia aplikacji, a także pozycję przewinięcia i zaznaczony wiersz każdej strony (i karty). Kliknij kartę lub naciśnij `Alt+Left` / `Alt+Right`. Wyszukiwarka, wiersze ustawień w palecie poleceń i odnośniki „ustaw w ustawieniach” otwierają kartę, na której leży ustawienie.
Strona [Klawiatura i mysz](#klawiatura-i-mysz) ma te same dwie karty (**Ogólne**, **Skróty klawiszowe**).

Klawisze można zmienić w **Klawiatura i mysz** (kontekst *preferences*, zob.
[Skróty klawiszowe](keybindings.md)).

### Edytory wielowierszowe

Ustawienia zawierające wiele wierszy (na przykład ścieżki częściowego checkoutu, opis szablonu commita, gałęzie bazowe zapobiegania konfliktom i hooki przepływu pracy) otwierają się w małym edytorze klawiszem `Enter` lub kliknięciem; `Ctrl+S` zachowuje tekst, a `Esc` go porzuca. Mysz działa tam jak w każdym polu tekstowym:

- Kliknięcie ustawia kursor w klikniętym miejscu.
- Przeciągnięcie zaznacza tekst; kółko przewija.
- Dwukrotne kliknięcie zaznacza słowo pod wskaźnikiem.
- `Shift` + kliknięcie rozszerza zaznaczenie od kursora do wskaźnika.

Klawiatura oferuje to samo: `Shift` ze strzałkami zaznacza, `Ctrl` ze strzałkami przeskakuje o słowo, `Ctrl+A` zaznacza wszystko, `Ctrl+C` i `Ctrl+X` kopiują i wycinają.

### Zamykanie Ustawień

`✕`, `Esc` i `q` działają tak samo. Ustawienia stosują każdą zmianę od razu, więc zwykle
po prostu się zamykają. Pytanie **Zamknąć Ustawienia?** pojawia się tylko wtedy, gdy coś
by przepadło:

- strona **Przepływ pracy** ma niezapisane zmiany;
- szablon commita ma niezapisane zmiany;
- profil nie ma e-maila autora (commity, które tworzy, nie dałoby się przypisać do ciebie).

Okno oferuje **Przejdź tam** (otwiera tę stronę), **Odrzuć i zamknij** lub **Zamknij mimo
to** oraz **Anuluj**. Małe edytory wewnątrz Ustawień (tekst instrukcji AI, hook
przepływu pracy, ścieżki częściowego checkoutu) mają własne `✕` i pytają **Odrzucić
zmiany?** tylko wtedy, gdy ich tekst edytowano.

### Okna nad Ustawieniami

Wiersz tytułu pokazuje ośmiornicę i nazwę aplikacji, tak jak okno główne. Kliknij
ośmiornicę, aby otworzyć okno O programie.

Kilka okien działających w całym oknie otwiera się nad Ustawieniami i oddaje je
w stanie, w jakim były (strona, pozycja przewijania i niezapisane zmiany Przepływu
pracy lub szablonów zostają):

- paleta poleceń (`Ctrl+P`),
- wybór motywu (`Ctrl+N t`, **Aplikacja: Zmień motyw...**), który pokazuje każdy
  motyw podglądem na samych Ustawieniach; `Esc` przywraca poprzedni motyw,
- wybór języka i kolorów grafu (`Ctrl+N`, potem te same litery co w oknie głównym),
- wyszukiwanie w obszarze roboczym (`Ctrl+Alt+F`),
- O programie (`Ctrl+N a`, ośmiornica albo **Pomoc: O programie GitPoulpi**) oraz
  nakładka pomocy (`?`).

Każde zamkniesz przez `Esc`, jego własne `✕` albo kliknięcie poza nim. Polecenia,
które działają na to, co zasłaniają Ustawienia (karta, gałąź, diff...), są
wymienione w palecie, ale wyszarzone z napisem „najpierw zamknij Ustawienia”, a
skróty `Ctrl+N` dla nich pokazują tę linię na dolnej krawędzi Ustawień: zamknij
Ustawienia i dopiero uruchom je.

`Ctrl+N` pokazuje plakietkę na każdym elemencie strony: polu wyszukiwania, liście stron (jedna plakietka, na bieżącej stronie), podkartach, liście i jej przyciskach, każdym polu widocznym na ekranie oraz krzyżyku zamykania. Naciśnij znak plakietki, a stanie się to samo co po kliknięciu tego elementu: pole tekstowe otworzy się do edycji, pole wyboru zostanie przełączone, lista rozwijana się otworzy, przycisk zostanie wykonany. `Esc` anuluje. Pole przewinięte poza ekran nie ma plakietki. Gdy piszesz w polu albo otwarta jest lista rozwijana, edytor lub wyszukiwanie, `Ctrl+N` należy do nich. Zobacz [Plakietki](keybindings.md#plakietki).

### Pole wyszukiwania

Pole na górze listy stron przeszukuje nazwę i opis każdej opcji na każdej
stronie. Kliknij je albo naciśnij `/` lub `Ctrl+F` i zacznij pisać. Rozwija się
lista trafień. Każde trafienie pokazuje opcję i jej stronę. `Up` / `Down`
wybierają, `Enter` przechodzi do opcji, `Esc` zamyka listę.

![Wyszukiwanie „diff” w Ustawieniach: pasujące opcje ze stron AI oraz Diff i edytor](../images/settings-search.png)

## Ustawienia z palety poleceń

Większość ustawień można zmienić bez otwierania tego ekranu: wpisz `Ustawienia:` lub `Settings:` w palecie poleceń (`Ctrl+P`). Wiersze są generowane z pól samych stron, więc nowe ustawienie pojawia się w obu miejscach.

- Pole wyboru to **Ustawienia: przełącz ...**. Przełącza się od razu, a wiersz pokazuje `[x]` lub `[ ]`.
- Lista rozwijana to **Ustawienia: ...**. `Enter` zawęża paletę do jej wartości, bieżąca jest oznaczona `bieżące`; `Enter` na wartości ją wybiera.
- Liczba lub tekst otwiera pole z bieżącą wartością. Wartość, którą odrzuciłaby strona (nie liczba, poza zakresem), jest odrzucana tak samo, a pole pozostaje otwarte.
- Ustawienie wymagające własnej strony (lista, combo z opcjami ustalanymi w trakcie działania) otwiera Ustawienia w tym miejscu.

![Paleta poleceń przefiltrowana na Settings: z jednym wierszem na ustawienie i jego bieżącą wartością po prawej](../images/palette-settings.png)

Wartość jest zapisywana do swojego pliku (`tui.toml` lub `config.toml`) i stosowana dokładnie tak jak na stronie. Angielskie słowa klucza ustawienia (`remember tabs`) znajdują wiersz w każdym języku.

## Strony

Lista stron ma dwie grupy:

- **Ustawienia**: ustawienia globalne.
- **Ustawienia repozytorium**: ustawienia repozytorium aktywnej karty.
  Są widoczne tylko przy otwartym repozytorium.

### Ogólne

![Lista rozwijana Język na stronie Ogólne: Auto i siedemnaście języków, z polem wyszukiwania](../images/settings-language.png)

| Opcja | Działanie |
|---|---|
| **Język** | Język interfejsu: **Auto** (język systemu, w przeciwnym razie angielski), English, Français, Español, Português (Brasil), Português (Portugal), Italiano, Deutsch, 简体中文, 日本語, 한국어, Русский, Українська, Polski, Čeština, Nederlands, Svenska, Türkçe. Zmiana działa od razu, bez restartu. Nazwy miesięcy i daty w Logu podążają za nią. Zmienna środowiskowa `GITPOULPI_LANG` (na przykład `GITPOULPI_LANG=fr`) nadpisuje ten wybór. Wyjście CLI i MCP pozostaje po angielsku. |
| **Tłumacz terminy git** | Domyślnie wyłączone: commit, push, pull, stash, tag i pozostałe terminy git zostają po angielsku w każdym języku. Włączone: używa się własnego słownictwa gita danego języka (polski: *Ściągnij*, *Wypchnij*, *Schowek*, *Znacznik*). Działa od razu. |
| **Angielskie słowa git** | Tylko dla japońskiego, koreańskiego i chińskiego: pokazuje *stage*, *unstage*, *staged* jako angielskie słowa zamiast transkrypcji. Bez efektu, gdy włączone jest **Tłumacz terminy git**. |
| **Pamiętaj karty** | Przy starcie otwiera ponownie karty z ostatniej sesji. Wyłączone: start w repozytorium otwiera tylko to repozytorium, a start poza repozytorium zaczyna na Home bez karty repozytorium. Wymaga restartu. |
| **Włącz Radar** | Domyślnie włączone (`general.radar_enabled`). Wyłączone ukrywa kartę Radaru, jego polecenia palety i `Alt+R`, a w tle nic dla niego nie jest pobierane. Działa od razu. Zob. [Radar](radar.md#zamykanie-karty-lub-wyłączenie-radaru). Niewidoczne w kompilacji bez Radaru. |
| **Karta startowa w repozytorium** | Karta, na której otwiera się start, gdy folder, z którego go uruchomiono, leży w repozytorium (`general.startup_tab_in_repo`): **Bieżące repozytorium** (domyślnie, karta tego repozytorium, w razie potrzeby otwierana), **Ostatnia karta** (karta aktywna przy ostatnim zamknięciu), **Home**, **Radar**. Repozytorium podane przez `--repo` zawsze otwiera własną kartę. **Ostatnia karta** to Home, gdy **Pamiętaj karty** jest wyłączone lub nic nie zapamiętano; **Radar** to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Czytane raz przy starcie; wymaga restartu. |
| **Karta startowa poza repozytorium** | Karta, na której otwiera się start, gdy folder, z którego go uruchomiono, nie jest repozytorium (na przykład folder użytkownika) (`general.startup_tab_outside_repo`): **Home** (domyślnie), **Ostatnia karta**, **Radar**. **Ostatnia karta** to Home, gdy **Pamiętaj karty** jest wyłączone lub nic nie zapamiętano; **Radar** to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Czytane raz przy starcie; wymaga restartu. |
| **Karta startowa przy uruchomieniu z systemu** | Karta, na której otwiera się start, gdy GitPoulpi uruchomiono ze skrótu na pulpicie lub w menu Start, który sam utworzył (przekazuje `--launcher`), niezależnie od folderu (`general.startup_tab_launcher`): **Ostatnia karta** (domyślnie), **Home**, **Radar**. Ma pierwszeństwo przed dwoma ustawieniami powyżej. **Ostatnia karta** to Home, gdy **Pamiętaj karty** jest wyłączone lub nic nie zapamiętano; **Radar** to Home, gdy Radar jest wyłączony lub nie ma go w buildzie. Skróty utworzone przez starszą wersję dostają ten przełącznik przy następnym starcie. Czytane raz przy starcie; wymaga restartu. W systemie Windows kopia przypięta do paska zadań również jest zapisywana na nowo przy następnym starcie, a start bez otaczającej powłoki (podwójne kliknięcie, przypięty sam plik exe) liczy się jako uruchomienie z systemu. |
| **Grupuj karty worktree** | Domyślnie włączone (`general.group_worktree_tabs`). Karty powiązanych worktree repozytorium stoją zaraz za kartą repozytorium, a odciągnięta karta worktree wraca na to miejsce. Wyłączone zostawia karty tam, gdzie je ułożysz. Włączenie grupuje otwarte karty od nowa. |
| **Domyślny folder klonowania** | Folder, od którego zaczyna się okno Klonuj. Puste: `%USERPROFILE%\source\repos` w Windows, `~/src` gdzie indziej. |
| **Okno Klonuj zaczyna od** | Źródło, od którego otwiera się okno Klonuj (`repos.clone_start_source`): **Ostatnio użyte** (domyślnie, źródło pokazywane przy ostatnim zamknięciu, przez sklonowanie lub anulowanie), **Klonuj z URL** albo jedna z połączonych integracji. Źródło, które nie jest już dostępne, wraca do Klonuj z URL. Klon uruchomiony z Radaru zachowuje własne źródło. |
| **Domyślny folder przeglądania** | Folder, w którym otwiera się Przeglądaj. Puste: ostatnio użyty folder, w przeciwnym razie katalog domowy. |
| **Lokalizacja danych repozytorium** | Gdzie GitPoulpi przechowuje dane poszczególnych repozytoriów (`session.per_repo_state_dir`): **Wewnątrz .git** (domyślnie: nic nie pojawia się w drzewie roboczym, wszystkie worktree dzielą dane), **W drzewie roboczym** (folder `.gitpoulpi`; potrzebne, by zatwierdzić zespołowy `workflow.toml`) albo **W moim folderze danych użytkownika** (poza repozytorium). Folder `.gitpoulpi` zostawiony przez starszą wersję jest przenoszony na nowe miejsce przy pierwszej potrzebie i usuwany, gdy zostanie pusty; plik śledzony przez git (zatwierdzony `workflow.toml`) nigdy nie jest przenoszony ani usuwany i pozostaje czytelny. |
| **Domyślna nazwa gałęzi** | Pierwsza gałąź repozytorium tworzonego przez Init. Puste: decyduje git. Nie zmienia nazw istniejących gałęzi. |
| **Poziom dziennika**, **Przechowuj pliki dziennika**, **Maksymalny rozmiar pliku dziennika** | Dziennik aplikacji za Dziennikiem aktywności: najbardziej szczegółowy zapisywany poziom (`log.level`), jak długo pliki są przechowywane (liczba całkowita i jednostka, dni, tygodnie, miesiące lub lata, w jednym wierszu: kliknij liczbę, by ją wpisać, Up i Down zmieniają ją o krok, Tab przechodzi do jednostki, a Enter otwiera jej listę; najwyżej od 1 do 10 lat, domyślnie 1 tydzień, `log.retention_days` i `log.retention_unit`) oraz rozmiar, przy którym plik z jednego dnia jest rotowany (liczba i MiB lub GiB, od 1 MiB do 1 GiB, `log.max_file_mb` i `log.max_file_unit`). |
| **Pokaż czas trwania operacji**, **Rozszerzone logowanie** | Karta Operacje git Dziennika aktywności: czas trwania każdej operacji (`log.timings`, domyślnie włączone) oraz praca w tle, odświeżanie i auto-fetch (`log.extended`, domyślnie wyłączone). Zob. [Gdy coś się nie powiedzie](troubleshooting.md#dziennik-aktywności). |
| **Cofanie włączone**, **Głębokość historii cofania**, **Zapisuj zmiany wykonane poza GitPoulpi** | Grupa **Historia cofania**. Włącza lub wyłącza Cofanie i ustawia, ile operacji zachowuje (domyślnie 100, `0` oznacza bez limitu). Wyłączenie Cofania kasuje historię cofania, po potwierdzeniu. Ostatnia opcja zapisuje też to, co powłoka lub agent robi z gałęziami, tagami, stashami i HEAD. Zob. [Cofnij i ponów](undo-redo.md). |

Polecenie palety **Aplikacja: Zmień język...** (`Ctrl+N l`) otwiera tę samą
listę języków. Wyróżniony język jest tam podglądany na żywo; `Enter` go zachowuje,
a `Esc` przywraca poprzedni.

### Uruchamianie z systemu

Strona **Uruchamianie z systemu** (`gitpoulpi ui show settings launcher`) tworzy skrót, który uruchamia GitPoulpi
z menu aplikacji systemu, w wybranym terminalu.

**Ta instalacja** (pierwsza grupa strony). Każde GitPoulpi na komputerze (repozytorium deweloperskie, pakiet, kopia przenośna) ma własną nazwę programu uruchamiającego, kolor, aliasy i wpis menu folderów, zapisane we własnej tabeli `[installation.settings.<id>]` pliku `config.toml`; zmiana tutaj nigdy nie dotyka innej instalacji. Nic z tego nie trafia do dziennika cofania (to integracja z systemem; drogą powrotu jest **Usuń**).

| Wiersz | Co robi |
|---|---|
| **Instalacja** | Tylko do odczytu: rodzaj, folder i środowisko tej instalacji. Na wąskiej stronie folder jest skracany w środku wielokropkiem; pełny tekst jest w wierszu pomocy pod pozycją. |
| **Nazwa programu uruchamiającego** | Nazwa skrótów tej instalacji i jej wpisu w menu folderów. Odrzucana, gdy ma ją inna instalacja, która nadal istnieje. Zmiana nazwy zmienia nazwy skrótów, które ta instalacja już ma (menu Start, Pulpit, wpis w menu aplikacji), i w miejscu zmienia etykietę jej wpisu w menu folderów; żaden nowy nie powstaje. Instalacja główna zawsze nazywa się `GitPoulpi`. |
| **Dodaj wersję do nazwy programu uruchamiającego** | Dodaje `<główna>.<podrzędna>` do nazwy (`GitPoulpi 1.2`); domyślnie wyłączone, bo skróty są zmieniane przy każdej aktualizacji. |
| **Kolor ośmiornicy** | Kolor ośmiornicy tej instalacji i jej skrótów (te same wybory co w oknie O programie). |
| **Zostań instalacją główną** | Instalacja główna ma nazwę `GitPoulpi` bez dopisków, starsze skróty i klucze `ui.*` odczytywane przez starsze wersje; domyślnie nikt nią nie jest. Pyta najpierw, gdy inna instalacja już jest główna. Poprzednia instalacja główna zachowuje skróty pod własną nazwą (Windows i Linux). Zmiana roli (zostanie instalacją główną albo rezygnacja) zmienia nazwy skrótów tej instalacji w miejscu, więc nigdy nie ma chwili bez skrótu, i przenosi jej wpis w menedżerze plików pod nową nazwę. |
| **Środowisko** | Instalacje jednego środowiska współdzielą okna, demony i historię cofania; domyślnie jest to środowisko własne instalacji. Zmiana wymaga ponownego uruchomienia otwartych okien tej instalacji. |
| **Podmenu menu folderów** | Grupuje wpis menu folderów w podmenu o tej nazwie; instalacje o tej samej nazwie współdzielą jedno. Niedostępne w macOS; Nemo i Thunar zawsze pokazują wpis na najwyższym poziomie. |
| **Inna: \<nazwa\>** | Pozostałe instalacje na tym komputerze. Ta, której folder zniknął, ma przycisk **Zapomnij** (pyta najpierw), który usuwa jej wpis oraz skróty, aliasy i wpisy menu folderów, które po niej zostały. |
| **Starsze skróty** | Dopóki nikt nie jest instalacją główną, skróty utworzone przez starszą wersję pod nazwą bez dopisków są wymienione z przyciskami **Zostań instalacją główną** i **Usuń starsze skróty** (pyta najpierw). Jednorazowa uwaga w wierszu stanu wskazuje to miejsce. |

Paleta ma **Uruchamianie: Zostań instalacją główną**, **Uruchamianie: Usuń starsze skróty** i **Uruchamianie: Zapomnij usunięte instalacje**, a każdy wiersz jest wpisem `Ustawienia:`. Wiersze aliasów poniżej należą do tej instalacji (dla repozytorium deweloperskiego sugerowane są `gpd` i `gpda`); alias utworzony przez inną instalację ma opis **Używany przez: \<nazwa\>** i nie można go tu utworzyć.

| Wiersz | Co robi |
|---|---|
| **Terminal dla skrótów** | `ui.launch_terminal`: **Automatycznie**, **Domyślny systemu** lub jeden z terminali znalezionych na tym komputerze (to samo wykrywanie co w **Otwórz w**). Automatycznie: Windows Terminal w Windows (w przeciwnym razie WezTerm, Alacritty, zwykła konsola), `xdg-terminal-exec` w Linuksie (Omarchy; w przeciwnym razie pierwszy znaleziony terminal), Terminal w macOS, chyba że zainstalowano iTerm2, Ghostty, WezTerm lub kitty. |
| **Otwórz ponownie w tym terminalu po dwukrotnym kliknięciu** | Tylko Windows (`ui.relaunch_in_terminal`). Dwukrotne kliknięcie pliku exe otwiera domyślną konsolę, która źle rysuje obrazy: GitPoulpi otwiera się ponownie w wybranym terminalu, a pierwsze okno się zamyka. Nie dzieje się tak, gdy uruchomiono go z terminala. |
| **Pokaż otwarte okno po kliknięciu** | `ui.reuse_window` (domyślnie włączone). Kliknięcie skrótu GitPoulpi (`gitpoulpi launch`) pokazuje okno już otwartego GitPoulpi zamiast uruchamiać kolejne. Nowy terminal uruchamia się tylko wtedy, gdy żaden nie jest otwarty, ustawienie jest wyłączone albo okno nie może wyjść na wierzch. Wychodzi na wierzch w Windows, w WezTerm i w Hyprland; na innych pulpitach Linuksa kliknięcie nadal uruchamia nowe okno. Nieużywane w macOS (robi to już Dock). Aby celowo otworzyć kolejne okno, przytrzymaj **Shift** podczas klikania (Windows, także na przypiętej ikonie paska zadań) albo użyj akcji **New Window** we wpisie programu startowego w Linuksie (prawy przycisk na ikonie); ze skryptu robi to `gitpoulpi launch --new-window`. |
| **Alias polecenia** | `ui.command_alias`: lista rozwijana z pozycjami **Brak** i `gp`, albo wpisz dowolną nazwę (litery, cyfry, `-`, `_`, `.`; bez spacji i separatorów ścieżki). Przycisk **Utwórz** w wierszu sprawia, że ta nazwa uruchamia GitPoulpi (gdy alias już istnieje, zmienia się w **Usuń** i usuwa każdy alias utworzony przez GitPoulpi). Pod wierszem pojawia się ostrzeżenie, gdy nazwa uruchamia już coś innego w twoim PATH, a w Windows także gdy PowerShell ma wbudowany alias o tej nazwie (`gp` to tam `Get-ItemProperty` i wygrywa z każdym plikiem). Aliasu ani funkcji powłoki nie widać stąd: oh-my-zsh samo definiuje `gp` i wygrywa z naszym. Każda instalacja ma własną nazwę; tylko instalacja główna zaczyna od wspólnego `ui.command_alias`, i nigdy z nazwą, której używa już inna instalacja. |
| **Alias nowej karty** | `ui.command_alias_add`: drugie polecenie, na przykład `gpa`, które otwiera bieżący folder jako nową kartę w już uruchomionym GitPoulpi (`gitpoulpi --add`) zamiast uruchamiać kolejny; gdy żadne nie działa, uruchamia GitPoulpi jak zwykle, a gdy działa kilka, pyta, do którego dołączyć. Lista rozwijana z pozycjami **Brak**, `gpa` i nazwą aliasu z dopisanym `a`, albo wpisz dowolną nazwę (te same zasady co wyżej). Jeden przycisk **Utwórz** tworzy oba polecenia, **Usuń** usuwa każde polecenie utworzone przez GitPoulpi; każde ma własny wiersz stanu. W Linuksie i macOS to dwuwierszowy skrypt, w Windows plik `.cmd`. Zob. [Dokumentacja CLI](cli-reference.md#dodawanie-folderu-do-otwartego-okna). |
| **Dodaj GitPoulpi do PATH** | Dodaje do PATH, dla nowych terminali, folder, z którego działa GitPoulpi, i folder aliasów. Przycisk wiersza to **Dodaj** lub **Usuń**, a **Usuń** usuwa dokładnie to, co dodano. Foldery już obecne w PATH zostają nietknięte (wiersz mówi wtedy **już w PATH**). |
| **Linux: Menu aplikacji** | Zapisuje `~/.local/share/applications/gitpoulpi.desktop` (uwzględnia `XDG_DATA_HOME`) i ikonę w `~/.local/share/icons/hicolor/256x256/apps`. W Omarchy wpis jest taki sam jak ten, który tworzy `omarchy-tui-install` (`xdg-terminal-exec --app-id=TUI.tile -e <exe>`); dla innych terminali odpowiednie polecenie (`kitty`, `wezterm start --`, `alacritty -e`, `ghostty -e`, `foot`, ...). |
| **Linux: Pulpit** | Ten sam wpis w folderze Pulpit, jeśli go masz. |
| **Windows: Menu Start** | `GitPoulpi.lnk` w `%APPDATA%\Microsoft\Windows\Start Menu\Programs`. Kopia przypięta do paska zadań sama podąża za zmianami ikony i koloru; jeśli jej ikona pozostanie stara, odepnij ją i przypnij ponownie (prawy przycisk na wpisie w menu Start, potem **Przypnij do paska zadań**). |
| **Windows: Pulpit** | `GitPoulpi.lnk` na pulpicie. |
| **macOS: Folder Aplikacje** | `~/Applications/GitPoulpi.app`, mały pakiet, który otwiera terminal z uruchomionym GitPoulpi. |

Każdy wiersz pokazuje swój stan: **nie utworzono**, **utworzono**, **nieaktualny, aktualizowanie...** (plik exe
został przeniesiony lub wybrano inny terminal; GitPoulpi sam zapisuje go od nowa w ciągu sekundy, więc rzadko to zobaczysz) lub
„istnieje plik, który nie pochodzi od GitPoulpi” (plik o tej nazwie istnieje i nie utworzył go
GitPoulpi, więc nigdy nie jest ruszany). **Usuń** kasuje tylko pliki utworzone przez GitPoulpi
(mają znacznik `X-GitPoulpi=1`). Ikoną jest ośmiornica w kolorze twojego logo (okno
O programie, klawisz `c`); w Linuksie każdy kolor ma własną nazwę ikony, więc menu aplikacji, które buforuje ikony, pokaże nowy kolor. Gdy skrót zostanie utworzony, sam pozostaje aktualny: gdy zmienisz kolor albo terminal lub plik exe się przeniesie, GitPoulpi zapisuje od nowa skróty, które utworzył (nigdy takiego, którego nie utworzył, i nigdy nie tworząc nowego), przy uruchomieniu i w trakcie działania. Nie ma przycisku Aktualizuj; jest tylko **Utwórz** (dla skrótu, którego jeszcze nie ma) i **Usuń**.

**Grupowanie na pasku zadań i w docku.** GitPoulpi działa w oknie terminala, więc pasek zadań zwykle pokazuje je pod ikoną terminala, bez kropki „uruchomione” pod przypiętą ikoną GitPoulpi. Windows: skróty niosą AppUserModelID `GitPoulpi.GitPoulpi`, a GitPoulpi uruchomiony z jednego z nich nadaje oknu swojego terminala ten sam identyfikator i swoją ikonę (znajduje okno, nadając mu na chwilę unikalny tytuł, po czym przywraca tytuł), dzięki czemu okno trafia pod przypiętą ikonę GitPoulpi. Okno terminala z wieloma kartami przenosi się jako całość: wszystkie jego karty grupują się pod ikoną GitPoulpi, dopóki działa w nim GitPoulpi. Wcześniej utworzone skróty i przypięcia na pasku zadań są przepisywane tą samą synchronizacją; jeśli przypięcie nadal pokazuje ikonę terminala, odepnij je i przypnij ponownie. Linux: kitty, foot, Alacritty, WezTerm i Ghostty są uruchamiane z klasą okna `gitpoulpi`, a wpis `.desktop` ją podaje (`StartupWMClass`), więc dock grupuje okno pod ikoną programu startowego; pozostałe terminale zachowują własną ikonę. macOS: nic do zrobienia, Dock grupuje według aplikacji.

![Ustawienia > Uruchamianie z systemu: terminal dla skrótów, build uruchamiany przez skrót, aliasy polecenia i nowej karty, stan aliasów oraz kontrola PATH](../images/settings-launcher.png)

#### Alias polecenia i PATH w poszczególnych systemach

| System | Alias | PATH |
|---|---|---|
| Linux, macOS | Dowiązanie symboliczne `~/.local/bin/<nazwa>` do `gitpoulpi`. Dowiązanie nie może zawierać znacznika, więc uznaje się je za nasze, gdy jego celem jest plik o nazwie `gitpoulpi` (lub `gitpoulpi-tui`, dawny cel, który jest automatycznie przepisywany na `gitpoulpi`); zwykły plik albo dowiązanie do czegokolwiek innego **nie jest nasze** i nigdy nie jest zastępowane ani usuwane. | Blok między `# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>` a `# <<< GitPoulpi PATH <<<` w `~/.bashrc` i `~/.zshrc` (w tych, które istnieją; `~/.zshrc` w macOS, w przeciwnym razie `~/.bashrc`, gdy nie ma żadnego) oraz `~/.config/fish/conf.d/gitpoulpi-path.fish`, gdy skonfigurowano fish. Usuń kasuje blok (lub plik fish) i nic więcej. |
| Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<nazwa>.cmd`, mały plik wsadowy zawierający `REM X-GitPoulpi=1` (eksportuje też `GITPOULPI_CMDCMDLINE`, po czym rozpoznaje się start z paska adresu). | PATH użytkownika w `HKCU\Environment`, zapisany przez PowerShell, a potem rozgłoszenie `WM_SETTINGCHANGE`, by nowe programy go widziały. Wartość `GitPoulpiPathAdded` zapisuje, co dodano, a Usuń zabiera tylko to. |

Nowe terminale widzą zmianę PATH, otwarte już nie. Żadna z tych akcji nie trafia do historii cofania: nie ma dla nich Cofnij, drogą powrotną jest **Usuń**.

Paleta ma te same akcje (**Uruchamianie: Utwórz skrót**, **Uruchamianie: Utwórz skrót na pulpicie**,
**Uruchamianie: Usuń skróty**, **Uruchamianie: Utwórz alias polecenia**, **Uruchamianie: Usuń alias polecenia**,
**Uruchamianie: Dodaj do PATH**, **Uruchamianie: Usuń z PATH**), a `gitpoulpi desktop shortcut
status|install|remove [--where launcher|desktop|startmenu]` robi to samo ze skryptu.

#### Wpisanie w pasku adresu Eksploratora plików (Windows)

Wpisz alias (`gp`) albo `gitpoulpi` w pasku adresu okna Eksploratora plików i naciśnij Enter: GitPoulpi otwiera się w terminalu wybranym w **Terminal dla skrótów**, na folderze pokazanym w oknie, bez otwierania **Ostatniej karty**. Gdy **Pokaż otwarte okno po kliknięciu** jest włączone i GitPoulpi już działa, folder staje się nową kartą tego okna.

- **`gp` (alias polecenia).** Gdy alias istnieje, jego utworzenie rejestruje też `gp.exe` w `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths`, wskazujące na `gitpoulpi-launch.exe`, a Usuń kasuje ten klucz (tylko gdy jest nasz). Gdy folder z aliasem jest w PATH (**Dodaj GitPoulpi do PATH**), Windows uruchamia najpierw plik `.cmd` i okno konsoli może na chwilę mignąć, zanim pojawi się GitPoulpi. Plik `.cmd` zapisany przez starszą wersję jest automatycznie zapisywany od nowa, gdy Ustawienia odczytają alias.
- **`gitpoulpi`.** Działa, gdy zainstalowano **Menu menedżera plików**: Zainstaluj rejestruje też `gitpoulpi.exe` w App Paths. Bez tego `gitpoulpi` jest znajdowany tylko wtedy, gdy folder GitPoulpi jest w PATH, a wtedy konsola może mignąć jak wyżej.

Eksplorator plików zachowuje zmienną PATH z chwili zalogowania się do Windows: folder dodany do PATH później będzie tam widoczny dopiero po wylogowaniu i ponownym zalogowaniu. Do tego czasu znajdowane są tylko wpisy App Paths, a nazwa, której Windows nie znajdzie, jest otwierana w przeglądarce jako adres internetowy.

Linux i macOS nie mają odpowiednika: paski lokalizacji ich menedżerów plików nie uruchamiają programów. Użyj wpisu **Menu menedżera plików** albo aliasu polecenia w terminalu.

#### Menu menedżera plików (Otwórz w GitPoulpi)

Pod skrótami wiersze **Menu menedżera plików** dodają **Otwórz w GitPoulpi** do menu prawego przycisku folderów. Jeden wiersz stanu na każdy menedżer plików (w Linuksie: po jednym dla każdego znalezionego tu menedżera i dla każdego, który wciąż ma wpis; wiersz **Wykryto** wylicza znalezione), potem przyciski **Zainstaluj** i **Usuń**. **Zainstaluj** zapisuje wpis dla każdego znalezionego menedżera plików; **Usuń** najpierw pyta, potem usuwa dokładnie to, co zapisało Zainstaluj, niezależnie od tego, czy ten menedżer plików jest jeszcze zainstalowany.

Wpis uruchamia `gitpoulpi launch --folder <folder>` (w Windows `gitpoulpi-launch.exe --folder` bez konsoli), a to polecenie decyduje w chwili kliknięcia: gdy **Pokaż otwarte okno po kliknięciu** jest włączone i okno GitPoulpi jest otwarte, folder wewnątrz repozytorium staje się nową kartą ostatnio używanego okna, które wychodzi na wierzch; w przeciwnym razie terminal wybrany w **Terminal dla skrótów** uruchamia GitPoulpi na tym folderze. Zmiana terminala nigdy więc nie czyni wpisu nieaktualnym; nieaktualny czyni go przeniesiony plik exe albo nowy kolor logo, a GitPoulpi sam zapisuje swoje wpisy od nowa (nigdy żadnego nie tworząc), tak jak skróty. Wpis wymaga programu startowego obok GitPoulpi (`gitpoulpi`, w Windows `gitpoulpi-launch.exe`); bez niego wiersz to mówi, a przyciski są wyłączone.

| System | Co zapisuje Zainstaluj |
|---|---|
| Windows | Klucze użytkownika, bez administratora: `HKCU\Software\Classes\Directory\shell\GitPoulpi`, `Directory\Background\shell\GitPoulpi` i `Drive\shell\GitPoulpi`, każdy z podkluczem `command`, więc wpis widać na folderze, na tle otwartego folderu i na dysku. Każdy klucz ma wartość `X-GitPoulpi`; klucz bez niej **nie jest nasz** i nigdy nie jest ruszany. Rejestruje też `gitpoulpi.exe` w `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths`, wskazujące na `gitpoulpi-launch.exe`. W Windows 11 wpis jest tylko w klasycznym menu: **Pokaż więcej opcji**, Shift+F10 albo Shift+prawy przycisk (nowe menu najwyższego poziomu wymienia tylko aplikacje pakietowe). Usuń kasuje nasze klucze, potem klucze nadrzędne utworzone przez Zainstaluj, tylko gdy zostały puste. |
| Linux | Tylko dla menedżerów plików znalezionych w PATH. GNOME Files i Caja: skrypt nazwany jak wpis w `~/.local/share/nautilus/scripts` lub `~/.config/caja/scripts` (w podmenu **Skrypty**). Dolphin: `~/.local/share/kio/servicemenus/gitpoulpi-open.desktop`, wykonywalny (KDE Frameworks 6 ignoruje menu usług użytkownika, które takie nie jest; KDE Frameworks 5 przed 5.85 nie jest obsługiwane). Nemo: dwie akcje w `~/.local/share/nemo/actions` (zaznaczony folder, tło). Thunar: jedna akcja dołączona do `~/.config/Thunar/uca.xml`; twoje własne akcje zostają, a Usuń oddaje plik bajt w bajt (albo kasuje go, jeśli utworzył go Zainstaluj). PCManFM i PCManFM-Qt: `~/.local/share/file-manager/actions/gitpoulpi-open.desktop`. Ikona trafia do `~/.local/share/icons/hicolor/256x256/apps/gitpoulpi-folder.png` i znika z ostatnim wpisem. `XDG_DATA_HOME` i `XDG_CONFIG_HOME` są respektowane. Wpis pokazuje się na zaznaczonym folderze oraz na tle folderu, gdzie menedżer plików to oferuje. |
| macOS | Szybka czynność, `~/Library/Services/Open in GitPoulpi.workflow`: kliknij folder w Finderze prawym przyciskiem, potem **Szybkie czynności**. Tylko zaznaczone foldery (Finder nie ma menu dla tła folderu). Przy pierwszym kliknięciu macOS zapyta, czy usługa przepływu pracy może sterować terminalem (monit Automatyzacji, na przykład „WorkflowServiceRunner.xpc chce uzyskać dostęp do sterowania ...”): zezwól. Jeśli wpisu brakuje, włącz go w Ustawienia systemowe > Klawiatura > Skróty klawiszowe > Usługi. |

Każdy plik i klucz zapisany przez GitPoulpi ma znacznik `X-GitPoulpi=1` (wiersz `# X-GitPoulpi=1` w skrypcie), a Usuń kasuje tylko oznaczone. Te akcje zmieniają system, nie repozytorium: Cofnij ich nie obejmuje, drogą powrotną jest **Usuń**. Paski lokalizacji menedżerów plików nie uruchamiają programów w Linuksie i macOS, więc nie ma tam czego wpisywać; w terminalu jest alias polecenia.

Paleta ma **Uruchamianie: Zainstaluj menu Otwórz w GitPoulpi** i **Uruchamianie: Usuń menu Otwórz w GitPoulpi**, a `gitpoulpi desktop folder-menu status|install|remove` robi to ze skryptu.

### Synchronizacja

Jak GitPoulpi utrzymuje repozytorium w zgodzie ze zdalnymi repozytoriami i co w tym czasie
robi z lokalnymi zmianami.

![Strona Synchronizacja: interwał automatycznego pobierania jako liczba i jednostka (minuty) w jednym wierszu, a pod nim pozostałe opcje](../images/settings-quantity.png)

| Opcja | Działanie |
|---|---|
| **Interwał auto-fetch** | Pobiera wszystkie zdalne repozytoria w tle co taki czas, liczba i jednostka (minuty lub godziny) w jednym wierszu. Domyślnie 5 minut, maksimum 60 minut (1 godzina), `0` wyłącza. `general.auto_fetch_interval_minutes` zostaje w minutach; `general.auto_fetch_interval_unit` tylko zapamiętuje sposób wyświetlania. Wymaga restartu. |
| **Fetch po otwarciu karty** | Uruchamia w tle fetch wszystkich zdalnych, gdy otwiera się karta repozytorium. Domyślnie włączone. Repozytorium pobrane w ciągu ostatnich 60 sekund nie jest pobierane ponownie. |
| **Auto-prune** | Fetch usuwa też gałęzie śledzące zdalne, które nie istnieją już na zdalnym repozytorium. Domyślnie włączone, więc gałąź usunięta na serwerze (na przykład na GitLabie) znika z sekcji REMOTE po następnym fetchu. Nie można tego cofnąć: wyczyszczona gałąź wraca przy następnym fetchu, jeśli znów pojawi się na zdalnym. |
| **Auto-prune tagów** | Fetch usuwa też lokalne tagi, które zniknęły ze zdalnego repozytorium. Dotyczy tylko tagów, które tam były: tag, którego nigdy nie wypchnięto (push), zostaje. Domyślnie wyłączone, tak jak w samym git. Wymaga git 2.41 lub nowszego. CLI: `gitpoulpi fetch --prune-tags`; klucz konfiguracji `general.auto_prune_tags`. Zobacz [Gałęzie, tagi i panel Refs](log-and-history.md#gałęzie-tagi-i-panel-refs). |
| **Aktualizuj submoduły** | Po checkout lub pull aktualizuje submoduły do commitów zapisanych w repozytorium. Zob. [Submoduły](submodules.md). |
| **Usuń pliki „.orig” po merge** | Po zakończeniu merge usuwa pliki kopii zapasowych `*.orig`. Domyślnie wyłączone. |
| **Auto-stash przed checkout, pull, merge lub rebase** | Co robić, gdy lokalne zmiany blokują jedną z tych operacji: **pytaj** (okno proponuje zrobienie stasha zmian, wykonanie operacji i przywrócenie stasha), **zawsze** (robi to bez pytania) lub **nigdy** (odmawia). |
| **Uwzględnij nieśledzone pliki w auto-stash** | Automatyczny stash obejmuje też pliki nieśledzone. |

Na tej stronie, w grupie **Aktualizacje**:

| Opcja | Działanie |
|---|---|
| **Sprawdzanie aktualizacji** | Lista rozwijana (`updates.mode`): **Powiadamiaj** (domyślnie) raz dziennie sprawdza w tle i pokazuje `vX.Y.Z dostępna` na pasku tytułu; **Automatycznie** dodatkowo pobiera aktualizację w tle, gdy GitPoulpi zainstalowano z archiwum wydania (moment ponownego uruchomienia nadal wybierasz ty); **Wył.** nigdy nie wysyła zapytań. Działa od razu. Zob. [Aktualizacje](updates.md). |
| **Kanał aktualizacji** | Lista rozwijana (`updates.channel`): **Stabilny** (domyślnie) albo **Wstępny** (także wersje kandydujące). Działa od razu. |
| **Sposób instalacji** | Tylko do odczytu. Jak GitPoulpi trafił na ten komputer (archiwum wydania, instalator dla tego użytkownika lub wszystkich, Chocolatey, Scoop, Homebrew, apt, pakiet MSI, obraz Docker, cargo albo kompilacja deweloperska). Od tego zależy, jak stosowane są aktualizacje, zob. [Aktualizacje](updates.md). |

Wiersz ustawiony przez administratora w polityce aktualizacji obowiązującej na całym komputerze pokazuje **Ustawione przez administratora** i nie można go tu zmienić.

Na tej stronie, w grupie **Przepisywanie historii**, limity czasu skryptów zewnętrznych (zob. [Przepisywanie historii](history-rewrite.md)):

| Opcja | Działanie |
|---|---|
| **Limit czasu na odpowiedź skryptu** | Ile czasu zewnętrzny skrypt ma na odpowiedź dla jednego commita, liczba i jednostka (milisekundy lub sekundy) w jednym wierszu. Domyślnie 5 sekund, od 1 milisekundy do 10 minut (`rewrite.script_timeout_ms`, `rewrite.script_timeout_unit`). Po przekroczeniu cały proces jest zatrzymywany, a przepisywanie kończy się błędem bez żadnych zmian. |
| **Limit czasu skryptu na przebieg** | Ile może trwać jeden pełny przebieg zewnętrznego skryptu, liczba i jednostka (sekundy, minuty lub godziny). Domyślnie 1 godzina, do 24 godzin (`rewrite.script_total_timeout_s`, `rewrite.script_total_timeout_unit`). |

### Profile

![Strona Profile](../images/settings-profiles.png)

Profile tożsamości commitów: imię i nazwisko oraz e-mail używane w commitach
tworzonych z GitPoulpi. Karta **Ogólne** zawiera listę rozwijaną **Aktywny profil**;
karta **Profile** to [lista](#listy) (dodawanie, duplikowanie, usuwanie, zmiana
kolejności) z polami zaznaczonego profilu. Każdy profil ma **ID profilu**, **Nazwa profilu**, **Imię i nazwisko
autora** i **E-mail autora**. Profil bez e-maila ma na liście `⚠`. Bez aktywnego profilu
albo przy profilu z pustą nazwą lub e-mailem obowiązują własne `user.name`
i `user.email` repozytorium.

Przy dwóch lub więcej profilach pasek narzędzi pokazuje na prawym końcu listę
rozwijaną profilu. Ustawia ono profil tylko dla aktywnego repozytorium. Po wyborze
okno proponuje zapisanie tej tożsamości także w konfiguracji tego
repozytorium. Polecenie palety to **Repozytorium: Zmień profil...**.

Te profile to nie profile hostingu ze strony Integracje ani profile AI ze
strony AI.

### SSH i poświadczenia

![Strona SSH i poświadczenia](../images/settings-ssh.png)

SSH korzysta z systemowego `ssh`, więc ssh-agent i konfiguracja ssh nadal
działają. GitPoulpi nigdy nie czyta materiału klucza.

| Opcja | Działanie |
|---|---|
| **Ścieżka klucza prywatnego** | Gdy ustawiona, fetch, pull, push i clone uruchamiają git z `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`, więc ssh proponuje tylko ten klucz. Puste: agent i konfiguracja ssh działają bez zmian. |
| **Ścieżka klucza publicznego** | Używana tylko przy pustej ścieżce klucza prywatnego: ssh wybiera pasujący klucz trzymany przez ssh-agent (lub klucz sprzętowy). |
| **Typ klucza** | Typ, który tworzy **Generuj parę kluczy**: `ed25519`, `rsa` (4096 bitów) lub `ecdsa` (NIST P-256). |
| **Poświadczenia HTTPS** | Jak fetch, pull, push i clone uwierzytelniają się przez HTTPS: **credential_helper** (własny menedżer poświadczeń gita, domyślnie), **prompt** (ignoruje menedżery poświadczeń; git pyta w terminalu) lub **none** (bez poświadczeń, tylko publiczne zdalne repozytoria). |
| **Zapomnij wszystkie nazwy użytkowników i hasła** | Usuwa każdy token zapisany przez GitPoulpi w menedżerze poświadczeń. Najpierw prosi o potwierdzenie. |
| **Generuj parę kluczy** | Tworzy parę kluczy wybranego **Typ klucza** w `~/.ssh/gitpoulpi_<type>`, bez hasła, i zapisuje obie ścieżki. Nigdy nie nadpisuje pliku. |

Ścieżki kluczy obowiązują od następnego transferu. Prefiks `~/` oznacza katalog
domowy.

Ustawienie zrobione gdzie indziej zawsze wygrywa ze ścieżkami z tej strony.
Kolejność, od najwyższego priorytetu:

0. Klucz wybrany dla jednego repozytorium na jego własnej stronie
   [SSH i poświadczenia](#ustawienia-repozytorium-ssh-i-poświadczenia) (grupa ustawień repozytorium).
1. `GIT_SSH_COMMAND` lub `GIT_SSH` w środowisku, w którym działa GitPoulpi.
2. `core.sshCommand` w konfiguracji git repozytorium lub w globalnej
   konfiguracji git. Pozycja *Klucz SSH...* w menu karty starszych wersji
   zapisywała to dla jednego repozytorium, a SSH i poświadczenia oferuje
   przeniesienie tego do GitPoulpi.
3. Dwie ścieżki kluczy z tej strony.
4. Własne domyślne ustawienia ssh i agent.

Klucz może też należeć do konta hostingu; zob.
[Hosting i trackery zgłoszeń](hosting-integration.md#klucz-ssh-na-profil).

### Integracje

![Strona Integracje: dostawcy po lewej i przycisk Połącz dla GitHub](../images/settings-integrations.png)

Łączy konta. Grupa **Hosting** zawiera GitHub, GitHub Enterprise, GitLab,
GitLab Self-Managed, Azure DevOps, Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea, Forgejo i Gerrit. Grupa **Trackery zgłoszeń** zawiera Jira Cloud, Jira Data Center, Trello, Linear, YouTrack, Redmine i Local Markdown. Każdy dostawca ma **Połącz**, pole tokenu, zakresy potrzebne tokenowi
i odnośnik do strony tokenów hosta. Ta strona ma własny przewodnik:
[Hosting i trackery zgłoszeń](hosting-integration.md).

Nad listą dostawców są dwie opcje. Naciśnij `Up` na pierwszym dostawcy, aby do
nich dojść.

| Opcja | Działanie |
|---|---|
| **Odświeżanie: pull requesty i zgłoszenia** | Jak często odświeżają się listy w panelu bocznym i znacznik na pasku narzędzi, liczba i jednostka (minuty lub godziny). Domyślnie 5 minut, do 2 godzin, `0` wyłącza licznik (`general.hosting_refresh_minutes`, `general.hosting_refresh_unit`). |
| **Publikuj jako** | Nazwa, pod którą wiadomości do hosta są publikowane, gdy token repozytorium należy do bota (token GitHub App), na przykład `my-app[bot]`; służy też do rozpoznawania twoich własnych komentarzy. Puste: pytany jest host, a w razie braku odpowiedzi nazwa jest poznawana z pierwszej opublikowanej wiadomości. Repozytorium może ją nadpisać w Ustawienia > Ustawienia repozytorium > SSH i poświadczenia > Hosting (`general.hosting_post_as`). |
| **Terminologia żądań przeglądu** | Słowo używane dla przeglądów na hoście: **Auto (wg hosta)**, **Pull Request** lub **Merge Request**. |
| **Ukryj chip pull requesta, gdy jest zamknięty lub scalony** | Domyślnie włączone (`ui.hide_finished_pr_chip`; na GitLabie nazywa się „merge request”). Chip na pasku narzędzi pokazuje najnowsze żądanie aktywnej gałęzi; gdy to żądanie zostanie zamknięte lub scalone, chip znika, a zamiast niego pojawia się przyciemniony chip **+ PR** (rozpocznij nowe żądanie), tak jakby gałąź go nie miała. Wyłączone: chip zawsze pokazuje najnowsze żądanie, niezależnie od jego stanu. Działa od razu. |

### AI

![Strona AI: Włącz AI, instrukcje globalne i bloki poszczególnych akcji](../images/settings-ai.png)

Na karcie **Ogólne**: przełącznik włączający AI, profil domyślny, instrukcje,
które dostaje każda akcja, oraz profil i instrukcje dla każdej akcji. Karta
**Profile (połączenia)** to [lista](#listy) profili AI z wbudowaną tabelą zmiennych
środowiskowych profilu. Blok *Przegląd kodu* na karcie Ogólne
zawiera też wartości domyślne [kontekstu przeglądu](ai.md#kontekst-przeglądu): **Uruchom agenta w repozytorium**
(*Ustawienie profilu*, *Tak*, *Nie*), **Dołącz CLAUDE.md**, **Dołącz AGENTS.md** i **Dodatkowy plik** (ścieżka lub
link; puste nie dodaje nic, z przyciskiem **Przeglądaj...** pod spodem, który otwiera wybór pliku; `Enter` na przycisku to jego forma klawiaturowa). Ta strona ma własny
przewodnik: [Wsparcie AI](ai.md).

Blok **Odpowiedzi na hoście** zawiera to, co przygotowuje komentarze, odpowiedzi i opisy zgłoszeń: jego profil i
instrukcje, **Prompt bazowy** (puste używa wbudowanego) oraz grupę **Dostęp do repozytorium**: **Domyślnie pozwalaj AI
czytać repozytorium** (wyłączone: wybór w każdym edytorze zaczyna niezaznaczony), **Zachowaj sesję agenta między
odpowiedziami** i **Przechowuj kontekst repozytorium przez (minuty)**. Limity bajtów **Największa rozmowa wysyłana do
odpowiedzi**, **Największy plik repozytorium dla odpowiedzi**, **Największa mapa repozytorium dla odpowiedzi**,
**Maks. trafień wyszukiwania na termin dla odpowiedzi** i **Maks. terminów wyszukiwania dla odpowiedzi** są razem z
innymi limitami wejścia. Zob. [Pomoc w pisaniu na hoście](ai.md#pomoc-w-pisaniu-na-hoście).

Pod blokami poszczególnych akcji jest grupa **Prywatność**:

| Opcja | Działanie |
|---|---|
| **Zastępuj sekrety przed wysłaniem** | Włączone (domyślnie): tokeny, klucze prywatne i hasła znalezione w żądaniu są zastępowane przez `[REDACTED:<rule>]`, zanim żądanie wyjdzie. Wyłączone: czerwony wiersz mówi, że sekrety są wysyłane bez zmian. |
| **Pomijaj wrażliwe pliki** | Włączone (domyślnie): zmiany plików `.env`, kluczy prywatnych i certyfikatów (`[privacy] sensitive_patterns`) nigdy nie są częścią żądania. Wyłączone: czerwony wiersz mówi, że są wysyłane, gdy są częścią żądania. |
| **Pokazuj, co zostanie wysłane, przed...** | Jeden przełącznik na akcję (opis commita, wyjaśnienie, opis stasha, opis pull requesta, kompozytor commitów, rozwiązanie konfliktu, przegląd kodu, podział commita): pokazuje żądanie w oknie **Co zostanie wysłane**, zanim wyjdzie. Pole *Nie pokazuj ponownie* w oknie wyłącza jeden z nich. Akcje przepisywania historii pokazują własne okno. |
| **Pokaż wszystko ponownie** | Pojawia się, gdy któryś przełącznik jest wyłączony; włącza każdy z powrotem. |

Zob. [AI > Prywatność](ai.md#prywatność).

### Agenci i MCP

Co agenci AI mogą robić w tym oknie, czy serwer MCP może działać i jak chroniony jest jego
punkt końcowy HTTP. Wiersze MCP są widoczne tylko w kompilacjach zawierających serwer; zob.
[MCP i agenci](mcp-and-agents.md).

| Opcja | Działanie |
|---|---|
| **Serwer MCP** | Włączone (domyślnie): agent może uruchomić serwer (`gitpoulpi mcp serve`). Wyłączone: odmawia startu. Zapisywane jako `mcp.enabled`. Zmiana działa od następnego uruchomienia serwera przez agenta; już działający serwer zachowuje swoje ustawienia. |
| **Port serwera MCP** | Port, na którym nasłuchuje `--http`, gdy nie podano adresu, od 1 do 65535 (domyślnie 8808). Zapisywany w porcie `mcp.bind`; część hosta zostaje (`127.0.0.1`, gdy pusta). Działa od następnego uruchomienia serwera przez agenta. Instalacja z własnym adresem MCP edytuje właśnie ten adres. |
| **Zaproponuj wolny port** | Szuka wolnego portu na tym samym adresie hosta i zapisuje go jako adres MCP tej instalacji (`mcp.bind` dla instalacji głównej bez nadpisania). Na port przypada jeden serwer MCP HTTP: druga instalacja na zajętym porcie odmawia startu, więc użyj tego, aby wybrać inny. Dostępne też w palecie jako *MCP: Zaproponuj wolny port*. |
| **Pozwól agentom AI sterować widokiem** | Pozwala agentom pokazać w tym oknie commit, plik lub powiadomienie (`ui show`). Wyłączone: żaden agent nie sięga do okna. |
| **Pozwól agentom AI proponować zmiany do mojego potwierdzenia** | Pozwala agentom otwierać wstępnie wypełnione okna Reword, commit, gałęzi lub tagu, które potwierdzasz. Domyślnie wyłączone. Zob. [MCP i agenci](mcp-and-agents.md#uprawnienia). |
| **Otwórz okno dla agentów**, **Polecenie uruchomienia (custom)** | Czy agent może otworzyć okno GitPoulpi, gdy żadne nie jest otwarte, i w jakim terminalu lub multiplekserze. Domyślnie wyłączone. |
| **Uwierzytelnianie HTTP** | Włączone (domyślnie): punkt końcowy MCP HTTP odpowiada tylko na żądanie z tokenem dostępu. Wyłączone pokazuje czerwone ostrzeżenie: każdy program, który ma dostęp do punktu końcowego, może uruchomić każde narzędzie dozwolone w ustawieniach MCP, a `--allow-remote` jest odrzucane. Zapisywane jako `mcp.http_auth`. |
| **Token dostępu** | Token, zamaskowany. Zamaskowana wartość jest przyciskiem: `Enter` (lub kliknięcie) pokazuje ją w całości w wierszu poniżej i ukrywa ponownie. **Kopiuj** kładzie ją w schowku bez echa w wierszu stanu, **Wymień** zastępuje ją po potwierdzeniu (klienci z poprzednim tokenem są odrzucani od razu, bez restartu serwera), **Utwórz** tworzy pierwszy. `Left` / `Right` przechodzą między przyciskami, a `Enter` naciska jeden. Ten sam token pokazuje `gitpoulpi mcp token`. |
| **TLS** | **Wył.** (domyślnie), **Samopodpisany** (serwer raz tworzy certyfikat i pokazuje jego odcisk) lub **Pliki** (twój certyfikat i klucz). Serwer sam kończy TLS. Zapisywane jako `mcp.tls`; działa od następnego uruchomienia serwera. |
| **Ścieżka certyfikatu**, **Ścieżka klucza** | Pliki PEM używane z **Pliki** (`mcp.tls_cert`, `mcp.tls_key`). |
| **Odcisk certyfikatu** | Tylko do odczytu: odcisk SHA-256 certyfikatu, który udostępnia punkt końcowy, do sprawdzenia przez klientów. |
| **Za tunelem lub zwrotnym proxy** | Twoje oświadczenie, że tunel lub proxy przed punktem końcowym szyfruje ruch, co pozwala `--allow-remote` działać bez TLS. Serwer nie może tego sprawdzić, więc pokazuje czerwone ostrzeżenie, gdy jest włączone. Zapisywane jako `mcp.behind_proxy`. Zob. [MCP i agenci](mcp-and-agents.md#dostęp-do-punktu-końcowego-z-innego-komputera). |

### Wygląd

![Strona Wygląd przewinięta w dół: źródła awatarów i awatar zapasowy, kolor pól tekstowych, głębia kolorów, kursor tekstowy i początek kolorów grafu (formaty dat są niżej)](../images/settings-ui.png)

![Lista Motyw otwarta na stronie Wygląd z wyróżnionym motywem Jasny: cały ekran Ustawień już maluje się w tym motywie](../images/settings-appearance-theme.png)

| Opcja | Działanie |
|---|---|
| **Motyw** | **System (zgodny z terminalem)**, motyw wbudowany albo jeden z własnych plików motywu. Wyróżniony motyw jest podglądany na żywo: `Enter` go zachowuje, `Esc` przywraca poprzedni. Pasek **Kolory** pod wierszem pokazuje obowiązujące kolory. Zob. [Motywy](themes.md). |
| **Awatary autorów** | **wył.** (domyślnie, bez sieci), **graf** (w kropce commita) lub **autor** (w kolumnie Autor). Po włączeniu hasz e-maila każdego autora jest wysyłany do Gravatar i do dostawcy hostingu. |
| **Źródła awatarów**, **Awatar zapasowy** | Gdzie szuka się obrazków i w jakiej kolejności. Dla autorów bez obrazka: **identicon** (generowany wzór) lub **inicjały**. |
| **Kolor pól tekstowych**, **Własny kolor pól tekstowych** | Tło pól tekstowych: **Zgodnie z motywem** (domyślnie), stały ciemny kolor albo własny `#rrggbb`. |
| **Kursor tekstowy** | `ui.text_cursor`: **terminal** (domyślnie) ustawia własny kursor terminala w miejscu karetki aktywnego pola tekstowego, jako pionową kreskę (obowiązują ustawienia kursora Twojego terminala, a okno kandydatów metody wprowadzania otwiera się przy karetce); **drawn** rysuje własną karetkę GitPoulpi i zostawia kursor terminala ukryty. Działa od razu. Paleta: `Ustawienia: Kursor tekstowy...`. |
| **Głębia kolorów** | **auto** (domyślnie), **truecolor**, **256** lub **16**. Zob. [Motywy](themes.md). |
| **Motyw kolorów grafu** | **Zgodnie z motywem** (domyślnie: własne kolory ścieżek motywu) lub dowolna paleta ścieżek, także twoje własne pliki motywów (po presetach). Każda paleta pokazuje kilka swoich kolorów; skonfigurowana nazwa, która już nie istnieje, jest oznaczona *nie znaleziono*, a graf używa ścieżek motywu. Zmiana obowiązuje po zamknięciu Ustawień. Polecenie palety **Aplikacja: Zmień motyw kolorów grafu...** (`Ctrl+N Shift+T`) podgląda każdą paletę na żywo. |
| **Tło ścieżek grafu** | **Wył.** (przezroczyste) lub **Podbarwienie wiersza**: delikatny pas koloru ścieżki za kolumną grafu, z cienkim paskiem na brzegu. |
| **Intensywność podbarwienia** | Jak blady jest pas podbarwienia: **Mocne**, **Średnie** (domyślnie) lub **Subtelne**. Pasek na brzegu zachowuje pełny kolor. |
| **Położenie paska narzędzi** | `ui.toolbar_position`: Gdzie leży pasek narzędzi: U góry, Na dole (domyślnie), Po lewej lub Po prawej. Po lewej i Po prawej robią z niego kolumnę obok paneli; w oknie węższym niż 80 kolumn jest rysowany na dole. Działa od razu. |
| **Zapamiętuj położenie i rozmiar okien dialogowych** | `ui.dialog_geometry`: **Na tę sesję** (domyślnie), **Zapisane na dysku** lub **Nigdy**: czy okno dialogowe, które przesunąłeś lub którego rozmiar zmieniłeś, zachowuje swoje miejsce. Jedno ustawienie dla wszystkich okien; każde okno pamięta własne miejsce i rozmiar. Działa od razu. |
| **Wyrównanie przycisków paska narzędzi** | `ui.toolbar_align`: Gdzie leżą przyciski wzdłuż paska: Początek (domyślnie), Środek lub Koniec. Początek to lewa strona wiersza i góra kolumny; koniec to prawa strona i dół. Działa od razu. |
| **Przypnij przycisk pull request na końcu** | `ui.toolbar_pr_pinned`: Włączone (domyślnie): przycisk pull request leży na samym końcu paska narzędzi (po prawej, w kolumnie na dole) przy każdym wyrównaniu. Wyłączone: płynie razem z innymi przyciskami i jest z nimi wyrównany. Działa od razu. |
| **Położenie paska stanu** | `ui.statusbar_position`: Gdzie leży pasek stanu (gałąź, liczniki, komunikaty, podpowiedzi klawiszy): U góry, nad kartami repozytoriów, lub Na dole (domyślnie). Karta Home zachowuje własny pasek stanu na dole. Działa od razu. |
| **Położenie paska kart** | `ui.tabs_position`: Gdzie leżą karty repozytoriów: U góry (domyślnie), Na dole lub Po lewej, lista z jedną kartą w wierszu przy lewej krawędzi okna. W oknie węższym niż 80 kolumn są rysowane u góry. Obok paska narzędzi w kolumnie karty zostają na zewnątrz. Działa od razu. |
| **Ustawienia regionalne daty/czasu** | **systemowy** (domyślnie): nazwy miesięcy i dni tygodnia w języku interfejsu. **english**: zawsze angielskie nazwy. |
| **Format daty**, **Format daty/czasu**, **Format daty rozszerzony** | Wzorce dat dla kolumny daty w Historii pliku i Blame, dla kolumny Data w Logu oraz dla szczegółów commita. Każdy pokazuje podgląd na żywo. Znaczniki opisuje [Opis konfiguracji](config-reference.md#ui-w-tuitoml). |

![Lista Motyw kolorów grafu: Zgodnie z motywem, a potem każda paleta ścieżek z kilkoma jej kolorami](../images/settings-log-graph-lanes.png)

Motyw i kolory grafu (Motyw, motyw kolorów grafu, kolorowanie i tło ścieżek, siła podkreślenia wierszy) należą do każdej instalacji: gdy kilka instalacji współdzieli jeden `config.toml`, zmiana ich w jednej nigdy nie zmienia pozostałych. Tylko instalacja główna zapisuje też wspólne klucze `ui.*`, w `tui.toml`; pozostałe trzymają swoje we własnym wpisie `config.toml`.
### Klawiatura i mysz

![Strona Klawiatura i mysz: każda akcja według kontekstu z jej klawiszami, pole wyszukiwania i filtry](../images/settings-keyboard.png)

**Ustawienia > Klawiatura i mysz** ma dwie karty (kliknięcie albo `Alt+Left` / `Alt+Right`).
**Ogólne** zawiera protokół klawiatury i ustawienia myszy; **Skróty klawiszowe**
wylicza każdy skrót według kontekstu, z polem wyszukiwania (`/`) i filtrami. W wierszu:

| Klawisz | Akcja |
|---|---|
| `Enter` lub podwójne kliknięcie | Przechwytuje nowy klawisz. Konflikty i ostrzeżenia terminala pokazują się podczas naciskania. |
| `a` | Dodaje klawisz alternatywny. |
| `u` | Odpina akcję. |
| `r` | Przywraca wiersz do domyślnego. |
| `R` | Resetuje wszystkie skróty, po potwierdzeniu. |
| `c` lub kliknięcie `!` w wierszu | Pokazuje akcję ze wszystkimi akcjami, z którymi koliduje jej klawisz; `Esc` znów pokazuje wszystko. Klawisz kolidujący z klawiszem innej akcji jest czerwony, ostrzeżenie dotyczące samego klawisza (terminal nie może go wysłać) pomarańczowe. |

Użyj tego do rozwiązywania kolizji z tmux, AutoHotkey, herdr lub terminalem.
Te same ustawienia znajdują się w tabeli `[keybindings]` pliku `tui.toml`;
zob. [Opis konfiguracji](config-reference.md#keybindings) i
[Skróty klawiszowe](keybindings.md).

![Strona Klawiatura i mysz po naciśnięciu c na Zakończ: klawisz q jest czerwony przy Zakończ i przy Stage lub unstage zaznaczony wiersz, dwóch kolidujących akcjach](../images/settings-keyboard-conflicts.png)

Trzy opcje są na karcie **Ogólne**.

| Opcja | Działanie |
|---|---|
| **Protokół klawiatury Kitty** | **auto** prosi terminal o protokół klawiatury kitty, dzięki czemu docierają `Ctrl+,`, `Ctrl+Enter` i `Ctrl+Shift+N`. **off** nigdy nie prosi; użyj, jeśli terminal źle działa. Wymaga restartu. |
| **Mysz włączona** | Wyłączone zostawia mysz i zaznaczanie tekstu terminalowi. Wymaga restartu. |
| **Linie kółka myszy** | O ile wierszy w Logu przesuwa jeden krok kółka. Wymaga restartu. |

### Sekwencje

**Ustawienia > Sekwencje** edytuje twoje nazwane [sekwencje](sequences.md): listy
poleceń palety uruchamianych jedno po drugim, każda jest wierszem palety i da się
ją przypisać do klawisza. To [strona z listą](#listy) z jedną twarzą: **+ Dodaj**
oferuje pustą sekwencję i szablony (**Kopia zapasowa**, **Skomponuj i push**,
**Worktree i agent**), wiersze to **Nazwa**, **Id**, **Opis**, **Zakres** (ta
instalacja, wszystkie instalacje na tym komputerze albo tylko to repozytorium),
przycisk **Skrót**, który otwiera okienko przechwytywania z **Klawiatura i mysz**,
**Edytuj kroki** i **Uruchom teraz**. **Edytuj kroki** przełącza listę na kroki
sekwencji, każdy z **Pauzą** (**Domyślnie**, **Zawsze**, **Nigdy**) i swoimi opcjami;
**+ Dodaj** tam otwiera filtrowalną listę poleceń, którymi może być krok. Każda
zmiana działa od razu. Zob. [Sekwencje](sequences.md).
### Log i graf

Jak rysowany jest graf commitów i ile commitów wczytuje Log. Kolory grafu są na
stronie **Wygląd**.

| Opcja | Działanie |
|---|---|
| **Renderer grafu** | **auto** rysuje obraz tam, gdzie terminal to obsługuje, a znaki gdzie indziej. **obraz** wymusza obraz, **tekst** wymusza znaki. Wymaga restartu. |
| **Protokół obrazów** | Jak obrazy trafiają do terminala: **auto**, **kitty**, **iterm2**, **sixel** lub **none**. Wymaga restartu. |
| **Pokaż stashe w grafie** | Wiersz dla każdego stasha w Logu. Wyłączone: stashe widać tylko w panelu bocznym. |
| **Pokaż widmową gałąź dla najechanego commita** | Domyślnie włączone (`ui.graph_ghost_ref`). W kolumnie Ref najechany commit, który nie ma własnego refa, pokazuje wyblakłą pigułkę z nazwą gałęzi, do której należy (w razie braku jej gałąź zdalną), wywnioskowaną z historii, tam gdzie kolumna ma miejsce. Gałąź usunięta po scaleniu nadal się pokazuje, przekreślona, pod nazwą z tematu merge commita (`Merge branch 'x'`, `Merge pull request #N from owner/x`, ...); scalenia squash, rebase i fast-forward nie zostawiają takiego śladu. Tagi nigdy nie są pokazywane jako widmo. Najechanie na pigułkę pokazuje jej rodzaj. Kliknięcie nie robi nic szczególnego; gałąź tworzy polecenie palety `Git: Utwórz gałąź dla zaznaczonego commita...`. |
| **Łącznik ref–commit** | Cienka linia od etykiety ref do kropki jej commita. |
| **Układ kolumn** | `ui.log_columns_layout`, stosowany od razu w Logu. **Dopasuj do panelu** (domyślnie): każda kolumna mieści się dokładnie w panelu i tylko Opis się rozciąga. **Przewijaj w poziomie**: kolumny zachowują swoje szerokości, Opis też, a tabela przewija się w poziomie, gdy jest szersza niż panel (`Shift+Left` / `Shift+Right`, pozioma rolka, pasek na dolnej krawędzi). Ten sam przełącznik jest w oknie wyboru kolumn Logu i w palecie. |
| **Zmiana szerokości kolumn** | `ui.column_resize`, stosowane od razu w Logu, Radarze i kolumnie commitów w widokach diffa, pliku i blame. **line** (domyślnie): każda granica kolumny to odstęp jednej komórki z małym znacznikiem w nagłówku, cienka linia akcentu rysuje się w dół całej tabeli, gdy wskaźnik jest na granicy, a przeciągnięcie z dowolnego miejsca wzdłuż niej zmienia szerokość. **header**: bez znacznika, pierwszy znak komórki nagłówka za granicą zapala się i jest jedynym miejscem, z którego można przeciągać. Klawisze zmiany szerokości (`Shift+C` w Logu, `C` w Radarze, `Tab` w kolumnie commitów) działają tak samo w obu. |
| **Pokaż liczniki zmian WIP w opisie, gdy kolumna Changes jest ukryta** | Pokazuje liczniki zmian (`✎ 3  + 2  − 1`) w opisie wiersza WIP. Gdy kolumna Zmiany jest widoczna, liczniki trafiają tam. |
| **Drzewo folderów gałęzi** | Grupuje gałęzie i tagi po `/` w foldery w panelu bocznym. `y` w panelu Refs przełącza. |
| **Sekcje panelu bocznego** | Od czego zależy, które sekcje panelu bocznego są pokazane lub ukryte: **per repository** (domyślnie; każde repozytorium ma własny zestaw, globalny jest punktem wyjścia) albo **global** (jeden zestaw dla wszystkich repozytoriów). Przełączanie tam i z powrotem zachowuje oba zestawy. Zob. [mysz i gesty](mouse-and-gestures.md). (`ui.sidebar_sections_scope`) |
| **Kliknięcie refu pokazuje go w grafie** | Pojedyncze kliknięcie refu w panelu Refs przewija Log do miejsca, na które ref wskazuje, i na chwilę podświetla wiersz (zaznaczenie commita i prawy panel pozostają bez zmian). Po wyłączeniu: kliknięcie tylko zaznacza wiersz paska bocznego. **Pokaż w grafie** działa w obu przypadkach. |
| **Propozycje kompozytora** | Czy kompozytor commitów AI otwiera się z plikami każdej propozycji **rozwinięte** czy **zwinięte**. |
| **Początkowe commity w grafie** | Ile commitów wczytuje Log po otwarciu repozytorium. Domyślnie 5000, minimum 500. Wyższe wartości spowalniają duże repozytoria. |
| **Leniwe wczytywanie commitów w grafie** | Włączone: kolejne commity wczytują się w tle, gdy przewiniesz do najstarszego wczytanego commita. Wyłączone: Log zatrzymuje się na commitach początkowych. |
| **Commity na stronę leniwego wczytywania** | Ile commitów pobiera jedno wczytanie w tle. |
| **Maksymalna liczba commitów w grafie** | Leniwe wczytywanie zatrzymuje się na tej liczbie. `0` oznacza brak limitu. Każdy commit zajmuje około 300 bajtów pamięci. |
| **Metoda liczenia: commity** | Jak pasek stanu ustala łączną liczbę w `wczytane / łącznie` (`log.commit_count`, lista rozwijana, działa od razu): **System git** uruchamia jeden proces `git rev-list --count` (domyślnie, zwykle najszybciej); **libgit2** liczy wewnątrz GitPoulpi i nie uruchamia procesu, dla maszyn, na których oprogramowanie antywirusowe skanuje każde polecenie i spowalnia każde wywołanie gita; **Wył.** nigdy nie liczy i pokazuje tylko wczytane commity. |
| **Pokaż autorów przez .mailmap** | Log, blame i historia pliku pokazują każdego autora tak, jak nazywa go `.mailmap` repozytorium (`log.use_mailmap`, przełącznik, domyślnie włączony, działa od razu). Zapisane commity nigdy się nie zmieniają. Wyłączone pokazuje nazwy takie, jakie zatwierdzono. Zob. `history-rewrite.md`, *Karta Tożsamości*. |

### Diff i edytor

![Strona Diff i edytor](../images/settings-editor.png)

| Opcja | Działanie |
|---|---|
| **Rozmiar tabulacji** | Kolumn na tabulator, od 1 do 8 (domyślnie 4). |
| **Koniec linii** | Co edytor zapisuje przy zapisie: **Auto (zachowaj z pliku)**, **LF** lub **CRLF**. |
| **Podświetlanie składni** | Kolory języka w edytorze. |
| **Numery linii** | Kolumna numerów linii w edytorze i w diff. |
| **Podążaj za zaznaczeniem w edytorze** | Gdy edytor jest otwarty, zaznaczenie innego pliku w prawym panelu otwiera go także w edytorze, po zapytaniu o zapisanie niezapisanych zmian. Plik, którego nie można edytować, otwiera się w zwykłym widoku. Tylko dla zmian roboczych: plik wybrany z commita opuszcza edytor na rzecz diffa tego commita. Domyślnie włączone (`editor.follow_selection`). |
| **Zawijanie wierszy** | Zawija długie linie diff. |
| **Podświetlaj zmienione słowa** | W zmienionej linii maluje słowa różniące się od usuniętej lub dodanej linii, z którą jest sparowana, mocniejszym tłem w widoku Diff (W linii i Obok siebie), w diffie przeglądu i w plikach commita. Domyślnie włączone (`ui.diff_intraline`); także **Diff: Przełącz podświetlanie zmienionych słów** w palecie. Zob. [Zmienione słowa](interface-tour.md#zmienione-słowa-w-diffie). |
| **Domyślny układ**, **Tylko hunki**, **Pokaż zmiany tylko w białych znakach**, **Minimapa** | Początkowy stan widoku diff. Pasek widoku diff zmienia je i zapisuje. |
| **Adnotacje Blame**, **Szerokość adnotacji Blame** | Po której stronie widoku Blame jest kolumna adnotacji (awatar, autor, wiek, opis): **right** (domyślnie) lub **left**, oraz jej szerokość w komórkach (od 24 do 90). Przeciągnięcie krawędzi kolumny w widoku Blame albo klawisze `>` / `<` zapisują tu szerokość (`ui.blame_side`, `ui.blame_width`). Zob. [Blame](log-and-history.md#blame). |
| **Edytor merge: obok siebie od (kolumn)** | Od tej szerokości edytor merge pokazuje A i B obok siebie. Domyślnie 100, minimum 40, `0` oznacza zawsze. Poniżej A i B są kartami. Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md). |
| **Zewnętrzny edytor** | Polecenie dla pozycji pliku *Otwórz w edytorze*. `{file}` to ścieżka. Puste: VS Code, jeśli jest zainstalowany, w przeciwnym razie domyślna aplikacja systemu. |
| **Zewnętrzne narzędzie diff** | Polecenie dla *Otwórz w zewnętrznym narzędziu diff*. `{local}` i `{remote}` to dwie wersje. Puste: `git difftool`. |
| **Zewnętrzne narzędzie merge** | Polecenie dla *Otwórz w zewnętrznym narzędziu merge* na karcie konfliktu. `{base}`, `{local}`, `{remote}` i `{merged}` to pliki. Puste: `git mergetool`. |
| **Eksplorator plików** | Polecenie dla *Pokaż w folderze*. `{path}` to plik, `{dir}` jego folder. |

### Otwórz w

![Strona Otwórz w, karta Wpisy: lista wpisów, edytowany wpis z etykietą i poleceniem oraz przycisk + Dodaj](../images/settings-open-in.png)

Edytuje listę przycisku **Otwórz w** na pasku tytułu. Karta **Ogólne** zawiera
przyciski folderu **Zakończ i cd tutaj**, **Wpis domyślny** i **Przywróć wykryte
ustawienia domyślne**. Karta **Wpisy** to [lista](#listy) (każdy wpis z etykietą
i poleceniem): **+ Dodaj ▾** oferuje ustawienia wstępne i pozycję **Własny**,
a wpis domyślny jest oznaczony `★`. Pola zaznaczonego wpisu
zawierają pole **Może otwierać worktree** (domyślnie zaznaczone: wpis jest
oferowany w menu Otwórz w wiersza worktree, zob. [Worktree](worktrees.md))
oraz przycisk **Testuj**. Zob. [Otwórz w](open-in.md).

Pierwszy wiersz, **Podążaj za folderem aktywnej karty**, ma dwie wartości:

- **przyciski** (domyślnie): rząd kart pokazuje **Zakończ i cd tutaj**, gdy
  aktywna karta nie jest folderem, w którym uruchomiono GitPoulpi.
- **auto**: bez przycisku. Każde zakończenie zmienia folder powłoki na aktywną
  kartę, a otwarcie nowej karty uruchamia domyślny wpis Otwórz w.

Zob. [Podążanie za folderem aktywnej karty](following-the-tab-folder.md).

### Obrazy

Jak pokazywane są pliki obrazów i którego programu używa **Edytuj w**. Zob.
[Obrazy](image-view.md). Karta **Ogólne** zawiera poniższe opcje podglądu,
**Wpis domyślny** i **Przywróć wykryte ustawienia domyślne**; karta **Edytory** to
lista edytorów obrazów. Kompilacja bez podglądu obrazów nie ma opcji podglądu.

| Opcja | Działanie |
|---|---|
| **Podgląd obrazów** | Pokazuje plik obrazu jako obraz w widoku Diff i widoku pliku. Wyłączone: zwykła informacja o pliku binarnym. Domyślnie włączone. |
| **Bez protokołu obrazów** | W terminalu bez protokołu obrazów w linii: **Kolory półbloków** (domyślnie) lub **Tylko karta informacyjna**. |
| **Maks. rozmiar podglądu** | Większe obrazy nie są dekodowane; zamiast tego pokazuje się karta informacyjna. Liczba i MiB lub GiB, domyślnie 32 MiB, do 1 GiB (`ui.image_view_max_mib`, `ui.image_view_max_unit`). |
| **Powiększ pikselowo od** | `1x`, `2x` (domyślnie) lub `never`: od tego powiększenia obraz pokazuje kwadratowe piksele zamiast wygładzonego obrazu. |
| **Synchronizuj powiększenie i przewijanie przed/po** | Oba panele diff obrazu dzielą jedno powiększenie i jedną pozycję przewijania. Domyślnie włączone. |

Karta **Edytory** to [lista](#listy) jak [Otwórz w](open-in.md): dla każdego edytora **Etykieta**, **Polecenie**, przyciski zmiennych
`${file}` `${dir}` `${name}` `${repo}` i **Testuj na przykładowym obrazie**;
**+ Dodaj ▾** oferuje ustawienia wstępne i pozycję **Własny**, a **Przywróć wykryte
ustawienia** resetuje listę. Dopóki jej nie zmienisz,
lista zawiera edytory zainstalowane na tym komputerze i domyślną aplikację
systemu. Każda zmiana działa od razu.

### Zapobieganie konfliktom

Globalna część proaktywnego wykrywania konfliktów (`gitpoulpi ui show settings conflict-prevention-global`); strona jednego repozytorium to
[Ustawienia repozytorium: Zapobieganie konfliktom](#ustawienia-repozytorium-zapobieganie-konfliktom). Otwiera ją polecenie palety **Konflikty: Edytuj listy gałęzi bazowych...**.

| Opcja | Co robi |
|---|---|
| **Włącz proaktywne wykrywanie konfliktów** | Domyślnie włączone (`general.conflict_detection`). Ostrzega w Log, że merge bieżącej gałęzi do gałęzi, z której powstała, spowodowałby konflikt, zanim go spróbujesz. Repozytorium może to wyłączyć dla siebie na jego własnej stronie **Zapobieganie konfliktom**. Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md#zapobieganie-konfliktom). Działa po zamknięciu Ustawień. |
| **Domyślne gałęzie bazowe** | Pod przełącznikiem. Lista wzorców gałęzi bazowych, której używają repozytoria bez własnej: **Wbudowana (`**`)**, czyli każda gałąź, albo jedna z twoich nazwanych list (`conflict_prevention.default_list`). Działa w otwartych repozytoriach po zamknięciu Ustawień. **Przywróć wbudowaną wartość domyślną** wraca do `**`. |
| **Listy gałęzi bazowych** | Także na tej stronie: twoje nazwane listy wzorców gałęzi bazowych (`[conflict_prevention.lists.<name>]`), po jednym bloku na listę, z jej **Nazwą** (zmiana nazwy zachowuje status domyślnej, jeśli go miała; repozytorium, które śledziło starą nazwę, używa listy domyślnej, dopóki ponownie jej nie wybierzesz), jej **Wzorcami** (jeden w wierszu, `Enter` lub kliknięcie otwiera edytor, `Ctrl+S` go zachowuje), **Ustaw jako domyślną** i **Usuń listę** (najpierw pyta i mówi, że repozytoria, które z niej korzystają, użyją zamiast niej listy domyślnej). Wpisz nazwę w **Nowa lista** i naciśnij `Enter`, aby dodać listę; zaczyna się od `**`. Edycja listy zmienia każde repozytorium, które z niej korzysta. |

### Ustawienia repozytorium: Git i kodowanie

![Strona Git i kodowanie dla tego repozytorium](../images/settings-git.png)

| Opcja | Działanie |
|---|---|
| **Backend gita** | `git2` (libgit2, domyślnie) lub `system` (zainstalowany `git`). Globalne. Działa dla demonów uruchomionych po zmianie. |
| **Przycisk Pull** | Co robi przycisk Pull: merge (fast-forward, jeśli możliwy), tylko fast-forward lub rebase. Globalne. Pozycje `▾` tego nie zmieniają. |
| **Merge fast-forward** | Co robi merge, który sam uruchamiasz, gdy gałąź dałoby się po prostu przewinąć fast-forward: **never** (domyślnie) zawsze tworzy merge commit, **allowed** robi fast-forward, gdy się da (domyślne zachowanie gita), **only** robi fast-forward albo odmawia. Globalne (`git.merge_fast_forward`); repozytorium nadpisuje to na swojej stronie [Opcje: merge](#ustawienia-repozytorium-opcje-merge). |
| **Przycisk Push** | **od razu** wypycha natychmiast; **pytaj** najpierw pokazuje menu zdalnych repozytoriów. Globalne. |
| **Sprawdź każde zdalne przed pushem** | Domyślnie włączone (`git.push_group_preflight`). Przed pushem gałęzi, która ma dodatkowe cele push, najpierw sprawdza każde zdalne (szybkie zapytanie, a pobranie z danego zdalnego tylko wtedy, gdy się różni), więc problem na trzecim zdalnym widać, zanim ruszy pierwszy push. Wyłączone: push do kolejnych zdalnych po kolei i raport, co się stało. Globalne. |
| **Dodatkowe cele push dla nowych gałęzi** | Tylko to repozytorium: pod tym nagłówkiem jest jedno pole wyboru na każde zdalne repozytorium. Zaznaczone zdalne staje się dodatkowym celem push każdej nowej gałęzi, obok jej upstreamu, w chwili gdy gałąź po raz pierwszy dostaje upstream (pierwszy push, który go ustawia, albo gałąź utworzona z gałęzi zdalnej). Wybór jest kopiowany do własnych kluczy `gitpoulpi-push-to` gałęzi, więc istniejące gałęzie się nie zmieniają: edytuj je z plakietki zdalnych (`⇉3`) w wierszu Refs (albo `u` na wierszu). Zapisane w danych repozytorium (`default_push_targets` w `repo-state.toml`). |
| **Domyślne kodowanie** | Kodowanie używane do pokazania tekstu, który **nie jest poprawnym UTF-8**: zawartości plików, blame, diffów i opisów commitów. Strona ma wiersz globalny i wiersz dla tego repozytorium; wiersz repozytorium wygrywa. Poprawny UTF-8 nigdy nie jest dekodowany ponownie, a staging niczego nie koduje ponownie. |
| **core.autocrlf**, **core.eol** | Własne ustawienia git tego repozytorium. Wiersz zapisuje `core.autocrlf` lub `core.eol` w konfiguracji git repozytorium. **(dziedziczone)** usuwa wartość repozytorium, więc obowiązuje globalna konfiguracja git. `Ctrl+Z` przywraca poprzednią wartość. Odpowiednik w CLI to `gitpoulpi repo-config`. |
| **Git LFS** | Pokazuje Git LFS w interfejsie. Zob. [Git LFS](lfs.md). |
| **Instaluj filtry LFS przed pull i push** | Przed pull lub push uruchamia `git lfs install --local`, gdy repozytorium używa LFS, a jego filtrów brakuje. |
| **Program git-lfs** | Wykryty program `git-lfs` (tylko do odczytu). |
| **Zainicjuj LFS dla tego repozytorium** | Uruchamia teraz `git lfs install --local`. |

### Ustawienia repozytorium: SSH i poświadczenia

`gitpoulpi ui show settings ssh-credentials`. Jak *to repozytorium* się loguje, przechowywane w prywatnym folderze
GitPoulpi: nic nie jest zapisywane w repozytorium ani w konfiguracji gita, a zmiany nie można cofnąć (jak żadnego
ustawienia). Używają tego tylko fetch, pull, push, podmoduły i LFS tego repozytorium. Odpowiednik w CLI to
`gitpoulpi repo-auth` (zob. [Dokumentacja CLI](cli-reference.md)). Stronę otwiera też paleta poleceń:
**Repozytorium: SSH i poświadczenia...**. Nic nie jest wysyłane, dopóki nie naciśniesz **Zapisz**; strona pyta demona,
więc nigdy nie blokuje.

| Wiersz | Co robi |
|---|---|
| **HTTPS: Loguj przez** | **System** (własni pomocnicy gita, domyślnie), **Konto GitPoulpi** (token integracji; konta, których host jest jednym ze zdalnych repozytoriów tego repozytorium, są na początku, a wiersz pod spodem mówi, co wykazało ostatnie sprawdzenie tokenu: brakujące i nadmiarowe zakresy), **Pomocnik poświadczeń** lub **Polecenie tokenu**. |
| **Nazwa użytkownika gita** | Wysyłana jako nazwa użytkownika zdalnych repozytoriów HTTPS. Z **System** pozwala menedżerowi poświadczeń wybrać jedno z kilku zapisanych kont. |
| **Ignoruj pomocników poświadczeń ustawionych przez to repozytorium** | Tylko **System**. Pomocnicy, których repozytorium ustawia samo (jego `.git/config` i dołączane pliki), są wypisani pod spodem z plikiem, z którego pochodzą; pomocnicy systemowi i globalni nadal działają. |
| **Pomocnik**, **Polecenie**, **Hosty**, **Pamięć podręczna (minuty)** | Polecenie, które repozytorium uruchamia, by się zalogować. *Pomocnik* to pomocnik poświadczeń gita; *polecenie tokenu* wypisuje token (lista argumentów jest dzielona po spacjach, z cudzysłowem wokół argumentu ze spacjami, i pokazana pod **Uruchamiane jako**; żadna powłoka go nie uruchamia, działa w folderze repozytorium). **Hosty** ogranicza je do tych hostów (puste: host głównego zdalnego repozytorium); **Pamięć podręczna** to czas ponownego użycia tokenu (domyślnie 50 minut). **Użyj obecnego pomocnika tego repozytorium** kopiuje do pola pomocnika, którego ustawia repozytorium. |
| **Użyj do API hostingu (zgłoszenia, pull requesty)** | Z poleceniem tokenu: to samo polecenie podaje też token wywołań zgłoszeń i pull requestów tego repozytorium, zamiast twojego osobistego tokenu (token instalacji GitHub App, więc komentarze pojawiają się jako App). Włączenie lub wyłączenie prosi o potwierdzenie. Zob. „Działanie jako bot GitHub App” w przewodniku [integracje z hostingiem](hosting-integration.md). |
| **Przypisywanie** | **Dozwolone** (domyślnie) lub **Zablokowane**, zapisywane od razu. Gdy zablokowane, w zgłoszeniach i pull requestach tego repozytorium nie można nic przypisać z aplikacji, CLI ani od agenta; istniejący przypisani pozostają widoczni. |
| **SSH: Loguj przez** | **System** (własna konfiguracja ssh, agent i domyślne klucze), **Plik klucza** lub **Klucz konta** (klucz integracji). |
| **Plik klucza prywatnego** | Wybierz jeden z kluczy znalezionych w `~/.ssh` albo wpisz ścieżkę. Podawany jest tylko ten klucz, każdemu zdalnemu repozytorium SSH tego repozytorium, także podmodułom. Gdy inne konta mogą czytać plik klucza (bit uprawnień dla grupy lub innych na Linuksie i macOS, wpis dla Everyone, Users lub Authenticated Users na Windows), ostrzeżenie mówi, że OpenSSH odrzuca taki klucz („bad permissions”), a **Ogranicz do mojego konta** to naprawia (`chmod 600`, a na Windows DACL tylko dla twojego konta i SYSTEM). |
| **Plik znanych hostów**, **Ścisła kontrola klucza hosta** | Opcjonalny plik zaufanych kluczy hostów oraz odrzucanie hosta, którego w nim nie ma. |
| **Program ssh** | Program, który zostanie uruchomiony (tylko do odczytu). |
| **Przypięty w konfiguracji gita** | Tylko gdy to repozytorium ustawia własne `core.sshCommand` (dawne przypięcie „Klucz SSH dla tego repozytorium”, które GitPoulpi kiedyś zapisywał). Polecenie jest pokazane tylko do odczytu. **Przenieś do GitPoulpi** zapisuje równoważny wybór **Plik klucza**, a potem usuwa przypięcie z konfiguracji gita; jest dostępne, gdy polecenie to zwykłe przypięcie `ssh -i <klucz>`, i tylko gdy strona nie ma niezapisanych zmian, a jej wybór SSH to nadal **System**. Każde inne polecenie jest pokazane z uwagą: edytuj je przez `git config`. Przeniesienia nie można cofnąć, jak każdego ustawienia tutaj, a usunięcie przypięcia też nie trafia na stos cofania (nic nie ginie: klucz jest w profilu). |
| **Zdalne repozytoria** | Tylko do odczytu: każde zdalne repozytorium z protokołem i tożsamością, której użyje. Ustawienia zastępowane przez profil (`core.sshCommand`, `credential.helper`) są wypisane z plikiem, a profil, którego nie można użyć, mówi dlaczego. |

Przyciski (każdy ma klawisz: `Tab`/`Left`/`Right` między nimi, `Enter` naciska):

- **Testuj połączenie** sprawdza każde zdalne repozytorium z zapisanymi wyborami, najpierw fetch, potem push (żadne dane
  się nie przesyłają), i otwiera panel wyników: dla każdego zdalnego repozytorium ok lub błąd, tożsamość, która
  odpowiedziała (klucz zaakceptowany przez serwer albo konto, którego token przyjęto), czas trwania i zwykły opis
  niepowodzenia. Działa w tle, ze spinnerem po chwili. Panel przewija się przez `Up`/`Down`, `PgUp`/`PgDn`,
  `Home`/`End`, kółko i pasek przewijania, a zamyka przez `Esc` lub krzyżyk. Odpowiednik w palecie to
  **Repozytorium: Testuj połączenie**.
- **Zapisz** wysyła wybory. **Cofnij zmiany** porzuca niezapisane. Zamknięcie Ustawień z niezapisanymi wyborami najpierw pyta.
- **Użyj domyślnych** pyta, a potem zapomina profil: pomocnicy gita dla HTTPS i własna konfiguracja ssh dla SSH.
  Odpowiednik w palecie to **Repozytorium: Użyj domyślnych poświadczeń**, który też pyta.

Zapis **Pomocnika poświadczeń** lub **Polecenia tokenu** najpierw pyta **Zezwolić na to polecenie?**: polecenie jest
pokazane słowo w słowo (znaki sterujące są wypisane, długie wiersze zawijane), z jego SHA-256 i dwiema odpowiedziami,
`1` **Zapisz i zezwól** oraz `2` **Anuluj**. Żadna odpowiedź nie jest wybrana, dopóki nie naciśniesz strzałki lub cyfry,
więc przypadkowy `Enter` nic nie robi; `Esc`, **Anuluj** i krzyżyk panelu anulują. `PgUp`/`PgDn`/`Home`/`End` i pasek
przewijania przewijają długie polecenie. Jeśli polecenie jest już zapisane i niezmienione, zapis drugiej połowy strony
nie pyta ponownie.

Profil, którego nie można użyć (brakujący plik klucza, źródło, którego ta wersja jeszcze nie obsługuje), powoduje
niepowodzenie transferu z podaniem powodu zamiast powrotu do tożsamości komputera.

Gdy profil nie jest w całości domyślny, menu **Push** kończy się wierszem mówiącym, jako kto push się loguje (**Loguje
jako: github-corp (token)** lub **Klucz SSH: work_ed25519**); otwiera on tę stronę. Ten sam wiersz jest dodawany do wpisów
Dziennika aktywności dla fetch, pull i push. Niepowodzenie spowodowane wyborami z tej strony (pomocnik poświadczeń, który
nie mógł się uruchomić, jeszcze nieznany klucz hosta SSH, odrzucone logowanie, polecenie tokenu, które zawiodło, token
niewysłany, bo serwer nie jest adresem konta) pokazuje na swoim wpisie Dziennika aktywności przycisk **SSH i
poświadczenia...** (`r` na wpisie), obok **Otwórz Integracje**, gdy źródłem może być konto integracji. Wiersz **SSH i
poświadczenia...** w menu karty też otwiera stronę; zastępuje dawne menu przypięcia **Klucz SSH...** i jego wpis w palecie.

### Ustawienia repozytorium: Częściowy checkout

Te same elementy co w oknie Częściowy checkout (`Repozytorium: Częściowy checkout...`), jako strona ustawień tego
repozytorium (`gitpoulpi ui show settings sparse-checkout`).

| Opcja | Działanie |
|---|---|
| **Stan** | Czy to drzewo robocze jest częściowym checkoutem. |
| **Katalogi do zachowania** | Jeden katalog w wierszu, względem katalogu głównego repozytorium. `Enter` (lub kliknięcie) otwiera wielowierszowy edytor: `Ctrl+S` zachowuje tekst, `Esc` go porzuca. Gdy częściowy checkout jest włączony, pole pokazuje bieżące katalogi. Tryb cone gita zna tylko katalogi (bez wzorców i pojedynczych plików), pliki w katalogu głównym są zawsze w checkoucie, a pusta lista jest odrzucana. |
| **Rozpocznij częściowy checkout** | Wypakowuje tylko wymienione katalogi. Gdy częściowy checkout jest już włączony, ten sam przycisk nazywa się **Zastosuj listę katalogów** i stosuje edytowaną listę: katalog, który z niej zniknął, jest usuwany z katalogu roboczego. Wyszarzony, dopóki lista jest pusta. |
| **Zakończ częściowy checkout** | Powrót do pełnego checkoutu: wszystkie pliki wracają (po potwierdzeniu). Wyszarzony, dopóki częściowy checkout jest wyłączony. |
| **Odśwież częściowy checkout** | Stosuje bieżącą listę jeszcze raz (na przykład gdy skrypt lub inne narzędzie wypakowało pliki spoza niej). Wysyła bieżące katalogi, nie niewysłany tekst pola. Wyszarzony, dopóki częściowy checkout jest wyłączony. |

Przyciski działają jak zwykłe akcje w tle (strona pokazuje „stosowanie częściowego checkoutu...”, gdy któraś trwa). Podobnie
jak okno, nie trafiają do historii cofania: nic nie jest tracone, a **Wyłącz** przywraca każdy plik.

### Ustawienia repozytorium: LFS

Stan LFS tego repozytorium (`gitpoulpi ui show settings lfs`; nie ma go w kompilacji bez LFS).

- **Niezainicjowany:** „Git LFS nie jest skonfigurowany w `<repozytorium>`” i przycisk **Skonfiguruj LFS tutaj**,
  który uruchamia `git lfs install --local` (filtry i hooki tego repozytorium, nigdy twoja konfiguracja globalna; wymaga
  `git-lfs`). Gdy `.gitattributes` repozytorium już kieruje pliki przez LFS, ale brakuje jego hooków, strona o tym
  informuje.
- **Zainicjowany:** „Git LFS jest skonfigurowany w `<repozytorium>`” oraz wzorce śledzone w `.gitattributes`
  (`filter=lfs`).

LFS uznaje się za **zainicjowany**, gdy repozytorium ma własne hooki LFS (hook `pre-push` lub `post-checkout`, który
zapisuje `git lfs install --local`). Same filtry się nie liczą: `git lfs install` na poziomie użytkownika ustawia je dla każdego
repozytorium, więc każde wyglądałoby na zainicjowane.

Przycisk **`LFS ▾`** na pasku narzędzi podąża za tym samym stanem: pokazuje się tylko w repozytorium, w którym LFS jest zainicjowany lub
które `.gitattributes` już go używa, a w zwykłym repozytorium pozostaje ukryty (oraz do czasu pierwszego odczytu LFS
repozytorium, więc może pojawić się chwilę po jego otwarciu). Inicjowanie z tej strony sprawia, że się pojawia. Polecenia palety
`LFS:` pozostają dostępne wszędzie, także **LFS: Zainicjuj dla tego repozytorium**.

### Ustawienia repozytorium: Commit

Jak zachowuje się commit tworzony w tym repozytorium (`gitpoulpi ui show settings commit`). Oba przełączniki są domyślnie
**wyłączone** i zapisują się tylko dla tego repozytorium (`[preferences]` w
[`repo-state.toml`](config-reference.md#ustawienia-repozytorium-gitpoulpirepo-statetoml)).

| Ustawienie | Działanie |
|---|---|
| **Push po utworzeniu każdego commita** | Po udanym commicie wypycha bieżącą gałąź do jej upstreamu, jak zwykły push (śledzony transfer, z tekstem w toku „zatwierdzanie, potem wypychanie...”). Gałąź bez upstreamu jest zatwierdzana, ale nie wypychana, i powiadomienie o tym informuje: pierwszy push nowej gałęzi pozostaje zadaniem przycisku **Push**, który ustawia upstream. Odrzucony push otwiera zwykłą kartę odrzuconego pusha; commit zostaje. |
| **Domyślnie omijaj hooki gita przy tworzeniu commita** | Pole **Pomiń hooki** formularza commita (obok **Popraw poprzedni commit**; `Shift+H` w panelu Status albo **Commit: Przełącz pomijanie hooków** w palecie) jest domyślnie zaznaczone, więc hooki `pre-commit` i `commit-msg` się nie uruchamiają (`git commit --no-verify`). Nadal jest to wybór dla każdego commita: po commicie wraca do tej wartości domyślnej. |

Bez tej wartości domyślnej hooki commita repozytorium uruchamiają się przy każdym commicie, niezależnie od używanego backendu
(repozytorium z hookami commita jest zatwierdzane przez systemowy `git`, bo libgit2 żadnego nie uruchamia). Hook, który się nie powiedzie,
pokazuje swoje wyjście we wpisie Dziennika aktywności (pasek stanu mówi, że commit się nie udał) i nic nie jest zatwierdzane.

### Ustawienia repozytorium: Opcje: merge

Dwa ustawienia. **Fast-forward** mówi, co robi merge, gdy gałąź dałoby się po prostu przewinąć fast-forward:

| Wybór | Merge, który uruchamiasz... |
|---|---|
| **(dziedziczone)** | podąża za globalnym ustawieniem **Merge fast-forward** ze strony Git (domyślnie **never**). |
| **never** | zawsze tworzy merge commit (`git merge --no-ff`), nawet gdy gałąź jest po prostu przed celem. Pierwszym rodzicem commita jest stary wierzchołek, drugim scalana gałąź, a jego opis to opis gita (`Merge branch 'x' into y`). |
| **allowed** | robi fast-forward, gdy się da, a w przeciwnym razie tworzy merge commit (domyślne zachowanie gita). |
| **only** | robi fast-forward i odmawia z komunikatem *cannot fast-forward*, gdy gałęzie się rozeszły (`git merge --ff-only`). |

Dotyczy każdego merge, który uruchamiasz: menu gałęzi, przeciągnięcie, `Merge` w palecie i `gitpoulpi merge`
(którego flagi `--ff`, `--no-ff` i `--ff-only` nadpisują je dla jednego merge). Menu mówią, co merge zrobi:
*Merge x do y (merge commit)*, *(fast-forward)* albo *(tylko fast-forward)*, a osobna pozycja
*Fast-forward y do x* (i jej polecenie palety) pozostaje o jeden gest dalej przy każdym wyborze. Merge commit jest
zwykłym wpisem cofania, tak jak fast-forward. Akcje Flow wybierają własną strategię merge i ignorują to ustawienie.

Twój własny wybór wygrywa z wyborem gita: merge uruchomiony przez GitPoulpi przy **never** lub **only** przekazuje flagę
jawnie, więc `merge.ff` w konfiguracji gita repozytorium jest nadpisane. Tylko przy **allowed** nadal obowiązuje `merge.ff` gita, a gdy
repozytorium je ustawia, strona pokazuje to pod wierszem. (Dla tej linii czytany jest tylko własny `.git/config` repozytorium, nie twoja globalna konfiguracja gita.)

**Squash** (domyślnie wyłączony). Gdy włączony, merge lokalnej gałęzi (menu gałęzi, przeciągnięcie, `Merge`
w palecie) dodaje zmiany gałęzi do poczekalni bez tworzenia merge commita (`git merge --squash`): commit zostaje po twojej stronie.
Formularz commita otwiera się z komunikatem squash gita (`SQUASH_MSG`, bez linii komentarzy), raz po każdym
squashu. Potem nic nie jest w toku (brak `MERGE_HEAD`), więc **Przerwij** nie ma zastosowania: aby się wycofać, odrzuć zmiany
w poczekalni. Squash, który zatrzymuje się na konfliktach, zostawia je w panelu Status jak każdy merge z konfliktem.
Zmiany w poczekalni nie są wpisem cofania (żaden ref się nie przesunął, więc nie ma czego przywracać); polecenie CLI
`gitpoulpi merge <branch> --squash` robi to samo, niezależnie od ustawienia. Squash wygrywa z **Fast-forward**.

### Ustawienia repozytorium: Szablon opisu

Opis, od którego zaczyna się formularz commita (`gitpoulpi ui show settings commit-template`).

- **Wstaw szablon do pustego panelu commita**: *pusty* formularz commita jest wstępnie wypełniany szablonem, raz na commit
  (ręczne wyczyszczenie jest respektowane do następnego commita).
- **Odrzucaj linie komentarzy przy tworzeniu commita**: przy tworzeniu commita linie zaczynające się od `#` (lub od `core.commentChar`
  repozytorium) są usuwane z opisu, tak jak zrobiłoby `git commit --cleanup=strip`. GitPoulpi tworzy commity przez
  `-m`, którego git nie czyści, więc robi to sam. Przy wyłączonym przełączniku linia `#` zostaje w commicie.
- **Podsumowanie** (etykieta liczy, ile zostało z 72 kolumn i ile jest ponad) oraz wielowierszowy **Opis**
  (`Enter` lub kliknięcie otwiera edytor, `Ctrl+S` go zachowuje).
- **Zapisz szablon** zapisuje szablon w `commit-template.txt` w folderze danych repozytorium (zob.
  `session.per_repo_state_dir`) i ustawia `commit.template` tego repozytorium na ten plik, więc korzystają z niego także
  `git commit` w wierszu poleceń i inne narzędzia. Jeśli `commit.template` wskazuje już inny plik, strona pokazuje ścieżkę
  tego pliku, edytuje *ten plik* i zostawia ustawienie w spokoju.

### Ustawienia repozytorium: Zapobieganie konfliktom

`gitpoulpi ui show settings conflict-prevention`. Strona proaktywnego wykrywania konfliktów tego repozytorium
(zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md#zapobieganie-konfliktom)):

- **Włącz proaktywne wykrywanie konfliktów z gałęzią docelową**: domyślnie włączone. Odznaczenie wyłącza wykrywanie
  tylko dla tego repozytorium; przełącznik globalny to **Włącz proaktywne wykrywanie konfliktów** na globalnej stronie **Zapobieganie konfliktom**.
  Oba muszą być włączone.
- **Gałęzie bazowe z**: skąd to repozytorium bierze gałęzie bazowe. **Domyślne** podąża za globalną wartością domyślną
  (wbudowane `**` albo lista wybrana w **Domyślne gałęzie bazowe** w Ustawienia > Zapobieganie konfliktom); **Lista: <nazwa>**
  podąża za jedną z twoich nazwanych list, więc edycja listy zmienia także to repozytorium; **Własna lista** to lista tylko
  tego repozytorium (jej wybór zaczyna od kopii obowiązujących wzorców). Plik repozytorium zapisany przed
  pojawieniem się tego wyboru i zawierający własne wzorce jest odczytywany jako **Własna lista**.
- **Gałęzie bazowe** (jeden wzorzec w wierszu): przy **Własnej liście** `Enter` lub kliknięcie otwiera edytor, a `Ctrl+S` go
  zachowuje; przy **Domyślne** lub **Lista** obowiązujące wzorce są pokazane tylko do odczytu. Tabela wzorców pod spodem jest
  ściągą:

  | Wzorzec | Pasuje do |
  |---|---|
  | `origin/main` | tylko ta gałąź |
  | `origin/**` | każda gałąź zdalnego `origin` |
  | `origin/feature/**` | gałęzie `origin` zaczynające się od `feature/` |
  | `!origin/abc` | wyklucza tę gałąź |

  Gałąź lokalna to sama nazwa (`main`), a gałąź śledząca zdalną ma przed nazwą swoje zdalne (`origin/main`). `*` i `?`
  zostają w obrębie jednego segmentu ścieżki, `**` je przekracza. Puste linie i komentarze `#` są ignorowane; lista
  samych wykluczeń oznacza „każda gałąź oprócz tych”. Wybór jest zapisywany w `repo-state.toml` i odczytywany ponownie po
  zamknięciu Ustawień.
- **Zapisz jako listę...**: wpisz nazwę i naciśnij `Enter`; obowiązujące wzorce stają się nazwaną listą (tą samą, którą
  edytuje Ustawienia > Zapobieganie konfliktom), a to repozytorium z niej korzysta.
- **Przywróć domyślne** (najpierw pyta): odrzuca wybór, listę i własne wzorce tego repozytorium; znów podąża za domyślnymi.
- **Użyj gałęzi bazowych przepływu pracy** (widoczne, gdy dla repozytorium skonfigurowano [przepływ pracy gałęzi](branching-workflows.md)
  z gałęziami bazowymi): wypełnia **Własną listę** tymi bazami, każdą jako gałąź lokalną i jako gałąź dowolnego
  zdalnego (`main` i `*/main`). Kopiuje je jeden raz; nie podąża za późniejszymi zmianami przepływu pracy.

### Ustawienia repozytorium: AI

![Strona AI repozytorium: instrukcje repozytorium z opcjami Dodaj do globalnych lub Zastąp globalne oraz profil i instrukcje dla każdej akcji](../images/settings-repo-ai.png)

Profil AI tego repozytorium i instrukcje dla każdej akcji oraz *Instrukcje
repozytorium*, które obowiązują dla każdej akcji w tym repozytorium. Puste pola
używają globalnych ustawień AI. Zob. [Wsparcie AI](ai.md#instrukcje). Jej blok *Przegląd kodu* powtarza
cztery wiersze kontekstu przeglądu: każdy wybór zaczyna się od *Użyj globalnych (wartość)*, pokazującego wartość
globalną, a pusty **Dodatkowy plik** pokazuje globalny wyszarzony (i ma ten sam przycisk **Przeglądaj...**; wybór zaczyna
w katalogu głównym repozytorium, a plik w repozytorium jest zapisywany względem niego); przełącznik **Brak dodatkowego pliku**
wyłącza globalny plik dodatkowy dla tego repozytorium, a puste pole nadal za nim podąża; **Przywróć globalne** (widoczne, gdy repozytorium ma
własne wartości) przywraca wszystkie. Są zapisane w pliku `repo-state.toml` tego repozytorium.

### Ustawienia repozytorium: Przepływ pracy

`gitpoulpi ui show settings workflow`. Przepływ pracy z gałęziami w tym repozytorium (szablon, wykrywanie, gałąź bazowa) ma własny przewodnik: [Przepływy pracy z gałęziami](branching-workflows.md#ustawienia--przepływ-pracy). Karta **Ogólne**
zawiera to, co nie jest regułą (szablon, wykrywanie, hooki, przepływ ze zgłoszeń, miejsce zapisu); karta **Reguły** to
[lista](#listy) w grupach (**Gałęzie bazowe**, **Typy gałęzi**, **Reguły
zakończenia**); **+ Dodaj ▾** wybiera rodzaj, a elementy przesuwają się w obrębie swojej
grupy. W odróżnieniu od innych stron trzyma wersję roboczą (pasek jest na obu kartach): pasek przypięty na dole strony
pokazuje **● Niezapisane** (albo **Bez zmian**) z przyciskami **Zapisz we wspólnym
pliku**, **Tylko dla mnie** i **Odrzuć zmiany**, a zamknięcie Ustawień z niezapisaną
wersją roboczą najpierw pyta.

## Pliki, zmienne środowiskowe i CLI

- Pliki ustawień to `config.toml` (ustawienia wspólne) i `tui.toml` (ustawienia
  interfejsu terminalowego UI) w katalogu konfiguracji systemu.
  `gitpoulpi config path` wypisuje ich położenie; `gitpoulpi config get <key>`
  mówi, w którym pliku jest klucz.
- Stan poszczególnych repozytoriów jest w `.gitpoulpi/repo-state.toml` wewnątrz
  repozytorium.
- `gitpoulpi config get <key>` i `gitpoulpi config set <key> <value>` czytają
  i zapisują każde ustawienie.
- Zmienne środowiskowe i flagi wiersza poleceń nadpisują plik.

Szczegóły podaje [Opis konfiguracji](config-reference.md).
