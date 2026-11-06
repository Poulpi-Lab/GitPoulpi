[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · **Svenska** · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Företagsdistribution

För administratörer som driftsätter GitPoulpi på många datorer från sina egna servrar: interna paketfeeds, speglar, en policy för hela datorn och en standardkonfiguration. Läs [Installation](installation.md) och [Uppdateringar](updates.md) först.

Två fakta styr allt här:

- Varje installation registrerar **hur den installerades** i markören `.gitpoulpi-dist` bredvid programmen (`channel=...`). GitPoulpi uppdaterar sig självt bara på det sätt som kanalen tillåter: det ersätter aldrig filer som en pakethanterare äger, och en MSI rörs aldrig.
- **Policyn** (en fil, och i Windows registret) är din. En användares egen `config.toml` kan aldrig lätta på den.

## Chocolatey, med en privat feed

Det offentliga paketet laddar ned installationsprogrammet från GitHub. För ett stängt nätverk internaliserar du det:

1. Hämta paketet (`choco download gitpoulpi --internalize` med de licensierade Chocolatey-verktygen, eller ladda ned `.nupkg` och packa upp den).
2. Peka `url64bit` i `tools\chocolateyinstall.ps1` på din egen kopia av `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` och låt `checksum64` vara lika med det första fältet i releasens `.sha256`-fil (eller kontrollsumman för din kopia).
3. `choco pack`, och push sedan `.nupkg` till din feed (Nexus, Artifactory, ProGet, en filresurs).

Installera det med paketparametrar:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parameter | Effekt |
|---|---|
| `/Channel:choco-internal` | Registrerar `channel=choco-internal` i markören, så att GitPoulpi vet att feeden är din och inte Chocolateys communityfeed. |
| `/Feed:<url>` | Registrerar feedens adress (`feed=` i markören). GitPoulpi söker där efter nya versioner och kör `choco upgrade gitpoulpi -y --source=<feed>`. |

Utan parametrar är kanalen `choco` och sökningen går till Chocolateys communityfeed. Paketet installeras för alla användare i Program Files, utan aliaset `gp`.

GitPoulpis uppdatering kör `choco upgrade` utan `--params`, så att inget du har driftsatt rörs. Chocolatey måste därför komma ihåg paketparametrarna från den första installationen (`useRememberedArgumentsForUpgrades`, på som standard sedan Chocolatey 2.0; i äldre versioner `choco feature enable -n useRememberedArgumentsForUpgrades`). Annars skulle uppgraderingen registrera den vanliga kanalen `choco`. En policy-`check_url` (nedan) håller sökningen och uppgraderingen riktade mot din feed vad markören än säger, och den har företräde framför markörens feed.

**Ingenting i paketet skriver `policy.toml` eller `config.toml`.** Det du har driftsatt blir kvar, vid installation och vid varje uppgradering. Se [Distribuera en standard-config.toml](#distribuera-en-standard-configtoml).

## winget, med en intern källa

winget uppdaterar GitPoulpi med `winget upgrade --id GitPoulpi.GitPoulpi --exact`, med den källa som paketet kom från. För en intern källa värdar du en Windows Package Manager REST-källa (vilken implementation av winget-REST-API:t som helst), publicerar releasens tre manifest (`GitPoulpi.GitPoulpi.yaml`, `...installer.yaml`, `...locale.en-US.yaml`) med `InstallerUrl` pekande på din kopia av installationsprogrammet och motsvarande `InstallerSha256`, och kör sedan:

```
winget source add --name corp --arg https://winget.example/api --type Microsoft.Rest
winget install --id GitPoulpi.GitPoulpi --source corp
```

Behåll manifestets installationsväljare: `/CHANNEL=winget` gör att markören säger `winget`, och `/ALIAS=` utelämnar aliaset `gp`. winget talar inte om för GitPoulpi vad som är nytt, så sökningen frågar fortfarande GitHub efter versionen. I ett nätverk utan GitHub pekar du den mot din egen `latest.json` med en policy: `check = "manifest"` (se [latest.json](#latestjson-och-download_base-en-spegel-av-releasen)).

## Scoop, med en privat bucket

Lägg `gitpoulpi.json` (manifestet från den offentliga bucketen) i `bucket/` i ett Git-repository som du värdar, ändra `url` till din kopia av releasens zip och sätt `hash` till dess sha256 (`autoupdate` kan vara kvar om din kopia följer releasenamnen). Kör sedan:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

Manifestets `post_install` skriver om markören med `channel=scoop`; GitPoulpi kör sedan själv `scoop update gitpoulpi` och byter aldrig filer bakom ryggen på Scoop. Scoop installerar per användare och kräver inga administratörsrättigheter.

## apt, med en spegel

Repositoryt är ett vanligt katalogträd (`dists/`, `pool/`, signeringsnyckelns `gitpoulpi-archive-keyring.gpg` och `.asc`, och exempelfilen `gitpoulpi.sources`) som serveras över HTTPS. Spegla det med `wget --mirror`, `rsync` eller din artefakthanterare, servera det internt och installera nyckeln och källan på varje dator:

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

Behåll filnamnet `gitpoulpi.sources` (eller `gitpoulpi.list`): GitPoulpi visar `apt`-kommandon bara när det hittar `/etc/apt/sources.list.d/gitpoulpi.*`. Använd projektets nyckel: spegeln förblir signerad av den, och `Signed-By` fäster just den nyckeln för just det här repositoryt. Repositoryt behåller de fem senaste versionerna av varje paket.

**`.deb` offline:** varje release bifogar `gitpoulpi_<version>_<arch>.deb`. Kopiera över den och kör `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Markören säger `channel=apt`; utan källfilen ber GitPoulpi användaren att ladda ned nästa `.deb` i stället för att visa ett `apt`-kommando.

## MSI, med Intune eller grupprincip

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` installeras för alla användare och är avsedd att hanteras: GitPoulpis panel säger bara "hanteras av din administratör" (med ditt `message`), den uppdaterar aldrig en MSI-installation. Uppgradera genom att driftsätta den nya MSI:n; det är en större uppgradering, så den gamla versionen ersätts.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Egenskap | Effekt |
|---|---|
| `INSTALLFOLDER` | Installationsmapp (standard `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (standard) lägger mappen i datorns PATH, `0` gör det inte. |
| `UPDATES_MODE` | Policyns `mode`: `off`, `notify` eller `auto`. |
| `UPDATES_PIN` | Policyns `pin`, till exempel `0.2` eller `0.2.4`. |
| `UPDATES_CHECK` | Policyns `check`: `github`, `nuget`, `manifest` eller `none`. |
| `UPDATES_CHECK_URL` | Policyns `check_url`. |
| `UPDATES_MESSAGE` | Policyns `message`, visas i uppdateringspanelen. |

En egenskap skriver sitt värde till `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` bara när du sätter den, så en osatt egenskap lämnar registret (och en policyfil) i fred. **Intune:** driftsätt MSI:n som en line-of-business-app eller Win32-app med kommandot ovan. **Grupprincip:** programinstallation kan inte skicka egenskaper, så använd antingen en transform (`.mst`) eller sätt registervärdena med Group Policy Preferences, se nedan.

## Homebrew, med en intern tap

Kopiera `gitpoulpi.rb` från den offentliga tapen till `Formula/` i ett Git-repository som du värdar, ändra raderna `url` till dina kopior av releasearkiven och uppdatera varje `sha256` (det första fältet i `.sha256`-filen bredvid varje arkiv). Kör sedan:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Formeln installerar markören med `channel=homebrew`: GitPoulpi kör själv `brew upgrade gitpoulpi`, som läser din tap.

## Spegel av Docker-registret

Avbildens namn är `poulpilab/gitpoulpi` (och `ghcr.io/poulpi-lab/gitpoulpi`). Panelen visar alltid det namnet, så en spegel måste vara transparent.

- **Genomströmmande cache:** kör ett register som proxy för Docker Hub (avbilden `registry` med `proxy.remoteurl: https://registry-1.docker.io`) och sätt `"registry-mirrors": ["https://registry.example"]` i Docker-daemonens `daemon.json`. `docker pull poulpilab/gitpoulpi` går då via din cache.
- **Kopia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, och kör sedan det namnet. Panelen visar fortfarande det offentliga, vilket går bra som en pekare till versionen.

Se [Köra i Docker](docker.md) för hur du kör den.

## Policyfilen och registret

Policyn anger hur uppdateringar beter sig på en dator och kan inte ändras av användare. Den läses från:

| OS | Policyfil |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, även | registernyckeln `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, ett `REG_SZ`-värde per nyckel nedan, med samma namn (för grupprincip, Intune och MSI-egenskaper) |

Filen har en tabell:

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

| Nyckel | Värden | Effekt |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Ett tak: `off` < `notify` < `auto`. Användare får välja ett lägre värde, aldrig ett högre. `off` döljer sökningen och gör inga anrop. |
| `release_channel` | `stable`, `prerelease` | Tvingar uppdateringskanalen. |
| `pin` | `1.4` eller `1.4.2` | Den högsta version som erbjuds: `1.4` tillåter vilken `1.4.x` som helst, `1.4.2` tillåter högst `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Var frågan "vilken är den senaste releasen" ställs. Standard per installation: GitHub, Chocolatey-feeden för Chocolatey-kanalerna. `none` söker aldrig. |
| `check_url` | URL | Med `nuget`: feeden. Med `manifest`: adressen till `latest.json`. Med `github`: GitHub-API:ts bas, för GitHub Enterprise Server (lägg till `repo`). |
| `repo` | `owner/name` | Repositoryt som söks med `check = "github"`. |
| `download_base` | URL | En spegel av releasefilerna (samma filnamn) för arkiv- och installationskanalerna. Nedladdningar från den här värden tillåts utöver GitHubs. |
| `allow_self_update` | `true`, `false` | `false` gör varje åtgärd "ersätt GitPoulpis filer" och "kör pakethanteraren" till en notering eller ett kommando att kopiera. Standard `true`. |
| `message` | text | Visas i uppdateringspanelen ("Från din administratör: ..."). |

**Företräde**, starkast först: registret, policyfilen, användarens `config.toml` `[updates]`, markörens `feed=`, de inbyggda standardvärdena. En nyckel som policyn sätter är **Inställt av din administratör** i Inställningar och kan inte redigeras där.

**Misslyckas stängt:** en policyfil eller ett registervärde som inte går att läsa (ett TOML-syntaxfel, `mode = "sometimes"`, en `pin` som inte är en version) behandlas som `mode = notify` och `allow_self_update = false`, och en rad skrivs till programloggen. Ett stavfel låter aldrig en flotta uppdatera sig själv. En okänd nyckel ignoreras med en varning. I Windows läser steget som har administratörsrättigheter bara datorns policy, aldrig användarens konfiguration.

### Exempel

Stäng av uppdateringar (en låst avbild, ett nätverk utan internetförbindelse):

```toml
[updates]
mode = "off"
```

Fäst flottan vid 1.4-linjen, bara meddela, och säg vem man ska fråga:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Sök i en intern Chocolatey-feed (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Samma sorts policy i Windows via registret (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json och download_base: en spegel av releasen

Kopiera en releases filer (arkiven, installationsprogrammet, `.sha256`-filerna, `latest.json`, **varje `.minisig`-signaturfil** och `security.json`) till en mapp på en intern webbserver och behåll namnen. `latest.json` bifogas varje release:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Policyn blir då:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Arkiv- och installationskanalerna kontrollerar manifestet och laddar ned från `download_base`, med samma verifiering som från GitHub. En spegel behöver bara kopiera filer; den behöver inget API.

Signaturerna följer med filerna, så en spegel kan inte försvaga dem:

- `latest.json` betros bara när `latest.json.minisig` ligger bredvid den och verifieras mot projektets nyckel som är inbyggd i GitPoulpi. Ett manifest utan sin signatur, eller ändrat efter att det signerades, nekas (hasharna i det är det nedladdningarna kontrolleras mot).
- Varje arkiv laddas ned tillsammans med `<archive>.minisig` från samma värd och installeras bara när den signaturen verifieras och namnger den här filen och den här versionen. En spegel som kopierade arkivet men inte dess `.minisig` får uppdateringen att misslyckas med "no signature", och ingenting ändras.
- `security.json` (säkerhetsmeddelandena, se [Uppdateringar](updates.md#säkerhetsmeddelanden)) och `security.json.minisig` söks bredvid manifestet. En spegel som inte kopierar dem visar helt enkelt inga meddelanden.

Kopiera filerna oförändrade: en signatur täcker de exakta byten. Säkerhetsmeddelandena publiceras om för den senaste releasen utan ett nytt bygge, så uppdatera `latest.json`, `security.json` och deras `.minisig`-filer från releasesidan varje gång du uppdaterar spegeln.

## Distribuera en standard-config.toml

GitPoulpi har ingen `config.toml` för hela datorn: inställningarna ligger i varje användares mapp (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). För att ge nya användare dina standardvärden kopierar du en gles `config.toml` (bara de nycklar du vill ha, se [Konfigurationsreferens](config-reference.md)) till den mappen när användaren inte har någon: ett inloggnings- eller Intune-skript i Windows, `/etc/skel/.config/gitpoulpi/` i Linux, ett skript vid första inloggningen i macOS. Ingenting i GitPoulpis paket skriver någonsin över en konfiguration, så den är din att hantera. Lägg inte uppdateringsregler där när de måste gälla: använd policyn, som en användare inte kan åsidosätta. En portabel konfiguration eller en per jobb kan också väljas med `GITPOULPI_CONFIG_DIR`.
