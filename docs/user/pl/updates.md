[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · **Polski** · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Aktualizacje

GitPoulpi sprawdza, czy są nowsze wydania, i zależnie od sposobu instalacji albo zastępuje własne pliki, albo przekazuje aktualizację programowi, który go zainstalował. Nic nie jest zastępowane ani uruchamiane ponownie bez twojego działania.

## Znaczek i panel

Pasek tytułu pokazuje działającą wersję (`v1.0.0`) między nazwą aplikacji a ścieżką repozytorium. Gdy znane jest nowsze wydanie, znaczek zmienia się w `v0.2.0 dostępna` w kolorze akcentu.

Kliknięcie znaczka otwiera panel aktualizacji tylko wtedy, gdy istnieje nowsze wydanie albo trwa aktualizacja; w przeciwnym razie otwiera kartę **Informacje o wersji**. Panel otwierają też `Ctrl+N`, a potem `u`, oraz **Aktualizacje: Pokaż aktualizację** z palety poleceń (`Ctrl+P`). Panel to okno nad aplikacją: zamyka je `Esc`, `q`, krzyżyk zamknięcia lub kliknięcie poza nim. Pokazuje:

- działającą wersję i **Sposób instalacji**: jak GitPoulpi trafił na ten komputer (od tego zależy wszystko poniżej);
- stan aktualizacji (sprawdzanie, pobieranie z procentami, gotowa, nieudana);
- przyciski procedury tej instalacji oraz dokładne polecenie, gdy jest jakieś;
- wiersz od twojego administratora, jeśli go ustawił.

Każdy przycisk ma klawisz (wypisany w wierszu podpowiedzi panelu) i polecenie palety, więc panel działa bez myszy:

| Przycisk | Klawisz | Polecenie palety |
|---|---|---|
| **Zaktualizuj i uruchom ponownie** (albo **Uruchom aktualizację**) | `u` | Aktualizacje: Zaktualizuj teraz |
| **Kopiuj polecenie** | `c` | Aktualizacje: Kopiuj polecenie aktualizacji |
| **Otwórz w terminalu** | `t` | Aktualizacje: Uruchom polecenie aktualizacji w terminalu |
| **Wróć do wersji ...** | `v` | Aktualizacje: Wróć do poprzedniej wersji |
| **Informacje o wersji** | `n` | Pomoc: Informacje o wersji |
| **Pomiń tę wersję** | `s` | Aktualizacje: Pomiń tę wersję |
| **Otwórz komunikat** (tylko przy komunikacie o bezpieczeństwie) | `a` | Aktualizacje: Otwórz komunikat o bezpieczeństwie |
| **Sprawdź teraz** | `r` | Aktualizacje: Sprawdź aktualizacje teraz |
| **Zamknij** | `Esc` | |

Polecenie, które nie dotyczy tej instalacji, jest w palecie wyszarzone z podaną przyczyną (na przykład „ta instalacja jest aktualizowana w inny sposób: zob. Aktualizacje: Pokaż aktualizację”).

## Co się dzieje, zależnie od instalacji

| Sposób instalacji | Sprawdzane w | Co robi panel | Powrót |
|---|---|---|---|
| Archiwum wydania lub `install.sh` | Wydania GitHub | **Zaktualizuj i uruchom ponownie**: GitPoulpi pobiera i weryfikuje archiwum, zamyka się, podmienia swoje pliki i otwiera się ponownie w tym samym terminalu. | **Wróć do wersji ...** przywraca poprzednie pliki. |
| Instalator Windows, tylko dla mnie | Wydania GitHub | To samo, bez pytania. W Windows otwiera się ponownie w nowym oknie. | **Wróć do wersji ...** przywraca. |
| Instalator Windows, dla wszystkich użytkowników | Wydania GitHub | To samo, ale Windows prosi o zatwierdzenie przez administratora. Sesje GitPoulpi innych użytkowników na komputerze są zamykane, co traci ich niezapisane dane wejściowe. Jeśli odmówisz, nic się nie zmienia. | **Wróć do wersji ...** przywraca, z tym samym pytaniem. |
| Chocolatey | Publiczny feed Chocolatey | **Uruchom aktualizację** uruchamia `choco upgrade gitpoulpi -y` (Windows prosi o zatwierdzenie przez administratora). | Panel pokazuje `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, wewnętrzny feed | Feed twojej organizacji | **Uruchom aktualizację** uruchamia `choco upgrade gitpoulpi -y --source=<feed>`. Parametry pakietu nie są wysyłane ponownie, więc wdrożona konfiguracja zostaje nietknięta. | Ten sam wiersz `choco install` z feedem. |
| Scoop | Wydania GitHub | **Uruchom aktualizację** uruchamia `scoop update gitpoulpi` po całkowitym zamknięciu GitPoulpi. | **Wróć do wersji ...** uruchamia `scoop reset gitpoulpi@<previous>`. |
| Homebrew | Wydania GitHub | **Uruchom aktualizację** uruchamia `brew upgrade gitpoulpi` w tym terminalu. | Panel pokazuje `brew install gitpoulpi@<previous>`. |
| apt | Wydania GitHub | Panel pokazuje `sudo apt update && sudo apt install --only-upgrade gitpoulpi` z przyciskami **Kopiuj polecenie** i **Otwórz w terminalu**. GitPoulpi nigdy nie uruchamia za ciebie `sudo`. Bez repozytorium apt GitPoulpi na tym komputerze panel mówi, by pobrać zamiast tego nowy plik `.deb`. | Panel pokazuje `sudo apt install gitpoulpi=<previous>`. |
| Pakiet MSI | Wydania GitHub lub źródło twojego administratora | Nie ma czego uruchamiać: instalacją zarządza twój administrator, a panel pokazuje jego komunikat. | Niedostępny. |
| cargo | Wydania GitHub | Panel pokazuje `cargo binstall gitpoulpi-cli`, gdy zainstalowano `cargo-binstall`, w przeciwnym razie `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | To samo polecenie z poprzednim tagiem. |
| Obraz Docker | Wydania GitHub | Nie ma czego uruchamiać: obraz się wymienia. Panel pokazuje `docker pull poulpilab/gitpoulpi:<version>` oraz `:latest`. Zob. [Uruchamianie w Dockerze](docker.md). | Panel pokazuje `docker pull poulpilab/gitpoulpi:<previous>`. |
| Archiwum wydania w folderze, do którego nie możesz pisać | Wydania GitHub | Panel mówi, że GitPoulpi nie może się tam zastąpić, i wskazuje **Pobierz**. | Niedostępny. |
| Kompilacja deweloperska (`cargo build`) | brak | Nigdy się nie aktualizuje. Znaczek pokazuje wersję i nic więcej; panel mówi, że nie ma zapisu instalacji. | Niedostępny. |

Wiersze „pokazuje” nigdy same niczego nie uruchamiają. **Otwórz w terminalu** otwiera twój terminal z wpisanym poleceniem, a ty naciskasz Enter.

Wydania zawsze pochodzą z repozytorium projektu na GitHubie, chyba że administrator skieruje sprawdzanie gdzie indziej (wewnętrzny feed, lustro lub `latest.json`), zob. [Dystrybucja w przedsiębiorstwie](enterprise-distribution.md). Informacje o wersji pochodzą z wydania albo z dziennika zmian wbudowanego w GitPoulpi, zależnie od tego, co jest osiągalne.

## Powiadamianie i tryb automatyczny

`updates.mode` ([Ustawienia > Synchronizacja](settings.md#synchronizacja)):

- **Powiadamiaj** (domyślnie): raz dziennie GitPoulpi sprawdza w tle i pokazuje znaczek. Nic nie jest pobierane, dopóki nie naciśniesz przycisku.
- **Automatycznie**: tam, gdzie GitPoulpi zastępuje własne pliki (archiwum wydania, instalator Windows), także pobiera i weryfikuje aktualizację w tle, gdy tylko ją znajdzie. Znaczek czyta wtedy **Uruchom ponownie, by zaktualizować**, a ponowne uruchomienie nadal należy do ciebie. Tam, gdzie aktualizacją zarządza menedżer pakietów, **Automatycznie** działa jak **Powiadamiaj**.
- **Wył.**: nigdy nie wysyła żadnego zapytania, a znaczek pokazuje tylko wersję.

Ustawienie **Kanał aktualizacji** (`updates.channel`) wybiera między **Stabilny** a **Wstępny** (także wersje kandydujące). Sprawdzenie uruchomione przez **Sprawdź teraz** ignoruje limit raz na dobę. Bez połączenia z siecią albo gdy repozytorium jest prywatne, a token github.com nie jest zapisany, sprawdzenie w tle cicho się nie udaje, zostawiając jeden wiersz w dzienniku aplikacji.

## Ponowne uruchomienie i co widać potem

Naciśnięcie **Zaktualizuj i uruchom ponownie** lub **Uruchom aktualizację** zapisuje sesję i zamyka okno. Pracę wykonuje osobny mały program, `gitpoulpi-updater`, więc własne pliki GitPoulpi nie są w użyciu, gdy są zastępowane, a menedżer pakietów nie jest blokowany przez czekającego rodzica. Najpierw zatrzymuje demony GitPoulpi działające w tle (zamykają się też inne okna i serwery MCP twojego użytkownika), wykonuje aktualizację, sprawdza, czy nowe `gitpoulpi --version` wypisuje oczekiwaną wersję, i uruchamia GitPoulpi ponownie: w tym samym terminalu w Linuksie i macOS, w nowym oknie w Windows. Jeśli sprawdzenie się nie powiedzie, przywraca stare pliki.

Następny start pokazuje jeden wiersz na pasku stanu:

- `Zaktualizowano do v1.4.2`
- `Aktualizacja się nie powiodła: <przyczyna>. Poprzednia wersja nadal jest zainstalowana.`
- `Aktualizacja anulowana: zatwierdzenie przez administratora zostało odrzucone. Nic się nie zmieniło.`

Naraz działa tylko jedna aktualizacja; plik blokady w folderze danych GitPoulpi (i drugi w folderze instalacji, gdy można do niego pisać) nie wpuszcza drugiej, a blokada starsza niż dziesięć minut jest ignorowana, więc przerwana aktualizacja nigdy nie zablokuje GitPoulpi na stałe. CLI i demon nie startują, dopóki istnieje świeża blokada.

## Powrót do poprzedniej wersji

Tam, gdzie GitPoulpi zastępuje własne pliki, **Wróć do wersji ...** przywraca poprzednie pliki i uruchamia ponownie. Zostają one jako `<name>.old` przez minutę po starcie nowej wersji, potem są usuwane, więc powrót służy pierwszemu uruchomieniu po aktualizacji. Tam, gdzie instalacją zarządza menedżer pakietów, powrót to polecenie z tabeli: panel je pokazuje lub uruchamia. Powrót zmienia własne pliki GitPoulpi, nie repozytorium, więc nie jest częścią dziennika cofania gita.

## Co jest sprawdzane i odrzucane

Zanim cokolwiek zostanie zastąpione, GitPoulpi pobiera archiwum, jego podpis `.minisig` i `.sha256`. Najpierw sprawdza podpis, kluczem wydań projektu wbudowanym w GitPoulpi, a zaufany komentarz podpisu musi wskazywać to archiwum i tę wersję. Potem sprawdza skrót. Odrzuca: wydanie bez `.minisig` (starsze wydanie opublikowane, zanim wydania były podpisywane, nie może być zainstalowane przez aktualizator, co dotyczy też przypinania i powrotu do niego), podpis innym kluczem, podpis innego pliku lub innej wersji, wydanie bez `.sha256`, skrót, który się nie zgadza, archiwum o nazwie innej niż dla tej platformy (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), archiwum, którego znacznik wskazuje inną wersję lub platformę, oraz link pobierania spoza hostów wydań GitHuba (sprawdzany jest każdy przeskok przekierowania) lub lustra skonfigurowanego przez administratora. Podmiana jest wszystko albo nic: jeśli inny program trzyma jeden z plików wykonywalnych, GitPoulpi wskazuje ten plik, niczego nie zmienia i zostawia pobranie do kolejnej próby. W Windows aktualizacja dla wszystkich użytkowników jest weryfikowana ponownie przez krok z uprawnieniami administratora, który sam pobiera `.minisig` i `.sha256` i sprawdza oba jeszcze raz, więc plik podmieniony w międzyczasie w folderze pobierania użytkownika jest odrzucany. Menedżery pakietów (Chocolatey, Scoop, Homebrew, apt) weryfikują własne pakiety: GitPoulpi niczego tam nie pobiera.

## Komunikaty o bezpieczeństwie

Obok każdego wydania projekt publikuje `security.json`: listę komunikatów o bezpieczeństwie (identyfikator, waga, dotknięte wersje jako zakres, np. `>=1.2.0, <1.4.3`, wersja z poprawką, link i jednozdaniowe streszczenie). To samo codzienne sprawdzenie, które szuka nowej wersji, pobiera też ten plik, z tego samego miejsca co wydanie: ze strony wydania repozytorium dla GitHuba, z folderu `latest.json` dla lustra, z GitHuba dla publicznego feedu Chocolatey. Komunikat jest używany tylko wtedy, gdy `security.json.minisig` weryfikuje się kluczem projektu; plik niepodpisany lub źle podpisany nigdy nie jest pokazywany (jeden wiersz w Dzienniku aktywności mówi, że został zignorowany).

Gdy komunikat wskazuje działającą wersję, znaczek robi się czerwony i czyta **Komunikat o bezpieczeństwie: ta wersja jest dotknięta**, w każdej instalacji, także zarządzanej przez menedżer pakietów. Panel aktualizacji wymienia wtedy każdy komunikat z wagą, streszczeniem, wersją z poprawką i linkiem; **Otwórz komunikat** (`a`, albo **Aktualizacje: Otwórz komunikat o bezpieczeństwie** w palecie) otwiera w przeglądarce link najpoważniejszego z nich. Tam, gdzie instalacją zarządza menedżer pakietów, panel dodaje „Zaktualizuj przez swojego menedżera pakietów.”

Przy wyłączonym sprawdzaniu aktualizacji (`updates.mode = "off"`), przy polityce administratora, w której `check` ma wartość `none`, oraz dla kompilacji zrobionej przez `cargo build` nic nie jest pobierane i żaden komunikat nie jest pokazywany. Własny feed NuGet organizacji nie sprawia, że GitPoulpi łączy się z GitHubem: jej komunikaty pochodzą tylko z lustra `download_base` z polityki.

## Ustawienia zablokowane przez administratora

Administrator może w polityce obowiązującej na całym komputerze ustawić tryb aktualizacji, kanał, przypiętą wersję, miejsce sprawdzania i to, czy GitPoulpi może w ogóle aktualizować się sam. W Ustawieniach te wiersze pokazują **Ustawione przez administratora**, panel pokazuje ich komunikat, a gdy samoaktualizacja jest wyłączona, panel oferuje polecenia lub notatkę zamiast przycisków zastępujących pliki. Nieczytelne pliki polityki kończą się odmową: tylko powiadamianie, bez samoaktualizacji. Strona administratora jest w [Dystrybucji w przedsiębiorstwie](enterprise-distribution.md#plik-polityki-i-rejestr).

## Środowisko

`GITPOULPI_UPDATE_API` kieruje sprawdzanie do innego głównego adresu API (używane w testach).
