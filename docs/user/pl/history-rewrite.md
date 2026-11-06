[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · **Polski** · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Przepisywanie historii

Przepisywanie historii zmienia commity, które już istnieją: kto je napisał i kiedy,
ich opisy, pliki, które zawierają, ich podpisy. GitPoulpi robi to za pomocą
**przepisu** (zakresu i uporządkowanej listy reguł) oraz trzech kroków, z których
środkowego nigdy nie da się pominąć:

1. **Podgląd** oblicza nową historię, niczego nie ruszając: liczby, co zrobiła
   każda reguła, gałęzie i tagi, które by się przesunęły, jeden wiersz na każdy
   zmieniony commit.
2. **Zastosuj** przesuwa na nią twoje lokalne gałęzie i tagi. To jeden wpis w
   [historii cofania](undo-redo.md): **Cofnij** przywraca każdy przesunięty ref
   (oraz wyewidencjonowane pliki).
3. **Push** (force push z leasem) i **Purge** to osobne, świadome kroki, bo nie da
   się ich cofnąć: zob. [Wysyłanie wyniku](#wysyłanie-wyniku) i
   [Usuwanie starej historii](#usuwanie-starej-historii-purge).

Warsztat, paleta, polecenia `gitpoulpi rewrite` i narzędzia MCP używają tego samego
silnika, więc podgląd zrobiony w jednym miejscu jest tym, co zastosuje inne.
[Z wiersza poleceń](#z-wiersza-poleceń) wymienia flagi;
[Utwórz reguły z AI](#utwórz-reguły-z-ai) wypełnia warsztat na podstawie opisu;
[Propozycje od agentów AI](#propozycje-od-agentów-ai) mówi, co agent może tu zrobić,
a czego nie.

**Reguły według tego, co zmieniają**

| Grupa | Reguły | Część |
|---|---|---|
| Autorzy | Zamień autora, Zanonimizuj autorów, Zastosuj .mailmap | [Autorzy i .mailmap](#autorzy-i-mailmap) |
| Daty | Przesuń daty, Zmień strefę czasową, Ogranicz do godzin, Data committera = data autora | [Daty](#daty) |
| Opisy | Znajdź i zamień w opisach, Usuń linie opisu, Dodaj lub usuń trailer, Napraw kodowanie opisów | [Opisy](#opisy) |
| Zawartość plików i ścieżki | Znajdź i zamień w plikach, Usuń pliki, Usuń duże pliki, Przenieś lub zmień nazwę ścieżek, Konwertuj końce linii, Usuń bit wykonywalności, Konwertuj kodowanie plików | [Pliki i ścieżki](#pliki-i-ścieżki) |
| LFS | Migruj pliki do LFS, Przywróć pliki z LFS | [Przenoszenie plików do Git LFS i z niego](#przenoszenie-plików-do-git-lfs-i-z-niego) |
| Podpisy | Usuń podpisy (i ponowne podpisywanie przy Zastosuj) | [Podpisy](#podpisy) |
| Skrypt | Lua, shell, Python | [Reguła Skrypt](#reguła-skrypt) |

**Zanim zaczniesz**

- Podgląd niczego nie zmienia. Zastosuj zmienia tylko twoje lokalne refy i nigdy
  niczego nie wypycha.
- Przepisane commity dostają nowe id. Każdy, kto ma starą historię, musi o tym
  wiedzieć: [Wysyłanie wyniku](#wysyłanie-wyniku) pisze odpowiednią notatkę.
- Sekret, który został wypchnięty, jest ujawniony. **Najpierw wymień go na nowy**,
  potem przepisz historię, wypchnij ją siłą i usuń starą na stałe.
- Commity, które zostają bez zmian, zachowują swoje id i podpisy.

## Warsztat

`Ctrl+G Shift+H` (albo **Przepisanie: Warsztat historii** w palecie) otwiera
warsztat przepisywania nad Logiem. Nie ma przycisku na pasku narzędzi:
zaawansowane akcje są w palecie (każdy wiersz `Przepisanie: ...` otwiera warsztat
z już dodaną daną regułą), w podmenu **Przepisz historię** w menu commita
(*Od tego commita do HEAD...*, *Zaznaczone commity...* przy kilku zaznaczonych,
*Otwórz warsztat...*) oraz pod skrótem.

`Ctrl+N` pokazuje plakietkę przy każdej kontrolce warsztatu (zakres, reguły, pola, karty, przyciski); naciśnięcie plakietki robi to, co kliknięcie tej kontrolki, a `Esc` zdejmuje plakietki (zob. [Plakietki](keybindings.md#plakietki)).

![Stanowisko do przepisywania historii: pasek zakresu, lista dwóch reguł Zamień autora z polami wybranej reguły oraz podsumowanie podglądu z liczbą commitów, które się zmienią](../images/history-rewrite.png)

- **Pasek zakresu:** które refy się przesuną (bieżąca gałąź, wszystkie gałęzie
  lokalne, wybrane gałęzie i tagi, zaznaczone commity), pole *Tagi* oraz, po
  *Od tego commita do HEAD...*, pole pozwalające zacząć po rodzicu tego commita.
- **Reguły:** ponumerowana lista, stosowana po kolei. Pole wyboru włącza lub
  wyłącza regułę, strzałki ją przesuwają, krzyżyk usuwa (*Przywróć usuniętą
  regułę* lub `Ctrl+Z` ją przywraca). *Dodaj regułę* wyświetla rodzaje
  pogrupowane. Reguła, która wymaga programu, którego nie masz, pozostaje
  wyszarzona z podaniem powodu.
- **Ustawienia reguły:** kliknięcie pola tekstowego pozwala je edytować; pole
  z kilkoma wartościami otwiera swoją listę; pola wyboru się przełącza.
- **Podgląd** (`F5`) oblicza nową historię, niczego nie ruszając, i pokazuje
  pasek postępu. Karty: **Podsumowanie** (liczby, co zrobiła każda reguła,
  ostrzeżenia), **Commity** (jeden wiersz na zmieniony commit, stronicowane
  z demona; filtr wybiera zmienione, usunięte lub wypchnięte), **Gałęzie i tagi**
  (co się przesuwa oraz chronione gałęzie już wypchnięte, każda z polem wyboru),
  **Skutki** (co oznacza przepisanie wypchniętej historii: force push z lease,
  współpracownicy, podpisy), **Sekrety**, **Tożsamości** i **Rozmiar** (zob.
  niżej).
- **Pliki commita:** `Enter` (albo dwukrotne kliknięcie) na wierszu karty Commity
  otwiera pliki, które zmieniły w nim reguły zawartości, a obok nich diff
  zaznaczonego. Zamyka go krzyżyk, `Esc` albo kliknięcie poza oknem. Usunięte
  linie reguły sekretów są ukryte.
- **Zastosuj** (`Ctrl+S` albo `Ctrl+Enter`) jest dostępne, gdy podgląd zgadza się
  z regułami. Pytania bez wartości domyślnej są zadawane za każdym razem: co
  zrobić z podpisami podpisanych commitów i jak wyodrębnić folder. Panel wyniku
  oferuje **Cofnij**, które przywraca każdy przesunięty ref.
- **Zapisz / Wczytaj** przechowują przepis w repozytorium (wspólny) albo tylko
  dla ciebie; wartości tajne nigdy nie są zapisywane.

Każdy przycisk ma swój klawisz: lista jest w Ustawieniach > Klawiatura i mysz (kontekst
*Warsztat przepisywania historii*) i w przewodniku `keybindings.md`.

### Autorzy i .mailmap

- **Zamień autora** dopasowuje autora lub committera po nazwie i/lub e-mailu
  (dosłownie albo wyrażeniem regularnym) i ustawia nową nazwę i/lub e-mail.
  *Dotyczy* wybiera autora, committera albo obu; *Także autorzy tagów* sprawia,
  że odtwarzane tagi z adnotacją idą za nimi. Pierwotny committer zostaje, chyba
  że uruchomienie mówi *ja, teraz*.
- **Zanonimizuj autorów** zamienia każdą tożsamość na ponumerowany zamiennik
  (`Contributor 1`, `contributor-1@anonymous.invalid`; oba wzorce możesz zmienić,
  `{n}` oznacza numer). W jednym uruchomieniu ta sama osoba zawsze dostaje ten sam
  numer; tożsamości z listy *zostaw bez zmian* zostają, jak są.
- **Karta Tożsamości** (także **Przepisanie: Merge tożsamości (.mailmap)...**
  w palecie) pozwala naprawić osobę, która commitowała pod kilkoma nazwami i
  adresami e-mail, **niczego nie przepisując**. Wymienia każdą nazwę i e-mail
  z zakresu z paska zakresu, z liczbą commitów napisanych i zatwierdzonych przez
  każdą, oraz informacją, które z nich `.mailmap` repozytorium już przypisuje
  (`przypisana do Name <email>`). Zaznacz tożsamości, które są tą samą osobą
  (`Spacja` albo kliknięcie; *Zaznacz wszystkie* / *Odznacz wszystkie*), potem
  **Merge do...** (`m`): okno pyta, która z nich zostaje (pierwsza jest ta
  z największą liczbą commitów), i **dopisuje do `.mailmap` jedną linię na każdą
  pozostałą tożsamość** (`Name <email> Old Name <old@email>`; linia, która już
  tam jest, nie jest powtarzana, a brakujący plik zostaje utworzony). Nic nie jest
  zatwierdzane: zrób commit `.mailmap` samodzielnie, aby go udostępnić. Log,
  blame i historia pliku od razu pokazują nazwę kanoniczną (ustawienie
  `log.use_mailmap`, domyślnie włączone; zapisane commity się nie zmieniają, więc
  nie ma czego wypychać siłą). Pasek stanu mówi, co zapisano, i oferuje
  **Cofnij** (`u`), które przywraca poprzedni `.mailmap` (to jeden wpis historii
  cofania, jak każda inna akcja). `r` wczytuje listę ponownie.
- **`.mailmap` zamiast przepisania.** Gdy reguła autora zmienia commity, które
  są już na zdalnym repozytorium, Podsumowanie mówi *.mailmap poprawia sam
  wyświetlany tekst, bez przepisywania. Użyć .mailmap zamiast tego?* i pokazuje
  przycisk **Użyj .mailmap** (`m` na Podsumowaniu). Otwiera on kartę Tożsamości
  z zaznaczonymi tożsamościami, które dopasowuje reguła, oraz z tożsamością,
  którą reguła by ustawiła, zaproponowaną jako ta do zachowania, choć żaden commit
  jeszcze jej nie ma.
- **Reguła `.mailmap`** (*Przepisanie: Zastosuj .mailmap do historii...* albo
  `rewrite preview --mailmap`) naprawdę przepisuje: autorzy i committerzy (do
  wyboru) stają się tym, jak nazywa ich `.mailmap` drzewa roboczego, czyli tym,
  co pokazuje `git log --use-mailmap`. Użyj jej, gdy sama historia ma nosić
  nazwy kanoniczne, na przykład przed jej opublikowaniem; to przepisanie jak
  każde inne (podgląd, Zastosuj, force push).
- **Reguły tożsamości a trailery.** Reguła tożsamości może zmieniać też osoby
  wymienione w trailerach: zaznacz **Mapuj też trailery** (`--map-trailers`).
  Każdy trailer, którego wartość to dokładnie `Name <email>` i pasuje do *E-mail
  to* / *Nazwa to* z reguły, dostaje nową nazwę i e-mail, niezależnie od tego, co
  reguła robi z autorem i committerem. Domyślnie jest wyłączone: `Signed-off-by`
  zapisuje, kto podpisał, więc jego zmiana to osobna decyzja.

### Daty

Cztery reguły dat działają na datę autora, datę committera albo obie (*Daty*
w panelu reguły).

- **Przesuń daty** przesuwa każdą datę o podaną wartość (`+2h`, `-1d`, `90m`,
  `1d 2h`). Strefa czasowa, w której napisano każdy commit, zostaje.
- **Zmień strefę czasową** zapisuje daty w innym przesunięciu UTC. *Zachowaj*
  mówi, co zostaje: **chwila** (ten sam moment, zegar pokazuje inną godzinę) albo
  **zegar** (ta sama godzina lokalna, zmienia się moment).
- **Ogranicz do godzin** przenosi każdy commit spoza dozwolonych godzin (na
  przykład `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) na najbliższą dozwoloną
  porę albo tylko wcześniej, albo tylko później (*Przesuń na*). *Nigdy wcześniej
  niż rodzic* zachowuje kolejność historii; *Ziarno* sprawia, że niewielki
  losowy rozrzut da się powtórzyć: to samo ziarno na tej samej historii daje te
  same daty.
- **Data committera = data autora** ustawia committera (nazwę, e-mail i datę) na
  dane autora, czyli to, co cofnęło `git commit --amend` albo rebase.

Daty są regułami jak inne: podgląd, sprawdź kartę **Commity** (każdy wiersz
pokazuje starą i nową datę), potem zastosuj. Id commita zależy od jego dat, więc
reguła dat zmienia każdy commit od pierwszego, którego dotknie.

### Opisy

- **Znajdź i zamień w opisach** oraz **Usuń linie opisu** przyjmują dosłowny tekst
  albo wyrażenie regularne (*Wyrażenie regularne*, *Rozróżniaj wielkość liter*);
  pusty zamiennik usuwa. Opis, który nie jest poprawnym UTF-8, nigdy nie jest przez
  nie ruszany: jeśli chcesz go edytować, uruchom najpierw **Napraw kodowanie
  opisów**.
- **Trailery.** *Dodaj lub usuń trailer* (*Przepisanie: Dodaj lub usuń trailery...*
  w palecie albo `--trailer-add`, `--trailer-remove`, `--trailer-map`) edytuje
  linie `Klucz: wartość` na końcu opisu, te, które zna `git interpret-trailers`
  (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...). Lista **Działanie** daje
  *Dodaj* (klucz **Trailer**, jego **Wartość** i **Jeśli już istnieje**: *Dodaj
  (chyba że identyczny)*, *Zastąp istniejące* albo *Nie zmieniaj commita*),
  *Usuń* (każdy trailer o tym kluczu albo tylko te, których wartość zawiera
  **Wartość zawiera**) lub *Zmień wartość* (**Szukaj w wartości** zamieniane na
  **Zamień na**, na przykład stary e-mail na nowy w `Signed-off-by`). Klucze są
  dopasowywane bez względu na wielkość liter. Zmieniają się tylko linie trailerów,
  reszta opisu zostaje dokładnie taka, jaka była, a opis, który nie ma jeszcze
  trailerów, dostaje pustą linię i własny blok. Karta Commity pokazuje każdą
  zmianę jako `+Signed-off-by: ...` / `-Co-authored-by: ...`. Trailerami jest
  ostatni akapit opisu (nigdy tytuł), więc linia taka jak `Note: see the ticket`
  w treści nie jest trailerem, chyba że stoi sama w ostatnim akapicie razem z
  innymi trailerami.
- **Id commitów w opisach.** **Zaktualizuj id commitów** (`--update-message-hashes`,
  domyślnie wyłączone; na końcu paneli reguł, które edytują opisy) sprawia, że id
  zapisane w opisach podążają za przepisaniem: skrócone id z 7 lub więcej znaków
  szesnastkowych (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`), które wskazuje
  dokładnie jeden commit przepisanej historii, jest zamieniane na nowe id tego
  commita, obcięte do tej samej długości. Id pasujące do kilku commitów, do żadnego
  albo do commita, który przepisanie tworzy dopiero później, zostaje, jakie było;
  Podsumowanie mówi, ile zamieniono, a ile zostało.
- **Napraw kodowanie opisów** (*Przepisanie: Napraw kodowanie opisów...* albo
  `--fix-message-encoding [--fix-name-encoding]`) koduje opisy na nowo w UTF-8, w
  dwóch przypadkach:
  - Commit z nagłówkiem `encoding` (`ISO-8859-1`, `Windows-1252`, `Shift_JIS`,
    cokolwiek znają git i tablice kodowań) jest dekodowany i zapisywany jako UTF-8,
    a nagłówek znika: tekst czyta się tak samo wszędzie, bez żadnej konwersji.
    Nazwy autora i committera tego commita były w tym samym kodowaniu i są
    dekodowane razem z nim, więc commit nigdy nie zostaje skonwertowany do połowy.
  - **Mojibake**, czyli UTF-8 odczytane kiedyś jako Latin-1 lub Windows-1252
    (`RÃ©Ã©criture` zamiast `Réécriture`), jest naprawiane **tylko wtedy, gdy
    konwersja w obie strony daje dokładnie to samo**: tekst musi zakodować się z
    powrotem do bajtów będących poprawnym UTF-8, a zepsucie wyniku w ten sam
    sposób musi dać oryginał. Opis, który tylko zawiera `Ã©` obok prawdziwych liter
    albo znaki spoza starych alfabetów, nie przechodzi i zostaje, jak jest.

  *Napraw też nazwy* (`--fix-name-encoding`) naprawia w ten sam sposób zepsute
  nazwy autorów i committerów. To, czego nie da się rozstrzygnąć (nieznana nazwa
  kodowania, bajty niepoprawne w zadeklarowanym kodowaniu, opis, który nie jest
  UTF-8 i niczego nie deklaruje, tekst wyglądający na zepsuty, ale bez dowodu),
  **zostaje bez zmian i jest zliczane**: Podsumowanie mówi *n opisów zostało bez
  zmian: nie da się rozstrzygnąć ich kodowania*, a liczba jest w
  `counts.encoding_left`. Karta Commity pokazuje każdą poprawkę ze starym i nowym
  opisem oraz wierszem *kodowanie* (`ISO-8859-1` zmienia się w `UTF-8`).

### Pliki i ścieżki

- **Reguły zawartości plików** (*Znajdź i zamień w plikach*, *Usuń pliki*,
  *Usuń duże pliki*): panel sprawdza każdą wartość w trakcie pisania i pokazuje
  powód pod polem (wzorzec, który się nie kompiluje, błędny wzorzec ścieżki,
  rozmiar poniżej jednego bajta). Rozmiary przyjmują jednostki (`500 KiB`,
  `5 MiB`, `1 GiB`). Wzorzec ścieżki bez ukośnika pasuje na dowolnej głębokości.
  *Wartości tajne* wpisuje się w zamaskowanym polu, są używane tylko podczas tego
  uruchomienia i nigdy nie są zapisywane; podgląd ukrywa linie, które je
  zawierały. *Puste commity* określa, czy commit, który po regułach został pusty,
  jest usuwany (domyślnie), czy zostaje. Pliki binarne i pliki powyżej limitu
  rozmiaru zostają bez zmian i są zliczane w Podsumowaniu.
- **Przenoszenie ścieżek.** *Przenieś lub zmień nazwę ścieżek* (*Przepisanie:
  Przenieś lub zmień nazwę ścieżek...* w palecie) ma trzy rodzaje, wybierane na
  liście **Przenieś**:
  - *Zmień nazwę ścieżki* (**Z**, **Do**; `--move-path FROM=TO`): te same pliki i
    foldery pojawiają się pod nową ścieżką w każdym commicie, który miał starą;
    nic innego się nie zmienia, każdy plik zachowuje swoje id. Commit, w którym
    nowa ścieżka już istnieje, nigdy nie jest nadpisywany: podgląd zostaje
    odrzucony i wymienia commity oraz kolidujące ścieżki (wybierz wolną nazwę albo
    najpierw przenieś istniejącą). Tylko dokładne ścieżki, bez wildcardów.
  - *Przenieś wszystko do folderu* (**Folder**; `--into-subdir DIR`): każda
    ścieżka w każdym commicie dostaje ten przedrostek. Przydatne do łączenia
    repozytoriów bez kolizji nazw.
  - *Wyodrębnij folder jako korzeń* (**Folder**; `--extract DIR`): folder staje się
    korzeniem, commity, które go nie dotykają, są pomijane (commit, który go
    tworzy, nie ma rodzica), merge'e, które niczego już nie łączą, są upraszczane,
    a gałęzie i tagi, które nie mają nic z tego folderu, są usuwane. **Za każdym
    razem** jesteś pytany, jak to się kończy:
    - *Do nowego klonu, otwartego w nowej karcie* (`--extract-mode clone`): w
      wybranym folderze powstaje kopia tego repozytorium (nowe repozytorium dostaje
      w nim folder nazwany jak wyodrębniony), jest tam przepisywana i czyszczona,
      tak że zawiera tylko nową historię i każdą lokalną gałąź oraz tag, które mają
      ten folder. To repozytorium nie jest ruszane. Nowe repozytorium nie ma
      zdalnego (oryginał nie jest jego upstreamem).
    - *Na miejscu: historia tego repozytorium jest przepisywana* (`--extract-mode
      in-place`): zwykłe przepisanie, podgląd, Zastosuj, cofnięcie. Gałęzie bez
      folderu są usuwane (cofnięcie je przywraca); gałąź z checkoutem musi go mieć.
    - *Jako submoduł tego repozytorium* (`--extract-mode submodule`): nowe
      repozytorium powstaje tak jak w przypadku klonu, a potem **jeden nowy
      commit** na bieżącej gałęzi zastępuje folder jego submodułem (`.gitmodules`
      i dowiązanie); folder zostaje jako wyewidencjonowany submoduł. Historia tego
      repozytorium **nie** jest przepisywana: jego stare commity zachowują pliki
      folderu, więc nic, co na nie wskazuje (tagi, inne klony), się nie psuje.
      Adres URL w `.gitmodules` to lokalna ścieżka nowego repozytorium: zmień go
      (`git config -f .gitmodules submodule.<folder>.url`), gdy już będzie na
      hoście. Edytowane lub nieśledzone pliki w folderze to blokują. Cofnięcie
      przywraca zwykły folder.
    Podsumowanie mówi, że podgląd pokazuje historię nowego repozytorium i do
    jakiego folderu trafia. Reguły po wyodrębnieniu (zmiana nazwy, usunięcie)
    działają na wyodrębnionym drzewie; dla klonu lub submodułu wyodrębnienie musi
    być pierwszą regułą ścieżek. Reguły ścieżek działają po regułach zawartości,
    więc wzorzec usuwania wskazuje ścieżki takie, jakie były.

#### Końce linii, bit wykonywalności i kodowanie plików

Trzy reguły w grupie **Zawartość** menu Dodaj (albo wiersze palety *Przepisanie:
Konwertuj końce linii...*, *Usuń bit wykonywalności...* i *Przepisanie:
Konwertuj kodowanie plików...*). Wszystkie trzy przyjmują **Pliki** (wzorce glob;
wzorzec bez ukośnika pasuje na dowolnej głębokości; puste oznacza każdy plik)
oraz **Z wyjątkiem plików**. Zmieniają tylko to, co wskazują wzorce, w każdym
commicie zakresu. Pliki binarne (bajt NUL w pierwszych 8 KiB) nigdy nie są
ruszane: Podsumowanie zlicza je jako *bez zmian*.

- **Konwertuj końce linii** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Konwertuj na*: LF (Linux, macOS), CRLF (Windows)
  albo CR (klasyczny Mac). Każdy koniec linii w pasującym pliku tekstowym zmienia
  się na ten, więc plik z mieszanymi końcami wychodzi jednolity; pliki już
  poprawne zachowują swoją wersję bez zmian. *Uwzględniaj .gitattributes*
  (domyślnie wyłączone, `--eol-follow-attributes`) czyta główny `.gitattributes`
  każdego commita: pliki oznaczone `-text` lub `binary` są pomijane, a pliki
  oznaczone `eol=lf` lub `eol=crlf` zachowują ten koniec linii zamiast wybranego
  przez ciebie.
- **Usuń bit wykonywalności** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Pasujące pliki przechodzą z trybu 100755 na 100644; ich zawartość
  nie jest ruszana. Wymagany jest co najmniej jeden wzorzec plików: zdjęcie bitu
  ze wszystkich wykonywalnych zepsułoby twoje skrypty.
- **Konwertuj kodowanie plików** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Konwertuj na*: UTF-8 (domyślnie), UTF-8 z BOM, UTF-16 LE lub BE
  (zapisywane ze znacznikiem kolejności bajtów), Windows-1252 lub ISO-8859-1.
  *Pliki są w kodowaniu*: domyślnie kodowanie każdego pliku jest wykrywane
  (znacznik kolejności bajtów, w przeciwnym razie poprawny UTF-8, a jeśli nie, to
  *Zapasowe*, czyli Windows-1252, o ile tego nie zmienisz); wskaż kodowanie, aby je
  wymusić, ale błędny wybór daje błędny tekst (UTF-8 odczytane jako Latin-1
  podwaja każdy akcent). Plik, którego kodowanie źródłowe nie umie odczytać, albo
  który zawiera znak, którego kodowanie docelowe nie pomieści (znak euro w
  ISO-8859-1), zostaje dokładnie taki, jaki jest, i jest zliczany w Podsumowaniu.

Reguły działają w kolejności listy, na tym samym pliku: konwersja pliku do UTF-16
i potem prośba o końce linii zostawia go w spokoju (to już nie jest zwykły
tekst). Zastosowanie to jeden wpis na liście cofania, jak każde przepisanie.

### Szukanie sekretów

- **Szukanie sekretów:** `gitpoulpi rewrite scan-secrets` (i narzędzie MCP
  `rewrite_scan_secrets`) czyta historię jednorazowo i wymienia prefiksy tokenów
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), klucze
  prywatne, JWT, hasła w adresach URL i wyglądające na losowe linie
  `password = value`, każdą z plikiem, pierwszym i ostatnim commitem oraz
  zamaskowanym podglądem, np. `ghp_****abcd`. Skan nie jest wyczerpujący, a
  wartość nigdy nie jest pokazywana, logowana ani zapisywana. `--remove-finding
  ID` zamienia znalezisko w regułę, która usuwa je z każdego commita. Wypchnięty
  sekret jest ujawniony: najpierw wymień go na nowy, potem przepisz historię,
  wypchnij ją siłą i usuń starą na stałe.
- **Karta Sekrety** robi to samo z warsztatu, dla zakresu z paska zakresu:
  **Szukaj sekretów** (`s`) uruchamia skan jako zadanie (pasek postępu ma podpis
  *Skanowanie*, a **Anuluj** lub `Esc` go zatrzymuje). Każde znalezisko to
  wiersz z rodzajem, zamaskowanym podglądem, plikiem i liczbą innych plików, które
  je zawierają; pod listą widać pierwszy i ostatni commit, w którym je
  zobaczono, oraz w ilu commitach i wersjach plików występuje. Karta mówi też, ile
  plików binarnych i zbyt dużych pominięto, oraz że skan nie jest wyczerpujący.
  Wszystkie znaleziska są na początku zaznaczone; **Usuń z historii** (`r`)
  zamienia zaznaczone w jedną regułę, która wskazuje je po id (nigdy po
  wartości), a którą oglądasz w podglądzie i stosujesz jak każdą inną.
  Po przepisaniu lub zmianie zakresu znaleziska się dezaktualizują: karta o tym
  informuje, przeskanuj ponownie. Warsztat nigdy nie przechowuje wartości sekretu,
  więc nie może jej pokazać.

### Szukanie tego, co waży

- **Szukanie tego, co waży:** `gitpoulpi rewrite analyze [--top N]` (i narzędzie
  MCP `rewrite_analyze`) to analiza rozmiaru, jak `filter-repo --analyze`:
  największe pliki ze ścieżką, rozmiarem, pierwszym i ostatnim commitem oraz
  informacją, czy nadal są w HEAD, największe foldery, pliki usunięte z HEAD,
  które historia wciąż przechowuje, i sumy. Czyta tylko rozmiary i ścieżki, nigdy
  zawartość plików, i niczego nie zmienia.
- **Karta Rozmiar** robi to samo z warsztatu (`F7` albo *Analizuj rozmiary
  historii* w palecie otwiera ją i uruchamia analizę): **Analizuj rozmiary** (`s`)
  to zadanie (*Analizowanie*, **Anuluj** lub `Esc` je zatrzymuje). **Pokaż** wybiera
  listę (`v`): *Pliki* (największe unikalne wersje plików, każda oznaczona jako
  *w HEAD*, *stara wersja* pliku, który nadal istnieje, albo *usunięty*), *Foldery*
  (wszystko, co kiedykolwiek zapisano pod każdym z nich) lub *Usunięte* (ścieżki,
  których nie ma już w HEAD, a które historia zachowuje). **Sortuj** (`o`) działa
  według rozmiaru, ścieżki lub usuniętych najpierw. Zaznaczaj wiersze (`Spacja`,
  `Alt+A` / `Alt+U` dla wszystkich / żadnego): karta mówi, ile zajmują, uczciwie
  jako "do około", bo miejsce wraca dopiero po przepisaniu, usunięciu na stałe
  starych obiektów i `gc`. **Usuń z historii** (`r`) zamienia zaznaczone ścieżki
  w regułę ścieżek (znaki specjalne są poprzedzone znakiem ucieczki, ścieżka
  będąca już w regule nie jest dodawana dwa razy); **Usuń każdy plik powyżej ...**
  (`b`, lista Pliki) zapisuje regułę rozmiaru tuż poniżej najmniejszego
  zaznaczonego pliku. Podgląd i zastosowanie jak przy każdej innej regule. Listy
  pokazują 200 największych wierszy; CLI mówi, ile jest więcej, a `--top` je
  wymienia. Po przepisaniu lub zmianie zakresu wiersze się dezaktualizują:
  przeanalizuj ponownie.

### Podpisy

- **Podpisy:** przepisany commit nie może zachować podpisu (podpisywał stare
  bajty), więc gdy przepisywane są podpisane commity, za każdym razem jesteś
  pytany: *Usuń podpisy* (najpierw widać liczbę), *Podpisz ponownie tylko te
  commity, które były podpisane* albo *Podpisz ponownie każdy przepisany commit*.
  Nie ma wartości domyślnej. Commity, które zostają bez zmian, zachowują swoje.
  Ponowne podpisywanie używa twojej własnej konfiguracji, tak jak podpisany commit:
  `gpg.format` (`openpgp`, `ssh` lub `x509`), `user.signingkey` oraz `gpg.program`
  / `gpg.ssh.program` / `gpg.x509.program`. Czwarty wiersz pytania, **Sprawdź
  podpisywanie** (`4`), raz podpisuje próbny ładunek i go weryfikuje: mówi, ile
  trwa jeden podpis, i szacuje czas dla liczby commitów, o które pytano, albo
  podaje, dlaczego nie działa (brak klucza, zablokowany agent, niezainstalowany
  program, wraz ze wskazówką instalacji). Podgląd robi ten sam test, zanim
  przeczyta historię, i odmawia z tym powodem, więc zepsuta konfiguracja nigdy nie
  zostawia do połowy podpisanego przepisania. Commity są podpisywane **podczas
  podglądu** (ich id zawierają podpis), więc podgląd z ponownym podpisywaniem
  trwa tyle, ile podpisywanie, Podsumowanie mówi *Ponownie podpisane commity: n, w
  12,5 s*, a wtedy może pojawić się prośba o hasło: najpierw odblokuj agenta
  (Sprawdź podpisywanie robi dokładnie to). **Anuluj** zatrzymuje między dwoma
  podpisami i niczego nie zostawia. Odtwarzane są tylko podpisy commitów;
  podpisany tag nadal traci podpis.
- **Reguła *Usuń podpisy*** (menu Dodaj, grupa *Podpisy*; *Przepisanie: Usuń
  podpisy...* w palecie; `--remove-signatures`) usuwa podpis z każdego commita w
  zakresie, nawet tam, gdzie nic innego się nie zmienia, i o ponowne podpisywanie
  nie pyta. Z CLI: `rewrite check-signing` oraz `--signatures
  strip|resign-signed|resign-all`.

### Cofanie przepisania

**Cofnij** (przycisk panelu wyniku, *Cofnij* na pasku narzędzi z `Ctrl+Z` w Logu
albo `gitpoulpi undo`) przywraca każdą przesuniętą gałąź, tag i notatkę, a razem z
nimi wyewidencjonowane pliki, ze starych commitów, które przepisanie przypina.
Najpierw pyta, gdy przepisana gałąź jest już na swoim upstreamie (lokalne
cofnięcie nie cofa pusha) i gdy masz niezatwierdzone zmiany: te zostają, a plik,
który zmieniłeś i który stara historia by nadpisała, zatrzymuje cofnięcie
(pliki przesuwają się tak, jak robi to `reset --keep`). Cofnięcie przestaje być
możliwe dla przepisania, gdy [usuniesz na stałe](#usuwanie-starej-historii-purge)
jego starą historię; lista historii oznacza je wtedy jako *usunięte na stałe*.

## Przenoszenie plików do Git LFS i z niego

Dwie reguły w grupie *LFS* menu Dodaj przepisują historię tak, aby duże pliki od
początku leżały w [Git LFS](lfs.md) (albo go opuszczały). Potrzebują programu
`git-lfs`: bez niego reguły są wyszarzone ze wskazówką instalacji, a podgląd
zostaje odrzucony. To nasz własny silnik, nie `git lfs migrate`, więc dostajesz
podgląd, zakres, cofanie i zabezpieczenie pusha jak przy każdej innej regule.

- **Migruj pliki do LFS** (*Przepisanie: Migruj pliki do LFS...* albo
  `--lfs-import PATTERN` z `--lfs-min-size BYTES`). Każdy plik, który pasuje do
  wzorców (`*.psd`, `assets/**`; wzorzec bez ukośnika pasuje na dowolnej
  głębokości) i ma co najmniej rozmiar z pola *Tylko pliki o rozmiarze od* (puste:
  każdy pasujący), staje się wskaźnikiem LFS w każdym commicie, który go zawiera.
  Jego zawartość trafia do lokalnego magazynu LFS (`.git/lfs/objects`, raz dla
  każdej różnej zawartości) przy Zastosuj, a główny `.gitattributes` dostaje
  `<wzorzec> filter=lfs diff=lfs merge=lfs -text` w każdym przepisanym commicie,
  który zawiera wskaźnik (tworzony, gdy go nie ma, pozostałe linie nietknięte,
  nigdy nie dublowane). Plik, który już jest wskaźnikiem, zostaje. Podsumowanie
  mówi *3 pliki (12 MiB) trafiają do LFS. Obiekty LFS do zapisania: 3.*, a karta
  Commity wymienia *przenosi big.psd do LFS* przy każdym commicie.
- **Przywróć pliki z LFS** (*Przepisanie: Przywróć pliki z LFS...* albo
  `--lfs-export PATTERN`). Pasujące wskaźniki znów stają się prawdziwymi plikami,
  a linie `.gitattributes` dla tych wzorców są usuwane. Obiekty muszą być w
  lokalnym magazynie LFS: gdy części brakuje, podgląd zostaje odrzucony z listą i
  przyciskiem **Pobierz obiekty LFS**, który pobiera je z `origin`; potem zrób
  podgląd ponownie. Import, a potem eksport tych samych wzorców daje dokładnie te
  same commity.

Po Zastosuj wyewidencjonowane pliki to prawdziwa zawartość (nie wskaźniki), a
**Cofnij** przywraca starą historię. Panel wyniku ma **Wyślij obiekty LFS** (`l`):
to samo co `LFS: Push All Objects`, więc nową historię można wypchnąć bez tego,
że późniejszy push utknie na brakujących obiektach (zwykły push przepisanych
gałęzi też je wysyła, przez hook pre-push, który instaluje Zastosuj). Notatka dla
współpracowników mówi, że potrzebują git-lfs i `git lfs pull`.

Warto wiedzieć: Zastosuj instaluje w repozytorium filtry LFS (`git lfs install
--local`), gdy ich brakuje, bo git potrzebuje ich do porównania prawdziwych plików
ze wskaźnikami. Zapisywany jest tylko główny `.gitattributes`, nigdy zagnieżdżony.
Plik *poniżej* minimalnego rozmiaru, który pasuje do wzorca, zostaje zwykłym
plikiem, ale git będzie chciał zamienić go we wskaźnik przy następnym dodaniu, więc
`git status` pokazuje go jako zmodyfikowany, dopóki go nie zatwierdzisz. Import
przesyła duże pliki strumieniowo, eksport trzyma w pamięci jeden plik naraz.

## Wysyłanie wyniku

Zastosowanie przepisania nigdy niczego nie wypycha. Przepisane gałęzie i tagi,
które były na zdalnym repozytorium, wymagają force push, a wypchnięcie nigdy nie
wymusza na ślepo: każdy ref jest wysyłany z leasem na id, które zdalne miało
w chwili podglądu (a nie to, co mówi teraz twoja gałąź śledząca), więc push
współpracownika w międzyczasie zostaje dla tego refa odrzucony zamiast
nadpisany, a refy idą jednym wypchnięciem typu wszystko albo nic (`--atomic`),
tam gdzie serwer na to pozwala. Z wiersza poleceń:

```
gitpoulpi rewrite push --dry-run        # pyta zdalne, niczego nie wysyła
gitpoulpi rewrite push --yes            # wysyła
gitpoulpi rewrite note                  # co mają uruchomić pozostali
gitpoulpi rewrite maps --out ./maps     # commit-map i ref-map
```

Gałęzie chronione (ustawienie `rewrite.protected_branches`, domyślnie `main`,
`master`, `develop`, `release/*`) są pomijane przy wypychaniu, i przy przepisaniu,
i znowu przy wypchnięciu, chyba że je wskażesz (`--include-protected main`) albo
dodasz wszystkie. O tym, czy host przyjmie force push, nadal decyduje jego własna
ochrona gałęzi; jej odmowa jest pokazana dla każdego refa.

W warsztacie robi to wszystko panel wyniku po Zastosuj:

- **Wyślij przepisane refy do zdalnego repozytorium** wymienia refy, które
  przepisanie przesunęło, a które ma jakieś zdalne, każdy z polem wyboru
  (`Spacja`). Gałęzie chronione zaczynają niezaznaczone, z napisem *chroniona*
  obok: zaznacz je pojedynczo albo użyj *Zaznacz wszystkie* / *Odznacz wszystkie*
  (`Alt+A` / `Alt+U`). **Push przepisane gałęzie** (`p`) wysyła zaznaczone;
  odpowiedź to jeden wiersz na ref (*wypchnięto*, *już aktualne*, *odrzucone:
  zdalne repozytorium zmieniło się od podglądu*, *odrzucone: chronione przez
  hosta*, *pominięte: gałąź chroniona*, *nie wypchnięto*). Wiersz odrzucony ma
  **Fetch i sprawdź** (`f`): warsztat się zamyka, a zdalne są pobierane, więc
  Log pokazuje to, co wypchnęli inni. Nic nie zostaje nadpisane: lease chroni ich
  pracę, a przed kolejną próbą robisz podgląd ponownie.
- **Kopiuj notatkę** (`c`) wkłada notatkę dla współpracowników do schowka,
  w języku interfejsu, z nieprzetłumaczonymi poleceniami git wokół.
- **Otwórz mapę commitów** (`m`) pokazuje, gdzie leżą mapa commitów i mapa refów;
  kliknięcie ścieżki (albo `1` / `2`) kopiuje ją.

Wypchnięcia nie da się cofnąć: jeśli potem zrobisz `undo` przepisania, zmieni się
tylko twoje repozytorium, a cofnięcie mówi o tym, zanim zapyta. Wyślij notatkę
każdemu, kto ma starą historię: wymienia stary i nowy czubek każdej gałęzi i tagu
oraz polecenia (`git fetch`, potem `git reset --hard origin/<branch>` lub
`git rebase --onto`). Obie mapy są przechowywane przez 24 godziny razem
z uruchomieniem; skopiuj je, jeśli potrzebuje ich czyszczenie repozytorium
w GitLab albo skrypt.

## Gdy host chroni gałąź

Host może sam chronić gałąź (ochrona gałęzi i rulesety w GitHub, chronione gałęzie
w GitLab, polityki gałęzi w Azure DevOps, ograniczenia gałęzi w Bitbucket) i
odrzucić force push, którego potrzebuje przepisanie. Gdy repozytorium jest
**powiązane** ze swoim hostem (sekcja pull requestów w panelu bocznym, zob.
[Integracja z hostingiem](hosting-integration.md)), warsztat pyta host, które
gałęzie chroni, twoim własnym tokenem:

- Te gałęzie są **domyślnie pomijane w przepisaniu**, dokładnie tak jak gałęzie
  chronione z `rewrite.protected_branches` i z workflow: karta Refs wymienia je,
  każdą z polem wyboru, oraz przyciski *Zaznacz wszystkie* / *Odznacz wszystkie* i
  mówi dlaczego: *chroniona przez GitHub*.
- Jeśli token nie może odczytać ochrony (brakuje mu zakresu albo nie jesteś
  administratorem), warsztat mówi *Ochrona po stronie hosta nieznana* i nie uznaje
  za chronione niczego na słowo hosta. Przepisanie nigdy nie jest przez to
  blokowane. Repozytorium, które nie jest powiązane, nigdy nie jest odpytywane.

Gdy chroniona gałąź stoi na przeszkodzie (podgląd został odrzucony, bo każda
gałąź jest chroniona, albo host odrzucił push: *odrzucone: chronione przez
hosta*), przycisk **Jak to obejść** (`Alt+H`) wyjaśnia trzy drogi wyjścia, z
dokładną stroną ustawień twojego dostawcy i repozytorium (zaznacz link i skopiuj go
klawiszem `c` albo otwórz klawiszem `o`). GitPoulpi nigdy nie robi żadnej z nich
za ciebie:

1. **Tymczasowo zdejmij ochronę** w ustawieniach hosta, zrób push, potem ją
   przywróć.
2. **Wypchnij przepisaną historię pod nową nazwą gałęzi**, potem ustaw tę gałąź
   jako domyślną (albo zmień nazwę) i usuń starą, gdy nikt jej już nie używa.
3. **W ostateczności usuń zdalne repozytorium i utwórz je od nowa** oraz wypchnij
   wszystko. Tracisz przy tym pull / merge requesty i ich przeglądy, zgłoszenia,
   powiązanie forków ze starą historią, gwiazdki i obserwujących, ustawienia CI,
   sekrety i zmienne oraz webhooki i klucze wdrożeniowe.

Z CLI ten sam tekst wraca jako `protected_by_host` w JSON-ie `rewrite preview` /
`rewrite run` (gdy host chroni gałąź z zakresu), odrzuconego `rewrite push` i
`rewrite note`: `provider`, `branches`, `ways` (każda z `steps` i `links`),
`recreate_loses` oraz całe wyjaśnienie jako `text`.

## Usuwanie starej historii (purge)

Po przepisaniu stare commity nadal są w twoim repozytorium: trzyma je cofnięcie
przepisania (po to, żeby dało się je cofnąć), a także reflogi. Jeśli przepisałeś
historię, żeby pozbyć się sekretu, ten sekret nadal leży w folderze `.git`.
Purge usuwa starą historię na stałe:

```
gitpoulpi rewrite purge                 # pyta: wymienia, co zostanie usunięte i utracone (kod wyjścia 3)
gitpoulpi rewrite purge --yes           # wykonuje
```

Porzuca przypięcia cofania tego przepisania, wygasza wpisy reflogu, które
wskazują stare commity, uruchamia `git gc --prune=now`, a potem sprawdza, czy
każdy stary commit i każda wersja pliku naprawdę zniknęły. Potwierdzenie mówi
z góry, co tracisz: **cofnięcie przestaje być możliwe** dla tego przepisania (i
dla każdej wcześniejszej operacji, której cofnięcie też potrzebowało starych
commitów); zostają na liście historii, oznaczone jako usunięte na stałe. Purge
nie da się zatrzymać po rozpoczęciu ani cofnąć. Nie dotyka żadnego zdalnego
repozytorium.

Purge blokuje wszystko, co nadal trzyma starą historię, i wtedy nic się nie
zmienia. Lista wymienia każdą taką rzecz i podpowiada, co zrobić: gałąź lub tag,
które nie były częścią przepisania, stash na starej historii (porzuć go albo
zostaw i zaakceptuj, że stara historia zostaje), inny worktree, którego HEAD na
niej stoi, oraz gałęzie śledzące zdalne, które nadal wskazują starą historię
(wypchnij przepisane gałęzie, potem zrób fetch). Lista stashy i reflogi innych
gałęzi nigdy nie są ruszane.

Sekret, który został wypchnięty, jest ujawniony: **najpierw wymień go na nowy**.
Po purge odpowiedź wymienia, czego host jeszcze potrzebuje, osobno dla każdego
zdalnego: GitHub trzyma stare commity dostępne w widokach z pamięci podręcznej
i refach pull requestów (napisz do pomocy technicznej GitHub albo usuń i utwórz
repozytorium od nowa), GitLab ma czyszczenie repozytorium, które przyjmuje
`commit-map` (`gitpoulpi rewrite maps --out DIR`) oraz konserwację, Azure DevOps
i Bitbucket wymagają zgłoszenia do pomocy technicznej albo nowego repozytorium,
a każdy, kto ma klon lub fork, nadal ma starą historię.

### Ekran purge

W warsztacie panel wyniku przepisania ma przycisk **Usuń starą historię na
stałe...** (klawisz `e`), a paleta ma **Przepisanie: Usuń starą historię na
stałe...** (działa na najnowsze przepisanie, które jeszcze nie zostało usunięte
na stałe, także po zamknięciu warsztatu). Ekran wymienia, co purge usuwa i co
tracisz (operacje, których cofnięcie znika razem z nim; te oznaczone *traci też
cofanie* nie są przepisaniami, ale potrzebowały starych commitów), a potem to,
co nadal trzyma starą historię, po jednym wierszu:

- gałąź śledząca zdalne: **Fetch** aktualizuje je ze zdalnych;
- stash: **Porzuć** pyta najpierw, bo stash to praca, której nigdy nie
  zatwierdzono;
- gałąź, tag, worktree lub inny ref: **Kopiuj nazwę**, a potem zajmij się nim
  tam, gdzie leży (przepisz lub usuń gałąź, przełącz przepisaną gałąź
  w worktree).

Sprawdzenie uruchamia się ponownie po każdej poprawce, a **Sprawdź ponownie**
(`r`) uruchamia je na żądanie. **Usuń na stałe** (`p`) jest dostępne dopiero, gdy
nic nie zostaje. Zadaje ostatnie pytanie, ponumerowane i bez domyślnej
odpowiedzi (`1` usuwa na zawsze, `2` zostawia wszystko po staremu), a potem
działa; ekran pokazuje krok, w którym jest, i mówi, że nie da się go zatrzymać.
Wynik mówi, że stara historia zniknęła z tego klona, albo ile starych obiektów
nadal w nim jest. Pod nim są kroki do wykonania na każdym zdalnym (link można
skopiować). Krzyżyk, `Esc` i kliknięcie poza oknem zamykają ekran, z wyjątkiem
czasu, gdy purge trwa.

Przez MCP narzędzie purge nie istnieje, dopóki nie ustawisz
`allow_history_purge = true` w tabeli `[mcp]` pliku `config.toml`.

## Reguła Skrypt

*Dodaj regułę > Skrypt* (albo **Przepisanie: Uruchom skrypt...** w palecie) dodaje
regułę skryptu Lua i otwiera jej edytor nad warsztatem; `Enter` na linii *Skrypt*
reguły otwiera go ponownie. Edytor startuje z małego szablonu.

- **Język** (wiersz panelu reguły): *Lua (wbudowane)*, *Skrypt powłoki*, *Python*
  lub *Lua (zainstalowane)*. Wszystko poza wbudowanym Lua to
  [skrypt zewnętrzny](#skrypty-zewnętrzne-shell-python-lua): bez piaskownicy,
  a język, który jest wyłączony w ustawieniach albo którego interpretera brakuje,
  jest na liście wyszarzony z powodem obok (wybranie go mówi, co zrobić). Zmiana
  języka skryptu, którego jeszcze nie napisałeś, podmienia szablon.
- **Skrypt** to edytor tekstu z numerami linii. To, co wpisujesz, od razu jest
  regułą; niczego nie trzeba potwierdzać.
- Pod nim linia **kontroli** mówi `OK: znaleziono rewrite_commit` albo wymienia
  problemy z numerem linii (`script:3: ...`; numer jest czerwony na marginesie).
  Uruchamia się chwilę po przerwaniu pisania, nigdy przy wciśniętym klawiszu.
- **Parametry** mają jeden wiersz na każdą linię `--! param` w nagłówku skryptu,
  z etykietą obok pola: tekst, liczba albo pole prawda/fałsz. Wartość, którą
  wyczyścisz, wraca do domyślnej z nagłówka; parametr tekstowy bez wartości
  domyślnej trzeba uzupełnić przed podglądem.
- **Plik** (ścieżka wewnątrz repozytorium, na początek
  `.gitpoulpi/rewrite/script.lua`) z **Wczytaj z pliku...** (zastępuje tekst) i
  **Zapisz do pliku...**.
- **Testuj na zaznaczeniu** uruchamia skrypt na commitach zaznaczonych w Logu albo
  na 20 najnowszych, gdy żaden nie jest zaznaczony, i niczego nie zapisuje.
  Tabela wymienia każdy commit (krótkie id), każde pole, które by zmienił, jako
  `stare -> nowe`, oraz linie `log(...)` skryptu. **Anuluj** przestaje czekać.
  Jeśli skrypt się nie powiedzie, widać commit i powód.
- Krzyżyk, kliknięcie poza edytorem i `Esc` zamykają go; `Tab` i `Shift+Tab`
  przechodzą między edytorem, parametrami, plikiem, przyciskami i wynikami;
  `F5` testuje.

Podgląd odmawia dla skryptu, który się nie kompiluje: panel podglądu mówi
*skrypt zakończył się błędem* z linią (albo, przy błędzie na commicie, jego
krótkim id i powodem). W wersji bez skryptów rodzaj zostaje w menu Dodaj,
wyszarzony.

## Napisz skrypt z AI

Przy włączonym AI ([Asystent AI](ai.md)) edytor skryptu ma przycisk
`✦ Napisz z AI...`, a paleta ma `✦ Przepisanie: Napisz skrypt z AI...`, który
otwiera warsztat z nową regułą Skrypt i otwartym oknem. Nic nie jest wysyłane,
dopóki nie naciśniesz **Wyślij**: okno pokazuje, w trakcie pisania, dokładny
tekst, który wyjdzie.

- **Co ma robić skrypt?** Własnymi słowami: *przenieś każdy commit zrobiony nocą na
  następny ranek*, *zamień mój stary e-mail na nowy, tylko przed 2024*.
- **Wyślij próbkę z N commitów** (wyłączone): po zaznaczeniu autor, data i temat
  commitów, których użyłoby „Testuj na zaznaczeniu” (zaznaczenie w Logu, a jeśli
  go nie ma, 20 najnowszych), są dodawane, żeby model zobaczył, jak wyglądają
  twoje dane. Nigdy zawartość plików ani treść commitów. Przy wyłączonym polu
  żadne dane o commitach nie opuszczają komputera.
- **Co zostanie wysłane** to panel tylko do odczytu na nieco ciemniejszym tle, zwinięty po
  otwarciu okna (okno jest wtedy niskie). Kliknij jego nagłówek, naciśnij `Alt+W` albo przejdź
  do niego `Tab` i naciśnij `Enter`, by go pokazać: wymienia instrukcje (stały kontrakt skryptu,
  potem twoje instrukcje AI) i twoją prośbę, z własnym paskiem przewijania (przeciągnij go, użyj
  kółka nad panelem albo strzałek, `PageUp`, `PageDown`, `Home` i `End`, gdy ma fokus). Profil,
  który je dostanie, jest w wierszu powyżej. Okno rośnie, by objąć panel, w granicach ekranu.
- **Wyślij** (`Tab` do niego, `Enter`). Gdy model pracuje, okno czeka, a `Esc`
  anuluje zapytanie. Jeśli się nie powiedzie, okno zostaje z powodem, abyś mógł
  spróbować ponownie.

Odpowiedź przechodzi tę samą kontrolę demona co skrypt wpisany ręcznie; błąd
kompilacji wraca do modelu jeden raz. Skrypt **zastępuje potem tekst w edytorze**
(okno o tym mówi, gdy edytor zawiera coś innego niż szablon), a tytuł edytora
brzmi *✦ napisane przez AI: przeczytaj przed podglądem*, dopóki nie zmienisz
tekstu. Nigdy nie jest testowany, oglądany w podglądzie ani stosowany za ciebie:
przeczytaj go, naciśnij **Testuj na zaznaczeniu**, potem podgląd jak dla każdego
skryptu. Skrypt działa w tej samej piaskownicy co ten, który napisałeś (bez
plików, zegara i sieci).

Z wiersza poleceń: `gitpoulpi ai rewrite-script "<instruction>" [--sample N]`
wypisuje skrypt i wynik `rewrite script check` dla niego jako JSON i niczego nie
uruchamia (kod wyjścia 2, gdy nadal się nie kompiluje). Używany jest profil
*Skrypt przepisania historii* z Ustawień > AI.

## Utwórz reguły z AI

Przy włączonym AI ([Asystent AI](ai.md)) warsztat ma przycisk `✦ Utwórz reguły z AI...`
obok Zapisz i Wczytaj, klawisz `Alt+I` (*Utwórz reguły przepisania z AI* w Ustawieniach >
Klawiatura i mysz, kontekst *Warsztat przepisywania historii*), a paleta ma `✦ Przepisanie: Utwórz reguły z AI...`.
Przy wyłączonym AI albo w kompilacji bez AI przycisku i wiersza palety nie ma. Opisujesz
własnymi słowami, jak przepisać historię (*zamień mój stary e-mail a@b.org na c@d.org*,
*usuń secrets.txt z całej historii*, *przesuń moje daty o dwie godziny*), a model proponuje
reguły i ustawienia, które **wypełniają warsztat**. Nic nie jest wysyłane, dopóki nie
naciśniesz **Wyślij**: okno pokazuje, w trakcie pisania, dokładny tekst, który wyjdzie.

- **Jak przepisać historię?** Pole wielowierszowe: `Enter` to nowa linia,
  `Ctrl+Enter` lub `Ctrl+S` wysyła, `Esc` i krzyżyk anulują.
- **Wyślij zakres, nazwy gałęzi i tagów oraz autorów historii** (wyłączone): po
  zaznaczeniu dodawane są zakres z paska zakresu, nazwy gałęzi i tagów oraz różni autorzy
  (nazwa i e-mail) najnowszych commitów pokazanych w Logu (najwyżej 20 commitów), aby model
  mógł dokładnie podać twój e-mail (polecenie wiersza poleceń `ai rewrite-rules --context`
  czyta zamiast tego 200 najnowszych commitów). Nigdy zawartość plików ani opisy commitów. Przy
  wyłączonym polu komputer opuszcza tylko twój opis i stałe instrukcje.
- **Zastąp obecne reguły** (tylko gdy warsztat ma już reguły; wyłączone): domyślnie nowe
  reguły są dodawane po tych, które masz; po zaznaczeniu je zastępują (zakres i ustawienia
  podane w odpowiedzi są stosowane w obu przypadkach).
- **Co zostanie wysłane** to panel tylko do odczytu na nieco ciemniejszym tle, oddzielony od pól
  nad nim i **zwinięty** po otwarciu okna (okno jest wtedy niskie). Kliknij jego nagłówek,
  naciśnij `Alt+W` albo przejdź do niego `Tab` i naciśnij `Enter`, by go pokazać: wymienia
  instrukcje (słownictwo reguł, potem twoje instrukcje AI) i twoją prośbę oraz ma własny pasek
  przewijania (przeciągnij go, użyj kółka nad panelem albo strzałek, `PageUp`, `PageDown`,
  `Home` i `End`, gdy ma fokus). Profil, który je dostanie, jest w wierszu powyżej. Okno rośnie,
  by objąć panel, w granicach ekranu. Pole opisu dostaje własny pasek przewijania, gdy tekst
  przerośnie pole.

Model może użyć każdego rodzaju reguły z tej strony z wyjątkiem **Skryptu** (to kod:
*Napisz skrypt z AI* pokazuje skrypt do przejrzenia) i formy *Znajdź i zamień w plikach*
z wartościami tajnymi (wartości tajne nigdy nie trafiają do modelu: użyj karty Sekrety).
Może ustawić zakres na bieżącą gałąź, wszystkie lokalne gałęzie i tagi albo wskazane
istniejące gałęzie i tagi, a także opcje *Tagi*, *committer*, *puste commity* i *skrócone
id w opisach*. Tego, czego warsztat nie potrafi wyrazić (limitu dat w rodzaju *od marca*,
zakresu commitów), model nie zgaduje: mówi o tym w uwadze. O tym, co się stanie
z podpisami, model nigdy nie decyduje: pytanie padnie przy Zastosuj.

**AI tylko przygotowuje reguły i ustawienia.** Nigdy niczego nie podgląda, nie stosuje
ani nie uruchamia: odpowiedź jest kopiowana do warsztatu jak wczytany przepis, podgląd na
ekranie przestaje być aktualny, a Podgląd i Zastosuj (z ich pytaniami i potwierdzeniami)
należą do ciebie, jak zawsze. Każda reguła jest osobno parsowana i sprawdzana tą samą
kontrolą, której używają edytor i silnik; reguła, która jej nie przejdzie, jest pomijana.
Gdy coś pominięto albo model zrobił uwagę, po wstawieniu reguł pokazuje się lista
(*Pominięte (poprawne reguły zachowano)*, każdy wpis z pozycją, nazwą i powodem), a `Enter`
lub `Esc` ją zamyka. Przeczytaj reguły, popraw je, a potem użyj przycisku Podgląd.

Z wiersza poleceń: `gitpoulpi ai rewrite-rules "<description>" [--context]` wypisuje
propozycję jako JSON (`recipe`, który czyta `rewrite preview --recipe-json`, `problems`,
`notes` i `applied: false`) i niczego nie uruchamia (kod wyjścia 2, gdy coś pominięto).
Narzędzie MCP to `ai_rewrite_rules`. Używany jest profil *Reguły przepisania historii*
z Ustawień > AI.

## Skrypty Lua

Reguła skryptu wywołuje twoją funkcję na każdym commicie. To zwykły Lua 5.4
w piaskownicy: bez plików, zegara, procesów, modułów i `pcall`; ten sam skrypt na
tej samej historii zawsze daje ten sam wynik, a wywołanie, które przekroczy
10 milionów instrukcji, 64 MiB lub 5 sekund, kończy się błędem wskazującym commit.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` zawiera `id`, `parents`, `tree`, `author` i `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` oraz `signed`. `ctx` zawiera `index`, `total`,
`params`, `state` (zwróć `state = {...}`, aby przekazać dane do następnego
commita) i `window.before` / `window.after` (ustaw `--! window: N`). Zwracaj tylko
to, co się zmienia: `author`, `committer`, `message`. Funkcje pomocnicze:
`regex_replace`, `regex_match`, `stable_hash`, `rand`, `weekday`, `hour`, `log`, a dla
opisów `trailers(message)` (lista `{key, value}`), `set_trailer(message, key, value)`
(zastępuje trailery o tym kluczu; czwarty argument `"add"` lub `"skip"` to zmienia),
`remove_trailer(message, key)` oraz `at("2024-03-05 18:30", offset_minutes)` (data w
danym przesunięciu jako czas Unix).

Z wiersza poleceń: `gitpoulpi rewrite script check my.lua` kompiluje skrypt,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` pokazuje,
co by zmienił, niczego nie zapisując, a
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` planuje
całe przepisanie.

## Skrypty zewnętrzne (shell, Python, Lua)

Ten sam kontrakt co skrypt Lua, ale uruchamiany przez program na twoim komputerze:
*Skrypt powłoki* (bash lub sh; PowerShell w Windows albo dowolny `.ps1`),
*Python* i *Lua (zainstalowane)* (twój własny `lua`, nie wbudowany; nie ma
biblioteki JSON, więc szablon czyta linię wzorcem). **Nie działają w piaskownicy:
skrypt zewnętrzny działa z twoimi uprawnieniami** i może czytać lub zmieniać
wszystko, co ty. Mówi o tym panel reguły i tytuł edytora.

Jak się porozumiewa: GitPoulpi uruchamia skrypt raz na przebieg i wysyła mu
jedno żądanie JSON na linię na standardowe wejście (pola commita, `params`,
`state` zwrócony przez ostatnią odpowiedź); skrypt odpowiada jedną linią JSON na
standardowe wyjście, z tymi samymi polami, które zwraca skrypt Lua (`author`,
`committer`, `message`, `state`, `log`), albo `{"error": "text"}`, aby się
poddać. Diagnostykę wypisuj na standardowe wyjście błędów: pojawia się jako linie
`log` commita. Linie nagłówka zaczynają się od `#!` dla powłoki i Pythona
(`#! name: ...`, `#! param old: text "Old"`; linia shebang pozostaje shebangiem)
oraz od `--!` dla Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Co dostaje.** Puste środowisko poza `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` i `LANG`; świeży folder tymczasowy jako folder roboczy
(z własną kopią skryptu); nie ścieżkę repozytorium. **Limity czasu:**
`rewrite.script_timeout_ms` (5000) na każdą odpowiedź i
`rewrite.script_total_timeout_s` (3600) na cały przebieg. Gdy limit zostanie
przekroczony, skrypt się wysypie, wypisze coś, co nie jest JSON, albo zadanie
zostanie anulowane, GitPoulpi zabija skrypt i wszystko, co uruchomił, a przepisanie
zatrzymuje się z commitem i powodem; nic się nie przesunęło.

**Jakie języki istnieją.** Wymienia je `rewrite.external_scripts` w
`config.toml`: domyślnie `["shell"]`; `"python"` i `"lua"` dodaj sam w
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]`; listy nie
da się ustawić przez `gitpoulpi config set`). Język, którego nie ma na liście,
jest w warsztacie wyszarzony, a CLI i MCP go odmawiają (`scripts_disabled`);
język z listy, którego interpretera nie znaleziono, jest wyszarzony ze wskazówką
instalacji (`tool_missing`).

**Zaufanie.** Nawet gdy język jest włączony, skrypt działa dopiero po tym, jak
zaufasz jego dokładnemu tekstowi. Skrypt wpisany w warsztacie jest twój i działa
od razu; ten, który przychodzi ze wspólnym przepisem albo plikiem repozytorium,
nie: Podgląd i *Testuj na zaznaczeniu* zatrzymują się i pytają *Zaufać temu
skryptowi?* z dwiema ponumerowanymi odpowiedziami i żadną zaznaczoną domyślnie
(`1` zaufaj i kontynuuj, `2` nie, `Esc` zostawia). Zaufanie zapamiętuje SHA-256
tekstu dla tego repozytorium, więc zmieniony skrypt pyta ponownie, a sklonowane
repozytorium nie może samo uruchomić u ciebie kodu. Z wiersza poleceń przeczytaj
plik, potem `gitpoulpi rewrite script trust FILE` (albo `--trust` przy `preview`,
`run`, `script check` i `script test`, co ufa i idzie dalej). Agent przez MCP nie
może ufać niczemu: uruchomi tylko skrypt, któremu wcześniej zaufałeś.

**Kontrola.** Linia kontroli pod edytorem mówi *Skrypt nie jest jeszcze zaufany*
dla skryptu bez zaufania (nic nie zostało uruchomione); dla zaufanego GitPoulpi
uruchamia go raz z wymyślonym commitem (`"dry_run": true`) i sprawdza, czy
odpowiedź spełnia kontrakt. `gitpoulpi rewrite script check FILE` robi to samo
i zgłasza `trust`, `hash` i `interpreter`.

## Z wiersza poleceń

`gitpoulpi rewrite` (i narzędzia MCP o tych samych nazwach) uruchamia ten sam
silnik, dla skryptów i agentów. **Przepis** to plik TOML lub JSON, taki, jaki
zapisuje warsztat (`--recipe FILE` albo `-` dla standardowego wejścia), a flagi
dodają reguły po regułach z samego przepisu.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plan, niczego nie zmienia
gitpoulpi rewrite apply --run ID --digest D           # wykonuje plan, który obejrzałeś
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # przywraca wszystko
```

- Reguły jako flagi: `--set-name` / `--set-email` z `--match-email` (tożsamość),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. Zakresem jest bieżąca gałąź, `--all-local` albo `--ref R`
  (można powtarzać), od `--since REV`.
- **Pytania bez wartości domyślnej** to flagi: `--signatures
  strip|resign-signed|resign-all`, gdy zmieniają się podpisane commity, oraz
  `--extract-mode` dla wyodrębnienia. Bez nich odpowiedzią jest odmowa albo
  `needs_choice` (kod wyjścia 3), nigdy zgadywanie.
- **Kody wyjścia:** 0 gotowe; 1 błąd (nieczytelny przepis, złe flagi); 2 odmowa z
  powodem (`stale`, `signature_choice_needed`, `dirty_paths_touched`,
  `empty_scope`, ...); 3 pytanie do rozstrzygnięcia albo wypchnięte refy, które
  wymagają `--yes`.
- `preview` trzyma plan w repozytorium, więc `apply` działa z późniejszego
  polecenia, po restarcie demona albo z innego frontendu; plan, który nie zgadza
  się już ze swoim skrótem, jest `stale`.

[Opis CLI](cli-reference.md) zawiera każdą flagę, a
[MCP i agenci](mcp-and-agents.md) mówi, jakie narzędzia dostaje agent (narzędzie
purge nie istnieje, dopóki go nie włączysz, a agent nie może zaufać skryptowi).

## Propozycje od agentów AI

Agent może **zaproponować** przepisanie, nigdy go nie uruchomi. `gitpoulpi ui
propose rewrite --recipe FILE|-` (narzędzie MCP `ui_propose_rewrite`, wyłączone,
dopóki nie ustawisz `[mcp] allow_ui_propose = true`) otwiera warsztat w twoim
działającym oknie z regułami przepisu już na liście i fioletowym banerem *✦
Propozycja od <agent>: przepisanie historii (3 reguły) · Ctrl+Enter akceptuje ·
Esc odrzuca*.

- **Nic się nie uruchamia.** Żadnego podglądu, zastosowania, pusha ani purge:
  propozycja tylko wypełnia reguły. Czytasz je, w razie potrzeby edytujesz,
  robisz **Podgląd** i stosujesz własnym klawiszem. Zamknięcie warsztatu odrzuca
  propozycję.
- **Czego agent nie może w niej umieścić:** skryptu (uruchomiłby się przy
  podglądzie), więcej niż 50 reguł, wartości tajnych (są usuwane z przepisu, zanim
  zostanie wysłany). Taka propozycja jest odrzucana z podaniem powodu.
- Agent może odczytać wynik w `ui state` (`accepted`, gdy zastosowałeś reguły tak,
  jak zaproponowano, `accepted_edited`, gdy najpierw je zmieniłeś, `rejected`, gdy
  zamknąłeś warsztat). Naraz jedna propozycja; jest odrzucana, gdy otwarte jest
  inne twoje okno dialogowe.

## Narzędzia

Większość przepisywania historii nie potrzebuje niczego poza Gitem: zmiana
autorów i committerów, zastosowanie `.mailmap`, anonimizacja, przesuwanie dat lub
ograniczanie ich do godzin, przepisywanie opisów i trailerów, zamiana zawartości,
usuwanie ścieżek lub dużych blobów i przenoszenie ścieżek działają wewnątrz
GitPoulpi.

**Skrypty Lua są wbudowane.** Interpreter jest częścią GitPoulpi, więc skrypt Lua
nie wymaga żadnego narzędzia ani konfiguracji. Skrypty powłoki, Pythona i
zainstalowanego Lua potrzebują swojego interpretera (zob. tabelę niżej) i są
opisane w części [Skrypty zewnętrzne](#skrypty-zewnętrzne-shell-python-lua).

Kilka funkcji używa programu zewnętrznego. GitPoulpi sprawdza jego obecność
(program jest szukany w `PATH`, a w Windows także przez `PATHEXT` i własny
`usr/bin` Git dla Windows, i musi odpowiedzieć na `--version` w ciągu 2 sekund);
odpowiedź jest zapamiętywana na minutę, a „Sprawdź ponownie” pyta jeszcze raz.
Reguła, która potrzebuje brakującego programu, zostaje na liście, wyszarzona,
z powodem i wskazówką instalacji, a Podgląd odmawia, zanim cokolwiek się uruchomi.

| Funkcja | Narzędzie | Uwagi |
|---|---|---|
| Ponowne podpisywanie commitów | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) lub `gpgsm` (`gpg.format = x509`) | Podpisywanie SSH i X.509 wymaga też `user.signingkey`. Uwzględniane są `gpg.program`, `gpg.ssh.program` i `gpg.x509.program`. |
| Migracja do LFS (obie reguły LFS) | `git-lfs` 3.0 lub nowszy | To samo sprawdzenie co w panelu LFS. |
| Skrypty Python | `python` 3.8 lub nowszy (`py -3` w Windows) | Domyślnie wyłączone: dodaj `"python"` do `rewrite.external_scripts`. |
| Skrypty Lua (zainstalowane) | `lua5.4`, `lua54`, `lua` lub `luajit` | Domyślnie wyłączone: dodaj `"lua"` do `rewrite.external_scripts`. |
| Skrypty powłoki | `bash`, `pwsh` (PowerShell) lub `sh` | Domyślnie włączone (`"shell"`). Skrypt PowerShell (plik `.ps1`, shebang `pwsh` albo dowolny skrypt powłoki w Windows) wymaga `pwsh` lub `powershell`; skrypt POSIX wymaga `bash` lub `sh` (w Windows dostarcza je Git dla Windows). |

Wskazówki instalacji, najpierw Windows:

- Windows: Git dla Windows zawiera `gpg`, `ssh-keygen`, `bash`, `sh` i Git LFS.
  W przeciwnym razie `winget install GnuPG.GnuPG`, Ustawienia > Funkcje
  opcjonalne > Klient OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: pakiety dystrybucji (`gnupg`, `openssh`, `git-lfs`, `python3`,
  `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` i `sh` są
  wbudowane.
