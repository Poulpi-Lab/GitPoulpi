[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · **Polski** · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Przegląd pull lub merge requesta

Przeglądaj pull request (GitHub, Gitea, Forgejo), merge request (GitLab) albo zmianę Gerrit (zob. [Gerrit](#gerrit))
**bez opuszczania okna repozytorium**. Widok przeglądu pokazuje opis, dyskusję, zmienione
pliki z wątkami komentarzy pod ich liniami oraz commity i tam właśnie piszesz przegląd: komentarze,
akceptację i prośby o zmiany. Wymaga profilu jednego z tych hostów (zob.
[Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md)) i powiązanego z nim repozytorium; inni dostawcy nie
oferują przeglądów. Ograniczenia hosta są respektowane: GitLab nie ma werdyktu *Zażądaj zmian* i przyjmuje jedną linię na komentarz.
Ta strona nazywa je wszystkie *żądaniem*.

![Widok przeglądu, strona Podsumowanie: nagłówek żądania z przyciskami metadanych, opis i dyskusja z wysłaną akceptacją](../images/review-overview.png)

![Strona Pliki: zmienione pliki z polami przejrzenia, diff z wątkiem komentarzy pod linią i rozwiązany wątek poza pokazanymi liniami](../images/review-files.png)

## Otwieranie przeglądu

Każda droga prowadzi do tego samego widoku:

- lista **PULL REQUESTS** na pasku bocznym: zaznacz wiersz i naciśnij **Enter** albo kliknij go prawym przyciskiem
  i wybierz **Przejrzyj...**;
- **Radar**: menu wiersza, **Przejrzyj...**, albo klawisz `e` (otwiera się repozytorium, a w nim przegląd;
  żądanie z repozytorium, którego nie sklonowałeś, pyta, czy je sklonować, czy przejrzeć przez API,
  zob. [Przegląd bez klonu](#przegląd-bez-klonu));
- **wyszukiwanie w obszarze roboczym** (`Ctrl+Alt+F`): podświetl pull lub merge request i naciśnij **`Alt+R`**;
- paleta: **Hosting: Przejrzyj żądanie...** wyświetla żądania do wyboru.

Widok wypełnia okno nad repozytorium (to tryb karty repozytorium, nie osobne okno).
**`Esc`** lub `✕` w rogu go zamyka. Podczas otwierania żądanie jest czytane z hosta, a jego kod
pobierany gitem w tle; jeśli trwa to dłużej chwili, pojawia się spinner.

## Nagłówek

Zawsze widoczny: numer i tytuł, stan (Otwarte, Szkic, Scalone, Zamknięte), gałęzie
(`main ← feature/moons`), autor, kontrole (`✓ kontrole zaliczone`, `✗ kontrole nie przeszły`, `● kontrole w toku`),
informacja, czy scala się bez konfliktów, czy z konfliktami, rozmiar oraz **`3 oczekujące komentarze`**, gdy jakieś napisałeś.

Cztery przyciski metadanych - **Przypisani**, **Recenzenci**, **Etykiety**, **Kamień milowy** - otwierają te same
selektory co pasek boczny (zob. [Przypisani, recenzenci, etykiety i kamień milowy](hosting-integration.md#przypisani-recenzenci-etykiety-i-kamień-milowy));
nagłówek pokazuje wynik, gdy tylko host go przyjmie. Przyciski akcji: **Checkout** (zwykły
checkout gałęzi żądania, który zamyka też widok), **Otwórz w przeglądarce**, **Odśwież**
(**`F5`**) i **Zakończ przegląd**.

## Trzy strony

Przełącznik segmentowy w górnej ramce zmienia stronę: **Podsumowanie**, **Pliki**, **Commity**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`** lub kliknięcie). Przy włączonym słownictwie lokalnym trzecia strona
nazywa się **Zatwierdzenia**.

### Podsumowanie

Opis (podstawowy Markdown jest pokazany jako taki: pogrubienie, kursywa, `kod`, odnośniki jako ich tekst, listy
punktowane i numerowane, nagłówki, cytaty i bloki kodu), potem dyskusja: komentarze, które nie dotyczą kodu, i
już wysłane przeglądy, od najstarszego, każdy z werdyktem (`zaakceptowano`, `zażądano zmian`, `skomentowano`,
`uchylono`). W GitLab werdyktami są jego akceptacje. Pod spodem pole odpowiedzi: kliknij je lub naciśnij **`c`**,
aby napisać komentarz do dyskusji. **Ten jest publikowany od razu**, a pole to mówi.

### Pliki

Zmienione pliki po lewej (litera stanu, `+dodane -usunięte`, licznik `●n` otwartych wątków, licznik `✎n`
twoich oczekujących komentarzy) i diff zaznaczonego pliku po prawej. Diff pochodzi z gita: head żądania
jest pobierany do `refs/gitpoulpi/pr/<n>` (nigdy gałąź, nic nie jest wyewidencjonowywane) i porównywany z
bazą scalenia, więc numery linii na marginesie są dokładnie tymi, których używają komentarze.

- **`[ ]` przed plikiem**: oznacz go jako *przejrzany* (kliknięcie albo **`v`** / `Spacja` na liście). Tytuł je zlicza.
  Znaczniki przejrzenia są pamiętane osobno dla każdego repozytorium i przetrwają restart.
- **Wątki** leżą tuż pod swoją linią: pierwszy komentarz, odpowiedzi i przyciski **Odpowiedz** oraz
  **Rozwiąż** / **Otwórz ponownie**. Rozwiązany wątek zwija się do jednego wiersza; **`z`** (albo `Enter` na nim, albo kliknięcie
  `▾`) zwija i rozwija dowolny wątek.
- **Nieaktualne wątki** i wątki przy liniach, których diff nie pokazuje, są wypisane po hunkach pod nagłówkiem
  *Komentarze poza pokazanymi liniami*, ze znacznikiem `nieaktualny`, gdy kod zmienił się pod nimi.
- **`c`** komentuje linię, na której stoi kursor (mały `+` na początku linii robi to samo
  myszą). **`Shift+Up` / `Shift+Down`** (albo przeciągnięcie po liniach) najpierw zaznaczają kilka linii,
  a komentarz obejmuje cały zakres. GitLab przyjmuje jedną linię na komentarz i widok mówi o tym zamiast zaznaczać.
- **`r`** odpowiada w wątku pod kursorem, **`x`** go rozwiązuje albo otwiera ponownie. Oba są wysyłane do hosta **od
  razu**.
- Klawisze: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` przesuwają; `Tab`, `Right` i `Enter` przechodzą z listy plików
  do diffa, `Left` i `Tab` wracają; `]` i `[` skaczą do następnego i poprzedniego pliku.

### Opcje wyświetlania diffa

Panel diffa stron **Pliki** i **Commity** to własny panel [widoku diff](staging-and-committing.md), z tym samym paskiem w pierwszym
wierszu i tymi samymi zapamiętanymi wyborami (to jedno ustawienie: to, co wybierzesz tutaj, jest tym, z czym otwiera się widok diff,
i odwrotnie):

- **W linii / Obok siebie**: jedna kolumna albo przed i po obok siebie. W układzie obok siebie wątek, oczekujący komentarz lub
  sugestia leży pod tą połową, do której należy: komentarz do usuniętej linii pod lewą połową, komentarz do
  dodanej lub niezmienionej linii pod prawą. Kliknij połowę, o którą ci chodzi, zanim naciśniesz **`c`** w wierszu, który
  pokazuje usunięcie i dodanie; klawiatura komentuje po stronie nowej.
- **Hunki**: tylko zmiany z kontekstem albo cały plik ze zmianami na swoich miejscach. W całym pliku
  wątek przy linii poza zmianami leży pod swoją linią zamiast w *Komentarze poza pokazanymi liniami*.
  Tekst pliku jest czytany w tle; hunki zostają na ekranie z linią o tym, dopóki nie dotrze (na
  stronie Commity w ten sposób czytanych jest do 40 plików commita).
- **WS**: pokaż albo zignoruj zmiany różniące się tylko białymi znakami.
- **Wrap**: zawijaj długie linie.
- **Strzałki w górę / w dół**: poprzedni i następny hunk.
- **Map** (**`M`**): minimapa obok paska przewijania, jedno ustawienie z widokiem diff. Kliknij ją lub przeciągnij, aby
  przewijać.
- **Auto** (**`E`** albo *Przegląd: Wybierz kodowanie...*): kodowanie, w jakim czytany jest plik, jak w widoku diff:
  Wykryj automatycznie albo jedno z kodowań z listy, wybrane dla pliku, który pokazuje panel (na stronie Commity plik
  pod kursorem). Obie strony pliku są czytane ponownie w tym kodowaniu, w tle, a
  wybór jest wspólny z widokiem diff dla tej samej ścieżki. Plik, którego nie da się tak odczytać, mówi dlaczego nad swoim
  diffem.
- **`⋯`**: gdy panel jest za wąski na wszystkie przyciski, te, które się nie mieszczą, są za `⋯` na końcu
  paska; jego lista uruchamia każdy tak, jakby go kliknięto. Ich klawisze i wiersze palety działają jak zawsze.

Linijka przewijania pokazuje, gdzie co jest: zmiany (tylko cały plik, jak w widoku diff), kropka `●`
dla każdego wątku, oczekującego komentarza i sugestii oraz kursor.

Długie linie są obcinane na krawędzi panelu (włącz **Wrap**, aby zobaczyć je w całości) i przewijają się w bok: **`Right`**
i **`Left`** na diffie (`Left` na lewej krawędzi wraca do listy plików), kółko poziome albo `Shift`
z kółkiem, pasek przewijania wzdłuż dolnej ramki lub *Przegląd: Przewiń diff w lewo / w prawo*.

### Zaznaczanie tekstu i linii

O tym, co zaznacza przeciągnięcie, decyduje miejsce jego początku. Przeciągnięcie zaczęte **w kodzie** zaznacza tekst, jak w widoku diff
(podwójne kliknięcie zaznacza słowo); **`Ctrl+C`** (albo *Przegląd: Kopiuj zaznaczony tekst lub linię*) kopiuje dokładnie te
znaki, bez numerów linii i znaczników, a bez zaznaczenia tekstu kopiuje linię pod kursorem.
Przeciągnięcie zaczęte **na rynnie** (numery linii, znacznik, `+`) zaznacza linie, by skomentować je tak jak
wcześniej, a podwójne kliknięcie na rynnie nadal zaczyna komentarz tej linii.

Klawisze widoku diff `n`, `p`, `m`, `w`, `s` należą do samego przeglądu, więc w przeglądzie przełączniki to wielkie
litery: **`V`** układ, **`H`** hunki albo cały plik, **`S`** białe znaki, **`W`** zawijanie, **`J`** / **`K`** następny i
poprzedni hunk, **`E`** kodowanie, **`M`** minimapa. Każdy ma wiersz w palecie (*Przegląd: Przełącz układ w linii / obok siebie*,
...).

### Commity

Tylko commity żądania (nie historia za nim), od najnowszego, i diff zaznaczonego z nich
(jego zmiany względem rodzica), ze wszystkimi plikami jeden pod drugim i tymi samymi opcjami wyświetlania.

## Pisanie przeglądu

Komentarz do kodu jest **oczekujący**: jest przechowywany na twoim komputerze (w pliku stanu repozytorium, więc przetrwa
restart) i liczony w nagłówku. Nic nie jest wysyłane, dopóki nie zakończysz przeglądu. Oczekujący komentarz jest pokazany
pod swoją linią ze znacznikiem `Oczekujący - linia 12` i dwoma przyciskami: **`e`** (Edytuj) i **`d`** (Usuń).

**Zakończ przegląd** (**`f`**, przycisk albo paleta) otwiera mały formularz: podsumowanie (opcjonalne) i
lista **Werdykt**: *Komentarz*, *Zaakceptuj*, *Zażądaj zmian*. Wyświetlane jest tylko to, na co pozwala host: GitLab nie ma
*Zażądaj zmian*. **Wyślij przegląd** (`Ctrl+S`) wysyła podsumowanie, werdykt i każdy oczekujący komentarz za
jednym razem. **Odrzuć przegląd...** wyrzuca oczekujące komentarze po potwierdzeniu.

Od razu, bez czekania na przegląd, wysyłane są: odpowiedzi, rozwiązywanie i ponowne otwieranie oraz komentarze do
dyskusji. Pole komentarza mówi, który to rodzaj (*Ten komentarz jest publikowany od razu* albo *Ten komentarz trafia do twojego
oczekującego przeglądu*). Jeśli host odrzuci odpowiedź, pole otwiera się ponownie z twoimi słowami.

Jeśli żądanie dostało nowe commity po napisaniu oczekujących komentarzy, widok ostrzega, że ich linie mogły
się przesunąć (GitHub i GitLab umieszczają komentarz na commicie, który czytałeś; przegląd starszego head jest odrzucany).

## Gerrit

Serwer Gerrit łączy się jak inne hosty (zob. [Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md): URL
serwera, nazwa użytkownika, hasło HTTP) i jest powiązany pod **PULL REQUESTS** z *projektem*. Jego zmiany są wypisane jak pull
requesty (`#7`, znacznik `GR`) i przeglądane w tym samym widoku, z różnicami, które przynosi Gerrit:

- **Patch sety.** Zmiana ma numerowane patch sety (każde wysłanie commita to jeden). Nagłówek ma pole
  **`Patch set N`** na górnej ramce, obok kart stron (**`P`** albo wiersz palety *Przegląd: Wybierz patch set...*).
  Zaczyna od najnowszego; wybranie innego czyta pliki i kod tego patch setu, a
  wątki napisane na innych patch setach są wypisane jako *nieaktualne*. Przegląd, który kończysz, jest wysyłany na patch secie,
  który jest pokazany.
- **Porównanie z innym patch setem.** Drugie pole po lewej od tego, **`Porównaj: baza`** (**`D`** albo *Przegląd: Porównaj
  patch sety...*), wypisuje bazę i każdy patch set starszy od pokazanego. Wybierz patch set 1, gdy pokazany jest patch set 3,
  a strona Pliki wypisze i pokaże tylko to, co patch set 3 zmienił od patch setu 1 (własna różnica serwera, więc rebase na
  nowszy cel nie wygląda jak zmiany); strona Commity pokazuje tę jedną różnicę. Oba patch sety są pobierane do
  `refs/gitpoulpi/pr/<n>` i `refs/gitpoulpi/pr/<n>-ps<k>`. Wątki napisane na innych patch setach niż pokazany są
  *nieaktualne*. Wybranie do czytania starszego patch setu niż porównywany przywraca porównanie z bazą. Odpowiednik w CLI to
  `review show|fetch --compare-with K`.
- **Wprowadzanie i porzucanie.** Gdy serwer mówi, że zmianę można wprowadzić (są głosy i kontrole, których potrzebuje), do
  przycisków nagłówka dochodzi **Wprowadź**; **Porzuć** jest dostępny, dopóki wolno ci zmianę porzucić. Oba (klawisze **`U`**
  i **`Q`**, paleta *Przegląd: Wprowadź zmianę...* / *Przegląd: Porzuć zmianę...*) najpierw pytają, w oknie, które mówi, że
  akcja dzieje się na serwerze i **GitPoulpi nie może jej cofnąć** (wprowadzona zmiana zostaje wprowadzona; porzuconą można
  przywrócić w interfejsie WWW Gerrita). Odmowa nic nie wysyła; po sukcesie widok czyta zmianę ponownie. Odpowiedniki w CLI
  to `gitpoulpi hosting review submit-change <n>` i `hosting review abandon <n> [--message M]`.
- **Głosy zamiast werdyktu.** **Zakończ przegląd** pokazuje jedno pole na każdą etykietę, na którą możesz głosować - `Code-Review`
  (-2 do +2) i `Verified`, gdzie projekt ją ma - z samymi wartościami, na które serwer pozwala **dla tej
  zmiany i dla ciebie** (programista bez uprawnień nie wybierze +2). Zaczyna od twojego bieżącego głosu; wysyłany jest tylko głos, który
  zmieniłeś. Podsumowanie i oczekujące komentarze idą tym samym wywołaniem. Szkice zapisane w interfejsie
  webowym Gerrit są publikowane razem z przeglądem.
- **Wątki** to łańcuchy komentarzy Gerrit. Wątek jest rozwiązany, gdy jego najnowszy komentarz ma `unresolved: false`;
  **rozwiąż** wysyła odpowiedź `Done`, która to robi, **otwórz ponownie** odpowiedź `Reopened`. Odpowiedzi i rozwiązywanie idą na serwer
  od razu, jak na innych hostach.
- **Wypchnięcie do przeglądu.** Gerrit tworzy zmianę, gdy wypchniesz do `refs/for/<branch>`. Gdy sekcja PULL REQUESTS
  jest powiązana z projektem Gerrit, przycisk **`+`** sekcji i wiersz palety **Hosting: {$Push} do przeglądu...**
  pytają o gałąź docelową i wypychają `HEAD` do `refs/for/<branch>` (temat można podać w wierszu poleceń).
  Wypchnięcia nie da się cofnąć z GitPoulpi (porzuć zmianę w Gerrit); komunikat tak mówi. Odpowiednik w CLI to
  `gitpoulpi hosting review push`.
- Przyciski metadanych są w Gerrit tylko do odczytu (recenzenci są pokazani), nie ma werdyktu *Zażądaj zmian*
  (głos -1 lub -2 to to samo), a znacznik kontroli to etykieta `Verified`.

## ✦ Szkic przeglądu AI

![Dwie sugestie ✦ pod swoimi liniami na stronie Pliki, każda z Przyjmij, Edytuj i Odrzuć, obok istniejącego wątku](../images/review-ai.png)

Przy włączonym AI (*Ustawienia > AI*, zob. [Wsparcie AI](ai.md)) nagłówek ma przycisk **`✦ Szkic przeglądu...`** (także **`s`**
i wiersz palety *Przegląd: ✦ Przygotuj szkic przeglądu...*). Otwiera [opcje przeglądu](#opcje-przeglądu), a
**`Enter`** w nich pyta model. To jedyna rzecz, która pyta model: samo otwarcie przeglądu
nigdy tego nie robi. Model czyta tytuł i opis żądania, istniejące już komentarze (żeby ich
nie powtarzał) oraz diff i odpowiada **szkicem przeglądu**: tekstem do wysłania, rekomendowanym werdyktem (Zaakceptuj, Zażądaj zmian lub Komentarz), ogólnym podsumowaniem i listą sugerowanych komentarzy. Długi
diff jest dla modelu przycinany - osobno dla pliku i w sumie - a Podsumowanie mówi, ile plików skrócono
lub pominięto. Gdy model pracuje, pasek stanu po chwili pokazuje spinner, a **`Esc`** (albo ten sam przycisk)
go anuluje. Instrukcje *Przegląd kodu* z [czterech poziomów](ai.md#instrukcje) obowiązują, więc zespół albo
obszar roboczy może powiedzieć, jak wygląda dobry przegląd.

### Opcje przeglądu

`s` otwiera okno **✦ Opcje przeglądu** wypełnione wartościami domyślnymi repozytorium (globalnymi tam, gdzie
repozytorium ich nie ma). `Enter` (albo przycisk **Uruchom**) rozpoczyna przebieg z tym, co pokazuje okno; `Esc`, **Anuluj**
lub krzyżyk je zamyka i o nic nie pyta.

| Kontrolka | Działanie |
|---|---|
| **Uruchom agenta w repozytorium** | *Ustawienie profilu* (decyduje folder roboczy profilu AI), *Tak* lub *Nie*. Wyszarzone, z powodem pod spodem, dla profilu HTTP (nie ma folderu roboczego) oraz gdy żądanie jest przeglądane przez API bez klonu. |
| **CLAUDE.md**, **AGENTS.md** | Przekaż modelowi ten plik repozytorium, odczytany w gałęzi, do której zmierza żądanie. Gdy agent działa w repozytorium, czyta je sam i okno o tym mówi. |
| **Dodatkowy plik** | Jeszcze jeden plik: ścieżka w repozytorium, ścieżka bezwzględna lub link do pliku na hoście kodu. **Przeglądaj...** (`Ctrl+B` albo `Enter` na przycisku) otwiera wybór pliku, który zaczyna w katalogu głównym repozytorium (albo obok już podanego pliku); plik w repozytorium jest zapisywany jako ścieżka względem katalogu głównego, plik spoza niego jako ścieżka bezwzględna. Wpisywanie w polu działa jak dotąd. |
| **Brak dodatkowego pliku** | Brak dodatkowego pliku w tym przebiegu, nawet gdy wskazuje go ustawienie globalne lub repozytorium; wpisanie pliku albo wybranie go przez **Przeglądaj...** odznacza to pole. Z **Zapisz jako domyślne dla tego repozytorium** staje się przełącznikiem repozytorium *Brak dodatkowego pliku*. Puste pole liczy się tu jako brak, bo okno startuje od tego, co przebieg odziedziczyłby. |
| **Zapisz jako domyślne dla tego repozytorium** | Zachowuje też wartości na kolejne przeglądy: w pliku stanu repozytorium albo w `remote-reviews.toml` dla repozytorium, które nie jest sklonowane. Zachowywana jest tylko wartość różna od globalnej, więc reszta nadal podąża za *Ustawieniami > AI*. |

Wszystko jest osiągalne przez `Tab`, `Space` przełącza pole wyboru, a pole ścieżki to zwykłe pole tekstowe. Te
same cztery ustawienia są w *Ustawieniach > AI* (*Przegląd kodu*) i w *Ustawieniach repozytorium > AI*; zob.
[Kontekst przeglądu](ai.md#kontekst-przeglądu). To, czego nie dało się przekazać modelowi (brakujący lub
nieczytelny plik dodatkowy), oraz każdy plik skrócony, by zmieścić się w limitach, są wypisane na karcie szkicu.

Sugestie **nie są komentarzami**. Widać je w dwóch miejscach:

- **Pod swoją linią na stronie Pliki**, jako duchy komentarzy oznaczone `✦` z wagą (`informacja`, `ostrzeżenie`, `problem`) i
  trzema przyciskami: **Przyjmij**, **Edytuj** i **Odrzuć**. Sugestia dotycząca linii, której diff nie pokazuje, jest
  zachowywana, nigdy gubiona: jest wypisana po hunkach pod nagłówkiem *Sugestie AI dotyczące całego pliku* i, po
  przyjęciu, ląduje na pierwszej zmienionej linii pliku ze słowami *(dotyczy linii N)* z przodu.
- **W Podsumowaniu**, pod opisem: podsumowanie, a potem lista z tymi samymi przyciskami oraz **Przyjmij wszystkie** /
  **Odrzuć wszystkie**.

**Przyjmij** (`y`) zamienia sugestię w zwykły **oczekujący komentarz** (nadal możesz go edytować lub usunąć);
**Edytuj** (`w`) otwiera pole komentarza ze słowami modelu na początku i przyjmuje to, co zostawisz; **Odrzuć** (`n`)
ją porzuca. **Przyjmij wszystkie** (`A`) nadal tylko wypełnia listę oczekujących. Na stronie Pliki klawisze działają na
sugestię pod kursorem, w Podsumowaniu na pierwszą z listy. **Użyj podsumowania AI** (`i` albo przycisk
w formularzu Zakończ przegląd) wkłada podsumowanie modelu do treści przeglądu; nigdy nie jest wstawiane samo.
**Nic nie jest wysyłane do hosta, dopóki nie wyślesz przeglądu.**

Podsumowanie pokazuje szkic jako kartę nad sugestiami: plakietkę werdyktu, tekst przeglądu i trzy
przyciski.

- **✦ Użyj szkicu...** (`u`) przyjmuje każdą otwartą sugestię jako oczekujący komentarz i otwiera *Zakończ przegląd* już
  wypełnione tekstem i werdyktem szkicu. Nadal sam go czytasz, zmieniasz i wysyłasz; anulowanie
  formularza zachowuje szkic. Limity hosta są respektowane: GitLab nie ma *Zażądaj zmian*, więc formularz tak mówi
  i wraca do *Komentarz*; w Gerrit werdykt staje się głosem Code-Review +1 lub -1, tylko gdy serwer
  dopuszcza dla ciebie taką wartość (w przeciwnym razie żaden głos nie jest ustawiany, o czym mówi formularz).
- **Edytuj tekst...** (`t`) zmienia tekst przeglądu ze szkicu, zanim go użyjesz.
- **Odrzuć szkic** (`N`) wyrzuca tekst, werdykt i pozostałe sugestie.

Szkic jest trzymany dla każdego żądania, więc nadal jest, gdy zamkniesz i ponownie otworzysz przegląd. Gdy
head żądania przesunął się od czasu, gdy model go napisał, karta mówi *Żądanie zmieniło się od czasu napisania tego szkicu.*
Wysłanie przeglądu zużywa szkic.

## Wszystko ma klawisz i wiersz palety

| Akcja | Klawisz | Paleta |
|---|---|---|
| Podsumowanie / Pliki / Commity | `Alt+1` / `Alt+2` / `Alt+3` | *Przegląd: Pokaż podsumowanie*, *Pokaż pliki*, *Pokaż commity* |
| Komentarz do linii lub zakresu | `c` | *Przegląd: Skomentuj linię...* |
| Komentarz do dyskusji | `c` w Podsumowaniu, `C` | *Przegląd: Skomentuj dyskusję...* |
| Odpowiedź w wątku | `r` | *Przegląd: Odpowiedz w wątku...* |
| Rozwiąż lub otwórz ponownie | `x` | *Przegląd: Rozwiąż lub otwórz ponownie wątek* |
| Zwiń lub rozwiń wątek | `z` | *Przegląd: Zwiń lub rozwiń wątek* |
| Edytuj / usuń oczekujący komentarz | `e` / `d` | *Przegląd: Edytuj oczekujący komentarz...*, *Usuń oczekujący komentarz* |
| Oznacz plik jako przejrzany | `v` | *Przegląd: Oznacz plik jako przejrzany* |
| W linii / Obok siebie | `V` | *Przegląd: Przełącz układ w linii / obok siebie* |
| Cały plik / tylko hunki | `H` | *Przegląd: Przełącz cały plik / tylko hunki* |
| Zmiany tylko w białych znakach | `S` | *Przegląd: Pokaż lub ukryj zmiany tylko w białych znakach* |
| Zawijanie linii | `W` | *Przegląd: Przełącz zawijanie linii* |
| Następny / poprzedni hunk | `J` / `K` | *Przegląd: Następny hunk*, *Poprzedni hunk* |
| Następny / poprzedni plik | `]` / `[` | *Przegląd: Następny plik*, *Poprzedni plik* |
| Zakończ / odrzuć przegląd | `f` | *Przegląd: Zakończ przegląd...*, *Odrzuć oczekujący przegląd...* |
| Odśwież | `F5`, `Ctrl+R` | *Przegląd: Odśwież* |
| Otwórz w przeglądarce / checkout | `o` / `b` | *Przegląd: Otwórz w przeglądarce*, *Checkout gałęzi* |
| Przypisani / recenzenci / etykiety / kamień milowy | `a` / `p` / `l` / `m` | *Przegląd: Edytuj przypisanych...* i trzy pozostałe |
| Wybierz patch set (Gerrit) | `P` | *Przegląd: Wybierz patch set...* |
| Porównaj ze starszym patch setem (Gerrit) | `D` | *Przegląd: Porównaj patch sety...* |
| Wprowadź / porzuć zmianę (Gerrit) | `U` / `Q` | *Przegląd: Wprowadź zmianę...*, *Przegląd: Porzuć zmianę...* |
| ✦ Szkic przeglądu... (AI włączone; otwiera opcje przeglądu) | `s` | *Przegląd: ✦ Przygotuj szkic przeglądu...* |
| Przyjmij / edytuj i przyjmij / odrzuć sugestię | `y` / `w` / `n` | *Przegląd: Przyjmij sugestię*, *Edytuj i przyjmij sugestię...*, *Odrzuć sugestię* |
| Przyjmij / odrzuć każdą sugestię | `A` / `N` | *Przegląd: Przyjmij wszystkie sugestie*, *Odrzuć wszystkie sugestie* |
| Użyj podsumowania AI w treści przeglądu | `i` | *Przegląd: Użyj podsumowania AI w treści przeglądu* |
| Użyj ✦ szkicu (wypełnia Zakończ przegląd) | `u` | *Przegląd: ✦ Użyj szkicu...* |
| Edytuj tekst szkicu | `t` | *Przegląd: Edytuj tekst ✦ szkicu...* |
| Odrzuć ✦ szkic | `N` | *Przegląd: Odrzuć ✦ szkic* |
| Zamknij | `Esc` | *Przegląd: Zamknij* |

Otwórz paletę (`Ctrl+P`), gdy widok jest otwarty: jej wiersze działają na nim i mówią, dlaczego któryś nic nie robi
w tej chwili („najpierw ustaw kursor na linii kodu”).

## Przegląd bez klonu

Żądanie z repozytorium, którego nie ma na twoim komputerze, też można przejrzeć. Z Radaru **Przejrzyj**
(`e`) na jego wierszu pyta, jak (zob. [Radar](radar.md#przegląd-żądania)); **Przejrzyj przez API** otwiera
ten sam widok w oknie Home, nad Radarem. Wszystko na tej stronie działa tak samo, z takimi
różnicami:

- Kod (zmienione pliki, diff każdego pliku, commity żądania, diff commita, cały plik) jest
  czytany z API hosta, więc pierwsze spojrzenie na duże żądanie chwilę trwa, a plik, dla którego host nie
  przysyła diffa (binarny lub bardzo duży), mówi o tym zamiast go pokazywać.
- Twoje oczekujące komentarze są trzymane w `remote-reviews.toml` w folderze konfiguracji, pod hostem
  i projektem (`host/project`), a nie w stanie repozytorium. Nadal są, gdy
  zamkniesz i ponownie otworzysz przegląd, a wysłanie przeglądu je zużywa.
- Nie ma wyboru kodowania (*Niedostępne bez klonu*) ani **Checkout** (mówi, by najpierw sklonować
  repozytorium).
- `Esc` lub `✕` w rogu opuszcza widok i oddaje Radar. Paleta (`Ctrl+P`) dociera do
  wierszy widoku tak jak w karcie repozytorium.

Aby zamiast tego zachować klon, wybierz **Sklonuj, potem przejrzyj** (albo wiersz palety *Radar: Sklonuj i przejrzyj...*):
okno Klonuj otwiera się na połączonym koncie żądania z zaznaczonym jego repozytorium, a przegląd
otwiera się w karcie nowego repozytorium, gdy klon wyląduje.

## Z wiersza poleceń i dla agentów

Ten sam przegląd jest dostępny jako `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(i narzędzia MCP z nich zrobione): zob. [Opis CLI](cli-reference.md). `threads` wypisuje też
już wysłane werdykty (`reviews`). `suggest <n>` pyta model (AI włączone) i wypisuje jego `verdict`, `body`, podsumowanie i
sugestie jako JSON, niczego nie publikując, więc skrypt albo agent może wybrać komentarze do przekazania
do `submit --comment`.

Nic z tego nie wymaga klonu: `--project owner/name` (z `--id`, gdy skonfigurowano kilka integracji)
uruchamia `show`, `threads`, `comment`, `reply`, `resolve`, `submit` i `suggest` względem projektu, o którym
bieżący katalog nic nie wie. `hosting review diff <n>` wypisuje diff z hunkami (`--api` czyta go z
hosta nawet wewnątrz klonu), a `hosting file <path> --project P [--ref R]` czyta plik w danej rewizji, na
przykład `CLAUDE.md` projektu. Oba czytają API hosta: GitHub, GitLab, Gitea, Forgejo i Gerrit.
