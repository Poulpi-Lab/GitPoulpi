[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · **Polski** · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalacja

GitPoulpi to jeden program, `gitpoulpi`. Uruchomiony bez podpolecenia otwiera interfejs terminalowy; z podpoleceniem jest interfejsem wiersza poleceń (`--json` dla skryptów); `gitpoulpi mcp serve` to serwer MCP dla agentów AI (zob. [MCP i agenci](mcp-and-agents.md)). Obok niego archiwum zawiera `gitpoulpi-updater`, osobny mały program, który aktualizuje instalację (zob. [Aktualizacje](updates.md)).

Dla zgodności archiwum zawiera też trzy małe przekaźniki, które tylko uruchamiają `gitpoulpi`: `gitpoulpi-tui` (otwiera interfejs; zostanie usunięty w jednym z kolejnych wydań), `gitpoulpi-mcp` (uruchamia serwer MCP; zostaje przez całą linię 1.x, więc istniejące konfiguracje agentów nadal działają) i `gitpoulpi-launch` (to uruchamiają skróty, by pokazać już otwarte okno GitPoulpi, `gitpoulpi launch`; w Windows nie ma okna konsoli, więc kliknięcie nigdy żadnego nie błyska). Wszystko jest instalowane, aktualizowane i usuwane razem.

## Czego potrzeba

- **Systemowego `git`** w `PATH`. GitPoulpi używa libgit2 do większości
  operacji, a systemowego `git` do reszty.
  Wymagana jest wersja **2.34 lub nowsza**. W macOS działa `git` z Apple Command Line Tools, a także ten z Homebrew.
- **Terminala** z obsługą myszy. kitty, Ghostty, WezTerm, Rio, iTerm2 (oraz Windows Terminal, foot i Tabby przez sixel) rysują graf
  commitów jako obrazy. Pozostałe terminale (konsola Windows, zellij, screen
  i większość innych) pokazują graf znakowy. Zob.
  [Wybór terminala](terminals.md) i [Multipleksery](multiplexers.md).
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
pliku `.sha256`, rozpakuj je i umieść jego pliki wykonywalne
(`gitpoulpi`, `gitpoulpi-updater` oraz przekaźniki `gitpoulpi-tui`, `gitpoulpi-mcp` i `gitpoulpi-launch`) w folderze
znajdującym się w `PATH` (zostaw `gitpoulpi-updater` i znacznik `.gitpoulpi-dist`
obok nich: znacznik mówi `channel=archive`, więc ta kompilacja może sama się
aktualizować, zob. [Aktualizacje](updates.md)). `INSTALL.txt` w archiwum podaje te same kroki.

### Weryfikacja pobranego pliku

Każdy plik wydania jest podpisany kluczem [minisign](https://jedisct1.github.io/minisign/) projektu, a GitPoulpi sam sprawdza ten podpis, zanim zainstaluje aktualizację. Dla pliku pobranego ręcznie podpisem jest `.minisig` obok niego. Klucz publiczny (id klucza `B461FFCBF6F6A4F2`) jest w repozytorium jako `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Przeczytaj zaufany komentarz wypisany przez polecenie: musi mieć postać `file:<nazwa pliku> version:<oczekiwana wersja>`. Podpis, który jest poprawny, ale wskazuje inny plik lub inną wersję, nie jest podpisem twojego pliku.

Jeśli wydanie zbudowano z poświadczeniami pochodzenia kompilacji (zmienna `ATTESTATIONS` repozytorium), GitHub CLI sprawdza, gdzie plik został zbudowany:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Plik `.sha256` wychwytuje tylko uszkodzone pobranie. Dopiero podpis pokazuje, że plik pochodzi od projektu.

## Z instalatorem

Każde wydanie zawiera też instalator dla każdego systemu. Oba podejmują te same decyzje
(skróty, PATH, alias polecenia, taki jak `gp`), wywołując zainstalowane
`gitpoulpi`, czyli ten sam kod, który stoi za Ustawienia > Uruchamianie z systemu, więc to, co
instalator konfiguruje, i to, co pokazują Ustawienia, zawsze się zgadza.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Pierwsze okno pyta, dla kogo jest instalacja:

| Wybór | Gdzie | Uprawnienia administratora | Co to znaczy dla aktualizacji |
|---|---|---|---|
| **Tylko dla mnie** (domyślnie) | `%LOCALAPPDATA%\Programs\GitPoulpi` | brak | GitPoulpi zastępuje własne pliki: **Zaktualizuj i uruchom ponownie** w [panelu aktualizacji](updates.md), bez pytania. |
| **Dla wszystkich użytkowników** | `Program Files\GitPoulpi` | jedno pytanie przy instalacji | Każda aktualizacja prosi o zatwierdzenie przez administratora (pytanie Windows), a sesje GitPoulpi innych użytkowników na komputerze są najpierw zamykane. |

Ten sam wybór działa po cichu: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` lub `/ALLUSERS`.

Pola wyboru: skrót w menu Start, skrót na pulpicie, dodanie do PATH. Jedna strona pyta o **alias polecenia**: domyślnie `gp`, dowolna nazwa albo puste, jeśli aliasu nie chcesz (`/ALIAS=name` lub `/ALIAS=` w wierszu poleceń). PowerShell używa już `gp` dla `Get-ItemProperty`, więc wybierz tam inną nazwę albo używaj `gitpoulpi`.

Instalator kopiuje pliki wykonywalne (w tym `gitpoulpi-updater`, mały program podmieniający pliki podczas aktualizacji) i znacznik `.gitpoulpi-dist`. Znacznik zapisuje, jak zainstalowano GitPoulpi (`inno-user` lub `inno-machine`), co panel aktualizacji pokazuje jako **Sposób instalacji**. Ponowne uruchomienie instalatora zachowuje twoje wybory (zakres, skróty, alias).

Odinstalowanie z Ustawień systemu Windows > Aplikacje uruchamia `gitpoulpi uninstall` (alias, wpisy w PATH, skróty) i usuwa pliki, także pozostałości `.old` po aktualizatorze. Twoje ustawienia i repozytoria zostają.

### Linux i macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Wybiera archiwum dla twojego systemu i procesora, sprawdza jego `.sha256`, instaluje (kanał `archive`: GitPoulpi aktualizuje się sam na miejscu) w
`~/.local/opt/gitpoulpi` i dowiązuje `gitpoulpi` (który uruchomiony bez podpolecenia otwiera interfejs terminalowy) oraz `gitpoulpi-mcp` (przekaźnik) w
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

## Z menedżerem pakietów

Każdy z poniższych kanałów zapisuje obok programów ten sam znacznik `.gitpoulpi-dist`, z własną nazwą kanału. GitPoulpi czyta go, by wiedzieć, kto zarządza aktualizacjami: menedżer pakietów, więc GitPoulpi nigdy nie podmienia plików za jego plecami. Panel aktualizacji (kliknij wersję na pasku tytułu) uruchamia wtedy własne polecenie menedżera pakietów albo je pokazuje, byś mógł je skopiować. Zob. [Aktualizacje](updates.md).

| Kanał | Instalacja | Uwagi |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Instaluje dla wszystkich użytkowników, więc aktualizacje proszą o zatwierdzenie przez administratora. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi`, potem `scoop install gitpoulpi` | Dla użytkownika, bez uprawnień administratora. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tap to `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | zob. [repozytorium apt](#apt-debian-i-ubuntu) | Podpisane repozytorium; `.deb` jest też dołączany do każdego wydania. |
| **MSI** (Windows, zarządzane) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Dla administratorów: wszyscy użytkownicy, bez aktualizacji w aplikacji. Zob. [Dystrybucja w przedsiębiorstwie](enterprise-distribution.md#msi-z-intune-lub-zasadami-grupy). |
| **cargo** | `cargo binstall gitpoulpi-cli` | Pobiera archiwum wydania zamiast kompilować. |
| **cargo, z crates.io** | `cargo install --locked gitpoulpi-cli` | Kompiluje; wymaga toolchaina Rust. Instaluje `gitpoulpi`, który uruchomiony bez podpolecenia otwiera interfejs terminalowy i obsługuje też MCP (`gitpoulpi mcp`). |
| **cargo, ze źródeł** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Kompiluje; wymaga toolchaina Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Zob. [Uruchamianie w Dockerze](docker.md). |

Powyższe nazwy to te, do których publikuje proces wydawniczy (zob. [Wydawanie](../../developer/releasing.md)): kanał, który jeszcze nie działa, odpowiada „not found”, a archiwum wydania lub instalator zawsze działają.

Obowiązuje własna obsługa PATH menedżera pakietów: Chocolatey, Scoop i Homebrew same dodają programy do PATH, a żaden z nich nie tworzy aliasu `gp` (uruchom `gitpoulpi alias create gp` albo Ustawienia > Uruchamianie z systemu).

### apt (Debian i Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Repozytorium obsługuje `amd64` i `arm64` i jest podpisane kluczem należącym do projektu. Programy trafiają do `/usr/lib/gitpoulpi` z dowiązaniami w `/usr/bin`. Pakiet zależy od `git` i zaleca `git-lfs`. Bez repozytorium zainstaluj `.deb` wydania przez `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi powie wtedy, byś sam pobrał następny `.deb`, bo oferuje polecenia `apt` tylko wtedy, gdy znajdzie repozytorium w `/etc/apt/sources.list.d/gitpoulpi.*`.

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

Skrypty `build-opti` w katalogu głównym repozytorium wykonują to budowanie
i wypisują rozmiary plików wykonywalnych:

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
systemy). Archiwum zawiera pliki wykonywalne, `INSTALL.txt`, `README.md`,
`LICENSE` i `CHANGELOG.md`. Obok zapisywany jest plik `.sha256`.

## Aktualizacje

Pasek tytułu pokazuje działającą wersję (`v1.0.0`) między nazwą aplikacji a ścieżką repozytorium. Kliknięcie jej otwiera panel aktualizacji tylko wtedy, gdy istnieje nowsze wydanie albo trwa aktualizacja; w przeciwnym razie otwiera kartę **Informacje o wersji**. Panel otwierają też `Ctrl+N`, a potem `u`, oraz **Aktualizacje: Pokaż aktualizację** z palety poleceń (`Ctrl+P`). Panel pokazuje wersję, sposobem instalacji GitPoulpi i tym, co zrobić z nowszym wydaniem. **Pomoc: Informacje o wersji** otwiera dziennik zmian.

Raz dziennie, w tle, GitPoulpi sprawdza miejsce, w którym publikowane są jego wydania (domyślnie GitHub), czy jest nowsze (`updates.mode`, [Ustawienia > Synchronizacja](settings.md#synchronizacja)). Gdy tak, znaczek czyta `v0.2.0 dostępna` w kolorze akcentu.

To, co robi panel, zależy od sposobu instalacji GitPoulpi: archiwum wydania lub instalator Windows zastępuje własne pliki, menedżer pakietów uruchamia własne polecenie, instalacja zarządzana lub kontenerowa mówi, jak docierają do niej aktualizacje. Kompilacja zrobiona przez `cargo build` nigdy się nie aktualizuje. Pełny opis, z powrotem do poprzedniej wersji, trybem `auto` wobec `notify` i tym, co może zablokować administrator, jest w [Aktualizacjach](updates.md).

## Sprawdzenie, czy działa

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` wypisuje wersję i commit, z którego zbudowano program. Potem przejdź
do [Pierwszych kroków](getting-started.md).

## Odinstalowanie

`gitpoulpi uninstall` (to uruchamiają oba instalatory) usuwa aliasy poleceń,
wpisy w PATH, skróty i wpisy menu menedżera plików (w Windows razem z kluczem
App Paths) utworzone przez GitPoulpi, a potem pliki programu
instalacji z wydania; odmawia dla kompilacji bez znacznika `.gitpoulpi-dist`
(kompilacja przez cargo), a `--keep-files` zostawia pliki programu. Nie jest
dostępne dla agentów MCP. Bez instalatora usuń pliki wykonywalne.
GitPoulpi zostawia też te pliki:

| Co | Gdzie |
|---|---|
| Ustawienia (`config.toml`) i sesje | Folder konfiguracji systemu. Zmienia go `GITPOULPI_CONFIG_DIR`. |
| Stan poszczególnych repozytoriów | `.gitpoulpi/` wewnątrz każdego repozytorium. |
| Pamięć podręczna (awatary) | Folder pamięci podręcznej systemu. |
| Tokeny i klucze API | Menedżer poświadczeń. Usuwa je **Ustawienia > SSH i poświadczenia > Zapomnij wszystkie nazwy użytkowników i hasła**. |

Dokładne ścieżki podaje [Opis konfiguracji](config-reference.md).
