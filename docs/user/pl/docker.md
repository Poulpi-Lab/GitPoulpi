[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · **Polski** · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Uruchamianie w Dockerze

Obraz `poulpilab/gitpoulpi` zawiera oficjalne pliki wykonywalne wydań (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) na `debian:bookworm-slim`, razem z `git`, `git-lfs`, `openssh-client`, `less` i `tini`. Działa na `linux/amd64` i `linux/arm64`. Jest w Docker Hub; te same tagi są też w `ghcr.io/poulpi-lab/gitpoulpi`, a wszystko poniżej działa z obiema nazwami.

## Uruchomienie interfejsu

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` daje interfejsowi terminal. Bez tego program nie ma na czym rysować.
- `-v "$PWD":/work` montuje bieżący folder jako katalog roboczy (`/work`). Otwórz stamtąd repozytorium albo zamontuj kilka repozytoriów w `/work`.
- `-v gitpoulpi-config:/config` to nazwany wolumen, który zachowuje ustawienia i sesje między uruchomieniami.
- `--user "$(id -u):$(id -g)"` uruchamia jako ty, więc pliki tworzone przez GitPoulpi w twoich repozytoriach zachowują właściciela z hosta zamiast `root`. Każdy folder, do którego pisze obraz, jest zapisywalny dla wszystkich, więc działa dowolne id użytkownika.

## Wolumeny i ścieżki

| Ścieżka | Co zawiera |
|---|---|
| `/work` | Katalog roboczy. Zamontuj tu swoje repozytoria. |
| `/config` | Ustawienia (`config.toml`), sesje oraz, jako podfoldery, dane i pamięć podręczna (`GITPOULPI_CONFIG_DIR=/config`). Użyj nazwanego wolumenu lub montowania folderu, by je zachować. |
| `/data`, `/cache` | Zadeklarowane także jako wolumeny, dla narzędzi używających zmiennych XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. To nie jest wolumen: w każdym kontenerze jest nowy. |

Bez wolumenu dla `/config` każde uruchomienie z `--rm` zaczyna z ustawieniami domyślnymi. Aby zacząć od własnego `config.toml`, zamontuj go: `-v "$PWD/config.toml":/config/config.toml:ro` (tylko do odczytu oznacza, że Ustawienia nie mogą zapisywać).

## Repozytoria należące do innego użytkownika

Git odmawia pracy z repozytorium, którego właściciel różni się od użytkownika, który go uruchamia („dubious ownership”). Z `--user` jak wyżej właściciel się zgadza. Gdy się nie zgadza (montowanie z innego konta, udział Docker Desktop), zaufaj każdemu katalogowi tak:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Punkt wejścia uruchamia wtedy `git config --global --add safe.directory '*'` przed startem interfejsu. Używaj tego tylko dla folderów, którym ufasz, bo znosi ochronę gita dla całego kontenera.

## SSH

Przekaż swojego agenta SSH zamiast kopiować klucze do kontenera.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) udostępnia agenta hosta pod stałą ścieżką wewnątrz swojej maszyny wirtualnej:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Z `--user` gniazdo musi być czytelne dla twojego id użytkownika, co jest spełnione, gdy należy do ciebie. Kontener nie ma `known_hosts`: pierwsze połączenie z hostem prosi o zaakceptowanie jego klucza, albo zamontuj własny plik przez `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Tożsamość git i poświadczenia

- **Tożsamość i ustawienia:** zamontuj swoją konfigurację Gita tylko do odczytu: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Albo ustaw tożsamość commitów w profilu GitPoulpi (Ustawienia > Profile), który leży w `/config`.
- **Poświadczenia HTTPS:** obraz nie ma helpera poświadczeń, takiego jak pęk kluczy. Zamontuj magazyn poświadczeń (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, z `credential.helper = store` w zamontowanym `.gitconfig`) albo użyj zdalnego repozytorium SSH z przekazanym agentem.
- **Tokeny hostingu** (pull requesty, zgłoszenia): bez helpera poświadczeń GitPoulpi trzyma token w pamięci tylko na czas sesji. Dla trwałej konfiguracji przekaż go przez środowisko: `-e GITPOULPI_TOKEN_<ID>=...`, gdzie `<ID>` to id integracji zapisane wielkimi literami, z znakami niealfanumerycznymi jako `_` (zob. [Integracje z hostingiem i trackerami zgłoszeń](hosting-integration.md)). Same integracje są w `config.toml` w `/config`.

## Terminal, kolory i mysz

Kolory i obsługa myszy pochodzą z twojego emulatora terminala. Docker przekazuje `TERM` z `-t`; dodaj `-e TERM -e COLORTERM`, gdy kolory lub motyw true-color wyglądają źle, na przykład `-e COLORTERM=truecolor`. Kliknięcia, przeciąganie i kółko myszy docierają przez terminal jak zwykle. Graf commitów jest rysowany znakami; obrazy (kitty, iTerm2, sixel) nie są przekazywane przez każdy terminal i multiplekser, zob. [Wybór terminala](terminals.md). Zmiana rozmiaru okna terminala zmienia rozmiar interfejsu: kontener dostaje informację o zmianie rozmiaru.

## CLI i MCP

Punkt wejścia uruchamia interfejs. Pozostałe programy są dostępne po jego nadpisaniu:

```
# skryptowalne CLI (dodaj --json dla skryptów)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# serwer MCP, przez standardowe wejście i wyjście, dla agenta, który go uruchamia
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Każde `docker run` to osobny kontener z własnym demonem GitPoulpi w tle, więc zmiany wykonane przez CLI nie pokazują się na żywo w interfejsie działającym w innym kontenerze. Aby sterować jednym interfejsem z CLI, uruchom oba w tym samym kontenerze: wystartuj interfejs z `--name gp` i używaj `docker exec gp gitpoulpi ...`. Dla agenta zarejestruj `docker run -i --rm --entrypoint gitpoulpi-mcp ...` jako polecenie serwera MCP, zob. [MCP i agenci](mcp-and-agents.md); przełączniki uprawnień MCP w zamontowanym `config.toml` działają jak zwykle.

## Tagi i aktualizowanie

| Tag | Znaczenie |
|---|---|
| `X.Y.Z` | Jedno dokładne wydanie. |
| `X.Y` | Najnowsza łatka tej wersji minor (tylko wydania stabilne). |
| `latest` | Najnowsze wydanie stabilne. |

Wydanie wstępne dostaje tylko swój dokładny tag `X.Y.Z-...`.

Obraz się wymienia, nigdy nie aktualizuje w miejscu. GitPoulpi wewnątrz kontenera o tym wie (jego zapis instalacji mówi „obraz Docker”): [panel aktualizacji](updates.md) pokazuje wersję i polecenia, a niczego nie zastępuje. Aby zaktualizować:

```
docker pull poulpilab/gitpoulpi
```

Potem uruchom nowy kontener. Twoje ustawienia zostają w wolumenie `/config`. Aby wrócić, uruchom starszy tag: `docker pull poulpilab/gitpoulpi:<previous>`. Aby przypiąć wersję w skryptach, użyj dokładnego tagu zamiast `latest`. Organizacja może zrobić lustro obrazu we własnym rejestrze, zob. [Dystrybucja w przedsiębiorstwie](enterprise-distribution.md#lustro-rejestru-docker).
