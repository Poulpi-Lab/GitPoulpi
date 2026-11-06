[English](../updates.md) · [Čeština](../cs/updates.md) · **Deutsch** · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Updates

GitPoulpi sucht nach neueren Releases und ersetzt je nach Installationsweg entweder seine eigenen Dateien oder übergibt das Update an das Programm, das es installiert hat. Nichts wird ohne Ihr Zutun ersetzt oder neu gestartet.

## Der Chip und das Fenster

Die Titelleiste zeigt die laufende Version (`v1.0.0`) zwischen dem App-Namen und dem Repository-Pfad. Ist ein neueres Release bekannt, wird der Chip in der Akzentfarbe zu `v0.2.0 verfügbar`.

Klicken Sie auf den Chip, drücken Sie `Ctrl+N` und dann `u`, oder führen Sie **Updates: Update-Fenster anzeigen** in der Befehlspalette (`Ctrl+P`) aus: Das Update-Fenster öffnet sich. Es liegt als Fenster über der App: `Esc`, `q`, das Schließen-Kreuz oder ein Klick außerhalb schließt es. Es zeigt:

- die laufende Version und **Installiert über**: wie GitPoulpi auf diesen Computer gekommen ist (das bestimmt alles Weitere);
- den Stand des Updates (Suche, Download mit Prozentangabe, bereit, fehlgeschlagen);
- die Schaltflächen des Verfahrens dieser Installation und, falls es einen gibt, den genauen Befehl;
- eine Zeile Ihres Administrators, wenn er eine hinterlegt hat.

Jede Schaltfläche hat eine Taste (in der Hinweiszeile des Fensters aufgeführt) und einen Palettenbefehl, das Fenster funktioniert also ohne Maus:

| Schaltfläche | Taste | Palettenbefehl |
|---|---|---|
| **Aktualisieren und neu starten** (oder **Update ausführen**) | `u` | Updates: Jetzt aktualisieren |
| **Befehl kopieren** | `c` | Updates: Update-Befehl kopieren |
| **Im Terminal öffnen** | `t` | Updates: Update-Befehl in einem Terminal ausführen |
| **Zurück** | `v` | Updates: Zur vorherigen Version zurückkehren |
| **Versionshinweise** | `n` | Hilfe: Versionshinweise |
| **Version überspringen** | `s` | Updates: Diese Version überspringen |
| **Hinweis öffnen** (nur bei einem Sicherheitshinweis) | `a` | Updates: Sicherheitshinweis öffnen |
| **Jetzt prüfen** | `r` | Updates: Jetzt nach Updates suchen |
| **Schließen** | `Esc` | |

Ein Befehl, der für diese Installation nicht gilt, ist in der Palette ausgegraut und nennt den Grund (zum Beispiel „diese Installation wird auf anderem Weg aktualisiert: siehe Updates: Update-Fenster anzeigen“).

## Was je Installation passiert

| Installiert über | Prüfung bei | Was das Fenster tut | Rückweg |
|---|---|---|---|
| Ein Release-Archiv oder `install.sh` | GitHub-Releases | **Aktualisieren und neu starten**: GitPoulpi lädt das Archiv herunter und prüft es, schließt sich, ersetzt seine Dateien und öffnet sich im selben Terminal neu. | **Zurück** tauscht die vorherigen Dateien wieder ein. |
| Das Windows-Installationsprogramm, nur für mich | GitHub-Releases | Dasselbe, ohne Rückfrage. Unter Windows öffnet es sich in einem neuen Fenster neu. | **Zurück** tauscht zurück. |
| Das Windows-Installationsprogramm, alle Benutzer | GitHub-Releases | Dasselbe, aber Windows verlangt die Zustimmung eines Administrators. GitPoulpi-Sitzungen anderer Benutzer auf dem Computer werden beendet, was deren ungespeicherte Eingaben verliert. Wenn Sie ablehnen, ändert sich nichts. | **Zurück** tauscht zurück, mit derselben Abfrage. |
| Chocolatey | Der Community-Feed von Chocolatey | **Update ausführen** führt `choco upgrade gitpoulpi -y` aus (Windows verlangt die Zustimmung eines Administrators). | Das Fenster zeigt `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, interner Feed | Der Feed Ihrer Organisation | **Update ausführen** führt `choco upgrade gitpoulpi -y --source=<feed>` aus. Paketparameter werden nicht erneut gesendet, eine verteilte Konfiguration bleibt also unberührt. | Dieselbe `choco install`-Zeile mit dem Feed. |
| winget | GitHub-Releases | **Update ausführen** führt `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` mit akzeptierten Vereinbarungen aus. winget verlangt die Zustimmung eines Administrators selbst, wenn die Installation für den ganzen Computer gilt. | Das Fenster zeigt `winget install --id GitPoulpi.GitPoulpi --version <previous>`. |
| Scoop | GitHub-Releases | **Update ausführen** führt `scoop update gitpoulpi` aus, nachdem GitPoulpi vollständig geschlossen wurde. | **Zurück** führt `scoop reset gitpoulpi@<previous>` aus. |
| Homebrew | GitHub-Releases | **Update ausführen** führt `brew upgrade gitpoulpi` in diesem Terminal aus. | Das Fenster zeigt `brew install gitpoulpi@<previous>`. |
| apt | GitHub-Releases | Das Fenster zeigt `sudo apt update && sudo apt install --only-upgrade gitpoulpi` mit **Befehl kopieren** und **Im Terminal öffnen**. GitPoulpi führt nie `sudo` für Sie aus. Ohne das apt-Repository von GitPoulpi auf diesem Computer steht dort, dass stattdessen das neue `.deb` heruntergeladen werden soll. | Das Fenster zeigt `sudo apt install gitpoulpi=<previous>`. |
| Ein MSI-Paket | GitHub-Releases oder die Quelle Ihres Administrators | Nichts auszuführen: Die Installation wird von Ihrem Administrator verwaltet, dessen Nachricht das Fenster zeigt. | Nicht angeboten. |
| Das AUR | GitHub-Releases | Das Fenster zeigt `yay -S gitpoulpi-bin` (oder `paru`, je nachdem, was es zuerst findet; ohne Helfer die Schritte `git clone` und `makepkg -si`). | Nicht angeboten. |
| cargo | GitHub-Releases | Das Fenster zeigt `cargo binstall gitpoulpi-cli`, wenn `cargo-binstall` installiert ist, sonst `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | Derselbe Befehl mit dem vorherigen Tag. |
| Ein Docker-Image | GitHub-Releases | Nichts auszuführen: Ein Image wird ersetzt. Das Fenster zeigt `docker pull poulpilab/gitpoulpi:<version>` und `:latest`. Siehe [In Docker ausführen](docker.md). | Das Fenster zeigt `docker pull poulpilab/gitpoulpi:<previous>`. |
| Ein Release-Archiv in einem Ordner ohne Schreibrecht | GitHub-Releases | Das Fenster sagt, dass GitPoulpi sich dort nicht ersetzen kann, und verweist auf **Herunterladen**. | Nicht angeboten. |
| Ein Entwicklungs-Build (`cargo build`) | keine | Aktualisiert nie. Der Chip zeigt nur die Version; das Fenster sagt, dass es keinen Installationseintrag gibt. | Nicht angeboten. |

Zeilen mit „Das Fenster zeigt“ führen von sich aus nie etwas aus. **Im Terminal öffnen** öffnet Ihr Terminal mit dem eingetippten Befehl, und Sie drücken Enter.

Releases kommen immer aus dem GitHub-Repository des Projekts, außer ein Administrator lenkt die Prüfung um (ein interner Feed, ein Spiegel oder eine `latest.json`), siehe [Unternehmensverteilung](enterprise-distribution.md). Die Versionshinweise stammen aus dem Release oder aus dem in GitPoulpi eingebauten Änderungsprotokoll, je nachdem, was erreichbar ist.

## Hinweisen und Automatisch

`updates.mode` ([Einstellungen > Synchronisierung](settings.md#synchronisierung)):

- **Hinweisen** (Standard): GitPoulpi prüft einmal täglich im Hintergrund und zeigt den Chip. Es wird nichts heruntergeladen, bis Sie die Schaltfläche drücken.
- **Automatisch**: Wo GitPoulpi seine eigenen Dateien ersetzt (Release-Archiv, das Windows-Installationsprogramm), lädt es das Update auch im Hintergrund herunter und prüft es, sobald es eines findet. Der Chip lautet dann **Für Update neu starten**, und der Neustart bleibt Ihre Entscheidung. Wo ein Paketmanager das Update besitzt, verhält sich **Automatisch** wie **Hinweisen**.
- **Aus**: Es wird nie eine Anfrage gestellt, und der Chip zeigt nur die Version.

Die Einstellung **Update-Kanal** (`updates.channel`) wählt zwischen **Stabil** und **Vorabversion** (auch Release Candidates). Eine Prüfung, die Sie mit **Jetzt prüfen** starten, ignoriert die Begrenzung auf einmal täglich. Ohne Netzwerkverbindung oder solange das Repository privat ist und kein github.com-Token gespeichert ist, schlägt eine Hintergrundprüfung stillschweigend fehl und schreibt eine Zeile ins Anwendungsprotokoll.

## Neustart und was Sie danach sehen

**Aktualisieren und neu starten** oder **Update ausführen** speichert Ihre Sitzung und schließt das Fenster. Ein eigenes kleines Programm, `gitpoulpi-updater`, erledigt die Arbeit, sodass die Dateien von GitPoulpi beim Ersetzen nicht in Benutzung sind und ein Paketmanager nicht durch ein wartendes Elternprogramm blockiert wird. Es beendet zuerst die Hintergrund-Daemons von GitPoulpi (die anderen Fenster und MCP-Server Ihres Benutzers schließen sich ebenfalls), führt das Update aus, prüft, dass das neue `gitpoulpi --version` die erwartete Version ausgibt, und startet GitPoulpi wieder: unter Linux und macOS im selben Terminal, unter Windows in einem neuen Fenster. Schlägt die Prüfung fehl, stellt es die alten Dateien wieder her.

Der nächste Start zeigt eine Zeile in der Statuszeile:

- `Auf v1.4.2 aktualisiert`
- `Update fehlgeschlagen: <Grund>. Die vorherige Version ist weiterhin installiert.`
- `Update abgebrochen: Die Zustimmung des Administrators wurde verweigert. Nichts wurde geändert.`

Es läuft immer nur ein Update; eine Sperrdatei im Datenordner von GitPoulpi (und eine im Installationsordner, wenn er beschreibbar ist) hält ein zweites fern, und eine Sperre, die älter als zehn Minuten ist, wird ignoriert, sodass ein abgestürztes Update GitPoulpi nie dauerhaft sperrt. Die CLI und der Daemon starten nicht, solange eine frische Sperre besteht.

## Zurück zur vorherigen Version

Wo GitPoulpi seine eigenen Dateien ersetzt, tauscht **Zurück** die vorherigen Dateien wieder ein und startet neu. Sie bleiben nach dem Start der neuen Version noch eine Minute als `<name>.old` erhalten und werden dann gelöscht, der Rückweg ist also für den ersten Start nach einem Update gedacht. Wo ein Paketmanager die Installation besitzt, ist der Rückweg der Befehl aus der Tabelle: Das Fenster zeigt oder führt ihn aus. Das Zurücksetzen ändert die eigenen Dateien von GitPoulpi, kein Repository, und gehört daher nicht zum Git-Rückgängig-Journal.

## Was geprüft und abgelehnt wird

Bevor etwas ersetzt wird, lädt GitPoulpi das Archiv, seine `.minisig`-Signatur und seine `.sha256` herunter. Zuerst prüft es die Signatur gegen den in GitPoulpi eingebauten Release-Schlüssel des Projekts, und der vertrauenswürdige Kommentar der Signatur muss dieses Archiv und diese Version nennen. Dann prüft es den Hash. Abgelehnt werden: ein Release ohne `.minisig` (ein älteres Release, das veröffentlicht wurde, bevor Releases signiert wurden, lässt sich nicht über den Updater installieren, auch nicht durch Festlegen auf eine solche Version oder Zurückgehen dorthin), eine Signatur eines anderen Schlüssels, eine Signatur für eine andere Datei oder eine andere Version, ein Release ohne `.sha256`, ein Hash, der nicht passt, ein Archiv, dessen Name nicht der für diese Plattform ist (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), ein Archiv, dessen Markierung eine andere Version oder Plattform nennt, und ein Download-Link außerhalb der Release-Hosts von GitHub (jeder Weiterleitungsschritt wird geprüft) oder des Spiegels, den ein Administrator eingerichtet hat. Der Austausch gilt ganz oder gar nicht: Hält ein anderes Programm eine der ausführbaren Dateien, nennt GitPoulpi die Datei, ändert nichts und lässt den Download für einen weiteren Versuch liegen. Unter Windows wird ein Update für alle Benutzer vom Schritt mit erhöhten Rechten erneut geprüft, der die `.minisig` und die `.sha256` selbst herunterlädt und beide noch einmal prüft, sodass eine in der Zwischenzeit im Download-Ordner des Benutzers untergeschobene Datei abgelehnt wird. Paketmanager (Chocolatey, winget, Scoop, Homebrew, apt, AUR) prüfen ihre Pakete selbst: GitPoulpi lädt dort nichts herunter.

## Sicherheitshinweise

Neben jedem Release veröffentlicht das Projekt `security.json`: die Liste der Sicherheitshinweise (eine Kennung, ein Schweregrad, die betroffenen Versionen als Bereich wie `>=1.2.0, <1.4.3`, die behobene Version, ein Link und eine Zusammenfassung in einem Satz). Dieselbe tägliche Prüfung, die nach einer neuen Version sucht, ruft sie ebenfalls ab, vom selben Ort wie das Release: die Release-Seite des Repositorys bei GitHub, der Ordner der `latest.json` bei einem Spiegel, GitHub beim öffentlichen Chocolatey-Feed. Ein Hinweis wird nur verwendet, wenn `security.json.minisig` gegen den Schlüssel des Projekts geprüft werden kann; eine unsignierte oder falsch signierte Datei wird nie angezeigt (eine Zeile im Aktivitätsprotokoll sagt, dass sie ignoriert wurde).

Nennt ein Hinweis die laufende Version, wird der Chip rot und lautet **Sicherheitshinweis: Diese Version ist betroffen**, bei jeder Installation, auch bei paketverwalteten. Das Update-Fenster listet dann jeden Hinweis mit Schweregrad, Zusammenfassung, behobener Version und Link auf; **Hinweis öffnen** (`a`, oder **Updates: Sicherheitshinweis öffnen** in der Palette) öffnet den Link des schwerwiegendsten im Browser. Wo ein Paketmanager die Installation besitzt, ergänzt das Fenster „Mit Ihrem Paketmanager aktualisieren.“

Bei ausgeschalteter Updateprüfung (`updates.mode = "off"`), bei einer Administrator-Richtlinie, deren `check` den Wert `none` hat, und bei einem mit `cargo build` erstellten Build wird nichts abgerufen und kein Hinweis angezeigt. Der eigene NuGet-Feed einer Organisation lässt GitPoulpi nicht GitHub kontaktieren: Seine Hinweise kommen nur vom `download_base`-Spiegel der Richtlinie.

## Vom Administrator gesperrte Einstellungen

Ein Administrator kann in einer computerweiten Richtlinie den Update-Modus, den Kanal, eine festgelegte Version, den Ort der Prüfung und die Frage festlegen, ob GitPoulpi sich überhaupt selbst aktualisieren darf. In den Einstellungen zeigen diese Zeilen **Vom Administrator festgelegt**, das Fenster zeigt seine Nachricht, und wenn die Selbstaktualisierung abgeschaltet ist, bietet das Fenster Befehle oder einen Hinweis statt Schaltflächen, die Dateien ersetzen. Nicht lesbare Richtliniendateien schlagen geschlossen fehl: nur Hinweisen, keine Selbstaktualisierung. Die Seite des Administrators steht in [Unternehmensverteilung](enterprise-distribution.md#die-richtliniendatei-und-die-registry).

## Umgebung

`GITPOULPI_UPDATE_API` richtet die Prüfung auf eine andere API-Wurzel (von den Tests verwendet).
