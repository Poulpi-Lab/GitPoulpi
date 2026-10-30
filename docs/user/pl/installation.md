[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · **Polski** · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalacja

GitPoulpi ma trzy pliki wykonywalne:

| Plik wykonywalny | Co to jest |
|---|---|
| `gitpoulpi-tui` | Interfejs terminalowy. |
| `gitpoulpi` | Interfejs wiersza poleceń (`--json` dla skryptów). Bez podpolecenia otwiera interfejs. |
| `gitpoulpi-mcp` | Serwer MCP dla agentów AI. Zob. [MCP i agenci](mcp-and-agents.md). |

Obok nich archiwum zawiera `gitpoulpi-launch`, malutki program, który skróty uruchamiają, by pokazać już otwarte okno GitPoulpi (`gitpoulpi launch`). W Windows nie ma okna konsoli, więc kliknięcie nigdy żadnego nie błyska. Jest instalowany, aktualizowany i usuwany razem z pozostałymi.

## Czego potrzeba

- **Systemowego `git`** w `PATH`. GitPoulpi używa libgit2 do większości
  operacji, a systemowego `git` do reszty.
  Wymagana jest wersja **2.34 lub nowsza**. W macOS działa `git` z Apple Command Line Tools, a także ten z Homebrew.
- **Terminala** z obsługą myszy. kitty, Ghostty, WezTerm i iTerm2 (oraz Windows Terminal i foot przez sixel) rysują graf
  commitów jako obrazy. Pozostałe terminale (konsola Windows, tmux
  i większość innych) pokazują graf znakowy. Zob.
  [Wybór terminala](terminals.md).
- **Nerd Fonta** albo czcionki z emoji, dla ikon na pasku narzędzi i w panelu
  bocznym.
- Opcjonalnie: **`git-lfs`** w wersji 3.0 lub nowszej, dla repozytoriów Git
  LFS. Bez niego pliki LFS wyglądają jak małe pliki wskaźników, a GitPoulpi
  mówi, dlaczego. Jak go zainstalować, opisuje [Git LFS](lfs.md).
- Opcjonalnie: **menedżer poświadczeń** (credential helper: Git Credential Manager, libsecret,
  `pass`, Keychain), do przechowywania tokenów hostingów i kluczy API AI. Zob.
  [Hosting i trackery zgłoszeń](hosting-integration.md).

## Z archiwum wydania

Archiwum wydania to `gitpoulpi-<version>-<target>.zip` (Windows) lub
`.tar.gz` (Linux, macOS), z plikiem `.sha256` obok. Sprawdź archiwum względem
pliku `.sha256`, rozpakuj je i umieść trzy pliki wykonywalne w folderze
znajdującym się w `PATH`. `INSTALL.txt` w archiwum podaje te same kroki.

## Z instalatorem

Każde wydanie zawiera też instalator dla każdego systemu. Oba podejmują te same decyzje
(skróty, PATH, alias polecenia, taki jak `gp`), wywołując zainstalowane
`gitpoulpi`, czyli ten sam kod, który stoi za Ustawienia > Ogólne > Uruchamianie z systemu, więc to, co
instalator konfiguruje, i to, co pokazują Ustawienia, zawsze się zgadza.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Instaluje dla bieżącego użytkownika, bez pytania o uprawnienia administratora, w
`%LOCALAPPDATA%\Programs\GitPoulpi` (pierwsze okno proponuje instalację dla
wszystkich użytkowników i wybór folderu). Pola wyboru: skrót w menu Start, skrót na pulpicie,
dodanie do PATH. Jedna strona pyta o **alias polecenia**: domyślnie `gp`,
dowolna nazwa albo puste, jeśli aliasu nie chcesz. PowerShell używa już `gp` dla
`Get-ItemProperty`, więc wybierz tam inną nazwę albo używaj `gitpoulpi`.

Instalator kopiuje trzy pliki wykonywalne i znacznik `.gitpoulpi-dist`, więc
[Zaktualizuj teraz](#aktualizacja-na-miejscu) działa dokładnie tak jak przy archiwum zip. Instalacja dla jednego użytkownika
jest zapisywalna, więc może sama się aktualizować; instalacja dla wszystkich użytkowników w Program
Files wymaga podniesienia uprawnień do zastąpienia, więc zainstaluj ją ponownie nowym instalatorem.
Odinstalowanie z Ustawień systemu Windows > Aplikacje uruchamia `gitpoulpi uninstall` (alias,
wpisy w PATH, skróty) i usuwa pliki, także pozostałości `.old` po aktualizatorze.
Twoje ustawienia i repozytoria zostają.

### Linux i macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Wybiera archiwum dla twojego systemu i procesora, sprawdza jego `.sha256`, instaluje w
`~/.local/opt/gitpoulpi` i dowiązuje `gitpoulpi` oraz `gitpoulpi-mcp` w
`~/.local/bin`. Nie jest potrzebny root i nic poza twoim katalogiem domowym nie jest ruszane.
Bez `--yes` pyta o alias (domyślnie `gp`), PATH i skróty.

| Flaga | Działanie |
|---|---|
| `--prefix DIR` | Instaluje pliki programu w `DIR` (ścieżka bezwzględna). |
| `--version X.Y.Z` | Instaluje to wydanie zamiast najnowszego. |
| `--archive FILE` | Używa lokalnego archiwum; obok musi leżeć `FILE.sha256` (`--no-verify` to pomija). Sieć nie jest używana. |
| `--alias NAME` / `--no-alias` | Sprawia, że `NAME` uruchamia GitPoulpi (`gitpoulpi alias create`), albo brak aliasu. |
| `--no-path` | Nie dodaje GitPoulpi do PATH nowych powłok (w przeciwnym razie `gitpoulpi path add`). |
| `--desktop` | Tworzy też skróty w menu aplikacji i na pulpicie. |
| `--yes` | Nigdy nie pyta: obowiązują flagi i wartości domyślne (alias `gp`, PATH tak, skróty nie). |
| `--uninstall` | Uruchamia `gitpoulpi uninstall`, potem usuwa dowiązania i folder. |

Sprawdzenie najnowszego wydania wymaga, by repozytorium było publiczne; przy prywatnym
pobierz archiwum w przeglądarce i użyj `--archive`.

## Ze źródeł

Potrzebny jest aktualny stabilny toolchain Rust (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Pliki wykonywalne są w `target/release/`. Profil `release` buduje się szybko.
Najmniejsze i najszybsze pliki wykonywalne daje profil `dist`; buduje się
dłużej:

```
cargo build --profile dist
```

Skrypty `build-opti` w katalogu głównym repozytorium wykonują to budowanie dla
trzech plików wykonywalnych i wypisują ich rozmiary:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd lub PowerShell)
```

Aby zbudować tylko używane języki i funkcje, zob.
[Własne kompilacje](custom-builds.md).

## Własne archiwum wydania

```
cargo run -p xtask -- dist
```

Buduje profil `dist` i zapisuje
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) lub `.tar.gz` (inne
systemy). Archiwum zawiera trzy pliki wykonywalne, `INSTALL.txt`, `README.md`,
`LICENSE` i `CHANGELOG.md`. Obok zapisywany jest plik `.sha256`.

## Aktualizacje

Pasek tytułu pokazuje działającą wersję (`v1.0.0`) między nazwą aplikacji
a ścieżką repozytorium. Kliknij ją albo uruchom **Pomoc: Informacje o wersji** z palety poleceń (`Ctrl+P`): otwiera się karta Informacje o wersji z dziennikiem zmian, przewiniętym
do działającej wersji.

Raz dziennie, w tle, GitPoulpi pyta GitHub, czy istnieje nowsze wydanie
(`updates.mode`, [Ustawienia > Synchronizacja](settings.md#synchronizacja)).
Gdy tak, znaczek czyta `v0.2.0 dostępna` w kolorze akcentu, a jego
kliknięcie otwiera kartę Informacje o wersji z notatkami tego wydania na górze. Karta ma
trzy przyciski, każdy z klawiszem i poleceniem palety:

- **Pobierz** (`d`, *Aktualizacje: Pobierz*) otwiera stronę wydania w przeglądarce.
- **Pomiń tę wersję** (`s`, *Aktualizacje: Pomiń tę wersję*) ukrywa powiadomienie,
  dopóki nie pojawi się nowsze wydanie.
- **Sprawdź teraz** (`r`, *Aktualizacje: Sprawdź aktualizacje teraz*) pyta ponownie od razu.

`updates.mode = "off"` w ogóle nie wysyła zapytań. Bez połączenia z siecią
albo gdy repozytorium jest prywatne, a token github.com nie jest zapisany, sprawdzenie
cicho się nie udaje (jeden wiersz w dzienniku aplikacji, nic w interfejsie).
`GITPOULPI_UPDATE_API` kieruje sprawdzanie do innego głównego adresu API (używane w testach).

### Aktualizacja na miejscu

Kompilacja zainstalowana z archiwum wydania może sama się zastąpić. Archiwum zawiera
mały plik `.gitpoulpi-dist` obok plików wykonywalnych; działająca kompilacja jest
„aktualizowalna” tylko wtedy, gdy ten plik leży obok jej pliku wykonywalnego (i wskazuje jej
platformę). Kompilacja zrobiona przez `cargo`, własna kompilacja skopiowana gdzie indziej lub
pakiet, który zgubił ten plik, nigdy nie aktualizuje się sama: jej karta Informacje o wersji
mówi o tym i oferuje tylko **Pobierz**.

Dla aktualizowalnej kompilacji karta dodaje:

- **Zaktualizuj teraz** (`u`, *Aktualizacje: Zaktualizuj teraz*) pobiera archiwum i jego
  `.sha256` w tle (zadanie we wskaźniku na pasku stanu, z procentami, do anulowania jak każde
  zadanie), sprawdza skrót, rozpakowuje archiwum i kładzie każdy nowy plik obok starego jako
  `<name>.new`. Nic, co działa, nie jest ruszane. W trybie `auto` dzieje się to samo
  samoczynnie, gdy tylko sprawdzenie znajdzie nowsze wydanie.
- **Uruchom ponownie, by zaktualizować** (`R`, *Aktualizacje: Uruchom ponownie, by zaktualizować*; znaczek mówi to
  samo, gdy pobranie jest już przygotowane) jest zawsze twoją akcją, nigdy automatyczną. Zatrzymuje
  samodzielny demon sesji (wbudowany znika razem z oknem), podmienia pliki i uruchamia nową wersję z tymi samymi argumentami
  i sesją.
- **Wróć do poprzedniej wersji** (`v`, *Aktualizacje: Wróć do poprzedniej wersji*) przywraca
  poprzednie pliki i uruchamia ponownie. Pliki poprzedniej wersji zostają jako
  `<name>.old`, dopóki nowa wersja nie działa przez minutę, potem
  są usuwane, więc przycisk jest dostępny przy tym pierwszym uruchomieniu. Nie jest częścią
  historii cofania gita: zmienia własne pliki GitPoulpi, nie repozytorium.

Jak pliki są zastępowane: w Linuksie i macOS każdy nowy plik jest przemianowywany na miejsce
starego (atomowo; działający program trzyma starą kopię do zakończenia). W
Windows działającego `.exe` nie można nadpisać, więc stary jest przemianowywany na
`<name>.old`, a nowy wstawiany; `.old`, który nadal działa, jest usuwany
przy późniejszym starcie. Podmiana jest wszystko albo nic: jeśli inny program trzyma jeden z
plików wykonywalnych (na przykład klient MCP uruchamiający `gitpoulpi-mcp`), GitPoulpi wskazuje
ten plik, niczego nie zmienia i zostawia pobranie przygotowane do kolejnej próby.

Co jest odrzucane, każde z komunikatem: wydanie bez pliku `.sha256`, skrót,
który się nie zgadza, plik archiwum o nazwie innej niż dla tej platformy
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), archiwum, którego znacznik wskazuje
inną wersję lub platformę, oraz każdy link pobierania spoza tych hostów: GitHuba
`github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` i `github-releases.githubusercontent.com`
(pierwsze żądanie i każdy przeskok przekierowania; przekierowanie gdziekolwiek indziej zatrzymuje
pobieranie). Archiwa prywatnego repozytorium wymagają sesji w przeglądarce, więc
pobieranie tam się nie udaje, dopóki repozytorium nie stanie się publiczne.

## Sprawdzenie, czy działa

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` wypisuje wersję i commit, z którego zbudowano program. Potem przejdź
do [Pierwszych kroków](getting-started.md).

## Odinstalowanie

`gitpoulpi uninstall` (to uruchamiają oba instalatory) usuwa aliasy poleceń,
wpisy w PATH i skróty utworzone przez GitPoulpi, a potem pliki programu
instalacji z wydania; odmawia dla kompilacji bez znacznika `.gitpoulpi-dist`
(kompilacja przez cargo), a `--keep-files` zostawia pliki programu. Nie jest
dostępne dla agentów MCP. Bez instalatora usuń trzy pliki wykonywalne.
GitPoulpi zostawia też te pliki:

| Co | Gdzie |
|---|---|
| Ustawienia (`config.toml`) i sesje | Folder konfiguracji systemu. Zmienia go `GITPOULPI_CONFIG_DIR`. |
| Stan poszczególnych repozytoriów | `.gitpoulpi/` wewnątrz każdego repozytorium. |
| Pamięć podręczna (awatary) | Folder pamięci podręcznej systemu. |
| Tokeny i klucze API | Menedżer poświadczeń. Usuwa je **Ustawienia > SSH i poświadczenia > Zapomnij wszystkie nazwy użytkowników i hasła**. |

Dokładne ścieżki podaje [Opis konfiguracji](config-reference.md).
