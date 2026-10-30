[English](../installation.md) · [Čeština](../cs/installation.md) · **Deutsch** · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi besteht aus drei ausführbaren Dateien:

| Programm | Was es ist |
|---|---|
| `gitpoulpi-tui` | Die Terminal-Oberfläche. |
| `gitpoulpi` | Die Kommandozeile (`--json` für Skripte). Ohne Unterbefehl öffnet sie die Oberfläche. |
| `gitpoulpi-mcp` | Der MCP-Server für KI-Agenten. Siehe [MCP und Agenten](mcp-and-agents.md). |

Neben diesen enthält das Archiv `gitpoulpi-launch`, ein winziges Programm, das die Verknüpfungen ausführen, um ein bereits geöffnetes GitPoulpi-Fenster zu zeigen (`gitpoulpi launch`). Unter Windows hat es kein Konsolenfenster, ein Klick lässt also nie eines aufblitzen. Es wird mit den anderen installiert, aktualisiert und entfernt.

## Voraussetzungen

- Ein **System-`git`** im `PATH`. GitPoulpi nutzt libgit2 für die meisten
  Vorgänge und Ihr `git` für den Rest.
  Erforderlich ist Version **2.34 oder neuer**. Unter macOS funktioniert das `git` aus den Command Line Tools von Apple ebenso wie das von Homebrew.
- Ein **Terminal** mit Mausunterstützung. kitty, Ghostty, WezTerm und iTerm2 (und Windows Terminal und foot über Sixel)
  zeichnen den Commit-Graphen als Bild. Andere Terminals (die
  Windows-Konsole, tmux und die meisten übrigen) zeigen einen Zeichen-Graphen.
  Siehe [Ein Terminal wählen](terminals.md).
- Eine **Nerd Font** oder eine Schrift mit Emoji, für die Symbole in der
  Symbolleiste und der Seitenleiste.
- Optional: **`git-lfs`** ab Version 3.0 für Git-LFS-Repositories. Ohne es
  erscheinen LFS-Dateien als kleine Zeigerdateien, und GitPoulpi erklärt
  Ihnen den Grund. Zur Installation siehe [Git LFS](lfs.md).
- Optional: ein **Credential-Helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), um Hosting-Tokens und KI-API-Schlüssel zu speichern.
  Siehe [Hosting und Issue-Tracker](hosting-integration.md).

## Aus einem Release-Archiv

Ein Release-Archiv heißt `gitpoulpi-<version>-<target>.zip` (Windows) oder
`.tar.gz` (Linux, macOS) und liegt neben einer `.sha256`-Datei. Prüfen Sie das
Archiv anhand der `.sha256`-Datei, entpacken Sie es und legen Sie die drei
Programme in einen Ordner im `PATH`. `INSTALL.txt` im Archiv nennt dieselben
Schritte.

## Mit einem Installationsprogramm

Jedes Release enthält außerdem ein Installationsprogramm je Betriebssystem. Beide treffen dieselben Entscheidungen
(Verknüpfungen, PATH, ein Befehlsalias wie `gp`), indem sie das installierte `gitpoulpi` selbst aufrufen, also den Code
hinter Einstellungen > Allgemein > Starter. Was das Installationsprogramm einrichtet und was die Einstellungen zeigen,
stimmt deshalb immer überein.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Es installiert für den aktuellen Benutzer, ohne Administrator-Abfrage, nach `%LOCALAPPDATA%\Programs\GitPoulpi` (der
erste Dialog bietet eine Installation für alle Benutzer und die Wahl des Installationsordners an). Kontrollkästchen:
Verknüpfung im Startmenü, Verknüpfung auf dem Desktop, zum PATH hinzufügen. Eine Seite fragt nach dem **Befehlsalias**:
standardmäßig `gp`, ein beliebiger Name, oder leer für keinen. PowerShell belegt `gp` bereits für `Get-ItemProperty`,
wählen Sie dort also einen anderen Namen oder nutzen Sie `gitpoulpi`.

Das Installationsprogramm kopiert die drei Programme und die Markierung `.gitpoulpi-dist`, sodass
[Jetzt aktualisieren](#sich-selbst-aktualisieren) genauso funktioniert wie bei der ZIP-Datei. Eine Installation für den
einzelnen Benutzer ist beschreibbar und kann sich daher selbst aktualisieren; eine Installation für alle Benutzer unter
Programme braucht zum Ersetzen erhöhte Rechte, installieren Sie sie also mit dem neuen Setup erneut. Deinstalliert wird
über die Windows-Einstellungen > Apps: Dort läuft `gitpoulpi uninstall` (Alias, PATH-Einträge, Verknüpfungen), und die
Dateien werden gelöscht, auch die `.old`-Reste des Updaters. Ihre Einstellungen und Repositories bleiben.

### Linux und macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Es wählt das Archiv für Ihr Betriebssystem und Ihre CPU, prüft dessen `.sha256`, installiert nach
`~/.local/opt/gitpoulpi` und verlinkt `gitpoulpi` und `gitpoulpi-mcp` nach `~/.local/bin`. Root wird nicht gebraucht,
und außerhalb Ihres Home-Ordners wird nichts angefasst. Ohne `--yes` fragt es nach dem Alias (Standard `gp`), dem PATH
und den Verknüpfungen.

| Option | Wirkung |
|---|---|
| `--prefix DIR` | Installiert die Programmdateien in `DIR` (absolut). |
| `--version X.Y.Z` | Installiert dieses Release statt des neuesten. |
| `--archive FILE` | Nutzt ein lokales Archiv; `FILE.sha256` muss daneben liegen (`--no-verify` überspringt das). Das Netzwerk wird nicht benutzt. |
| `--alias NAME` / `--no-alias` | Lässt `NAME` GitPoulpi starten (`gitpoulpi alias create`), oder keinen Namen. |
| `--no-path` | Fügt GitPoulpi nicht zum PATH neuer Shells hinzu (sonst `gitpoulpi path add`). |
| `--desktop` | Legt zusätzlich die Verknüpfungen im Anwendungsmenü und auf dem Desktop an. |
| `--yes` | Fragt nie: Die Optionen und die Standardwerte gelten (Alias `gp`, PATH ja, Verknüpfungen nein). |
| `--uninstall` | Führt `gitpoulpi uninstall` aus und entfernt dann die Links und den Ordner. |

Die Suche nach dem neuesten Release setzt ein öffentliches Repository voraus; bei einem privaten laden Sie das Archiv im
Browser herunter und nutzen `--archive`.


## Aus dem Quellcode

Sie brauchen eine aktuelle stabile Rust-Toolchain (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Die Programme liegen in `target/release/`. Das Profil `release` baut schnell.
Für die kleinsten und schnellsten Programme nehmen Sie das Profil `dist`; es
braucht länger beim Bauen:

```
cargo build --profile dist
```

Die `build-opti`-Skripte im Wurzelverzeichnis des Repositorys führen diesen
Build für die drei Programme aus und geben ihre Größen aus:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd oder PowerShell)
```

Um nur die Sprachen und Funktionen zu bauen, die Sie nutzen, siehe
[Eigene Builds](custom-builds.md).

## Ein Release-Archiv selbst bauen

```
cargo run -p xtask -- dist
```

Das baut das Profil `dist` und schreibt
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) oder `.tar.gz`
(andere Systeme). Das Archiv enthält die drei Programme, `INSTALL.txt`,
`README.md`, `LICENSE` und `CHANGELOG.md`. Daneben entsteht eine
`.sha256`-Datei.

## Updates

Die Titelleiste zeigt die laufende Version (`v1.0.0`) zwischen dem Programmnamen und dem Repository-Pfad. Ein Klick darauf oder
**Hilfe: Versionshinweise** in der Befehlspalette (`Ctrl+P`) öffnet einen Tab Versionshinweise mit dem Änderungsprotokoll,
gescrollt bis zur laufenden Version.

Einmal täglich fragt GitPoulpi im Hintergrund bei GitHub nach, ob es ein neueres Release gibt (`updates.mode`,
[Einstellungen > Synchronisierung](settings.md#synchronisierung)). Gibt es eines, steht auf dem Chip `v0.2.0 verfügbar` in
der Akzentfarbe, und ein Klick darauf öffnet den Tab Versionshinweise mit den Hinweisen dieses Releases ganz oben. Der Tab
hat drei Schaltflächen, jede mit einer Taste und einem Palettenbefehl:

- **Herunterladen** (`d`, *Updates: Herunterladen*) öffnet die Release-Seite in Ihrem Browser.
- **Version überspringen** (`s`, *Updates: Diese Version überspringen*) blendet den Hinweis aus, bis ein neueres Release
  erscheint.
- **Jetzt prüfen** (`r`, *Updates: Jetzt nach Updates suchen*) fragt sofort noch einmal nach.

`updates.mode = "off"` stellt gar keine Anfrage. Ohne Netzwerkverbindung oder solange das Repository privat ist und kein
Token für github.com gespeichert ist, scheitert die Prüfung leise (eine Zeile im Anwendungsprotokoll, nichts in der
Oberfläche). `GITPOULPI_UPDATE_API` richtet die Prüfung auf eine andere API-Wurzel (von den Tests verwendet).

### Sich selbst aktualisieren

Ein aus einem Release-Archiv installierter Build kann sich selbst ersetzen. Das Archiv enthält neben den Programmen eine
kleine Datei `.gitpoulpi-dist`; ein laufender Build ist nur dann „aktualisierbar“, wenn diese Datei neben seiner
ausführbaren Datei liegt (und seine Plattform nennt). Ein mit `cargo` gebauter Build, ein eigener Build, der anderswohin
kopiert wurde, oder ein Paket, das die Datei weggelassen hat, aktualisiert sich nie selbst: Sein Tab Versionshinweise
sagt das und bietet nur **Herunterladen** an.

Bei einem aktualisierbaren Build kommt auf dem Tab Folgendes hinzu:

- **Jetzt aktualisieren** (`u`, *Updates: Jetzt aktualisieren*) lädt das Archiv und seine `.sha256` im Hintergrund herunter
  (ein Job in der Anzeige der Statuszeile, mit Prozentangabe, abbrechbar wie jeder Job), prüft den Hash, entpackt das
  Archiv und legt jede neue Datei neben die alte als `<Name>.new`. Nichts Laufendes wird angefasst. Im Modus `auto`
  geschieht dasselbe von selbst, sobald eine Prüfung ein neueres Release findet.
- **Für Update neu starten** (`R`, *Updates: Für Update neu starten*; der Chip sagt dasselbe, sobald der Download bereit
  liegt) ist immer Ihre Aktion, nie automatisch. Es beendet den eigenständigen Daemon der Sitzung (ein eingebetteter geht
  mit dem Fenster), tauscht die Dateien aus und startet die neue Version mit denselben Argumenten und derselben Sitzung.
- **Zurück zur vorherigen Version** (`v`, *Updates: Zur vorherigen Version zurückkehren*) tauscht die vorherigen Dateien zurück und
  startet neu. Die Dateien der vorherigen Version bleiben als `<Name>.old` liegen, bis die neue Version eine Minute lang
  lief, und werden dann gelöscht; die Schaltfläche gibt es also für diesen ersten Lauf. Sie gehört nicht zum
  Rückgängig-Journal von Git: Sie ändert die eigenen Dateien von GitPoulpi, kein Repository.

Wie die Dateien ersetzt werden: Unter Linux und macOS wird jede neue Datei über die alte umbenannt (atomar; ein laufendes
Programm behält seine alte Kopie, bis es endet). Unter Windows lässt sich eine laufende `.exe` nicht überschreiben, daher
wird die alte in `<Name>.old` umbenannt und die neue hineinverschoben; eine `.old`, die noch läuft, löscht ein späterer
Start. Der Austausch gelingt ganz oder gar nicht: Hält ein anderes Programm eine der ausführbaren Dateien (etwa ein
MCP-Client, der `gitpoulpi-mcp` ausführt), nennt GitPoulpi die Datei, ändert nichts und lässt den Download für einen
neuen Versuch bereitliegen.

Was abgelehnt wird, jeweils mit einer Meldung: ein Release ohne `.sha256`-Datei, ein Hash, der nicht passt, ein Archiv, dessen
Dateiname nicht der für diese Plattform ist (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), ein Archiv, dessen Markierung
eine andere Version oder Plattform nennt, und jeder Download-Link außerhalb dieser Hosts: `github.com`,
`api.github.com`, `objects.githubusercontent.com`, `release-assets.githubusercontent.com` und
`github-releases.githubusercontent.com` von GitHub (die erste Anfrage und jede Weiterleitung; eine Weiterleitung
anderswohin beendet den Download). Die Archive eines privaten Repositorys brauchen eine Browser-Sitzung, der Download
scheitert dort also, bis das Repository öffentlich ist.


## Prüfen, ob es funktioniert

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` gibt die Version und den Commit aus, aus dem der Build entstand.
Weiter geht es mit [Erste Schritte](getting-started.md).

## Deinstallation

`gitpoulpi uninstall` (das führen beide Installationsprogramme aus) entfernt die von GitPoulpi angelegten
Befehlsaliasse, PATH-Einträge und Verknüpfungen und danach die Programmdateien einer Release-Installation; bei einem Build
ohne die Markierung `.gitpoulpi-dist` (ein cargo-Build) lehnt es ab, und `--keep-files` lässt die Programmdateien stehen.
Für MCP-Agenten ist es nicht verfügbar. Ohne Installationsprogramm löschen Sie die drei Programme. GitPoulpi legt außerdem
diese Dateien an:

| Was | Wo |
|---|---|
| Einstellungen (`config.toml`) und Sitzungen | Der Konfigurationsordner Ihres Betriebssystems. `GITPOULPI_CONFIG_DIR` ändert ihn. |
| Zustand pro Repository | `.gitpoulpi/` in jedem Repository. |
| Caches (Avatare) | Der Cache-Ordner Ihres Betriebssystems. |
| Tokens und API-Schlüssel | Ihr Credential-Helper. **Einstellungen > SSH & Zugangsdaten > Alle Benutzernamen und Passwörter vergessen** entfernt sie. |

Die genauen Pfade stehen in der [Konfigurationsreferenz](config-reference.md).
