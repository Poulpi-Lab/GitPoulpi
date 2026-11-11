[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · **Polski** · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Cofnij i ponów

**Zawsze można wrócić.** Każda zmiana, którą GitPoulpi wprowadza w twoim
repozytorium, jest zapisywana (domyślnie do 100 akcji, `general.undo_depth`),
tak samo jak każda pozycja, na której była gałąź, także po ruchach wykonanych
przez inne programy. Jeśli popełnisz błąd, jedno z poniższych przywróci pracę:

| Chcesz | Użyj |
|---|---|
| cofnąć ostatnią rzecz | `Ctrl+Z` (ponów: `Ctrl+Y` lub `Ctrl+Shift+Z`) albo przycisków **Cofnij** i **Ponów** na pasku narzędzi |
| cofnąć kilka rzeczy naraz albo jedną dawną | **Historii cofania** (`Alt+Z`, strzałka obok Cofnij, `Edycja: Historia cofania...`) |
| odzyskać gałąź, tag, stash, plik lub hunk usunięty kilka akcji temu, **bez cofania tego, co zrobiono potem** | **Przywróć tylko tę** w Historii cofania |
| przesunąć gałąź tam, gdzie była, nawet jeśli przesunął ją inny program | **Reflog** gałęzi (menu refa, `Reflog...`) |
| znaleźć commity, do których nie sięga już żadna gałąź | **Utracona praca** (`Edycja: Odzyskaj utracone commity...`) |
| cofnąć to, co powłoka lub **agent AI** zrobił z gałęziami, tagami, stashami lub HEAD | to samo `Ctrl+Z`: te zmiany też są w historii, oznaczone jako **zewnętrzne** (zob. [Zmiany wykonane poza GitPoulpi](#zmiany-wykonane-poza-gitpoulpi)) |

Po każdej akcji, która coś niszczy i da się ją cofnąć (odrzucenie zmian,
usunięcie pliku, gałęzi lub tagu, hard reset, porzucenie stasha, usunięcie
zdalnego repozytorium), pasek stanu to mówi: „odrzucenie zmian: gotowe -
Ctrl+Z, aby cofnąć”, z klawiszem takim, jaki masz przypisany. Podpowiedź
przycisku Cofnij mówi, ile akcji można cofnąć.

`Ctrl+Z` cofa ostatnią rzecz, jaką GitPoulpi zrobił w twoim repozytorium.
`Ctrl+Y` (lub `Ctrl+Shift+Z`) przywraca ją z powrotem. Na pasku narzędzi są
przyciski **Cofnij** i **Ponów**, paleta ma `Edycja: Cofnij` i `Edycja: Ponów`,
a podpowiedź każdego przycisku mówi, co by cofnął („Cofnij Commit "fix typo"
(a81b9e4)”).

![Pasek narzędzi po Stage wszystkie zmiany: Cofnij jest podświetlone, a jego podpowiedź nazywa operację, którą by cofnął](../images/undo-toolbar.png)

Ponów nigdy nie uruchamia operacji od nowa. Przywraca zapisany stan,
więc ponowiony commit jest dokładnie tym samym commitem, z tym samym
identyfikatorem i tą samą datą.

`Ctrl+Z` nie działa, gdy klawiaturę przejęło pole tekstowe (opis commita,
pole filtra, edytor, który ma własne cofanie tekstu).

## Historia cofania

`Alt+Z` (można zmienić), strzałka obok przycisku Cofnij albo `Edycja: Historia
cofania...` otwiera panel ze wszystkimi zapisanymi akcjami, od najnowszej:
ikona rodzaju akcji (commit, gałąź, tag, stash, reset, merge i podobne,
odrzucenie zmian, poczekalnia, konfiguracja, zmiana zewnętrzna), co zrobiła w
twoim języku, jak dawno i w jakim jest stanie (**do osobnego przywrócenia**,
**zablokowana** z powodem albo **cofnięta** i gotowa do ponowienia). Gdy wpisy
obejmują kilka dni, cienki nagłówek (Dzisiaj, Wczoraj, potem data) zaczyna
każdy dzień. Nagłówek panelu je zlicza („Można cofnąć 12 akcji (do 100)”).

Akcje, które cofnięto, leżą **nad** najnowszą akcją, pod linią podziału
(„cofnięte - Ponów je przywróci”), przyciemnione, a ta, którą Ponów przywróci
jako następną, jest najbliżej linii. Zaznaczenie zaczyna się na najnowszej
akcji, nie na nich. Strzałki przesuwają zaznaczenie (nagłówki dni i linia
podziału są pomijane), `1` / `2` / `3` (albo `Ctrl+Tab` / `Ctrl+Shift+Tab`)
przełączają trzy widoki, `Esc` zamyka. `Enter` nigdy nie uruchamia akcji: każdy
przycisk ma własną literę, widoczną w wierszu podpowiedzi pod przyciskami. Dla
zaznaczonego wiersza:

- **Cofnij do tego miejsca** (`c`): cofa tę akcję i każdą nowszą, od
  najnowszej, po jednym potwierdzeniu, które je wszystkie wymienia. Krok, który
  demon uzna za ryzykowny, pyta jeszcze raz, zanim się wykona.
- **Przywróć tylko tę** (`r`): przywraca to, co usunęła ta jedna akcja, i
  zostawia wszystko, co zrobiono po niej. Usunięta gałąź lub tag wraca na
  commit, który miała (jeśli jej nazwa jest teraz zajęta, program prosi o inną i
  podpowiada wolną); porzucony stash trafia na wierzch listy stashy; odrzucony
  lub usunięty plik jest zapisywany z powrotem na miejscu, jeśli od tamtej pory
  nie był ruszany, w przeciwnym razie obok jako `name.restored` (nic nie jest
  nigdy nadpisywane); usunięte zdalne repozytorium, upstream, wiersz ignorowania
  lub wiersz atrybutów zostaje dodane ponownie. Reset, amend, rebase, merge lub
  commit dostają gałąź `restore/<branch>-<sha>` na dawnym końcu - twoja bieżąca
  gałąź i pliki nigdy się nie przesuwają. Potwierdzenie pokazuje dokładnie, co
  się stanie, a przywrócenie jest nową akcją: `Ctrl+Z` je cofa.
- **Ponów do tego miejsca** (`p`) na cofniętym wierszu.
- **Przywróć main tam, gdzie zostawiła ją akcja Commit** (`o`, albo
  `Alt+Shift+Z` / `Edycja: Przywróć gałąź tam, gdzie zostawiła ją zablokowana
  akcja` spoza panelu): gdy inny program przesunął gałąź i zablokował Cofnij,
  reflog nadal wie, gdzie akcja ją zostawiła.

Kliknij wiersz prawym przyciskiem (albo naciśnij `.`), by zobaczyć te same
akcje jako menu.

![Historia cofania po usunięciu gałęzi i zrobieniu dwóch commitów: wiersz usuniętej gałęzi jest zaznaczony i można go przywrócić osobno](../images/undo-history.png)

## Reflog

Menu refa gałęzi lokalnej ma **Reflog...**; `Edycja: Reflog...` pokazuje reflog
HEAD, a Historia cofania ma na niego kartę. Lista zawiera pozycje, na których
była gałąź, od najnowszej: krótki SHA, zapisany powód, jak dawno i kto - także
ruchy wykonane w powłoce. Opis wybranej pozycji i zmienione w niej pliki widać
obok listy.

Reflog jest czytany po 200 pozycji naraz. Dłuższy doczytuje się sam: przesunięcie
kursora (albo przewijanie) blisko końca listy pobiera następną stronę, a ostatni
wiersz brzmi **... Wczytaj więcej**, dopóki jest co wczytać. **Wczytaj więcej**
(`w`, albo `Reflog: Wczytaj więcej` w palecie) pobiera ją od razu i pisze na pasku
stanu „wczytane pozycje: N”. Repozytoria w formacie reftable gita są czytane tak
samo jak inne.

![Reflog gałęzi main: każda pozycja, na której była, od najnowszej, z opisem zaznaczonej po prawej](../images/reflog.png)

Są też **Pokaż w Log** (`l`), **Utwórz gałąź tutaj** (`u`), **Utwórz tag tutaj**
(`t`), **Zastosuj w drzewie roboczym** (`z`), **Zapisz jako stash** (`s`) (trzy
ostatnie działają tak jak w Utraconej pracy niżej) oraz **Przywróć
`<branch>` do tej pozycji** (`r`). Przywrócenie można cofnąć. Gdy gałąź jest
bieżąca, a masz lokalne zmiany, których nie może ominąć bez nadpisania, lista
rozwijana pyta, co z nimi zrobić: zachować moje zmiany, soft, mixed albo hard.
Hard odrzuca zmiany w plikach, ale najpierw je zapisuje, więc `Ctrl+Z` nadal je
przywraca (mówi o tym na czerwono).

## Utracona praca

`Edycja: Odzyskaj utracone commity...` (także w menu commita w Logu) znajduje
commity, do których nie sięga żadna gałąź, tag ani stash: te, które nadal
wymienia reflog (commity usuniętej gałęzi), te, które trzyma tylko pin cofania,
i osierocone. Lista pojawia się od razu, a wolniejsze szukanie osieroconych
commitów uzupełnia ją, gdy kręci się wskaźnik; nigdy nie blokuje interfejsu.

![Utracona praca: dwa commity usuniętej gałęzi (znalezione przez reflog) i commity osierocone](../images/lost-work.png)

**Utwórz gałąź tutaj** (`u`) odzyskuje commit pod wybraną nazwą, **Utwórz tag
tutaj** (`t`) przypina go tagiem (domyślna nazwa `restored-<krótki id>`),
**Cherry-pick** (`c`) stosuje go na bieżącej gałęzi. Utracony commit nie jest w
Log, więc ten widok nie ma „Pokaż w Log” (widok Reflog zachowuje je dla commitów,
które nadal są osiągalne).

**Przywróć zmiany** oddaje to, co zmienił commit, bez commitowania:
**Zastosuj w drzewie roboczym** (`z`) odtwarza zmiany commita w plikach, wszystko
albo nic (indeks i HEAD zostają; merge commit jest odrzucany). Gdy nie da się
tego zastosować czysto, nic nie jest ruszane, a okno proponuje **Zapisz jako
stash** (`s`), który zamiast tego zachowuje te same zmiany jako nowy stash; ta
akcja jest też zawsze dostępna osobno. Obie są zapisywane w historii: `Ctrl+Z`
przywraca pliki albo znów porzuca stash. Te same akcje to `gitpoulpi apply-commit
<rev>` i `gitpoulpi stash from-commit <rev> [-m M]`, a widok Reflog też je ma.

### Panele, hunki i okno

Reflog i Utracona praca pokazują listę commitów po lewej, a po prawej nagłówek
wybranego commita, **każdy zmieniony przez niego plik** i hunki wybranego pliku
(tylko hunki, zawijane do szerokości panelu). Lista plików jest taka jak w prawym
panelu głównego okna: `Ścieżki` lub `Drzewo` (`v`, albo kliknij przełącznik),
folder zwija się klawiszami `Enter`, `Left` lub `Right`, kliknięcie albo strzałki
wybierają plik, a jego hunki pojawiają się niżej (demon jest pytany poza wątkiem
interfejsu; kilka ostatnich plików jest pamiętanych). Lista commitów, lista
plików i hunki mają każde pasek przewijania, który można nacisnąć i przeciągnąć.

- `Tab` / `Shift+Tab` przechodzą między panelami (lista, pliki, hunki); strzałki,
  `PageUp`, `PageDown`, `Home` i `End` działają na panelu, który ma klawiaturę
  (jego ramka jest podświetlona). `Enter` na pliku przenosi do jego hunków.
- **Separatory**: przeciągnij ten między listą a prawym panelem albo ten między
  plikami a hunkami; dwukrotne kliknięcie separatora przywraca go. Klawiatura:
  `Alt+Left` / `Alt+Right`, `Alt+Up` / `Alt+Down`, `Alt+0` przywraca oba. Paleta ma
  `Reflog / Utracona praca: Przywróć rozmiary paneli`, `Następny panel` i
  `Lista plików jako ścieżki lub drzewo`.
- `Ctrl+N` pokazuje swoje plakietki nad trzema kartami widoków i trzema panelami,
  tak jak w głównym oknie; klawisz plakietki przenosi tam.
- **Przesuwanie i zmiana rozmiaru okna**: przeciągnij pasek tytułu, aby je
  przesunąć, przeciągnij lewą, prawą i dolną krawędź albo dolny róg (uchwyt `◢`),
  aby zmienić rozmiar (nigdy poniżej 60 x 16, zawsze na ekranie). Krzyżyk
  zamknięcia zachowuje swoje kliknięcie, a puszczenie przycisku poza oknem nigdy
  go nie zamyka. Odpowiednik z klawiatury to `Ctrl+N`, potem `m`: strzałki
  przesuwają, `Shift`+strzałki zmieniają rozmiar, `0` przywraca, `Enter` lub `Esc`
  kończą (mówi o tym ramka, gdy tryb jest włączony). Paleta ma `Okno: Przesuń lub
  zmień rozmiar` i `Okno: Przywróć położenie i rozmiar`. To, czy miejsce jest
  zapamiętywane, ustawia `ui.dialog_geometry` (*Ustawienia > Wygląd >
  Zapamiętuj położenie i rozmiar okien dialogowych*): `session` (domyślnie),
  `saved` (przywracane przy następnym uruchomieniu) lub `never`.

## Z palety poleceń

Każda akcja wiersza w tych trzech widokach jest też wierszem palety poleceń, więc
panel nie wymaga ani myszy, ani zapamiętywania liter. Naciśnij `Ctrl+P`, gdy
panel jest otwarty: paleta otworzy się nad nim, a jej wiersze **Historia
cofania:**, **Reflog:** i **Utracona praca:** działają na zaznaczonym wierszu
widoku widocznego na ekranie.

| Widok | Wiersze palety |
|---|---|
| Historia cofania | `Historia cofania: Cofnij do tego miejsca...`, `Ponów do tego miejsca...`, `Przywróć tylko tę...`, `Przywróć gałąź tam, gdzie była...`, `Odśwież`, `Zamknij` |
| Reflog | `Reflog: Pokaż w Log`, `Utwórz gałąź tutaj...`, `Utwórz tag tutaj...`, `Zastosuj zmiany w drzewie roboczym`, `Zapisz zmiany jako stash`, `Przywróć do tej pozycji...`, `Wczytaj więcej`, `Odśwież` |
| Utracona praca | `Utracona praca: Utwórz gałąź tutaj...`, `Utwórz tag tutaj...`, `Cherry-pick`, `Zastosuj zmiany w drzewie roboczym`, `Zapisz zmiany jako stash`, `Szukaj ponownie` |

Wiersze są wyszarzone (z podaniem powodu), dopóki ich widok nie jest otwarty;
trzy wiersze otwierające, `Edycja: Historia cofania...`, `Edycja: Reflog...` i
`Edycja: Odzyskaj utracone commity...`, przełączają też już otwarty panel na swój
widok. Wiersz, którego zaznaczenie nie może wykonać, podaje powód na pasku
stanu zamiast nic nie robić: „ta akcja jest już cofnięta: użyj Ponów do tego
miejsca”, powód, dla którego wpisu nie można przywrócić osobno, „można cofnąć
tylko gałąź lub HEAD” i tak dalej. Wiersze z `...` otwierają to samo
potwierdzenie lub pole nazwy co przycisk.

## Co można cofnąć

| Obszar | Operacje |
|---|---|
| Commity | commit, amend, reword, kompozytor commitów AI (wszystkie jego commity jako jeden krok) |
| Gałęzie i tagi | tworzenie, usuwanie, zmiana nazwy, ustawienie lub usunięcie upstreamu, utworzenie gałęzi śledzącej, utworzenie / usunięcie / opisanie tagu, fast-forward, checkout pull requesta jako gałęzi |
| Przemieszczanie się | checkout gałęzi lub commita (także odłączony HEAD) |
| Historia | merge (fast-forward lub commit scalający), rebase (zwykły i interaktywny, po zakończeniu), cherry-pick, revert, reset (soft, mixed i hard), lokalna część pulla |
| Zmiany robocze | stage i unstage (plik, folder, hunk, linie, wszystko), odrzucenie (plik, folder, hunk, linie, checkout pliku), zastosowanie patcha lub zmian commita (`apply-commit`), **usunięcie pliku z dysku**, **ignorowanie** (plik, rozszerzenie lub folder: wiersz w wybranym `.gitignore`), **traktowanie jako binarny lub tekstowy** (wiersz w `.gitattributes`) |
| Stashe | push, apply, pop, drop, zapisanie zmian commita jako stasha (`stash from-commit`) |
| Zdalne repozytoria i konfiguracja | dodanie, usunięcie, zmiana nazwy, zmiana adresu URL lub push URL zdalnego repozytorium, Git LFS track i untrack |
| Na serwerze | **usunięcie gałęzi lub tagu na zdalnym repozytorium** (z menu refa, palety, `branch delete-remote`, `tag delete-remote`, `tag delete --both` albo zakończenia workflow, które usuwa gałąź zdalną): zob. niżej |

Gdy merge, cherry-pick lub revert jest zatrzymany na konfliktach, **Zapisz i oznacz jako rozwiązany**,
stage i unstage również można cofnąć (każdy z nich jest cofany
przed tym, który jest pod nim), a **Przerwij merge** (albo cherry-pick, revert) jest
odwracalne: cofnięcie przywraca całą zatrzymaną operację wraz z jej
konfliktami (trzy wersje każdego nierozwiązanego pliku), plikami już
rozwiązanymi i dodanymi do poczekalni, znacznikami konfliktów w drzewie roboczym oraz opisem
merge; ponowienie przerywa znowu. Stage i rozwiązania przestają być odwracalne, gdy
operacja się kończy (zatwierdzona lub przerwana). *Unstage* i *Unstage All* nigdy nie
dotykają pliku, który nadal jest w konflikcie: nie ma w nim zmiany w poczekalni do cofnięcia.

Wszystko, co odrzucenie lub hard reset wyrzuca, jest najpierw zachowywane, więc cofnięcie
przywraca dokładne bajty, w tym pliki nieśledzone, bity wykonywalności i dowiązania
symboliczne. Cofnięcie usuniętego stasha oddaje ten sam stash, z tym samym
identyfikatorem commita (ląduje na wierzchu listy stashy jako `stash@{0}`, nie na swojej dawnej
pozycji). Cofnięcie usunięcia zdalnego repozytorium przywraca jego gałęzie
zdalne, alias `origin/HEAD` i ustawienia upstreamu gałęzi, które
je śledziły.

### Usunięcie gałęzi lub tagu na zdalnym repozytorium

Przed usunięciem GitPoulpi pyta serwer, co gałąź lub tag trzyma w tej chwili,
i to zapisuje: zdalne repozytorium, nazwę i dokładny commit (dla tagu
z adnotacją sam obiekt tagu, więc adnotacja zostaje). Usunięcie jest potem
wykonywane tylko wtedy, gdy serwer nadal trzyma tę wartość.

**Cofnięcie wypycha ją z powrotem** pod tą samą nazwą, na ten sam commit,
nigdy bez wymuszania. Ponieważ to zmiana na serwerze, który widzą inne osoby,
Cofnij **zawsze pyta najpierw** i mówi o tym („Cofnięcie odtworzy gałąź topic
w origin na 3f2a1c0...”); kroki wpisu też to mówią (`gitpoulpi undo --dry-run`).
Używa tych samych poświadczeń co push. Razem z nią wraca gałąź zdalna
śledząca. **Ponów** usuwa ją znowu, z tym samym zabezpieczeniem.

Cofnięcie odmawia, niczego nie zmieniając, gdy:

- gałąź lub tag został od tego czasu **utworzony ponownie na zdalnym
  repozytorium** albo przesunięty: wskazuje inny commit niż zapisany, a wypchnięcie
  nadpisałoby czyjąś pracę (odmowa podaje oba commity);
- commita (lub obiektu tagu) **nie ma już w twoim repozytorium** (został usunięty
  przez garbage collection): cofnięcie nie jest możliwe.

„Usuń lokalnie i na zdalnym” to **jeden wpis**: Cofnij przywraca razem gałąź lub
tag lokalny i zdalny, Ponów usuwa oba znowu. Usunięcie tylko gałęzi zdalnej
zostawia lokalną w spokoju; jej cofnięcie robi to samo. Zakończenie workflow, które
usuwa też gałąź zdalną, można cofnąć w całości.

## Czego nie można cofnąć i dlaczego

- **Większości rzeczy, które dotyczą serwera**: push, wypchnięcia tagu, LFS push,
  lock i unlock, otwarcia pull requesta. Ten stan jest wspólny z innymi osobami;
  cofnięcie pusha byłoby force-pushem lub usunięciem po ich stronie, a GitPoulpi
  nie zrobi tego za ciebie. (Wyjątkiem jest usunięcie gałęzi lub tagu na zdalnym
  repozytorium: cofnięcie tylko przywraca to, co trzymał serwer, i odmawia, gdy
  cokolwiek się tam zmieniło.)
  Cofanie nigdy też nie jest blokowane przez taką operację.
- **Fetch**: przesuwa tylko refy zdalne, które odzwierciedlają serwer. Następny
  fetch zrobiłby to znowu.
- **Przerwania rebase**: nie oferuje się dla niego cofania (rebase przesuwa gałęzie
  i ma własny stan; samo przerwanie jest drogą powrotną). Przerwanie
  **merge, cherry-pick lub revert** *jest* zapisywane, patrz wyżej.
- **Tworzenia worktree**, **tworzenia pliku patcha**, clone, init, akcji
  hostingu (pull requesty, zgłoszenia) i wywołań AI: nie należą do historii
  repozytorium.
- **Utworzenia nowego pustego pliku** z menu pliku, automatycznego usuwania
  plików `*.orig` po merge i zapisu pliku we wbudowanym edytorze
  nie zapisuje się. Tak samo zmiany **polecenia SSH** (`core.sshCommand`) i zmiany **częściowego checkoutu**
  (nic nie ginie; *Wyłącz* przywraca każdy plik).
- **Zapis do `.git/info/exclude` lub `.git/info/attributes`** (wybór „tylko ten klon” na listach ignorowania i traktowania jako binarne/tekstowe): te pliki leżą poza drzewem roboczym, więc nie są rejestrowane.
- **Zmian plików w drzewie roboczym wykonanych poza GitPoulpi** (edytor, skrypt,
  agent AI piszący kod) nie przechwytuje się nigdy: nie istnieje ich migawka, więc
  `git reset --hard` albo `git checkout -- file` zrobione w powłoce bezpowrotnie
  traci niezatwierdzone zmiany. To, co z pracy powłoki *jest* przechwytywane, leży
  w refach: zob. następną sekcję.

## Zmiany wykonane poza GitPoulpi

Gdy pracujesz z agentem AI (albo z powłoką obok GitPoulpi), większość pracy z
gitem dzieje się poza oknem: commity, nowe gałęzie i worktree, usunięte
gałęzie, reset, rebase, stashe. GitPoulpi widzi to także. Demon trzyma ostatnie
refy, które widział (gałęzie lokalne, tagi, HEAD, listę stashy, listę
worktree), i gdy repozytorium się zmieniło, a nie zrobił tego sam GitPoulpi,
zapisuje **jeden wpis na spójną zmianę** w historii cofania, oznaczony jako
**zewnętrzny**, z powodem, który zapisał git (`commit: fix login`, `rebase
(finish)`, `reset: moving to HEAD~1`, `checkout: moving from a to b`), i z nazwą,
którą git zapisał dla wykonawcy. GitPoulpi nie odróżnia powłoki od agenta AI i
niczego nie zgaduje; wpis mówi tylko, że powstał poza nim.

- **Seria to jeden wpis.** Dwadzieścia commitów zrobionych między dwoma
  spojrzeniami to jeden wpis od pierwszej wartości do ostatniej; rebase to jeden
  wpis, zapisany, gdy się kończy (nic nie jest zapisywane, gdy rebase lub merge
  jest zatrzymany).
- **Usunięte refy są przypinane.** Gałąź, tag lub stash, który zniknął, jest
  trzymany pod `refs/gitpoulpi/undo/` na swoim dawnym commicie, zanim cokolwiek
  zdąży go oczyścić, więc **Przywróć tylko tę** (albo `Ctrl+Z`) przywraca
  gałąź usuniętą przez agenta, nawet po reflogu i garbage collection.
- **Gdy GitPoulpi był zamknięty.** Migawka jest zapisana obok dziennika
  (`.git/gitpoulpi/refs-snapshot.json`), więc to, co zdarzyło się, gdy żaden
  GitPoulpi nie działał, jest znajdowane przy następnym starcie i oznaczane
  „gdy GitPoulpi było zamknięte”.
- **`Ctrl+Z` działa na nie jak na każdy wpis**: refy wracają, usunięta gałąź
  jest tworzona ponownie, porzucony stash jest zapisywany znowu, zewnętrzny
  checkout robi ponownie checkout starej gałęzi. Bieżąca gałąź cofa się jak `git
  reset --keep`: pliki podążają za nią, a gdy któraś z twoich lokalnych zmian
  stoi na przeszkodzie, **odmawia i niczego nie zmienia**. Ponów stosuje to
  znowu. **Przywróć tylko tę** też działa: usunięta gałąź wraca, a późniejsze
  akcje zostają; przesunięta gałąź wraca jako `restore/<branch>-<sha>`.
- **Pasek stanu mówi o tym raz**: „zapisano zmianę spoza GitPoulpi - Ctrl+Z, by
  cofnąć”, gdy pojawi się nowy wpis zewnętrzny, a okno jest na wierzchu.
- Ponieważ zmiana zewnętrzna jest teraz najnowszym wpisem, dawna odmowa
  „zablokowane przez zmianę zewnętrzną” w większości znika: najpierw cofasz tę
  zmianę.

Czego się nie zapisuje: przesunięć samego fetch w gałęziach śledzących zdalne
(odzwierciedlają serwer), własnych ukrytych refów GitPoulpi, worktree dodanego
lub usuniętego bez żadnej zmiany gałęzi (wpis gałęzi, którą niesie, mówi
„dodano worktree <path>”; cofnięcie przesuwa tylko refy, nie usuwa worktree) i
wszystkiego wewnątrz plików. `general.journal_outside = false` (Ustawienia >
Ogólne, „Zapisuj zmiany wykonane poza GitPoulpi”) to wyłącza: nic zewnętrznego
nie jest zapisywane, a zmiana wykonana na zewnątrz blokuje najnowszy wpis, jak
dawniej.

## Kiedy cofanie odmawia

Przed działaniem GitPoulpi sprawdza, czy wszystko, czego dotknęła operacja, jest nadal
dokładnie takie, jakie zostawiła. Jeśli nie, **niczego nie zmienia** i mówi, co
się przesunęło:

- `main` jest na 3f2a1c0, a nie na a81b9e4, gdzie zostawił go Commit (na przykład commit zrobiony
  w powłoce);
- plik, indeks, stash lub sekcja konfiguracji została od tamtej pory zmieniona;
- trwa merge, rebase lub cherry-pick;
- operacja nigdy się nie zakończyła, bo GitPoulpi został zatrzymany w trakcie jej wykonywania (wpis
  jest oznaczony jako *interrupted*, nigdy nie jest cofany automatycznie, a `gitpoulpi
  undo --list` pokazuje, co miał zmienić, żebyś mógł to naprawić ręcznie).

Cofanie działa ściśle według zasady „ostatni wchodzi, pierwszy wychodzi”: zablokowany najnowszy wpis blokuje też
starsze, bo cofanie starszej operacji pod nowszą, która
nadal obowiązuje, byłoby zgadywaniem. Jeśli sam przywrócisz stan, wpis
znów staje się ważny. Zmiany, które nie dotykają niczego, czego dotknął wpis (commit
z powłoki na innej gałęzi, fetch), niczego nie blokują.

### Zablokowany wpis, którego nie da się odblokować: Zapomnij

Gdy najnowszy wpis utknął na stałe (repozytorium zmieniło się poza
GitPoulpi albo wpis nigdy się nie zakończył), nic pod nim też nie da się cofnąć.
Podpowiedź przy Cofnij i komunikat o odmowie mówią o tym i wskazują wyjście:
**Edycja: Zapomnij zablokowany wpis cofania** w palecie (najpierw pyta o potwierdzenie) lub
`gitpoulpi undo-forget` w wierszu poleceń. Zapomnienie usuwa z dziennika tylko ten jeden
wpis; repozytorium nie jest nigdy ruszane, a wpisu nie da się już
cofnąć ani ponowić. Wpis pod nim staje się wtedy najnowszy.

Dwa przypadki rozwiązują się same:

- konfliktowy merge lub rebase, który kończysz zwykłym gitem: wpis jest
  zamykany przy następnym odczytaniu stanu repozytorium przez GitPoulpi;
- pliki konfliktowego stash pop lub apply, które rozwiązujesz przez **Zapisz i oznacz jako rozwiązany**:
  Cofnij przywraca konflikt, dopóki rozwiązany plik jest nadal
  w poczekalni i niezatwierdzony.

## Potwierdzenia

Cofanie pyta tylko wtedy, gdy jest ryzykowne:

- commit (lub merge, rebase, cherry-pick, revert), który cofasz, jest
  już w upstreamie gałęzi: cofnięcie przepisuje opublikowaną historię,
  a wynik trzeba by wypchnąć siłą (force-push);
- masz niezatwierdzone zmiany, a cofnięcie rusza drzewo robocze: twoje
  zmiany są zachowane, ale cofanie zatrzymuje się bez żadnych zmian, jeśli jedna z nich jest w
  pliku, który musi przepisać;
- wpis usunął gałąź lub tag na zdalnym repozytorium: jego cofnięcie **wypycha je
  z powrotem na serwer** (zob. wyżej), więc zawsze pyta, tak samo jak ponowienie,
  które usuwa je tam znowu.

Ponów i każde cofnięcie samego przesunięcia refa nigdy nie pytają: druga strona jest
o jeden klawisz dalej.

## Jak daleko wstecz

`general.undo_depth` (Ustawienia > Ogólne) to liczba przechowywanych operacji.
Domyślnie **100**; `0` oznacza bez limitu. Zmniejszenie tej wartości natychmiast zapomina najstarsze
wpisy. Rozpoczęcie nowej operacji po cofnięciu odrzuca to, co można było
ponowić, jak w każdym edytorze.

Historia jest przechowywana osobno dla każdego worktree (powiązany worktree ma własną), w katalogu
git (`.git/gitpoulpi/journal.json`), więc przetrwa zamknięcie programu
i nigdy nie jest częścią `git status` ani commita.

## Wyłączanie

`general.undo_enabled = false` (Ustawienia > Ogólne, „Cofanie włączone”) wyłącza
całą funkcję (a z nią zapisywanie zmian zewnętrznych). Nic nie jest zapisywane ani przechowywane, więc stage,
odrzucanie i commitowanie są nieco szybsze. Przyciski Cofnij i Ponów
znikają, a `Ctrl+Z` pokazuje „Cofanie jest wyłączone”.

**Wyłączenie kasuje historię**: plik dziennika i ukryte refy
opisane niżej są usuwane przy pierwszym żądaniu, które dotrze do demona
podczas wyłączenia. Ponowne włączenie zaczyna od pustej historii.

## Co jest przechowywane

Aby cofnięcie nigdy nie mogło zgubić danych, każdy commit, obiekt tagu, stash lub
migawka, których wpis może potrzebować, jest utrzymywany przy życiu przez ukryty ref w
`refs/gitpoulpi/undo/` tak długo, jak wpis jest w historii. Własne listy GitPoulpi
ich nie pokazują, ale `git log --all` lub
`git for-each-ref` w powłoce je pokaże, a `git push --mirror` by je wypchnął (użyj zamiast tego
`git push --all`). Są usuwane, gdy ich wpis wypada z
historii lub gdy cofanie zostaje wyłączone. Nic innego nie jest nigdy usuwane.

## Z wiersza poleceń i MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` opisuje kroki i niczego nie zmienia.
- `--list` wypisuje dziennik, najnowszy na końcu (`--all` dodaje wpisy, które nie są
  już możliwe do ponowienia, ale nadal przechowywane), z identyfikatorem i stanem każdego wpisu.
- `--expect <id>` sprawia, że wywołanie odmawia jako *stale*, o ile ten wpis nie jest nadal
  najnowszy: skrypt (lub drugie okno) nigdy nie cofnie czegoś innego niż
  to, na co patrzył.
- `undo-forget` usuwa zablokowany najnowszy wpis z dziennika (`--redo`: najnowszy
  wpis ponawiania). Odmawia, gdy wpis nie jest zablokowany, i kończy się kodem **3**
  bez `--yes`. Dla MCP to mutacja, ale nie destrukcyjna: repozytorium
  pozostaje nietknięte.
- Kod wyjścia **0** wykonano, **2** odmowa (JSON podaje powód, na przykład
  `RemoteRefMoved`, gdy usunięta gałąź zdalna została utworzona ponownie), **3**
  ryzykowne i wymaga `--yes` (cofnięcie usunięcia gałęzi lub tagu na zdalnym
  repozytorium, które wypycha na serwer).

Wracanie bez cofania wszystkiego, co było potem:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (także: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (także: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` pokazuje przy każdym wpisie, co mogłoby przywrócić **Przywróć
  tylko tę** (`restore.items`) albo dlaczego nic (`restore.not_restorable`), oraz
  kto go wykonał: `"source": "gitpoulpi"` albo `"source": "outside"` z
  `outside.message` (powód podany przez gita) i `outside.committer` dla zmiany
  zrobionej przez powłokę lub agenta AI. `undo`, `redo` i `undo-restore`
  traktują wpisy zewnętrzne jak każde inne.
- `undo-restore <id>` przywraca to, co usunął ten jeden wpis (usuniętą gałąź lub
  tag, porzucony stash, odrzucony lub usunięty plik, usunięte zdalne repozytorium
  z jego gałęziami zdalnymi i upstreamami, wiersz ignorowania) i zachowuje każdą
  późniejszą operację. Niczego nie nadpisuje: plik zmieniony od tamtej pory
  dostaje zachowaną wersję obok siebie jako `<name>.restored`; zajęta nazwa
  gałęzi to odmowa (kod 2) z propozycją wolnej, którą podaje się przez `--name`;
  po reset, amend lub rebase powstaje nowa gałąź na dawnym końcu, a twoja gałąź
  się nie przesuwa. Przywrócenie to jeden wpis dziennika, więc `gitpoulpi undo`
  je cofa.
- `reflog` wypisuje, gdzie był `HEAD` lub gałąź, z powodem podanym przez gita,
  także to, co zrobiła powłoka. `reflog-restore` przesuwa gałąź (lub `HEAD`) na
  jedną z tych pozycji; bieżąca gałąź z niezatwierdzonymi zmianami wymaga
  `--mode`. To też trafia do dziennika.
- `lost-commits` wypisuje commity, do których nie sięga żadna gałąź, tag,
  zdalne repozytorium ani stash. Aby odzyskać pracę jednego z nich bez
  commitowania: `apply-commit <rev>` (do drzewa roboczego, wszystko albo nic)
  lub `stash from-commit <rev>` (jako stash); oba są zapisywane w historii.
- Gdy ruch w powłoce blokuje najnowszy wpis, odmowa zawiera pozycję reflogu,
  która trzyma to, czego wpis oczekuje (`reason.RefMoved.restore`), do
  przekazania w `reflog-restore`.

Istnieją dwa polecenia, dzięki którym powyższe akcje można cofać z każdego interfejsu:
`gitpoulpi rm-file <paths>` (usuwa pliki z drzewa roboczego) oraz
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (dopisuje wiersz do
`.gitignore`; nic się nie dzieje, gdy wiersz już tam jest; cofnięcie przywraca plik
do poprzedniego stanu albo usuwa go, jeśli utworzyło go to polecenie, i odmawia, jeśli
plik został w międzyczasie zmieniony).

Serwer MCP ma jedno narzędzie na polecenie, więc `undo`, `redo`, `rm-file` i
`ignore` też są narzędziami. `undo`, `redo` i `rm-file` są klasyfikowane jako destrukcyjne
(bramkowane jak `reset`, patrz [MCP i agenci](mcp-and-agents.md)); `ignore` to
zwykła mutacja. Dziennik jest plikiem w repozytorium, więc `gitpoulpi
undo` w powłoce cofa to, co zrobiło okno, a okno może cofnąć to, co zrobił
skrypt.
