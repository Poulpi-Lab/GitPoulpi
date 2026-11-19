[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · **Polski** · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Własne kompilacje

Dla tych, którzy chcą mniejszego pliku binarnego: interaktywny skrypt pyta, które
języki, integracje z serwisami hostingu, motywy, kolory logo i funkcje uwzględnić, a następnie buduje
tylko je, używając zoptymalizowanego profilu. Zwykła kompilacja i oficjalne wydanie
pozostają bez zmian (wszystko włączone).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd lub PowerShell (scripts/build-custom.ps1)
```

Wynik trafia do `target/custom/`, nigdy do oficjalnego `target/dist/`. Skrypt
wypisuje rozmiar każdego pliku binarnego oraz różnicę względem `target/dist/`, jeśli
ta kompilacja istnieje. Obok plików binarnych zapisuje też `BUILD-FEATURES.txt`.

## Co wybierasz

| Wybór | Jeśli go wyłączysz |
|---|---|
| Języki | Tylko angielski (budowany zawsze) oraz zaznaczone języki. Lista wyboru w Ustawieniach pokazuje wyłącznie zbudowane języki. |
| Motywy | GitPoulpi Dark, GitPoulpi Light i Terminal są budowane zawsze (`system` wymaga Dark i Light). Każdy inny motyw wybiera się według rodziny (`catppuccin`, `gruvbox`, `nightfox`, ...; wypisuje je `--list-themes`). Niewybrane motywy nie są kompilowane: brakuje ich na liście w Ustawieniach i w `gitpoulpi theme list`, a konfiguracja wskazująca taki motyw wraca do zachowania jak dla nieznanego motywu, ze zwykłym ostrzeżeniem. |
| Kolory logo | Pikselowa ośmiornica w oknie O programie GitPoulpi występuje w 20 wersjach kolorystycznych (`--list-logo-colors`). Wybierz te, które mają zostać skompilowane (co najmniej jedną), wersję domyślną (okno pokazuje ją jako pierwszą; do niej wraca też zapisane `ui.logo_color`, którego nie ma w kompilacji; musi być jedną ze skompilowanych; oficjalna domyślna: lavender) oraz kolor ikony pliku wykonywalnego w Windows (dowolna wersja, domyślnie: domyślna dla logo). Przy jednej wersji logo nadal obraca się po kliknięciu, ale zachowuje ten kolor, a lista kolorów (`c`, prawy przycisk myszy) nie jest oferowana. |
| Integracje (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Bez żadnej: tylko klonowanie z adresu URL; brak zgłoszeń i pull/merge requestów. |
| AI | Brak przycisków i poleceń AI oraz narzędzi `ai` w CLI/MCP. |
| MCP | Brak przekaźnika `gitpoulpi-mcp` i polecenia `gitpoulpi mcp`. Polecenia agenta `ui` pozostają. |
| LFS, Worktree, Submodules | Odpowiednie ekrany zarządzania i polecenia znikają; zwracana jest typowana odpowiedź „not built into this binary”. |
| Pobieranie awatarów | Tylko plakietki z inicjałami, bez pobierania z sieci. |
| Podgląd obrazów | Pliki obrazów pokazują zwykłą informację o pliku binarnym; **Edytuj w** nadal działa. |
| Radar | Brak karty Radar, `Alt+R` i polecenia palety; Radar nigdy się nie otwiera. Wymaga integracji (bez niej i tak nic nie robi). |
| Mermaid | Brak Kopiuj jako gitGraph Mermaid (menu Log, `Shift+Y`, paleta) i brak `log --format mermaid` w CLI/MCP. Także brak przycisku **Otwórz diagram** pod blokiem `mermaid` w [rozmowie](conversations.md) (blok zostaje jako kod) i brak wiersza palety *Rozmowa: Otwórz diagram wpisu*. |
| Board | Brak widoku tablicy Kanban (przycisk w nagłówku ZGŁOSZENIA, paleta) i polecenia `hosting board`. Wymaga integracji. |
| Rozmowa | Brak widoku [rozmowy](conversations.md) zgłoszenia lub żądania w aplikacji: Enter na wierszu zgłoszenia, karcie tablicy lub wierszu Radaru otwiera przeglądarkę jak dawniej, nie ma wiersza *Otwórz rozmowę* ani poleceń palety *Rozmowa: ...* i nie ma **Pisz z AI** w rozmowie (edytory przeglądu, formularza zgłoszenia i formularza pull requesta zachowują swoje). Polecenia rozmowy w CLI i MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) oraz `ai hosting-reply` zostają. Wymaga integracji. |
| Review | Strona Podsumowanie przeglądu to widok rozmowy, więc build z funkcją Review zawiera też widok rozmowy, niezależnie od wiersza *Rozmowa*. Brak widoku przeglądu kodu w aplikacji (menu wiersza, Enter na wierszu pull requesta, paleta, `Przejrzyj` w Radarze) i brak sugestii przeglądu od AI. Polecenia CLI/MCP `hosting review` zostają (korzystają z nich Gerrit i agenci). Wymaga integracji. |
| Warsztat historii (zaawansowane przepisywanie historii: autorzy, daty, komunikaty, zawartość) | Brak Warsztatu historii: brak poleceń palety `Przepisanie:`, akordu warsztatu, klawiszy i sekcji pomocy, brak wierszy „Przepisz historię” w menu Log, brak przycisku „Zmień historię głębiej...” w oknie Amend, brak „Utwórz reguły przepisania z AI” i AI dla skryptów, brak limitów skryptów na stronie Synchronizacja w ustawieniach i brak bloków AI przepisywania historii w ustawieniach AI. Polecenia CLI/MCP `rewrite`, `ai rewrite-script`, `ai rewrite-rules` i `ui propose rewrite` znikają z drzewa poleceń. Tożsamości `.mailmap` (`mailmap`, `log.use_mailmap`) nie są jego częścią i zostają. |
| Wyszukiwanie globalne | Brak przycisku Wyszukiwanie globalne na pasku tytułu, brak `Ctrl+Alt+F`, polecenia palety `Szukaj: Obszar roboczy...` i polecenia CLI/MCP `gitpoulpi search`. Pasek wyszukiwania w Logu (`ui show search`) to inna funkcja i zostaje. |
| Obszary robocze | Okno działa z jedną domyślną listą kart („Brak obszaru”): brak plakietki obszaru, karty Obszary robocze, `Alt+W` / `Alt+J`, poleceń palety `Obszar roboczy:`, wierszy kart „Dodaj do obszaru roboczego”, poleceń CLI/MCP `gitpoulpi workspace` i flagi `--workspace` w `commit` i `search`. Karty i sesje działają jak zwykle. Plik `workspaces.toml` zapisany przez inną kompilację nie jest ani czytany, ani zmieniany; profil domyślny i instrukcje AI obszaru roboczego nie obowiązują. Radar i wyszukiwanie globalne obejmują otwarte karty. |
| Skrypty przepisywania (wbudowany Lua) | Reguły Script przepisywania historii są odrzucane typowaną odpowiedzią „niewbudowane w ten plik binarny”, a Lua nie jest linkowany. Sam Warsztat historii zostaje (ma osobny wybór). Bez znaczenia, z ostrzeżeniem, gdy Warsztat historii jest wyłączony. |

Pliki konfiguracyjne są wspólne dla wszystkich kompilacji: ustawienie dotyczące
czegoś, czego nie zbudowano, jest zachowywane i ignorowane.

## Tryb interaktywny

Skrypt Bash używa list wyboru `whiptail` lub `dialog` (trzy ekrany, potem
podsumowanie) albo numerowanych pytań, gdy żadnego z nich nie ma. Skrypt PowerShell używa
`Out-GridView`, jeśli jest dostępny, w przeciwnym razie zadaje pytania. Poprzednie wybory są
zaznaczone z góry.

## Tryb nieinteraktywny

Dowolny argument pomija pytania i przechodzi od razu do
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # poprzedni wybór, tylko wypisz polecenie cargo
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Opcja | Znaczenie |
|---|---|
| `--langs fr,de` / `all` / `en` | Języki poza angielskim. Kody to nazwy plików lokalizacji (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Dostawcy: GitHub, GitLab (także self-managed), Azure DevOps (Services i Server), Bitbucket (Cloud i Data Center) (hosting), Gitea i Forgejo, także Codeberg (`gitea`) (hosting), Jira, Trello, Linear, YouTrack (Cloud i Server), Redmine (systemy zgłoszeń), Gerrit Code Review (`gerrit`, tylko przegląd), zgłoszenia Local Markdown (`local-issues`, folder lub skarbiec Obsidian). Każdy z `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` jest osobną funkcją cargo; rozmiary w [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Rodziny motywów do skompilowania (domyślnie `all`). `--list-themes` wypisuje rodziny. |
| `--logo-colors all` / `violet,teal` | Wersje kolorystyczne logo do skompilowania (domyślnie `all`, w kolejności z tabeli, czyli w kolejności obrotu). `--list-logo-colors` je wypisuje (`*` = oficjalna domyślna). |
| `--logo-default teal` | Domyślna wersja kolorystyczna; musi należeć do skompilowanych (domyślnie: lavender, w przeciwnym razie pierwsza skompilowana). |
| `--icon-color sun` | Kolor ikony pliku wykonywalnego w Windows (domyślnie: domyślny kolor logo). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Każda domyślnie włączona. |
| `--[no-]history-rewrite` | Warsztat historii (zaawansowane przepisywanie historii: autorzy, daty, komunikaty, zawartość). Domyślnie włączony. |
| `--[no-]global-search` | Wyszukiwanie globalne (przycisk na pasku tytułu, `Ctrl+Alt+F`, `gitpoulpi search`). Domyślnie włączone. |
| `--[no-]workspaces` | Obszary robocze (nazwane grupy repozytoriów). Domyślnie włączone; wyłączone zostawia tylko domyślną listę kart. |
| `--[no-]rewrite-scripts` | Wbudowany Lua dla reguł Script przepisywania. Domyślnie włączony; ignorowany z ostrzeżeniem, gdy Warsztat historii jest wyłączony. |
| `--save <name\|file>` | Zapisuje rozstrzygnięty wybór jako zapisany wybór (nazwa lub ścieżka pliku). |
| `--preset <name\|file>` | Zaczyna od zapisanego wyboru (nazwa lub ścieżka pliku); pozostałe opcje go nadpisują, niezależnie od miejsca w wierszu. |
| `--list-presets`, `--delete-preset <name>` | Wypisuje zapisane wybory i ich folder / usuwa jeden według nazwy. |
| `--last` | Używa ponownie `target/custom-build/last-choices.env`; pozostałe opcje go nadpisują. |
| `--full` | Bez żadnego wyboru (wszystko włączone). |
| `--profile custom\|dist\|release` | Domyślnie `custom` (ustawienia `dist`, osobny katalog wyjściowy). |
| `--dry-run` | Wypisuje dokładne polecenie cargo i kończy. |
| `-- <cargo args>` | Przekazywane do cargo, na przykład `--target x86_64-unknown-linux-musl`. |

Nazwy w PowerShell: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

Wybór motywów trafia do kompilacji jako zmienna środowiskowa
`GITPOULPI_THEMES` (`all`, `none` lub lista rodzin rozdzielona przecinkami), odczytywana
przez skrypt budowania `gitpoulpi-config`, który w czasie kompilacji odrzuca pozostałe
palety; można ją ustawić samodzielnie przy zwykłym `cargo build`. `gitpoulpi
--version` wypisuje wiersz `themes:` ze zbudowanym zestawem.

Wybory dotyczące logo to zmienne środowiskowe `GITPOULPI_LOGO_COLORS`
(`all` lub lista slugów rozdzielona przecinkami), `GITPOULPI_LOGO_DEFAULT` i
`GITPOULPI_ICON_COLOR`, odczytywane przez skrypty budowania `gitpoulpi-tui` (filtruje
`crates/gitpoulpi-logo-build/colourways.json` do tabeli osadzanej w oknie O programie GitPoulpi)
i `gitpoulpi-cli` (ikona). Źródłem kolorów pozostaje
`crates/gitpoulpi-logo-build/colourways.json`.

### Ikona pliku wykonywalnego w Windows

Gdy celem jest Windows, plik wykonywalny `gitpoulpi`
dostaje ośmiornicę jako ikonę, generowaną podczas budowania (bez potrzeby
Pythona ani ImageMagick) w wybranym kolorze ikony, w rozmiarach od 16 do 128 px.
Jest osadzana za pomocą crate'a `winresource` i wymaga kompilatora zasobów platformy
(`rc.exe` z Windows SDK dla MSVC, `windres` dla GNU). Jeśli go brakuje, kompilacja
tylko wypisuje ostrzeżenie, a plik wykonywalny zachowuje domyślną ikonę. W toolchainie GNU ikonę ma tylko `gitpoulpi` (zasób
zostałby inaczej dołączony dwukrotnie). Kompilacje dla Linuksa i macOS ignorują to wszystko
poza tabelą kolorów.

## Zapisane wybory

Aby zbudować nową wersję bez ponownego powtarzania całego wyboru, zapisz go pod nazwą i później zacznij od niego. Opcje podane razem z `--preset` nadal nadpisują zapisane wartości.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Zapisane wybory to zwykłe pliki `KLUCZ=wartość` (format `last-choices.env`, czytelny dla powłoki). Leżą w folderze `custom-builds` w katalogu konfiguracji GitPoulpi, poza `target/`, więc `cargo clean` ich nie usuwa:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (domyślnie `~/.config/gitpoulpi/custom-builds/`)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` przenosi ten folder razem z konfiguracją. `--preset` i `--save` przyjmują też ścieżkę do pliku (wartość zawierającą `/` lub `\` albo kończącą się na `.env`), aby trzymać wybór w repozytorium dotfiles.

Wybór zapisany przez inną wersję GitPoulpi nadal się wczyta. Nieznany klucz albo język, integracja, motyw lub kolor, które już nie istnieją, są pomijane z ostrzeżeniem. Klucz, którego brakuje w pliku (funkcja dodana później), dostaje wartość domyślną, a integracja dodana później jest dołączana; oba przypadki są zgłaszane na standardowym wyjściu błędów.

Gdy istnieje zapisany wybór, tryb interaktywny najpierw proponuje „Zacznij od” (poprzednie wybory, zapisany wybór lub wszystko włączone), a na końcu pyta, czy zapisać bieżący wybór pod nazwą.

## Ile to oszczędza

Zmierzone w Linuksie dnia 2026-10-29 na commicie `65999ef5c`, rozmiar `gitpoulpi` (całego programu: interfejs, CLI i serwer MCP) z pełnej kompilacji, różnica względem niej w nawiasach
(szczegóły w `docs/developer/binary-size.md`). Przekaźniki `gitpoulpi-tui`, `gitpoulpi-mcp` i `gitpoulpi-launch` mają dobrze poniżej 1 MiB niezależnie od wyboru:

| Kompilacja (od `full`, o ile nie zaznaczono inaczej) | `gitpoulpi` (MiB) |
|---|---|
| Pełna (wszystko włączone) | 64,6 |
| Tylko angielski | 52,0 (-12,6) |
| MCP wyłączone | 61,6 (-3,0) |
| AI wyłączone | 62,6 (-2,0) |
| Wszystkie integracje wyłączone | 56,3 (-8,2) |
| AI + integracje + pobieranie awatarów wyłączone | 52,2 (-12,3) |
| Warsztat historii wyłączony (`--no-history-rewrite`) | 61,6 (-3,0) |
| Skrypty przepisywania wyłączone | 63,1 (-1,5) |
| Wyszukiwanie globalne wyłączone | 63,5 (-1,1) |
| Obszary robocze wyłączone | 63,5 (-1,1) |
| Radar wyłączony | 63,4 (-1,2) |
| Board wyłączony | 63,6 (-1,0) |
| Review wyłączone | 63,2 (-1,4) |
| Mermaid wyłączony | 63,6 (-1,0) |
| LFS wyłączone | 63,5 (-1,1) |
| Worktree wyłączone | 63,7 (-0,9) |
| Submodules wyłączone | 63,7 (-0,9) |
| Pobieranie awatarów wyłączone | 63,7 (-0,9) |
| Podgląd obrazów wyłączony | 63,3 (-1,2) |
| Wszystko wyłączone, tylko angielski (LFS zostaje) | 35,2 (-29,4) |
| **Wszystko wyłączone, tylko angielski** | **34,9 (-29,7)** |

## Uwagi

- Własna kompilacja jest wolna jak każda kompilacja `dist` (thin LTO, kilka minut).
- Działa bez sccache, tak jak `build-opti`.
- Kody języków są sprawdzane względem dostarczanych lokalizacji; nieznany kod jest
  odrzucany wraz z listą poprawnych.
