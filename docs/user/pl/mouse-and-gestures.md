[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · **Polski** · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Mysz i gesty

Wszystko, co można zrobić myszą, ma też odpowiednik na klawiaturze. Zob.
[Skróty klawiszowe](keybindings.md). Nazwy obszarów są w
[przewodniku po interfejsie](interface-tour.md).

## Zasady ogólne

- **Kliknij** wiersz, by go zaznaczyć. Działa to w każdym panelu, liście, menu i
  oknie dialogowym.
- **Kliknij dwukrotnie** wiersz, by go otworzyć lub uruchomić. Podwójne kliknięcie lokalnej gałęzi,
  tagu lub plakietki refa robi jej checkout. Podwójne kliknięcie wiersza repozytorium otwiera
  to repozytorium.
- **Kliknij prawym przyciskiem** wiersz, by zobaczyć jego menu. `.`, `Shift+F10` lub klawisz Menu otwiera
  to samo menu dla zaznaczonego wiersza.
- **Kółko** przewija obszar pod wskaźnikiem. Na liście przesuwa
  tylko widok. Zaznaczenie zostaje tam, gdzie było, nawet gdy wyjedzie poza
  widok. Zaznaczenie przesuwa tylko kliknięcie lub klawisz.
- **Shift+kółko** przewija w bok, w diffie, widoku pliku i edytorze,
  gdy zawijanie wierszy jest wyłączone.
- **Najechanie** na przycisk, wiersz lub suwak paska przewijania podświetla go. Wiele terminali
  nie potrafi zmienić wskaźnika myszy, więc to podświetlenie jest często jedyną oznaką,
  że coś jest klikalne. Zatrzymaj wskaźnik na przycisku lub komórce Log, by zobaczyć
  podpowiedź.
- **Kliknięcie poza** polem tekstowym opuszcza je. To, co wpisano, zostaje. Filtr
  Refs stosuje to, co wpisano, tak jak `Enter`. Pasek wyszukiwania Log
  zamyka się, tak jak po `Esc`.

## Zamykanie okien i dialogów

Każde okno z ramką i każdy dialog ma `✕` w prawym górnym rogu ramki, a wszędzie
obowiązują te same reguły:

- **Kliknij `✕`**, by zamknąć. Tak samo działa `Esc`, który najpierw zamyka to, co
  najbardziej wewnętrzne (otwartą listę rozwijaną, pole tekstowe, potwierdzenie), a dopiero
  potem okno.
- **Kliknij poza** dialogiem, panelem pomocy lub informacji albo oknem podręcznym, by go
  zamknąć. Kliknięcie poza nigdy nie zamyka pełnego widoku (Ustawienia, widok Diff,
  Widok pliku, edytory, Przegląd, Tablica): one nie mają „poza”.
- Kliknięcie **wewnątrz** panelu nigdy go nie zamyka.
- Formularz z wpisanym tekstem (nowe zgłoszenie, pull lub merge request) ignoruje
  kliknięcie poza, więc przypadkowe kliknięcie nie zgubi tekstu. `✕` i `Esc` najpierw
  pytają **Odrzucić zmiany?**; **Edytuj dalej** wraca do formularza. Formularz bez
  wpisanego tekstu zamyka się od razu, tak samo jak jednowierszowy dialog.
- Paleta poleceń, selektory (kart, motywu, języka), małe dialogi Przeglądu, Tablicy,
  merge i edytora oraz monit o potwierdzenie też mają `✕`. Menu kontekstowe i listy
  rozwijane go nie mają: zamykają się `Esc` lub kliknięciem poza.

## Listy w Ustawieniach

Listy w Ustawieniach (Profile, profile AI, Przepływ pracy, Otwórz w, Obrazy) obsługuje
się myszą tak. **Kliknij podkartę** (Ogólne albo listę), aby się na nią przełączyć; `Alt+Left` / `Alt+Right` robi to samo.

- **Kliknij** wiersz, by go zaznaczyć, **kliknij dwukrotnie**, by przejść do jego pól,
  **kółko** przewija bez zaznaczania.
- **Naciśnij wiersz i przeciągnij** go w górę lub w dół, by zmienić kolejność: pasek
  pokazuje, gdzie wyląduje, puszczenie go upuszcza, `Esc` anuluje. Odpowiednik na
  klawiaturze to `Alt+Up` / `Alt+Down`.
- **Kliknij `+ Dodaj`, `Duplikuj`, `Usuń`, `↑`, `↓`** pod listą. Klawisze: `Insert` lub
  `Ctrl+N`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; zob.
  [Ustawienia](settings.md#listy).

## Listy w oknach podręcznych

Te zasady dotyczą każdej listy w oknie podręcznym, które ma pole filtra: paleta
poleceń, jej selektory (motyw, język, kolory grafu), selektor kart, listy
rozwijane z wyszukiwaniem, lista odwołań do zgłoszeń i pomoc.

- **Kliknij** wiersz, by go zaznaczyć.
- **Kliknij dwukrotnie** wiersz, by go uruchomić lub wybrać, tak jak `Enter`.
- **Kółko** przewija listę. Nie przesuwa zaznaczenia.
- Gdy lista jest dłuższa niż okno, na jego prawej krawędzi pojawia się **pasek przewijania**.
  Przeciągnij suwak lub kliknij tor.

Krótka lista rozwijana bez pola filtra wybiera wiersz pierwszym kliknięciem.
Menu kontekstowe uruchamia wiersz pierwszym kliknięciem. Klawisze tych list są
w [Skrótach klawiszowych](keybindings.md#filtrowalne-listy-w-oknach).

## Paski przewijania

Każdy obszar, który może się przewijać, pokazuje cienki pasek przewijania na swojej ramce, gdy jego
zawartość się nie mieści: Log, sekcje Refs, listy plików, diff,
widok pliku, edytor i listy w oknach podręcznych.

- **Przeciągnij suwak**, by przewijać.
- **Kliknij tor**, by przesunąć o stronę w stronę kliknięcia. W Logu i na listach
  plików widok przeskakuje do miejsca kliknięcia.
- **Przytrzymaj przycisk na strzałce końcowej**, by przewijać dalej. Zaczyna po
  krótkim opóźnieniu. Zjedź ze strzałki, by wstrzymać.
- Pasek przewijania przesuwa tylko widok. Nigdy nie przesuwa zaznaczenia.

W diffie i edytorze pionowy pasek przewijania pokazuje też cały plik
w małych znacznikach: zielone dla dodanych linii, czerwone dla usuniętych, bursztynowe dla
zmienionych, `◆` dla linii kursora i `▌` dla każdego innego miejsca, w którym występuje
zaznaczony tekst. Edytor merge oznacza każdy konflikt na swoich
paskach przewijania. Kliknij znacznik, by tam przejść.

## Błędy i Dziennik aktywności

- **Kliknij czerwony błąd** na pasku stanu, by otworzyć Dziennik aktywności na jego wpisie; **kliknij Kopiuj / sugerowany przycisk** (Ponów, Najpierw Pull, Stash i ponów, Otwórz Integracje) we wpisie.
- **Kliknij Aktywność** na prawym końcu paska stanu, by otworzyć Dziennik aktywności. Kliknij kartę, by przełączyć, kliknij pole poziomu (albo **Wszystko** / **Nic**), by pokazać lub ukryć te wiersze, kliknij wpis karty Operacje git, by otworzyć jego szczegóły, kliknij **Wyczyść dziennik** na karcie Aplikacja (najpierw pyta). Długie wiersze przewijaj na boki przez `Shift` + kółko lub poziomy pasek przewijania. Zob. [Dziennik aktywności](troubleshooting.md#dziennik-aktywności).
- **Kliknij `[x]`** przy wskaźniku zadań po prawej stronie paska stanu, aby anulować trwające zadanie, **kliknij jego etykietę**, aby otworzyć panel zadań. Przycisk **Aktywność** w prawym dolnym rogu karty Start (lub `Ctrl+N Shift+G`) otwiera dziennik aktywności na Wszystkich repozytoriach (jego pole Zakres, lista rozwijana pod kartami, wybiera repozytorium albo obszar roboczy).

## Zmiana rozmiaru

- **Przeciągnij separator** między panelem bocznym Refs, Log i prawym panelem. Klawiatura przesuwa je o jeden krok naraz: `Ctrl+N Left` / `Ctrl+N Right` przesuwają separator Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` separator Log | prawy panel (separator idzie w stronę, w którą wskazuje strzałka).
  Przeciągnij go poza minimalną szerokość, by zwinąć ten panel. Klawiaturowe odpowiedniki to `Ctrl+N Left` / `Ctrl+N Right` (separator między panelem Refs a Log) i `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (separator między Log a prawym panelem), po jednym kroku, separator idzie w stronę, w którą wskazuje strzałka; w palecie są to `Nawigacja: Poszerz panel Refs`, `... Zwęź panel Refs`, `... Poszerz panel Status` i `... Zwęź panel Status` oraz `Nawigacja: Zwiń/rozwiń panel Refs` / `... Zwiń/rozwiń panel Status`.
- **Przeciągnij dolną krawędź** sekcji Refs, by zmienić jej wysokość.
- **Przeciągnij dolną krawędź** bloku opisu commita w prawym panelu.
- **Przeciągnij krawędź między dwoma nagłówkami Log** (domyślnie odstęp, z nagłówka lub z dowolnego miejsca w dół tabeli; Ustawienia > Log i graf > Zmiana szerokości kolumn), by zmienić rozmiar kolumny (klawiatura: `Shift+C`, by wybrać kolumnę, potem `]` / `[`).
- **Kliknij dwukrotnie granicę między dwoma nagłówkami Log**, by dopasować kolumnę po jej lewej do jej zawartości (klawiatura: `Shift+C`, by wybrać kolumnę, potem `=`; `+` dopasowuje każdą kolumnę). Ostatnia kolumna ma uchwyt na swojej ostatniej komórce: przeciągnij go, by zmienić jej szerokość, a kolumna Opis przejmie różnicę.
- **Przewijaj Log w poziomie** (układ kolumn `scroll`; Ustawienia > Log i graf > Układ kolumn albo pole `Przewijaj w poziomie` w oknie wyboru): pozioma rolka lub `Shift`+rolka nad tabelą, pasek na dolnej krawędzi (przeciągnij suwak, naciśnij puste tło paska, kliknij strzałki na jego końcach) albo `Shift+Left` / `Shift+Right`. W domyślnym układzie `fit` kolumny wypełniają panel i nie ma czego przewijać.
- **Przeciągnij nagłówek Log**, by przenieść kolumnę: duszek jego nazwy podąża za wskaźnikiem, nagłówek, który opuścił, jest przyciemniony, a pasek akcentu wskazuje miejsce, w którym wyląduje. Kolumny przesuwają się po puszczeniu; `Esc` anuluje, a wskaźnik pokazuje otwartą dłoń, dopóki ją niesiesz (klawiatura: `Shift+C`, by wybrać kolumnę, potem `<` / `>`).

GitPoulpi zapamiętuje każdy rozmiar. Zob.
[Rozmiary są zapamiętywane](interface-tour.md#rozmiary-są-zapamiętywane).

## Pasek tytułu, rząd kart i pasek narzędzi

- **Kliknij 🐙** na pasku tytułu, by otworzyć **O programie GitPoulpi**. W tym oknie źrenice ośmiornicy
  podążają za wskaźnikiem; kliknij logo, by obrócić je do następnego koloru, kliknij je prawym
  przyciskiem, by zobaczyć listę kolorów, kliknij link, by go otworzyć.
- **Kliknij Otwórz w ▾** na pasku tytułu, by zobaczyć listę programów. Zob.
  [Otwórz w](open-in.md).
- **Kliknij wersję** (`v1.0.0` albo `vX.Y.Z dostępna`) na pasku tytułu, by otworzyć kartę Informacje o wersji.
- **Kliknij lupę i `Szukaj wszędzie`** na pasku tytułu, tuż na lewo od koła zębatego, by otworzyć
  [wyszukiwanie globalne](search.md).
- **Kliknij koło zębate** na pasku tytułu, na lewo od ×, by otworzyć Ustawienia.
- **Kliknij ×** na prawym końcu paska tytułu, by zakończyć.
- **Kliknij kartę**, by się na nią przełączyć. **Kliknij jej `x`** (widoczny na aktywnej karcie
  i na karcie pod wskaźnikiem), by ją zamknąć. **Przeciągnij kartę** w bok, aby zmienić jej
  kolejność. **Kliknij kartę prawym przyciskiem**, by zobaczyć menu karty.
- **Kliknij kartę środkowym przyciskiem**, by ją zamknąć, tak jak jej `x`.
- **Kliknij `+`** za ostatnią kartą, by otworzyć menu: Nowa karta (`Alt+T`) oraz, gdy ich karty są zamknięte, Radar i Obszary robocze (`Alt+J`).
- **Kliknij ikonę domu**, by przejść do karty Home.
- **Kliknij kartę Radar lub Obszary robocze**, by ją otworzyć; jej `x`, kliknięcie środkowym przyciskiem albo `Ctrl+W` ją zamyka. **Kliknij plakietkę obszaru roboczego** (`Alt+W`), by przełączyć obszar roboczy.
- **Kliknij Zakończ i cd tutaj** (widoczne po prawej stronie kart, gdy GitPoulpi uruchomiono przez nakładkę powłoki `gp`), by zakończyć program i zostawić powłokę w folderze aktywnej karty. Zob. [Podążanie za folderem aktywnej karty](following-the-tab-folder.md).
- **Kliknij przycisk paska narzędzi**, by go uruchomić. **Kliknij `▾`** obok Pull, Push lub
  LFS, by zobaczyć jego listę. **Kliknij `⋯`**, by zobaczyć przyciski, które się nie zmieściły.
- **Kliknij listę rozwijaną profilu** na prawym końcu paska narzędzi, by wybrać
  tożsamość commita dla tego repozytorium.

## Karta Home

- **Kliknij** wiersz, by go zaznaczyć. **Kliknij dwukrotnie** go, by otworzyć repozytorium.
- **Kliknij gwiazdkę**, by dodać lub usunąć ulubione.
- **Kliknij Otwórz, Przełącz, Zamknij lub Usuń** w wierszu albo **`⋮`**, by zobaczyć menu
  wiersza. **Kliknij prawym przyciskiem** wiersz, by zobaczyć to samo menu.
- **Kliknij tytuł sekcji**, by zwinąć sekcję.
- **Kliknij Przeglądaj, Klonuj, Init, Zwiń wszystko lub Rozwiń wszystko** u góry oraz
  **Zamknij wszystkie karty** lub **Usuń wszystkie** na tytule sekcji.

W selektorze folderów **Przeglądaj**: kliknij miejsce lub folder, by tam przejść,
kliknij dwukrotnie folder, by go otworzyć, kliknij pole ścieżki, by wpisać ścieżkę. Zaznacz
**Pokaż foldery sąsiednie** (`Alt+S`, zapamiętywane), a lewe drzewo wypisze też
pozostałe foldery każdego otwartego poziomu: foldery na otwartej ścieżce mają `▾`,
pozostałe `▸`, a kliknięcie jednego z nich tam przechodzi. W
**Klonuj** i **Init**: kliknij pole, by w nim pisać, i kliknij przycisk lub
pole wyboru, by z nich skorzystać.

## Panel boczny Refs

- **Kliknij** wiersz, by go zaznaczyć. **Kliknij dwukrotnie** lokalną gałąź lub tag, by zrobić
  checkout. **Kliknij dwukrotnie** worktree lub submoduł, by otworzyć go w
  karcie.
- **Najedź** na wiersz, by pokazać ikonę po jego lewej stronie. **Kliknij** ją, by ukryć ref
  w Logu lub pokazać go ponownie. Gdy **Solo** jest włączone, to samo miejsce włącza solo
  dla refa. W wierszu folderu ikona działa na każdy ref w folderze.
- **Kliknij Zakończ solo** lub **Pokaż wszystko** w tytule panelu bocznego, by wyczyścić wszystkie
  solo lub wszystkie ukryte refy.
- **Kliknij prawym przyciskiem** wiersz, by zobaczyć jego menu: checkout, merge, fast-forward, rebase,
  utworzenie worktree, zmiana nazwy, usunięcie, ukrycie, solo i więcej. Własny wiersz zdalnego repozytorium
  ma **Fetch**, **Edytuj**, **Usuń** i inne akcje zdalnego repozytorium. Stash ma
  **Pop**, **Zastosuj** i **Drop**. Wyszarzony wiersz mówi, dlaczego nie można go uruchomić.
- **Kliknij `+`** na tytule ZDALNE, by dodać zdalne repozytorium.
- **Kliknij wiersz folderu**, by zwinąć lub rozwinąć folder.

### Pull requesty i zgłoszenia

| Gest | Akcja |
|---|---|
| Kliknięcie wiersza | Zaznacza go. |
| Podwójne kliknięcie wiersza | Otwiera go w przeglądarce. |
| Kliknięcie wiersza prawym przyciskiem | Jego menu: otwórz, skopiuj URL lub numer, zrób checkout gałęzi źródłowej, utwórz gałąź ze zgłoszenia. |
| Kliknięcie `+` na tytule sekcji | Otwiera formularz **Utwórz pull request** albo **Nowe zgłoszenie**. |
| Kliknięcie wiersza filtra (`Otwarte ▾`) | Wybiera, które żądania lub zgłoszenia wypisać. |
| Kliknięcie tytułu grupy | Zwija lub rozwija grupę. |
| Kliknięcie wiersza dostawcy lub repozytorium (ZGŁOSZENIA) | Wybiera źródło zgłoszeń. **Brak (odłącz to repozytorium)** je usuwa. |
| Kliknięcie wiersza zapytania (ZGŁOSZENIA) | Wpisz zapytanie JQL lub WIQL (Jira, Azure DevOps) albo wybierz listy Trello. |
| Kliknięcie chipa na pasku narzędzi (`GH #12`, `GL !53`, ...) | Otwiera pull request w przeglądarce. Kliknij go prawym przyciskiem, by zobaczyć **Otwórz w przeglądarce**, **Kopiuj URL**, **Odśwież** i **Zmień profil**. |
| Kliknięcie przygaszonego chipa `+ PR` | Zaczyna pull request dla wypchniętej gałęzi. |

Zob. [Hosting i trackery zgłoszeń](hosting-integration.md).

## Log

- **Kliknij** commit, by go zaznaczyć. Jego szczegóły pojawiają się w prawym panelu.
- **Ctrl+klik** dodaje commit do zaznaczenia lub go usuwa.
  **Shift+klik** zaznacza zakres (**Alt+klik** w terminalach, które zachowują Shift+klik dla
  własnego zaznaczania tekstu, takich jak kitty i Windows Terminal). Kliknij zaznaczony commit
  prawym przyciskiem, by zrobić cherry-pick wszystkich naraz. Zaznaczenie może obejmować wiersz
  WIP i stashe; dwa lub więcej wierszy zamienia prawy panel w porównanie, zob.
  [Porównywanie commitów](log-and-history.md#porównywanie-commitów).
- **Kliknij kafelek** panelu porównania, by przenieść kursor Logu do tego wiersza;
  **kliknij Zamień**, by zamienić końce Od i Do. Zwykłe kliknięcie wiersza opuszcza porównanie.
- **Kliknij wiersz WIP**, by zobaczyć zmiany robocze. **Kliknij jego tekst `// WIP`**,
  by wpisać tam podsumowanie commita.
- **Najedź** na komórkę, by zobaczyć pełny tekst: opis, autora i e-mail, pełną
  datę, pełny hash.
- **Najedź na plakietkę `+N`**, by zobaczyć pozostałe refy na commicie. Wejdź w okno
  podręczne, by kliknąć tam ref. Kliknij dwukrotnie ref w oknie, by zrobić jego checkout.
- **Kliknij dwukrotnie plakietkę refa**, by zrobić jej checkout.
- **Kliknij commit prawym przyciskiem**, by zobaczyć menu commita. Ma jedno podmenu dla każdego refa
  na commicie.
- **Kliknij plakietkę refa prawym przyciskiem**, by zobaczyć jedno menu z akcjami refa i
  akcjami commita.
- **Kliknij nagłówek prawym przyciskiem**, by pokazać lub ukryć kolumny: lista kontrolna (`[x]` / `[ ]`) z ✕ w tytule; kliknij wiersz albo użyj strzałek i `Spacji` / `Enter`, potem zamknij ją przez ✕, `Esc` lub kliknięcie poza nią (paleta: `Widok: Log: Wybierz kolumny do pokazania`). Pod polami wyboru: `Dopasuj <kolumna> do zawartości` (kolumna kliknięta prawym przyciskiem), `Dopasuj wszystkie kolumny do zawartości` i `Zresetuj szerokości kolumn`.
- **Kliknij `↑`, `↓` lub `✕`** na pasku wyszukiwania, by przejść do poprzedniego lub następnego
  trafienia albo zamknąć pasek.
- **Kliknij `#12`, `ABC-12` lub `AB#12`** w opisie commita, w prawym panelu,
  by otworzyć to zgłoszenie lub pull request w przeglądarce.

### Sekcje panelu bocznego

**Foldery** (gałęzie i tagi nazwane `a/b/c`, pliki prawego panelu w widoku drzewa)
zwija się i rozwija naraz: **kliknij prawym przyciskiem nagłówek lub puste miejsce
sekcji**, która ma foldery, aby wybrać **Rozwiń wszystko** / **Zwiń wszystko** (oraz
**Pokaż/ukryj sekcje...**, które prowadzi do listy kontrolnej niżej), a **prawym
przyciskiem wiersz folderu**, aby wybrać **Rozwiń wszystkie podfoldery** / **Zwiń
wszystkie podfoldery** (folder i wszystko pod nim). Klawisze, w panelu Refs, na
listach Unstaged i Staged oraz na liście zmienionych plików commita: `=` rozwija
wszystko, `-` zwija wszystko, `*` (albo `Shift+Right`) rozwija folder pod kursorem
i wszystko pod nim, `_` (albo `Shift+Left`) go zwija. Paleta: *Refs: Rozwiń
wszystkie foldery*, *Refs: Zwiń wszystkie foldery*, *Refs: Rozwiń zaznaczony folder
i jego podfoldery*, *Refs: Zwiń zaznaczony folder i jego podfoldery* oraz cztery
bliźniacze *Pliki:* dla aktywnej listy plików. Gdy aktywny jest filtr nazwy (filtr
Refs, *Filtruj pliki* commita), każdy folder z trafieniem jest pokazany jako
otwarty, cokolwiek zwinąłeś, a twoje własne zwinięcia nie są ani zmieniane, ani
zapisywane: wyczyść filtr, a wrócą. Zwinięcie zrobione w czasie działania filtra
działa na to, co widać, i jest zapominane razem z filtrem.

**Kliknij prawym przyciskiem nagłówek sekcji** (sekcji bez folderów albo puste miejsce panelu bocznego), by zobaczyć listę kontrolną jego
sekcji: LOKALNE, ZDALNE, WORKTREES, Stashe, Tagi, PULL REQUESTS, ZGŁOSZENIA i SUBMODUŁY. Kliknięcie odznacza lub zaznacza sekcję;
menu zostaje otwarte. Ukryta sekcja nie jest rysowana, a klawiatura ją pomija. Ostatniej widocznej sekcji nie można
odznaczyć. Wybór jest zapamiętywany osobno dla każdego repozytorium, także między uruchomieniami; repozytorium bez własnego wyboru stosuje ustawienie domyślne (globalne `ui.sidebar_sections_hidden`). *Użyj we wszystkich repozytoriach* czyni bieżący wybór domyślnym, a *Przywróć domyślne* odrzuca własny wybór tego repozytorium (paleta: *Panel boczny: Użyj tych sekcji we wszystkich repozytoriach*). Paleta:
*Panel boczny: Pokaż/ukryj sekcje...*, po jednym poleceniu *Pokaż/ukryj* na sekcję, *Panel boczny: Przywróć domyślne sekcje*.

### Przeciąganie refów

Naciśnij gałąź w panelu bocznym Refs albo plakietkę refa w Logu i przeciągnij
ją na inną gałąź. Stopka mówi, co niesiesz. Cel, który może
przyjąć upuszczenie, się podświetla. Puść, by zobaczyć menu. `Esc` anuluje przeciąganie.

| Przeciągnięcie | Menu |
|---|---|
| Lokalna gałąź X na lokalną gałąź Y | **Merge X do Y**, **Rebase X na Y**, **Interaktywny rebase X na Y...**, **Fast-forward Y do X** |
| Lokalna gałąź na zdalną gałąź | **Push X** do tego zdalnego repozytorium |
| Zdalna gałąź na lokalną gałąź Y | **Merge**, **Rebase Y na nią**, **Fast-forward Y do niej** oraz **Pull**, gdy jest upstreamem Y |

Gdy akcja zmienia gałąź, która nie jest wybrana, GitPoulpi najpierw robi jej
checkout. Jeśli git odmówi checkoutu, nic się nie zmienia. Tagów nie można
przeciągać i nie mogą przyjmować upuszczenia.

Klawiatura: „oznacz, potem upuść” działa jak przeciągnięcie, dla dowolnej pary
refów. W panelu Refs naciśnij `m` na gałęzi (paleta: `Refs: Oznacz jako źródło
upuszczenia`). Ref dostaje `◆` i jest podkreślony, także na swoich plakietkach
w Logu, a pasek stanu mówi, dokąd iść. Zaznacz inny ref, w dowolnej sekcji, i
naciśnij `Shift+M` (paleta: `Refs: Upuść tutaj oznaczony ref...`): pojawia się
to samo menu co po przeciągnięciu, a jego wiersze działają dokładnie tak samo.
Oznaczenie jest zużywane, gdy menu się otworzy; `Esc` albo `m` na oznaczonym
refie usuwa je wcześniej. Polecenie palety `Refs: Upuść zaznaczony ref na
bieżącą gałąź...` otwiera menu dla zaznaczonego refa i bieżącej gałęzi.

## Prawy panel

### Zmiany robocze

- **Kliknij** plik, by otworzyć jego diff. **Kliknij ten sam plik ponownie**, by wrócić
  do Logu. Jeśli plik jest otwarty w edytorze z niezapisanymi zmianami,
  GitPoulpi pyta najpierw.
- **Najedź** na wiersz, by pokazać jego przycisk **Stage plik** lub **Unstage plik**. Wiersz
  folderu ma **Stage folder** lub **Unstage folder**.
- **Kliknij Stage wszystkie zmiany** lub **Unstage wszystkie zmiany** na tytule sekcji.
- **Kliknij tytuł sekcji** (`Pliki unstaged`, `Pliki staged`), by ją zwinąć.
- **Kliknij Ścieżka lub Drzewo**, by wybrać płaską listę lub foldery. **Kliknij Zwiń
  wszystko** lub **Rozwiń wszystko**, by zwinąć każdy folder.
- **Kliknij `[✖]`** po lewej stronie tytułu, by odrzucić wszystkie zmiany (pyta
  najpierw). **Kliknij `[✦]`** po prawej, by wyjaśnić zmiany przez AI.
- **Kliknij plik prawym przyciskiem**, by zobaczyć jego menu: stage, odrzucenie, ignorowanie, LFS, stash
  pliku, historia, blame, otwarcie w zewnętrznym programie, kopiowanie ścieżki,
  utworzenie patcha, edycja, usunięcie. Plik z konfliktem zaczyna się od wierszy
  **Rozwiąż przez ...**.
- **Kliknij wiersz folderu prawym przyciskiem**, by zobaczyć menu folderu: stage, odrzucenie, ignorowanie,
  stash, utworzenie patcha, otwarcie folderu, utworzenie w nim pliku.
- **Kliknij prawym przyciskiem puste miejsce** w Pliki unstaged, by zobaczyć **Utwórz plik**.
- **Ignoruj**, **Traktuj jako binarny** i **Traktuj jako tekstowy** (menu pliku i folderu) najpierw otwierają listę plików `.gitignore` lub `.gitattributes`, do których może trafić reguła. Kliknij wiersz i naciśnij `Enter` (lub użyj strzałek); `Esc` albo kliknięcie poza listą anuluje bez zapisu.
- **Ctrl+klik** na wierszu pliku lub folderu dodaje go do zaznaczenia lub usuwa, **Shift+klik** (**Alt+klik**) zaznacza zakres; tak samo w zmienionych plikach commitu i w widoku Wszystkie pliki. Zwykłe kliknięcie zaczyna od nowa. **Kliknięcie prawym przyciskiem zaznaczonego wiersza** otwiera menu wszystkich zaznaczonych plików.
- **Kliknięcie prawym przyciskiem pustego miejsca** listy oferuje też **Otwórz folder repozytorium**; Unstaged dodaje **Stage wszystkie zmiany** i **Odrzuć wszystkie zmiany...**, Staged dodaje **Unstage wszystkie zmiany**.
- **Konwertuj** (menu pliku, folderu i zaznaczenia) przepisuje kodowanie, zakończenia wierszy lub wcięcia plików na dysku; jedno `Ctrl+Z` cofa całą operację.

### Formularz commita

- **Kliknij** Podsumowanie lub Opis, by w nim pisać.
- **Kliknij Popraw poprzedni commit**, by włączyć lub wyłączyć amend.
- **Kliknij przycisk commita**, by zatwierdzić. Gdy przycisk jest wyszarzony, kliknięcie mówi
  dlaczego (na przykład *Zrób stage zmian, by zrobić commit*).
- **Kliknij `✦`** po prawej stronie Podsumowania, by napisać opis przez AI.
- **Kliknij `#▾`** na końcu Podsumowania, by zobaczyć listę zgłoszeń. Kliknij jedno,
  by wstawić jego odwołanie.
- **Kliknij Skomponuj commity przez AI**, by otworzyć kompozytor commitów.

### Szczegóły commita

- **Kliknij** zmieniony plik, by otworzyć jego diff. **Kliknij go ponownie**, by wrócić do
  Logu.
- **Przeciągnij** dolną krawędź opisu, by zmienić jego rozmiar. Kółko nad
  opisem przewija długi opis szczegółowy.
- **Kliknij prawym przyciskiem** podsumowanie, opis lub blok autora, by wybrać *Popraw
  podsumowanie...*, *Popraw opis...* albo *Popraw autora...*; **kliknij dwukrotnie** jedno
  z nich (albo datę autora), by otworzyć pełne okno Popraw z kursorem w tym polu.
  Odpowiedniki klawiaturowe to `Ctrl+G Shift+E` (wszystkie pola) i `Ctrl+G Shift+A` (autor);
  podsumowanie i opis są w palecie.
- **Kliknięcie prawym przyciskiem** zmienionego pliku, wiersza folderu, pustego miejsca listy lub zaznaczenia otwiera menu (otwórz, historia, blame, otwórz w programie, kopiuj ścieżki, ignoruj, traktuj jako binarny lub tekstowy, konwertuj, przywróć z commitu, usuń). **Ctrl+klik** i **Shift+klik** zaznaczają wiele plików.

Zob. [Stage i commity](staging-and-committing.md).

## Diff i widok pliku

- **Kliknij przycisk** na pasku nad diffem: Widok pliku, Widok diff, Blame,
  przełącznik kolumny `Plik │ Wszystkie`, Edytuj, `↑` `↓`, W linii, Obok siebie, Hunki, WS,
  Zawij, Mapa, Kodowanie. Włączony przycisk jest podświetlony. Kliknięcie `Plik` lub
  `Wszystkie` pokazuje tę listę w kolumnie commitów; kliknięcie segmentu, który jest już
  wybrany, ukrywa kolumnę (zob. [Kolumna commitów](interface-tour.md#kolumna-commitów)).
- **Kliknij `✕`** w tytule, by zamknąć plik.
- **Najedź na hunk**, by pokazać **Stage hunk** lub **Unstage hunk** oraz
  **Odrzuć hunk**.
- **Najedź na zmienioną linię**, by pokazać jej własne przyciski **+ Stage**, **− Unstage** i
  **✖ Odrzuć** na prawym końcu wiersza.
- **Kliknij linię prawym przyciskiem**, by zobaczyć menu hunka: stage, unstage lub odrzucenie hunka
  albo zaznaczonych linii, historia pliku, blame, kopiowanie.
- **Kliknij minimapę**, by przenieść widok do tej linii. Przytrzymaj przycisk
  i poruszaj, by przewijać.

Każde odrzucenie pyta najpierw.

### Blame i historia pliku

- **Blame.** Zatrzymaj wskaźnik na adnotacji, aby zobaczyć kartę hunka; kliknięcie wybiera hunk, prawy przycisk otwiera jego menu. Przeciągnij krawędź kolumny adnotacji, aby zmienić jej szerokość (zapisuje się). Kliknij lub przeciągnij na minimapie, aby przewijać.
- **Historia pliku.** Kliknięcie wybiera commit; `Shift+kliknięcie` wybiera zakres do porównania; dwukrotne kliknięcie pokazuje commit w Logu; prawy przycisk otwiera menu wiersza; przeciągnij krawędź między listą a diffem, aby zmienić szerokość; ✕ na liście ją ukrywa (kliknij pasek `›`, aby ją przywrócić); kliknij `Widok pliku` / `Diff` / `Blame` na pasku, aby przełączyć, co pokazuje prawa strona, a przełącznik `Plik │ Wszystkie`, aby wybrać listę kolumny. Kółko przewija połowę pod wskaźnikiem. Każde ma odpowiednik klawiaturowy ([Blame](log-and-history.md#blame), [Historia pliku](log-and-history.md#historia-pliku)).

### Kolumna commitów (lista Wszystkie)

- **Najedź na wiersz**, by zobaczyć podpowiedź Logu (cały opis, nazwę i e-mail autora, pełną datę, zależnie od części pod wskaźnikiem).
- **Kliknij wiersz prawym przyciskiem**, by zobaczyć jego menu: pokazanie go w Logu, diff pliku w tym commicie, plik w tym commicie, jego blame, a potem menu Logu dla commita. Klawisz menu robi to samo z klawiatury.
- **Przeciągnij granicę w nagłówku** (`ui.column_resize`: domyślnie jednokomórkowy separator między dwiema częściami, oznaczony przygaszonym `│` w nagłówku i rysowany jako linia akcentu w dół kolumny, gdy wskaźnik jest na nim, a przeciągnięcie z dowolnego miejsca wzdłuż niego zmienia szerokość; przy `header` pierwsza litera następnej nazwy, świecąca, gdy wskaźnik jest na niej), by ustawić szerokość części po jej lewej: torów, opisu, autora, daty, refów, zmian lub SHA. Komórka tuż za ostatnią częścią jest granicą tej ostatniej części.
- **Przeciągnij nazwę w nagłówku** nad inną, by przenieść tam część: ten sam duszek, przyciemnione źródło i pasek akcentu co w nagłówku Logu, a część przesuwa się po puszczeniu (`Esc` anuluje; `<` / `>` na klawiaturze, po `Tab`, robią to samo o jedno miejsce naraz).
- **Kliknij prawym przyciskiem w nagłówku**, by wybrać pokazywane części (Ref, Graf, Autor, Data, Zmiany, SHA; Opis jest zawsze widoczny); lista kontrolna ma ✕ w tytule. Paleta ma `Wybierz kolumny` i przełącznik dla każdej części.
- **Shift+kółko** albo poziome kółko przewija wiersze na boki, gdy są szersze niż kolumna.
- **Przeciągnij prawą krawędź** kolumny, by zmienić jej szerokość. **Kliknij pasek `›`** (albo ✕ na kolumnie, albo **kliknij wybrany segment** przełącznika `Plik │ Wszystkie` na pasku), by ją przywrócić lub ukryć.

### Zaznaczanie i kopiowanie tekstu

Można zaznaczać i kopiować tekst diffa, widoku pliku, Blame i
Historii pliku.

- **Naciśnij i przeciągnij**, by zaznaczyć tekst. Zaznaczenie zostaje na tym samym tekście,
  gdy przewijasz.
- **Kliknij dwukrotnie**, by zaznaczyć słowo.
- **Zwykłe kliknięcie** nie zaznacza tekstu. Przesuwa kursor lub zaznacza
  wiersz i czyści zaznaczenie tekstu.
- **`y` lub `Ctrl+C`** kopiuje zaznaczenie. Bez zaznaczenia kopiują
  bieżącą linię.
- **Kliknij prawym przyciskiem** w diffie, by zobaczyć **Kopiuj zaznaczenie**, **Kopiuj linię** i
  **Kopiuj hunk**.

GitPoulpi kopiuje tylko tekst pliku. Numery linii, znaki `+` i `-`, nagłówki
hunków i kolumny Blame nie są kopiowane. W układzie Obok siebie zaznaczenie zostaje
po stronie, na której zaczęto przeciąganie.

W diffie przeciągnięcie zaznacza też linie, które przecina, tak jak `v`. `Enter`
dodaje te linie do stage. `Esc` najpierw czyści zaznaczenie tekstu, potem zaznaczenie linii,
potem zamyka diff.

Zaznaczenie tekstu podświetla też każde inne miejsce, w którym ten tekst występuje w
diffie i edytorze.

## Pola tekstowe

Dotyczy to każdego pola tekstowego: filtrów, paska wyszukiwania, palety,
Podsumowania i Opisu, pól okien dialogowych i wartości Ustawień.

- **Kliknij**, by ustawić tam kursor.
- **Naciśnij i przeciągnij**, by zaznaczyć tekst.
- **Kliknij dwukrotnie**, by zaznaczyć słowo.
- Pisz, by zastąpić zaznaczenie. `Backspace` lub `Delete` je usuwa.

Klawisze pól tekstowych są w [Skrótach klawiszowych](keybindings.md).

## Edytor

- **Kliknij**, by ustawić tam kursor. **Shift+klik** rozszerza zaznaczenie.
- **Przeciągnij**, by zaznaczyć. Przeciągnij poza górną lub dolną krawędź, by przewijać.
- **Kliknij dwukrotnie**, by zaznaczyć słowo.
- **Kółko** przewija bez przesuwania kursora.
- **Kliknij Zapisz, Cofnij, Ponów lub Szukaj** na pasku. W trybie szukania **`↑`** i
  **`↓`** przechodzą do poprzedniego i następnego trafienia.
- **Kliknij `✕`**, by zamknąć. GitPoulpi pyta najpierw, gdy są niezapisane
  zmiany.

## Obrazy

Mysz w widoku obrazu opisują [Obrazy](image-view.md).

## Przesuwanie i zmiana rozmiaru okien dialogowych

Okna dialogowe można przesuwać, a większe okna także zmieniać ich rozmiar:

- **Przeciągnij pasek tytułu**, aby przesunąć okno (krzyżyk zamknięcia zachowuje swoje kliknięcie). Paleta poleceń i selektory nie mają osobnego paska tytułu: przeciągnij górną krawędź.
- **Przeciągnij lewą, prawą lub dolną krawędź** albo dolny róg (uchwyt `◢`), aby zmienić rozmiar okna. Okno zostaje na ekranie i nigdy nie robi się mniejsze niż jego minimum (60 x 16 dla Historii cofania, mniej dla pozostałych). Okno o zmienionym rozmiarze wykorzystuje miejsce: pola tekstowe i listy rosną, a tekst zawija się do nowej szerokości.
- Przeciąganie trwa dalej, gdy wskaźnik opuści okno, a puszczenie przycisku poza oknem nigdy go nie zamyka.
- **Przeciągnij separatory** wewnątrz widoku Reflog / Utracona praca w Historii cofania; dwukrotne kliknięcie przywraca separator.
- Odpowiedniki z klawiatury to `Ctrl+N`, potem `m` (strzałki przesuwają, `Shift`+strzałki zmieniają rozmiar, `0` przywraca, `Enter` lub `Esc` kończą) oraz `Alt`+strzałki dla separatorów; paleta ma `Okno: Przesuń lub zmień rozmiar` i `Okno: Przywróć położenie i rozmiar`. Działają na okno na wierzchu. Powierzchnie, które używają `Ctrl+N` do własnych znaczników (formularz opisu commita, strona przepisywania historii), przyjmują to samo `m` po swoim `Ctrl+N`, a pasek stanu je podaje. Paleta poleceń i selektory używają `Ctrl+N` do przesuwania w dół listy, więc dla nich klawiszem jest `Alt+G` (`palette.arrange`) albo własny wiersz palety `Okno: Przesuń lub zmień rozmiar`: oba ustawiają okno na wierzchu i zostawiają je otwarte.
- **Jedno ustawienie** (`ui.dialog_geometry`, Ustawienia > Wygląd > Układ) obejmuje wszystkie okna: każde pamięta własne miejsce i rozmiar na tę sesję, na dysku do następnego uruchomienia albo wcale.

| Okno | Przesuwanie | Zmiana rozmiaru |
|---|---|---|
| Historia cofania / Reflog / Utracona praca, Dziennik aktywności, panel zadań, wybór folderu | tak | tak |
| Paleta poleceń, selektory kart / motywu / języka, wyszukiwanie w obszarze roboczym | tak | tak (lista rośnie z wysokością, wiersze z szerokością) |
| Clone (lista repozytoriów), Pull request i Nowe zgłoszenie (opis), „Co jest wysyłane” (panel), Mermaid (podgląd), opcje przeglądu (pola, tylko szerokość) | tak | tak |
| Edytory długich tekstów w Ustawieniach (szablony, hooki, prompty AI), pole AI formularza obszaru roboczego | tak | tak (pole tekstowe) |
| Przepisywanie historii: dwa dialogi pomocnika AI | tak | tak (opis, panel wysyłanych danych) |
| O programie, panel aktualizacji, kontrola bezpieczeństwa, przewidywane konflikty, „Przekroczono limit AI”, usunięcie tagu, pliki wrażliwe | tak | nie (rozmiar wynika z tekstu) |
| Init, Worktree, Sparse checkout, dialog Flow, dialogi przepływu zgłoszeń, formularz opisu commita (reword), formularz obszaru roboczego, selektory etykiet / kamieni milowych | tak | nie |
| Potwierdzenia (każde „Na pewno?”) | tak | nie |
| Dialogi Przeglądu i Tablicy: ich małe dialogi (komentarz, zakończenie, wybór, kolumny) | tak | nie |
| Okna strony Klawiatura i mysz (potwierdzenie resetu, przechwycenie klawisza), okno odpowiedzi AI | tak | nie |
| Ustawienia: wyniki i potwierdzenie uwierzytelniania repozytorium | tak | nie |
| Przepisywanie historii: dialogi czyszczenia, tożsamości i pytań | tak | nie |
| Okno przenoszenia pliku w edytorze commita, komunikaty merge | tak | nie |

Nie można przesuwać: ekranów pełnostronicowych (Ustawienia, Pomoc, strony Przeglądu i Tablicy, które już wypełniają okno), menu kontekstowych i list rozwijanych (wiszą przy elemencie, który je otworzył), podpowiedzi oraz małych okien wewnątrz strony (formularz zdalnych repozytoriów, edytor dzielenia commita).

Wskaźnik ma kształt przesuwania nad paskiem tytułu i kształt zmiany rozmiaru nad krawędziami i rogami (terminale z `OSC 22`).

## Kształt wskaźnika myszy

Niektóre terminale pozwalają GitPoulpi zmienić wskaźnik myszy: wskaźnik zmiany rozmiaru nad
separatorami, nad granicą kolumny w Logu, Radarze i kolumnie commitów
(także podczas jej przeciągania), rękę nad klikalnymi elementami. Działa to w kitty,
foot, Ghostty, WezTerm i nowszym xterm (sekwencja `OSC 22`). Jest wysyłany
tylko wtedy, gdy kształt się zmienia, nigdy w tmux, screen, zellij ani herdr,
a domyślny wskaźnik wraca, gdy GitPoulpi się kończy.
W innych terminalach wskaźnik się nie zmienia. Separator lub przycisk
pod wskaźnikiem podświetla się w każdym terminalu.

Ten sam rodzaj komponentu daje wszędzie w aplikacji ten sam wskaźnik.

- **Wskazująca ręka** jest na kontrolkach: przyciskach, polach wyboru i
  przełącznikach, polach kombi (w Ustawieniach, panelach zgłoszeń i pull requestów,
  Radarze, oknach dialogowych), kartach i ich krzyżykach, przyciskach `+`,
  linkach i odwołaniach (`#12`, `!45`), nagłówkach grup i folderów ze strzałką
  rozwijania, pigułkach ref, paskach przewijania oraz przyciskach Aktywność,
  zadania i wersja na pasku stanu. Pole wyboru lub przycisk wewnątrz wiersza
  zachowuje rękę.
- **Strzałka** jest na wierszach. Wiersz podświetla się w całości, gdy wskaźnik
  jest na nim (tym samym światłem w każdej liście), i sam nie jest kontrolką:
  wiersze paneli Refs, Worktrees, Stashes i Submodules, Log i kolumna commitów
  (rękę bierze tylko pigułka ref), linie diffa, zmienione pliki, elementy pull
  requestów, zgłoszeń i Radaru oraz wiersze list w oknach dialogowych i panelach
  (klonowanie, selektor folderów, Workspaces, karty tablicy, panel Jobs, selektory
  osoby przypisanej, etykiet i kamieni milowych, kroki rebase, pliki i commity
  przeglądu, wybory w menu i polach kombi).
- **Kursor tekstowy (I-beam)** jest na polach wprowadzania, nad całym polem: pola
  wyszukiwania (globalne, Ustawienia, paleta poleceń, Home), filtry (Refs, pliki,
  Radar, Dziennik aktywności, wyszukiwanie w panelu zgłoszeń), wiersze tekstowe,
  liczbowe i obszarów tekstowych w Ustawieniach (także podczas edycji), pola
  okien dialogowych, pola komentarzy przeglądu, kompozytor commitów, tekst edytora
  plików i edytowalny wynik edytora merge (jego panele tylko do odczytu zachowują
  strzałkę). W szczegółach commita opis i treść są edytowalne, więc I-beam
  pokazuje się na nich, a cały blok lekko się podświetla.
- **W szczegółach commita** linie informacyjne są kontrolkami: kliknij
  `commit: <id>`, by skopiować krótki identyfikator, kliknij dwukrotnie linię
  autora lub daty, by ją poprawić (ręka i lekki znacznik je oznaczają). Linia
  statystyk tylko informuje i zachowuje strzałkę.

Wskaźniki zmiany rozmiaru zostają nad separatorami i granicami kolumn, a otwarta
dłoń pokazuje się tylko wtedy, gdy niesiesz ref.

## Kopiowanie do schowka

**Kopiuj ścieżkę** i niektóre inne akcje kopiowania wysyłają tekst do terminala, który
umieszcza go w schowku (OSC 52). Jeśli twój terminal tego nie obsługuje,
nic nie zostaje skopiowane. Akcje kopiowania zaznaczonego tekstu używają bezpośrednio
schowka systemowego.
