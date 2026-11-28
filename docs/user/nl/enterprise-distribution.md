[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · **Nederlands** · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distributie voor organisaties

Voor beheerders die GitPoulpi vanaf hun eigen servers op veel computers uitrollen: interne pakketfeeds, mirrors, een updatebeleid voor de hele machine en een standaardconfiguratie. Lees eerst [Installatie](installation.md) en [Updates](updates.md).

Twee feiten bepalen alles hier:

- Elke installatie legt **hoe ze is geïnstalleerd** vast in de markering `.gitpoulpi-dist` naast de programma's (`channel=...`). GitPoulpi werkt zichzelf alleen bij op de manier die dat kanaal toestaat: het vervangt nooit bestanden die een pakketbeheerder beheert, en een MSI wordt nooit aangeraakt.
- Het **beleid** (een bestand, en op Windows het register) is van jou. De eigen `config.toml` van een gebruiker kan het nooit versoepelen.

## Chocolatey, met een privéfeed

Het openbare pakket downloadt het installatieprogramma van GitHub. Voor een gesloten netwerk maak je het intern:

1. Haal het pakket op (`choco download gitpoulpi --internalize` met de gelicentieerde Chocolatey-tools, of download de `.nupkg` en pak hem uit).
2. Wijs in `tools\chocolateyinstall.ps1` `url64bit` naar je eigen kopie van `gitpoulpi-<versie>-x86_64-pc-windows-msvc-setup.exe` en houd `checksum64` gelijk aan het eerste veld van het `.sha256`-bestand van de release (of aan de checksum van je kopie).
3. `choco pack`, en push de `.nupkg` daarna naar je feed (Nexus, Artifactory, ProGet, een bestandsshare).

Installeer het met pakketparameters:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parameter | Effect |
|---|---|
| `/Channel:choco-internal` | Legt `channel=choco-internal` vast in de markering, zodat GitPoulpi weet dat de feed van jou is en niet de communityfeed van Chocolatey. |
| `/Feed:<url>` | Legt het feedadres vast (`feed=` in de markering). GitPoulpi controleert er nieuwe versies en voert `choco upgrade gitpoulpi -y --source=<feed>` uit. |

Zonder parameters is het kanaal `choco` en gaat de controle naar de communityfeed van Chocolatey. Het pakket installeert voor alle gebruikers in Program Files, zonder alias `gp`.

De update van GitPoulpi voert `choco upgrade` uit zonder `--params`, zodat niets wat je hebt uitgerold wordt aangeraakt. Chocolatey moet daarom de pakketparameters van de eerste installatie onthouden (`useRememberedArgumentsForUpgrades`, standaard aan sinds Chocolatey 2.0; op oudere versies `choco feature enable -n useRememberedArgumentsForUpgrades`). Anders zou de upgrade het gewone kanaal `choco` vastleggen. Een beleids-`check_url` (hieronder) houdt de controle en de upgrade op jouw feed gericht, wat de markering ook zegt, en heeft voorrang op de feed van de markering.

**Niets in het pakket schrijft `policy.toml` of `config.toml`.** Wat je hebt uitgerold blijft staan, bij de installatie en bij elke upgrade. Zie [Een standaard-config.toml uitrollen](#een-standaard-configtoml-uitrollen).

## Scoop, met een privébucket

Zet `gitpoulpi.json` (het manifest uit de openbare bucket) in `bucket/` van een Git-repository die je host, wijzig `url` naar je kopie van de release-zip en zet `hash` op zijn sha256 (`autoupdate` mag blijven staan als je kopie de releasenamen volgt). Dan:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

De `post_install` van het manifest herschrijft de markering met `channel=scoop`; GitPoulpi voert dan zelf `scoop update gitpoulpi` uit en wisselt nooit bestanden achter Scoop om. Scoop installeert per gebruiker en heeft geen beheerdersrechten nodig.

## apt, met een mirror

De repository is een gewone mappenboom (`dists/`, `pool/`, de `gitpoulpi-archive-keyring.gpg` en `.asc` van de ondertekeningssleutel, en de voorbeeldbron `gitpoulpi.sources`) die via HTTPS wordt aangeboden. Spiegel hem met `wget --mirror`, `rsync` of je artefactbeheer, bied hem intern aan, en installeer de sleutel en de bron op elke computer:

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

Houd de bestandsnaam `gitpoulpi.sources` (of `gitpoulpi.list`): GitPoulpi toont `apt`-opdrachten alleen als het `/etc/apt/sources.list.d/gitpoulpi.*` vindt. Gebruik de sleutel van het project: de mirror blijft ermee ondertekend, en `Signed-By` pint alleen die sleutel voor deze repository. De repository bewaart de laatste vijf versies van elk pakket.

**Offline `.deb`:** elke release heeft `gitpoulpi_<versie>_<arch>.deb` als bijlage. Kopieer het over en voer `sudo apt install ./gitpoulpi_<versie>_<arch>.deb` uit. De markering zegt `channel=apt`; zonder het bronbestand zegt GitPoulpi de gebruiker de volgende `.deb` te downloaden in plaats van een `apt`-opdracht te tonen.

## MSI, met Intune of Groepsbeleid

`gitpoulpi-<versie>-x86_64-pc-windows-msvc.msi` installeert voor alle gebruikers en is bedoeld om beheerd te worden: het paneel van GitPoulpi zegt alleen "beheerd door je beheerder" (met jouw `message`) en werkt een MSI-installatie nooit bij. Upgrade door de nieuwe MSI uit te rollen; het is een major upgrade, dus de oude versie wordt vervangen.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Eigenschap | Effect |
|---|---|
| `INSTALLFOLDER` | Installatiemap (standaard `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (standaard) voegt de map toe aan het PATH van de machine, `0` niet. |
| `UPDATES_MODE` | Beleid `mode`: `off`, `notify` of `auto`. |
| `UPDATES_PIN` | Beleid `pin`, bijvoorbeeld `0.2` of `0.2.4`. |
| `UPDATES_CHECK` | Beleid `check`: `github`, `nuget`, `manifest` of `none`. |
| `UPDATES_CHECK_URL` | Beleid `check_url`. |
| `UPDATES_MESSAGE` | Beleid `message`, getoond in het updatepaneel. |

Een eigenschap schrijft zijn waarde alleen naar `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` als je hem instelt, dus een niet-ingestelde laat het register (en een beleidsbestand) met rust. **Intune:** rol de MSI uit als line-of-business- of Win32-app met de opdracht hierboven. **Groepsbeleid:** software-installatie kan geen eigenschappen meegeven, dus gebruik een transform (`.mst`) of stel de registerwaarden in met Group Policy Preferences, zie hieronder.

## Homebrew, met een interne tap

Kopieer `gitpoulpi.rb` uit de openbare tap naar `Formula/` van een Git-repository die je host, wijzig de `url`-regels naar je kopieën van de releasearchieven en werk elke `sha256` bij (het eerste veld van het `.sha256`-bestand naast elk archief). Dan:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

De formule installeert de markering met `channel=homebrew`: GitPoulpi voert zelf `brew upgrade gitpoulpi` uit, dat je tap leest.

## Spiegel van een Docker-registry

De imagenaam is `poulpilab/gitpoulpi`. Het paneel toont altijd die naam, dus een mirror moet transparant zijn.

- **Pull-through-cache:** draai een registry als proxy van Docker Hub (de image `registry` met `proxy.remoteurl: https://registry-1.docker.io`) en zet `"registry-mirrors": ["https://registry.example"]` in de `daemon.json` van de Docker-daemon. `docker pull poulpilab/gitpoulpi` loopt dan via je cache.
- **Kopie:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, en draai daarna die naam. Het paneel toont nog steeds de openbare, wat prima is als verwijzing naar de versie.

Zie [GitPoulpi in Docker draaien](docker.md) voor hoe je hem draait.

## Het beleidsbestand en het register

Het beleid bepaalt hoe updates zich op een computer gedragen en kan niet door gebruikers worden gewijzigd. Het wordt gelezen uit:

| Besturingssysteem | Beleidsbestand |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, ook | de registersleutel `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, één `REG_SZ`-waarde per sleutel hieronder, met dezelfde namen (voor Groepsbeleid, Intune en MSI-eigenschappen) |

Het bestand heeft één tabel:

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

| Sleutel | Waarden | Effect |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Een plafond: `off` < `notify` < `auto`. Gebruikers mogen een lagere waarde kiezen, nooit een hogere. `off` verbergt de controle en doet geen verzoek. |
| `release_channel` | `stable`, `prerelease` | Forceert het updatekanaal. |
| `pin` | `1.4` of `1.4.2` | De hoogste aangeboden versie: `1.4` staat elke `1.4.x` toe, `1.4.2` hoogstens `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Waar wordt gevraagd "wat is de laatste release". Standaard per installatie: GitHub, de Chocolatey-feed voor de Chocolatey-kanalen. `none` controleert nooit. |
| `check_url` | URL | Met `nuget`: de feed. Met `manifest`: het adres van `latest.json`. Met `github`: de basis van de GitHub-API, voor GitHub Enterprise Server (voeg `repo` toe). |
| `repo` | `owner/name` | De repository die wordt gecontroleerd met `check = "github"`. |
| `download_base` | URL | Een mirror van de releasebestanden (dezelfde bestandsnamen) voor de archief- en installatiekanalen. Downloads van deze host zijn toegestaan naast die van GitHub. |
| `allow_self_update` | `true`, `false` | `false` verandert elke actie "vervang de bestanden van GitPoulpi" en "voer de pakketbeheerder uit" in een opmerking of een te kopiëren opdracht. Standaard `true`. |
| `message` | tekst | Getoond in het updatepaneel ("Van je beheerder: ..."). |

**Voorrang**, sterkste eerst: het register, het beleidsbestand, de `[updates]` van de `config.toml` van de gebruiker, de `feed=` van de markering, de ingebouwde standaardwaarden. Een sleutel die het beleid instelt, is **Ingesteld door je beheerder** in de Instellingen en kan daar niet worden bewerkt.

**Veilig falen:** een beleidsbestand of registerwaarde die niet kan worden gelezen (een TOML-syntaxisfout, `mode = "sometimes"`, een `pin` die geen versie is) wordt behandeld als `mode = notify` en `allow_self_update = false`, en één regel gaat naar het toepassingslogboek. Een typefout laat nooit een heel park zichzelf bijwerken. Een onbekende sleutel wordt met een waarschuwing genegeerd. Op Windows leest de stap met beheerdersrechten alleen het beleid van de machine, nooit de config van de gebruiker.

### Voorbeelden

Updates uitzetten (een vergrendelde image, een netwerk zonder internet):

```toml
[updates]
mode = "off"
```

Het park vastpinnen op de 1.4-lijn, alleen melden, en zeggen wie je moet vragen:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Een interne Chocolatey-feed (NuGet) controleren:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Hetzelfde soort beleid op Windows via het register (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json en download_base: een mirror van de release

Kopieer de bestanden van een release (de archieven, de setup, de `.sha256`-bestanden, `latest.json`, **elk `.minisig`-handtekeningbestand** en `security.json`) naar één map van een interne webserver, met behoud van de namen. `latest.json` is bij elke release bijgevoegd:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Het beleid is dan:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

De archief- en installatiekanalen controleren het manifest en downloaden van `download_base`, met dezelfde verificatie als vanaf GitHub. Een mirror hoeft alleen bestanden te kopiëren; hij heeft geen API nodig.

De handtekeningen reizen met de bestanden mee, dus een mirror kan ze niet verzwakken:

- `latest.json` wordt alleen vertrouwd als `latest.json.minisig` ernaast staat en klopt met de in GitPoulpi ingebouwde sleutel van het project. Een manifest zonder zijn handtekening, of gewijzigd nadat het is ondertekend, wordt geweigerd (de hashes erin zijn waartegen de downloads worden gecontroleerd).
- Elk archief wordt samen met `<archief>.minisig` van dezelfde host gedownload en alleen geïnstalleerd als die handtekening klopt en dit bestand en deze versie noemt. Een mirror die het archief maar niet zijn `.minisig` heeft gekopieerd, laat de update mislukken met "no signature", en er wordt niets gewijzigd.
- `security.json` (de beveiligingsmeldingen, zie [Updates](updates.md#beveiligingsmeldingen)) en `security.json.minisig` worden naast het manifest gezocht. Een mirror die ze niet kopieert, toont gewoon geen meldingen.

Kopieer de bestanden ongewijzigd: een handtekening dekt de exacte bytes. De beveiligingsmeldingen worden voor de laatste release opnieuw gepubliceerd zonder nieuwe build, dus ververs `latest.json`, `security.json` en hun `.minisig`-bestanden van de releasepagina telkens als je de mirror ververst.

## Een standaard-config.toml uitrollen

GitPoulpi heeft geen `config.toml` voor de hele machine: instellingen staan in de map van elke gebruiker (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Om nieuwe gebruikers je standaardwaarden te geven, kopieer je een spaarzame `config.toml` (alleen de sleutels die je wilt, zie de [Configuratiereferentie](config-reference.md)) naar die map als de gebruiker er geen heeft (interfacestandaarden zoals `ui.theme` of `ui.language` komen in een spaarzame `tui.toml` ernaast, die de terminal-UI zelf leest; zie [Twee bestanden](config-reference.md#twee-bestanden-configtoml-en-tuitoml)): een aanmeld- of Intune-script op Windows, `/etc/skel/.config/gitpoulpi/` op Linux, een script bij de eerste aanmelding op macOS. Niets in de pakketten van GitPoulpi overschrijft ooit een config, dus die beheer je zelf. Zet daar geen updateregels in als ze moeten gelden: gebruik het beleid, dat een gebruiker niet kan overschrijven. Een draagbare config of een config per taak kan ook worden gekozen met `GITPOULPI_CONFIG_DIR`.
