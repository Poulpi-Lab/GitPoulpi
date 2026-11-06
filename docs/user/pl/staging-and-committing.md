[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · **Polski** · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage i commity

Jak zobaczyć, co zmieniono, wybrać dokładnie to, co trafia do commita, napisać
commit i przyjrzeć się plikom dokładniej (blame, historia, wbudowany edytor).
Pojęcia takie jak *wiersz WIP* i *hunk* wyjaśnia
[przewodnik po interfejsie](interface-tour.md).

## Zmiany robocze

Niezatwierdzona praca to **pierwszy wiersz Logu** (przerywane kółko, `// WIP`, z
licznikami `✎ n` zmodyfikowanych i `+ n` dodanych). Zaznacz go, a prawy panel pokazuje
zmiany robocze:

![Zmiany robocze: pliki Unstaged i Staged po prawej, poniżej formularz commita](../images/hero.png)

- **Pliki unstaged**: edytowane, dodane (nieśledzone) i usunięte pliki, których jeszcze nie ma
  w następnym commicie.
- **Pliki staged**: to, co będzie zawierał następny commit.
- **Ścieżka** lub **Drzewo** (`m`) pokazuje pliki płasko albo w folderach. **Zwiń wszystko**
  (`z`) zwija foldery. **Wszystkie pliki** (`a`) pokazuje całe drzewo
  zamiast tylko zmienionych plików. Prawy przycisk na pustym miejscu listy daje
  **Rozwiń wszystko** / **Zwiń wszystko**, na wierszu folderu **Rozwiń wszystkie podfoldery** /
  **Zwiń wszystkie podfoldery**; klawisze `=`, `-`, `*`, `_` (zob.
  [Mysz i gesty](mouse-and-gestures.md)).
- Tytuł mówi, ile plików się zmieniło i na której gałęzi.

### Stage, unstage, odrzucanie

| Aby | Zrób |
|---|---|
| Dodać plik do stage | Najedź na jego wiersz i kliknij **Stage plik**, albo zaznacz go i naciśnij `Ctrl+Space`, albo `Ctrl+G s` |
| Dodać folder do stage | **Stage folder** w wierszu folderu |
| Dodać wszystko do stage | **Stage wszystkie zmiany** w tytule sekcji albo `Ctrl+G a` |
| Cofnąć stage | Te same przyciski i klawisze w sekcji Staged (`Ctrl+G u`) |
| Odrzucić zmiany pliku | Kliknij plik prawym przyciskiem > **Odrzuć zmiany...** (najpierw pyta; `Ctrl+Z` przywraca zmiany) |

### Wrażliwe pliki

Dodanie do stage pliku, folderu lub wszystkiego najpierw sprawdza, co zostałoby dodane.
Plik, którego nazwa pasuje do `[privacy] sensitive_patterns` (`.env` i `.env.*` z wyjątkiem
tych z końcówką `.example`, `.sample` i `.template`, `*.pem`, `*.key`, prywatne klucze SSH,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), albo
którego tekst zawiera sekret rozpoznany przez skaner, nie trafia do stage. Pytanie **Wrażliwe
pliki** wymienia każdy plik z przyczyną (wzorzec albo rodzaj sekretu
z zamaskowanym podglądem, nigdy wartość) i oferuje **1. Stage mimo to** albo **2. Anuluj,
bez stage**. Żadna odpowiedź nie jest podświetlona, dopóki nie naciśniesz strzałki lub cyfry,
więc przypadkowy Enter nie odpowiada na nic; `Esc`, krzyżyk zamknięcia i kliknięcie poza oknem anulują.
To samo pytanie pojawia się z klawiatury, przycisku, menu, palety lub
przeciągnięcia, bo wszystkie wysyłają to samo żądanie. Anulowanie nie dodaje nic do stage,
nawet zwykłych plików z tego samego stage.

Pliki tekstowe do 1 MiB każdy i 32 MiB łącznie są czytane; większe pliki i pliki
binarne są oceniane tylko po nazwie. `[privacy] staging_guard = false` wyłącza sprawdzanie.
Stage jednego hunka lub kilku linii nie jest sprawdzany: już pracujesz nad tym plikiem.

Kliknij plik prawym przyciskiem, by zobaczyć jego menu: stage lub unstage, odrzucenie, kopiowanie ścieżki, historia
pliku i blame, otwarcie w zewnętrznym edytorze, narzędziu diff lub menedżerze plików (ustawia się je
w [Ustawienia > Diff i edytor](settings.md#diff-i-edytor)) i więcej.

## Ignorowanie plików i traktowanie ich jako binarnych lub tekstowych

Kliknij plik prawym przyciskiem > **Ignoruj**, aby zignorować plik lub jego rozszerzenie, albo wiersz folderu > **Ignoruj wszystkie pliki w folderze**. GitPoulpi najpierw pyta, do którego pliku trafi reguła: `.gitignore` w katalogu głównym repozytorium (domyślnie), `.gitignore` każdego folderu aż do folderu pliku albo `.git/info/exclude` (tylko ten klon, nigdy nie zatwierdzany). Pliki, które jeszcze nie istnieją, są oznaczone jako *nowy* i tworzone po potwierdzeniu. Wiersz jest zapisywany względem wybranego pliku: `src/a/b.txt` staje się `/a/b.txt` w `src/.gitignore`. Poruszaj się strzałkami, wpisuj, aby zawęzić listę, `Enter` zapisuje, `Esc` anuluje. `Ctrl+Z` cofa zapis do `.gitignore`; `.git/info/exclude` leży poza drzewem roboczym i nie da się go cofnąć.

![Lista pytająca, do którego pliku trafi reguła ignorowania: .gitignore repozytorium, .gitignore folderu pliku albo .git/info/exclude, każdy z wierszem, który zostałby zapisany](../images/ignore-picker.png)

Kliknij plik prawym przyciskiem > **Traktuj jako binarny** (albo **Traktuj jako tekstowy**, gdy git już traktuje go jako binarny), aby zapisać regułę `.gitattributes`: `binary` lub `text diff`. Wybierz sam plik albo wszystkie pliki z jego rozszerzeniem, potem `.gitattributes` do zapisu, z tą samą listą co wyżej (i `.git/info/attributes` tylko dla tego klonu). Reguła już zapisana dla tego samego wzorca jest zastępowana, nigdy nie przeczy. Diff pliku odświeża się od razu, a pasek stanu mówi, co i gdzie zapisano (i ostrzega, gdy inna reguła nadal ma pierwszeństwo). Wiersz folderu ma **Traktuj pliki w folderze jako** > **Binarne** lub **Tekstowe**. To samo jest w palecie poleceń (**Zmiany robocze: Traktuj zaznaczony plik jako binarny...**) i w wierszu poleceń (`gitpoulpi treat binary|text <ścieżki>`). Widok pliku również stosuje tę regułę: plik traktowany jako tekst pokazuje swoje wiersze, nawet jeśli zawiera bajt NUL, a plik, który git porównuje jako binarny, pokazuje informację o pliku binarnym.

## Menu, zaznaczanie wielu plików i konwersja na listach plików

Listy plików w prawym panelu (Unstaged, Staged, zmienione pliki commitu i **Wszystkie pliki**) mają te same menu i gesty. Kliknij prawym przyciskiem plik, wiersz folderu lub puste miejsce listy, albo naciśnij `.` (`Shift+F10`, klawisz Menu) w wierszu z fokusem albo uruchom **Pliki: otwórz menu wiersza lub zaznaczenia** z palety.

- **Plik commitu** (i plik w **Wszystkie pliki**): **Otwórz diff**, **Otwórz plik**, **Historia pliku**, **Blame pliku**, otwórz w edytorze, w programie domyślnym lub w **Otwórz za pomocą...**, **Pokaż w folderze**, **Edytuj plik**, **Kopiuj ścieżkę pliku**, **Kopiuj pełną ścieżkę pliku**, **Ignoruj**, **Traktuj jako binarny** lub **Traktuj jako tekstowy**, **Konwertuj**, **Przywróć plik z tego commita** (najpierw pyta; jak `git checkout <commit> -- <plik>` także dodaje plik do stage, a `Ctrl+Z` przywraca poprzedni plik) oraz **Usuń plik**. Wiersz dotyczący kopii roboczej jest wyszarzony, z podanym powodem, gdy pliku nie ma w drzewie roboczym. **Diff względem drzewa roboczego** otwiera diff wersji z commita względem pliku na dysku. **Utwórz patch ze zmian pliku** zapisuje patch zmiany tego pliku w commicie (ma go też zaznaczenie i wiersz folderu; w **Wszystkie pliki** na zmianach roboczych tworzy patch zmian pliku w drzewie roboczym).
- **Wiersz folderu**: **Otwórz folder**, **Kopiuj ścieżkę pliku**, **Ignoruj wszystkie pliki w folderze**, **Traktuj pliki w folderze jako**, **Konwertuj**, **Utwórz plik w tym folderze**.
- **Puste miejsce listy**: **Utwórz plik** i **Otwórz folder repozytorium**. Lista Unstaged dodaje **Stage wszystkie zmiany** i **Odrzuć wszystkie zmiany...**, lista Staged - **Unstage wszystkie zmiany**.

![Menu prawego przycisku myszy dla pliku commita: otwórz diff, historię i blame pliku, otwórz w edytorze lub programie domyślnym, kopiuj ścieżki, ignoruj, konwertuj, przywróć z commita i usuń](../images/commit-file-menu.png)

### Zaznaczanie wielu plików

Gesty są takie jak w Logu. **Ctrl+klik** dodaje wiersz do zaznaczenia lub go usuwa. **Shift+klik** (**Alt+klik** w terminalach, które zatrzymują Shift+klik) zaznacza zakres od kotwicy. Zwykłe kliknięcie zaczyna od nowa i otwiera diff jak dotąd. `Esc` czyści zaznaczenie. Z klawiatury `Space` lub `Insert` przełącza bieżący wiersz na każdej liście (w Unstaged i Staged `Ctrl+Space` dodaje do stage zaznaczenie albo bieżący wiersz), a `Shift+Góra` / `Shift+Dół` rozszerzają zakres. Wiersz folderu zaznacza wszystkie pliki pod nim, zwinięty lub nie. Zaznaczone wiersze są podświetlone, a pasek stanu podaje ich liczbę. `Ctrl+A` zaznacza wszystkie wiersze aktywnej listy.

Kliknij prawym przyciskiem zaznaczony wiersz (albo naciśnij `.`), aby otworzyć menu całego zaznaczenia z nagłówkiem **N plików**: stage lub unstage, odrzuć, ignoruj, traktuj jako binarny lub tekstowy, konwertuj, kopiuj ścieżki, przywróć z commitu, usuń. Stage, unstage i przywrócenie to po jednej akcji; odrzucenie i usunięcie najpierw pytają i podają liczbę plików. Kliknięcie prawym przyciskiem wiersza poza zaznaczeniem zaczyna zaznaczanie od nowa od tego wiersza.

![Cztery pliki zaznaczone na liście Unstaged, z menu zatytułowanym 4 pliki: stage, odrzuć, ignoruj, traktuj jako binarny lub tekstowy, konwertuj, kopiuj ścieżki i usuń](../images/file-pane-multiselect.png)

### Konwersja kodowania, zakończeń wierszy i wcięć

**Konwertuj** jest w menu pliku, folderu lub zaznaczenia. `gitpoulpi convert` robi to samo z wiersza poleceń.

- **Kodowanie**: UTF-8, UTF-8 BOM, UTF-16 LE i BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Zakończenia wierszy**: LF, CRLF lub CR.
- **Wcięcia**: białe znaki na początku wiersza jako spacje lub tabulatory, z szerokością tabulatora.

Pliki w drzewie roboczym są przepisywane w tle tymi samymi funkcjami co pasek stanu edytora. Plik, który nie jest poprawnym UTF-8 i nie ma znacznika kolejności bajtów, jest czytany jako Windows-1252, tak jak w edytorze. Pliki binarne, brakujące oraz takie, których kodowanie docelowe nie potrafi przedstawić, są pomijane i zgłaszane (*Przekonwertowano 12 plików, pominięto 3 (3 binarnych)*), nigdy uszkadzane; plik, którego nie da się zapisać, wywołuje powiadomienie o błędzie. Cała operacja to jeden wpis w dzienniku cofania: jedno `Ctrl+Z` przywraca wszystkie pliki. Plik otwarty w edytorze blokuje konwersję, dopóki nie zostanie zamknięty.

## Czytanie diffa

Kliknij nazwę pliku (albo naciśnij na nim `d`), a jego diff zastąpi Log
pośrodku. `Esc` lub `q` przywraca Log.

![Diff z dwoma hunkami, każdy z przyciskami Odrzuć hunk i Stage hunk](../images/diff.png)

Pasek nad nim przełącza widok:

- **W linii** pokazuje diff ujednolicony; **Obok siebie** ustawia stan przed i po obok
  siebie (`m` przełącza układy).
- **Hunki** (`Shift+H`) pokazuje tylko zmienione hunki z jedną linią kontekstu.
  Wyłącz, by zobaczyć cały plik.
- **WS** (`s`) ukrywa zmiany różniące się tylko białymi znakami.
- **Zawij** (`w`) zawija długie linie zamiast je obcinać.
- **Mapa** (`Shift+M`) pokazuje minimapę całego pliku obok paska przewijania.
  Kliknij ją, by przeskoczyć. Inne wystąpienia zaznaczonego słowa są podświetlone i oznaczone
  na pasku przewijania.
- `n` i `p` (lub strzałki na pasku) przeskakują między hunkami.
- **Auto** (ostatni przycisk, `Shift+C`) to kodowanie, w jakim czytany jest plik. Kliknij go (albo naciśnij
  `Shift+C`, albo uruchom z palety *Diff: Wybierz kodowanie...*) i wybierz jedno z listy:
  najpierw **Wykryj automatycznie**, potem UTF-8, UTF-8 BOM, UTF-16 LE i BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 i EUC-KR (wpisz, aby
  przeszukać listę). Obie strony pliku są czytane ponownie, jako dokładne bajty, w tym kodowaniu, więc
  plik Latin-1 pokazuje swoje akcenty, a plik UTF-16 (który git uznaje za binarny) pokazuje swoje linie. Wybór
  jest pamiętany dla ścieżki tego pliku do zakończenia programu, w tym widoku i w
  [przeglądzie](review.md). Gdy kodowanie jest wybrane, hunki są własne dla tego widoku, więc stage i
  odrzucanie są wyłączone (wybierz **Wykryj automatycznie**, aby znów dodawać do poczekalni). Wolny odczyt po chwili mówi *odczyt jako ...* w
  tytule.

![Ten sam diff w układzie Obok siebie, przed i po obok siebie](../images/diff-split.png)

Tekst można zaznaczać myszą (przeciągnięciem lub dwukliknięciem słowa) i kopiować
przez `y` lub `Ctrl+C`; kopiowany jest tylko tekst pliku, nigdy numery linii ani znaczniki `+`
i `-`.

### Stage hunka lub pojedynczej linii

W diffie drzewa roboczego każdy nagłówek hunka ma **Stage hunk** i **Odrzuć
hunk** (po stronie Staged jest **Unstage hunk**).

- `Space` dodaje do stage hunk pod kursorem.
- Aby dodać kilka linii, naciśnij `v`, by zacząć zaznaczanie, poruszaj się przez `j`/`k` i
  naciśnij `Enter`. Liczą się tylko linie *zmienione*.
- `x` odrzuca hunk, a `Shift+X` zaznaczone linie. Oba pytają najpierw, a
  `Ctrl+Z` przywraca zmiany.
- Kliknij prawym przyciskiem (lub `.`) na hunku, by zobaczyć jego menu: stage, unstage, odrzucenie, historia
  pliku, blame, kopiowanie.

Diff commita jest tylko do odczytu. Zmieniony obraz pokazuje się jako obraz; zob.
[Obrazy](image-view.md).

## Pisanie commita

Formularz commita jest na dole prawego panelu, a jego **Podsumowanie** jest też
polem w wierszu zmian roboczych w Logu: tekst wpisany w jednym pojawia się w
drugim.

![Formularz commita wypełniony opisem napisanym przez AI: podsumowanie z licznikiem znaków, opis i zielony przycisk commita](../images/ai-commit-message.png)

- **Podsumowanie** (licznik pokazuje, ile z 72 znaków zostało) i
  **Opis**. `Enter` w podsumowaniu przechodzi do opisu, `Enter`
  tam dodaje linię.
- **Commit zmian w N plikach** (zielony przycisk) lub `Ctrl+Enter` zatwierdza zmiany
  w stage. Jeśli nic nie jest w stage, przycisk o tym mówi i nic nie robi.
- **Popraw poprzedni commit** (`Shift+A`) wchłania zmiany ze stage i nowy opis
  do ostatniego commita. Commit, który jest już wypchnięty, będzie wtedy wymagał force
  push (with lease).
- **Pomiń hooki** (`Shift+H` albo **Commit: Przełącz pomijanie hooków** w palecie) tworzy commit
  bez uruchamiania hooków `pre-commit` i `commit-msg` repozytorium. Dotyczy jednego commita;
  **Ustawienia > Ustawienia repozytorium > Commit** sprawia, że opcja jest domyślnie
  zaznaczona. Pokazywana, gdy wiersz ma miejsce.
- `Ctrl+G c` lub `c` w panelu Status przenosi kursor do formularza. `Esc`
  go opuszcza.
- `Ctrl+G Enter` (**Git: Zrób commit zmian ze stage od razu** w palecie) to
  przycisk commita dostępny z dowolnego miejsca: robi commit zmian ze stage z opisem
  z formularza, z uwzględnieniem **Popraw poprzedni commit** i **Pomiń hooki**, bez
  przenoszenia fokusu. Gdy nic nie jest w stage albo brakuje podsumowania, mówi o tym
  i zamiast tego ustawia kursor w podsumowaniu. Gdy zapytanie AI jeszcze trwa, nie
  tworzy żadnego commita i mówi o tym (`Esc` anuluje zapytanie). `Ctrl+Z` cofa ten
  commit jak każdy inny.
- Dla repozytorium [Ustawienia](settings.md#ustawienia-repozytorium-commit) mogą **robić push po
  każdym commicie**, zaczynać formularz od **szablonu commita** (i usuwać z opisu jego
  komentarze `#`) oraz sprawiać, że merge jest **squash**: formularz otwiera się wtedy
  z komunikatem squash gita.
- Przy podłączonej integracji z hostingiem `Alt+R` w formularzu wstawia odwołanie do
  zgłoszenia; zob. [Hosting i trackery zgłoszeń](hosting-integration.md).

Gdy [AI](ai.md) jest włączone, przycisk **`✦`** na prawo od podsumowania pisze opis
na podstawie zmian w stage (`Ctrl+G i`), a **Skomponuj commity przez AI** dzieli
twoją pracę na kilka commitów (zob. niżej). Tekst trafia do formularza, abyś mógł
go edytować; nic nie jest zatwierdzane, dopóki nie naciśniesz przycisku commita.

## Kompozytor commitów: podziel zmiany na commity

*(Wymaga włączonego AI: zob. [Wsparcie AI](ai.md).)* Naciśnij **Skomponuj commity
przez AI** (lub `Ctrl+G C`, albo kliknij prawym przyciskiem wiersz zmian roboczych). Model proponuje
kilka commitów, każdy z jego plikami; każdy zmieniony plik jest w dokładnie jednym
commicie, a nieprawidłowy plan jest jednokrotnie odsyłany do modelu. Propozycja otwiera się
jako przegląd, w którym nic jeszcze nie zostało zatwierdzone:

![Kompozytor commitów: cztery proponowane commity, każdy rozwijany do swoich plików, ze sterowaniem w górę, w dół, edycją i pominięciem oraz przyciskami Resetuj, Anuluj i Utwórz 4 commity](../images/composer.png)

| Aby | Zrób |
|---|---|
| Rozwinąć lub zwinąć commit | `Left` / `Right` albo kliknij szewron; `*` przełącza wszystkie |
| Przesunąć commit w górę lub w dół | `▲` `▼` w jego wierszu albo `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Przenieść plik do innego commita | Przeciągnij go na commit (z listy albo z listy zmian roboczych po prawej); albo `m` na pliku i wybierz commit z listy rozwijanej; albo `Alt+Up` / `Alt+Down` na pliku |
| Umieścić plik we własnym commicie | `n` na pliku albo przycisk `+ własny commit` |
| Zmienić opis commita | Ołówek, `e`, `Enter` lub dwukrotne kliknięcie: edytuj opis, `Enter` go zachowuje, `Esc` odrzuca |
| Pominąć commit | `drop` (`d` lub `Delete`); jego pliki zostają niezatwierdzone. `przywróć` go przywraca |
| Zacząć od nowa | **Resetuj** (`r`) wraca do planu modelu |
| Utworzyć je | **Utwórz N commitów** (`Ctrl+S`, `Ctrl+Enter` lub `a`) |
| Wyjść | **Anuluj** lub `Esc`: nic nie zrobiono |

Tworzenie commitów robi unstage wszystkiego, a potem dla każdego commita dodaje jego pliki do stage
i commituje, więc twoje drzewo robocze nigdy nie jest ruszane. Jeśli krok się nie powiedzie, proces się zatrzymuje
i mówi, ile commitów powstało. Do stage trafiają całe pliki: plik, który był
w stage tylko częściowo, zostaje zatwierdzony w całości. Profil i instrukcje, których używa
kompozytor, wybiera się w *Ustawienia > AI*; zob. [Wsparcie AI](ai.md#kompozytor-commitów).

## Widok pliku, Blame i Historia pliku

Pasek widoku ma **Widok pliku** (zawartość pliku), **Widok diff**, **Blame** oraz
przełącznik `Plik │ Wszystkie` [kolumny commitów](interface-tour.md#kolumna-commitów) (jej lista
`Plik` to Historia pliku); te same są w menu prawego przycisku pliku. Pokazują plik
w stanie z commita, z którego go otwarto, albo jako niezatwierdzony (`@ WIP`), gdy otwarto go
ze zmian roboczych.

- **Blame** i **Historia pliku** to pełnoprawne widoki: zob. [Blame](log-and-history.md#blame) i [Historia pliku](log-and-history.md#historia-pliku) w rozdziale *Praca z Logiem*. `Enter` na hunku blame lub wierszu historii przechodzi do tego commita w Logu.
- `y` lub `Ctrl+C` kopiuje zaznaczony tekst (w Blame tylko kod).
- `Shift+G` przeskakuje do ostatniej linii. `q` zamyka widok.

## Wbudowany edytor

**Edytuj** (`e`) otwiera *kopię pliku w drzewie roboczym* w małym edytorze
nad środkowym panelem: kolory składni, numery linii, cofanie i ponawianie, szukanie i
pasek stanu (pozycja, koniec linii, wcięcie, język). Zawsze edytuje
kopię roboczą, nawet gdy otwarto go z diffa starego commita.

![Edytor pokazujący plik Rust z podświetlaniem składni i numerami linii; pasek ma Zapisz, Cofnij, Ponów i Szukaj](../images/editor.png)

| Klawisz | Akcja |
|---|---|
| `Ctrl+S` | Zapisz (plik jest zapisywany do pliku tymczasowego, któremu potem zmienia się nazwę, więc nigdy nie jest zapisany w połowie; końce linii, BOM i końcowa nowa linia są zachowane) |
| `Ctrl+Z`, `Ctrl+Y` | Cofnij, ponów |
| `Ctrl+F`, `F3` | Szukaj, szukaj następnego |
| `Tab`, `Shift+Tab` | Wcięcie, zmniejszenie wcięcia (zgodnie z własnym stylem pliku) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Zaznacz wszystko, kopiuj, wytnij, wklej |
| `Esc` | Zamknij. Przy niezapisanych zmianach pyta: **Zapisz** (`s`), **Odrzuć** (`d`) lub **Anuluj** (`c`) |

Pliki binarne są odrzucane. Rozmiar tabulatora, końce
linii, podświetlanie składni, numery linii i zawijanie wierszy są w *Ustawienia >
Diff i edytor*; zob. [Ustawienia](settings.md#diff-i-edytor).

Gdy zaznaczone jest pole **Podążaj za zaznaczeniem** (na pasku edytora, także *Edytor: przełącz podążanie za zaznaczeniem* w palecie i *Ustawienia > Diff i edytor*), zaznaczenie innego pliku w prawym panelu przy otwartym edytorze otwiera w edytorze kopię roboczą tego pliku zamiast jego diffa lub widoku pliku. Przy niezapisanych zmianach edytor najpierw pyta: **Zapisz**, **Odrzuć** lub **Anuluj**; Anuluj zostawia bieżący plik. Plik, którego nie można edytować (binarny, zbyt duży, usunięty), otwiera się w zwykłym widoku, a pasek stanu podaje powód. Zamknięcie edytora prowadzi do widoku zaznaczonego pliku. Domyślnie zaznaczone. Dotyczy to zmian roboczych: wybranie pliku z commita w trakcie edycji opuszcza edytor (po uprzednim zapytaniu o zapis) i pokazuje diff tego pliku w tym commicie, a wraca z nim kolumna commitów, której edytor nie ma.

Pasek stanu po prawej pokazuje **kodowanie**, **końce wierszy** i **wcięcia** pliku, a potem pozycję
kursora i składnię. Pierwsze trzy to przyciski: każdy otwiera listę rozwijaną (strzałki lub kółko, wpisz
tekst, aby filtrować, `Enter` wybiera, `Esc` zamyka). Paleta (`Ctrl+P`) otwiera te same menu: *Edytor: Zmień kodowanie*, *Edytor: Otwórz ponownie z kodowaniem*, *Edytor: Konwertuj na kodowanie*, *Edytor: Konwertuj końce linii*, *Edytor: Zmień wcięcia*.

![Menu kodowania edytora otwarte pod polem UTF-8 na pasku: otwórz plik ponownie w innym kodowaniu albo go przekonwertuj](../images/editor-encoding-menu.png)

![Menu końców wierszy edytora otwarte pod polem LF: przekonwertuj plik na LF, CRLF lub CR](../images/editor-eol-menu.png)

- **Kodowanie.** Plik otwiera się w kodowaniu, które zapowiada (BOM: UTF-8 lub UTF-16), w przeciwnym razie
  jako UTF-8, w przeciwnym razie jako Windows-1252, a pasek mówi, które to. **Otwórz ponownie z kodowaniem**
  czyta bajty pliku jeszcze raz w innym kodowaniu (przy niezapisanych zmianach najpierw pyta): użyj, gdy
  rozpoznanie było błędne. **Konwertuj na kodowanie** zachowuje tekst i zmienia kodowanie, w którym plik jest
  zapisywany (UTF-8, UTF-8 BOM, UTF-16 LE i BE, Windows-1252, ISO-8859-1 i -15, Windows-1250, Windows-1251,
  KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). Nic nie jest zapisywane przed zapisem, a Cofnij to
  odwraca. Jeśli tekst ma znaki, których kodowanie docelowe nie pomieści, pasek stanu podaje ich liczbę
  i miejsce pierwszego, a nic nie jest konwertowane; zapis jest odrzucany tak samo i nigdy nie zapisze `?`.
- **Końce wierszy.** Pokazuje `LF`, `CRLF`, `CR` albo `Mieszane`, gdy plik używa kilku. Menu zamienia wszystkie
  końce wierszy pliku na LF, CRLF lub CR jednym krokiem cofania. Nowe wiersze używają końca
  wiersza pliku. Twój wybór wygrywa z ustawieniem *Końce wierszy*.
- **Wcięcia.** Pokazuje `Spacje: 4` lub `Tabulatory: 4`. Menu ustawia, co wstawiają `Tab` i automatyczne wcięcie
  (spacje lub tabulatory), rozmiar (od 1 do 8: krok wcięcia i szerokość tabulatora w tym pliku) i zamienia
  początkowe białe znaki każdego wiersza na spacje lub tabulatory, jednym krokiem cofania.

## Stash, patche i Git LFS

- **Stash** i **Pop** są na pasku narzędzi; zob.
  [Praca z Logiem](log-and-history.md#gałęzie-tagi-i-panel-refs).
- **Utwórz patch** i **Zastosuj patch** są w menu commitów i refów; CLI
  ma `gitpoulpi patch`.
- Duże pliki przechowywane przez Git LFS pokazują się jako małe pliki wskaźników, dopóki nie ściągniesz
  ich zawartości; zob. [Git LFS](lfs.md).

## Z wiersza poleceń

Wszystko na tej stronie można zautomatyzować skryptem:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # po odmowie opisanej niżej
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` odrzuca wrażliwy plik tak samo: nic nie trafia do stage, kod wyjścia to 2,
a `--json` wypisuje `{error: "sensitive_files", files: [{path, reason}]}`
(reguła i zamaskowany podgląd, nigdy wartość). `--allow-sensitive` to ponowna próba. Agent
MCP dostaje tę samą odpowiedź i nie może przekazać `allow_sensitive`, chyba że
`[mcp] allow_sensitive_staging = true`: ma zapytać ciebie.

Zob. [Opis CLI](cli-reference.md).
