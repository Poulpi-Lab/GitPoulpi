[English](../interface-tour.md) · [Čeština](../cs/interface-tour.md) · [Deutsch](../de/interface-tour.md) · [Español](../es/interface-tour.md) · [Français](../fr/interface-tour.md) · [Italiano](../it/interface-tour.md) · [日本語](../ja/interface-tour.md) · [한국어](../ko/interface-tour.md) · [Nederlands](../nl/interface-tour.md) · **Polski** · [Português (Brasil)](../pt-BR/interface-tour.md) · [Português (Portugal)](../pt-PT/interface-tour.md) · [Русский](../ru/interface-tour.md) · [Svenska](../sv/interface-tour.md) · [Türkçe](../tr/interface-tour.md) · [Українська](../uk/interface-tour.md) · [简体中文](../zh-CN/interface-tour.md)

# Przewodnik po interfejsie

Ta strona nazywa każdy obszar okna GitPoulpi, od góry do dołu. Pozostałe
strony używają tych samych nazw. Zdjęcia pokazują małe repozytorium demo
ze zmyślonymi autorami.

![Główne okno z jego obszarami: pasek kart, pasek narzędzi, panel boczny Refs, Log, panel zmian roboczych, formularz commita i pasek stanu](../images/hero.png)

## Rama okna

### Pasek tytułu

Górna krawędź okna wyświetla
<img src="../../../assets/logo/gitpoulpi.png" alt="Ikona GitPoulpi" height="16"> `GitPoulpi vX.Y.Z -- <repository path>` (nazwa sesji jest na pasku stanu, domyślnie na dole).

- **<img src="../../../assets/logo/gitpoulpi.png" alt="Ikona GitPoulpi" height="16">** otwiera okno **O programie**: wersja, funkcje kompilacji, licencja, linki i zewnętrzne crate'y. W terminalu, który rysuje obrazy, ośmiornica jest pikselowym logo w wybranym kolorze (gdzie indziej zostaje emoji). Paleta: `Pomoc: O programie GitPoulpi`; klawisze: `Ctrl+N`, potem `A`. W oknie O programie źrenice ośmiornicy podążają za wskaźnikiem myszy. Kliknij logo (albo naciśnij `Space`), aby obrócić je do następnego koloru. Kliknij logo prawym przyciskiem (albo naciśnij `c`), aby zobaczyć listę kolorów. GitPoulpi zapisuje wybór w `ui.logo_color`. Linki otwierają się kliknięciem lub własnym klawiszem dostępu (`s` kod źródłowy, `d` dokumentacja, `i` licencja), a `l` wypisuje zewnętrzne crate'y z wersjami i licencjami (przewijana lista; `Esc` lub `l` wraca, podobnie jak kliknięcie jej tytułu). Każdy crate na liście ma odnośnik do swojej strony (repozytorium, a w razie jego braku strona na crates.io): kliknij wiersz albo wybierz go klawiszami `↑` `↓` i naciśnij `Enter`, aby otworzyć go w przeglądarce. `t` (lub polecenie palety `Pomoc: Licencje i podziękowania`) pokazuje informacje o licencjach i podziękowania dla wszystkiego, co jest wbudowane w program, jako przewijany tekst (`Up` / `Down`, `PgUp` / `PgDn`, `Home` / `End`; powrót: `Esc` lub `t`). W skrypcie ten sam tekst wypisuje `gitpoulpi licenses`.
- **Wersja** (`v1.0.0`, a gdy jest nowsze wydanie, `vX.Y.Z dostępna`)
  stoi między nazwą a ścieżką: kliknięcie otwiera panel aktualizacji tylko wtedy, gdy
  istnieje nowsze wydanie albo trwa aktualizacja; w przeciwnym razie otwiera kartę
  **Informacje o wersji**, zob. [Aktualizacje](updates.md).
- **Otwórz w ▾**, tuż za ścieżką, otwiera folder karty w terminalu, edytorze,
  multiplekserze lub menedżerze plików (`Alt+O`). Zob. [Otwórz w](open-in.md).
- **⌕ Szukaj wszędzie** (lupa z etykietą), tuż na lewo od koła zębatego, otwiera [wyszukiwanie globalne](search.md) (`Ctrl+Alt+F`). Jako pierwszy ustępuje miejsca, gdy pasek tytułu jest za wąski: najpierw znika etykieta (lupa zostaje), potem cały przycisk.
- **Koło zębate** (`⚙`), tuż na lewo od **×**, otwiera **Ustawienia** (`Ctrl+,`).
- **×** na prawym końcu kończy program, tak jak `q`.

![Okno O programie nad oknem głównym: pikselowa ośmiornica, wersja, licencja, funkcje kompilacji, linki i liczba crate'ów](../images/about.png)

![Lista zewnętrznych crate'ów w oknie O programie: w każdym wierszu nazwa, wersja, link do strony crate'a i licencja](../images/about-crates.png)

### Pasek stanu

Pasek stanu to domyślnie ostatni wiersz okna (**Ustawienia > Wygląd > Położenie paska stanu** przenosi go na górę, nad karty repozytoriów). Pokazuje bieżącą gałąź, liczbę wczytanych commitów, `dirty` lub
`clean`, nazwę sesji oraz najprzydatniejsze klawisze: `?` pomoc, `Ctrl+G`
akcje git, `Ctrl+N` nawigacja, `Ctrl+P` polecenia, `Shift+F10` menu, `q`
koniec. W wąskim oknie najmniej przydatne klawisze są pomijane. Pasek stanu pokazuje
też komunikaty o stanie i postęp. Pokazywane klawisze są zawsze tymi
obowiązującymi, także zmienionymi przez ciebie (zob.
[Ustawienia](settings.md#klawiatura-i-mysz)).

Liczba commitów wygląda jak `500 / 48213 commita`, dopóki zostaje starsza historia do wczytania (`500 / … commity`, gdy suma jest dopiero liczona), a po wczytaniu całości tylko `2500 commita`. W częściowym checkoutcie (sparse) po nazwie sesji pojawia się znaczek `częściowy`. Sposób liczenia sumy to ustawienie: **Metoda liczenia: commity** (zob. [Ustawienia](settings.md#log-i-graf)).

Na jej prawym końcu przycisk **Aktywność** otwiera Dziennik aktywności: kartę Operacje git (operacje z czasem ich trwania, komunikaty i błędy), kartę Aplikacja (dziennik aplikacji z polem poziomu) oraz pole Zakres, które zawęża obie do tego repozytorium, obszaru roboczego albo wszystkich repozytoriów. Liczba na przycisku zlicza błędy, których jeszcze nie obejrzano. Gdy operacja się nie powiedzie, linia stanu na czerwono mówi dlaczego, niezależnie od tego, gdzie jesteś; kliknięcie jej (albo polecenie palety `Dziennik aktywności: Otwórz ostatni błąd`) otwiera wpis w Aktywności ze szczegółami, przyciskiem Kopiuj dla AI i proponowanym następnym krokiem. Oba opisano w [Gdy coś się nie powiedzie](troubleshooting.md#gdy-coś-się-nie-powiedzie).

Na karcie **Aplikacja** przycisk **Wyczyść dziennik** (`x`) po potwierdzeniu opróżnia pliki dziennika aplikacji. Na lewo od przycisku **Aktywność** wskaźnik zadań pokazuje działające zadanie w tle z `[x]` do jego anulowania (lista zadań ma przycisk **Anuluj wszystko**, a paleta polecenie *Zadania: anuluj wszystko, co można anulować*; zadanie, którego nie da się bezpiecznie zatrzymać, podaje powód). Karta Start ma ten sam pasek stanu z przyciskiem **Aktywność** w prawym dolnym rogu (otwiera się na Wszystkich repozytoriach).

### Nakładki

Paleta poleceń, pomoc, menu kontekstowe, okna dialogowe, okna podręczne
i Ustawienia otwierają się nad oknem. Każda z nich ma `✕` w prawym górnym rogu
ramki; zamyka ją `Esc` albo kliknięcie `✕`, a kliknięcie poza oknem zamyka okno
dialogowe, okno podręczne lub panel (zob. [Zamykanie okien i dialogów](mouse-and-gestures.md#zamykanie-okien-i-dialogów)).

### Gdy okno jest małe

Nic nie znika po cichu, gdy okno się zwęża. Przycisk najpierw traci długą etykietę
(przyciętą z `...`), potem zawija się do drugiego wiersza albo trafia do menu `⋯`
swojego wiersza, a jego klawisz i wpis palety działają cały czas. Okno dialogowe zbyt duże
dla okna rysuje małą ramkę z tytułem, `✕` i wierszem **Okno za małe: powiększ je**
zamiast niczego; `Esc` i `✕` je zamykają, a po powiększeniu okna wraca w całości.

## Rząd kart

Karta **Home** (ikona domku) jest na lewym końcu, a za nią
[plakietka obszaru roboczego](workspaces.md) (gdy masz już obszar roboczy), przypięta karta
**Radar** (zamyka się jak każda karta; zob. [Radar](radar.md)) i karta
**Obszary robocze**, gdy ją otworzysz (`Alt+J`; zamyka się jak każda karta). Dalej idzie
jedna karta na każde otwarte repozytorium. Krzyżyk `x` zamykający kartę widać tylko na
aktywnej karcie i na karcie pod wskaźnikiem. Rząd kart zawiera tylko karty (przycisk
[wyszukiwania globalnego](search.md) jest na pasku tytułu).

Gdy karty repozytoriów się nie mieszczą, rząd się przewija zamiast je zmniejszać:
karty leżą między dwoma przyciskami ze strzałkami, `‹` z lewej i `›` z prawej.
Strzałka jest przygaszona, gdy po jej stronie nie ma ukrytej karty. Kliknij
strzałkę, by pokazać jeszcze jedną kartę z tej strony (przytrzymaj ją, by
przewijać dalej), albo użyj kółka nad rzędem (w górę lub w lewo pokazuje
wcześniejsze karty). Gdy tylko zmieni się bieżąca karta (przełączasz karty z
klawiatury, przez selektor kart lub paletę, otwierasz albo zamykasz kartę, albo
zmieniasz rozmiar okna), rząd przewija się akurat tyle, by pokazać ją w całości.
Gdy jest miejsce, przycisk `…` za prawą strzałką otwiera selektor kart.
Przeciągnięcie karty na strzałkę przewija rząd, dopóki ją tam trzymasz, więc
możesz upuścić kartę za tymi widocznymi na ekranie. `Ctrl+N` pokazuje plakietkę
przy każdej aktywnej strzałce.

**Ustawienia > Wygląd > Położenie paska kart** przenosi ten wiersz na dół okna (pod pasek stanu) albo zamienia go w kolumnę przy lewej krawędzi: jedna karta w wierszu, najpierw Start, potem znacznik obszaru roboczego, Radar, karta Obszary robocze, karty (długa nazwa kończy się `…`) i `+`. Gdy kart jest więcej niż wierszy, wiersz `…` pokazuje, ile ukrytych jest powyżej (`▲`) i poniżej (`▼`), i otwiera listę kart; kółko przewija kolumnę. Obok paska narzędzi w kolumnie karty zostają na zewnątrz. Okno węższe niż 80 kolumn trzyma karty u góry. Kliknięcia, krzyżyk zamykania, środkowy przycisk, menu prawego przycisku, przeciąganie do zmiany kolejności i klawiatura działają tak samo w każdym położeniu.

- **+** tuż za ostatnią kartą otwiera małe menu: **Nowa karta** (`Alt+T`,
  paleta **Karta: Nowa karta**) oraz, gdy ich karty są zamknięte, **Radar**
  i **Obszary robocze** (`Alt+J`), które otwierają te karty; polecenie palety **Karta: Nowa...**
  otwiera to samo menu. **Nowa karta** to widok Home z gotowym polem wyszukiwania, więc pisanie
  od razu filtruje twoje repozytoria. Wybierz jedno (albo Przeglądaj, Klonuj,
  Init), a jego karta pojawi się na końcu paska; repozytorium, które jest już
  otwarte, jest po prostu przełączane. `Esc` w pustym polu wyszukiwania,
  `Ctrl+W`, jej `x` albo kliknięcie środkowym przyciskiem zamyka Nową kartę
  i wraca do karty, na której byłeś. Nigdy nie jest zapisywana w sesji.

  ![Nowa karta: widok Home z gotowym polem wyszukiwania, nad otwartymi kartami](../images/new-tab.png)
- Kliknij kartę prawym przyciskiem albo naciśnij `Ctrl+T`, aby zobaczyć menu
  karty: **Nowa karta**, **Zamknij kartę**, **Zamknij inne karty...**, **Zamknij karty po
  prawej...**, **Dodaj do ulubionych** (albo **Usuń z ulubionych**), **Dodaj do
  obszaru roboczego** (albo **Usuń z obszaru roboczego**, gdy masz obszary robocze),
  **Kopiuj ścieżkę**, **Otwórz w ›**, **SSH i poświadczenia...** i **Częściowy checkout...**.
- `Ctrl+PageUp` i `Ctrl+PageDown` przełączają karty, podobnie `Ctrl+Shift+Tab` i
  `Ctrl+Tab`, jeśli terminal je przekazuje (protokół klawiatury kitty; niektóre
  terminale i multiplexery zatrzymują je dla siebie). `Ctrl+W` zamyka bieżącą
  kartę. Kliknięcie karty środkowym przyciskiem też ją zamyka (z tym samym
  pytaniem).
- `t` (albo `Ctrl+N Tab`) otwiera selektor kart, przeszukiwalną listę otwartych
  kart.
- **Zmiana kolejności kart:** naciśnij kartę repozytorium i przeciągnij ją w bok (cień
  karty podąża za wskaźnikiem, a pasek pokazuje, gdzie wyląduje; `Esc` anuluje),
  albo przeciągnij wiersz selektora kart, naciśnij `Alt+Up` / `Alt+Down` na
  podświetlonym wierszu selektora, albo uruchom z palety **Karta: Przesuń w lewo** / **Karta: Przesuń
  w prawo**. Kolejność jest zapisywana razem z sesją. Zwykłe kliknięcie
  bez ruchu nadal zaznacza kartę. Domek, plakietka obszaru roboczego i
  `+` nigdy się nie przesuwają.
- **Zakończ i cd tutaj** pojawia się po prawej stronie kart, gdy uruchomiono
  GitPoulpi przez wrapper powłoki `gp`, a folder karty różni się od folderu,
  w którym go uruchomiono. Zob. [Podążanie za folderem karty](following-the-tab-folder.md).
- **Plakietka obszaru roboczego** tuż za domkiem (kolorowa kropka i nazwa albo
  „Brak obszaru”) przełącza [obszar roboczy](workspaces.md): kliknij ją albo
  naciśnij `Alt+W`. Pojawia się, gdy masz już jakiś obszar roboczy.

![Selektor kart z listą otwartych repozytoriów](../images/tab-picker.png)

Gdy **Pamiętaj karty** jest włączone (domyślnie), karty wracają przy
następnym uruchomieniu. Worktree otwiera się we własnej karcie. Ta karta
zamyka się, gdy worktree zostanie usunięte.

Karta worktree trafia zaraz za kartę swojego repozytorium, a odciągnięta karta worktree wraca na to miejsce (**Grupuj karty worktree** w Ustawieniach > Ogólne, domyślnie włączone).

### Home

Karta Home to menedżer repozytoriów. Ma cztery sekcje: **Otwarte
repozytoria**, **Ulubione**, **Ostatnie repozytoria** i **Wszystkie
repozytoria**. Każdy wiersz pokazuje gałąź, liczby ahead i behind oraz liczbę
zmian, a potem ścieżkę folderu, gdy jest miejsce: skróconą przez `~` i ucinaną
pośrodku, gdy jest ciasno, nigdy nie wypycha nazwy, właściciela ani gałęzi.
Każdy wiersz ma gwiazdkę (ulubione), **Otwórz** lub **Przełącz**,
**Zamknij** lub **Usuń** oraz menu `⋮`.

Na górze: **Przeglądaj**, **Klonuj**, **Init**, pole wyszukiwania, **Zwiń
wszystko** i **Rozwiń wszystko**. Folder, który już nie istnieje, sam znika
z sekcji *Ostatnie* i *Wszystkie*. Brakujące ulubione zostaje, z przyciskiem
**Usuń**. **Usuń** niczego nie kasuje na dysku.

Klawisze: `Enter` otwiera, `f` ulubione, `x` zamyka lub usuwa, `y` kopiuje ścieżkę, `o` otwiera w domyślnym narzędziu (pozycja **Otwórz w** menu `⋮` wybiera inne), `.` menu, `/`
wyszukiwanie, `Space` zwija sekcję, `Esc` wraca do karty.
`Ctrl+N` pokazuje plakietkę na polu wyszukiwania, każdej sekcji i każdym przycisku: naciśnij znak plakietki, aby ustawić na nim fokus lub go nacisnąć (zob. [Plakietki](keybindings.md#plakietki)).

Jedno pole wyszukiwania filtruje wszystko pod nim, także Ulubione: repozytorium zostaje, jeśli jego nazwa, gałąź lub ścieżka zawiera tekst. Obszary robocze mają własną kartę i nie są stroną karty Home: otwórz ją z menu przy `+`, klawiszem `Alt+J` albo z palety, a zamknij jej `x`: zob. [Obszary robocze](workspaces.md).

**Klonuj** wypisuje dla każdego połączonego profilu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) repozytoria, do których masz dostęp, po jednej sekcji na właściciela: najpierw własne, potem po jednej na organizację lub grupę (alfabetycznie), każda pod nagłówkiem. Kłódka oznacza repozytorium prywatne. Pole wyszukiwania filtruje wszystkie sekcje; sekcja bez dopasowań znika. `Up` i `Down` pomijają nagłówki, a `Home`, `End`, `PgUp` i `PgDn` działają na liście.

![Karta Home: otwarte, ulubione i ostatnie repozytoria oraz przyciski Przeglądaj, Klonuj i Init](../images/home.png)

## Pasek narzędzi

Domyślnie rząd pod panelami i nad paskiem stanu. **Ustawienia > Wygląd > Położenie paska narzędzi** stawia go u góry (pod kartami), na dole albo jako kolumnę po lewej lub prawej stronie paneli (okno węższe niż 80 kolumn rysuje kolumnę jako rząd na dole). Przyciski leżą na początku rzędu lub kolumny; **Ustawienia > Wygląd > Wyrównanie przycisków paska narzędzi** wyśrodkowuje je albo przesuwa na koniec. Każdy przycisk ma własny kolor ikony, który ustawia motyw (zob. [Motywy](themes.md)). Gdy nie ma miejsca na wszystkie, najmniej ważne przyciski przenoszą się za `⋯`. W kolumnie każdy przycisk zajmuje wiersz, jego `▾` jest na końcu wiersza, a menu otwiera się obok kolumny; menu dolnego paska narzędzi otwiera się w górę.

| Przycisk | Co robi |
|---|---|
| **Cofnij**, **Ponów** | Cofają lub przywracają ostatnią operację (`Ctrl+Z`, `Ctrl+Y`). Ukryte, gdy Cofnij jest wyłączone. Zob. [Cofnij i ponów](undo-redo.md). |
| **Pull** `▾` | Wykonuje pull zgodnie z ustawieniem **Ustawienia > Git i kodowanie > Przycisk Pull**. Lista `▾` ma **Fetch wszystkich**, **Pull (fast-forward, jeśli się da)**, **Pull (tylko fast-forward)** i **Pull (rebase)**, potem **Pull wszystkie gałęzie** (`Ctrl+G Ctrl+P`) oraz to samo z każdym innym trybem dla bieżącej gałęzi. Pozycja listy działa jeden raz i nie zmienia przycisku. |
| **Push** `▾` | Wypycha bieżącą gałąź. Lista `▾` wypycha jednorazowo do wybranego zdalnego repozytorium. |
| **Gałąź** | Tworzy gałąź w bieżącym commicie. |
| **Tag** | Tworzy tag w bieżącym commicie. |
| **Stash**, **Pop** | Robi stash wszystkich zmian. Robi pop najnowszego stasha. |
| **LFS** `▾` | Akcje Git LFS: fetch, checkout, pull, push, prune (`Ctrl+G l`). Ukryte, gdy Git LFS jest wyłączony. Zob. [Git LFS](lfs.md). |
| **Flow** `▾` | Menu przepływu pracy z gałęziami (start, finish, sync, promote; `Ctrl+G Shift+W`). Tylko w repozytorium, które ma przepływ pracy. Zob. [Przepływy pracy z gałęziami](branching-workflows.md). |
| **Szukaj** | Otwiera pasek wyszukiwania commitów nad Logiem (`f`). |

Na końcu (z prawej w rzędzie, na dole w kolumnie), gdy repozytorium jest powiązane z hostem, **chip** pokazuje pull request bieżącej gałęzi (na przykład `GH #12` lub `GL !53`) albo przygaszony `+ PR`. **Ustawienia > Wygląd > Przypnij przycisk pull request na końcu** (domyślnie włączone) trzyma go tam; wyłączone, płynie z innymi przyciskami i podąża za ich wyrównaniem. W wąskim rzędzie chip kurczy się jako pierwszy (`● #12`, potem sam symbol stanu) i dopiero potem trafia do menu `⋯`. Zob. [Hosting i trackery zgłoszeń](hosting-integration.md). W kolumnie lista rozwijana profili zajmuje ostatni wiersz.

Na prawym końcu, gdy masz dwa lub więcej [profili](settings.md#profile),
**lista rozwijana profili** wybiera tożsamość commitów dla tego repozytorium.
Pierwszy wiersz, *Domyślny*, podąża za **Ustawienia > Profile**. Po wybraniu
profilu GitPoulpi pyta, czy zapisać też jego nazwę i e-mail w konfiguracji git
repozytorium (`user.name`, `user.email`). **Zapisz w konfiguracji git** je
zapisuje. `Esc` zachowuje wybór tylko w GitPoulpi. Paleta:
`Repozytorium: Zmień profil...`.

Każdy przycisk ma polecenie w palecie i skrót.

![Lista Pull otwarta pod paskiem narzędzi: Fetch wszystkich i trzy sposoby wykonania pull](../images/pull-push-tooltip.png)

## Banery

Pasek nad Logiem informuje, gdy repozytorium jest w szczególnym stanie. Ma
przyciski wskazujące drogę naprzód. Każdy przycisk jest też poleceniem
w palecie.

| Baner | Przyciski |
|---|---|
| Merge, cherry-pick lub revert zatrzymał się na konfliktach | **Rozwiąż konflikty...**, **Commit** lub **Kontynuuj**, **Przerwij** |
| Trwa rebase | **Rozwiąż konflikty...**, **Kontynuuj**, **Pomiń**, **Przerwij** |
| **Stosowanie patchy (git am)** | **Rozwiąż konflikty**, **Kontynuuj**, **Pomiń**, **Przerwij** |
| Pozostały `index.lock` blokuje każdą zmianę | **Usuń index.lock** (najpierw pyta) |
| Auto-stash nie został przywrócony | **Pop teraz**, **Pokaż stash**, **Zacznij od nowa**, **Drop** (wybór zależy od stanu) |

Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md) i
[Rozwiązywanie problemów](troubleshooting.md).

## Panel boczny Refs (po lewej)

Drzewo wszystkiego, co nazywa commit. Każda sekcja ma liczbę w tytule,
chevron do zwijania i własną pozycję przewijania. Przeciągnij granicę między
dwiema sekcjami, aby zmienić ich rozmiar.

- Pole **Filtr** na górze (`/`): filtruje każdą sekcję po nazwie.
- **LOKALNE**: gałęzie lokalne, w folderach dzielonych po `/` (`feature/`,
  `fix/`). Znacznik wyboru oznacza bieżącą gałąź. Foldery można
  wyłączyć w Ustawieniach.
- **ZDALNE**: każde zdalne repozytorium z jego gałęziami pod spodem.
- **WORKTREES**: powiązane worktree, jeśli są (albo gdy pokazałeś sekcję).
  Przycisk **+** w jej nagłówku tworzy nowe; kliknij worktree dwukrotnie, aby
  otworzyć je we własnej karcie, kliknij je prawym przyciskiem, aby zobaczyć
  Otwórz w, Merge i Usuń. Zob. [Worktree](worktrees.md).
- **STASHES**: stashe. Kliknij jeden prawym przyciskiem, aby zobaczyć
  **Pop**, **Zastosuj** i **Drop**.
- **TAGS**: tagi, w folderach wydzielanych po `/` tak jak gałęzie (`releases/`,
  `backups/`). Strzałka oznacza, że tag jest też na zdalnym repozytorium.
- **PULL REQUESTS** i **ZGŁOSZENIA**: po połączeniu konta hostingu i powiązaniu
  repozytorium.
- **SUBMODUŁY**: submoduły i ich stan, jeśli są. Zob.
  [Submoduły](submodules.md).

**Pokazywanie i ukrywanie sekcji:** kliknij prawym przyciskiem nagłówek sekcji lub puste miejsce panelu, aby zobaczyć listę kontrolną sekcji (ostatnia widoczna zostaje). Wybór jest zapamiętywany dla każdego repozytorium, między uruchomieniami; zob. [Sekcje lewego panelu](mouse-and-gestures.md#sekcje-panelu-bocznego).

Najedź na wiersz, aby pokazać jego przyciski: ukrycie lub pokazanie refa
w Logu oraz **Solo** (pokaż tylko commity tego refa). Kliknij dwukrotnie
gałąź lokalną lub tag, aby zrobić checkout. Kliknij wiersz prawym
przyciskiem, aby zobaczyć jego menu. `‹` w tytule panelu zwija panel boczny
(`Ctrl+N Shift+H`).

Oba panele boczne (panel boczny Refs, `Ctrl+N Shift+H`, oraz prawy panel,
`Ctrl+N Shift+L`) należą do Logu: pokazanie lub zwinięcie któregoś z nich, gdy
otwarty jest diff, widok pliku albo edytor, dotyczy tylko tego widoku. Diff lub
widok pliku sam też zwija panel boczny Refs, aby zrobić miejsce na treść. Po
powrocie do Logu oba panele są takie, jak je tam zostawiono, i ten stan
zapisuje się na następne uruchomienie.

## Log (pośrodku)

![Log: wiersz nagłówka z kolumnami, wiersz zmian roboczych na górze, wiersz stasha, plakietki refów po lewej od grafu](../images/log-graph.png)

Jeden wiersz na commit, od najnowszego. Kliknij nagłówek prawym przyciskiem,
aby pokazać lub ukryć kolumny. Przeciągnij granicę między dwoma nagłówkami,
aby zmienić szerokość kolumny. Przeciągnij nagłówek, aby przenieść kolumnę.
Szerokości należą do repozytorium: są z nim zapisywane i wracają po ponownym otwarciu,
a repozytorium otwarte po raz pierwszy ma każdą kolumnę dopasowaną do tego, co pokazuje Log
(`Dopasuj wszystkie kolumny do zawartości` robi to ponownie, `Zresetuj szerokości kolumn` wraca
do szerokości automatycznych, które już zostają).

Panel nie ma tytułu: mówią o tym nagłówki kolumn. Przy commicie bez refa najechany wiersz pokazuje wyblakłą pigułkę z nazwą gałęzi, do której należy; jeśli ta gałąź została usunięta, przekreślona pigułka pokazuje nazwę odczytaną z merge commita, który ją scalił. Tagi są rysowane tam, gdzie są, nigdy jako widmo (wyłącza to **Pokaż widmową gałąź dla najechanego commita** w Ustawieniach > Log i graf).

| Kolumna | Pokazuje |
|---|---|
| **Ref** | Kolorowe plakietki gałęzi i tagów na commicie: ikona laptopa dla lokalnych, kula ziemska dla zdalnych, ikona tagu dla tagów, `+N`, gdy wskazuje tam więcej refów. |
| **Graf** | Ścieżki gałęzi, merge i miejsca rozgałęzień. Rysowany jako obrazy w kitty, Ghostty, WezTerm, Rio, iTerm2 (oraz Windows Terminal, foot i Tabby przez sixel), a w innych terminalach jako znaki. Kolory ścieżek pochodzą z motywu lub z wybranej palety (zob. [Motywy](themes.md)). Przy najmniejszej szerokości (jedna kropka commita) kolumna jest zwarta: każdy commit na jednym torze, bez linii gałęzi i merge. |
| **Opis** | Podsumowanie commita. |
| **Autor** | Autor, z opcjonalnym awatarem. |
| **Data** | Data autora, w wybranym formacie. |
| **Zmiany** | Opcjonalnie: zmienione pliki oraz linie `+dodane -usunięte`. |
| **SHA** | Krótki hash. |

![Menu kolumn nagłówka Logu z polami wyboru Ref, Graf, Opis, Autor, Data, Zmiany i SHA](../images/log-columns-menu.png)

Wiersze specjalne:

- **Wiersz zmian roboczych** (wiersz WIP) na górze: przerywane kółko,
  `// WIP` lub wersja robocza opisu oraz liczby zmodyfikowanych i dodanych
  plików. Kliknij go, aby zrobić stage i commit w prawym panelu.
- **Wiersze stashy**: mały kwadrat na ścieżce, obok commita, na którym zrobiono
  stash.
- Podczas wyszukiwania wiersze, które nie pasują, są przyciemnione.

Najedź na komórkę, aby zobaczyć podpowiedź z pełnym tekstem. Kliknij wiersz
prawym przyciskiem, aby zobaczyć [menu commita](log-and-history.md#menu-commita).

![Menu prawego przycisku commita: cherry-pick, revert, reset, edycja opisu, wyjaśnienie, rebase, gałąź, tag, checkout, worktree, patch i akcje kopiowania](../images/context-menu.png)

## Prawy panel

Prawy panel podąża za zaznaczeniem w Logu.

**Zmiany robocze** (zaznaczony jest wiersz WIP):

- Tytuł, taki jak `4 file changes on main`. Podczas merge z konfliktami dodaje
  liczbę plików z konfliktami. Czerwone `✗` po lewej odrzuca wszystkie
  zmiany (najpierw pyta). Gdy AI jest włączone, `✦` po prawej wyjaśnia
  zmiany.
- **Ścieżka** lub **Drzewo** (płaska lista albo foldery, `m`), **Zwiń
  wszystko** i **Wszystkie pliki** (`a`).
- **Pliki unstaged** i **Pliki staged**, każde z liczbą i przyciskiem **Stage
  wszystkie zmiany** lub **Unstage wszystkie zmiany**. Najedź na wiersz, aby
  zobaczyć jego przycisk **Stage plik** lub **Unstage plik**.
- **Formularz commita** na dole: **Popraw poprzedni commit**, **Skomponuj
  commity przez AI** (gdy AI jest włączone), **Podsumowanie** z licznikiem
  znaków i przyciskiem `✦`, **Opis** oraz przycisk **Commit zmian w N plikach**.

**Szczegóły commita** (zaznaczony jest commit), od góry do dołu: pełny hash,
autor z awatarem i datą, hash rodzica i liczby zmian, potem opis (podsumowanie
i treść; przeciągnij jego dolną krawędź, aby zmienić rozmiar). Panel
**Zmienione pliki** poniżej zaczyna się od własnych kontrolek: przełącznika
sortowania, **Ścieżka** / **Drzewo**, **Wszystkie pliki** i pola **Filtruj pliki** (z polem wyboru **Tylko LFS** na jego
prawym końcu w repozytorium z plikami LFS), przytwierdzonych nad listą, która
jako jedyna się przewija. Kliknij plik, aby
otworzyć jego diff. W niskim oknie najpierw ustępuje opis (do jednego wiersza),
potem lista. Widok zmian roboczych zachowuje rząd **Ścieżka** / **Drzewo** /
**Wszystkie pliki** na górze list plików, nad listami unstaged i staged. W trybie
Drzewo zwinięty folder kończy się liczbą plików w środku, razem z podfolderami,
jako przygaszone `(12)`.

**Linki w opisie.** Adres internetowy wpisany w opisie (`https://...` albo cel
markdownowego `[tekst](https://...)`) jest podkreślony, pokazuje wskaźnik w
kształcie dłoni, a po kliknięciu pyta **Otworzyć ten link?** z całym adresem,
zanim cokolwiek się otworzy: tekst opisu nie jest zaufany. **Otwórz** (`O`)
uruchamia przeglądarkę, **Anuluj** (`A`, `Esc`, krzyżyk) nic nie robi.
Proponowane są tylko adresy `http` i `https`. Odpowiednik z klawiatury to
polecenie palety `Szczegóły commita: Otwórz link w opisie...`, które pyta o
pierwszy adres z opisu, a pozostałe przechodzi przyciskiem **Dalej** (`D`).
Niektóre terminale same podkreślają adres pod wskaźnikiem i otwierają go po
kliknięciu z modyfikatorem (często Shift+klik); to funkcja terminala, nie
GitPoulpi.

Każda lista plików ma menu dla plików, folderów i pustego miejsca, a wiele plików można zaznaczyć naraz (**Ctrl+klik**, **Shift+klik**); zob. [Stage i commity](staging-and-committing.md).

## Środkowy panel przy otwartym pliku

Kliknij zmieniony plik, a jego widok zastąpi Log. `Esc` przywraca Log. Wzdłuż
góry biegnie pasek przycisków:

![Zmodyfikowany plik otwarty jako diff, z paskiem widoku nad nim: Widok pliku, Blame, przełącznik Plik │ Wszystkie, Edytuj, nawigacja po hunkach, W linii, Obok siebie, Hunki, białe znaki, Zawiń i Mapa](../images/diff.png)

| Przycisk | Co robi |
|---|---|
| **Widok pliku**, **Diff**, **Blame** | Wybierają, co panel pokazuje dla pliku. |
| **Plik │ Wszystkie** | Wybiera listę [kolumny commitów](#kolumna-commitów) obok widoku i pokazuje kolumnę. |
| **Edytuj** | Otwiera wbudowany edytor na kopii roboczej. |
| `↑` `↓` | Poprzedni lub następny hunk. |
| **W linii**, **Obok siebie** | Jedna kolumna albo obok siebie. |
| **Hunki** | Tylko zmienione hunki albo cały plik. |
| **WS** | Pokaż lub ukryj zmiany samych białych znaków. |
| **Zawij** | Zawija długie linie. |
| **Mapa** | Minimapa całego pliku obok paska przewijania. |
| **Kodowanie** | Kodowanie, w którym plik jest czytany (najpierw **Auto**). |

#### Zmienione słowa w diffie

Gdy linia zmienia się tylko częściowo, słowa, które się różnią, są malowane
mocniejszym tłem w kolorze samej linii (mocniejszy czerwony na usuniętej linii,
mocniejszy zielony na dodanej), a reszta linii zachowuje płaski odcień. W obrębie
hunka każdy ciąg usuniętych linii jest łączony w pary z następującym po nim
ciągiem dodanych: pierwsza z pierwszą, druga z drugą i tak dalej; para jest
oznaczana tylko wtedy, gdy co najmniej 40% jej znaków niebiałych jest takich
samych, więc linia napisana od zera zostaje płaska, zamiast zamienić się w jeden
blok wyróżnienia. Porównywane są słowa, ciągi białych znaków i pojedyncze znaki
interpunkcyjne, pojedynczy odstęp między dwoma zmienionymi słowami łączy je w jedno
oznaczenie, a linie dłuższe niż 1000 znaków nie są oznaczane. Działa w widokach
W linii i Obok siebie, z zawijaniem i przewijaniem w bok oraz w panelu diffa
recenzji. Wyłączysz to w **Ustawienia > Diff i edytor > Podświetlaj zmienione
słowa** (`ui.diff_intraline`) albo z palety poleceń (**Diff: Przełącz podświetlanie
zmienionych słów**, bez własnego klawisza). W terminalu ograniczonym do 16 kolorów
albo z motywem bez tła diffa zmienione słowa są zamiast tego pogrubione i
podkreślone.

W diffie drzewa roboczego każdy hunk ma przyciski **Stage hunk** i **Odrzuć
hunk**. Naciśnij `v`, aby zaznaczyć linie, a potem zrób stage lub odrzuć
tylko te linie.

![Blame pliku: kod z kolorami składni i po prawej kolumna adnotacji dla każdego hunka z awatarem, autorem, wiekiem i podsumowaniem oraz paski autora i wieku](../images/blame.png)

![Historia pliku: commity pliku po lewej, w dwóch wierszach, i diff wybranego commita po prawej](../images/file-history.png)

Oba widoki opisano w [Blame](log-and-history.md#blame) i [Historia pliku](log-and-history.md#historia-pliku).

Wbudowany **edytor** ma kolorowanie składni, numery linii, cofanie, szukanie
oraz pasek stanu z linią, kolumną, końcem linii, wcięciem i językiem.

![Wbudowany edytor pokazujący plik Rust z kolorowaniem składni i numerami linii](../images/editor.png)

Plik graficzny jest pokazywany jako obraz: przed i po w diffie albo sam plik
w widoku pliku. Zob. [Obrazy](image-view.md).

Plik binarny, który nie jest obrazem (PDF, dokument, archiwum), pokazuje przycisk **Otwórz w**, gdy system zna aplikacje dla jego typu: zob. [Otwórz w](open-in.md).

## Kolumna commitów

Po lewej stronie pliku commita otwartego jako diff, w widoku pliku lub w Blame może stać jedna kolumna (nigdy obok wbudowanego edytora, który edytuje kopię roboczą: kolumna znika, gdy edytor się otwiera, i wraca, gdy zamyka się on na diffie). Ma trzy stany:

- **Plik**: commity tego jednego pliku, po dwa wiersze każdy ([Historia pliku](log-and-history.md#historia-pliku)). Prawa strona pokazuje wersję pliku z zaznaczonego commita.
- **Wszystkie**: zwarta lista commitów z Logu wraz z grafem, opisana niżej.
- **Ukryta**: wąski pasek `›` po lewej stronie widoku przywraca kolumnę (jego podpowiedź podaje klawisz).

Pasek przycisków widoku zawsze biegnie przez całą szerokość nad kolumną, więc żaden przycisk się nie przesuwa, gdy kolumna się otwiera, zamyka albo zmienia listę. Na nim przełącznik **`Plik │ Wszystkie`** (dwa segmenty jednej kontrolki, zawsze tej samej szerokości) wybiera listę: kliknięcie segmentu pokazuje tę listę i otwiera kolumnę, jeśli była ukryta; kliknięcie segmentu, który jest już wybrany, ukrywa widoczną kolumnę. Segment listy widocznej na ekranie jest podświetlony; gdy kolumna jest ukryta, podkreślony jest ten, z którym wróci, a lista jest zachowana. Jego podpowiedź podaje kliknięcie, ponowne kliknięcie wybranego segmentu i `Alt+C`. Historia pliku otwiera się też z miejsc, które zawsze ją otwierały (menu plików, `Historia pliku: ...` w palecie).

![Kolumna commitów w stanie Wszystkie obok diffa pliku commita, z nagłówkiem Ref, Graf, Opis, Autor i Data oraz etykietami gałęzi i tagów w części Ref](../images/commit-column-all.png)

- **Pokazywanie i ukrywanie**: `Alt+C` (paleta `Widok: Przełącz kolumnę commitów`), kliknięcie wybranego segmentu przełącznika, ✕ na kolumnie albo pasek `›`. W palecie `Widok: Kolumna commitów: Ukryj`, `... : Historia pliku` i `... : Wszystkie commity` wybierają stan. Wybór, razem z listą, jaką miała kolumna, jest zapamiętywany z innymi ustawieniami kolumny. Bez wyboru o liście Wszystkie decyduje szerokość terminala: pokazuje się od 140 kolumn. Kolumna nie jest rysowana, gdy widok obok miałby mniej niż 30 kolumn, a graf listy Wszystkie także dla diffa drzewa roboczego ani porównania (historia pliku nadal się pokazuje, bo nie potrzebuje commita).
- **Szerokość**: przeciągnij jej prawą krawędź (podświetlona krawędź podąża za wskaźnikiem) albo użyj `Widok: Poszerz kolumnę commitów` i `Widok: Zawęź kolumnę commitów`. Może zająć do 70% kolumny środkowej (reszta zostaje widokowi, który nigdy nie dostaje mniej niż 30 kolumn). Jest zapamiętywana razem z innymi rozmiarami.
- **Wybór commita**: kliknij go albo daj kolumnie klawiaturę (`Ctrl+N` i jej odznaka, lub `Widok: Przenieś fokus na kolumnę commitów`) i użyj `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End`. `Enter` lub `Right` przechodzi do listy plików commita, `Esc` oddaje klawiaturę widokowi. Prawy panel podąża za wyborem: szczegóły commita i jego zmienione pliki, żeby można było otworzyć inny jego plik.
- **Co robi widok**: otwarty plik pozostaje otwarty. Jeśli wybrany commit go zmienił, widok pokazuje jego diff w tym commicie. Jeśli nie, pokazuje plik taki, jaki był wtedy, z napisem "bez zmian w tym commicie" w tytule. Jeśli commit nie ma takiego pliku, widok to mówi i pozostaje używalny. Blame pozostaje Blame, według stanu wybranego commita.

Lista Wszystkie stoi pod nagłówkiem (nazwy kolumn samego Logu). Jeden commit w wierszu: tory grafu, tytuł (opis), imię autora i krótka data oraz, jeśli to włączysz, etykiety gałęzi i tagów. To lista samego Logu, z tymi samymi commitami w tej samej kolejności, a starsze dociąga podczas przewijania. Tory są rysowane tak jak w kolumnie Graf Logu: w terminalu pokazującym obrazy (zob. [Wybór terminala](terminals.md)) jako połączone tory z okrągłymi znacznikami i, gdy **Awatary autorów** mają wartość **graf**, z obrazkami autorów w kółkach; w innych terminalach jako znaki. Domyślnie tory zajmują tyle miejsca, ile potrzebują, a tytuł zawsze zachowuje kilka słów: w wąskiej kolumnie najpierw ustępuje autor, potem data, zanim tory zostaną przycięte. Zmniejszona do najmniejszej szerokości, jednej kropki commita, część Graf jest zwarta, jak w Logu: każdy commit na jednym torze, bez linii gałęzi i merge. Część ucięta przez panel (wiersz przewinięty na boki albo część Graf szersza niż miejsce) zachowuje swoje obrazy i awatary, przycięte na krawędzi. Kółko przewija listę pod wskaźnikiem.

- **Najechanie i menu**: zatrzymanie wskaźnika na wierszu pokazuje podpowiedź Logu dla części pod wskaźnikiem (cały opis na tytule, nazwa i e-mail autora, pełna data). Kliknięcie wiersza prawym przyciskiem (albo klawisz menu, `Shift+F10`, gdy kolumna ma klawiaturę; `Widok: Kolumna commitów: Menu zaznaczonego wiersza` w palecie) otwiera jego menu: *Pokaż w Log*, diff pliku w tym commicie, plik taki, jaki był w tym commicie, blame pliku w tym commicie, a potem własne menu Logu dla tego commita (kopiowanie, cherry-pick, gałąź, a pod kreską wiersze, które coś przepisują, na przykład revert). Każdy z czterech pierwszych wierszy ma też polecenie w palecie (`Widok: Kolumna commitów: Pokaż ...`). Na liście Plik wiersze mają własne menu Historii pliku i te same podpowiedzi.
- **Rozmiar części i przewijanie na boki**: przeciągnij granicę w nagłówku, aby ustawić szerokość części po jej lewej: torów, opisu, autora, daty, refów, zmian lub SHA. Ostatnia część też ma granicę: komórka tuż za nią (własna krawędź kolumny, gdy wiersz wypełnia miejsce), świecąca tak samo, która ustawia szerokość tej ostatniej części. Jak w Logu (`ui.column_resize`), granicą jest domyślnie komórka separatora między dwiema częściami, w której kończy się odcień wiersza Graf (odcień biegnie przez nią, więc żadna zwykła komórka nie leży między nią a następną częścią): w nagłówku spoczywa w niej znacznik, a gdy wskaźnik jest na niej lub ją przeciąga, w dół kolumny rysuje się cienka linia, a przeciągnięcie z dowolnego punktu linii zmienia szerokość; przy `header` jest to pierwsza litera następnej nazwy, która świeci, gdy wskaźnik jest na niej (nie rysuje się żadna linia, nagłówek jest jedynym miejscem do przeciągania). Dopóki niczego nie ustawisz, opis bierze to, co zostawią inne; pierwsze przeciągnięcie ustala każdą część na jej bieżącej szerokości, także opis, więc zmiana jednej części przesuwa części za nią i nie zmienia żadnej innej szerokości. Część, której rozmiar ustawiłeś, jest zachowywana i nigdy nie ustępuje miejsca: gdy wiersz jest wtedy szerszy niż kolumna, przewija się na boki przez Shift+kółko, poziome kółko albo `Shift+Left` / `Shift+Right`, gdy kolumna ma klawiaturę, a pasek przewijania na jej dolnej krawędzi pokazuje, gdzie jesteś. Z klawiatury: `Tab` wybiera część (spośród widocznych), `]` ją poszerza, `[` zwęża, `0` przywraca wszystkim częściom szerokość automatyczną (paleta ma `Widok: Kolumna commitów: Wybierz następną część`, `... Poszerz część`, `... Zwęź część`, `... Zresetuj szerokości części`, `... Przewiń w lewo` i `... Przewiń w prawo`). Szerokości są zapamiętywane. Gdy tory są ucięte przez przewinięcie, są rysowane jako znaki, nawet w terminalu pokazującym obrazy.
- **Wybór części**: kliknięcie prawym przyciskiem w nagłówku otwiera menu z polem wyboru dla każdej części, czyli każdej kolumny Logu: Ref, Graf, Opis, Autor, Data, Zmiany i SHA. Opis jest zawsze widoczny. **Ref**, **Zmiany** i **SHA** są domyślnie wyłączone; włączone, Ref niesie etykiety gałęzi i tagów commita, Zmiany jego pliki i wiersze `+dodane -usunięte`, a SHA jego krótki skrót, wszystko jak własne kolumny Logu. Tak jak Ref, Zmiany i SHA nigdy nie ustępują miejsca: wiersz przewija się wtedy na boki. Z klawiatury `Widok: Kolumna commitów: Wybierz kolumny` otwiera to samo menu (strzałki i `Enter` wybierają wiersz), a `Widok: Kolumna commitów: Przełącz Ref`, `... Przełącz Graf`, `... Przełącz Autor`, `... Przełącz Data`, `... Przełącz Zmiany` i `... Przełącz SHA` przełączają część od razu. Widoczne części, ich kolejność i szerokości należą tylko do tej kolumny: są zapisywane w `config.toml` (`ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths`), osobno od kolumn Logu, które nigdy się z nimi nie zmieniają, ani odwrotnie.
- **Przesuwanie części**: przeciągnij nazwę w nagłówku nad inną, by przenieść tam część, jak w Logu (wiersz podąża za wskaźnikiem; przy `ui.column_resize = "header"` pierwsza litera nazwy jest granicą zmiany rozmiaru, więc naciśnij na pozostałą część nazwy). Każda część może trafić gdziekolwiek, także tory i etykiety Ref. Z klawiatury `Tab` wybiera część, a `<` / `>` przesuwają ją o jedno miejsce w lewo lub w prawo wśród widocznych (paleta ma `Widok: Kolumna commitów: Przesuń część w lewo` i `... Przesuń część w prawo`). Kolejność jest zapamiętywana.

  ![Menu prawego przycisku nagłówka kolumny commitów: pole wyboru dla Ref, Graf, Opis (zawsze widoczny), Autor, Data, Zmiany i SHA](../images/commit-column-menu.png)

Gesty myszy są też wymienione w części [Mysz i gesty](mouse-and-gestures.md#kolumna-commitów-lista-wszystkie).

## Rozmiary są zapamiętywane

GitPoulpi zapamiętuje każdy przeciągnięty rozmiar, także po przełączeniu
kart i po restarcie. Rozmiary są takie same dla każdego repozytorium. Dotyczy to
podziałów między panelem bocznym Refs, Logiem i prawym panelem, wysokości
każdej sekcji Refs, bloku opisu commita, szerokości kolumn Logu, paneli
edytora merge i podziału selektora folderów. Rozmiary są zapisywane jako części
okna, więc układ zrobiony na szerokim terminalu pasuje także do wąskiego.

Aby zresetować wszystkie rozmiary, uruchom polecenie palety
`Nawigacja: Resetuj układ`. Niektóre rozmiary mają też własny reset:
`Nawigacja: Resetuj rozmiary sekcji Refs`,
`Szczegóły commita: Resetuj rozmiar opisu`, a w edytorze merge dwukrotne
kliknięcie podziału lub `Alt+0`.

## Nakładki i ekrany specjalne

- **Paleta poleceń** (`Ctrl+P`): pole filtra nad każdym poleceniem. Każdy
  wiersz pokazuje swój skrót. Polecenie, którego nie można teraz uruchomić,
  jest wyszarzone, a powód widać na dole. Polecenia mają prefiks grupy, taki
  jak `Git:`, `Refs:`, `Nawigacja:`, `Diff:`, `Zmiany robocze:`, `Commit:`,
  `Widok:`, `Repozytorium:`, `Karta:`, `LFS:`, `Pull requesty:`, `Zgłoszenia:`
  lub `AI:`. Wpisz prefiks, aby zobaczyć jego grupę.
- **Selektory**: niektóre polecenia palety otwierają drugą listę, na przykład
  `Aplikacja: Zmień motyw...`, `Aplikacja: Zmień język...` i
  `Aplikacja: Zmień motyw kolorów grafu...`. Okno pokazuje każdy wybór na
  żywo, gdy poruszasz się po liście. `Enter` go zachowuje. `Esc` przywraca
  stary wybór. Lista motywów ma filtry **Wszystkie**, **Ciemne**, **Jasne**
  i **Kolorowe**. Zob. [Motywy](themes.md).
- **Pomoc** (`?`): karty. **Klawisze** wypisuje każdy klawisz i gest; pozostałe karty (Start, Log, Zmiany, Refs, Hosting, Ustaw., AI i CLI) to krótkie przewodniki po akcjach, z informacją, jak do nich dotrzeć. Kartę przełączasz klawiszami `Tab` / `Shift+Tab`, `Left` / `Right`, `Alt+1`..`Alt+8` lub kliknięciem. Wpisywanie filtruje bieżącą kartę. `Ctrl+D` albo kliknięcie linku na dole otwiera stronę dokumentacji karty, na której jesteś.

  ![Karta Hosting pomocy: krótki przewodnik, jak dotrzeć do każdej akcji](../images/help-hosting.png)

  ![Nakładka pomocy: sekcja Global z Ctrl+P, Ctrl+N, Ctrl+G, ? i klawiszami ruchu, potem sekcja Podstawy myszy](../images/help.png)
- **Pasek wyszukiwania** (`f`): w prawym górnym rogu Logu. Pokazuje `n of N`
  dopasowań.
- **Menu kontekstowe**: kliknij prawym przyciskiem albo naciśnij `.` na
  zaznaczonym wierszu. Wiersz z `›` otwiera podmenu.
- **Okna dialogowe**: potwierdzenia, nazwa gałęzi lub tagu, Klonuj, Init, Utwórz
  pull request, wybory po odrzuconym push i inne. `Enter` akceptuje, `Esc`
  anuluje.
- **Interaktywny rebase** i **kompozytor commitów** zastępują Log ekranem
  przeglądu. Zob.
  [Praca z Logiem](log-and-history.md#interaktywny-rebase) i
  [Stage i commity](staging-and-committing.md#kompozytor-commitów-podziel-zmiany-na-commity).

  ![Kompozytor commitów przeglądający cztery proponowane commity, każdy ze swoimi plikami, oraz przyciski Resetuj, Anuluj i Utwórz 4 commity](../images/composer.png)

- **Ustawienia** (`Ctrl+,` lub koło zębate na pasku tytułu): lista stron po lewej i opcje
  strony po prawej. Zob. [Ustawienia](settings.md).

  ![Ustawienia: lista stron po lewej, z polem wyszukiwania nad nią, i strona Ogólne po prawej](../images/settings.png)

- **Okna podręczne** dla długich odpowiedzi, takie jak okno AI **Wyjaśnianie
  zmian** (`↑` `↓` przewijają, `c` kopiuje, `Esc` zamyka).

  ![Okno Wyjaśnij zmiany robocze z wyjaśnieniem AI i listą rzeczy do ponownego sprawdzenia](../images/ai-explain.png)

Wszystkie listy w oknach podręcznych działają tak samo. Zob.
[Skróty klawiszowe](keybindings.md) i [Mysz i gesty](mouse-and-gestures.md).

## Słowa używane w tej dokumentacji

| Słowo | Znaczenie |
|---|---|
| Panel | Jedna z trzech kolumn: panel boczny Refs, Log (lub otwarty plik), prawy panel. |
| Plakietka | Etykieta gałęzi lub tagu w kolumnie Ref. |
| Wiersz WIP | Wiersz zmian roboczych na górze Logu. |
| Hunk | Jeden blok zmienionych linii w diffie. |
| Kompozytor | Formularz commita oraz przegląd AI, który dzieli zmiany na commity. |
| Akord | Klawisz, po którym następuje drugi, na przykład `Ctrl+G`, potem `s`. |
| Prefiks | `Ctrl+G` (akcje git) lub `Ctrl+N` (nawigacja). |
| Plakietka klawisza | Litera, którą `Ctrl+N` pokazuje obok kontrolki. Naciśnij ją, aby przenieść fokus na tę kontrolkę. |
