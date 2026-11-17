[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · **Polski** · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Rozmowy

Widok rozmowy zgłoszenia, elementu pracy, karty lub pull / merge requesta. Wymaga konta i powiązanego repozytorium: zob. [Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md); listy, które go otwierają, są w [Pull requestach i zgłoszeniach](pull-requests-and-issues.md); recenzowanie kodu jest w [Recenzowaniu pull requesta](review.md).

**Rozmowa** to wszystko, co powiedziano o jednym zgłoszeniu, elemencie pracy, karcie, zmianie lub pull / merge
requeście: opis, komentarze, przeglądy, wątki w kodzie i zdarzenia (etykiety, przypisania, zmiany stanu,
pushe). GitPoulpi czyta ją z hosta, pokazuje sformatowaną i pozwala odpowiedzieć bez opuszczania okna.

### Otwieranie

| Skąd | Jak |
|---|---|
| Wiersz ZGŁOSZENIA | `Enter` lub dwukrotne kliknięcie; **Otwórz rozmowę** w menu wiersza; paleta *Hosting: Otwórz rozmowę zgłoszenia...* (lista zgłoszeń na ekranie). **Otwórz w przeglądarce** w menu wiersza zostawia przeglądarkę. |
| Wiersz PULL REQUESTS w Azure DevOps lub Bitbucket | `Enter` lub **Otwórz rozmowę**. W GitHub, GitLab, Gitea, Forgejo i Gerrit wiersz otwiera [przegląd](review.md), którego strona Podsumowanie jest tą samą rozmową (oś czasu, klawisze i pole komentarza pod nią). |
| Karta [tablicy zgłoszeń](pull-requests-and-issues.md#tablica-zgłoszeń) | `Enter` lub dwukrotne kliknięcie. |
| [Radar](radar.md) | `i` na wierszu zgłoszenia lub żądania, **Otwórz rozmowę** w menu wiersza albo *Radar: Otwórz rozmowę zgłoszenia*. |
| Odwołanie w innej rozmowie | Kliknij je (albo `l`, potem wybierz): `#7`, `!12`, `ABC-12` powiązanego źródła otwierają się w aplikacji i trafiają na stos Wstecz. |
| Agent | `gitpoulpi ui show issue <n> --view conversation` (także `ui show pr`), zob. [opis CLI](cli-reference.md). |

**Repozytoria, które nie są sklonowane.** Wiersz Radaru z repozytorium, którego nie masz sklonowanego, też otwiera
swoją rozmowę: widok potrzebuje tylko profilu integracji i projektu, więc Radar wyświetla go nad własnym oknem,
a `Esc` oddaje Radar z powrotem. Pominięte jest to, co wymaga klonu: odwołanie do commita otwiera się na hoście
zamiast w Logu, a *Pozwól AI czytać repozytorium* jest wyszarzone z powodem (zob. [rozdział o AI](ai.md#pomoc-w-pisaniu-na-hoście)).
Żądanie sklonowanego repozytorium ma zamiast tego swój przegląd (`e`), a **Otwórz rozmowę** w Radarze mówi, że nie
ma tu dla niego rozmowy do odczytu; zgłoszenie sklonowanego repozytorium otwiera ją na karcie tego repozytorium.

Zgłoszenie hosta, który żadnych nie ma do odczytu (Bitbucket i Gerrit nie mają zgłoszeń), otwiera przeglądarkę jak
dawniej, a build bez funkcji `conversation` robi tak wszędzie ([własne buildy](custom-builds.md)).

### Widok

Widok na całe okno: nagłówek (stan, kto i kiedy to otworzył, etykiety, liczba komentarzy, **Pokaż zdarzenia**,
**Wstecz**, **Odśwież**, **Otwórz w przeglądarce**), linia **Publikowanie jako**, oś czasu, a pod nią pole komentarza.

![Rozmowa zgłoszenia #7: opis z odnośnikiem do #9, dwa komentarze, zdarzenie etykiety pokazane przez Pokaż zdarzenia i puste pole Nowy komentarz z Pisanie, Podgląd i Podział](../images/issue-conversation.png)

![Strona Przegląd merge requesta !58: opis, nierozwiązany i rozwiązany wątek w kodzie z odnośnikami do pliku i linii, komentarze, Odpowiedz i Cytuj przy zaznaczonym wątku oraz pole komentarza](../images/pr-conversation.png)

- Oś czasu czyta się od najstarszego do najnowszego i zaczyna od najnowszego okna (najwyżej 300 wpisów). **Wczytaj
  starsze** na jej górze (`m`, paleta *Rozmowa: Wczytaj starsze wpisy*) czyta okno wcześniejsze, z liczbą wpisów,
  które jeszcze zostały, gdy host ją podaje.
- Komentarz pokazuje autora, datę, znacznik `edytowano`, znacznik `bot` dla botów, `wewnętrzny` dla notatek
  prywatnych oraz przyciski **Odpowiedz**, **Cytuj**, **Edytuj**, **Usuń** i **Linki**, na które pozwala zaznaczony
  wpis. Wątek w kodzie pokazuje `ścieżka:linia`, `rozwiązany` lub `nierozwiązany`, `nieaktualny` i swoje
  komentarze. Przegląd pokazuje swój werdykt.
- Zdarzenia to po jednej przygaszonej linii (*carol dodał(a) etykietę lfs*). **Pokaż zdarzenia** (`v`) je ukrywa;
  wybór jest pamiętany dla repozytorium. Hosty bez historii zdarzeń (YouTrack, elementy pracy Azure DevOps,
  notatki lokalne) nie mają takiego pola wyboru.
- Tekst jest renderowanym Markdownem: nagłówki, listy zagnieżdżone i z zadaniami, cytaty, alerty w stylu GitHub,
  podświetlony kod, tabele, linki, obrazy jako `[image: alt]`. Tekst, który przyszedł we własnych znacznikach hosta,
  jest najpierw konwertowany (zob. [Konwersja znaczników](#konwersja-znaczników)). Blok kodu `mermaid` dostaje
  przycisk **Otwórz diagram**, który pokazuje źródło diagramu w przewijanym podglądzie (`Enter` je kopiuje); paleta
  ma *Rozmowa: Otwórz diagram wpisu*. Wymaga to buildu z funkcjami `conversation` i `mermaid`.
- W podsumowaniu przeglądu `ścieżka:linia` wątku w kodzie jest linkiem do kodu (`G`, *Rozmowa: Otwórz kod wątku*;
  zob. [Przegląd](review.md)); w rozmowie otwartej osobno (zgłoszenie, wiersz Radaru) nie ma strony kodu, do której można przejść.
- Widok czyta ponownie co 60 sekund i po `F5` (**Odśwież**).

**Klawisze.** Każdy ma przycisk, wiersz menu lub wiersz palety (*Rozmowa: ...*), a `Ctrl+N` numeruje przyciski.

| Klawisz | Akcja |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Poprzedni / następny wpis |
| `PageUp`, `PageDown`, `Home`, `End`, kółko, pasek przewijania | Przewijanie |
| `Tab`, `Shift+Tab` | Oś czasu, pole komentarza, potem przyciski nagłówka |
| `r` | Odpowiedź: do wątku zaznaczonego wpisu albo z jego tekstem w cytacie, gdy host nie ma wątków |
| `Q` | Cytuje zaznaczony wpis w nowym komentarzu |
| `c` | Nowy komentarz |
| `e`, `d` | Edytuje, usuwa jeden z twoich komentarzy (przed usunięciem pada pytanie) |
| `l`, `Enter` | Linki wpisu: `Enter` idzie za jedynym albo otwiera ich menu |
| `y`, `o` | Kopiuje adres wpisu, otwiera go w przeglądarce |
| `n` | Następny nieprzeczytany wpis |
| `v`, `m`, `F5` | Pokaż lub ukryj zdarzenia, Wczytaj starsze, Odśwież |
| `Alt+Left`, `Backspace` | Wstecz po stosie Wstecz |
| `q`, `Esc` | Zamyka (pyta najpierw, gdy pole zawiera tekst) |

### Pisanie

**Nowy komentarz** (`c` albo `Tab` do pola), **Odpowiedz** (`r`), **Cytuj** (`Q`) i **Edytuj** (`e`) używają jednego
edytora Markdown: **Pisanie**, **Podgląd** i **Podział** (`Alt+P` przełącza Pisanie i Podgląd, `Alt+L` włącza Podział
i wymaga okna szerokiego na 100 kolumn). `Ctrl+Enter` (także `Ctrl+S` i `Alt+Enter`) lub **Wyślij** publikuje,
**Anuluj** odrzuca pole, a `Alt+E` otwiera **Pisz z AI**, gdy AI jest włączone ([rozdział o AI](ai.md#pomoc-w-pisaniu-na-hoście)).
Podgląd jest układany tym samym rendererem co oś czasu, z tymi samymi odwołaniami. Gdy host nie przyjmuje Markdown,
stopka podaje, w jakich znacznikach tekst jest wysyłany (*Wysłano jako Jira wiki markup*, *Wysłano jako Textile*).

![Odpowiedź na zgłoszenie pisana w widoku Podział: szkic Markdown po lewej, jego renderowany podgląd po prawej i przycisk Wyślij](../images/issue-reply-composer.png)

- Komentarz jest dodawany do osi czasu od razu i zajmuje swoje miejsce, gdy host odpowie. Jeśli host odmówi, wpis
  znika, a pole otwiera się ponownie z twoim tekstem i powodem.
- Tekst, którego nie wysłano, jest przechowywany dla każdego elementu (`remote-conversations.toml` w folderze
  konfiguracji) i czeka przy następnym otwarciu tego pola; wysłanie lub odrzucenie go usuwa. Zamknięcie pola z
  tekstem najpierw pyta.
- **Odpowiedz** w wątku (wątek przeglądu, dyskusja GitLab, komentarz Linear, wątek Bitbucket lub Azure DevOps)
  publikuje do tego wątku. Gdy host ma tylko płaską listę, **Odpowiedz** zaczyna nowy komentarz z zaznaczonym
  wpisem w cytacie (`> **autor**`), tak samo jak **Cytuj**.
- **Notatka wewnętrzna** (pole wyboru, `Alt+I`): w GitLab i Redmine komentarz widzą tylko członkowie projektu.
  Jest pokazywana tylko na hostach, które mają takie notatki, i tylko przy nowym komentarzu.
- Publikowanie jest działaniem na zewnątrz: nic nie jest zapisywane w dzienniku i **z GitPoulpi nie da się niczego
  cofnąć**. Bez połączenia pole zostaje, **Wyślij** jest wyszarzone z powodem, a nagłówek mówi, że pokazano
  zapisaną kopię.

**Kto może edytować lub usuwać.** Tylko twoje własne komentarze mają **Edytuj** i **Usuń**. Pierwsze słowo ma
host, gdy je daje (`viewerCanUpdate` i `viewerCanDelete` w GitHub, `can_edit` gdzie indziej), potem twój login
porównany z autorem komentarza, bez rozróżniania wielkości liter; komentarze bota App muszą też pochodzić z konta
bota. Dopóki login bota GitHub App nie jest znany, liczą się tylko komentarze opublikowane przez tę instalację
(repozytorium zachowuje ich id): nic nie jest zgadywane z nazwy. W wątku **Edytuj** i **Usuń** działają na ostatnim
komentarzu wątku, który jest twój. Host, który nie umie edytować ani usuwać (zob. [tabelę](#czego-nie-potrafi-każdy-host)),
pokazuje przyciski wyszarzone z powodem.

### Publikowanie jako

Linia pod nagłówkiem mówi, jako kto są publikowane twoje wiadomości, a ta sama linia jest w każdym polu:

| Linia | Znaczenie |
|---|---|
| *Publikowanie jako ann (token osobisty)* | Token profilu: twój własny login (`GET /user`). |
| *Publikowanie jako ci-bot (polecenie tokenu repozytorium)* | Do API hostingu używane jest [polecenie tokenu](hosting-integration.md#8-działanie-jako-bot-github-app) repozytorium. |
| *Publikowanie jako my-app[bot] (bot GitHub App)* | Token instalacji. Nazwa pochodzi z **Publikuj jako**, a w razie braku z hosta. |
| *Publikowanie jako bot GitHub App (jego nazwa nie jest jeszcze znana: ustaw „Publikuj jako” w Ustawieniach)* | Nie skonfigurowano nazwy i jeszcze żadnej nie poznano. |
| *Publikowanie jako ... (bot projektu)* | Token dostępu projektu lub grupy GitLab. |
| *Pisanie jako kai (notatka lokalna)* | Folder notatek Markdown: `local.me` profilu, a w razie braku podłączony użytkownik. |
| *Nieznane jeszcze, jako kto publikujesz. Ustaw „Publikuj jako” w Ustawieniach.* | Nic nie dało się ustalić. |

**Publikuj jako** to nazwa: dla GitHub App jest to `my-app[bot]`. Ustaw ją dla każdego repozytorium w **Ustawienia >
Integracje > Publikuj jako** (`general.hosting_post_as`) albo dla jednego repozytorium w **Ustawienia > Ustawienia
repozytorium > SSH i poświadczenia > Hosting**, co ją nadpisuje. Gdy jest pusta, GitPoulpi pyta GitHub (GraphQL
`viewer { login }`, na co token instalacji może, ale nie musi odpowiedzieć), a w razie braku odpowiedzi poznaje
nazwę z autora pierwszego komentarza, który opublikuje, i przechowuje ją z repozytorium. Token osobisty nigdy jej
nie potrzebuje. Zob. [Działanie jako bot GitHub App](hosting-integration.md#8-działanie-jako-bot-github-app).

### Odwołania krzyżowe

Odwołania w rozmowie są podkreślone i klikalne. Najpierw dopasowywane są powiązane źródła ZGŁOSZEŃ i PULL REQUESTS
repozytorium, więc `ABC-12` powiązanego projektu Jira otwiera się w aplikacji nawet wewnątrz komentarza GitHub;
to, co dodaje własna gramatyka hosta, otwiera jego stronę po potwierdzeniu linku. Id commita otwiera się w Logu,
gdy repozytorium jest sklonowane, a id ma co najmniej 7 znaków, w przeciwnym razie na hoście. Nic wewnątrz
fragmentu kodu, bloku ogrodzonego, linku Markdown, adresu e-mail ani URL nie jest odwołaniem, podobnie jak `C#1`,
`UTF-8`, `SHA-256` i `ISO-8601`.

| Host | Rozpoznawane |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, id commitów |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (projekt siostrzany), `@user`, `~label` i `%milestone` jako chipy, id commitów |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, id commitów |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, wzmianki `{uuid}`, id commitów |
| Gerrit | wartości `Change-Id` (`I` i 40 cyfr szesnastkowych), id commitów |
| Redmine | `#7`, rewizje `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, tylko dla projektu lub zespołu powiązanego źródła |
| Trello | adresy kart (`trello.com/c/...`) |
| Notatki lokalne | `[[note]]`, `[[note\|alias]]` i względne linki `.md` do innej notatki folderu |

`gitpoulpi hosting markdown --body T --id I` wypisuje, co aplikacja robi z tekstem, wraz z odwołaniami.

### Konwersja znaczników

Hosty, które nie mówią Markdownem, są konwertowane w obie strony, więc wszędzie czytasz i piszesz Markdown. To, czego
konwerter nie rozumie, pozostaje czytelnym tekstem.

- **Jira** (Cloud i Data Center) przechowuje komentarze jako wiki markup przez API v2. To, co czytasz, jest
  konwertowane na Markdown; to, co piszesz, jest konwertowane na wiki markup, a stopka mówi *Wysłano jako Jira wiki
  markup*.
- **Redmine** formatuje tekst jako Textile lub Markdown, czego API nie podaje: mówi o tym GitPoulpi wiersz profilu
  **Format tekstu serwera** (Ustawienia > Integracje, `markup`, domyślnie Textile). Textile jest konwertowany na
  Markdown do czytania i z powrotem na Textile do pisania.
- **Azure DevOps Server** zwraca komentarze elementów pracy jako HTML i przyjmuje je jako HTML: są pokazywane jako
  Markdown skonwertowany z niego i zapisywane jako HTML. Azure DevOps Services i wątki pull requestów przyjmują
  Markdown.
- **Gerrit** to zwykły tekst: nie jest renderowany jako Markdown, a to, co piszesz, jest wysyłane tak, jak wpisano.
- Każdy importer najpierw usuwa sekwencje sterujące i ucieczki, więc komentarz nie może poruszyć terminalem.

### Zapisana kopia, Wczytaj starsze, nieprzeczytane

- Każda otwarta rozmowa jest zapisywana (`<katalog stanu>/conversations/<integracja>/<projekt>/<rodzaj>-<n>.json`,
  tylko dla użytkownika, bez tokenu, najwyżej 2 MB na plik, zachowywanych jest 50 najnowszych plików). Maluje się od
  razu po otwarciu elementu i jest odświeżana w tle. Gdy host jest nieosiągalny, zapisana kopia zostaje na ekranie
  pod *Offline: pokazano kopię zapisaną 3 min temu (powód)*, a Wyślij jest wyszarzone.
- **Nieprzeczytane:** to, co widziałeś ostatnio, jest pamiętane dla każdej rozmowy (zachowywane jest 200 najnowszych).
  Wpisy innych osób, które są nowsze, dostają pasek akcentu i znacznik `nowe`, a `n` przeskakuje do następnego.
  Rozmowa otwierana po raz pierwszy nie ma nic nowego.

### Czego nie potrafi każdy host

Każde `nie` poniżej to prawdziwe ograniczenie: kontrolka jest ukryta lub wyszarzona z powodem, a CLI odpowiada
błędem z nazwą hosta, zanim cokolwiek zostanie wysłane.

| Host | Elementy | Odpowiedź w wątku | Edycja i usuwanie własnych | Rozwiązanie wątku | Zdarzenia | Notatki wewnętrzne | Zapisywane jako |
|---|---|---|---|---|---|---|---|
| GitHub | zgłoszenia, pull requesty | pull requesty | oba | pull requesty | tak | nie | GitHub Markdown |
| GitLab | zgłoszenia, merge requesty | tak (dyskusje) | oba | merge requesty | tak | tak | GitLab Markdown |
| Gitea | zgłoszenia, pull requesty | pull requesty | oba | pull requesty | tak | nie | Markdown |
| Forgejo | zgłoszenia, pull requesty | pull requesty | oba | nie (host nie ma endpointu) | tak | nie | Markdown |
| Azure DevOps | elementy pracy, pull requesty | pull requesty | oba | nie (status pokazany, niezmieniany) | tylko pull requesty | nie | Markdown (HTML w elementach pracy Server) |
| Bitbucket (Cloud, Data Center) | tylko pull requesty | tak | oba | nie (pokazane, niezmieniane) | tak | nie | Markdown |
| Gerrit | tylko zmiany | tak | nie (żadnego nie da się edytować ani usunąć) | tak | tak | nie | zwykły tekst |
| Jira | zgłoszenia | nie (płaska lista) | oba | nie | tak (changelog) | nie (wymaga roli w projekcie) | wiki markup |
| Linear | zgłoszenia | tak (odpowiedzi na komentarz) | oba | nie | tak | nie | Markdown |
| YouTrack | zgłoszenia | nie | oba | nie | nie | nie | Markdown |
| Redmine | zgłoszenia | nie | tylko edycja (Redmine 5 i nowszy), nigdy usuwanie | nie | tak (dzienniki) | tak (notatki prywatne) | Markdown lub Textile |
| Trello | karty | nie | oba | nie | tak (przeniesienia) | nie | Markdown |
| Notatki lokalne | notatki | nie | własne lokalnego autora | nie | nie | nie | Markdown |

Ponadto: żaden host nie pozwala zacząć z rozmowy nowego wątku przy linii kodu (rób to w przeglądzie), a reakcje są
pokazywane tylko w GitHub. Tekst dłuższy, niż przyjmuje host, jest odrzucany (GitHub 65 536 znaków, GitLab 1 000 000,
Azure DevOps 150 000, Jira 32 767, Trello 16 384): CLI sprawdza to najpierw, w aplikacji pokazywany jest własny
komunikat hosta, a twój tekst zostaje. Notatka lokalna trzyma swoje komentarze w końcowej sekcji `## Comments` samego
pliku notatki, po jednym wpisie `### autor - czas`, więc widzą je także Obsidian i powłoka.

W wierszu poleceń te same rozmowy to `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
i `comment-delete` oraz `hosting review conversation` (zob. [opis CLI](cli-reference.md)); dla agentów są to
narzędzia MCP o tych samych nazwach (zob. [MCP i agenci](mcp-and-agents.md)).
