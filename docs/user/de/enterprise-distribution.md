[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · **Deutsch** · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Unternehmensverteilung

Für Administratoren, die GitPoulpi von eigenen Servern auf viele Computer verteilen: interne Paket-Feeds, Spiegel, eine computerweite Update-Richtlinie und eine Standardkonfiguration. Lesen Sie zuerst [Installation](installation.md) und [Updates](updates.md).

Zwei Tatsachen bestimmen hier alles:

- Jede Installation hält **den Installationsweg** in der Markierung `.gitpoulpi-dist` neben den Programmen fest (`channel=...`). GitPoulpi aktualisiert sich nur auf dem Weg, den dieser Kanal erlaubt: Es ersetzt nie Dateien, die einem Paketmanager gehören, und ein MSI wird nie angetastet.
- Die **Richtlinie** (eine Datei und unter Windows die Registry) gehört Ihnen. Die eigene `config.toml` eines Benutzers kann sie nie lockern.

## Chocolatey mit privatem Feed

Das öffentliche Paket lädt das Installationsprogramm von GitHub herunter. Für ein geschlossenes Netzwerk internalisieren Sie es:

1. Beschaffen Sie das Paket (`choco download gitpoulpi --internalize` mit den lizenzierten Chocolatey-Werkzeugen, oder laden Sie die `.nupkg` herunter und entpacken Sie sie).
2. Lassen Sie in `tools\chocolateyinstall.ps1` `url64bit` auf Ihre eigene Kopie von `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` zeigen und halten Sie `checksum64` gleich dem ersten Feld der `.sha256`-Datei des Releases (oder der Prüfsumme Ihrer Kopie).
3. `choco pack`, dann die `.nupkg` in Ihren Feed hochladen (Nexus, Artifactory, ProGet, eine Dateifreigabe).

Installieren Sie es mit Paketparametern:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parameter | Wirkung |
|---|---|
| `/Channel:choco-internal` | Trägt `channel=choco-internal` in die Markierung ein, damit GitPoulpi weiß, dass der Feed Ihrer ist und nicht der Community-Feed von Chocolatey. |
| `/Feed:<url>` | Trägt die Feed-Adresse ein (`feed=` in der Markierung). GitPoulpi prüft sie auf neue Versionen und führt `choco upgrade gitpoulpi -y --source=<feed>` aus. |

Ohne Parameter ist der Kanal `choco`, und die Prüfung geht an den Community-Feed von Chocolatey. Das Paket installiert für alle Benutzer nach Program Files, ohne `gp`-Alias.

Das Update von GitPoulpi führt `choco upgrade` ohne `--params` aus, damit nichts angetastet wird, was Sie verteilt haben. Chocolatey muss sich daher die Paketparameter der ersten Installation merken (`useRememberedArgumentsForUpgrades`, seit Chocolatey 2.0 standardmäßig an; bei älteren Versionen `choco feature enable -n useRememberedArgumentsForUpgrades`). Sonst würde das Upgrade den einfachen Kanal `choco` eintragen. Ein Richtlinien-`check_url` (unten) hält Prüfung und Upgrade unabhängig von der Markierung auf Ihren Feed gerichtet und hat Vorrang vor dem Feed der Markierung.

**Nichts im Paket schreibt `policy.toml` oder `config.toml`.** Was Sie verteilt haben, bleibt, bei der Installation und bei jedem Upgrade. Siehe [Eine Standard-config.toml verteilen](#eine-standard-configtoml-verteilen).

## Scoop mit privatem Bucket

Legen Sie `gitpoulpi.json` (das Manifest aus dem öffentlichen Bucket) in `bucket/` eines Git-Repositorys, das Sie hosten, ändern Sie `url` auf Ihre Kopie des Release-Zip und setzen Sie `hash` auf dessen SHA-256 (`autoupdate` kann bleiben, wenn Ihre Kopie den Namen der Releases folgt). Dann:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

Das `post_install` des Manifests schreibt die Markierung mit `channel=scoop` neu; GitPoulpi führt dann `scoop update gitpoulpi` selbst aus und tauscht nie Dateien hinter dem Rücken von Scoop aus. Scoop installiert pro Benutzer und braucht keine Administratorrechte.

## apt mit Spiegel

Das Repository ist ein einfacher Verzeichnisbaum (`dists/`, `pool/`, die `gitpoulpi-archive-keyring.gpg` und `.asc` des Signaturschlüssels und die Beispieldatei `gitpoulpi.sources`), der über HTTPS ausgeliefert wird. Spiegeln Sie es mit `wget --mirror`, `rsync` oder Ihrem Artefakt-Manager, liefern Sie es intern aus und installieren Sie Schlüssel und Quelle auf jedem Computer:

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

Behalten Sie den Dateinamen `gitpoulpi.sources` (oder `gitpoulpi.list`) bei: GitPoulpi zeigt `apt`-Befehle nur, wenn es `/etc/apt/sources.list.d/gitpoulpi.*` findet. Verwenden Sie den Schlüssel des Projekts: Der Spiegel bleibt damit signiert, und `Signed-By` legt genau diesen einen Schlüssel nur für dieses Repository fest. Das Repository behält die letzten fünf Versionen jedes Pakets.

**Offline-`.deb`:** Jedes Release hängt `gitpoulpi_<version>_<arch>.deb` an. Kopieren Sie es hinüber und führen Sie `sudo apt install ./gitpoulpi_<version>_<arch>.deb` aus. Die Markierung sagt `channel=apt`; ohne die Quelldatei sagt GitPoulpi dem Benutzer, dass er stattdessen die nächste `.deb` herunterladen soll, statt einen `apt`-Befehl zu zeigen.

## MSI mit Intune oder Gruppenrichtlinie

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` installiert für alle Benutzer und ist zur zentralen Verwaltung gedacht: Das Fenster von GitPoulpi sagt nur „von Ihrem Administrator verwaltet“ (mit Ihrer `message`), es aktualisiert eine MSI-Installation nie. Aktualisieren Sie, indem Sie das neue MSI verteilen; es ist ein Hauptupgrade, die alte Version wird also ersetzt.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Eigenschaft | Wirkung |
|---|---|
| `INSTALLFOLDER` | Installationsordner (Standard `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (Standard) fügt den Ordner dem PATH des Computers hinzu, `0` nicht. |
| `UPDATES_MODE` | Richtlinie `mode`: `off`, `notify` oder `auto`. |
| `UPDATES_PIN` | Richtlinie `pin`, zum Beispiel `0.2` oder `0.2.4`. |
| `UPDATES_CHECK` | Richtlinie `check`: `github`, `nuget`, `manifest` oder `none`. |
| `UPDATES_CHECK_URL` | Richtlinie `check_url`. |
| `UPDATES_MESSAGE` | Richtlinie `message`, im Update-Fenster angezeigt. |

Eine Eigenschaft schreibt ihren Wert nur dann nach `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, wenn Sie sie setzen; eine nicht gesetzte lässt die Registry (und eine Richtliniendatei) in Ruhe. **Intune:** Verteilen Sie das MSI als Branchen-App oder Win32-App mit dem obigen Befehl. **Gruppenrichtlinie:** Die Softwareinstallation kann keine Eigenschaften übergeben, verwenden Sie daher entweder eine Transformation (`.mst`) oder setzen Sie die Registry-Werte mit Group Policy Preferences, siehe unten.

## Homebrew mit internem Tap

Kopieren Sie `gitpoulpi.rb` aus dem öffentlichen Tap nach `Formula/` eines Git-Repositorys, das Sie hosten, ändern Sie die `url`-Zeilen auf Ihre Kopien der Release-Archive und aktualisieren Sie jedes `sha256` (das erste Feld der `.sha256`-Datei neben jedem Archiv). Dann:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Die Formel installiert die Markierung mit `channel=homebrew`: GitPoulpi führt `brew upgrade gitpoulpi` selbst aus, was Ihren Tap liest.

## Docker-Registry-Spiegel

Der Image-Name ist `poulpilab/gitpoulpi`. Das Fenster zeigt immer diesen Namen, ein Spiegel muss also transparent sein.

- **Pull-Through-Cache:** Betreiben Sie eine Registry als Proxy von Docker Hub (das Image `registry` mit `proxy.remoteurl: https://registry-1.docker.io`) und setzen Sie `"registry-mirrors": ["https://registry.example"]` in der `daemon.json` des Docker-Daemons. `docker pull poulpilab/gitpoulpi` läuft dann über Ihren Cache.
- **Kopie:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, dann diesen Namen ausführen. Das Fenster zeigt weiterhin den öffentlichen, was als Hinweis auf die Version in Ordnung ist.

Wie es ausgeführt wird, steht unter [In Docker ausführen](docker.md).

## Die Richtliniendatei und die Registry

Die Richtlinie legt fest, wie sich Updates auf einem Computer verhalten, und lässt sich von Benutzern nicht ändern. Sie wird gelesen aus:

| Betriebssystem | Richtliniendatei |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, außerdem | der Registry-Schlüssel `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, ein `REG_SZ`-Wert je Schlüssel unten, mit denselben Namen (für Gruppenrichtlinie, Intune und MSI-Eigenschaften) |

Die Datei hat eine Tabelle:

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

| Schlüssel | Werte | Wirkung |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Eine Obergrenze: `off` < `notify` < `auto`. Benutzer dürfen einen niedrigeren Wert wählen, nie einen höheren. `off` blendet die Prüfung aus und stellt keine Anfrage. |
| `release_channel` | `stable`, `prerelease` | Erzwingt den Update-Kanal. |
| `pin` | `1.4` oder `1.4.2` | Die höchste angebotene Version: `1.4` erlaubt jedes `1.4.x`, `1.4.2` erlaubt höchstens `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Wo nach dem neuesten Release gefragt wird. Standard je Installation: GitHub, bei den Chocolatey-Kanälen der Chocolatey-Feed. `none` prüft nie. |
| `check_url` | URL | Bei `nuget`: der Feed. Bei `manifest`: die Adresse der `latest.json`. Bei `github`: die GitHub-API-Basis, für GitHub Enterprise Server (`repo` ergänzen). |
| `repo` | `owner/name` | Das mit `check = "github"` geprüfte Repository. |
| `download_base` | URL | Ein Spiegel der Release-Dateien (gleiche Dateinamen) für die Archiv- und Installer-Kanäle. Downloads von diesem Host sind zusätzlich zu denen von GitHub erlaubt. |
| `allow_self_update` | `true`, `false` | `false` macht aus jeder Aktion „GitPoulpis Dateien ersetzen“ und „den Paketmanager ausführen“ einen Hinweis oder einen Befehl zum Kopieren. Standard `true`. |
| `message` | Text | Wird im Update-Fenster angezeigt („Von Ihrem Administrator: ...“). |

**Rangfolge**, die stärkste zuerst: die Registry, die Richtliniendatei, die `[updates]` der `config.toml` des Benutzers, das `feed=` der Markierung, die eingebauten Standardwerte. Ein Schlüssel, den die Richtlinie setzt, ist in den Einstellungen **Vom Administrator festgelegt** und lässt sich dort nicht bearbeiten.

**Geschlossen fehlschlagen:** Eine Richtliniendatei oder ein Registry-Wert, der sich nicht lesen lässt (ein TOML-Syntaxfehler, `mode = "sometimes"`, ein `pin`, der keine Version ist), wird wie `mode = notify` und `allow_self_update = false` behandelt, und eine Zeile geht ins Anwendungsprotokoll. Ein Tippfehler lässt nie eine ganze Flotte sich selbst aktualisieren. Ein unbekannter Schlüssel wird mit einer Warnung ignoriert. Unter Windows liest der Schritt mit Administratorrechten nur die Richtlinie des Computers, nie die Konfiguration des Benutzers.

### Beispiele

Updates ausschalten (ein gesperrtes Image, ein vom Internet getrenntes Netzwerk):

```toml
[updates]
mode = "off"
```

Die Flotte auf die Linie 1.4 festlegen, nur hinweisen und sagen, wen man fragen soll:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Einen internen Chocolatey-Feed (NuGet) prüfen:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Dieselbe Art Richtlinie unter Windows über die Registry (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json und download_base: ein Spiegel des Releases

Kopieren Sie die Dateien eines Releases (die Archive, das Setup, die `.sha256`-Dateien, `latest.json`, **jede `.minisig`-Signaturdatei** und `security.json`) in einen Ordner eines internen Webservers und behalten Sie die Namen bei. `latest.json` hängt an jedem Release:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Dann lautet die Richtlinie:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Archiv- und Installer-Kanäle prüfen das Manifest und laden von `download_base` herunter, mit derselben Prüfung wie von GitHub. Ein Spiegel muss nur Dateien kopieren; er braucht keine API.

Die Signaturen reisen mit den Dateien, ein Spiegel kann sie also nicht abschwächen:

- `latest.json` wird nur dann vertraut, wenn `latest.json.minisig` daneben liegt und gegen den in GitPoulpi eingebauten Schlüssel des Projekts geprüft werden kann. Ein Manifest ohne seine Signatur oder nach der Signierung bearbeitet wird abgelehnt (die Hashes darin sind das, wogegen die Downloads geprüft werden).
- Jedes Archiv wird zusammen mit `<archive>.minisig` vom selben Host heruntergeladen und nur installiert, wenn diese Signatur geprüft werden kann und diese Datei und diese Version nennt. Ein Spiegel, der das Archiv, aber nicht seine `.minisig` kopiert hat, lässt das Update mit „keine Signatur“ fehlschlagen, und nichts wird geändert.
- `security.json` (die Sicherheitshinweise, siehe [Updates](updates.md#sicherheitshinweise)) und `security.json.minisig` werden neben dem Manifest gesucht. Ein Spiegel, der sie nicht kopiert, zeigt einfach keine Hinweise.

Kopieren Sie die Dateien unverändert: Eine Signatur deckt genau diese Bytes ab. Die Sicherheitshinweise werden für das neueste Release ohne neuen Build neu veröffentlicht, aktualisieren Sie daher `latest.json`, `security.json` und deren `.minisig`-Dateien von der Release-Seite, wann immer Sie den Spiegel aktualisieren.

## Eine Standard-config.toml verteilen

GitPoulpi hat keine computerweite `config.toml`: Die Einstellungen liegen im Ordner jedes Benutzers (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Um neuen Benutzern Ihre Standardwerte zu geben, kopieren Sie eine schlanke `config.toml` (nur die gewünschten Schlüssel, siehe die [Konfigurationsreferenz](config-reference.md)) in diesen Ordner, wenn der Benutzer keine hat (Oberflächenstandards wie `ui.theme` oder `ui.language` gehören in eine schlanke `tui.toml` daneben, die die Terminal-UI von selbst liest; siehe [Zwei Dateien](config-reference.md#zwei-dateien-configtoml-und-tuitoml)): ein Anmelde- oder Intune-Skript unter Windows, `/etc/skel/.config/gitpoulpi/` unter Linux, ein Skript bei der ersten Anmeldung unter macOS. Nichts in den Paketen von GitPoulpi überschreibt je eine Konfiguration, sie bleibt also Ihre Sache. Legen Sie dort keine Update-Regeln ab, wenn sie verbindlich sein müssen: Verwenden Sie die Richtlinie, die ein Benutzer nicht überschreiben kann. Eine portable oder je Auftrag gültige Konfiguration lässt sich auch mit `GITPOULPI_CONFIG_DIR` wählen.
