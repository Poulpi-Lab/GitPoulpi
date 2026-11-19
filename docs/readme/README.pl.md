[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · **Polski** · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="Ośmiornica GitPoulpi" width="128"></p>

**Klient Git dla terminala, który działa jak aplikacja desktopowa.** GitPoulpi
daje pełny graf commitów, po którym można klikać, przeciągać i używać prawego
przycisku myszy, paletę poleceń sterowaną z klawiatury, która sięga każdej
funkcji, integrację z hostingami kodu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; w chmurze lub self-managed) i trackerami zgłoszeń (Jira, Trello, Linear, YouTrack, Redmine albo folderem notatek Markdown) oraz te same operacje jako skryptowalne CLI
i serwer MCP, dzięki czemu agent AI może nim sterować dokładnie tak jak Ty.
GitPoulpi jest inspirowany GitKraken i napisany od nowa w Rust: natywne pliki binarne, lekkie na dysku i w pamięci, bez Electrona, bez konta i bez telemetrii. Działa w Twoim ulubionym terminalu ([w którym?](../../docs/user/pl/terminals.md)), bo niektórzy z nas w nim mieszkają.

![GitPoulpi z grafem commitów z gałęziami, tagami i stashem, panelem Refs po lewej oraz zmianami roboczymi z formularzem commita po prawej](../../docs/user/images/hero.png)

- **Mysz i klawiatura na równi.** Klikaj, klikaj dwukrotnie, używaj prawego
  przycisku i przeciągaj refy na commity; albo naciśnij `Ctrl+P` i wpisz, co
  chcesz zrobić. Każdy gest myszy ma odpowiednik na klawiaturze, a każde
  polecenie jest w palecie.
- **Prawdziwy graf commitów.** Gałęzie, merge'e, tagi, stashe i kopia robocza
  w jednym przewijanym Logu, rysowanym pikselami w terminalach, które to
  umożliwiają, a znakami wszędzie indziej. Wiele ciemnych i jasnych motywów
  kolorystycznych, awatary autorów, wyszukiwanie.
- **Precyzyjny stage.** Dodawaj do stage, usuwaj ze stage lub odrzucaj plik,
  folder, hunk albo pojedynczą linię prosto z diffa. W linii lub obok siebie,
  z minimapą.
- **Bezpieczna edycja historii.** Zmieniaj opis dowolnego commita lub poprawiaj go
  (opis, autor, daty), rób interaktywny rebase z ekranem przeglądu, przepisuj
  wiele commitów naraz z podglądem i jednym cofnięciem, cherry-pickuj kilka
  commitów, przewijaj fast-forward, twórz worktree i patche.
- **Wbudowane hostingi i trackery zgłoszeń.** Przeglądaj, wykonuj checkout i twórz pull i merge requesty (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) oraz recenzuj zmiany w Gerrit. Przeglądaj i twórz zgłoszenia, work items i karty (także Jira, Trello, Linear, YouTrack, Redmine i lokalne notatki Markdown), w razie potrzeby na tablicy Kanban. Klonuj ze swojego konta i zarządzaj kluczami SSH.
- **CLI i MCP dla agentów.** `gitpoulpi <polecenie> --json` obejmuje wszystko,
  co robi interfejs, a `gitpoulpi-mcp` udostępnia to jako narzędzia MCP. Jeden
  demon w tle na repozytorium utrzymuje każdy interfejs aktualny i zsynchronizowany.
- **Opcjonalne AI, nigdy wymagane.** Domyślnie wyłączone i bez konta: użyj
  własnego klucza API albo lokalnego agenta CLI, takiego jak `claude -p`.
  Generuj opisy commitów, wyjaśniaj zmiany, pisz opisy pull requestów, dziel
  zmiany robocze na kilka commitów i przygotowuj reguły przepisania historii.
- **Git LFS, submoduły, worktree, patche, blame i historia pliku, diffy
  obrazów, wbudowany edytor.**

## Zrzuty ekranu

| | |
|---|---|
| ![Graf Log z motywem kolorów Material i włączonym zabarwieniem wierszy](../../docs/user/images/theme-material.png) **Graf commitów**, tu w motywie Material z zabarwieniem wierszy. | ![Diff zmodyfikowanego pliku z przyciskami Stage hunk i Odrzuć hunk przy każdym hunku](../../docs/user/images/diff.png) **Diff**, ze stage i odrzucaniem dla hunka lub pojedynczej linii. |
| ![Paleta poleceń przefiltrowana słowem stash](../../docs/user/images/palette.png) **Paleta poleceń** (`Ctrl+P`) sięga każdego polecenia. | ![Kompozytor commitów proponujący cztery commity dla zmian roboczych, gotowe do przeglądu](../../docs/user/images/composer.png) **Kompozytor commitów** pozwala przejrzeć commity zaproponowane przez AI, zanim cokolwiek zostanie zatwierdzone. |
| ![Menu prawego przycisku commita z akcjami cherry-pick, revert, reset, rebase, gałąź i tag](../../docs/user/images/context-menu.png) **Prawy przycisk** na czymkolwiek pokazuje jego akcje. | ![Strona AI w Ustawieniach z profilami, instrukcjami dla poszczególnych akcji i przełącznikiem włączenia](../../docs/user/images/settings-ai.png) **Ustawienia**, z wyszukiwaniem, także ze stroną opcjonalnego AI. |

Więcej obrazków, z objaśnieniami, jest w [dokumentacji użytkownika](../../docs/user/pl/README.md).

## Instalacja

Potrzebny jest systemowy `git`; `git-lfs` jest opcjonalny.

### Linux i macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Instalator dla Windows

Pobierz `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` z
[najnowszego wydania](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)
i uruchom go. Dla administratorów dostępny jest MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Zob. [Uruchamianie w Dockerze](../user/pl/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Archiwum wydania

Pobierz je z
[najnowszego wydania](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
rozpakuj i postępuj według `INSTALL.txt` w środku.

### Ze źródeł

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Własne archiwum wydania, mniejsza wersja na zamówienie i uwagi dla poszczególnych systemów:
[docs/user/pl/installation.md](../user/pl/installation.md) oraz
[docs/user/pl/custom-builds.md](../user/pl/custom-builds.md).

## Szybki start

```
cd path/to/your/repository
gitpoulpi
```

- `?` otwiera pomoc z każdym klawiszem i gestem, `Ctrl+P` paletę poleceń,
  `Ctrl+,` Ustawienia, a `q` kończy program.
- Kliknij commit, aby zobaczyć jego zmiany; kliknij górny wiersz Logu (zmiany
  robocze), aby dodać pliki do stage i zrobić commit.
- W terminalu, który to umożliwia (kitty, Ghostty, WezTerm, iTerm2 (oraz Windows Terminal i foot przez sixel)), graf jest
  rysowany pikselami. W pozostałych (zwykła konsola Windows, tmux)
  automatycznie przechodzi na znaki.

Wersja krok po kroku jest w [Pierwszych krokach](../../docs/user/pl/getting-started.md),
a [przewodnik po interfejsie](../../docs/user/pl/interface-tour.md) nazywa każdy
obszar ekranu.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI jest opcjonalne i domyślnie wyłączone: włącz własne AI GitPoulpi w
*Ustawienia > AI* ([Wsparcie AI](../../docs/user/pl/ai.md)); pozwól zewnętrznemu
agentowi sterować GitPoulpi za pomocą [MCP i agenci](../../docs/user/pl/mcp-and-agents.md)
(uprawnienia, każde narzędzie) oraz skilla agenta
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Dokumentacja

Pełna dokumentacja użytkownika zaczyna się od **[docs/user/pl/README.md](../../docs/user/pl/README.md)**:

| | |
|---|---|
| [Pierwsze kroki](../../docs/user/pl/getting-started.md) | Instalacja, otwarcie repozytorium, pierwszy commit |
| [Przewodnik po interfejsie](../../docs/user/pl/interface-tour.md) | Każdy obszar okna z nazwą |
| [Praca z Logiem](../../docs/user/pl/log-and-history.md) | Graf, refy, gałęzie, tagi, stashe, rebase, cherry-pick |
| [Stage i commity](../../docs/user/pl/staging-and-committing.md) | Status, diff, hunki i linie, kompozytor, edytor |
| [Ustawienia](../../docs/user/pl/settings.md) | Każda strona i opcja Ustawień |
| [Wsparcie AI](../../docs/user/pl/ai.md) | Konfiguracja wbudowanego AI, każda akcja AI, prywatność |
| [Hosting i trackery zgłoszeń](../../docs/user/pl/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokeny, powiązanie repozytorium, pull i merge requesty |
| [CLI](../../docs/user/pl/cli-reference.md) i [MCP](../../docs/user/pl/mcp-and-agents.md) | Skryptuj je albo pozwól agentowi nimi sterować (uprawnienia, każde narzędzie) |
| [Skróty klawiszowe](../../docs/user/pl/keybindings.md) (i każde polecenie palety), [mysz](../../docs/user/pl/mouse-and-gestures.md), [konfiguracja](../../docs/user/pl/config-reference.md), [LFS](../../docs/user/pl/lfs.md), [cofanie](../../docs/user/pl/undo-redo.md), [rozwiązywanie problemów](../../docs/user/pl/troubleshooting.md) | Dokumentacja referencyjna i pomoc |

Deweloperzy: [CLAUDE.md](../../CLAUDE.md) i `docs/architecture/overview.md` opisują
crate'y oraz projekt; [CHANGELOG.md](../../CHANGELOG.md) wymienia zawartość każdego
wydania.

## Licencja

[MIT](../../LICENSE).

## Podziękowania i licencje stron trzecich

GitPoulpi opiera się na bibliotekach open source i używa wartości kolorów z opublikowanych motywów. Ich licencje i podziękowania, wraz z autorami motywów, znajdują się w [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). Plik jest dołączany do każdego wydania; pokazuje go także okno O programie oraz `gitpoulpi licenses`.

## Znaki towarowe

GitKraken jest znakiem towarowym swojego właściciela. GitPoulpi jest niezależnym projektem, niepowiązanym z tym właścicielem, niezatwierdzonym i niesponsorowanym przez niego; nazwa jest wymieniona wyłącznie w celu opisania zgodności i inspiracji.
