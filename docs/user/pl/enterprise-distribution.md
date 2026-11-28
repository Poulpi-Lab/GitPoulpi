[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · **Polski** · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Dystrybucja w przedsiębiorstwie

Dla administratorów, którzy wdrażają GitPoulpi na wielu komputerach z własnych serwerów: wewnętrzne feedy pakietów, lustra, polityka aktualizacji obowiązująca na całym komputerze i konfiguracja domyślna. Najpierw przeczytaj [Instalację](installation.md) i [Aktualizacje](updates.md).

Wszystko tutaj wynika z dwóch faktów:

- Każda instalacja zapisuje **sposób instalacji** w znaczniku `.gitpoulpi-dist` obok programów (`channel=...`). GitPoulpi aktualizuje się sam tylko tak, jak pozwala ten kanał: nigdy nie zastępuje plików, którymi zarządza menedżer pakietów, a MSI nigdy nie jest ruszane.
- **Polityka** (plik, a w Windows rejestr) należy do ciebie. Własny `config.toml` użytkownika nigdy jej nie złagodzi.

## Chocolatey, z prywatnym feedem

Publiczny pakiet pobiera instalator z GitHuba. Dla sieci zamkniętej zrób jego wewnętrzną kopię:

1. Weź pakiet (`choco download gitpoulpi --internalize` z licencjonowanymi narzędziami Chocolatey albo pobierz `.nupkg` i go rozpakuj).
2. W `tools\chocolateyinstall.ps1` skieruj `url64bit` na własną kopię `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` i zostaw `checksum64` równe pierwszemu polu pliku `.sha256` wydania (albo sumie kontrolnej twojej kopii).
3. `choco pack`, potem wypchnij `.nupkg` do swojego feedu (Nexus, Artifactory, ProGet, udział plikowy).

Zainstaluj go z parametrami pakietu:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parametr | Skutek |
|---|---|
| `/Channel:choco-internal` | Zapisuje `channel=choco-internal` w znaczniku, żeby GitPoulpi wiedział, że feed jest twój, a nie publicznym feedem Chocolatey. |
| `/Feed:<url>` | Zapisuje adres feedu (`feed=` w znaczniku). GitPoulpi sprawdza go pod kątem nowych wersji i uruchamia `choco upgrade gitpoulpi -y --source=<feed>`. |

Bez parametrów kanał to `choco`, a sprawdzanie idzie do publicznego feedu Chocolatey. Pakiet instaluje się dla wszystkich użytkowników w Program Files, bez aliasu `gp`.

Aktualizacja GitPoulpi uruchamia `choco upgrade` bez `--params`, żeby nic z tego, co wdrożyłeś, nie zostało ruszone. Chocolatey musi więc pamiętać parametry pakietu z pierwszej instalacji (`useRememberedArgumentsForUpgrades`, domyślnie włączone od Chocolatey 2.0; w starszych wersjach `choco feature enable -n useRememberedArgumentsForUpgrades`). Inaczej aktualizacja zapisałaby zwykły kanał `choco`. `check_url` z polityki (niżej) trzyma sprawdzanie i aktualizację skierowane na twój feed niezależnie od znacznika i ma pierwszeństwo przed feedem ze znacznika.

**Nic w pakiecie nie zapisuje `policy.toml` ani `config.toml`.** To, co wdrożyłeś, zostaje, przy instalacji i przy każdej aktualizacji. Zob. [Wdrażanie domyślnego config.toml](#wdrażanie-domyślnego-configtoml).

## Scoop, z prywatnym bucketem

Wstaw `gitpoulpi.json` (manifest z publicznego bucketa) do `bucket/` repozytorium Git, które sam hostujesz, zmień `url` na swoją kopię zipa wydania i ustaw `hash` na jego sha256 (`autoupdate` może zostać, jeśli twoja kopia podąża za nazwami wydań). Potem:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

`post_install` manifestu przepisuje znacznik z `channel=scoop`; GitPoulpi sam uruchamia wtedy `scoop update gitpoulpi` i nigdy nie podmienia plików za plecami Scoop. Scoop instaluje dla użytkownika i nie wymaga uprawnień administratora.

## apt, z lustrem

Repozytorium to zwykłe drzewo katalogów (`dists/`, `pool/`, pliki `gitpoulpi-archive-keyring.gpg` i `.asc` z kluczem podpisującym oraz przykładowy `gitpoulpi.sources`) serwowane przez HTTPS. Zrób jego lustro przez `wget --mirror`, `rsync` albo swojego menedżera artefaktów, udostępnij je wewnętrznie i zainstaluj klucz oraz źródło na każdym komputerze:

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Zachowaj nazwę pliku `gitpoulpi.sources` (albo `gitpoulpi.list`): GitPoulpi pokazuje polecenia `apt` tylko wtedy, gdy znajdzie `/etc/apt/sources.list.d/gitpoulpi.*`. Użyj klucza projektu: lustro pozostaje przez niego podpisane, a `Signed-By` przypina ten jeden klucz tylko dla tego repozytorium. Repozytorium trzyma ostatnie pięć wersji każdego pakietu.

**`.deb` offline:** każde wydanie dołącza `gitpoulpi_<version>_<arch>.deb`. Skopiuj go i `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Znacznik mówi `channel=apt`; bez pliku źródła GitPoulpi mówi użytkownikowi, by pobrał następny `.deb`, zamiast pokazywać polecenie `apt`.

## MSI, z Intune lub Zasadami grupy

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` instaluje dla wszystkich użytkowników i jest przeznaczony do zarządzania: panel GitPoulpi mówi tylko „zarządzane przez administratora” (z twoim `message`), nigdy nie aktualizuje instalacji MSI. Aktualizuj, wdrażając nowy MSI; to duża aktualizacja, więc stara wersja jest zastępowana.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Właściwość | Skutek |
|---|---|
| `INSTALLFOLDER` | Folder instalacji (domyślnie `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (domyślnie) dodaje folder do PATH komputera, `0` nie. |
| `UPDATES_MODE` | `mode` polityki: `off`, `notify` lub `auto`. |
| `UPDATES_PIN` | `pin` polityki, na przykład `0.2` lub `0.2.4`. |
| `UPDATES_CHECK` | `check` polityki: `github`, `nuget`, `manifest` lub `none`. |
| `UPDATES_CHECK_URL` | `check_url` polityki. |
| `UPDATES_MESSAGE` | `message` polityki, pokazywany w panelu aktualizacji. |

Właściwość zapisuje swoją wartość w `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` tylko wtedy, gdy ją ustawisz, więc nieustawiona zostawia rejestr (i plik polityki) w spokoju. **Intune:** wdróż MSI jako aplikację line-of-business lub Win32 z powyższym poleceniem. **Zasady grupy:** instalacja oprogramowania nie przekazuje właściwości, więc użyj transformacji (`.mst`) albo ustaw wartości rejestru przez Preferencje zasad grupy, zob. niżej.

## Homebrew, z wewnętrznym tapem

Skopiuj `gitpoulpi.rb` z publicznego tapa do `Formula/` repozytorium Git, które sam hostujesz, zmień wiersze `url` na swoje kopie archiwów wydania i zaktualizuj każde `sha256` (pierwsze pole pliku `.sha256` obok każdego archiwum). Potem:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Formuła instaluje znacznik z `channel=homebrew`: GitPoulpi sam uruchamia `brew upgrade gitpoulpi`, które czyta twój tap.

## Lustro rejestru Docker

Nazwa obrazu to `poulpilab/gitpoulpi`. Panel zawsze pokazuje tę nazwę, więc lustro musi być przezroczyste.

- **Pamięć podręczna pull-through:** uruchom rejestr jako proxy Docker Hub (obraz `registry` z `proxy.remoteurl: https://registry-1.docker.io`) i ustaw `"registry-mirrors": ["https://registry.example"]` w `daemon.json` demona Dockera. `docker pull poulpilab/gitpoulpi` idzie wtedy przez twoją pamięć podręczną.
- **Kopia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, a potem uruchom tę nazwę. Panel nadal pokazuje publiczną, co jest w porządku jako wskazanie wersji.

Jak go uruchomić, zob. [Uruchamianie w Dockerze](docker.md).

## Plik polityki i rejestr

Polityka ustawia, jak aktualizacje zachowują się na komputerze, i użytkownicy nie mogą jej zmienić. Jest czytana z:

| System | Plik polityki |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, także | klucz rejestru `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, jedna wartość `REG_SZ` na każdy klucz poniżej, o tych samych nazwach (dla Zasad grupy, Intune i właściwości MSI) |

Plik ma jedną tabelę:

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| Klucz | Wartości | Skutek |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Pułap: `off` < `notify` < `auto`. Użytkownicy mogą wybrać niższą wartość, nigdy wyższą. `off` ukrywa sprawdzanie i nie wysyła żadnych zapytań. |
| `release_channel` | `stable`, `prerelease` | Wymusza kanał aktualizacji. |
| `pin` | `1.4` lub `1.4.2` | Najwyższa oferowana wersja: `1.4` dopuszcza każde `1.4.x`, `1.4.2` dopuszcza co najwyżej `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Gdzie pytać, „jakie jest najnowsze wydanie”. Domyślnie zależnie od instalacji: GitHub, a dla kanałów Chocolatey feed Chocolatey. `none` nigdy nie sprawdza. |
| `check_url` | URL | Z `nuget`: feed. Z `manifest`: adres `latest.json`. Z `github`: baza API GitHuba, dla GitHub Enterprise Server (dodaj `repo`). |
| `repo` | `owner/name` | Repozytorium sprawdzane przy `check = "github"`. |
| `download_base` | URL | Lustro plików wydania (te same nazwy plików) dla kanałów archiwów i instalatorów. Pobieranie z tego hosta jest dozwolone obok pobierania z GitHuba. |
| `allow_self_update` | `true`, `false` | `false` zamienia każdą akcję „zastąp pliki GitPoulpi” i „uruchom menedżer pakietów” w notatkę lub polecenie do skopiowania. Domyślnie `true`. |
| `message` | tekst | Pokazywany w panelu aktualizacji („Od administratora: ...”). |

**Pierwszeństwo**, od najsilniejszego: rejestr, plik polityki, `[updates]` w `config.toml` użytkownika, `feed=` ze znacznika, wbudowane wartości domyślne. Klucz ustawiony przez politykę ma w Ustawieniach **Ustawione przez administratora** i nie można go tam edytować.

**Odmowa przy błędzie:** plik polityki lub wartość rejestru, których nie da się odczytać (błąd składni TOML, `mode = "sometimes"`, `pin`, który nie jest wersją), są traktowane jak `mode = notify` i `allow_self_update = false`, a jeden wiersz trafia do dziennika aplikacji. Literówka nigdy nie zostawia floty aktualizującej się samej. Nieznany klucz jest ignorowany z ostrzeżeniem. W Windows krok z uprawnieniami administratora czyta tylko politykę komputera, nigdy konfigurację użytkownika.

### Przykłady

Wyłącz aktualizacje (zablokowany obraz, sieć odizolowana):

```toml
[updates]
mode = "off"
```

Przypnij flotę do linii 1.4, tylko powiadamianie, i powiedz, kogo pytać:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Sprawdzaj wewnętrzny feed Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Ta sama polityka w Windows przez rejestr (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json i download_base: lustro wydania

Skopiuj pliki wydania (archiwa, instalator, pliki `.sha256`, `latest.json`, **każdy plik podpisu `.minisig`** i `security.json`) do jednego folderu wewnętrznego serwera WWW, zachowując nazwy. `latest.json` jest dołączany do każdego wydania:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Wtedy polityka wygląda tak:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Kanały archiwów i instalatorów sprawdzają manifest i pobierają z `download_base`, z taką samą weryfikacją jak z GitHuba. Lustro musi tylko kopiować pliki; nie potrzebuje API.

Podpisy podróżują razem z plikami, więc lustro nie może ich osłabić:

- `latest.json` jest zaufany tylko wtedy, gdy obok niego leży `latest.json.minisig` i weryfikuje się kluczem projektu wbudowanym w GitPoulpi. Manifest bez podpisu albo edytowany po podpisaniu jest odrzucany (skróty w nim zawarte to to, z czym sprawdzane są pobrania).
- Każde archiwum jest pobierane razem z `<archive>.minisig` z tego samego hosta i jest instalowane tylko wtedy, gdy ten podpis się weryfikuje i wskazuje ten plik oraz tę wersję. Lustro, które skopiowało archiwum, ale nie jego `.minisig`, powoduje niepowodzenie aktualizacji z komunikatem „no signature” i nic się nie zmienia.
- `security.json` (komunikaty o bezpieczeństwie, zob. [Aktualizacje](updates.md#komunikaty-o-bezpieczeństwie)) i `security.json.minisig` są szukane obok manifestu. Lustro, które ich nie kopiuje, po prostu nie pokazuje komunikatów.

Kopiuj pliki bez zmian: podpis obejmuje dokładne bajty. Komunikaty o bezpieczeństwie są publikowane ponownie dla najnowszego wydania bez nowej kompilacji, więc odświeżaj `latest.json`, `security.json` i ich pliki `.minisig` ze strony wydania za każdym razem, gdy odświeżasz lustro.

## Wdrażanie domyślnego config.toml

GitPoulpi nie ma `config.toml` obowiązującego na całym komputerze: ustawienia leżą w folderze każdego użytkownika (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Aby dać nowym użytkownikom swoje wartości domyślne, skopiuj rzadki `config.toml` (tylko klucze, które chcesz, zob. [Opis konfiguracji](config-reference.md)) do tego folderu, gdy użytkownik go nie ma (domyślne ustawienia interfejsu, takie jak `ui.theme` czy `ui.language`, trafiają do rzadkiego `tui.toml` obok niego, który terminalowe UI czyta samodzielnie; zob. [Dwa pliki](config-reference.md#dwa-pliki-configtoml-i-tuitoml)): skrypt logowania lub Intune w Windows, `/etc/skel/.config/gitpoulpi/` w Linuksie, skrypt pierwszego logowania w macOS. Nic w pakietach GitPoulpi nigdy nie nadpisuje konfiguracji, więc zarządzanie nią należy do ciebie. Nie umieszczaj tam reguł aktualizacji, gdy muszą obowiązywać: użyj polityki, której użytkownik nie może nadpisać. Przenośną konfigurację lub konfigurację dla jednego zadania można też wybrać przez `GITPOULPI_CONFIG_DIR`.
