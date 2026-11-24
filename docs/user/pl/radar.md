[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · **Polski** · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Radar

Radar to osobna karta, tuż za Home: jedna lista tego, co na ciebie czeka w **kilku repozytoriach**: pull (albo
merge) requesty, które napisałeś, te, które proszą o twój przegląd, zgłoszenia przypisane
do ciebie lub utworzone przez ciebie oraz repozytoria, w których zostawiłeś pracę w toku.
To okno Home z widokiem Radaru na wierzchu, aby nic pod spodem nie
mogło przechwycić kliknięcia ani klawisza.

Wymaga profilu hosta kodu (zob. [Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md); listy jednego repozytorium są w [Pull requestach i zgłoszeniach](pull-requests-and-issues.md), a dyskusja wiersza otwiera się jako [rozmowa](conversations.md))
oraz repozytoriów z nim powiązanych albo profilu z włączonymi listami obejmującymi całe konto (zob. „Poza zakresem”
niżej). Repozytorium, które nie jest powiązane, nigdy nie jest
odpytywane: zamiast tego pokazuje wiersz **Powiąż...** (zob. „Repozytoria, które mają coś do
powiedzenia” niżej).

![Radar nad dwoma repozytoriami: przypięty request, potem Czekają na mój przegląd, Moje pull requesty, Przypisane do mnie, zgłoszenia i praca w toku](../images/radar.png)

**Przejrzyj...** w wierszu pull lub merge requesta (albo `e`) otwiera repozytorium z przeglądem tego żądania w aplikacji
([Przegląd pull lub merge requesta](review.md)). Dla żądania z repozytorium, które sklonowałeś, ale nigdy nie powiązałeś z hostem, klon jest najpierw
powiązany z projektem żądania (chyba że ma już własne powiązanie z pull requestem), więc przegląd zaczyna się w jego karcie.

### Kolumna Akcje

Z prawej strony wiersza żądania lub zgłoszenia małe przyciski robią to, co klawisz na zaznaczonym wierszu (kliknięcie zaznacza wiersz i
uruchamia akcję; zatrzymaj wskaźnik na przycisku, by zobaczyć jego nazwę i klawisz):

```
Szeroko: #58  Round invoice totals  feature/login → main   alice   2d   Przejrzyj  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Wąsko:   #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Zgłoszenie: #12  Crash on empty list                       bob     5h   Zacznij pracę  [globe]  [talk]  ×  ⋮
```

| Przycisk | Klawisz | Paleta |
|---|---|---|
| **Przejrzyj** (żądanie czekające na twój przegląd; pyta jak, gdy nie jest sklonowane) | `e` | `Radar: Przejrzyj żądanie...` |
| **Zacznij pracę** (sklonowane zgłoszenie) | `n` | `Radar: Zacznij pracę nad zgłoszeniem...` |
| globus: otwórz stronę na hoście | `o` | `Radar: Otwórz w przeglądarce` |
| glif z liniami: otwórz rozmowę | `i` | `Radar: Otwórz rozmowę zgłoszenia` |
| `»` **Oznacz jako gotowe** (szkic żądania) | `y` | `Radar: Oznacz jako gotowe` |
| `⊕` **Merge** (żądanie) | `m` | `Radar: Merge żądania...` |
| `×` **Zamknij** (żądanie lub zgłoszenie) | `x` | `Radar: Zamknij...` |
| `⇣` **Klonuj** (wiersz, którego repozytorium nie jest sklonowane) | `g` | `Radar: Klonuj repozytorium...` |

Wiersz bez akcji zostawia swoją komórkę pustą, więc każdy przycisk zostaje w tej samej kolumnie w każdym wierszu. Gdy okno jest
wąskie, słowa Przejrzyj i Zacznij pracę zamieniają się w glify, zanim zniknie jakakolwiek inna kolumna; przyciski Gotowe, Merge, Zamknij i Klonuj
opuszczają kolumnę przed tytułem, ale zostają w menu wiersza (`⋮`, prawy przycisk myszy albo `Shift+F10`), gdzie akcja,
której nie można wykonać, jest wyszarzona z powodem. Kolumna jest automatyczna (nie należy do kolumn, którym
można zmieniać rozmiar). Podwójne kliknięcie lub `Enter` na żądaniu, które nie jest sklonowane, pyta, jak je przejrzeć, jak **Przejrzyj**.

### Gałęzie żądania

Wiersz żądania pokazuje `źródło → cel` w kolumnie **Gałęzie**, zaraz po repozytorium (`alice:feature/login → main`,
gdy gałąź źródłowa jest w forku); wiersz zgłoszenia zostawia tę komórkę pustą, a Radar bez żądania, które podaje gałęzie,
nie ma takiej kolumny. Gdy komórka jest za wąska, najpierw ucinane jest źródło (wielokropek), a cel zostaje w całości, dopóki się da.
Kolumna kurczy się, zanim zniknie: w wąskim oknie jako ostatnia z kolumn opcjonalnych odchodzi
(po stanie, autorze i wieku, przed repozytorium i tytułem); gdy jej nie ma, pełne gałęzie zostają
w podpowiedzi tytułu i na pasku stanu zaznaczonego wiersza. Szeroka kolumna autora, którą sam ustawiłeś, ustępuje
tytułowi i gałęziom. Kolumna Akcje nigdy nie jest wypychana. Zatrzymaj wskaźnik na uciętym tekście, by zobaczyć go w całości.
GitHub i Gitea wypisują żądania bez gałęzi, więc Radar czyta je po liście (raz na żądanie, dopóki się nie zmieni). Host, który
nie podał gałęzi, nie pokazuje tam nic. Wiersz zgłoszenia nazywa swoje repozytorium w tym samym chipie co wiersz żądania (nazwa
folderu klonu albo `owner/name`, gdy nie jest sklonowane).

### Działanie na hoście

**Merge**, **Zamknij** i **Oznacz jako gotowe** działają na GitHubie, GitLabie, Gitea i Forgejo (inne hosty wyszarzają je z „Nieobsługiwane przez ...”;
zgłoszenia można zamykać na tych samych hostach).

- **Merge** pyta najpierw: okno nazywa żądanie, jego projekt i gałęzie oraz oferuje metodę (zatwierdzenie scalające, squash
  i rebase tam, gdzie host go ma; GitLab scala lub robi squash). Jest odrzucane z powodem dla szkicu, żądania, które
  ma konflikt ze swoim celem, albo niepowodzących się kontroli; odmowa hosta (brak uprawnień, reguła gałęzi) jest pokazywana
  na pasku stanu.
- **Zamknij** pyta najpierw i zamyka bez merge.
- **Oznacz jako gotowe** zdejmuje szkic ze statusu szkicu (GitLab i Gitea: znacznik `Draft:` / `WIP:` znika z tytułu). Niczego
  nie pyta: na hoście jest odwracalne.
- **Klonuj** otwiera zwykłe okno Klonowania na projekcie wiersza (URL jest wypełniony, wybór folderu jest zwykły). Gdy
  klon istnieje, Radar umieszcza wiersz na nim przy następnym odczycie zdalnych (otwórz go ponownie lub Odśwież), a Klonuj
  mówi wtedy, gdzie jest klon.

Merge i Zamknij zmieniają hosta i **nie można ich cofnąć z GitPoulpi** (okno tak mówi); otwórz zamknięte żądanie ponownie na hoście.
Po udanym wywołaniu listy są pobierane ponownie, więc scalony lub zamknięty wiersz znika, a gotowy traci swój chip `draft`.

## Otwieranie

- **`Alt+R`** (można zmienić: `global.radar`), z dowolnego miejsca.
- Paleta: `Radar: Otwórz`.
- Karta **Radar** zaraz za Home na pasku kart.
- Wiersz **Radar** w menu **`+`**, gdy karta jest zamknięta.

Z karty repozytorium przenosi to do okna Home z otwartym Radarem; `Esc` albo **Wstecz**
wraca do listy repozytoriów, a `⌂` robi to samo.

## Zamykanie karty lub wyłączenie Radaru

Kartę zamyka się jak kartę repozytorium: jej **x**, środkowym przyciskiem myszy, `Ctrl+W`, gdy jest widoczna, albo poleceniem palety `Radar: Zamknij kartę`. Home to jedyna karta, której nie można zamknąć. Zamknięta karta zostaje zamknięta w tej sesji także po ponownym uruchomieniu, dopóki nie otworzysz Radaru znowu (`Alt+R`, `Radar: Otwórz` albo menu `+`); wraca wtedy zaraz za Home. Zamknięcie nie zmienia żadnego ustawienia.

Jeśli go nie używasz, wyłącz go w **Ustawieniach > Ogólne > Włącz Radar** (`general.radar_enabled`, zob. [opis konfiguracji](config-reference.md)). Po wyłączeniu znikają karta, jej polecenia palety i `Alt+R`, a GitPoulpi niczego dla niego nie pobiera. Działa od razu, bez ponownego uruchamiania.

## Co obejmuje: zakres

Lista rozwijana **Zakres** wybiera, które repozytoria są odpytywane:

- **obszar roboczy**, według nazwy (jego repozytoria, także te bez otwartej karty);
- **Otwarte karty**: repozytoria otwarte jako karty tej sesji;
- **Domyślny folder klonowania** i **Domyślny folder przeglądania**: każde repozytorium znalezione w folderze
  ustawionym w [Ustawieniach](config-reference.md) (`repos.clone_root`, `repos.browse_root`), z kartą lub bez.

Otwiera się na ostatnim zakresie, który tu wybrałeś (zapisanym w `radar.toml`), dopóki nadal istnieje; przed
pierwszym wyborem albo gdy ten obszar roboczy usunięto, na aktywnym obszarze roboczym sesji, a w przeciwnym razie na
otwartych kartach. Wybór dotyczy tylko tego widoku: nie zmienia aktywnego obszaru roboczego okna. Gdy dwa repozytoria mają tę samą nazwę folderu,
do znacznika dodawany jest folder nadrzędny, aby dało się je odróżnić.

### Dwa zakresy folderów

Zakres folderu szuka repozytoriów **na dwóch poziomach**: w folderach bezpośrednio w nim i w folderach wewnątrz
tych, które same nie są repozytoriami (`~/src/work/app`). Foldery ukryte są pomijane, do repozytorium się nie
wchodzi, a na liście jest najwyżej 200 repozytoriów. Szukanie działa w tle, nigdy w wątku, który rysuje ekran,
a jego wynik zostaje do odświeżenia: przycisk Odśwież, `r`, `F5` (klawisz odświeżania okna, do zmiany) lub `Radar: Odśwież` przeszukuje folder od nowa
(repozytorium sklonowane od tamtej pory pojawi się na liście).

Hosting pozostaje opcjonalny dla każdego repozytorium: repozytorium z folderu, które **nie jest powiązane**
z hostem, nigdy nie jest odpytywane. Dokłada tylko swój wiersz pracy w toku, ma wiersz `niepowiązane` z akcją
**Powiąż...** w sekcji Repozytoria (otwiera sekcję pull requestów tego repozytorium, gdzie wybierasz dostawcę
i projekt), a pasek stanu je zlicza (`3 niepowiązane (tylko praca w toku)`).

Folder, którego nie ustawiono, jest wyszarzony na liście (`Domyślny folder przeglądania (ustaw w Ustawieniach)`):
wybranie go otwiera jego wiersz w Ustawieniach (Ustawienia > Ogólne) i zostawia zakres bez zmian. Folder
klonowania liczy się jako ustawiony dzięki domyślnej wartości dla systemu (folder, od którego zaczyna okno
Klonuj), o ile ten folder istnieje; folder przeglądania nie ma wartości domyślnej. Lista zakresów jest czytana
z ustawień przy każdym otwarciu, więc zmiana dokonana tam działa od razu.

## Lista

| Grupa | Co zawiera |
|---|---|
| Przypięte | Wszystko, co przypiąłeś, bez względu na rodzaj, w kolejności przypięcia |
| Czekają na mój przegląd | Requesty, które proszą o twój przegląd (i których nie napisałeś) |
| Moje requesty | Requesty, które napisałeś; znacznik mówi `szkic` dla szkicu |
| Przypisane do mnie | Requesty przypisane do ciebie |
| Wspominają o mnie | Otwarte requesty i zgłoszenia, które cię wspominają i nie są jeszcze w grupie powyżej (zgłoszenie, które nie jest ani przypisane do ciebie, ani przez ciebie utworzone, trafia tu tylko wtedy, gdy cię wspomina) |
| Pozostałe requesty | Pozostałe otwarte requesty |
| Zgłoszenia przypisane do mnie | Otwarte zgłoszenia przypisane do ciebie |
| Zgłoszenia utworzone przeze mnie | Otwarte zgłoszenia, które utworzyłeś |
| Praca w toku | Repozytoria zakresu z niezatwierdzonymi zmianami lub niewypchniętymi commitami |
| Odłożone | Elementy, które ukryłeś na jakiś czas; zwinięte, dopóki ich nie otworzysz |
| Repozytoria | Repozytoria, które mają coś do powiedzenia (offline, token do odnowienia, niepowiązane) |

Słowa „pull request” i „merge request” podążają za zwykłym ustawieniem terminologii i hostem.

Każdy wiersz pokazuje rodzaj elementu (`⇄` request, `●` zgłoszenie, `✎` praca w toku),
znacznik repozytorium (w kolorze obszaru roboczego), numer, tytuł, autora, wiek i, gdy
jest miejsce, etykiety. Wiersz pracy w toku pokazuje gałąź i to, co w niej czeka: `✎`
zmienione pliki, `+` nowe pliki, `↑` commity do wypchnięcia, `↓` commity do ściągnięcia.

Liczby są w każdym nagłówku. W obrębie grupy pierwszy jest element ostatnio zmieniony.

Pasek stanu pod filtrem pokazuje wskaźnik postępu, gdy hosty odpowiadają dłużej niż
chwilę. Listy z pamięci podręcznej (tej samej, której używa pasek boczny) są widoczne od razu
i wymieniane w miarę nadchodzenia odpowiedzi.

### Kolumny

Znacznik repozytorium, gałęzie, `#` (numer i znaczniki kontroli), autor i wiek to **kolumny o regulowanej szerokości**. Na automatycznej kolumna repozytorium jest
tak szeroka jak najdłuższa nazwa na ekranie (do 40 komórek i najwyżej jedna trzecia okna), więc `owner/name` oraz
`group/sub/project` z listy konta nie są ucinane na zwykłym terminalu. Tytuł dostaje resztę. Nazwa, która się
nie mieści, kończy się wielokropkiem (`group/sub...`), a ponadto:

- **Zatrzymaj wskaźnik** na uciętym repozytorium, tytule, autorze lub liście etykiet, a podpowiedź pokaże cały
  tekst. Zaznacz wiersz, a pasek stanu pod filtrem poda też pełny projekt i tytuł, więc widzi to też
  klawiatura.
- **Przeciągnij separator** (komórka tuż za nazwą repozytorium, za `#` albo tuż przed autorem lub wiekiem), by ustawić szerokość
  kolumny; **kliknij go dwukrotnie**, by znów dopasować kolumnę. Lista ma **wiersz nagłówka** (repozytorium, gałęzie, `#`,
  tytuł, autor, wiek), a separatory podążają za `ui.column_resize` jak w Logu: przy `line` (domyślnie) przygaszony `│`
  spoczywa w separatorze w wierszu nagłówka, a gdy wskaźnik jest na separatorze (w nagłówku lub w dowolnym wierszu listy),
  gdy kolumna jest wybrana klawiszami albo gdy przeciągasz, cienka linia akcentu rysuje się w dół całej listy; przeciągnięcie
  z dowolnego jej punktu zmienia szerokość. Po puszczeniu linia znika, chyba że wskaźnik nadal jest na separatorze (wtedy znika
  razem z nim): przeciągnięcie myszą nigdy nie zostawia wybranej kolumny. Przy `header` nie ma znacznika ani linii: pierwsza
  komórka za separatorem, w wierszu nagłówka, zapala się i jest jedynym miejscem do przeciągania. Wskaźnik zmienia się
  we wskaźnik zmiany rozmiaru nad separatorem w terminalach, które to obsługują. Chip stanu ma
  stałą szerokość, podobnie wiek (najszerszy pokazany wiek, dopóki go nie ustawisz; wiek ucięty przez wąską kolumnę pokazuje
  pełną datę w podpowiedzi), więc autor zaczyna się w tej samej komórce w każdym wierszu.
- Albo użyj klawiszy: **`C`** wybiera kolumnę (ponownie lub `Tab` - następną), **`]`** poszerza ją o dwie
  komórki, **`[`** zwęża, **`0`** przywraca wszystkim kolumnom tryb automatyczny, `Esc` kończy. Paleta ma to
  samo: `Radar: Wybierz kolumnę`, `Radar: Poszerz kolumnę`, `Radar: Zwęź kolumnę`, `Radar: Resetuj szerokości kolumn`.

Szerokości są zapisane w `radar.toml` (`[columns]`, `repo`, `branches`, `number`, `author` i `age`, w komórkach; plik bez `branches` wczytuje się jak dawniej, kolumna dopasowuje się wtedy do zawartości do 28 komórek) i przetrwają restart. To te
same gesty zmiany rozmiaru co w kolumnach Log.

### Kontrole i konflikty

Wiersz requesta niesie po numerze dwa małe znaczniki, gdy host coś o tym powiedział:

| Znacznik | Znaczenie |
|---|---|
| `✓` | kontrole commita head zaliczone |
| `✗` | któraś kontrola nie przeszła |
| `●` | kontrole czekają w kolejce lub trwają |
| `!` (obok niego) | request ma konflikt ze swoją gałęzią docelową |

Zatrzymaj wskaźnik na znacznikach, a podpowiedź powie to słowami („Kontrole nie przeszły”, „Konflikty z main”);
zaznacz wiersz, a pasek stanu pod filtrem powie to samo, więc widzi to też klawiatura. Request
bez znacznika to taki, o którym host nic nie powiedział (brak skonfigurowanego CI albo nie zapytano).

- **GitLab** wysyła status pipeline i flagę konfliktu razem z listą merge requestów: każdy wiersz ma
  swoje znaczniki.
- **GitHub** tego nie robi: lista nie ma żadnego z nich. GitPoulpi pyta o nie osobno dla każdego requesta (o sam request
  dla jego stanu scalenia, o połączony status commita head i jego check runy: trzy wywołania), dla najwyżej
  **15 otwartych requestów na repozytorium**, twoje najpierw. Jeśli GitHub odrzuci jedno z tych wywołań (limit),
  reszta zostaje bez znaczników do następnego odświeżenia. Odpowiedzi są zapisywane razem z pamięcią podręczną hostingu.

- **Gitea i Forgejo** są czytane jak GitHub: lista niesie flagę konfliktu (`mergeable`), status commita jest
  pobierany osobno dla każdego requesta (najwyżej 15 na repozytorium, twoje najpierw), a zgłoszenia to te same grupy.
  Wzmianki korzystają z wyszukiwania serwera (`mentioned_by`).

### Wzmianki

**Wspominają o mnie** i filtr **Pokaż: Wspominają o mnie** korzystają z tego, co host wie o wzmiankach. GitHub jest
pytany przez wyszukiwanie (`mentions:`), GitLab przez twoją listę zadań: liczą się tylko wzmianki nadal **oczekujące**
na niej, więc zadanie oznaczone jako zrobione opuszcza grupę.

Zamknięte i scalone requesty oraz zamknięte zgłoszenia nie są wymieniane.

## Widoki i filtry

Cztery listy rozwijane i pole tekstowe zawężają listę. Każdy wybór to lista rozwijana:
kliknij ją albo użyj jej klawisza.

- **Widok** (`v`): Wszystko, requesty, Zgłoszenia, WIP (praca w toku), Odłożone.
- **Pokaż** (`f`): Wszyscy, Moje (requesty, które napisałeś, zgłoszenia, które utworzyłeś),
  Przypisane do mnie, Poproszono o przegląd, Wspominają o mnie.
- **Zakres** (`w`): zob. wyżej.
- **Profil** (`P`): Wszystkie profile albo jeden profil integracji z Ustawienia > Integracje (nazwany tak jak w
  Ustawieniach, z dopisanym dostawcą, gdy dwa profile mają tę samą nazwę). Zawęża każdą sekcję, wiersze repozytoriów
  i wiersze obejmujące całe konto, do wierszy tego profilu (wiersz pracy w toku nie ma profilu, więc wybrany profil go
  ukrywa). Jeden profil naraz. Wybór jest zapamiętywany w `radar.toml`; profil, który już nie istnieje, wraca do Wszystkie profile.
  Lista rozwijana jest zawsze, jak pozostałe trzy, a wąskie okno skraca ją (a potem etykiety), zamiast ją pomijać.
  Paleta: `Radar: Filtruj według profilu...`. Gdy profil ukrywa wszystko, pusta lista to mówi.
- **Tekst filtra** (`/` albo `Ctrl+F`): przybliżone dopasowanie do repozytorium, numeru,
  tytułu, autora, gałęzi i etykiet. `Enter` oddaje klawisze liście i zachowuje tekst,
  `Esc` go czyści.

`Ctrl+N` pokazuje plakietkę na filtrze, każdej z czterech list rozwijanych, przyciskach Odśwież i Wstecz oraz każdym nagłówku sekcji widocznym na ekranie. Naciśnij znak plakietki: filtr dostaje klawiaturę, lista rozwijana się otwiera, przycisk jest wykonywany, a nagłówek dostaje zaznaczenie (`Enter` lub `Space` zwija go potem). `Esc` anuluje. Zob. [Plakietki](keybindings.md#plakietki).

## Akcje

Zaznacz wiersz i użyj jego menu (prawy przycisk, `⋮` na końcu wiersza, `.` albo `Menu`),
klawisza albo palety (`Radar: ...`). Akcja myszą zawsze ma klawisz i odpowiednik
w palecie.

| Akcja | Klawisz | Paleta |
|---|---|---|
| Otwórz repozytorium (jego karta się otwiera albo jest przełączana; żądanie, które nie jest sklonowane, pyta, jak je przejrzeć) | `Enter` / podwójne kliknięcie | `Radar: Otwórz repozytorium` |
| Przejrzyj... (żądania; żądanie z niesklonowanego repozytorium pyta, jak) | `e` | `Radar: Przejrzyj żądanie...` |
| Przejrzyj żądanie przez API hosta (bez klonu) | `a` w pytaniu | `Radar: Przejrzyj przez API` |
| Sklonuj repozytorium, potem przejrzyj żądanie | `c` w pytaniu | `Radar: Sklonuj i przejrzyj...` |
| Otwórz w przeglądarce | `o` | `Radar: Otwórz w przeglądarce` |
| Checkout gałęzi (requestu) | `c` | `Radar: Checkout gałęzi` |
| Utwórz worktree... (requestu) | `t` | `Radar: Utwórz worktree...` |
| Pokaż w repozytorium (jego lista) | `s` | `Radar: Pokaż w repozytorium` |
| Zacznij pracę nad zgłoszeniem... (zgłoszenia) | `n` | `Radar: Zacznij pracę nad zgłoszeniem...` |
| Przypnij na górze albo Odepnij | `p` / kliknij `★` | `Radar: Przypnij lub odepnij` |
| Edytuj przypisanych... / recenzentów... / etykiety..., Ustaw kamień milowy... (GitHub, GitLab) | menu wiersza | `Radar: Edytuj przypisanych...`, `Radar: Edytuj recenzentów...`, `Radar: Edytuj etykiety...`, `Radar: Ustaw kamień milowy...` |
| Odłóż... | `z` | `Radar: Odłóż...` |
| Przestań odkładać | `u` | |
| Pokaż lub ukryj żądania z niesklonowanych repozytoriów | `a` | `Radar: Pokaż niesklonowane żądania` |
| Pokaż lub ukryj szkice w wierszach obejmujących całe konto | `d` | `Radar: Pokaż szkice` |
| Włącz żądania do przeglądu z całego konta hosta | `T` / `Enter` na jego wierszu podpowiedzi / **Włącz** | `Radar: Włącz żądania do przeglądu z całego konta...` |
| Odśwież | `r`, `F5` | `Radar: Odśwież` |
| Menu wiersza | `.` | `Radar: Menu wiersza...` |
| Wróć do Home | `Esc` | `Radar: Wróć do strony głównej` |

Akcje, które potrzebują własnego okna repozytorium (checkout, worktree, pokaż, zacznij pracę,
powiąż, selektory metadanych), najpierw przełączają na jego kartę i uruchamiają się tam po jej wczytaniu:

- **Checkout** używa własnego checkoutu paska bocznego: gałąź, gdy jest znana lokalnie,
  w przeciwnym razie head requestu pobrany do lokalnej gałęzi (na przykład z forka).
- **Utwórz worktree...** otwiera okno Utwórz worktree na gałęzi requestu.
- **Pokaż w repozytorium** zaznacza element na liście PULL REQUESTS albo ISSUES.
- **Zacznij pracę nad zgłoszeniem** otwiera pytanie o nową gałąź z nazwą utworzoną ze
  zgłoszenia (`12-fix-the-thing`).

### Przypinanie i odkładanie

**Przypięcie** trzyma element na górze. **Odłożenie** ukrywa go na **1 godzinę**, **4 godziny**,
**Do jutra** (08:00 następnego ranka, według twojego czasu lokalnego, także gdy teraz nie ma jeszcze 08:00) albo **Do pierwszej zmiany** (host zgłasza zmianę elementu albo, dla
wiersza pracy w toku, ruszają się jego liczniki). Oba są zapisane w `radar.toml` w folderze
konfiguracji, według hosta, repozytorium, rodzaju i numeru, i są wspólne dla każdej sesji na
komputerze.

### Repozytoria, które mają coś do powiedzenia

Repozytorium z problemem dostaje własny wiersz na końcu listy, a jego elementy z pamięci
podręcznej zostają na ekranie:

- **offline, sprzed 5m**: nie udało się połączyć z hostem;
- **Połącz ponownie**: token jest brakujący lub odrzucony; przycisk otwiera Ustawienia > Integracje;
- **wstrzymane w tej rundzie**: host kazał zwolnić (albo odrzucił token), więc GitPoulpi nie pytał
  go ponownie o inne repozytoria tej rundy;
- **niepowiązane, sugerowany GitHub**: zdalne repozytorium pasuje do profilu, ale hosting jest
  włączany osobno dla każdego repozytorium, więc nic nie jest odpytywane, dopóki go nie powiążesz.
  **Powiąż...** prowadzi do sekcji PULL REQUESTS repozytorium, gdzie wybierasz dostawcę
  i projekt.

Odpytywane jest jednocześnie najwyżej cztery repozytoria i nigdy w wątku, który rysuje ekran.

## Poza zakresem: wiersze obejmujące całe konto

Zakres obejmuje repozytoria, które sklonowałeś. Żądanie czekające na twój przegląd w repozytorium, którego
nigdy nie sklonowałeś, jest tam niewidoczne. **Wiersze obejmujące całe konto** pytają sam host, raz na profil, i
dołączają do listy niezależnie od zakresu: dotyczą ciebie, a nie folderu.

Są **domyślnie wyłączone** (wyszukiwanie hosta kosztuje limit API). Dopóki połączony host kodu nie ma
włączonej żadnej ze swoich list, na górze listy jest dla niego linia:

`GitHub: pokaż żądania czekające na Twój przegląd, także w repozytoriach, których nie sklonowano  [Włącz]`

Kliknij **Włącz**, naciśnij `Enter` na tej linii, naciśnij `T` albo uruchom `Radar: Włącz żądania do przeglądu
z całego konta...`: zapisuje `integrations.radar.review_requests = true` dla tego profilu w `config.toml` i
od razu pyta host. Pozostałe listy (każde otwarte żądanie twoich przestrzeni nazw, zgłoszenia przypisane do ciebie,
szkice, przestrzenie nazw) ustawia się przez `gitpoulpi hosting integrations radar` albo w ustawieniach profilu; zob.
[opis konfiguracji](config-reference.md) i [opis CLI](cli-reference.md). Odpowiadają GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center i Gerrit; Bitbucket Cloud nie ma punktu końcowego dla całego
konta, więc jego requesty pokazują się tylko dla powiązanych repozytoriów zakresu.

**Przestrzenie nazw** profilu ograniczają wszystkie jego listy do wybranych repozytoriów (Ustawienia > Integracje > Radar: pole tekstowe z jednym wpisem w wierszu i pole wyboru **Wyrażenia regularne**). Wiersz pasuje do pełnej ścieżki repozytorium, gdy jest jej równy albo jest jej prefiksem kończącym się na `/`: `orbit` zachowuje `orbit/products/x`, a odrzuca `orbitx/y` i `training/y`. Przy włączonym polu każdy wiersz jest wyrażeniem regularnym uruchamianym na pełnej ścieżce, tak jak ją zapisujesz (`^orbit/(products|infra)/`); wiersz nieprawidłowy jest pokazany pod polem i nie jest zapisywany. Puste oznacza brak limitu.

Jak wiersze są umieszczane:

- Wiersz, który zakres już wymienia (ten sam host, projekt, rodzaj i numer), pojawia się raz: **wygrywa
  wiersz zakresu**.
- Wiersz, którego projekt sklonowałeś gdzieś (ostatnie repozytoria, obszary robocze, foldery znalezione
  przez zakresy folderów; dopasowanie po adresie zdalnym), niesie ten klon: **Enter**, checkout, worktree i
  pozostałe otwierają go jak każdy wiersz.
- Każdy inny wiersz jest **niesklonowany**: plakietka repozytorium jest przyciemniona i pokazuje projekt (`acme/app`), a
  linia stanu po zaznaczeniu mówi `Niesklonowane`. **Otwórz w przeglądarce** działa, **Przejrzyj** pyta, jak
  go przejrzeć (niżej), a checkout, worktree, pokaż w repozytorium i zacznij pracę mówią
  `najpierw sklonuj repozytorium`.

### Przegląd żądania

**Przejrzyj** (`e`) na żądaniu z repozytoriów własnego zakresu otwiera kartę tego repozytorium z przeglądem
w niej. Przy żądaniu z wiersza obejmującego całe konto sposób zależy od tego, czy znany jest klon, oraz od
ustawienia profilu **Żądanie z repozytorium, które nie jest sklonowane** (Ustawienia > Integracje, `review_mode`
w [opisie konfiguracji](config-reference.md)):

| Ustawienie | Klon jest znany | Klon nie jest znany |
|---|---|---|
| Zapytaj (domyślnie) | pyta: **Otwórz lokalny klon** (`l`) lub **Przejrzyj przez API** (`a`) | pyta: **Sklonuj, potem przejrzyj** (`c`) lub **Przejrzyj przez API** (`a`) |
| Sklonuj | otwiera klon | otwiera okno Klonuj |
| Przejrzyj przez API | przegląda przez API | przegląda przez API |

Pytanie to małe okno: litery wybierają, **Enter** wybiera podświetlony wiersz, `Esc`, **Anuluj**,
`✕` i kliknięcie poza oknem je zamykają, a **Zapamiętaj dla GitHub** (`r`) zapisuje ustawienie profilu, żeby
pytanie nie padało ponownie (wiersz w Ustawieniach zmienia to z powrotem). `Ctrl+N` pokazuje plakietki na jego kontrolkach.

- **Przejrzyj przez API** otwiera przegląd w oknie Home, nad Radarem, z kodem pobranym od
  hosta (zob. [Przegląd bez klonu](review.md#przegląd-bez-klonu)). `Esc` lub `✕` oddaje
  Radar; wiersze palety *Przegląd: ...* na nim działają.
- **Sklonuj, potem przejrzyj** otwiera okno Klonuj na połączonym koncie, z którego pochodzi wiersz (ta sama lista
  repozytoriów, pogrupowana według właściciela), z zaznaczonym repozytorium żądania i wypełnionym domyślnym folderem
  klonowania; zmień jedno lub drugie, potem sklonuj, z tym samym logowaniem co przy każdym klonowaniu z tego konta.
  Jeśli repozytorium nie ma na liście konta (nie jesteś jego członkiem), lista pozostaje otwarta z wyszukiwaniem jego
  nazwy, a notka o tym informuje; dla hosta, który nie potrafi wylistować repozytoriów, okno otwiera się na
  **Klonuj z URL** z wypełnionym adresem. Gdy klon wyląduje, nowe repozytorium zostaje powiązane z hostem
  (jego źródło PULL REQUESTS oraz źródło ISSUES, gdy host ma zgłoszenia, a żadnego nie ustawiono), jego karta się otwiera
  i otwiera się w niej przegląd tego żądania. Anulowanie okna albo nieudany klon niczego nie wiąże.
- Żądania Azure DevOps i Bitbucket nie mają przeglądu w aplikacji: **Przejrzyj** otwiera stronę żądania
  w przeglądarce i mówi o tym w linii stanu.

Wiersze palety *Radar: Przejrzyj przez API* i *Radar: Sklonuj i przejrzyj...* pomijają pytanie.

Dwa pola wyboru obok filtra (pojawiają się, gdy pytana jest lista obejmująca całe konto), zapisywane w
`radar.toml`:

| Pole wyboru | Klawisz | Domyślnie | Efekt |
|---|---|---|---|
| `[x] Niesklonowane` | `a` | włączone | Wypisuje wiersze repozytoriów bez znanego klonu. Wyłączone: zostają tylko wiersze znanych klonów. |
| `[ ] Szkice` | `d` | wyłączone | Wypisuje szkice żądań z wierszy obejmujących całe konto. Szkice z repozytoriów własnego zakresu są zawsze wypisywane. |

Oba są na plakietkach `Ctrl+N` i w palecie (`Radar: Pokaż niesklonowane żądania`, `Radar: Pokaż szkice`).

Odpowiedź profilu jest trzymana w `radar-account.json` przez **5 minut**: ponowne otwarcie Radaru w tym
czasie pokazuje wiersze bez pytania hosta. **Odśwież** (`r` lub `F5`) zawsze pyta: każde repozytorium i każdą listę z całego konta, bez względu na wiek ostatniej odpowiedzi. Host, który kazał zwolnić
albo odrzucił token, nie jest pytany ponownie w tej samej rundzie. Na profil i rodzaj zachowuje się najwyżej 300 wierszy,
a linia pod **Repozytoria** mówi, ile więcej istniało, albo dlaczego nie dało się odczytać profilu
(**Połącz ponownie** otwiera Ustawienia > Integracje).

## Ograniczenia

- Wiersze obejmujące całe konto pochodzą z profili GitHub, GitLab (także self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket Data Center i Gerrit (nie z Bitbucket Cloud). Zgłoszenia przypisane do ciebie pochodzą z pierwszych
  pięciu; Bitbucket i Gerrit nie mają zgłoszeń.
- Listy dla poszczególnych repozytoriów używają profili GitHub, GitLab (także self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket i Gerrit; kontrole, konflikty i wzmianki są czytane tylko dla pierwszych czterech.
- Wzmianki w GitLab pochodzą tylko z oczekujących zadań; kontrole GitHub, Gitea i Forgejo są pobierane dla 15 requestów na repozytorium.
