[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · **Polski** · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS („Large File Storage”) trzyma duże pliki (pliki projektowe, wideo, zbiory danych) poza historią repozytorium: git przechowuje zamiast nich trzywierszowy tekstowy *wskaźnik* (pointer), a prawdziwa zawartość leży na serwerze LFS i w lokalnej pamięci podręcznej w `.git/lfs`.

GitPoulpi nie potrzebuje do tego żadnego konta. LFS używa poświadczeń, których już używa twoje zdalne repozytorium (menedżer poświadczeń gita lub ssh-agent).

## Co wymaga programu `git-lfs`, a co nie

GitPoulpi uruchamia standardowy program `git-lfs` do wszystkiego, co konwertuje lub przenosi pliki (install, track, fetch, pull, push, blokady).

Odczyt *stanu* LFS nie wymaga programu: czy repozytorium używa LFS, które pliki są wskaźnikami, jak są duże i czy prawdziwy obiekt jest już zapisany lokalnie.

| | wymaga `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (zgłasza brak programu zamiast kończyć się błędem) | nie |
| `gitpoulpi lfs files` | nie |
| install / track / untrack / prune | tak |
| fetch / pull / push | tak |
| blokady | tak |

## Instalacja `git-lfs`

- **Windows**: jest dołączony do Git for Windows; albo `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Cała reszta: https://git-lfs.com

Wymagana jest wersja 3.0 lub nowsza. Po instalacji GitPoulpi zauważa program w ciągu minuty (albo natychmiast przy `lfs status`).

## Gdy brakuje `git-lfs`

GitPoulpi nigdy się przez to nie zawiesza ani nie kończy działania:

- `gitpoulpi lfs status` kończy się powodzeniem i wypisuje `git-lfs: not installed` wraz z linkiem do instalacji.
- Operacja, która wymaga programu, kończy się wyraźnym komunikatem, na końcu którego jest link do instalacji (`https://git-lfs.com`), i niezerowym kodem wyjścia; przez MCP narzędzie zwraca ten komunikat.
- Repozytorium używające LFS nadal się otwiera; jego duże pliki po prostu widać jako wskaźniki, dopóki program nie zostanie zainstalowany.

## Podgląd plików LFS

```
gitpoulpi lfs status            # program, wzorce, filtry, hook
gitpoulpi lfs files             # pliki wskaźnikowe w indeksie, z rozmiarem i stanem lokalny/nie pobrano
gitpoulpi lfs files --at v1.0   # ... według rewizji
gitpoulpi show big.bin --at HEAD  # zgłasza „Git LFS object, 12.3 MB, sha256:...” zamiast tekstu wskaźnika
```

## Konfigurowanie repozytorium: install, track, untrack

```
gitpoulpi lfs install                 # filtry + hooki dla TEGO repozytorium (git lfs install --local)
gitpoulpi lfs install --user          # zamiast tego do globalnej konfiguracji git (nigdy po cichu)
gitpoulpi lfs track '*.psd'           # kieruje *.psd przez LFS
gitpoulpi lfs track '*.psd' --lockable   # ... i robi je tylko do odczytu do czasu zablokowania
gitpoulpi lfs track '*.bin' --renormalize  # ... i ponownie dodaje do poczekalni już zatwierdzone *.bin jako wskaźniki
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # usuwa stare, już wypchnięte obiekty z lokalnego magazynu
```

- `install` domyślnie zapisuje do `.git/config` i hooków samego repozytorium, więc twoja globalna konfiguracja gita nigdy nie jest ruszana, chyba że podasz `--user`.
- `track` i `untrack` edytują `.gitattributes` i zostawiają go **bez stage**: pojawia się jako zwykła zmiana w drzewie roboczym, którą zatwierdzasz samodzielnie. Pliki dodane *po* rozpoczęciu śledzenia są zapisywane jako wskaźniki przy swoim następnym commicie.
- Pliki **już zatwierdzone** jako zwykła zawartość pozostają zwykłe w historii. `--renormalize` ponownie dodaje je do poczekalni, tak że twój następny commit zastąpi je wskaźnikami. Aby przenieść je także we wszystkich dawnych commitach, użyj reguły przepisywania historii *Migruj pliki do LFS* (zob. [Przepisywanie historii](history-rewrite.md#przenoszenie-plików-do-git-lfs-i-z-niego)); to własna reguła GitPoulpi, nie `git lfs migrate`.
- Ujmij wzorzec w cudzysłów, żeby powłoka go nie rozwijała.
- Przez MCP są to zwykłe mutacje (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), podlegające bramkowaniu `[mcp]` w `config.toml`.

## Przenoszenie obiektów: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # pobiera obiekty do lokalnego magazynu; drzewo robocze nietknięte
gitpoulpi lfs pull  [--remote origin]           # fetch, potem zastępuje pliki wskaźnikowe prawdziwą zawartością
gitpoulpi lfs push  [--remote origin] [--all]   # wysyła obiekty bieżącej gałęzi (albo każdego refa z --all)
```

- Domyślnie używane jest domyślne zdalne repozytorium, tak jak przy `fetch` i `push`.
- Używane są zwykłe poświadczenia twojego zdalnego repozytorium (menedżer poświadczeń, ssh-agent). GitPoulpi nigdy nie prosi o własne konto.
- `pull` instaluje filtry LFS dla tego repozytorium, jeśli ich brakuje: bez nich git-lfs pobiera obiekty, ale pomija zastępowanie wskaźników, a późniejszy checkout przywróciłby wskaźniki.
- MCP traktuje `lfs push` jak `push`: jako narzędzie destrukcyjne (zmienia zdalne repozytorium).
- Okno pokazuje postęp transferu (pliki i bajty) na pasku stanu. CLI czeka na wynik.

### Wypełnianie plików zawartością: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: zapisuje prawdziwą zawartość nad plikami wskaźnikowymi, z lokalnego magazynu
```

Nigdy niczego nie pobiera (to robi `lfs pull`); obiekty, których brakuje w lokalnym magazynie, pozostają wskaźnikami. Tak jak `pull`, najpierw instaluje filtry LFS repozytorium, jeśli żadne nie są skonfigurowane. Jak każdej operacji LFS, nie da się jej cofnąć; zob. [Cofnij i ponów](undo-redo.md). Przez MCP jest to `lfs_checkout` (zapis, nie destrukcyjny).

### Przycisk LFS na pasku narzędzi oraz pull / push

Pasek narzędzi ma przycisk **`LFS ▾`** po **Pop** (ukryty, gdy `git.lfs_enabled` jest wyłączone, oraz w repozytorium, w którym LFS nie jest ani zainicjowany, ani używany przez jego `.gitattributes`: zob. [Ustawienia > LFS](settings.md#ustawienia-repozytorium-lfs)). Kliknięcie otwiera listę: **Fetch wszystkich plików LFS** (`lfs fetch --all`), **Checkout wszystkich plików LFS** (`lfs checkout`), **Pull wszystkich plików LFS** (`lfs pull`), **Push wszystkich plików LFS** (`lfs push --all`, obiekty każdego refa) i **Wyczyść lokalny magazyn LFS** (`lfs prune`). Każdy wiersz jest też poleceniem palety (`LFS: Fetch wszystkich obiektów`, `LFS: Checkout wszystkich plików`, `LFS: Pobierz obiekty (pull)`, `LFS: Push wszystkich obiektów`, `LFS: Wyczyść lokalne obiekty`). `LFS: Otwórz menu` otwiera listę. Każde ma akord `Ctrl+G`: `l` dla menu, potem `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` i `Shift+V` dla wierszy. Można je zmienić w *Ustawienia > Klawiatura i mysz*. Gdy brakuje `git-lfs`, każdy wiersz prowadzi do okna instalacji.

![Lista LFS na pasku narzędzi: fetch, checkout, pull i push wszystkich plików LFS oraz czyszczenie lokalnego magazynu](../images/lfs-menu.png)

**Pull i push obsługują LFS same.** W repozytorium używającym LFS, przy zainstalowanym `git-lfs`, zwykły `pull` najpierw sprawdza, czy filtry LFS repozytorium są zainstalowane, a zwykły `push` sprawdza jego hook `pre-push`. Gdy któregoś brakuje, GitPoulpi uruchamia `git lfs install --local` (tylko to repozytorium) przed transferem, a pasek stanu mówi „filtry LFS zainstalowane”. Bez hooka push nie wysłałby żadnych obiektów LFS. Wyłączysz to przez `git.lfs_autofix_filters = false` (*Ustawienia > Git i kodowanie > Instaluj filtry LFS przed pull i push*). Zwykły `push` nadal wysyła tylko obiekty wypychanej gałęzi, nigdy wszystkich refów; prune nigdy nie jest niejawny.

## Blokady plików

Niektórych plików nie da się scalać (plik Photoshopa, poziom gry, zasób binarny). *Blokady* Git LFS pozwalają jednej osobie zasygnalizować, że edytuje plik: nikt inny nie może go zablokować ani wypchnąć w nim zmian, dopóki nie zostanie odblokowany.

```
gitpoulpi lfs locks  [--remote origin]              # kto trzyma który plik (twoje są oznaczone)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Blokowanie jest funkcją **serwera LFS** (mają ją GitHub, GitLab i Gitea). Dostęp do niej odbywa się za pomocą własnych poświadczeń zdalnego repozytorium; GitPoulpi nie ma własnego konta.
- Serwer bez blokowania powoduje, że `lock` kończy się komunikatem, który to mówi (typowany `LfsLockingUnsupported`). `locks` na takim serwerze nic nie wypisuje, co zgłasza też sam git-lfs.
- Śledzenie wzorca z `--lockable` (`lfs track '*.psd' --lockable`) sprawia, że te pliki są tylko do odczytu, dopóki ich nie zablokujesz.
- `unlock` cudzej blokady jest odrzucany; `--force` ją łamie, dlatego MCP traktuje `lfs_unlock` jako narzędzie destrukcyjne. `lfs_locks` jest tylko do odczytu.

## Klonowanie repozytorium LFS

`gitpoulpi clone <url>` (i okno Klonuj w TUI) wykonuje checkout repozytorium *bez* uruchamiania filtrów LFS, a potem, gdy repozytorium używa LFS i `git-lfs` jest zainstalowany, pobiera obiekty przez `git lfs pull` (instalując filtry dla tego repozytorium, jeśli żadne nie są skonfigurowane) i pokazuje to jako osobną fazę, *Pobieranie obiektów LFS*. `--no-lfs` pomija ten krok i zostawia wskaźniki.

Klonowanie **nigdy nie kończy się błędem z powodu LFS**: gdy brakuje `git-lfs` albo pobieranie się nie powiedzie, klonowanie nadal się udaje, a duże pliki zostają wskaźnikami; stan LFS repozytorium mówi dlaczego, a `gitpoulpi lfs pull` dokończy zadanie po naprawieniu problemu.

## W interfejsie terminala

- **Znacznik `LFS`.** Plik zapisany przez Git LFS ma mały znacznik `LFS` po nazwie na listach Unstaged i Staged panelu Status oraz na liście zmienionych plików commita. **Tylko LFS** (`o` przy aktywnym panelu Status lub Logu, pole wyboru na pasku narzędzi list plików albo **Pliki: Przełącz tylko LFS** w palecie) zawęża te listy do plików LFS. Pole wyboru leży na prawym końcu wiersza filtra (panel Status: na pasku narzędzi albo w jego menu przepełnienia, gdy jest wąsko); na liście plików commita w wąskim panelu kurczy się do `[ ] LFS` zamiast znikać. Pokazuje się tylko wtedy, gdy wyświetlane pliki jakieś mają. W panelu Status i wśród zmienionych plików commita plik liczy się jako LFS, gdy jego ścieżka jest wskaźnikiem w indeksie (jedno zbiorcze wywołanie `git` na odświeżenie, nic dla repozytorium bez atrybutów `filter=lfs`). Przy włączonym **Wszystkie pliki** w Logu wyświetlane jest całe drzewo wybranej rewizji, a plikami LFS są pliki tej rewizji, odczytane z jej własnych blobów (z pamięcią podręczną per rewizja, poza wątkiem renderowania, bez programu `git-lfs`), więc plik, który jest tam LFS, ale nie w indeksie, dostaje znacznik i zostaje po filtrze.
- **Karta diffa.** Otwarcie diffa pliku LFS pokazuje kartę zamiast tekstu wskaźnika: `Obiekt Git LFS`, `1.2 MB -> 1.4 MB`, krótkie stare i nowe oid oraz informację, czy nowy obiekt jest zapisany lokalnie (`zapisany lokalnie`), czy `nie pobrano`. Naciśnij `r`, by zobaczyć surowy diff wskaźnika, i `r` ponownie, by wrócić. Do pliku LFS stosuje się tylko stage na poziomie pliku (nie ma hunków ani linii do dodania). Pod kartą **Otwórz w domyślnym** i **Otwórz w** (`o`, `Shift+O` lub paleta) otwierają prawdziwy plik, nigdy wskaźnik: zob. [Otwórz w](open-in.md#pliki-git-lfs).
- **Widok pliku** pliku LFS mówi `Git LFS object, 12.3 MB, sha256:ab12cd34...` i gdzie zdobyć zawartość.
- **Baner.** Dla repozytorium, które używa LFS, gdy `git-lfs` brakuje albo jest starszy niż 3.0, informuje o tym baner w panelu Status. Kliknij go (lub użyj pozycji palety *LFS: Otwórz stronę instalacji*), by otworzyć https://git-lfs.com. Nic poza tym się nie zmienia: okno się otwiera, diffy działają, duże pliki po prostu widać jako wskaźniki.
- **Menu pliku i paleta.** Kliknij prawym przyciskiem zmieniony plik w panelu Status: *Śledź przez LFS: ten plik* / *to rozszerzenie* dla zwykłego pliku; *Zablokuj plik (LFS)*, *Odblokuj plik (LFS)* lub *Wymuś odblokowanie (LFS, cudza blokada)* dla pliku LFS, zależnie od tego, kto trzyma jego blokadę. Wiersz zablokowanego pliku mówi `zablokowane przez ciebie` / `zablokowane przez <name>`. Wszystko jest też poleceniem palety (`Ctrl+P`, wpisz `LFS`): inicjalizacja, śledzenie lub zaprzestanie śledzenia wzorca, fetch, pull lub push obiektów, prune, pokazanie blokad, otwarcie strony instalacji.
- **Nieudana akcja sama się tłumaczy.** Jeśli akcja LFS kończy się błędem, bo brakuje `git-lfs`, okno dialogowe niesie komunikat i przycisk *Otwórz stronę instalacji*.
- **Ustawienia.** *Ustawienia > LFS* (dla repozytorium) pokazuje, czy LFS jest skonfigurowany, wylicza śledzone wzorce i ma przycisk **Skonfiguruj LFS tutaj**. *Git i kodowanie* (dla repozytorium) ma przełącznik **Git LFS**, wiersz tylko do odczytu **Program git-lfs** i przycisk **Zainicjuj LFS dla tego repozytorium**.
- **Git LFS** wyłączony (`git.lfs_enabled = false`, zob. [Opis konfiguracji](config-reference.md)) ukrywa to wszystko. Własne filtry LFS gita nadal działają.
- Stan LFS jest odczytywany krótko po otwarciu repozytorium, po każdej zmianie, przy ręcznym odświeżeniu, gdy zmienia się `.gitattributes`, i co dziesięć sekund dla repozytorium używającego LFS.

## Znane ograniczenia

- Repozytorium, którego wzorce LFS leżą tylko w *zagnieżdżonym* `.gitattributes` (nie w głównym ani w `.git/info/attributes`), jest wykrywane jako używające LFS dopiero wtedy, gdy ma magazyn obiektów LFS (`.git/lfs`), nie wcześniej.
- Samo `git lfs migrate` nie jest używane. Przenoszenie plików do LFS lub z niego w całej historii robią reguły przepisywania historii *Migruj pliki do LFS* i *Przywróć pliki z LFS*: działają na lokalnym klonie, zapisywany jest tylko główny `.gitattributes`, a obiekty trzeba potem jeszcze wypchnąć (*Wyślij obiekty LFS* w panelu wyniku albo `lfs push --all`).
